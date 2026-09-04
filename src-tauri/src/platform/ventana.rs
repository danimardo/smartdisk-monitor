//! Presentación de la ventana principal.
//!
//! Vive aquí y no en `lib.rs` porque lo necesitan dos caminos distintos: el arranque normal y la
//! devolución de llamada de instancia única (ADR-025), que corre en el primer proceso cuando
//! alguien lanza un segundo.

use tauri::{Manager, Runtime};

/// Identificador de la ventana principal en `tauri.conf.json`. En un sitio y no repetido: un
/// literal mal escrito no falla al compilar, solo deja de encontrar la ventana en tiempo de
/// ejecución.
pub const VENTANA_PRINCIPAL: &str = "main";

/// Trae la ventana principal al frente.
///
/// El orden importa y no es evidente: una ventana minimizada **sigue siendo visible** para
/// `is_visible()`, así que hay que desminimizar antes de mostrar; y `set_focus()` sobre una
/// ventana oculta no hace nada. Windows además ignora la petición de foco si la aplicación no
/// tiene entrada reciente, de ahí que se llame después de mostrarla.
///
/// No devuelve error: es una cortesía de interfaz. Si falla, se registra y la aplicación sigue.
pub fn restaurar_ventana_principal<R: Runtime>(app: &tauri::AppHandle<R>) {
    let Some(ventana) = app.get_webview_window(VENTANA_PRINCIPAL) else {
        tracing::warn!(
            ventana = VENTANA_PRINCIPAL,
            "no se encontró la ventana principal al intentar restaurarla"
        );
        return;
    };

    if let Ok(true) = ventana.is_minimized() {
        if let Err(e) = ventana.unminimize() {
            tracing::warn!(error = %e, "no se pudo desminimizar la ventana principal");
        }
    }
    if let Err(e) = ventana.show() {
        tracing::warn!(error = %e, "no se pudo mostrar la ventana principal");
    }
    if let Err(e) = ventana.set_focus() {
        tracing::warn!(error = %e, "no se pudo enfocar la ventana principal");
    }
}
