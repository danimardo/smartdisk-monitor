//! Repositorio de `metric_aggregates` (`docs/data-model.md` §2, `open-questions.md` J.14).

use rusqlite::{params, Connection};

use crate::domain::tipos::{AggregateResolution, MetricAggregate, MetricTarget};

fn resolution_to_str(v: AggregateResolution) -> &'static str {
    match v {
        AggregateResolution::FiveMinutes => "five_minutes",
        AggregateResolution::Hourly => "hourly",
    }
}

fn resolution_from_str(s: &str) -> AggregateResolution {
    match s {
        "hourly" => AggregateResolution::Hourly,
        _ => AggregateResolution::FiveMinutes,
    }
}

fn row_to_aggregate(
    row: &rusqlite::Row,
    target: MetricTarget,
) -> rusqlite::Result<MetricAggregate> {
    Ok(MetricAggregate {
        target,
        metric_key: row.get("metric_key")?,
        bucket_start_utc: row.get("bucket_start_utc")?,
        bucket_end_utc: row.get("bucket_end_utc")?,
        resolution: resolution_from_str(&row.get::<_, String>("resolution")?),
        value_min: row.get("value_min")?,
        value_max: row.get("value_max")?,
        value_avg: row.get("value_avg")?,
        value_first: row.get("value_first")?,
        value_last: row.get("value_last")?,
        sample_count: row.get("sample_count")?,
        unit: row.get("unit")?,
    })
}

/// Agregados de un dispositivo en un rango, para una resolución concreta y ordenados por tiempo.
/// `domain::series` decide qué hacer con los huecos; este repositorio solo devuelve lo que hay.
pub fn device_aggregates(
    conn: &Connection,
    device_id: &str,
    metric_key: &str,
    resolution: AggregateResolution,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<MetricAggregate>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM metric_aggregates
         WHERE device_id = ?1 AND metric_key = ?2 AND resolution = ?3
           AND bucket_start_utc BETWEEN ?4 AND ?5
         ORDER BY bucket_start_utc",
    )?;
    let filas = stmt.query_map(
        params![
            device_id,
            metric_key,
            resolution_to_str(resolution),
            from_utc,
            to_utc
        ],
        |r| row_to_aggregate(r, MetricTarget::Device(device_id.to_string())),
    )?;
    filas.collect()
}

pub fn insert_aggregate(conn: &Connection, a: &MetricAggregate) -> rusqlite::Result<()> {
    let (device_id, volume_id): (Option<&str>, Option<&str>) = match &a.target {
        MetricTarget::Device(id) => (Some(id.as_str()), None),
        MetricTarget::Volume(id) => (None, Some(id.as_str())),
    };
    conn.execute(
        "INSERT INTO metric_aggregates (
            device_id, volume_id, metric_key, bucket_start_utc, bucket_end_utc, resolution,
            value_min, value_max, value_avg, value_first, value_last, sample_count, unit
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            device_id,
            volume_id,
            a.metric_key,
            a.bucket_start_utc,
            a.bucket_end_utc,
            resolution_to_str(a.resolution),
            a.value_min,
            a.value_max,
            a.value_avg,
            a.value_first,
            a.value_last,
            a.sample_count,
            a.unit,
        ],
    )?;
    Ok(())
}

pub fn count_aggregates_for_device(conn: &Connection, device_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM metric_aggregates WHERE device_id = ?1",
        params![device_id],
        |r| r.get(0),
    )
}
