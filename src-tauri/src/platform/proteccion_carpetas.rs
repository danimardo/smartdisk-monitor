//! Control de acceso a carpetas de Windows Defender y la excepción que necesita `smartctl.exe`
//! (`docs/open-questions.md` J.56, ADR-043): sin ella, cualquier lectura SMART de un disco SATA
//! que use ATA PASS THROUGH se bloquea en silencio — Defender lo clasifica como una escritura de
//! bajo nivel en el disco, aunque `smartctl` solo lea. El Explorador de Windows nunca lo dispara
//! (usa E/S de archivos, no el mismo camino), y por eso un disco puede verse perfectamente desde
//! fuera y a la vez aparecer sin datos SMART dentro de la aplicación.
//!
//! No hay una API de Win32 pública para esto: la única vía documentada por Microsoft son los
//! cmdlets `Get-MpPreference`/`Add-MpPreference` de PowerShell (respaldados por la clase WMI
//! `MSFT_MpPreference`, sin cliente WMI en la pila fija del proyecto). Se invocan como procesos
//! externos, mismo patrón ya establecido para `smartctl.exe`/`chkdsk` — nada aquí se prueba en
//! unidad más allá de la comparación de rutas, que sí es pura: la llamada real necesita Windows
//! y Defender reales, igual que `platform::energia`.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use super::proceso_externo::ejecutar_con_limite;

/// Mismo criterio que el resto de consultas externas de este backend (J.55/J.57): nunca esperar
/// para siempre a PowerShell, y nunca dejar que su ventana de consola parpadee.
const TIEMPO_MAXIMO: Duration = Duration::from_secs(15);

/// `true` si `ruta` aparece, línea a línea, entre las aplicaciones permitidas. Comparación
/// insensible a mayúsculas: PowerShell y NSIS pueden diferir en el uso de mayúsculas de la unidad.
fn contiene_ruta(lista: &str, ruta: &str) -> bool {
    let ruta = ruta.trim();
    lista
        .lines()
        .any(|linea| linea.trim().eq_ignore_ascii_case(ruta))
}

/// Comprueba si `ruta` ya está en la lista de aplicaciones permitidas de Control de acceso a
/// carpetas. `None` si no se pudo determinar (cmdlet no disponible, PowerShell falló o se agotó el
/// tiempo, salida irreconocible) — un dato ausente, nunca un "no" engañoso.
#[cfg(windows)]
pub fn esta_permitido(ruta: &Path) -> Option<bool> {
    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "(Get-MpPreference).ControlledFolderAccessAllowedApplications -join \"`n\"",
    ]);
    let salida = ejecutar_con_limite(cmd, TIEMPO_MAXIMO).ok()??;
    if !salida.status.success() {
        return None;
    }
    let texto = String::from_utf8_lossy(&salida.stdout);
    Some(contiene_ruta(&texto, &ruta.to_string_lossy()))
}

#[cfg(not(windows))]
pub fn esta_permitido(_ruta: &Path) -> Option<bool> {
    None
}

/// Intenta añadir `ruta` a la lista de aplicaciones permitidas y confirma releyéndola: la
/// Protección contra alteraciones puede dejar que el cmdlet "tenga éxito" sin que el cambio llegue
/// a aplicarse de verdad, así que un proceso que termina con éxito no basta por sí solo.
#[cfg(windows)]
pub fn intentar_permitir(ruta: &Path) -> Result<(), String> {
    let ruta_str = ruta.to_string_lossy();
    let comando = format!(
        "Add-MpPreference -ControlledFolderAccessAllowedApplications '{}'",
        ruta_str.replace('\'', "''")
    );
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &comando]);
    let salida = ejecutar_con_limite(cmd, TIEMPO_MAXIMO)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Windows Defender no respondió a tiempo".to_string())?;
    if !salida.status.success() {
        let stderr = String::from_utf8_lossy(&salida.stderr).into_owned();
        return Err(if stderr.trim().is_empty() {
            "smartctl.exe no está en la lista de aplicaciones permitidas de Windows Defender \
             (código de salida distinto de cero, PowerShell no explicó por qué)"
                .to_string()
        } else {
            stderr
        });
    }
    match esta_permitido(ruta) {
        Some(true) => Ok(()),
        Some(false) => Err(
            "Windows Defender no aplicó el cambio, probablemente por la Protección contra \
             alteraciones (bloquea incluso cambios de un proceso con privilegios de administrador)"
                .to_string(),
        ),
        None => Err("no se pudo confirmar si el cambio se aplicó".to_string()),
    }
}

#[cfg(not(windows))]
pub fn intentar_permitir(_ruta: &Path) -> Result<(), String> {
    Err("Control de acceso a carpetas es una protección exclusiva de Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn una_ruta_presente_se_reconoce_sin_importar_mayusculas() {
        let lista = "C:\\Windows\\System32\\notepad.exe\nF:\\Apps\\smartdisk\\bin\\smartctl.exe\n";
        assert!(contiene_ruta(lista, r"f:\apps\smartdisk\bin\smartctl.exe"));
    }

    #[test]
    fn una_ruta_ausente_no_se_reconoce() {
        let lista = "C:\\Windows\\System32\\notepad.exe\n";
        assert!(!contiene_ruta(lista, r"F:\Apps\smartdisk\bin\smartctl.exe"));
    }

    #[test]
    fn una_lista_vacia_no_contiene_nada() {
        assert!(!contiene_ruta("", r"F:\Apps\smartdisk\bin\smartctl.exe"));
    }
}
