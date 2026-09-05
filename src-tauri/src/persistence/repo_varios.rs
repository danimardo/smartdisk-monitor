//! Repositorio de `system_events`, `event_cursors`, `test_runs` y `settings`
//! (`docs/data-model.md` §2).

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::tipos::{
    EventLevel, MappingConfidence, SystemEvent, TestRun, TestStatus, TestType,
};

// ---- system_events -------------------------------------------------------------------------

fn level_to_str(v: EventLevel) -> &'static str {
    match v {
        EventLevel::Critical => "critical",
        EventLevel::Error => "error",
        EventLevel::Warning => "warning",
        EventLevel::Information => "information",
    }
}

fn level_from_str(s: &str) -> EventLevel {
    match s {
        "critical" => EventLevel::Critical,
        "error" => EventLevel::Error,
        "warning" => EventLevel::Warning,
        _ => EventLevel::Information,
    }
}

fn mapping_confidence_to_str(v: MappingConfidence) -> &'static str {
    match v {
        MappingConfidence::Exact => "exact",
        MappingConfidence::Inferred => "inferred",
        MappingConfidence::Unknown => "unknown",
    }
}

fn mapping_confidence_from_str(s: &str) -> MappingConfidence {
    match s {
        "exact" => MappingConfidence::Exact,
        "inferred" => MappingConfidence::Inferred,
        _ => MappingConfidence::Unknown,
    }
}

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<SystemEvent> {
    Ok(SystemEvent {
        id: row.get("id")?,
        channel: row.get("channel")?,
        record_id: row.get("record_id")?,
        occurred_at_utc: row.get("occurred_at_utc")?,
        provider: row.get("provider")?,
        event_id: row.get("event_id")?,
        level: level_from_str(&row.get::<_, String>("level")?),
        message: row.get("message")?,
        raw_xml: row.get("raw_xml")?,
        device_id: row.get("device_id")?,
        volume_id: row.get("volume_id")?,
        mapping_confidence: mapping_confidence_from_str(
            &row.get::<_, String>("mapping_confidence")?,
        ),
        dedup_hash: row.get("dedup_hash")?,
    })
}

/// Inserta un evento. `(channel, record_id)` es su identidad: si ya existe, no se duplica —
/// `INSERT OR IGNORE` es correcto aquí porque cerrar y reabrir la aplicación relee desde el cursor
/// sin volver a insertar lo ya visto, y esto es la última red de seguridad (FR-017).
pub fn insert_event_if_new(conn: &Connection, e: &SystemEvent) -> rusqlite::Result<bool> {
    let filas = conn.execute(
        "INSERT OR IGNORE INTO system_events (
            channel, record_id, occurred_at_utc, provider, event_id, level, message, raw_xml,
            device_id, volume_id, mapping_confidence, dedup_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            e.channel,
            e.record_id,
            e.occurred_at_utc,
            e.provider,
            e.event_id,
            level_to_str(e.level),
            e.message,
            e.raw_xml,
            e.device_id,
            e.volume_id,
            mapping_confidence_to_str(e.mapping_confidence),
            e.dedup_hash,
        ],
    )?;
    Ok(filas > 0)
}

pub fn list_events_for_device(
    conn: &Connection,
    device_id: &str,
    limit: i64,
) -> rusqlite::Result<Vec<SystemEvent>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM system_events WHERE device_id = ?1 ORDER BY occurred_at_utc DESC LIMIT ?2",
    )?;
    let filas = stmt.query_map(params![device_id, limit], row_to_event)?;
    filas.collect()
}

/// Un evento por su `id` autoincremento (no `(channel, record_id)`): lo que usa
/// `get_event_raw_xml`, que recibe el id que la lista ya entregó.
pub fn get_event_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<SystemEvent>> {
    conn.query_row(
        "SELECT * FROM system_events WHERE id = ?1",
        params![id],
        row_to_event,
    )
    .optional()
}

/// Filtros de `get_system_events` (`docs/ui-contract.md` §3.5). `cursor` es el par
/// `occurred_at_utc|id` del último evento de la página anterior (paginación por conjunto de
/// claves, más reciente primero): sin él se empieza desde el principio.
#[derive(Debug, Clone, Default)]
pub struct FiltroEventos<'a> {
    pub device_id: Option<&'a str>,
    pub volume_id: Option<&'a str>,
    pub levels: Option<&'a [EventLevel]>,
    pub providers: Option<&'a [String]>,
    pub from_utc: Option<&'a str>,
    pub to_utc: Option<&'a str>,
    pub cursor: Option<(&'a str, i64)>,
    pub limit: i64,
}

/// Construye el `WHERE` dinámico y sus parámetros. Solo la presencia de cada cláusula es
/// dinámica; los valores siempre se enlazan con parámetros, nunca se interpolan en el SQL.
fn construir_where(f: &FiltroEventos) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut clausulas: Vec<String> = Vec::new();
    let mut valores: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(id) = f.device_id {
        clausulas.push("device_id = ?".to_string());
        valores.push(Box::new(id.to_string()));
    }
    if let Some(id) = f.volume_id {
        clausulas.push("volume_id = ?".to_string());
        valores.push(Box::new(id.to_string()));
    }
    if let Some(niveles) = f.levels {
        let marcadores = vec!["?"; niveles.len()].join(", ");
        clausulas.push(format!("level IN ({marcadores})"));
        for nivel in niveles {
            valores.push(Box::new(level_to_str(*nivel).to_string()));
        }
    }
    if let Some(proveedores) = f.providers {
        let marcadores = vec!["?"; proveedores.len()].join(", ");
        clausulas.push(format!("provider IN ({marcadores})"));
        for p in proveedores {
            valores.push(Box::new(p.clone()));
        }
    }
    if let Some(desde) = f.from_utc {
        clausulas.push("occurred_at_utc >= ?".to_string());
        valores.push(Box::new(desde.to_string()));
    }
    if let Some(hasta) = f.to_utc {
        clausulas.push("occurred_at_utc <= ?".to_string());
        valores.push(Box::new(hasta.to_string()));
    }
    if let Some((cursor_tiempo, cursor_id)) = f.cursor {
        // Estrictamente anterior al último evento de la página previa: en tiempo, o en el mismo
        // instante pero con id menor (dos eventos pueden compartir `occurred_at_utc`).
        clausulas.push("(occurred_at_utc < ? OR (occurred_at_utc = ? AND id < ?))".to_string());
        valores.push(Box::new(cursor_tiempo.to_string()));
        valores.push(Box::new(cursor_tiempo.to_string()));
        valores.push(Box::new(cursor_id));
    }

    let where_sql = if clausulas.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clausulas.join(" AND "))
    };
    (where_sql, valores)
}

/// Página de eventos según el filtro, más recientes primero. `limit + 1` filas se piden para
/// saber si hay una página siguiente sin una segunda consulta `COUNT`.
pub fn list_events(
    conn: &Connection,
    filtro: &FiltroEventos,
) -> rusqlite::Result<Vec<SystemEvent>> {
    let (where_sql, valores) = construir_where(filtro);
    let sql = format!(
        "SELECT * FROM system_events {where_sql} ORDER BY occurred_at_utc DESC, id DESC LIMIT ?"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut parametros: Vec<&dyn rusqlite::ToSql> = valores.iter().map(|v| v.as_ref()).collect();
    let limite = filtro.limit + 1;
    parametros.push(&limite);
    let filas = stmt.query_map(parametros.as_slice(), row_to_event)?;
    filas.collect()
}

/// Total de eventos que cumplen el filtro, ignorando `cursor` y `limit` (el total es del
/// conjunto completo, no de la página).
pub fn count_events(conn: &Connection, filtro: &FiltroEventos) -> rusqlite::Result<i64> {
    let mut sin_paginacion = filtro.clone();
    sin_paginacion.cursor = None;
    let (where_sql, valores) = construir_where(&sin_paginacion);
    let sql = format!("SELECT COUNT(*) FROM system_events {where_sql}");
    let mut stmt = conn.prepare(&sql)?;
    let parametros: Vec<&dyn rusqlite::ToSql> = valores.iter().map(|v| v.as_ref()).collect();
    stmt.query_row(parametros.as_slice(), |r| r.get(0))
}

// ---- event_cursors --------------------------------------------------------------------------

/// Bookmark del canal, no un `RecordId` suelto: si el canal se limpia, los identificadores se
/// reinician y un cursor numérico dejaría de importar eventos nuevos sin avisar
/// (`docs/open-questions.md` J.7).
pub fn get_cursor(conn: &Connection, channel: &str) -> rusqlite::Result<Option<Vec<u8>>> {
    conn.query_row(
        "SELECT bookmark_blob FROM event_cursors WHERE channel = ?1",
        params![channel],
        |r| r.get(0),
    )
    .optional()
}

pub fn set_cursor(
    conn: &Connection,
    channel: &str,
    bookmark: &[u8],
    updated_at_utc: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO event_cursors (channel, bookmark_blob, updated_at_utc) VALUES (?1, ?2, ?3)
         ON CONFLICT(channel) DO UPDATE SET bookmark_blob = excluded.bookmark_blob, updated_at_utc = excluded.updated_at_utc",
        params![channel, bookmark, updated_at_utc],
    )?;
    Ok(())
}

// ---- test_runs -------------------------------------------------------------------------------

pub fn test_type_to_str(v: TestType) -> &'static str {
    match v {
        TestType::Benchmark => "benchmark",
        TestType::ChkdskScan => "chkdsk_scan",
        TestType::SmartShort => "smart_short",
    }
}

fn test_type_from_str(s: &str) -> TestType {
    match s {
        "chkdsk_scan" => TestType::ChkdskScan,
        "smart_short" => TestType::SmartShort,
        _ => TestType::Benchmark,
    }
}

pub fn test_status_to_str(v: TestStatus) -> &'static str {
    match v {
        TestStatus::Pending => "pending",
        TestStatus::Running => "running",
        TestStatus::Cancelling => "cancelling",
        TestStatus::Completed => "completed",
        TestStatus::Failed => "failed",
        TestStatus::Cancelled => "cancelled",
        TestStatus::Interrupted => "interrupted",
    }
}

fn test_status_from_str(s: &str) -> TestStatus {
    match s {
        "running" => TestStatus::Running,
        "cancelling" => TestStatus::Cancelling,
        "completed" => TestStatus::Completed,
        "failed" => TestStatus::Failed,
        "cancelled" => TestStatus::Cancelled,
        "interrupted" => TestStatus::Interrupted,
        _ => TestStatus::Pending,
    }
}

fn row_to_test_run(row: &rusqlite::Row) -> rusqlite::Result<TestRun> {
    Ok(TestRun {
        id: row.get("id")?,
        test_type: test_type_from_str(&row.get::<_, String>("test_type")?),
        target_device_id: row.get("target_device_id")?,
        target_volume_id: row.get("target_volume_id")?,
        status: test_status_from_str(&row.get::<_, String>("status")?),
        started_at_utc: row.get("started_at_utc")?,
        finished_at_utc: row.get("finished_at_utc")?,
        progress_percent: row.get("progress_percent")?,
        result_summary_json: row.get("result_summary_json")?,
        parameters_json: row.get("parameters_json")?,
        temp_path: row.get("temp_path")?,
    })
}

pub fn create_test_run(conn: &Connection, t: &TestRun) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO test_runs (
            id, test_type, target_device_id, target_volume_id, status, started_at_utc,
            finished_at_utc, progress_percent, result_summary_json, parameters_json, temp_path
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            t.id,
            test_type_to_str(t.test_type),
            t.target_device_id,
            t.target_volume_id,
            test_status_to_str(t.status),
            t.started_at_utc,
            t.finished_at_utc,
            t.progress_percent,
            t.result_summary_json,
            t.parameters_json,
            t.temp_path,
        ],
    )?;
    Ok(())
}

pub fn update_test_run_status(
    conn: &Connection,
    id: &str,
    status: TestStatus,
    progress_percent: Option<i64>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE test_runs SET status = ?2, progress_percent = ?3 WHERE id = ?1",
        params![id, test_status_to_str(status), progress_percent],
    )?;
    Ok(())
}

pub fn list_test_runs(
    conn: &Connection,
    device_id: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<TestRun>> {
    match device_id {
        Some(id) => {
            let mut stmt = conn.prepare(
                "SELECT * FROM test_runs WHERE target_device_id = ?1
                 ORDER BY started_at_utc DESC LIMIT ?2",
            )?;
            let filas = stmt.query_map(params![id, limit], row_to_test_run)?;
            filas.collect()
        }
        None => {
            let mut stmt =
                conn.prepare("SELECT * FROM test_runs ORDER BY started_at_utc DESC LIMIT ?1")?;
            let filas = stmt.query_map(params![limit], row_to_test_run)?;
            filas.collect()
        }
    }
}

pub fn get_test_run(conn: &Connection, id: &str) -> rusqlite::Result<Option<TestRun>> {
    conn.query_row(
        "SELECT * FROM test_runs WHERE id = ?1",
        params![id],
        row_to_test_run,
    )
    .optional()
}

/// `test.busy` (`docs/ui-contract.md` §1, `docs/open-questions.md` J.29): ¿hay ya una prueba no
/// terminada sobre alguno de estos dispositivos o volúmenes? El llamador pasa el objetivo directo
/// **y** los discos/volúmenes enlazados, para que la exclusión sea por disco físico subyacente y
/// no solo por el id exacto recibido (cubre también "el autotest no corre junto al benchmark").
pub fn has_active_test_run(
    conn: &Connection,
    device_ids: &[String],
    volume_ids: &[String],
) -> rusqlite::Result<bool> {
    if device_ids.is_empty() && volume_ids.is_empty() {
        return Ok(false);
    }
    let device_placeholders = std::iter::repeat("?")
        .take(device_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let volume_placeholders = std::iter::repeat("?")
        .take(volume_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT COUNT(*) FROM test_runs
         WHERE status IN ('pending', 'running', 'cancelling')
           AND (target_device_id IN ({device_placeholders})
                OR target_volume_id IN ({volume_placeholders}))"
    );
    let mut stmt = conn.prepare(&sql)?;
    let parametros: Vec<&dyn rusqlite::ToSql> = device_ids
        .iter()
        .map(|d| d as &dyn rusqlite::ToSql)
        .chain(volume_ids.iter().map(|v| v as &dyn rusqlite::ToSql))
        .collect();
    let count: i64 = stmt.query_row(parametros.as_slice(), |r| r.get(0))?;
    Ok(count > 0)
}

/// Cierre final de una prueba: estado terminal, fin, resumen y progreso al 100 %.
/// `temp_path` se pasa de nuevo explícitamente (no se conserva el de la fila) porque es aquí donde
/// se decide si el archivo quedó huérfano (`docs/open-questions.md` J.29).
pub fn finish_test_run(
    conn: &Connection,
    id: &str,
    status: TestStatus,
    finished_at_utc: &str,
    result_summary_json: Option<&str>,
    temp_path: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE test_runs
         SET status = ?2, finished_at_utc = ?3, progress_percent = 100,
             result_summary_json = ?4, temp_path = ?5
         WHERE id = ?1",
        params![
            id,
            test_status_to_str(status),
            finished_at_utc,
            result_summary_json,
            temp_path,
        ],
    )?;
    Ok(())
}

// ---- settings --------------------------------------------------------------------------------

/// Claves tipadas, no un diccionario libre (`docs/ui-contract.md` §3.1): el valor viaja como JSON
/// y quien lo lee sabe a qué tipo deserializarlo porque conoce la clave.
pub fn get_setting_raw(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value_json FROM settings WHERE key = ?1",
        params![key],
        |r| r.get(0),
    )
    .optional()
}

pub fn set_setting_raw(
    conn: &Connection,
    key: &str,
    value_json: &str,
    updated_at_utc: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value_json, updated_at_utc) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at_utc = excluded.updated_at_utc",
        params![key, value_json, updated_at_utc],
    )?;
    Ok(())
}

/// Todas las claves de `settings`, para el ZIP de diagnóstico (T090): el propio ajuste, no una
/// vista parcial.
pub fn list_settings(conn: &Connection) -> rusqlite::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT key, value_json FROM settings ORDER BY key")?;
    let filas = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    filas.collect()
}

/// Borra una clave: `reset_settings` (T097) vuelve así a su valor de fábrica sin tener que
/// conocerlo aquí — quien lee con `get_setting_raw`/las funciones `leer_ajuste_*` de `commands`
/// ya cae al valor por defecto cuando la clave no existe.
pub fn delete_setting(conn: &Connection, key: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("repo_varios");
        let conn = db::open(&dir).unwrap().0;
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn evento(record_id: i64) -> SystemEvent {
        SystemEvent {
            id: 0,
            channel: "System".to_string(),
            record_id,
            occurred_at_utc: "2026-09-04T10:00:00Z".to_string(),
            provider: "disk".to_string(),
            event_id: 51,
            level: EventLevel::Warning,
            message: Some("Un mensaje del sistema".to_string()),
            raw_xml: None,
            device_id: None,
            volume_id: None,
            mapping_confidence: MappingConfidence::Unknown,
            dedup_hash: format!("hash-{record_id}"),
        }
    }

    #[test]
    fn reabrir_no_duplica_el_mismo_evento() {
        let conn = conn_de_prueba();
        assert!(insert_event_if_new(&conn, &evento(1)).unwrap());
        assert!(
            !insert_event_if_new(&conn, &evento(1)).unwrap(),
            "el segundo intento no inserta"
        );

        let cuenta: i64 = conn
            .query_row("SELECT COUNT(*) FROM system_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cuenta, 1);
    }

    #[test]
    fn list_events_sin_filtro_devuelve_todos_del_mas_reciente_al_mas_antiguo() {
        let conn = conn_de_prueba();
        let mut e1 = evento(1);
        e1.occurred_at_utc = "2026-09-04T10:00:00Z".to_string();
        let mut e2 = evento(2);
        e2.occurred_at_utc = "2026-09-04T11:00:00Z".to_string();
        insert_event_if_new(&conn, &e1).unwrap();
        insert_event_if_new(&conn, &e2).unwrap();

        let filtro = FiltroEventos {
            limit: 10,
            ..Default::default()
        };
        let eventos = list_events(&conn, &filtro).unwrap();
        assert_eq!(eventos.len(), 2);
        assert_eq!(eventos[0].record_id, 2, "el más reciente primero");
    }

    #[test]
    fn list_events_filtra_por_nivel_y_por_proveedor() {
        let conn = conn_de_prueba();
        let mut critico = evento(1);
        critico.level = EventLevel::Error;
        critico.provider = "Ntfs".to_string();
        insert_event_if_new(&conn, &critico).unwrap();
        insert_event_if_new(&conn, &evento(2)).unwrap(); // Warning, "disk"

        let solo_error = FiltroEventos {
            levels: Some(&[EventLevel::Error]),
            limit: 10,
            ..Default::default()
        };
        assert_eq!(list_events(&conn, &solo_error).unwrap().len(), 1);

        let solo_ntfs = FiltroEventos {
            providers: Some(&["Ntfs".to_string()]),
            limit: 10,
            ..Default::default()
        };
        assert_eq!(list_events(&conn, &solo_ntfs).unwrap().len(), 1);
    }

    #[test]
    fn list_events_respeta_el_cursor_de_paginacion() {
        let conn = conn_de_prueba();
        for i in 1..=3 {
            let mut e = evento(i);
            e.occurred_at_utc = format!("2026-09-04T1{i}:00:00Z");
            insert_event_if_new(&conn, &e).unwrap();
        }

        let primera_pagina = FiltroEventos {
            limit: 1,
            ..Default::default()
        };
        let pagina1 = list_events(&conn, &primera_pagina).unwrap();
        assert_eq!(
            pagina1.len(),
            2,
            "limit+1 para saber si hay página siguiente"
        );
        assert_eq!(pagina1[0].record_id, 3);

        let siguiente = FiltroEventos {
            cursor: Some((&pagina1[0].occurred_at_utc, pagina1[0].id)),
            limit: 10,
            ..Default::default()
        };
        let pagina2 = list_events(&conn, &siguiente).unwrap();
        assert_eq!(pagina2.len(), 2);
        assert_eq!(pagina2[0].record_id, 2);
        assert_eq!(pagina2[1].record_id, 1);
    }

    #[test]
    fn count_events_ignora_el_cursor_pero_respeta_los_demas_filtros() {
        let conn = conn_de_prueba();
        insert_event_if_new(&conn, &evento(1)).unwrap();
        insert_event_if_new(&conn, &evento(2)).unwrap();

        let filtro = FiltroEventos {
            cursor: Some(("2026-09-04T10:00:00Z", 1)),
            limit: 1,
            ..Default::default()
        };
        assert_eq!(count_events(&conn, &filtro).unwrap(), 2);
    }

    #[test]
    fn get_event_by_id_devuelve_el_evento_o_ninguno() {
        let conn = conn_de_prueba();
        insert_event_if_new(&conn, &evento(1)).unwrap();
        let listado = list_events(
            &conn,
            &FiltroEventos {
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
        let id = listado[0].id;

        assert_eq!(get_event_by_id(&conn, id).unwrap().unwrap().record_id, 1);
        assert!(get_event_by_id(&conn, id + 999).unwrap().is_none());
    }

    #[test]
    fn el_cursor_persiste_como_bookmark_binario() {
        let conn = conn_de_prueba();
        assert!(get_cursor(&conn, "System").unwrap().is_none());

        set_cursor(&conn, "System", &[1, 2, 3], "2026-09-04T10:00:00Z").unwrap();
        assert_eq!(get_cursor(&conn, "System").unwrap(), Some(vec![1, 2, 3]));

        set_cursor(&conn, "System", &[4, 5], "2026-09-04T11:00:00Z").unwrap();
        assert_eq!(get_cursor(&conn, "System").unwrap(), Some(vec![4, 5]));
    }

    #[test]
    fn crea_y_actualiza_una_ejecucion_de_prueba() {
        let conn = conn_de_prueba();
        let t = TestRun {
            id: "t1".to_string(),
            test_type: TestType::Benchmark,
            target_device_id: Some("d1".to_string()),
            target_volume_id: None,
            status: TestStatus::Pending,
            started_at_utc: None,
            finished_at_utc: None,
            progress_percent: None,
            result_summary_json: None,
            parameters_json: None,
            temp_path: None,
        };
        create_test_run(&conn, &t).unwrap();
        update_test_run_status(&conn, "t1", TestStatus::Running, Some(40)).unwrap();

        let listado = list_test_runs(&conn, None, 10).unwrap();
        assert_eq!(listado.len(), 1);
        assert_eq!(listado[0].status, TestStatus::Running);
        assert_eq!(listado[0].progress_percent, Some(40));
    }

    #[test]
    fn settings_se_guarda_y_se_sobrescribe() {
        let conn = conn_de_prueba();
        assert!(get_setting_raw(&conn, "theme").unwrap().is_none());

        set_setting_raw(&conn, "theme", "\"dark\"", "2026-09-04T10:00:00Z").unwrap();
        assert_eq!(
            get_setting_raw(&conn, "theme").unwrap(),
            Some("\"dark\"".to_string())
        );

        set_setting_raw(&conn, "theme", "\"light\"", "2026-09-04T11:00:00Z").unwrap();
        assert_eq!(
            get_setting_raw(&conn, "theme").unwrap(),
            Some("\"light\"".to_string())
        );
    }
}
