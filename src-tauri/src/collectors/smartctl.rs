//! Invocación de `smartctl -j`, con su cascada de modos de acceso y un tiempo máximo por intento
//! (US-010, `docs/open-questions.md` I.5: riesgo R2, sin medir contra hardware real todavía).
//!
//! Lo que se puede probar sin hardware —construcción de la línea de órdenes y qué resultado de la
//! cascada se acepta— está en pruebas unitarias. La invocación real no: necesita el binario y un
//! dispositivo, y **medir el tiempo de la cascada, no solo si acierta**, es justo lo que el riesgo
//! R2 deja pendiente hasta poder probar con hardware variado.

use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::time::Duration;

use crate::platform::proceso_externo::ejecutar_con_limite;

/// Orden de intento cuando la autodetección de `smartctl` no basta (`open-questions.md` I.5).
/// **Sin medir**: es el orden que la documentación de smartmontools sugiere, no uno verificado
/// contra los puentes USB y controladoras RAID reales que decidirán el orden final.
pub const CASCADA_MODOS: &[&str] = &["sat", "nvme", "sntjmicron", "csmi"];

pub const TIEMPO_MAXIMO_POR_INTENTO: Duration = Duration::from_secs(15);

/// Ruta al `smartctl.exe` redistribuido. En desarrollo, junto al repositorio; en producción, junto
/// al ejecutable instalado, en la carpeta de recursos que declara `tauri.conf.json`.
///
/// **Sin verificar contra una compilación empaquetada real** (`docs/testing-strategy.md`, puerta
/// "por cada versión publicada"): el bundler de Tauri podría colocar los recursos en una ruta
/// distinta a la asumida aquí, y eso solo se confirma abriendo el instalador de verdad.
pub fn resolve_smartctl_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../third-party/smartmontools/bin/smartctl.exe")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join("bin/smartctl.exe")))
            .unwrap_or_else(|| PathBuf::from("bin/smartctl.exe"))
    }
}

/// Construye los argumentos de una consulta. Función pura para poder probarla sin invocar nada.
pub fn build_args(device_path: &str, modo: Option<&str>) -> Vec<String> {
    let mut args = vec!["-a".to_string(), "-j".to_string()];
    if let Some(m) = modo {
        args.push("-d".to_string());
        args.push(m.to_string());
    }
    args.push(device_path.to_string());
    args
}

/// Bits de `exit_status` que significan "no se pudo hablar con el dispositivo con este modo": ni
/// sintaxis (bit 0) ni apertura (bit 1). El resto de bits son hallazgos SMART legítimos —el
/// dispositivo respondió— y no deben hacer que la cascada siga probando otro modo.
fn intento_utilizable(status: ExitStatus) -> bool {
    let bits = status.code().unwrap_or(-1);
    if bits < 0 {
        return false;
    }
    (bits & 0b11) == 0
}

#[derive(Debug)]
pub enum ErrorConsulta {
    Io(std::io::Error),
    /// Ningún modo de la cascada logró hablar con el dispositivo.
    NingunModoFunciono,
}

impl From<std::io::Error> for ErrorConsulta {
    fn from(e: std::io::Error) -> Self {
        ErrorConsulta::Io(e)
    }
}

/// Prueba la autodetección y, si no basta, la cascada de `CASCADA_MODOS` en orden, devolviendo el
/// JSON del primer intento utilizable. Ninguna llamada aquí se prueba en unidad: necesita el
/// binario real y un dispositivo (igual que `windows_storage::list_physical_disks`).
pub fn query_device_json(device_path: &str) -> Result<String, ErrorConsulta> {
    let ejecutable = resolve_smartctl_path();

    for modo in std::iter::once(None).chain(CASCADA_MODOS.iter().map(|m| Some(*m))) {
        let mut cmd = Command::new(&ejecutable);
        cmd.args(build_args(device_path, modo));

        if let Some(salida) = ejecutar_con_limite(cmd, TIEMPO_MAXIMO_POR_INTENTO)? {
            if intento_utilizable(salida.status) {
                return Ok(String::from_utf8_lossy(&salida.stdout).into_owned());
            }
        }
    }

    Err(ErrorConsulta::NingunModoFunciono)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construye_los_argumentos_sin_modo() {
        assert_eq!(
            build_args(r"\\.\PhysicalDrive0", None),
            vec!["-a", "-j", r"\\.\PhysicalDrive0"]
        );
    }

    #[test]
    fn construye_los_argumentos_con_modo() {
        assert_eq!(
            build_args(r"\\.\PhysicalDrive0", Some("sat")),
            vec!["-a", "-j", "-d", "sat", r"\\.\PhysicalDrive0"]
        );
    }

    fn estado_con_codigo(codigo: i32) -> ExitStatus {
        // No hay constructor público de ExitStatus con código arbitrario en std estable:
        // se obtiene lanzando un proceso real y forzando su código de salida.
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(["/C", "exit", &codigo.to_string()])
                .status()
                .unwrap()
        }
        #[cfg(not(windows))]
        {
            std::process::Command::new("sh")
                .args(["-c", &format!("exit {codigo}")])
                .status()
                .unwrap()
        }
    }

    #[test]
    fn exit_status_cero_es_utilizable() {
        assert!(intento_utilizable(estado_con_codigo(0)));
    }

    #[test]
    fn exit_status_con_hallazgos_smart_sigue_siendo_utilizable() {
        // Bit 2 (fallo de salud pasado) y superiores son hallazgos, no fallos de acceso.
        assert!(intento_utilizable(estado_con_codigo(0b0100)));
    }

    #[test]
    fn exit_status_de_apertura_fallida_no_es_utilizable() {
        assert!(!intento_utilizable(estado_con_codigo(0b0010)));
        assert!(!intento_utilizable(estado_con_codigo(0b0011)));
    }

    #[test]
    fn la_cascada_prueba_sat_antes_que_nvme() {
        assert_eq!(CASCADA_MODOS[0], "sat");
        assert_eq!(CASCADA_MODOS[1], "nvme");
    }
}
