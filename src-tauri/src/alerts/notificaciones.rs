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

/// Política de notificación de una ocurrencia repetida (`alert-rules.md` §2, columna "Cooldown de
/// notificación"). Un episodio nuevo, una recaída o una escalada notifican siempre, sea cual sea la
/// política; esto solo decide qué hacer cuando la misma condición se vuelve a cumplir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notificacion {
    /// Cada ocurrencia notifica (`smart.health.failed`, `nvme.critical_warning`).
    Siempre,
    /// Notifica si ha pasado el tiempo indicado desde la última.
    TrasCooldown(Duration),
    /// **Nunca** en ocurrencia repetida: solo al cruzar un umbral / cambiar de nivel
    /// (`capacity.*` — `alert-rules.md`: «Nunca una notificación por muestra»).
    SoloAlCambiarDeNivel,
}

/// Política por regla. Una regla fuera de esta lista cae a un cooldown moderado de una hora: más
/// vale una notificación de menos que un aluvión de una regla que no se ha calibrado.
fn politica_notificacion(rule_key: &str) -> Notificacion {
    use Notificacion::*;
    match rule_key {
        "smart.health.failed" | "nvme.critical_warning" => Siempre,
        "smart.media_errors" | "smart.error_log" => TrasCooldown(Duration::hours(1)),
        "smart.spare_below_threshold" | "smart.unreadable" => TrasCooldown(Duration::hours(6)),
        "smart.wear_high" => TrasCooldown(Duration::days(7)),
        "temp.above_configured_warn" | "temp.above_vendor_limit" => {
            TrasCooldown(Duration::minutes(30))
        }
        "temp.above_configured_crit" => TrasCooldown(Duration::minutes(15)),
        "collector.stalled" => TrasCooldown(Duration::hours(1)),
        "capacity.low" | "capacity.critical" => SoloAlCambiarDeNivel,

        // Reglas de eventos de Windows (`docs/alert-rules.md` §2, columna «Cooldown de
        // notificación» — spec 003).
        "events.disk_error"
        | "events.filesystem_error"
        | "events.controller_reset"
        | "events.delayed_write"
        | "events.storage_space_degraded" => TrasCooldown(Duration::hours(1)),
        "events.paging_error" | "events.io_retry" => TrasCooldown(Duration::hours(6)),
        "events.filesystem_repaired" | "events.disk_predictive" => {
            TrasCooldown(Duration::hours(24))
        }
        "events.filesystem_repair_storm" => TrasCooldown(Duration::hours(6)),
        // «ninguno: siempre notifica».
        "device.removed_unexpected" => Siempre,
        // «una sola vez por par»: notifica al crearse, nunca en ocurrencia repetida.
        "inventory.duplicate_id" => SoloAlCambiarDeNivel,

        _ => TrasCooldown(Duration::hours(1)),
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
        politica_notificacion(&grupo.rule_key),
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
    politica: Notificacion,
    desde_ultima: Option<Duration>,
) -> bool {
    if !notificaciones_activas || silenciada {
        return false;
    }
    match transicion {
        // Un episodio nuevo, una recaída o una escalada (que en `capacity.*` es el cambio de nivel)
        // notifican siempre, sea cual sea la política.
        Transicion::CreadaActiva | Transicion::Reactivada | Transicion::Escalada => true,
        Transicion::OcurrenciaRepetida => match politica {
            Notificacion::Siempre => true,
            Notificacion::TrasCooldown(cd) => desde_ultima.map_or(true, |d| d >= cd),
            Notificacion::SoloAlCambiarDeNivel => false,
        },
        // Una alerta ignorada nunca notifica, ni siquiera si la ocurrencia sube de severidad
        // (ADR-044, spec FR-004).
        Transicion::SinCambio | Transicion::Resuelta | Transicion::OcurrenciaIgnorada => false,
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
    fn health_failed_notifica_en_cada_ocurrencia() {
        assert_eq!(
            politica_notificacion("smart.health.failed"),
            Notificacion::Siempre
        );
    }

    #[test]
    fn wear_high_tiene_cooldown_de_siete_dias() {
        assert_eq!(
            politica_notificacion("smart.wear_high"),
            Notificacion::TrasCooldown(Duration::days(7))
        );
    }

    #[test]
    fn smart_unreadable_tiene_cooldown_de_seis_horas() {
        assert_eq!(
            politica_notificacion("smart.unreadable"),
            Notificacion::TrasCooldown(Duration::hours(6))
        );
    }

    #[test]
    fn temp_warn_y_crit_tienen_cooldowns_distintos() {
        assert_eq!(
            politica_notificacion("temp.above_configured_warn"),
            Notificacion::TrasCooldown(Duration::minutes(30))
        );
        assert_eq!(
            politica_notificacion("temp.above_configured_crit"),
            Notificacion::TrasCooldown(Duration::minutes(15))
        );
    }

    #[test]
    fn temp_del_fabricante_hereda_el_cooldown_de_treinta_minutos() {
        assert_eq!(
            politica_notificacion("temp.above_vendor_limit"),
            Notificacion::TrasCooldown(Duration::minutes(30))
        );
    }

    #[test]
    fn collector_stalled_tiene_cooldown_de_una_hora() {
        assert_eq!(
            politica_notificacion("collector.stalled"),
            Notificacion::TrasCooldown(Duration::hours(1))
        );
    }

    #[test]
    fn capacidad_solo_notifica_al_cambiar_de_nivel() {
        // `alert-rules.md` §Cooldown: «Nunca una notificación por muestra».
        assert_eq!(
            politica_notificacion("capacity.low"),
            Notificacion::SoloAlCambiarDeNivel
        );
        assert_eq!(
            politica_notificacion("capacity.critical"),
            Notificacion::SoloAlCambiarDeNivel
        );
    }

    #[test]
    fn una_regla_desconocida_no_se_queda_sin_cooldown() {
        assert_eq!(
            politica_notificacion("regla.inventada"),
            Notificacion::TrasCooldown(Duration::hours(1))
        );
    }

    #[test]
    fn con_las_notificaciones_apagadas_no_se_envia_nada_aunque_sea_una_alerta_nueva() {
        // ADR-037: el apagado explícito manda sobre cualquier transición.
        assert!(!debe_enviar(
            false,
            false,
            Transicion::CreadaActiva,
            Notificacion::Siempre,
            None
        ));
    }

    #[test]
    fn una_alerta_silenciada_no_notifica_aunque_las_notificaciones_esten_activas() {
        assert!(!debe_enviar(
            true,
            true,
            Transicion::Escalada,
            Notificacion::Siempre,
            None
        ));
    }

    #[test]
    fn con_las_notificaciones_activas_una_alerta_nueva_o_escalada_siempre_notifica() {
        for t in [
            Transicion::CreadaActiva,
            Transicion::Reactivada,
            Transicion::Escalada,
        ] {
            // Incluso con la política más restrictiva: un cambio de nivel siempre avisa.
            assert!(debe_enviar(
                true,
                false,
                t,
                Notificacion::SoloAlCambiarDeNivel,
                None
            ));
        }
    }

    #[test]
    fn una_ocurrencia_repetida_respeta_el_cooldown() {
        // Dentro del cooldown → no; pasado el cooldown → sí; sin registro previo → sí.
        let cd = Notificacion::TrasCooldown(Duration::minutes(30));
        assert!(!debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            cd,
            Some(Duration::minutes(10))
        ));
        assert!(debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            cd,
            Some(Duration::minutes(31))
        ));
        assert!(debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            cd,
            None
        ));
    }

    #[test]
    fn una_ocurrencia_repetida_de_capacidad_nunca_notifica() {
        // Aunque haya pasado mucho tiempo desde la última: `SoloAlCambiarDeNivel` no cede.
        assert!(!debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            Notificacion::SoloAlCambiarDeNivel,
            Some(Duration::days(30))
        ));
        assert!(!debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            Notificacion::SoloAlCambiarDeNivel,
            None
        ));
    }

    #[test]
    fn una_ocurrencia_repetida_con_politica_siempre_notifica() {
        assert!(debe_enviar(
            true,
            false,
            Transicion::OcurrenciaRepetida,
            Notificacion::Siempre,
            Some(Duration::seconds(1))
        ));
    }

    #[test]
    fn sin_cambio_o_resuelta_nunca_notifican() {
        for t in [Transicion::SinCambio, Transicion::Resuelta] {
            assert!(!debe_enviar(true, false, t, Notificacion::Siempre, None));
        }
    }

    #[test]
    fn una_ocurrencia_ignorada_nunca_notifica_pase_lo_que_pase() {
        for politica in [
            Notificacion::Siempre,
            Notificacion::TrasCooldown(Duration::hours(1)),
            Notificacion::SoloAlCambiarDeNivel,
        ] {
            for silenciada in [true, false] {
                assert!(!debe_enviar(
                    true,
                    silenciada,
                    Transicion::OcurrenciaIgnorada,
                    politica,
                    None
                ));
                assert!(!debe_enviar(
                    true,
                    silenciada,
                    Transicion::OcurrenciaIgnorada,
                    politica,
                    Some(Duration::days(30))
                ));
            }
        }
    }

    #[test]
    fn las_reglas_de_eventos_tienen_la_politica_de_alert_rules_md_2() {
        use Notificacion::*;
        assert_eq!(
            politica_notificacion("events.disk_error"),
            TrasCooldown(Duration::hours(1))
        );
        assert_eq!(
            politica_notificacion("events.paging_error"),
            TrasCooldown(Duration::hours(6))
        );
        assert_eq!(
            politica_notificacion("events.io_retry"),
            TrasCooldown(Duration::hours(6))
        );
        assert_eq!(
            politica_notificacion("events.filesystem_repaired"),
            TrasCooldown(Duration::hours(24))
        );
        assert_eq!(
            politica_notificacion("events.disk_predictive"),
            TrasCooldown(Duration::hours(24))
        );
        assert_eq!(
            politica_notificacion("events.filesystem_repair_storm"),
            TrasCooldown(Duration::hours(6))
        );
        // «ninguno: siempre notifica».
        assert_eq!(politica_notificacion("device.removed_unexpected"), Siempre);
        // «una sola vez por par»: nunca en ocurrencia repetida.
        assert_eq!(
            politica_notificacion("inventory.duplicate_id"),
            SoloAlCambiarDeNivel
        );
    }

    /// Todas las reglas en alcance (`docs/open-questions.md` J.16) tienen sus claves de i18n
    /// propias: sin ellas, `enviar_y_registrar` mostraría la clave cruda como título de la
    /// notificación.
    #[test]
    fn las_reglas_en_alcance_tienen_titulo_y_resumen_propios() {
        for regla in [
            "smart.health.failed",
            "nvme.critical_warning",
            "smart.media_errors",
            "smart.error_log",
            "smart.spare_below_threshold",
            "smart.wear_high",
            "smart.unreadable",
            "temp.above_configured_warn",
            "temp.above_configured_crit",
            "temp.above_vendor_limit",
            "capacity.low",
            "capacity.critical",
            "collector.stalled",
            "events.disk_error",
            "events.filesystem_error",
            "events.filesystem_repaired",
            "events.filesystem_repair_storm",
            "events.controller_reset",
            "events.paging_error",
            "events.io_retry",
            "events.delayed_write",
            "events.disk_predictive",
            "events.storage_space_degraded",
            "device.removed_unexpected",
            "inventory.duplicate_id",
        ] {
            for sufijo in ["title", "summary"] {
                let clave = format!("alert.rule.{regla}.{sufijo}");
                assert_ne!(t("es", &clave), clave, "falta la clave de i18n {clave:?}");
            }
        }
    }
}
