//! Lectura del color de acento de Windows.
//!
//! **El registro guarda el color en ABGR, no en RGB.** Leerlo como RGB devuelve el color
//! invertido: el azul de fábrica (`0xFFD47800`) saldría naranja `#D47800` en vez de azul
//! `#0078D4`. Medido sobre un equipo real (`open-questions.md` §O.7).
//!
//! Se devuelve además `AccentPalette`, los siete tonos que Windows deriva del acento y usa para su
//! propia interfaz. La UI los prefiere a cualquier derivación propia para el acento **como texto**:
//! son los tonos que el usuario ya está viendo en el resto del sistema.

use serde::Serialize;
use ts_rs::TS;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct WindowsAccent {
    /// `#RRGGBB` ya convertido desde el ABGR del registro.
    pub hex: String,
    /// Tonos de `AccentPalette`, del más claro al más oscuro. Vacío si no se pudo leer.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub palette: Vec<String>,
}

const ACCENT_KEY: &str = r"Software\Microsoft\Windows\DWM";
const PALETTE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent";

/// Convierte el `0x00BBGGRR` del registro a `#RRGGBB`.
fn abgr_to_hex(abgr: u32) -> String {
    let r = abgr & 0xFF;
    let g = (abgr >> 8) & 0xFF;
    let b = (abgr >> 16) & 0xFF;
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[cfg(windows)]
pub fn read() -> AppResult<WindowsAccent> {
    use windows_registry::CURRENT_USER;

    let dwm = CURRENT_USER.open(ACCENT_KEY).map_err(|e| {
        Box::new(
            AppError::new("accent.unavailable", "error.accentUnavailable")
                .with_detail(e.to_string()),
        )
    })?;

    let raw: u32 = dwm.get_u32("AccentColor").map_err(|e| {
        Box::new(
            AppError::new("accent.unavailable", "error.accentUnavailable")
                .with_detail(e.to_string()),
        )
    })?;

    // La paleta es opcional: si falta, la UI deriva sus propios tonos y sigue cumpliendo AA.
    let palette = CURRENT_USER
        .open(PALETTE_KEY)
        .ok()
        .and_then(|k| k.get_value("AccentPalette").ok())
        .map(|v| parse_palette(v.as_ref()))
        .unwrap_or_default();

    Ok(WindowsAccent {
        hex: abgr_to_hex(raw),
        palette,
    })
}

#[cfg(not(windows))]
pub fn read() -> AppResult<WindowsAccent> {
    Err(Box::new(
        AppError::new("accent.unavailable", "error.accentUnavailable")
            .with_detail("el acento del sistema solo existe en Windows"),
    ))
}

/// `AccentPalette` son 8 entradas de 4 bytes (RGBA). Solo interesan las siete primeras, que son la
/// rampa de tonos; la octava es un color secundario sin relación con ella.
fn parse_palette(bytes: &[u8]) -> Vec<String> {
    bytes
        .chunks_exact(4)
        .take(7)
        .map(|c| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_registro_guarda_abgr_no_rgb() {
        // Valor real leído de un equipo con el acento azul de fábrica.
        assert_eq!(abgr_to_hex(0x00D47800), "#0078d4");
        // Si se interpretara como RGB saldría "#d47800", que es naranja.
        assert_ne!(abgr_to_hex(0x00D47800), "#d47800");
    }

    #[test]
    fn la_paleta_toma_solo_los_siete_tonos() {
        let bytes: Vec<u8> = (0..32).collect();
        assert_eq!(parse_palette(&bytes).len(), 7);
    }

    #[test]
    fn una_paleta_ausente_no_es_un_error() {
        assert!(parse_palette(&[]).is_empty());
    }
}
