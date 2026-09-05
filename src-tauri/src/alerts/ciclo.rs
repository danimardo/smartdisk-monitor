//! Ciclo de vida por acción del usuario: reconocer, silenciar y archivar
//! (`docs/ui-contract.md` §3.4, `alert-rules.md` §1).
//!
//! El silencio es **ortogonal** al estado: nunca decide si un grupo cuenta para la salud del
//! dispositivo, solo si notifica. Esa es la propiedad que este módulo existe para no romper por
//! accidente — `repo_alertas::list_groups_counting_toward_health` es la única implementación de
//! qué cuenta, y ni reconocer ni silenciar la tocan.

use time::{Duration, OffsetDateTime};

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
}
