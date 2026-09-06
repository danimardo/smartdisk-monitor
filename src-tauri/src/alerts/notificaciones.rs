//! Decide y envía la notificación nativa cuando corresponde (T053, `docs/alert-rules.md`
//! "Cooldown de notificación", `docs/open-questions.md` I.2/R1).
//!
//! `alerts::agrupacion` deja explícito en su propia cabecera que esto no es asunto suyo: aquí se
//! decide, a partir de la `Transicion` que devolvió y del silencio (`muted_until`) y el cooldown
//! por regla que `agrupacion` no conoce.
//!
//! **R1 sin medir todavía**: si el toast nativo llega de verdad bajo `requireAdministrator` con el
//! identificador de aplicación registrado solo puede comprobarse al empaquetar (`research.md` R1) —
//! en desarrollo, sin ese identificador, una prueba ahora mediría otra cosa. La alternativa ya
//! decidida (ventana propia con `Toast`) queda pendiente de esa medición, no implementada aquí.

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use time::{Duration, OffsetDateTime};

use crate::alerts::{agrupacion::Transicion, ciclo};
use crate::domain::tipos::AlertGroup;
use crate::persistence::db::AppState;
use crate::persistence::{repo_alertas, repo_varios};
use crate::platform::rotulos::{locale_actual, t};

/// Cooldown por regla (`alert-rules.md` §2, columna "Cooldown de notificación"). `None` = "ninguno:
/// siempre notifica". Una regla fuera de esta lista no debería llegar aquí con las 8 en alcance de
/// J.16, pero si ocurre cae a un cooldown moderado de una hora: más vale una notificación de menos
/// que un aluvión de una regla que no se ha calibrado.
fn cooldown_de(rule_key: &str) -> Option<Duration> {
    match rule_key {
        "smart.health.failed" | "nvme.critical_warning" => None,
        "smart.media_errors" | "smart.error_log" => Some(Duration::hours(1)),
        "smart.spare_below_threshold" | "smart.unreadable" => Some(Duration::hours(6)),
        "smart.wear_high" => Some(Duration::days(7)),
        "temp.above_configured_warn" => Some(Duration::minutes(30)),
        "temp.above_configured_crit" => Some(Duration::minutes(15)),
        _ => Some(Duration::hours(1)),
    }
}

/// Aplica la decisión a cada transición no trivial de un ciclo de `refresh_smart`. Se llama tras
/// soltar el bloqueo de conexión que usó `alerts::evaluar_smart`, no dentro de él.
pub fn procesar_transiciones(app: &AppHandle, transiciones: &[(String, Transicion)]) {
    for (grupo_id, transicion) in transiciones {
        if let Err(e) = procesar_una(app, grupo_id, *transicion) {
            tracing::warn!(grupo = %grupo_id, error = ?e, "no se pudo evaluar si tocaba notificar");
        }
    }
}

fn procesar_una(app: &AppHandle, grupo_id: &str, transicion: Transicion) -> rusqlite::Result<()> {
    let estado = app.state::<AppState>();
    let (grupo, notificaciones_activas) = {
        let conn = estado
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        let grupo = repo_alertas::get_group(&conn, grupo_id)?;
        // Apagado explícito del toast (ADR-037), independiente de pausar. `notifications.enabled`
        // ausente = mostrar (valor de fábrica).
        let activas = repo_varios::get_setting_raw(&conn, "notifications.enabled")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<bool>(&s).ok())
            .unwrap_or(true);
        (grupo, activas)
    };
    let Some(grupo) = grupo else {
        return Ok(());
    };

    let ahora = OffsetDateTime::now_utc();
    let silenciada = ciclo::esta_silenciada(grupo.muted_until.as_deref(), ahora);

    let desde_ultima = {
        let mapa = estado
            .notified_at
            .lock()
            .expect("el mutex de notificaciones no se envenena: sin pánicos dentro");
        mapa.get(grupo_id).map(|ultima| ahora - *ultima)
    };

    if debe_enviar(
        notificaciones_activas,
        silenciada,
        transicion,
        cooldown_de(&grupo.rule_key),
        desde_ultima,
    ) {
        enviar_y_registrar(app, &estado, &grupo, ahora);
    }
    Ok(())
}

/// ¿Toca enviar el toast? Decisión pura, sin `AppHandle` ni base de datos, para poder fijarla en
/// pruebas (el resto de `procesar_una` necesita un proceso Tauri real, `research.md` R1).
fn debe_enviar(
    notificaciones_activas: bool,
    silenciada: bool,
    transicion: Transicion,
    cooldown: Option<Duration>,
    desde_ultima: Option<Duration>,
) -> bool {
    if !notificaciones_activas || silenciada {
        return false;
    }
    match transicion {
        Transicion::CreadaActiva | Transicion::Reactivada | Transicion::Escalada => true,
        Transicion::OcurrenciaRepetida => match cooldown {
            None => true,
            Some(cd) => desde_ultima.map_or(true, |d| d >= cd),
        },
        Transicion::SinCambio | Transicion::Resuelta => false,
    }
}

fn enviar_y_registrar(
    app: &AppHandle,
    estado: &AppState,
    grupo: &AlertGroup,
    ahora: OffsetDateTime,
) {
    let locale = locale_actual();
    let titulo = t(locale, &format!("alert.rule.{}.title", grupo.rule_key));
    let cuerpo = t(locale, &format!("alert.rule.{}.summary", grupo.rule_key));
    if let Err(e) = app
        .notification()
        .builder()
        .title(titulo)
        .body(cuerpo)
        .show()
    {
        tracing::warn!(error = %e, "no se pudo mostrar la notificación nativa");
    }
    estado
        .notified_at
        .lock()
        .expect("el mutex de notificaciones no se envenena: sin pánicos dentro")
        .insert(grupo.id.clone(), ahora);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_failed_no_tiene_cooldown_siempre_notifica() {
        assert_eq!(cooldown_de("smart.health.failed"), None);
    }

    #[test]
    fn wear_high_tiene_cooldown_de_siete_dias() {
        assert_eq!(cooldown_de("smart.wear_high"), Some(Duration::days(7)));
    }

    #[test]
    fn smart_unreadable_tiene_cooldown_de_seis_horas() {
        assert_eq!(cooldown_de("smart.unreadable"), Some(Duration::hours(6)));
    }

    #[test]
    fn temp_warn_y_crit_tienen_cooldowns_distintos() {
        assert_eq!(
            cooldown_de("temp.above_configured_warn"),
            Some(Duration::minutes(30))
        );
        assert_eq!(
            cooldown_de("temp.above_configured_crit"),
            Some(Duration::minutes(15))
        );
    }

    #[test]
    fn una_regla_desconocida_no_se_queda_sin_cooldown() {
        assert_eq!(cooldown_de("regla.inventada"), Some(Duration::hours(1)));
    }

    #[test]
    fn con_las_notificaciones_apagadas_no_se_envia_nada_aunque_sea_una_alerta_nueva() {
        // ADR-037: el apagado explícito manda sobre cualquier transición.
        assert!(!debe_enviar(
            false,
            false,
            Transicion::CreadaActiva,
            None,
            None
        ));
    }

    #[test]
    fn una_alerta_silenciada_no_notifica_aunque_las_notificaciones_esten_activas() {
        assert!(!debe_enviar(true, true, Transicion::Escalada, None, None));
    }

    #[test]
    fn con_las_notificaciones_activas_una_alerta_nueva_o_escalada_siempre_notifica() {
        for t in [
            Transicion::CreadaActiva,
            Transicion::Reactivada,
            Transicion::Escalada,
        ] {
            assert!(debe_enviar(true, false, t, Some(Duration::days(7)), None));
        }
    }

    #[test]
    fn una_ocurrencia_repetida_respeta_el_cooldown() {
        // Dentro del cooldown → no; pasado el cooldown → sí; sin registro previo → sí.
        assert!(!debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            Some(Duration::minutes(30)),
            Some(Duration::minutes(10))
        ));
        assert!(debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            Some(Duration::minutes(30)),
            Some(Duration::minutes(31))
        ));
        assert!(debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            Some(Duration::minutes(30)),
            None
        ));
    }

    #[test]
    fn sin_cambio_o_resuelta_nunca_notifican() {
        for t in [Transicion::SinCambio, Transicion::Resuelta] {
            assert!(!debe_enviar(true, false, t, None, None));
        }
    }

    /// Las 8 reglas en alcance (`docs/open-questions.md` J.16) tienen sus claves de i18n propias:
    /// sin ellas, `enviar_y_registrar` mostraría la clave cruda como título de la notificación.
    #[test]
    fn las_ocho_reglas_en_alcance_tienen_titulo_y_resumen_propios() {
        for regla in [
            "smart.health.failed",
            "nvme.critical_warning",
            "smart.media_errors",
            "smart.error_log",
            "smart.spare_below_threshold",
            "smart.wear_high",
            "temp.above_configured_warn",
            "temp.above_configured_crit",
        ] {
            for sufijo in ["title", "summary"] {
                let clave = format!("alert.rule.{regla}.{sufijo}");
                assert_ne!(t("es", &clave), clave, "falta la clave de i18n {clave:?}");
            }
        }
    }
}
