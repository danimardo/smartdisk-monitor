//! Ejecución de `chkdsk /scan` capturando su salida literal (T080,
//! `docs/product-specification.md` §6: "solo en volúmenes NTFS", "no se ofrecen inicialmente
//! opciones de reparación fuera de línea").
//!
//! Los bytes crudos se conservan sin decodificar: `tests::codificacion::detectar` decide después
//! qué página de códigos usar, porque `chkdsk` no coincide con `fsutil`/`vssadmin` en la misma
//! máquina (`docs/open-questions.md` §Q). Ninguna llamada de este módulo se prueba en unidad —
//! necesita el binario real de Windows, igual que `collectors::smartctl::query_device_json`.

use std::process::{Command, Stdio};
use std::time::Duration;

/// `chkdsk /scan` solo existe para NTFS: en ReFS, exFAT y FAT32 la acción debe aparecer
/// deshabilitada, no fallar al intentarlo (`product-specification.md` §6).
pub fn admite_scan(filesystem: Option<&str>) -> bool {
    filesystem
        .map(|f| f.eq_ignore_ascii_case("NTFS"))
        .unwrap_or(false)
}

#[derive(Debug)]
pub struct SalidaChkdsk {
    pub exit_status: i32,
    pub stdout_crudo: Vec<u8>,
    pub stderr_crudo: Vec<u8>,
}

#[derive(Debug)]
pub enum ErrorChkdsk {
    Io(std::io::Error),
    Cancelado,
}

impl From<std::io::Error> for ErrorChkdsk {
    fn from(e: std::io::Error) -> Self {
        ErrorChkdsk::Io(e)
    }
}

/// Comando literal que se le mostrará al usuario antes de confirmar (`ui-design.md` §5): lo que
/// de verdad se ejecuta, no una aproximación.
pub fn comando_literal(drive_letter: char) -> String {
    format!("chkdsk {drive_letter}: /scan")
}

/// Ejecuta `chkdsk /scan` sobre `drive_letter`, sondeando cada 100 ms para poder cancelar entre
/// medias (mismo patrón que `collectors::smartctl::ejecutar_con_limite`, sin límite de tiempo
/// máximo: un `/scan` real puede tardar minutos en un volumen grande y **no** es señal de fallo).
pub fn ejecutar_scan(
    drive_letter: char,
    mut cancelado: impl FnMut() -> bool,
) -> Result<SalidaChkdsk, ErrorChkdsk> {
    let volumen = format!("{drive_letter}:");
    let mut hijo = Command::new("chkdsk.exe")
        .args(["/scan", &volumen])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    loop {
        if cancelado() {
            let _ = hijo.kill();
            let _ = hijo.wait();
            return Err(ErrorChkdsk::Cancelado);
        }
        if let Some(_estado) = hijo.try_wait()? {
            let salida = hijo.wait_with_output()?;
            return Ok(SalidaChkdsk {
                exit_status: salida.status.code().unwrap_or(-1),
                stdout_crudo: salida.stdout,
                stderr_crudo: salida.stderr,
            });
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntfs_admite_scan() {
        assert!(admite_scan(Some("NTFS")));
        assert!(admite_scan(Some("ntfs")));
    }

    #[test]
    fn refs_exfat_y_fat32_no_admiten_scan() {
        assert!(!admite_scan(Some("ReFS")));
        assert!(!admite_scan(Some("exFAT")));
        assert!(!admite_scan(Some("FAT32")));
    }

    #[test]
    fn un_sistema_de_archivos_desconocido_no_admite_scan() {
        assert!(!admite_scan(None));
    }

    #[test]
    fn el_comando_literal_incluye_la_letra_y_el_modificador() {
        assert_eq!(comando_literal('C'), "chkdsk C: /scan");
    }
}
