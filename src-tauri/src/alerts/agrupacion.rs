//! Agrupación y ciclo de vida de las alertas: deduplicación, contador de ocurrencias y las
//! transiciones de `docs/alert-rules.md` §1. Persiste lo que `alerts::motor` evaluó puro; no
//! decide severidad por sí solo.
//!
//! **Cooldown de notificación** (aviso al usuario) es responsabilidad de `T053`, no de este
//! módulo: aquí toda ocurrencia se registra siempre — "la cronología del grupo lo registra todo"
//! (`alert-rules.md`, "Cooldown de notificación") —, se notifique o no.

use rusqlite::Connection;

use crate::domain::tipos::{AlertGroup, AlertSeverity, AlertStatus};
use crate::persistence::repo_alertas;

/// El resultado de evaluar una regla sobre un objetivo concreto, listo para agrupar. `motor.rs`
/// decide `severity_si_activa` y `resuelto`; este módulo solo sabe de identidad y persistencia.
#[derive(Debug, Clone)]
pub struct EvaluacionAlerta {
    pub rule_key: String,
    pub target_device_id: Option<String>,
    pub target_volume_id: Option<String>,
    /// Discriminante propio de la regla (id. de sensor, bit, `provider:event_id`…). `None` cuando
    /// la regla no lo necesita: el objetivo ya es suficiente clave.
    pub context: Option<String>,
    pub severity_si_activa: Option<AlertSeverity>,
    /// Solo se consulta cuando `severity_si_activa` es `None` y existe un grupo activo o
    /// reconocido: ¿la condición de resolución de `motor.rs` se cumple ya?
    pub resuelto: bool,
    pub value: Option<f64>,
    pub occurred_at_utc: String,
    /// `system_events.id` del evento de Windows que provocó esta evaluación, para
    /// `alert_occurrences.triggering_event_id` (spec 003). `None` para las reglas de SMART y
    /// capacidad, que no salen de un evento.
    pub triggering_event_id: Option<i64>,
}

/// `deduplication_key = rule_key | target_type:target_id | context` (`alert-rules.md` §1). Se
/// reutiliza como `id` del grupo: ya es única por construcción (`UNIQUE` en el esquema), y generar
/// un identificador aparte solo añadiría una dependencia (`uuid`) para no ganar nada.
pub fn deduplication_key(
    rule_key: &str,
    target_device_id: Option<&str>,
    target_volume_id: Option<&str>,
    context: Option<&str>,
) -> String {
    let objetivo = match (target_device_id, target_volume_id) {
        (Some(d), _) => format!("device:{d}"),
        (None, Some(v)) => format!("volume:{v}"),
        (None, None) => "sin_objetivo".to_string(),
    };
    match context {
        Some(c) => format!("{rule_key}|{objetivo}|{c}"),
        None => format!("{rule_key}|{objetivo}"),
    }
}

fn severidad_sube(nueva: AlertSeverity, actual: AlertSeverity) -> bool {
    matches!(
        (actual, nueva),
        (AlertSeverity::Warning, AlertSeverity::Critical)
    )
}

/// Qué le pasó al grupo, para que T053 decida si hay que notificar — este módulo no lo decide
/// (ver cabecera): un episodio nuevo o una recaída siempre se notifican; una ocurrencia repetida
/// respeta el cooldown de la regla; una resolución nunca notifica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transicion {
    /// Nada cambió: ni se activó, ni resolvió, ni hubo ocurrencia nueva.
    SinCambio,
    /// No había grupo y la regla se activó: primer episodio.
    CreadaActiva,
    /// El grupo ya estaba activo o reconocido y se registró una ocurrencia más, sin escalar.
    OcurrenciaRepetida,
    /// Reconocida que sube de severidad y vuelve a `active` (`alert-rules.md` §1, B.6).
    Escalada,
    /// Resuelta o archivada que vuelve a cumplirse: nuevo ciclo sobre el mismo grupo.
    Reactivada,
    /// Pasó a `resolved`.
    Resuelta,
}

/// Aplica una evaluación al estado persistido. Cubre las cinco combinaciones reales de la tabla de
/// transiciones; el resto (grupo resuelto/archivado que sigue sin activarse, activo que aún no
/// resuelve) no cambia nada — silencio correcto, no un caso olvidado.
pub fn procesar(conn: &Connection, ev: &EvaluacionAlerta) -> rusqlite::Result<Transicion> {
    let clave = deduplication_key(
        &ev.rule_key,
        ev.target_device_id.as_deref(),
        ev.target_volume_id.as_deref(),
        ev.context.as_deref(),
    );
    let existente = repo_alertas::get_group_by_dedup_key(conn, &clave)?;

    let transicion = match (existente, ev.severity_si_activa) {
        (None, None) => Ok::<_, rusqlite::Error>(Transicion::SinCambio),

        // Primer episodio: no había grupo y la regla se activa.
        (None, Some(severidad)) => {
            let grupo = AlertGroup {
                id: clave.clone(),
                deduplication_key: clave.clone(),
                rule_key: ev.rule_key.clone(),
                target_device_id: ev.target_device_id.clone(),
                target_volume_id: ev.target_volume_id.clone(),
                severity: severidad,
                status: AlertStatus::Active,
                muted_until: None,
                cycle: 1,
                first_occurrence_at_utc: ev.occurred_at_utc.clone(),
                last_occurrence_at_utc: ev.occurred_at_utc.clone(),
                occurrence_count: 1,
                acknowledged_at_utc: None,
                resolved_at_utc: None,
                archived_at_utc: None,
                last_value_real: ev.value,
                context_json: None,
            };
            repo_alertas::create_group(conn, &grupo)?;
            Ok(Transicion::CreadaActiva)
        }

        // Sigue activa (o reconocida): nueva ocurrencia sobre el mismo grupo (SC-004), con
        // posible escalado. "acknowledged que sube de severidad vuelve a active y notifica de
        // nuevo" (`alert-rules.md` §1); bajar de severidad se refleja pero no cambia el estado.
        (Some(g), Some(severidad))
            if matches!(g.status, AlertStatus::Active | AlertStatus::Acknowledged) =>
        {
            repo_alertas::record_occurrence(
                conn,
                &g.id,
                g.cycle,
                &ev.occurred_at_utc,
                ev.value,
                None,
            )?;
            if severidad != g.severity {
                repo_alertas::set_severity(conn, &g.id, severidad)?;
            }
            let escala =
                g.status == AlertStatus::Acknowledged && severidad_sube(severidad, g.severity);
            if escala {
                repo_alertas::set_status(conn, &g.id, AlertStatus::Active, &ev.occurred_at_utc)?;
            }
            Ok(if escala {
                Transicion::Escalada
            } else {
                Transicion::OcurrenciaRepetida
            })
        }

        // Resuelto o archivado que vuelve a activarse: nuevo ciclo, no un grupo nuevo — se
        // conserva el contador histórico (`alert_groups.cycle`).
        (Some(g), Some(severidad)) => {
            repo_alertas::reopen_as_new_cycle(conn, &g.id, &ev.occurred_at_utc, ev.value)?;
            repo_alertas::set_severity(conn, &g.id, severidad)?;
            Ok(Transicion::Reactivada)
        }

        // Activo o reconocido que cumple ya su condición de resolución.
        (Some(g), None)
            if matches!(g.status, AlertStatus::Active | AlertStatus::Acknowledged)
                && ev.resuelto =>
        {
            repo_alertas::set_status(conn, &g.id, AlertStatus::Resolved, &ev.occurred_at_utc)?;
            Ok(Transicion::Resuelta)
        }

        // Nada que hacer: sigue sin activarse, o activo pero aún no resuelve.
        _ => Ok(Transicion::SinCambio),
    }?;

    // Si la transición registró una ocurrencia y el evaluador nos dio el evento de Windows que la
    // provocó (reglas `events.*`), lo estampamos en esa ocurrencia (`alert_occurrences`
    // .triggering_event_id) — así el detalle de la alerta puede enlazar al suceso (spec 003, D5).
    if let Some(evento_id) = ev.triggering_event_id {
        if matches!(
            transicion,
            Transicion::CreadaActiva
                | Transicion::OcurrenciaRepetida
                | Transicion::Escalada
                | Transicion::Reactivada
        ) {
            repo_alertas::set_triggering_event_ultima_ocurrencia(conn, &clave, evento_id)?;
        }
    }

    Ok(transicion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("alerts_agrupacion");
        let conn = db::open(&dir).unwrap().0;
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn evaluacion(sev: Option<AlertSeverity>, resuelto: bool, cuando: &str) -> EvaluacionAlerta {
        EvaluacionAlerta {
            rule_key: "temp.above_configured_warn".to_string(),
            target_device_id: Some("d1".to_string()),
            target_volume_id: None,
            context: None,
            severity_si_activa: sev,
            resuelto,
            value: Some(75.0),
            occurred_at_utc: cuando.to_string(),
            triggering_event_id: None,
        }
    }

    #[test]
    fn una_activacion_sin_grupo_previo_crea_uno_nuevo() {
        let conn = conn_de_prueba();
        let transicion = procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        assert_eq!(transicion, Transicion::CreadaActiva);

        let grupos = repo_alertas::list_groups(&conn).unwrap();
        assert_eq!(grupos.len(), 1);
        assert_eq!(grupos[0].status, AlertStatus::Active);
        assert_eq!(grupos[0].occurrence_count, 1);
    }

    #[test]
    fn el_evento_disparador_se_estampa_en_la_ocurrencia_de_la_regla_de_evento() {
        let conn = conn_de_prueba();
        // Un `system_events` real al que apuntar (FK).
        conn.execute(
            "INSERT INTO system_events (id, channel, record_id, occurred_at_utc, provider, event_id, level, mapping_confidence, dedup_hash)
             VALUES (77, 'System', 1, '2026-09-04T10:00:00Z', 'disk', 7, 'error', 'exact', 'h')",
            [],
        )
        .unwrap();

        let mut ev = evaluacion(Some(AlertSeverity::Critical), false, "2026-09-04T10:00:00Z");
        ev.rule_key = "events.disk_error".to_string();
        ev.triggering_event_id = Some(77);

        procesar(&conn, &ev).unwrap();
        let id = repo_alertas::list_groups(&conn).unwrap()[0].id.clone();
        let ocurrencias = repo_alertas::list_occurrences(&conn, &id).unwrap();
        assert_eq!(ocurrencias[0].triggering_event_id, Some(77));

        // Una regla sin evento (SMART) deja el campo en `None`.
        let ev_smart = evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T11:00:00Z");
        procesar(&conn, &ev_smart).unwrap();
        let id_smart = repo_alertas::list_groups(&conn)
            .unwrap()
            .into_iter()
            .find(|g| g.rule_key == "temp.above_configured_warn")
            .unwrap()
            .id;
        assert_eq!(
            repo_alertas::list_occurrences(&conn, &id_smart).unwrap()[0].triggering_event_id,
            None
        );
    }

    #[test]
    fn repetir_la_activacion_incrementa_el_contador_de_un_solo_grupo_sc004() {
        let conn = conn_de_prueba();
        for i in 0..5 {
            let transicion = procesar(
                &conn,
                &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
            )
            .unwrap();
            let esperada = if i == 0 {
                Transicion::CreadaActiva
            } else {
                Transicion::OcurrenciaRepetida
            };
            assert_eq!(transicion, esperada);
        }
        let grupos = repo_alertas::list_groups(&conn).unwrap();
        assert_eq!(
            grupos.len(),
            1,
            "cinco activaciones, un solo grupo (SC-004)"
        );
        assert_eq!(grupos[0].occurrence_count, 5);
    }

    #[test]
    fn una_activacion_con_la_misma_clave_pero_distinto_contexto_crea_grupos_distintos() {
        let conn = conn_de_prueba();
        let mut ev1 = evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z");
        ev1.rule_key = "temp.above_vendor_limit".to_string();
        ev1.context = Some("sensor-1".to_string());
        let mut ev2 = ev1.clone();
        ev2.context = Some("sensor-2".to_string());

        procesar(&conn, &ev1).unwrap();
        procesar(&conn, &ev2).unwrap();

        assert_eq!(
            repo_alertas::list_groups(&conn).unwrap().len(),
            2,
            "sensores distintos, grupos distintos"
        );
    }

    #[test]
    fn resolver_no_deja_el_grupo_activo() {
        let conn = conn_de_prueba();
        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        let transicion = procesar(&conn, &evaluacion(None, true, "2026-09-04T11:00:00Z")).unwrap();
        assert_eq!(transicion, Transicion::Resuelta);

        let grupo = repo_alertas::list_groups(&conn)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(grupo.status, AlertStatus::Resolved);
    }

    #[test]
    fn no_resuelto_todavia_no_cambia_el_estado() {
        let conn = conn_de_prueba();
        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        procesar(&conn, &evaluacion(None, false, "2026-09-04T11:00:00Z")).unwrap();

        let grupo = repo_alertas::list_groups(&conn)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(
            grupo.status,
            AlertStatus::Active,
            "sin cumplir la resolución, sigue activo"
        );
    }

    #[test]
    fn recaer_tras_resolverse_incrementa_el_ciclo_en_vez_de_crear_grupo_nuevo() {
        let conn = conn_de_prueba();
        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        procesar(&conn, &evaluacion(None, true, "2026-09-04T11:00:00Z")).unwrap();
        let transicion = procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Critical), false, "2026-09-05T09:00:00Z"),
        )
        .unwrap();
        assert_eq!(transicion, Transicion::Reactivada);

        let grupos = repo_alertas::list_groups(&conn).unwrap();
        assert_eq!(grupos.len(), 1, "la recaída no crea un segundo grupo");
        assert_eq!(grupos[0].cycle, 2);
        assert_eq!(grupos[0].status, AlertStatus::Active);
        assert_eq!(grupos[0].severity, AlertSeverity::Critical);
    }

    #[test]
    fn reconocer_no_cambia_el_estado_al_repetirse_la_misma_severidad() {
        let conn = conn_de_prueba();
        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        let id = repo_alertas::list_groups(&conn).unwrap()[0].id.clone();
        repo_alertas::set_status(
            &conn,
            &id,
            AlertStatus::Acknowledged,
            "2026-09-04T10:30:00Z",
        )
        .unwrap();

        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T11:00:00Z"),
        )
        .unwrap();

        let grupo = repo_alertas::get_group(&conn, &id).unwrap().unwrap();
        assert_eq!(
            grupo.status,
            AlertStatus::Acknowledged,
            "reconocer no lo devuelve a activo mientras la severidad no suba (constitución §I)"
        );
        assert_eq!(
            grupo.occurrence_count, 2,
            "pero la ocurrencia sigue contando en la cronología"
        );
    }

    #[test]
    fn reconocida_que_sube_de_severidad_vuelve_a_activa_y_notifica_de_nuevo() {
        let conn = conn_de_prueba();
        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        let id = repo_alertas::list_groups(&conn).unwrap()[0].id.clone();
        repo_alertas::set_status(
            &conn,
            &id,
            AlertStatus::Acknowledged,
            "2026-09-04T10:30:00Z",
        )
        .unwrap();

        let transicion = procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Critical), false, "2026-09-04T11:00:00Z"),
        )
        .unwrap();
        assert_eq!(transicion, Transicion::Escalada);

        let grupo = repo_alertas::get_group(&conn, &id).unwrap().unwrap();
        assert_eq!(grupo.status, AlertStatus::Active);
        assert_eq!(grupo.severity, AlertSeverity::Critical);
    }

    #[test]
    fn reconocida_que_baja_de_severidad_se_queda_reconocida() {
        let conn = conn_de_prueba();
        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Critical), false, "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        let id = repo_alertas::list_groups(&conn).unwrap()[0].id.clone();
        repo_alertas::set_status(
            &conn,
            &id,
            AlertStatus::Acknowledged,
            "2026-09-04T10:30:00Z",
        )
        .unwrap();

        procesar(
            &conn,
            &evaluacion(Some(AlertSeverity::Warning), false, "2026-09-04T11:00:00Z"),
        )
        .unwrap();

        let grupo = repo_alertas::get_group(&conn, &id).unwrap().unwrap();
        assert_eq!(
            grupo.status,
            AlertStatus::Acknowledged,
            "bajar de severidad no reactiva la alerta"
        );
        assert_eq!(grupo.severity, AlertSeverity::Warning);
    }
}
