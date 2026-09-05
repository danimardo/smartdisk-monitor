//! Presentación de la ventana principal.
//!
//! Vive aquí y no en `lib.rs` porque lo necesitan dos caminos distintos: el arranque normal y la
//! devolución de llamada de instancia única (ADR-025), que corre en el primer proceso cuando
//! alguien lanza un segundo.

use tauri::{Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

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

/// Qué hacer al pulsar la X (US-072, T099, `docs/open-questions.md` J.19): minimizar a la bandeja
/// o salir de verdad. La decisión de **cuál** de los dos toca (leer `lifecycle.close_action` de
/// `settings`) vive en `lib.rs`, que es quien tiene la conexión; aquí solo la mecánica de cada
/// caso, igual que `restaurar_ventana_principal` solo sabe mostrar, no decidir cuándo.
pub fn gestionar_cierre<R: Runtime>(
    app: &tauri::AppHandle<R>,
    ventana: &tauri::WebviewWindow<R>,
    salir: bool,
) {
    if salir {
        app.exit(0);
        return;
    }
    if let Err(e) = ventana.hide() {
        tracing::warn!(error = %e, "no se pudo minimizar la ventana a la bandeja");
    }
    avisar_primera_minimizacion(app);
}

/// Aviso nativo, una sola vez por arranque del proceso (`open-questions.md` J.44): sin él, la
/// primera vez que se minimiza a la bandeja parece que la aplicación se cerró sola, en vez de
/// seguir vigilando en segundo plano.
fn avisar_primera_minimizacion<R: Runtime>(app: &tauri::AppHandle<R>) {
    let estado = app.state::<crate::persistence::db::AppState>();
    // `swap` devuelve el valor anterior: si ya era `true`, esta llamada no hace nada más.
    if estado
        .aviso_bandeja_mostrado
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }
    let locale = crate::platform::rotulos::locale_actual();
    if let Err(e) = app
        .notification()
        .builder()
        .title(crate::platform::rotulos::t(locale, "tray.minimizedTitle"))
        .body(crate::platform::rotulos::t(locale, "tray.minimizedBody"))
        .show()
    {
        tracing::warn!(error = %e, "no se pudo mostrar el aviso de minimizado a la bandeja");
    }
}
