//! Compactación de retención: agrupa muestras `raw` antiguas en `metric_aggregates`, conservando
//! mínimo, máximo, promedio, primera y última lectura (`docs/data-model.md` §4,
//! `open-questions.md` J.14). Corre en transacción: o se agrega y se borra el crudo, o no pasa nada.

use rusqlite::Connection;
use time::OffsetDateTime;

use crate::domain::tipos::{AggregateResolution, MetricAggregate, MetricTarget};
use crate::persistence::repo_agregados;

fn parse_utc(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
}

fn format_utc(t: OffsetDateTime) -> String {
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// El inicio del bucket al que pertenece `t`: trunca a múltiplos de 5 minutos o a la hora en punto.
fn bucket_start(t: OffsetDateTime, resolution: AggregateResolution) -> OffsetDateTime {
    let minuto = t.minute();
    let minuto_truncado = match resolution {
        AggregateResolution::FiveMinutes => minuto - (minuto % 5),
        AggregateResolution::Hourly => 0,
    };
    t.replace_minute(minuto_truncado)
        .unwrap_or(t)
        .replace_second(0)
        .unwrap_or(t)
        .replace_nanosecond(0)
        .unwrap_or(t)
}

fn bucket_duration(resolution: AggregateResolution) -> time::Duration {
    match resolution {
        AggregateResolution::FiveMinutes => time::Duration::minutes(5),
        AggregateResolution::Hourly => time::Duration::hours(1),
    }
}

/// Compacta las muestras `raw` de `target`/`metric_key` anteriores a `cutoff_utc` en agregados de
/// `resolution`. Devuelve el número de buckets creados. **Nunca inventa un cero**: un bucket sin
/// ninguna muestra con valor no se crea; solo se agregan muestras que ya tenían un valor real.
pub fn compact_samples(
    conn: &mut Connection,
    target: &MetricTarget,
    metric_key: &str,
    cutoff_utc: &str,
    resolution: AggregateResolution,
) -> rusqlite::Result<usize> {
    let (device_id, volume_id) = match target {
        MetricTarget::Device(id) => (Some(id.clone()), None),
        MetricTarget::Volume(id) => (None, Some(id.clone())),
    };

    struct FilaCruda {
        id: i64,
        sampled_at_utc: String,
        value_real: Option<f64>,
        value_integer: Option<i64>,
        unit: String,
    }

    // Muestras raw a compactar, ordenadas por tiempo: el orden es lo que permite calcular
    // "primera" y "última" sin una segunda consulta.
    let filas: Vec<FilaCruda> = {
        let mut stmt = conn.prepare(
            "SELECT id, sampled_at_utc, value_real, value_integer, unit FROM metric_samples
             WHERE device_id IS ?1 AND volume_id IS ?2 AND metric_key = ?3
               AND resolution = 'raw' AND sampled_at_utc < ?4
             ORDER BY sampled_at_utc",
        )?;
        let filas = stmt.query_map(
            rusqlite::params![device_id, volume_id, metric_key, cutoff_utc],
            |r| {
                Ok(FilaCruda {
                    id: r.get(0)?,
                    sampled_at_utc: r.get(1)?,
                    value_real: r.get(2)?,
                    value_integer: r.get(3)?,
                    unit: r.get(4)?,
                })
            },
        )?;
        filas.collect::<rusqlite::Result<_>>()?
    };

    if filas.is_empty() {
        return Ok(0);
    }

    // Agrupa en buckets consecutivos, en el mismo orden en que llegaron (ya ordenadas por tiempo).
    struct Bucket {
        inicio: OffsetDateTime,
        valores: Vec<f64>,
        ids: Vec<i64>,
        unit: String,
    }
    let mut buckets: Vec<Bucket> = Vec::new();

    for fila in &filas {
        let Some(t) = parse_utc(&fila.sampled_at_utc) else {
            continue;
        };
        let valor = match (fila.value_real, fila.value_integer) {
            (Some(v), _) => v,
            (None, Some(v)) => v as f64,
            (None, None) => continue, // no debería ocurrir: insert_sample lo impide
        };
        let inicio = bucket_start(t, resolution);

        match buckets.last_mut() {
            Some(b) if b.inicio == inicio => {
                b.valores.push(valor);
                b.ids.push(fila.id);
            }
            _ => buckets.push(Bucket {
                inicio,
                valores: vec![valor],
                ids: vec![fila.id],
                unit: fila.unit.clone(),
            }),
        }
    }

    let tx = conn.transaction()?;
    let mut creados = 0usize;
    let mut todos_los_ids: Vec<i64> = Vec::new();

    for b in &buckets {
        let n = b.valores.len() as f64;
        let suma: f64 = b.valores.iter().sum();
        let min = b.valores.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = b.valores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let agregado = MetricAggregate {
            target: target.clone(),
            metric_key: metric_key.to_string(),
            bucket_start_utc: format_utc(b.inicio),
            bucket_end_utc: format_utc(b.inicio + bucket_duration(resolution)),
            resolution,
            value_min: Some(min),
            value_max: Some(max),
            value_avg: Some(suma / n),
            value_first: b.valores.first().copied(),
            value_last: b.valores.last().copied(),
            sample_count: b.valores.len() as i64,
            unit: b.unit.clone(),
        };
        repo_agregados::insert_aggregate(&tx, &agregado)?;
        creados += 1;
        todos_los_ids.extend(&b.ids);
    }

    for id in &todos_los_ids {
        tx.execute("DELETE FROM metric_samples WHERE id = ?1", [id])?;
    }

    tx.commit()?;
    Ok(creados)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{MetricQuality, MetricSample, MetricSource, Resolution};
    use crate::persistence::{db, repo_metricas};

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("retencion");
        let conn = db::open(&dir).unwrap().0;
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
    fn conserva_min_max_promedio_primera_y_ultima() {
        let mut conn = conn_de_prueba();
        for (valor, hora) in [
            (40.0, "2026-09-01T10:00:00Z"),
            (44.0, "2026-09-01T10:01:00Z"),
            (38.0, "2026-09-01T10:02:00Z"),
            (42.0, "2026-09-01T10:03:00Z"),
        ] {
            repo_metricas::insert_sample(&conn, &muestra(valor, hora)).unwrap();
        }

        let creados = compact_samples(
            &mut conn,
            &MetricTarget::Device("d1".to_string()),
            "temperature_celsius",
            "2026-09-02T00:00:00Z",
            AggregateResolution::FiveMinutes,
        )
        .unwrap();

        assert_eq!(
            creados, 1,
            "las cuatro muestras caen en el mismo bucket de 5 minutos"
        );
        assert_eq!(
            repo_agregados::count_aggregates_for_device(&conn, "d1").unwrap(),
            1
        );

        let agregado: (f64, f64, f64, f64, f64, i64) = conn
            .query_row(
                "SELECT value_min, value_max, value_avg, value_first, value_last, sample_count FROM metric_aggregates",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .unwrap();
        assert_eq!(agregado, (38.0, 44.0, 41.0, 40.0, 42.0, 4));
    }

    #[test]
    fn borra_las_muestras_raw_compactadas() {
        let mut conn = conn_de_prueba();
        repo_metricas::insert_sample(&conn, &muestra(40.0, "2026-09-01T10:00:00Z")).unwrap();

        compact_samples(
            &mut conn,
            &MetricTarget::Device("d1".to_string()),
            "temperature_celsius",
            "2026-09-02T00:00:00Z",
            AggregateResolution::FiveMinutes,
        )
        .unwrap();

        assert!(
            repo_metricas::latest_device_sample(&conn, "d1", "temperature_celsius")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn no_toca_muestras_mas_recientes_que_el_corte() {
        let mut conn = conn_de_prueba();
        repo_metricas::insert_sample(&conn, &muestra(40.0, "2026-09-01T10:00:00Z")).unwrap();
        repo_metricas::insert_sample(&conn, &muestra(50.0, "2026-09-03T10:00:00Z")).unwrap();

        compact_samples(
            &mut conn,
            &MetricTarget::Device("d1".to_string()),
            "temperature_celsius",
            "2026-09-02T00:00:00Z",
            AggregateResolution::FiveMinutes,
        )
        .unwrap();

        let restante = repo_metricas::latest_device_sample(&conn, "d1", "temperature_celsius")
            .unwrap()
            .unwrap();
        assert_eq!(
            restante.value_real,
            Some(50.0),
            "la muestra posterior al corte se conserva"
        );
    }

    #[test]
    fn sin_muestras_que_compactar_no_crea_nada() {
        let mut conn = conn_de_prueba();
        let creados = compact_samples(
            &mut conn,
            &MetricTarget::Device("d1".to_string()),
            "temperature_celsius",
            "2026-09-02T00:00:00Z",
            AggregateResolution::FiveMinutes,
        )
        .unwrap();
        assert_eq!(creados, 0);
    }

    #[test]
    fn separa_en_buckets_distintos_por_tiempo() {
        let mut conn = conn_de_prueba();
        repo_metricas::insert_sample(&conn, &muestra(40.0, "2026-09-01T10:00:00Z")).unwrap();
        repo_metricas::insert_sample(&conn, &muestra(41.0, "2026-09-01T10:07:00Z")).unwrap();

        let creados = compact_samples(
            &mut conn,
            &MetricTarget::Device("d1".to_string()),
            "temperature_celsius",
            "2026-09-02T00:00:00Z",
            AggregateResolution::FiveMinutes,
        )
        .unwrap();

        assert_eq!(
            creados, 2,
            "10:00 y 10:07 caen en buckets de 5 minutos distintos"
        );
    }

    /// FR-016 (feature `004-ignorar-alertas`): la compactación de retención solo toca
    /// `metric_samples`. Un grupo de alerta `ignored` y su cronología no se tocan nunca — igual que
    /// no se toca ninguna alerta (constitución §V: la retención nunca borra alertas). La prueba fija
    /// esa garantía por si `compact_samples` cambiara de alcance.
    #[test]
    fn compactar_no_toca_las_alertas_ignoradas_ni_su_cronologia() {
        let mut conn = conn_de_prueba();
        conn.execute(
            "INSERT INTO alert_groups (id, deduplication_key, rule_key, target_device_id, severity, status,
                cycle, first_occurrence_at_utc, last_occurrence_at_utc, occurrence_count, ignored_at_utc)
             VALUES ('g1', 'k1', 'temp.above_configured_warn', 'd1', 'warning', 'ignored',
                1, '2026-08-01T00:00:00Z', '2026-08-02T00:00:00Z', 3, '2026-08-03T00:00:00Z')",
            [],
        )
        .unwrap();
        for cuando in [
            "2026-08-01T00:00:00Z",
            "2026-08-01T12:00:00Z",
            "2026-08-02T00:00:00Z",
        ] {
            conn.execute(
                "INSERT INTO alert_occurrences (alert_group_id, cycle, occurred_at_utc, value_real)
                 VALUES ('g1', 1, ?1, 62.0)",
                [cuando],
            )
            .unwrap();
        }
        repo_metricas::insert_sample(&conn, &muestra(40.0, "2026-08-01T10:00:00Z")).unwrap();

        compact_samples(
            &mut conn,
            &MetricTarget::Device("d1".to_string()),
            "temperature_celsius",
            "2026-09-02T00:00:00Z",
            AggregateResolution::Hourly,
        )
        .unwrap();

        let grupos: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM alert_groups WHERE id = 'g1' AND status = 'ignored'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(grupos, 1);
        let ocurrencias: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM alert_occurrences WHERE alert_group_id = 'g1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            ocurrencias, 3,
            "la cronología de una alerta ignorada no se compacta"
        );
    }
}
