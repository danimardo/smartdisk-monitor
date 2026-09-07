//! Invocación de `smartctl -j`, con su cascada de modos de acceso y un tiempo máximo por intento
//! (US-010, `docs/open-questions.md` I.5: riesgo R2, sin medir contra hardware real todavía).
//!
//! Lo que se puede probar sin hardware —construcción de la línea de órdenes y qué resultado de la
//! cascada se acepta— está en pruebas unitarias. La invocación real no: necesita el binario y un
//! dispositivo, y **medir el tiempo de la cascada, no solo si acierta**, es justo lo que el riesgo
//! R2 deja pendiente hasta poder probar con hardware variado.

use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

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

/// Ejecuta con un límite de tiempo por sondeo, sin depender de un crate de temporización externo:
/// `Child` no tiene una espera con plazo en la biblioteca estándar.
///
/// Los pipes de `stdout`/`stderr` tienen un búfer acotado por el sistema operativo. Si nadie los
/// vacía mientras el hijo sigue escribiendo, el hijo se bloquea en su propio `write()` en cuanto lo
/// llena, y nunca llega a salir — un punto muerto de facto entre el hijo y este bucle, que antes
/// leía los pipes **después** de que `try_wait()` confirmara la salida. Verificado contra hardware
/// real (`docs/open-questions.md` J.47): la salida de `smartctl -a -j` en un SATA con la tabla de
/// atributos completa (10-13 KB en los dos discos de esta máquina) supera ese búfer con facilidad,
/// y el proceso se colgaba los 15 s completos en cada uno de los cinco modos de la cascada —
/// `NingunModoFunciono` sin decir por qué. La de un NVMe (7 KB en esta máquina) se quedaba por
/// debajo del umbral y nunca lo mostraba, lo que hizo parecer un problema específico de SATA hasta
/// medirlo. Los hilos lectores vacían los pipes según llegan, sin esperar a que el proceso termine.
fn ejecutar_con_limite(
    mut hijo: std::process::Child,
    limite: Duration,
) -> std::io::Result<Option<std::process::Output>> {
    use std::io::Read;

    let mut stdout = hijo.stdout.take();
    let mut stderr = hijo.stderr.take();
    let lector_stdout = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(s) = stdout.as_mut() {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });
    let lector_stderr = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(s) = stderr.as_mut() {
            let _ = s.read_to_end(&mut buf);
        }
        buf
    });

    let inicio = Instant::now();
    loop {
        if let Some(estado) = hijo.try_wait()? {
            let salida = lector_stdout.join().unwrap_or_default();
            let error = lector_stderr.join().unwrap_or_default();
            return Ok(Some(std::process::Output {
                status: estado,
                stdout: salida,
                stderr: error,
            }));
        }
        if inicio.elapsed() >= limite {
            let _ = hijo.kill();
            let _ = hijo.wait();
            let _ = lector_stdout.join();
            let _ = lector_stderr.join();
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Prueba la autodetección y, si no basta, la cascada de `CASCADA_MODOS` en orden, devolviendo el
/// JSON del primer intento utilizable. Ninguna llamada aquí se prueba en unidad: necesita el
/// binario real y un dispositivo (igual que `windows_storage::list_physical_disks`).
pub fn query_device_json(device_path: &str) -> Result<String, ErrorConsulta> {
    let ejecutable = resolve_smartctl_path();

    for modo in std::iter::once(None).chain(CASCADA_MODOS.iter().map(|m| Some(*m))) {
        let hijo = Command::new(&ejecutable)
            .args(build_args(device_path, modo))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(salida) = ejecutar_con_limite(hijo, TIEMPO_MAXIMO_POR_INTENTO)? {
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

    #[test]
    #[cfg(windows)]
    fn ejecutar_con_limite_no_se_bloquea_con_una_salida_mas_grande_que_el_pipe() {
        // Regresión (J.47): antes se leían los pipes solo después de que `try_wait()` confirmara
        // la salida del proceso. Con una salida mayor que el búfer del pipe, el hijo se bloqueaba
        // escribiendo y nunca llegaba a salir — se agotaba el límite entero, indistinguible de un
        // dispositivo que de verdad no responde. No hace falta smartctl real para reproducirlo:
        // cualquier proceso que escriba lo bastante lo dispara.
        let hijo = std::process::Command::new("cmd")
            .args([
                "/C",
                "for /L %i in (1,1,4000) do @echo 0123456789012345678901234567890123456789",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let resultado = ejecutar_con_limite(hijo, Duration::from_secs(10)).unwrap();
        let salida = resultado.expect("no debería agotar el límite de tiempo");
        assert!(salida.status.success());
        assert!(
            salida.stdout.len() > 64 * 1024,
            "la salida capturada debería superar el búfer típico de un pipe, midió {} bytes",
            salida.stdout.len()
        );
    }
}
