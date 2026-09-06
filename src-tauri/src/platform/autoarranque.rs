//! Autoarranque de la aplicación con el sistema (US-002, paso 3 del asistente; ADR-038).
//!
//! **No se usa la clave `Run` de HKCU.** La aplicación corre bajo `requireAdministrator`: una
//! entrada `Run` la lanzaría con el token sin elevar y Windows pediría UAC en cada inicio de
//! sesión (o fallaría en silencio). El mecanismo correcto es una **tarea programada** con
//! disparador «al iniciar sesión» y «ejecutar con los privilegios más altos», que el Programador
//! de tareas eleva sin diálogo.
//!
//! Se invoca `schtasks.exe` (del sistema, sin dependencia nueva) y **solo se mira el código de
//! salida**: la codificación de su salida de texto no es fiable entre configuraciones de Windows
//! (`.claude/rules/backend-rust.md`), así que no se parsea `stdout`.

use std::path::Path;

use crate::error::{AppError, AppResult};

/// Nombre de la tarea en el Programador. Estable: `aplicar(true)` la sustituye (`/F`), `aplicar(false)`
/// la borra.
const TAREA: &str = "SmartDisk Monitor - Autostart";

/// Línea de comando que ejecutará la tarea: la ruta del ejecutable entre comillas (puede tener
/// espacios). Aislado en su propia función para poder fijar el formato en una prueba sin lanzar
/// `schtasks`.
fn linea_de_comando(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

#[cfg(windows)]
pub fn aplicar(activar: bool) -> AppResult<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    /// `CREATE_NO_WINDOW`: sin esto, lanzar `schtasks` desde una app sin consola hace parpadear una.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let salida = if activar {
        let exe = std::env::current_exe().map_err(|e| {
            Box::new(
                AppError::new("autostart.failed", "error.autostartFailed")
                    .with_detail(format!("no se pudo resolver la ruta del ejecutable: {e}")),
            )
        })?;
        Command::new("schtasks")
            .args([
                "/Create",
                "/F",
                "/TN",
                TAREA,
                "/TR",
                &linea_de_comando(&exe),
                "/SC",
                "ONLOGON",
                "/RL",
                "HIGHEST",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    } else {
        Command::new("schtasks")
            .args(["/Delete", "/F", "/TN", TAREA])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    }
    .map_err(|e| {
        Box::new(
            AppError::new("autostart.failed", "error.autostartFailed")
                .with_detail(format!("no se pudo ejecutar schtasks: {e}")),
        )
    })?;

    // Borrar una tarea que no existe devuelve un código distinto de cero y es correcto: el estado
    // final buscado (sin tarea) ya se cumple. Crear que falle sí es un error real.
    if activar && !salida.status.success() {
        return Err(Box::new(
            AppError::new("autostart.failed", "error.autostartFailed").with_detail(format!(
                "schtasks /Create salió con {:?}",
                salida.status.code()
            )),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn aplicar(_activar: bool) -> AppResult<()> {
    // El autoarranque solo tiene sentido en Windows; en otras plataformas es una operación vacía.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn la_ruta_del_ejecutable_va_entre_comillas() {
        let ruta = PathBuf::from(r"C:\Program Files\SmartDisk Monitor\smartdisk.exe");
        assert_eq!(
            linea_de_comando(&ruta),
            r#""C:\Program Files\SmartDisk Monitor\smartdisk.exe""#
        );
    }

    #[test]
    fn el_nombre_de_la_tarea_es_estable() {
        // Si cambia, una instalación previa dejaría una tarea huérfana con el nombre viejo.
        assert_eq!(TAREA, "SmartDisk Monitor - Autostart");
    }
}
