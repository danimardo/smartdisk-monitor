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
use crate::persistence::repo_alertas;
use crate::platform::rotulos::{locale_actual, t};

/// Cooldown por regla (`alert-rules.md` §2, columna "Cooldown de notificación"). `None` = "ninguno:
/// siempre notifica". Una regla fuera de esta lista no debería llegar aquí con las 8 en alcance de
/// J.16, pero si ocurre cae a un cooldown moderado de una hora: más vale una notificación de menos
/// que un aluvión de una regla que no se ha calibrado.
fn cooldown_de(rule_key: &str) -> Option<Duration> {
    match rule_key {
        "smart.health.failed" | "nvme.critical_warning" => None,
        "smart.media_errors" | "smart.error_log" => Some(Duration::hours(1)),
        "smart.spare_below_threshold" => Some(Duration::hours(6)),
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
    let grupo = {
        let conn = estado
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        repo_alertas::get_group(&conn, grupo_id)?
    };
    let Some(grupo) = grupo else {
        return Ok(());
    };

    let ahora = OffsetDateTime::now_utc();
    if ciclo::esta_silenciada(grupo.muted_until.as_deref(), ahora) {
        return Ok(());
    }

    let debe_notificar = match transicion {
        Transicion::CreadaActiva | Transicion::Reactivada | Transicion::Escalada => true,
        Transicion::OcurrenciaRepetida => match cooldown_de(&grupo.rule_key) {
            None => true,
            Some(cooldown) => {
                let mapa = estado
                    .notified_at
                    .lock()
                    .expect("el mutex de notificaciones no se envenena: sin pánicos dentro");
                match mapa.get(grupo_id) {
                    None => true,
                    Some(ultima) => ahora - *ultima >= cooldown,
                }
            }
        },
        Transicion::SinCambio | Transicion::Resuelta => false,
    };

    if debe_notificar {
        enviar_y_registrar(app, &estado, &grupo, ahora);
    }
    Ok(())
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
