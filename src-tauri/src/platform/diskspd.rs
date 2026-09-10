//! Ruta al `diskspd.exe` redistribuido (ADR-053), el motor de la prueba de Rendimiento.
//!
//! Mismo patrón que `collectors::smartctl::resolve_smartctl_path`: en desarrollo el binario vive
//! junto al repositorio, en producción junto al ejecutable instalado, en la carpeta de recursos
//! que declara `tauri.conf.json`.

use std::path::PathBuf;

/// Ruta al `diskspd.exe` redistribuido.
///
/// **Sin verificar contra una compilación empaquetada real** (`docs/testing-strategy.md`, puerta
/// "por cada versión publicada"): el bundler de Tauri podría colocar los recursos en una ruta
/// distinta a la asumida aquí, y eso solo se confirma abriendo el instalador de verdad.
pub fn resolve_diskspd_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../third-party/diskspd/bin/diskspd.exe")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join("bin/diskspd.exe")))
            .unwrap_or_else(|| PathBuf::from("bin/diskspd.exe"))
    }
}
