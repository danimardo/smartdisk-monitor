//! Repositorio de `alert_groups` y `alert_occurrences` (`docs/data-model.md` §2).
//!
//! El repositorio persiste lo que `alerts/` decide; no evalúa reglas ni decide severidad. Ninguna
//! función de aquí calcula el estado de salud de un dispositivo: eso vive en `domain::salud` y
//! cuenta `active` + `acknowledged`, nunca al revés (constitución §I).

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::tipos::{AlertGroup, AlertOccurrence, AlertSeverity, AlertStatus};

fn severity_to_str(v: AlertSeverity) -> &'static str {
    match v {
        AlertSeverity::Warning => "warning",
        AlertSeverity::Critical => "critical",
    }
}

fn severity_from_str(s: &str) -> AlertSeverity {
    match s {
        "critical" => AlertSeverity::Critical,
        _ => AlertSeverity::Warning,
    }
}

fn status_to_str(v: AlertStatus) -> &'static str {
    match v {
        AlertStatus::Active => "active",
        AlertStatus::Acknowledged => "acknowledged",
        AlertStatus::Resolved => "resolved",
        AlertStatus::Archived => "archived",
    }
}

fn status_from_str(s: &str) -> AlertStatus {
    match s {
        "acknowledged" => AlertStatus::Acknowledged,
        "resolved" => AlertStatus::Resolved,
        "archived" => AlertStatus::Archived,
        _ => AlertStatus::Active,
    }
}

fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<AlertGroup> {
    Ok(AlertGroup {
        id: row.get("id")?,
        deduplication_key: row.get("deduplication_key")?,
        rule_key: row.get("rule_key")?,
        target_device_id: row.get("target_device_id")?,
        target_volume_id: row.get("target_volume_id")?,
        severity: severity_from_str(&row.get::<_, String>("severity")?),
        status: status_from_str(&row.get::<_, String>("status")?),
        muted_until: row.get("muted_until")?,
        cycle: row.get("cycle")?,
        first_occurrence_at_utc: row.get("first_occurrence_at_utc")?,
        last_occurrence_at_utc: row.get("last_occurrence_at_utc")?,
        occurrence_count: row.get("occurrence_count")?,
        acknowledged_at_utc: row.get("acknowledged_at_utc")?,
        resolved_at_utc: row.get("resolved_at_utc")?,
        archived_at_utc: row.get("archived_at_utc")?,
        last_value_real: row.get("last_value_real")?,
        context_json: row.get("context_json")?,
    })
}

pub fn get_group_by_dedup_key(
    conn: &Connection,
    deduplication_key: &str,
) -> rusqlite::Result<Option<AlertGroup>> {
    conn.query_row(
        "SELECT * FROM alert_groups WHERE deduplication_key = ?1",
        params![deduplication_key],
        row_to_group,
    )
    .optional()
}

pub fn get_group(conn: &Connection, id: &str) -> rusqlite::Result<Option<AlertGroup>> {
    conn.query_row(
        "SELECT * FROM alert_groups WHERE id = ?1",
        params![id],
        row_to_group,
    )
    .optional()
}

/// Grupos activos o reconocidos: los que cuentan para el estado de salud
/// (constitución §I — reconocer no apaga el color).
pub fn list_groups_counting_toward_health(conn: &Connection) -> rusqlite::Result<Vec<AlertGroup>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM alert_groups WHERE status IN ('active', 'acknowledged') ORDER BY severity DESC",
    )?;
    let filas = stmt.query_map([], row_to_group)?;
    filas.collect()
}

pub fn list_groups(conn: &Connection) -> rusqlite::Result<Vec<AlertGroup>> {
    let mut stmt =
        conn.prepare("SELECT * FROM alert_groups ORDER BY last_occurrence_at_utc DESC")?;
    let filas = stmt.query_map([], row_to_group)?;
    filas.collect()
}

/// Crea un grupo nuevo, con su primera fila en `alert_occurrences`: sin ella, la cronología del
/// grupo (`docs/ui-design.md` §7, "Alertas") empezaría vacía aunque acabe de ocurrir algo. Un
/// grupo con la misma `deduplication_key` que ya existe no se crea aquí: eso lo decide
/// `alerts::agrupacion`, que llama a `record_occurrence` en su lugar.
pub fn create_group(conn: &Connection, g: &AlertGroup) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO alert_groups (
            id, deduplication_key, rule_key, target_device_id, target_volume_id, severity, status,
            muted_until, cycle, first_occurrence_at_utc, last_occurrence_at_utc, occurrence_count,
            acknowledged_at_utc, resolved_at_utc, archived_at_utc, last_value_real, context_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            g.id,
            g.deduplication_key,
            g.rule_key,
            g.target_device_id,
            g.target_volume_id,
            severity_to_str(g.severity),
            status_to_str(g.status),
            g.muted_until,
            g.cycle,
            g.first_occurrence_at_utc,
            g.last_occurrence_at_utc,
            g.occurrence_count,
            g.acknowledged_at_utc,
            g.resolved_at_utc,
            g.archived_at_utc,
            g.last_value_real,
            g.context_json,
        ],
    )?;
    tx.execute(
        "INSERT INTO alert_occurrences (alert_group_id, cycle, occurred_at_utc, value_real, context_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            g.id,
            g.cycle,
            g.first_occurrence_at_utc,
            g.last_value_real,
            g.context_json,
        ],
    )?;
    tx.commit()
}

/// Registra una ocurrencia nueva sobre un grupo existente: incrementa el contador y actualiza la
/// última fecha y valor, sin crear un grupo nuevo (SC-004: N repeticiones, un solo grupo).
pub fn record_occurrence(
    conn: &Connection,
    alert_group_id: &str,
    cycle: i64,
    occurred_at_utc: &str,
    value_real: Option<f64>,
    context_json: Option<&str>,
) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO alert_occurrences (alert_group_id, cycle, occurred_at_utc, value_real, context_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![alert_group_id, cycle, occurred_at_utc, value_real, context_json],
    )?;
    tx.execute(
        "UPDATE alert_groups SET
            occurrence_count = occurrence_count + 1,
            last_occurrence_at_utc = ?2,
            last_value_real = ?3
         WHERE id = ?1",
        params![alert_group_id, occurred_at_utc, value_real],
    )?;
    tx.commit()
}

pub fn set_status(
    conn: &Connection,
    id: &str,
    status: AlertStatus,
    when_utc: &str,
) -> rusqlite::Result<()> {
    match status {
        AlertStatus::Acknowledged => conn.execute(
            "UPDATE alert_groups SET status = ?2, acknowledged_at_utc = ?3 WHERE id = ?1",
            params![id, status_to_str(status), when_utc],
        ),
        AlertStatus::Resolved => conn.execute(
            "UPDATE alert_groups SET status = ?2, resolved_at_utc = ?3 WHERE id = ?1",
            params![id, status_to_str(status), when_utc],
        ),
        AlertStatus::Archived => conn.execute(
            "UPDATE alert_groups SET status = ?2, archived_at_utc = ?3 WHERE id = ?1",
            params![id, status_to_str(status), when_utc],
        ),
        // Reactivar (recaída desde `acknowledged` por subida de severidad) no toca ninguna fecha
        // propia: solo `resolved_at_utc`/`archived_at_utc` se limpian, porque una alerta activa
        // de nuevo ya no está resuelta ni archivada. `when_utc` no aplica a esta transición.
        AlertStatus::Active => conn.execute(
            "UPDATE alert_groups SET status = ?2, resolved_at_utc = NULL, archived_at_utc = NULL WHERE id = ?1",
            params![id, status_to_str(status)],
        ),
    }?;
    Ok(())
}

/// Actualiza la severidad de un grupo ya existente: una regla puede escalar o desescalar sin que
/// cambie su identidad ni su cronología (`alert-rules.md` §1, tabla de transiciones).
pub fn set_severity(conn: &Connection, id: &str, severity: AlertSeverity) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE alert_groups SET severity = ?2 WHERE id = ?1",
        params![id, severity_to_str(severity)],
    )?;
    Ok(())
}

pub fn set_muted_until(
    conn: &Connection,
    id: &str,
    muted_until: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE alert_groups SET muted_until = ?2 WHERE id = ?1",
        params![id, muted_until],
    )?;
    Ok(())
}

/// Recae un grupo previamente resuelto: incrementa `cycle` en vez de crear un grupo nuevo, para no
/// perder el contador histórico (`docs/data-model.md` §2, `alert_groups.cycle`), y registra la
/// primera ocurrencia del ciclo nuevo en `alert_occurrences` — sin esa fila la cronología saltaría
/// del último ciclo resuelto al segundo suceso del nuevo, con el primero desaparecido.
pub fn reopen_as_new_cycle(
    conn: &Connection,
    id: &str,
    occurred_at_utc: &str,
    value_real: Option<f64>,
) -> rusqlite::Result<i64> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE alert_groups SET
            cycle = cycle + 1,
            status = 'active',
            resolved_at_utc = NULL,
            acknowledged_at_utc = NULL,
            first_occurrence_at_utc = ?2,
            last_occurrence_at_utc = ?2,
            occurrence_count = occurrence_count + 1,
            last_value_real = ?3
         WHERE id = ?1",
        params![id, occurred_at_utc, value_real],
    )?;
    let nuevo_ciclo = tx.query_row(
        "SELECT cycle FROM alert_groups WHERE id = ?1",
        params![id],
        |r| r.get(0),
    )?;
    tx.execute(
        "INSERT INTO alert_occurrences (alert_group_id, cycle, occurred_at_utc, value_real)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, nuevo_ciclo, occurred_at_utc, value_real],
    )?;
    tx.commit()?;
    Ok(nuevo_ciclo)
}

/// Cronología completa de un grupo, de más reciente a más antigua (`docs/ui-design.md` §7,
/// "Alertas": lista de `AlertCard` + detalle con cronología de ocurrencias).
pub fn list_occurrences(
    conn: &Connection,
    alert_group_id: &str,
) -> rusqlite::Result<Vec<AlertOccurrence>> {
    let mut stmt = conn.prepare(
        "SELECT occurred_at_utc, cycle, value_real, triggering_event_id, context_json
         FROM alert_occurrences WHERE alert_group_id = ?1
         ORDER BY occurred_at_utc DESC, id DESC",
    )?;
    let filas = stmt.query_map(params![alert_group_id], |row| {
        Ok(AlertOccurrence {
            occurred_at_utc: row.get(0)?,
            cycle: row.get(1)?,
            value_real: row.get(2)?,
            triggering_event_id: row.get(3)?,
            context_json: row.get(4)?,
        })
    })?;
    filas.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("repo_alertas");
        let conn = db::open(&dir).unwrap().0;
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn grupo(id: &str, dedup: &str, status: AlertStatus) -> AlertGroup {
        AlertGroup {
            id: id.to_string(),
            deduplication_key: dedup.to_string(),
            rule_key: "temperature_high".to_string(),
            target_device_id: Some("d1".to_string()),
            target_volume_id: None,
            severity: AlertSeverity::Critical,
            status,
            muted_until: None,
            cycle: 1,
            first_occurrence_at_utc: "2026-09-04T10:00:00Z".to_string(),
            last_occurrence_at_utc: "2026-09-04T10:00:00Z".to_string(),
            occurrence_count: 1,
            acknowledged_at_utc: None,
            resolved_at_utc: None,
            archived_at_utc: None,
            last_value_real: Some(70.0),
            context_json: None,
        }
    }

    #[test]
    fn repetir_la_condicion_incrementa_el_contador_de_un_solo_grupo() {
        let conn = conn_de_prueba();
        create_group(&conn, &grupo("g1", "dedup-1", AlertStatus::Active)).unwrap();

        record_occurrence(&conn, "g1", 1, "2026-09-04T10:05:00Z", Some(71.0), None).unwrap();
        record_occurrence(&conn, "g1", 1, "2026-09-04T10:10:00Z", Some(72.0), None).unwrap();

        assert_eq!(
            list_groups(&conn).unwrap().len(),
            1,
            "debe seguir siendo un único grupo"
        );
        let g = get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(g.occurrence_count, 3);
        assert_eq!(g.last_value_real, Some(72.0));
    }

    #[test]
    fn reconocer_no_lo_saca_de_lo_que_cuenta_para_salud() {
        let conn = conn_de_prueba();
        create_group(&conn, &grupo("g1", "dedup-2", AlertStatus::Active)).unwrap();

        set_status(
            &conn,
            "g1",
            AlertStatus::Acknowledged,
            "2026-09-04T11:00:00Z",
        )
        .unwrap();

        let cuentan = list_groups_counting_toward_health(&conn).unwrap();
        assert_eq!(cuentan.len(), 1, "reconocido sigue contando para la salud");
        assert_eq!(cuentan[0].status, AlertStatus::Acknowledged);
    }

    #[test]
    fn resuelto_deja_de_contar_para_salud() {
        let conn = conn_de_prueba();
        create_group(&conn, &grupo("g1", "dedup-3", AlertStatus::Active)).unwrap();
        set_status(&conn, "g1", AlertStatus::Resolved, "2026-09-04T12:00:00Z").unwrap();

        assert!(list_groups_counting_toward_health(&conn)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn recaer_incrementa_el_ciclo_en_vez_de_crear_grupo_nuevo() {
        let conn = conn_de_prueba();
        create_group(&conn, &grupo("g1", "dedup-4", AlertStatus::Resolved)).unwrap();

        let nuevo_ciclo =
            reopen_as_new_cycle(&conn, "g1", "2026-09-05T00:00:00Z", Some(42.0)).unwrap();

        assert_eq!(nuevo_ciclo, 2);
        assert_eq!(list_groups(&conn).unwrap().len(), 1);
        let grupo = get_group(&conn, "g1").unwrap().unwrap();
        assert_eq!(grupo.status, AlertStatus::Active);
        assert_eq!(grupo.last_value_real, Some(42.0));
    }

    #[test]
    fn list_occurrences_incluye_la_primera_ocurrencia_y_la_del_nuevo_ciclo() {
        let conn = conn_de_prueba();
        create_group(&conn, &grupo("g1", "dedup-5", AlertStatus::Resolved)).unwrap();
        reopen_as_new_cycle(&conn, "g1", "2026-09-05T00:00:00Z", Some(42.0)).unwrap();

        let ocurrencias = list_occurrences(&conn, "g1").unwrap();
        assert_eq!(
            ocurrencias.len(),
            2,
            "la primera ocurrencia del grupo y la del nuevo ciclo, sin ninguna perdida"
        );
        assert_eq!(ocurrencias[0].cycle, 2, "la más reciente va primero");
        assert_eq!(ocurrencias[1].cycle, 1);
    }

    #[test]
    fn list_occurrences_de_un_grupo_sin_ocurrencias_esta_vacia() {
        let conn = conn_de_prueba();
        assert!(list_occurrences(&conn, "no-existe").unwrap().is_empty());
    }
}
