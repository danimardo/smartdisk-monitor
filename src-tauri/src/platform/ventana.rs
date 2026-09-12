//! Presentación de la ventana principal.
//!
//! Vive aquí y no en `lib.rs` porque lo necesitan dos caminos distintos: el arranque normal y la
//! devolución de llamada de instancia única (ADR-025), que corre en el primer proceso cuando
//! alguien lanza un segundo.

use tauri::{LogicalPosition, LogicalSize, Manager, Runtime, WebviewWindow};
use tauri_plugin_notification::NotificationExt;

use crate::commands::{ahora_rfc3339, guardar_ajuste, leer_ajuste_bool, leer_ajuste_i64};

/// Identificador de la ventana principal en `tauri.conf.json`. En un sitio y no repetido: un
/// literal mal escrito no falla al compilar, solo deja de encontrar la ventana en tiempo de
/// ejecución.
pub const VENTANA_PRINCIPAL: &str = "main";

/// Mínimo técnico de `tauri.conf.json` (`docs/ui-design.md` §4.0). Aquí solo para sanear una
/// geometría guardada disparatada: nunca se restaura una ventana más pequeña que esto.
///
/// `MIN_H` incluye la barra de título propia (spec 011): 560 de contenido + 35 de la barra
/// (`--sdm-control-lg` en `tokens.css`). Este número tiene que coincidir siempre con `minHeight` de
/// `tauri.conf.json` — ninguno de los dos lados puede leer el token CSS del otro.
const MIN_W: i64 = 1024;
const MIN_H: i64 = 595;

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

// ---- geometría de la ventana (ADR-040) -----------------------------------------------------------
//
// El tamaño, la posición y el «maximizada» de la última sesión viven en la tabla `settings`
// (claves `window.*`), no en un fichero aparte ni en `localStorage` (constitución §V). El frontend
// no participa: se aplica aquí al arrancar —antes de mostrar la ventana, que nace oculta, así que
// no hay salto— y se guarda al cerrar. Ausente == se usa lo de `tauri.conf.json` (primer arranque).

/// Rectángulo en píxeles **lógicos** (independientes del escalado de Windows), que es como se
/// guardan las claves `window.*` y como razona `geometria_visible`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RectLog {
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

/// ¿Se puede alcanzar la barra de título de esta ventana en algún monitor? Pura y con pruebas: es
/// la única lógica de verdad de todo esto (el resto es llamar a la API de Tauri). `true` si en
/// algún monitor hay al menos `MARGEN` px de solape horizontal **y** la banda de la barra de
/// título cae dentro del alto visible. Evita restaurar una ventana en un monitor que ya no está.
pub(crate) fn geometria_visible(v: RectLog, monitores: &[RectLog]) -> bool {
    const MARGEN: i64 = 80;
    monitores.iter().any(|m| {
        let solape_x = (v.x + v.w).min(m.x + m.w) - v.x.max(m.x);
        let barra = v.y + 20;
        solape_x >= MARGEN && barra >= m.y && barra <= m.y + m.h - 20
    })
}

fn monitores_logicos<R: Runtime>(ventana: &WebviewWindow<R>) -> Vec<RectLog> {
    ventana
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let sf = m.scale_factor().max(0.1);
            let p = m.position();
            let s = m.size();
            RectLog {
                x: (f64::from(p.x) / sf).round() as i64,
                y: (f64::from(p.y) / sf).round() as i64,
                w: (f64::from(s.width) / sf).round() as i64,
                h: (f64::from(s.height) / sf).round() as i64,
            }
        })
        .collect()
}

/// Aplica la geometría guardada a la ventana principal **antes** de mostrarla. Sin claves
/// guardadas (primer arranque) no toca nada: manda `tauri.conf.json`. Cortesía de arranque, como
/// `restaurar_ventana_principal`: cada fallo se registra y se sigue.
pub(crate) fn aplicar_geometria_guardada<R: Runtime>(
    ventana: &WebviewWindow<R>,
    conn: &rusqlite::Connection,
) {
    let w = leer_ajuste_i64(conn, "window.width", 0);
    let h = leer_ajuste_i64(conn, "window.height", 0);
    if w <= 0 || h <= 0 {
        return; // nada guardado todavía
    }
    let w = w.max(MIN_W);
    let h = h.max(MIN_H);

    if let Err(e) = ventana.set_size(LogicalSize::new(w as f64, h as f64)) {
        tracing::warn!(error = %e, "no se pudo restaurar el tamaño de la ventana");
    }

    // `i64::MIN` como centinela: distinguir «no guardado» de una coordenada 0 legítima.
    let x = leer_ajuste_i64(conn, "window.x", i64::MIN);
    let y = leer_ajuste_i64(conn, "window.y", i64::MIN);
    if x != i64::MIN && y != i64::MIN {
        let rect = RectLog { x, y, w, h };
        if geometria_visible(rect, &monitores_logicos(ventana)) {
            if let Err(e) = ventana.set_position(LogicalPosition::new(x as f64, y as f64)) {
                tracing::warn!(error = %e, "no se pudo restaurar la posición de la ventana");
            }
        } else {
            tracing::info!(
                "la posición guardada de la ventana queda fuera de pantalla; se abre centrada"
            );
        }
    }

    if leer_ajuste_bool(conn, "window.maximized", false) {
        if let Err(e) = ventana.maximize() {
            tracing::warn!(error = %e, "no se pudo restaurar la ventana maximizada");
        }
    }
}

/// Guarda la geometría actual de la ventana en `settings`. Se llama al cerrar (pulsar la X) y al
/// salir de verdad (bandeja, apagado). Best-effort: si algo falla, se registra y no impide cerrar.
///
/// Si la ventana está **maximizada** se guarda solo `window.maximized = true` y **no** se tocan
/// tamaño ni posición: así, al restaurar y quitar la maximización, vuelve al tamaño que el usuario
/// había elegido antes de maximizar.
pub(crate) fn persistir_geometria<R: Runtime>(
    ventana: &WebviewWindow<R>,
    conn: &rusqlite::Connection,
) {
    let ahora = ahora_rfc3339();
    let maximizada = ventana.is_maximized().unwrap_or(false);

    let guardar = |clave: &str, valor: &i64| {
        if let Err(e) = guardar_ajuste(conn, clave, valor, &ahora) {
            tracing::warn!(error = ?e, clave, "no se pudo guardar la geometría de la ventana");
        }
    };

    if !maximizada {
        let sf = ventana.scale_factor().unwrap_or(1.0).max(0.1);
        if let Ok(size) = ventana.inner_size() {
            let w = (f64::from(size.width) / sf).round() as i64;
            let h = (f64::from(size.height) / sf).round() as i64;
            if w >= MIN_W && h >= MIN_H {
                guardar("window.width", &w);
                guardar("window.height", &h);
            }
        }
        if let Ok(pos) = ventana.outer_position() {
            let x = (f64::from(pos.x) / sf).round() as i64;
            let y = (f64::from(pos.y) / sf).round() as i64;
            guardar("window.x", &x);
            guardar("window.y", &y);
        }
    }

    if let Err(e) = guardar_ajuste(conn, "window.maximized", &maximizada, &ahora) {
        tracing::warn!(error = ?e, "no se pudo guardar el estado maximizado de la ventana");
    }
}

#[cfg(test)]
mod tests {
    use super::{geometria_visible, RectLog};

    const HD: RectLog = RectLog {
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
    };

    #[test]
    fn ventana_dentro_de_un_monitor_es_visible() {
        let v = RectLog {
            x: 100,
            y: 80,
            w: 1400,
            h: 900,
        };
        assert!(geometria_visible(v, &[HD]));
    }

    #[test]
    fn ventana_en_un_monitor_que_ya_no_esta_no_es_visible() {
        // Guardada en un segundo monitor a la izquierda que se ha desconectado.
        let v = RectLog {
            x: -1800,
            y: 100,
            w: 1400,
            h: 900,
        };
        assert!(!geometria_visible(v, &[HD]));
    }

    #[test]
    fn barra_de_titulo_por_encima_del_monitor_no_es_visible() {
        let v = RectLog {
            x: 200,
            y: -60,
            w: 1400,
            h: 900,
        };
        assert!(!geometria_visible(v, &[HD]));
    }

    #[test]
    fn asomando_poco_por_el_borde_derecho_no_cuenta() {
        // Solo ~40 px dentro del monitor: por debajo del margen de 80.
        let v = RectLog {
            x: 1880,
            y: 100,
            w: 1400,
            h: 900,
        };
        assert!(!geometria_visible(v, &[HD]));
    }

    #[test]
    fn segundo_monitor_a_la_izquierda_con_coordenadas_negativas() {
        let izquierdo = RectLog {
            x: -1920,
            y: 0,
            w: 1920,
            h: 1080,
        };
        let v = RectLog {
            x: -1800,
            y: 50,
            w: 1400,
            h: 900,
        };
        assert!(geometria_visible(v, &[izquierdo, HD]));
    }
}
