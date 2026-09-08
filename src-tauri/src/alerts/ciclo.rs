//! Ciclo de vida por acción del usuario: reconocer, silenciar y archivar
//! (`docs/ui-contract.md` §3.4, `alert-rules.md` §1).
//!
//! El silencio es **ortogonal** al estado: nunca decide si un grupo cuenta para la salud del
//! dispositivo, solo si notifica. Esa es la propiedad que este módulo existe para no romper por
//! accidente — `repo_alertas::list_groups_counting_toward_health` es la única implementación de
//! qué cuenta, y ni reconocer ni silenciar la tocan.

use time::{Duration, OffsetDateTime};

use crate::alerts::reglas;
use crate::domain::tipos::AlertStatus;
use crate::persistence::repo_alertas;

/// `"infinite"` es el valor especial de silencio indefinido (`alert-rules.md` §1, "Valores").
pub const SILENCIO_INDEFINIDO: &str = "infinite";

/// El valor que debe guardarse en `muted_until` para silenciar `minutos` a partir de `ahora`, o el
/// silencio indefinido si `minutos` es `None` (`docs/ui-contract.md` §3.4: `15 | 60 | 480 | null`).
pub fn calcular_muted_until(minutos: Option<u32>, ahora: OffsetDateTime) -> String {
    match minutos {
        None => SILENCIO_INDEFINIDO.to_string(),
        Some(m) => (ahora + Duration::minutes(i64::from(m)))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
    }
}

/// Si un grupo está silenciado **ahora mismo**. No participa en absoluto en si cuenta para la
/// salud del dispositivo — eso es cosa exclusiva de `status` — solo en si debe notificar (T053).
pub fn esta_silenciada(muted_until: Option<&str>, ahora: OffsetDateTime) -> bool {
    match muted_until {
        None => false,
        Some(SILENCIO_INDEFINIDO) => true,
        Some(fecha) => {
            time::OffsetDateTime::parse(fecha, &time::format_description::well_known::Rfc3339)
                .map(|t| t > ahora)
                .unwrap_or(false)
        }
    }
}

pub fn reconocer(conn: &rusqlite::Connection, id: &str, ahora_utc: &str) -> rusqlite::Result<()> {
    repo_alertas::set_status(conn, id, AlertStatus::Acknowledged, ahora_utc)
}

pub fn silenciar(
    conn: &rusqlite::Connection,
    id: &str,
    minutos: Option<u32>,
    ahora: OffsetDateTime,
) -> rusqlite::Result<()> {
    let hasta = calcular_muted_until(minutos, ahora);
    repo_alertas::set_muted_until(conn, id, Some(&hasta))
}

pub fn reanudar_notificaciones(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<()> {
    repo_alertas::set_muted_until(conn, id, None)
}

pub fn archivar(conn: &rusqlite::Connection, id: &str, ahora_utc: &str) -> rusqlite::Result<()> {
    repo_alertas::set_status(conn, id, AlertStatus::Archived, ahora_utc)
}

/// Por qué no se pudo ignorar un grupo.
#[derive(Debug)]
pub enum IgnorarError {
    /// El grupo no existe.
    NoExiste,
    /// La regla del grupo está en `alerts::reglas::REGLAS_NO_IGNORABLES` (ADR-044).
    ReglaNoIgnorable,
    /// Fallo de base de datos.
    Sqlite(rusqlite::Error),
}

impl From<rusqlite::Error> for IgnorarError {
    fn from(e: rusqlite::Error) -> Self {
        IgnorarError::Sqlite(e)
    }
}

/// «Ignorar» un grupo (ADR-044): estado terminal `ignored`, salvo que su regla esté vetada. Desde
/// cualquier estado (`active`, `acknowledged`, `resolved`, `archived`) — spec FR-008.
pub fn ignorar(conn: &rusqlite::Connection, id: &str, ahora_utc: &str) -> Result<(), IgnorarError> {
    let grupo = repo_alertas::get_group(conn, id)?.ok_or(IgnorarError::NoExiste)?;
    if !reglas::regla_es_ignorable(&grupo.rule_key) {
        return Err(IgnorarError::ReglaNoIgnorable);
    }
    repo_alertas::set_status(conn, id, AlertStatus::Ignored, ahora_utc)?;
    Ok(())
}

/// «Dejar de ignorar»: el grupo sale de `ignored` y queda `resolved`. Si su condición se sigue
/// cumpliendo, el motor lo sube a `active` (`cycle + 1`) en el siguiente ciclo (spec FR-011).
pub fn dejar_de_ignorar(
    conn: &rusqlite::Connection,
    id: &str,
    ahora_utc: &str,
) -> rusqlite::Result<()> {
    repo_alertas::dejar_de_ignorar(conn, id, ahora_utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{AlertGroup, AlertSeverity};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("alerts_ciclo");
        let conn = db::open(&dir).unwrap().0;
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn grupo_activo(id: &str) -> AlertGroup {
        AlertGroup {
            id: id.to_string(),
            deduplication_key: format!("dedup-{id}"),
            rule_key: "temp.above_configured_crit".to_string(),
            target_device_id: Some("d1".to_string()),
            target_volume_id: None,
            severity: AlertSeverity::Critical,
            status: AlertStatus::Active,
            muted_until: None,
            cycle: 1,
            first_occurrence_at_utc: "2026-09-04T10:00:00Z".to_string(),
            last_occurrence_at_utc: "2026-09-04T10:00:00Z".to_string(),
            occurrence_count: 1,
            acknowledged_at_utc: None,
            resolved_at_utc: None,
            archived_at_utc: None,
            ignored_at_utc: None,
            last_value_real: Some(85.0),
            context_json: None,
        }
    }

    // ---- El silencio no decide el color ----

    #[test]
    fn silencio_indefinido_no_expira_nunca() {
        let dentro_de_diez_anos = OffsetDateTime::now_utc() + Duration::days(3650);
        assert!(esta_silenciada(
            Some(SILENCIO_INDEFINIDO),
            dentro_de_diez_anos
        ));
    }

    #[test]
    fn silencio_temporal_expira_pasado_su_plazo() {
        let ahora = OffsetDateTime::now_utc();
        let hasta = calcular_muted_until(Some(15), ahora);
        assert!(esta_silenciada(Some(&hasta), ahora));
        assert!(!esta_silenciada(
            Some(&hasta),
            ahora + Duration::minutes(16)
        ));
    }

    #[test]
    fn sin_silencio_nunca_esta_silenciada() {
        assert!(!esta_silenciada(None, OffsetDateTime::now_utc()));
    }

    // ---- Reconocer y silenciar no cambian lo que cuenta para la salud (T049) ----

    #[test]
    fn reconocer_sigue_contando_para_la_salud_del_dispositivo() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(&conn, &grupo_activo("g1")).unwrap();

        reconocer(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

        let cuentan = repo_alertas::list_groups_counting_toward_health(&conn).unwrap();
        assert_eq!(
            cuentan.len(),
            1,
            "reconocida sigue siendo la peor alerta activa (constitución §I)"
        );
        assert_eq!(cuentan[0].status, AlertStatus::Acknowledged);
    }

    #[test]
    fn silenciar_sigue_contando_para_la_salud_del_dispositivo() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(&conn, &grupo_activo("g1")).unwrap();

        silenciar(&conn, "g1", Some(60), OffsetDateTime::now_utc()).unwrap();

        let cuentan = repo_alertas::list_groups_counting_toward_health(&conn).unwrap();
        assert_eq!(
            cuentan.len(),
            1,
            "silenciar suprime la notificación, nunca el color"
        );
    }

    #[test]
    fn reconocer_y_silenciar_a_la_vez_tampoco_cambia_lo_que_cuenta() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(&conn, &grupo_activo("g1")).unwrap();

        reconocer(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();
        silenciar(&conn, "g1", None, OffsetDateTime::now_utc()).unwrap();

        assert_eq!(
            repo_alertas::list_groups_counting_toward_health(&conn)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn archivar_deja_de_contar_para_la_salud() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(&conn, &grupo_activo("g1")).unwrap();

        archivar(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

        assert!(repo_alertas::list_groups_counting_toward_health(&conn)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn reanudar_notificaciones_quita_el_silencio_sin_tocar_el_estado() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(&conn, &grupo_activo("g1")).unwrap();
        silenciar(&conn, "g1", None, OffsetDateTime::now_utc()).unwrap();

        reanudar_notificaciones(&conn, "g1").unwrap();

        let grupo = repo_alertas::get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(grupo.muted_until, None);
        assert_eq!(
            grupo.status,
            AlertStatus::Active,
            "quitar el silencio no toca el estado"
        );
    }

    // ---- Ignorar / dejar de ignorar (ADR-044, feature 004) ----

    fn grupo_con_regla(id: &str, rule_key: &str, status: AlertStatus) -> AlertGroup {
        AlertGroup {
            rule_key: rule_key.to_string(),
            status,
            ..grupo_activo(id)
        }
    }

    #[test]
    fn ignorar_deja_el_grupo_en_ignored_con_fecha_desde_cualquier_estado() {
        for estado in [
            AlertStatus::Active,
            AlertStatus::Acknowledged,
            AlertStatus::Resolved,
            AlertStatus::Archived,
        ] {
            let conn = conn_de_prueba();
            repo_alertas::create_group(
                &conn,
                &grupo_con_regla("g1", "temp.above_configured_warn", estado),
            )
            .unwrap();

            ignorar(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

            let grupo = repo_alertas::get_group(&conn, "g1").unwrap().unwrap();
            assert_eq!(grupo.status, AlertStatus::Ignored, "desde {estado:?}");
            assert_eq!(
                grupo.ignored_at_utc.as_deref(),
                Some("2026-09-04T11:00:00Z")
            );
        }
    }

    #[test]
    fn un_grupo_ignorado_no_cuenta_para_la_salud() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(
            &conn,
            &grupo_con_regla("g1", "temp.above_configured_warn", AlertStatus::Active),
        )
        .unwrap();

        ignorar(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

        assert!(repo_alertas::list_groups_counting_toward_health(&conn)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn ignorar_conserva_la_cronologia_y_el_contador() {
        let conn = conn_de_prueba();
        let mut g = grupo_con_regla("g1", "temp.above_configured_warn", AlertStatus::Active);
        g.occurrence_count = 4;
        g.first_occurrence_at_utc = "2026-09-01T00:00:00Z".to_string();
        repo_alertas::create_group(&conn, &g).unwrap();
        repo_alertas::record_occurrence(&conn, "g1", 1, "2026-09-02T00:00:00Z", Some(1.0), None)
            .unwrap();

        ignorar(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

        let grupo = repo_alertas::get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(grupo.occurrence_count, 5);
        assert_eq!(grupo.first_occurrence_at_utc, "2026-09-01T00:00:00Z");
        assert_eq!(
            repo_alertas::list_occurrences(&conn, "g1").unwrap().len(),
            2
        );
    }

    #[test]
    fn ignorar_una_regla_vetada_falla_y_no_cambia_el_estado() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(
            &conn,
            &grupo_con_regla("g1", "smart.wear_high", AlertStatus::Active),
        )
        .unwrap();

        let r = ignorar(&conn, "g1", "2026-09-04T11:00:00Z");
        assert!(matches!(r, Err(IgnorarError::ReglaNoIgnorable)));

        let grupo = repo_alertas::get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(grupo.status, AlertStatus::Active, "no debe haber cambiado");
    }

    #[test]
    fn ignorar_un_grupo_inexistente_falla_con_no_existe() {
        let conn = conn_de_prueba();
        assert!(matches!(
            ignorar(&conn, "fantasma", "2026-09-04T11:00:00Z"),
            Err(IgnorarError::NoExiste)
        ));
    }

    #[test]
    fn dejar_de_ignorar_deja_resolved_limpiando_ignored_at_utc() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(
            &conn,
            &grupo_con_regla("g1", "temp.above_configured_warn", AlertStatus::Active),
        )
        .unwrap();
        ignorar(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

        dejar_de_ignorar(&conn, "g1", "2026-09-05T09:00:00Z").unwrap();

        let grupo = repo_alertas::get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(grupo.status, AlertStatus::Resolved);
        assert_eq!(grupo.ignored_at_utc, None);
        assert_eq!(
            grupo.resolved_at_utc.as_deref(),
            Some("2026-09-05T09:00:00Z")
        );
    }

    #[test]
    fn un_grupo_archivado_luego_ignorado_al_dejar_de_ignorar_queda_resolved_no_archived() {
        let conn = conn_de_prueba();
        repo_alertas::create_group(
            &conn,
            &grupo_con_regla("g1", "temp.above_configured_warn", AlertStatus::Archived),
        )
        .unwrap();
        ignorar(&conn, "g1", "2026-09-04T11:00:00Z").unwrap();

        dejar_de_ignorar(&conn, "g1", "2026-09-05T09:00:00Z").unwrap();

        let grupo = repo_alertas::get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(
            grupo.status,
            AlertStatus::Resolved,
            "nunca de vuelta a archived"
        );
    }

    #[test]
    fn tras_dejar_de_ignorar_el_motor_reactiva_si_la_condicion_se_cumple() {
        use crate::alerts::agrupacion::{procesar, EvaluacionAlerta, Transicion};
        let conn = conn_de_prueba();
        let ev = |sev, cuando: &str| EvaluacionAlerta {
            rule_key: "temp.above_configured_warn".to_string(),
            target_device_id: Some("d1".to_string()),
            target_volume_id: None,
            context: None,
            severity_si_activa: sev,
            resuelto: false,
            value: Some(65.0),
            occurred_at_utc: cuando.to_string(),
            triggering_event_id: None,
        };

        // Grupo creado por el propio motor: así `id`/`deduplication_key` casan.
        procesar(
            &conn,
            &ev(Some(AlertSeverity::Warning), "2026-09-04T10:00:00Z"),
        )
        .unwrap();
        let id = repo_alertas::list_groups(&conn).unwrap()[0].id.clone();
        ignorar(&conn, &id, "2026-09-04T11:00:00Z").unwrap();
        dejar_de_ignorar(&conn, &id, "2026-09-05T09:00:00Z").unwrap();

        assert_eq!(
            procesar(
                &conn,
                &ev(Some(AlertSeverity::Warning), "2026-09-05T10:00:00Z")
            )
            .unwrap(),
            Transicion::Reactivada
        );

        let grupo = repo_alertas::get_group(&conn, &id).unwrap().unwrap();
        assert_eq!(grupo.status, AlertStatus::Active);
        assert_eq!(grupo.cycle, 2);
    }
}
