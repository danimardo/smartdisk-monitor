//! Icono de la bandeja del sistema: color agregado, menú y clic (FR-012,
//! `docs/product-specification.md` §3, `docs/open-questions.md` B.5).
//!
//! El color es el mismo `HealthState` de toda la interfaz (`domain::salud::tray_state`, espejo
//! exacto de `trayState()` en `src/lib/design/health.ts`); no hay un tipo ni una paleta nuevos. El
//! icono se genera en memoria a partir de los mismos `--sdm-*` que usa el resto de la aplicación,
//! sin fichero `.ico` versionado: `docs/decisions.md` (línea 102) señala systray como pantalla aún
//! sin revisión visual, y esto es un primer trazo funcional, no el diseño final.
//!
//! **Alcance de esta primera versión** (`docs/open-questions.md` J.19): el color se recalcula en
//! los puntos de sincronización existentes —arranque, `refresh_now`, comandos de alerta,
//! pausar/reanudar— y no en un ciclo real cada 30 s: el planificador en segundo plano (T020), la
//! emisión de eventos (T021) y la reanudación automática al arrancar (T022) siguen sin
//! implementar. El botón de cierre (`X`) minimiza siempre a la bandeja; la pregunta "minimizar o
//! salir" con opción de recordar (FR-012) queda pendiente de un diálogo propio.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{image::Image, AppHandle, Manager};

use crate::domain::salud::tray_state;
use crate::domain::tipos::HealthState;
use crate::persistence::db::AppState;
use crate::platform::rotulos::{locale_actual, t, tp};
use crate::platform::ventana;

const LADO: u32 = 32;

const ID_ABRIR: &str = "tray-abrir";
const ID_RESUMEN: &str = "tray-resumen";
const ID_PAUSAR: &str = "tray-pausar";
const ID_SALIR: &str = "tray-salir";

/// El `TrayIcon` ya construido, gestionado por Tauri para poder actualizarlo desde cualquier
/// comando sin reconstruirlo entero cada vez.
struct BandejaIcono(TrayIcon);

fn color_rgba(estado: HealthState) -> [u8; 4] {
    // Mismos valores que `--sdm-{ok,warn,crit,unknown}` en tema claro (`tokens.css`): el icono no
    // inventa un color que la interfaz no use ya.
    match estado {
        HealthState::Ok => [0x2f, 0x72, 0x56, 0xff],
        HealthState::Warn => [0x7d, 0x56, 0x19, 0xff],
        HealthState::Crit => [0xa8, 0x3d, 0x45, 0xff],
        HealthState::Unknown => [0x5d, 0x61, 0x6d, 0xff],
    }
}

/// Un cuadrado sólido del color del estado. Sin biblioteca de imágenes: es un búfer RGBA plano.
fn icono(estado: HealthState) -> Image<'static> {
    let [r, g, b, a] = color_rgba(estado);
    let mut buffer = Vec::with_capacity((LADO * LADO * 4) as usize);
    for _ in 0..(LADO * LADO) {
        buffer.extend_from_slice(&[r, g, b, a]);
    }
    Image::new_owned(buffer, LADO, LADO)
}

/// El texto de resumen: mismas claves y misma lógica que `globalLabel` en `+layout.svelte`, para
/// que el menú de la bandeja diga exactamente lo mismo que la barra de la ventana principal.
fn texto_resumen(locale: &str, pausado: bool, estados: &[HealthState]) -> String {
    if pausado {
        return t(locale, "global.paused");
    }
    if estados.is_empty() {
        return t(locale, "global.noDevices");
    }
    let necesitan_atencion = estados
        .iter()
        .filter(|e| matches!(e, HealthState::Warn | HealthState::Crit))
        .count();
    if necesitan_atencion == 0 {
        return t(locale, "global.allGood");
    }
    tp(locale, "global.needsAttention", necesitan_atencion as i64)
}

fn construir_menu(
    app: &AppHandle,
    pausado: bool,
    resumen: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    let locale = locale_actual();
    let abrir = MenuItem::with_id(app, ID_ABRIR, t(locale, "tray.open"), true, None::<&str>)?;
    let resumen_item = MenuItem::with_id(app, ID_RESUMEN, resumen, false, None::<&str>)?;
    let pausar = MenuItem::with_id(
        app,
        ID_PAUSAR,
        if pausado {
            t(locale, "nav.resume")
        } else {
            t(locale, "nav.pause")
        },
        true,
        None::<&str>,
    )?;
    let salir = MenuItem::with_id(app, ID_SALIR, t(locale, "tray.exit"), true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &abrir,
            &resumen_item,
            &PredefinedMenuItem::separator(app)?,
            &pausar,
            &PredefinedMenuItem::separator(app)?,
            &salir,
        ],
    )
}

/// Recalcula color, menú y texto emergente a partir del estado real y los aplica al icono ya
/// construido. Se llama tras cada acción que puede cambiar el color: arranque, `refresh_now`,
/// las seis acciones sobre alertas y pausar/reanudar (`docs/open-questions.md` J.19).
pub fn actualizar(app: &AppHandle) {
    let Some(bandeja) = app.try_state::<BandejaIcono>() else {
        return;
    };
    let estado = app.state::<AppState>();
    let conn = estado
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let respuesta = match crate::commands::get_devices_impl(&conn) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = ?e, "no se pudo recalcular el icono de la bandeja");
            return;
        }
    };
    drop(conn);
    let pausado = estado
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro")
        .is_some();

    let estados: Vec<HealthState> = respuesta.devices.iter().map(|d| d.state).collect();
    let fallo_recopilador = respuesta.sources.iter().any(|s| {
        matches!(
            s.status,
            crate::commands::SourceStatus::Timeout | crate::commands::SourceStatus::Error
        )
    });
    let color = tray_state(pausado, fallo_recopilador, &estados);
    let locale = locale_actual();
    let resumen = texto_resumen(locale, pausado, &estados);

    if let Err(e) = bandeja.0.set_icon(Some(icono(color))) {
        tracing::warn!(error = %e, "no se pudo actualizar el icono de la bandeja");
    }
    if let Err(e) = bandeja.0.set_tooltip(Some(&resumen)) {
        tracing::warn!(error = %e, "no se pudo actualizar el texto emergente de la bandeja");
    }
    match construir_menu(app, pausado, &resumen) {
        Ok(menu) => {
            if let Err(e) = bandeja.0.set_menu(Some(menu)) {
                tracing::warn!(error = %e, "no se pudo actualizar el menú de la bandeja");
            }
        }
        Err(e) => tracing::warn!(error = %e, "no se pudo construir el menú de la bandeja"),
    }
}

fn alternar_pausa(app: &AppHandle) {
    let estado = app.state::<AppState>();
    let ya_pausado = {
        let mut pausa = estado
            .paused
            .lock()
            .expect("el mutex de pausa no se envenena: sin pánicos dentro");
        let estaba = pausa.is_some();
        *pausa = if estaba {
            None
        } else {
            Some(
                time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            )
        };
        estaba
    };
    tracing::info!(
        pausado_ahora = !ya_pausado,
        "monitorización conmutada desde la bandeja"
    );
    actualizar(app);
}

/// Construye el icono inicial y registra sus manejadores de clic. Se llama una sola vez, desde
/// `setup()`. El color se calcula de inmediato con `actualizar()`, no se deja en un valor inicial
/// inventado.
pub fn instalar(app: &AppHandle) -> tauri::Result<()> {
    let menu_inicial = construir_menu(app, false, "")?;

    let tray = TrayIconBuilder::new()
        .icon(icono(HealthState::Unknown))
        .menu(&menu_inicial)
        .tooltip("SmartDisk Monitor")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_ABRIR => ventana::restaurar_ventana_principal(app),
            ID_PAUSAR => alternar_pausa(app),
            ID_SALIR => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                ventana::restaurar_ventana_principal(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(BandejaIcono(tray));
    actualizar(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_resumen_pausado_ignora_los_dispositivos() {
        assert_eq!(
            texto_resumen("es", true, &[HealthState::Crit]),
            "Monitorización pausada"
        );
    }

    #[test]
    fn el_resumen_sin_dispositivos_lo_dice() {
        assert_eq!(texto_resumen("es", false, &[]), "Sin discos monitorizados");
    }

    #[test]
    fn el_resumen_todo_correcto_lo_dice() {
        assert_eq!(
            texto_resumen("es", false, &[HealthState::Ok, HealthState::Ok]),
            "Todo en orden"
        );
    }

    #[test]
    fn el_resumen_cuenta_los_que_necesitan_atencion() {
        assert_eq!(
            texto_resumen(
                "es",
                false,
                &[HealthState::Ok, HealthState::Warn, HealthState::Crit]
            ),
            "2 discos necesitan atención"
        );
    }

    #[test]
    fn el_resumen_en_singular_no_usa_plural() {
        assert_eq!(
            texto_resumen("es", false, &[HealthState::Warn]),
            "1 disco necesita atención"
        );
    }
}
