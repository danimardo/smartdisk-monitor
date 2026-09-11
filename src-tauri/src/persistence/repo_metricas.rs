//! Repositorio de `metric_samples` y `smart_snapshots` (`docs/data-model.md` §2).
//!
//! Registra procedencia y antigüedad en cada muestra (FR-005): quien lea de aquí sabe siempre de
//! dónde salió el dato y cuándo se leyó.

use rusqlite::{params, Connection};

use crate::domain::tipos::{MetricQuality, MetricSample, MetricSource, MetricTarget, Resolution};

fn source_to_str(v: MetricSource) -> &'static str {
    match v {
        MetricSource::Smartctl => "smartctl",
        MetricSource::WindowsStorage => "windows_storage",
        MetricSource::PerformanceCounter => "performance_counter",
        MetricSource::Filesystem => "filesystem",
    }
}

fn source_from_str(s: &str) -> MetricSource {
    match s {
        "smartctl" => MetricSource::Smartctl,
        "windows_storage" => MetricSource::WindowsStorage,
        "performance_counter" => MetricSource::PerformanceCounter,
        _ => MetricSource::Filesystem,
    }
}

fn quality_to_str(v: MetricQuality) -> &'static str {
    match v {
        MetricQuality::Exact => "exact",
        MetricQuality::Inferred => "inferred",
        MetricQuality::VendorSpecific => "vendor_specific",
        MetricQuality::Stale => "stale",
    }
}

fn quality_from_str(s: &str) -> MetricQuality {
    match s {
        "exact" => MetricQuality::Exact,
        "inferred" => MetricQuality::Inferred,
        "vendor_specific" => MetricQuality::VendorSpecific,
        _ => MetricQuality::Stale,
    }
}

fn resolution_to_str(v: Resolution) -> &'static str {
    match v {
        Resolution::Raw => "raw",
        Resolution::FiveMinutes => "five_minutes",
        Resolution::Hourly => "hourly",
    }
}

fn resolution_from_str(s: &str) -> Resolution {
    match s {
        "five_minutes" => Resolution::FiveMinutes,
        "hourly" => Resolution::Hourly,
        _ => Resolution::Raw,
    }
}

/// Inserta una muestra. Un dato ausente **no se llama**: si `value_real` y `value_integer` son
/// `None`, la fila no representa ninguna medición real y no debe insertarse — se comprueba con un
/// `assert` de invariante, no silenciosamente.
pub fn insert_sample(conn: &Connection, s: &MetricSample) -> rusqlite::Result<()> {
    debug_assert!(
        s.value_real.is_some() || s.value_integer.is_some(),
        "una muestra sin ningún valor no debe insertarse: el dato ausente se omite, no se guarda"
    );

    let (device_id, volume_id): (Option<&str>, Option<&str>) = match &s.target {
        MetricTarget::Device(id) => (Some(id.as_str()), None),
        MetricTarget::Volume(id) => (None, Some(id.as_str())),
    };

    conn.execute(
        "INSERT INTO metric_samples (
            device_id, volume_id, metric_key, value_real, value_integer, unit, sampled_at_utc,
            source, quality, resolution
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            device_id,
            volume_id,
            s.metric_key,
            s.value_real,
            s.value_integer,
            s.unit,
            s.sampled_at_utc,
            source_to_str(s.source),
            quality_to_str(s.quality),
            resolution_to_str(s.resolution),
        ],
    )?;
    Ok(())
}

fn row_to_sample(row: &rusqlite::Row) -> rusqlite::Result<MetricSample> {
    let device_id: Option<String> = row.get("device_id")?;
    let volume_id: Option<String> = row.get("volume_id")?;
    let target = match (device_id, volume_id) {
        (Some(d), None) => MetricTarget::Device(d),
        (None, Some(v)) => MetricTarget::Volume(v),
        _ => unreachable!("el CHECK de la migración garantiza exactamente uno de los dos"),
    };
    Ok(MetricSample {
        target,
        metric_key: row.get("metric_key")?,
        value_real: row.get("value_real")?,
        value_integer: row.get("value_integer")?,
        unit: row.get("unit")?,
        sampled_at_utc: row.get("sampled_at_utc")?,
        source: source_from_str(&row.get::<_, String>("source")?),
        quality: quality_from_str(&row.get::<_, String>("quality")?),
        resolution: resolution_from_str(&row.get::<_, String>("resolution")?),
    })
}

/// Serie de un dispositivo entre dos instantes UTC, ambos incluidos, ordenada por tiempo. El
/// dominio (`domain/series.rs`) decide dónde van los huecos; este repositorio solo devuelve lo que
/// hay, sin inventar continuidad.
pub fn device_series(
    conn: &Connection,
    device_id: &str,
    metric_key: &str,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<MetricSample>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM metric_samples
         WHERE device_id = ?1 AND metric_key = ?2 AND sampled_at_utc BETWEEN ?3 AND ?4
         ORDER BY sampled_at_utc",
    )?;
    let filas = stmt.query_map(
        params![device_id, metric_key, from_utc, to_utc],
        row_to_sample,
    )?;
    filas.collect()
}

/// Igual que `device_series`, para un volumen. Hoy siempre vacía en la práctica: la capacidad se
/// guarda directamente en `volumes.capacity_bytes`/`free_bytes` (T059), no todavía como serie de
/// `metric_samples` — persistir una muestra periódica de capacidad es trabajo aparte, no cubierto
/// por esta tarea (`docs/open-questions.md`).
pub fn volume_series(
    conn: &Connection,
    volume_id: &str,
    metric_key: &str,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<MetricSample>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM metric_samples
         WHERE volume_id = ?1 AND metric_key = ?2 AND sampled_at_utc BETWEEN ?3 AND ?4
         ORDER BY sampled_at_utc",
    )?;
    let filas = stmt.query_map(
        params![volume_id, metric_key, from_utc, to_utc],
        row_to_sample,
    )?;
    filas.collect()
}

/// Qué métricas tienen algún dato de un dispositivo en el rango, en cualquier resolución
/// (crudo o agregado): un informe (T087) no puede asumir una lista fija de claves, y una métrica
/// cuyos crudos ya purgó la retención puede seguir teniendo agregados dentro del rango.
pub fn distinct_metric_keys(
    conn: &Connection,
    device_id: &str,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT metric_key FROM metric_samples
           WHERE device_id = ?1 AND sampled_at_utc BETWEEN ?2 AND ?3
         UNION
         SELECT metric_key FROM metric_aggregates
           WHERE device_id = ?1 AND bucket_start_utc BETWEEN ?2 AND ?3
         ORDER BY metric_key",
    )?;
    let filas = stmt.query_map(params![device_id, from_utc, to_utc], |r| {
        r.get::<_, String>(0)
    })?;
    filas.collect()
}

/// Las `n` muestras más recientes de una métrica, de más reciente a más antigua: la forma que
/// espera `alerts::motor` (histéresis sobre `&[f64]`, más reciente primero). Menos de `n` filas
/// simplemente devuelve las que haya — es `motor` quien decide que un contador incompleto no basta
/// para evaluar, no este repositorio.
pub fn latest_n_values(
    conn: &Connection,
    device_id: &str,
    metric_key: &str,
    n: u32,
) -> rusqlite::Result<Vec<f64>> {
    let mut stmt = conn.prepare(
        "SELECT value_real FROM metric_samples
         WHERE device_id = ?1 AND metric_key = ?2 AND value_real IS NOT NULL
         ORDER BY sampled_at_utc DESC LIMIT ?3",
    )?;
    let filas = stmt.query_map(params![device_id, metric_key, i64::from(n)], |row| {
        row.get(0)
    })?;
    filas.collect()
}

/// Igual que `latest_n_values` pero para una métrica de **volumen** (`capacity.low/critical` sobre
/// `volume_free_bytes`, v3).
pub fn latest_n_values_volume(
    conn: &Connection,
    volume_id: &str,
    metric_key: &str,
    n: u32,
) -> rusqlite::Result<Vec<f64>> {
    let mut stmt = conn.prepare(
        "SELECT value_real FROM metric_samples
         WHERE volume_id = ?1 AND metric_key = ?2 AND value_real IS NOT NULL
         ORDER BY sampled_at_utc DESC LIMIT ?3",
    )?;
    let filas = stmt.query_map(params![volume_id, metric_key, i64::from(n)], |row| {
        row.get(0)
    })?;
    filas.collect()
}

/// La última muestra conocida de una métrica, para calcular frescura (FR-005/FR-006): sin ella no
/// se puede saber si un dispositivo está `unknown` por falta de lecturas recientes.
pub fn latest_device_sample(
    conn: &Connection,
    device_id: &str,
    metric_key: &str,
) -> rusqlite::Result<Option<MetricSample>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT * FROM metric_samples
         WHERE device_id = ?1 AND metric_key = ?2
         ORDER BY sampled_at_utc DESC LIMIT 1",
        params![device_id, metric_key],
        row_to_sample,
    )
    .optional()
}

/// Como `latest_device_sample`, pero acotada a `sampled_at_utc <= at_utc`: la línea base para un
/// delta (spec 009, informe imprimible) — el valor del contador justo antes o al empezar el
/// intervalo, para no confundir "valor absoluto" con "variación".
pub fn latest_device_sample_at_or_before(
    conn: &Connection,
    device_id: &str,
    metric_key: &str,
    at_utc: &str,
) -> rusqlite::Result<Option<MetricSample>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT * FROM metric_samples
         WHERE device_id = ?1 AND metric_key = ?2 AND sampled_at_utc <= ?3
         ORDER BY sampled_at_utc DESC LIMIT 1",
        params![device_id, metric_key, at_utc],
        row_to_sample,
    )
    .optional()
}

/// La lectura más reciente de cada métrica que `source` haya aportado para el dispositivo: una
/// fila por `metric_key`, la de mayor `sampled_at_utc`. Alimenta los contadores del detalle de
/// disco (`DeviceDetail.counters`) sin tener que conocer de antemano qué claves existen.
pub fn latest_samples_by_source(
    conn: &Connection,
    device_id: &str,
    source: MetricSource,
) -> rusqlite::Result<Vec<MetricSample>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM metric_samples WHERE device_id = ?1 AND source = ?2
         ORDER BY metric_key, sampled_at_utc DESC",
    )?;
    let filas = stmt.query_map(params![device_id, source_to_str(source)], row_to_sample)?;

    let mut vistas = std::collections::HashSet::new();
    let mut resultado = Vec::new();
    for fila in filas {
        let muestra = fila?;
        if vistas.insert(muestra.metric_key.clone()) {
            resultado.push(muestra);
        }
    }
    Ok(resultado)
}

/// ¿Alguna vez `smartctl` leyó bien este disco? Se usa para no disparar `smart.unreadable` en un
/// disco que **nunca** respondió —eso es «no compatible», no «dejó de responder»
/// (`alert-rules.md`)—. `smart_snapshots` no se poda, así que la señal es estable en el tiempo.
///
/// Cuentan los dos `query_status` de éxito que escribe `persist_smart_reading` (`ok` y
/// `no_health_field` — este último es un disco que da SMART pero sin campo de autoevaluación); no
/// los de fallo (`error`, `unparseable`).
pub fn hubo_lectura_smart_correcta(conn: &Connection, device_id: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM smart_snapshots
         WHERE device_id = ?1 AND query_status IN ('ok', 'no_health_field'))",
        params![device_id],
        |r| r.get::<_, i64>(0),
    )
    .map(|n| n != 0)
}

/// Registra una captura completa de `smartctl`, con su procedencia y el JSON crudo cuando exista.
#[allow(clippy::too_many_arguments)]
pub fn insert_smart_snapshot(
    conn: &Connection,
    device_id: &str,
    captured_at_utc: &str,
    smartctl_version: Option<&str>,
    exit_status: Option<i64>,
    query_status: &str,
    raw_json_path: Option<&str>,
    fields_json: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO smart_snapshots (
            device_id, captured_at_utc, smartctl_version, exit_status, query_status,
            raw_json_path, fields_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            device_id,
            captured_at_utc,
            smartctl_version,
            exit_status,
            query_status,
            raw_json_path,
            fields_json,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("repo_metricas");
        let (conn, _) = db::open(&dir).unwrap();
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn muestra(valor: f64, cuando: &str) -> MetricSample {
        MetricSample {
            target: MetricTarget::Device("d1".to_string()),
            metric_key: "temperature_celsius".to_string(),
            value_real: Some(valor),
            value_integer: None,
            unit: "celsius".to_string(),
            sampled_at_utc: cuando.to_string(),
            source: MetricSource::Smartctl,
            quality: MetricQuality::Exact,
            resolution: Resolution::Raw,
        }
    }

    #[test]
    fn inserta_y_recupera_serie_ordenada() {
        let conn = conn_de_prueba();
        insert_sample(&conn, &muestra(40.0, "2026-09-04T10:00:00Z")).unwrap();
        insert_sample(&conn, &muestra(41.0, "2026-09-04T10:05:00Z")).unwrap();
        insert_sample(&conn, &muestra(39.0, "2026-09-04T09:00:00Z")).unwrap();

        let serie = device_series(
            &conn,
            "d1",
            "temperature_celsius",
            "2026-09-04T00:00:00Z",
            "2026-09-04T23:59:59Z",
        )
        .unwrap();

        assert_eq!(serie.len(), 3);
        assert_eq!(serie[0].sampled_at_utc, "2026-09-04T09:00:00Z");
        assert_eq!(serie[2].sampled_at_utc, "2026-09-04T10:05:00Z");
    }

    #[test]
    fn la_ultima_muestra_es_la_de_mayor_fecha() {
        let conn = conn_de_prueba();
        insert_sample(&conn, &muestra(40.0, "2026-09-04T10:00:00Z")).unwrap();
        insert_sample(&conn, &muestra(42.0, "2026-09-04T11:00:00Z")).unwrap();

        let ultima = latest_device_sample(&conn, "d1", "temperature_celsius")
            .unwrap()
            .unwrap();
        assert_eq!(ultima.value_real, Some(42.0));
    }

    #[test]
    fn sin_muestras_devuelve_ninguna() {
        let conn = conn_de_prueba();
        assert!(latest_device_sample(&conn, "d1", "temperature_celsius")
            .unwrap()
            .is_none());
    }

    #[test]
    fn latest_n_values_devuelve_de_mas_reciente_a_mas_antigua_y_recorta_a_n() {
        let conn = conn_de_prueba();
        insert_sample(&conn, &muestra(39.0, "2026-09-04T09:00:00Z")).unwrap();
        insert_sample(&conn, &muestra(40.0, "2026-09-04T10:00:00Z")).unwrap();
        insert_sample(&conn, &muestra(41.0, "2026-09-04T11:00:00Z")).unwrap();

        let valores = latest_n_values(&conn, "d1", "temperature_celsius", 2).unwrap();
        assert_eq!(valores, vec![41.0, 40.0]);
    }

    #[test]
    fn latest_n_values_sin_muestras_devuelve_vacio() {
        let conn = conn_de_prueba();
        assert!(latest_n_values(&conn, "d1", "temperature_celsius", 3)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn hubo_lectura_smart_correcta_distingue_un_snapshot_ok_de_uno_fallido() {
        let conn = conn_de_prueba();
        assert!(
            !hubo_lectura_smart_correcta(&conn, "d1").unwrap(),
            "sin snapshots, no"
        );

        insert_smart_snapshot(
            &conn,
            "d1",
            "2026-09-04T10:00:00Z",
            None,
            Some(1),
            "error",
            None,
            None,
        )
        .unwrap();
        assert!(
            !hubo_lectura_smart_correcta(&conn, "d1").unwrap(),
            "un snapshot fallido no cuenta"
        );

        insert_smart_snapshot(
            &conn,
            "d1",
            "2026-09-04T10:05:00Z",
            None,
            Some(0),
            "ok",
            None,
            None,
        )
        .unwrap();
        assert!(hubo_lectura_smart_correcta(&conn, "d1").unwrap());
    }

    #[test]
    fn latest_samples_by_source_devuelve_una_fila_por_metrica_la_mas_reciente() {
        let conn = conn_de_prueba();
        insert_sample(&conn, &muestra(40.0, "2026-09-04T10:00:00Z")).unwrap();
        insert_sample(&conn, &muestra(42.0, "2026-09-04T11:00:00Z")).unwrap();

        let mut otra = muestra(500.0, "2026-09-04T11:00:00Z");
        otra.metric_key = "power_on_hours".to_string();
        insert_sample(&conn, &otra).unwrap();

        let ultimas = latest_samples_by_source(&conn, "d1", MetricSource::Smartctl).unwrap();

        assert_eq!(
            ultimas.len(),
            2,
            "una fila por metric_key, no una por muestra"
        );
        let temp = ultimas
            .iter()
            .find(|m| m.metric_key == "temperature_celsius")
            .unwrap();
        assert_eq!(
            temp.value_real,
            Some(42.0),
            "la más reciente de las dos de temperatura"
        );
        let horas = ultimas
            .iter()
            .find(|m| m.metric_key == "power_on_hours")
            .unwrap();
        assert_eq!(horas.value_real, Some(500.0));
    }

    #[test]
    fn latest_samples_by_source_no_mezcla_fuentes_distintas() {
        let conn = conn_de_prueba();
        insert_sample(&conn, &muestra(40.0, "2026-09-04T10:00:00Z")).unwrap();

        let ninguna = latest_samples_by_source(&conn, "d1", MetricSource::WindowsStorage).unwrap();
        assert!(ninguna.is_empty());
    }
}
