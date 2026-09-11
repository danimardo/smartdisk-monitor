//! Exportación tabular (CSV) y estructurada (JSON) del historial de métricas (T087,
//! `docs/product-specification.md` §9, `docs/open-questions.md` J.30).
//!
//! CSV y JSON son el volcado completo: una fila/objeto por `(dispositivo, metric_key, marca de
//! tiempo)`, con la misma cascada de resolución que ya usa `commands::get_metric_series_impl` para
//! las gráficas — reimplementada aquí, no compartida, para no arriesgar una regresión en la
//! gráfica por una necesidad distinta (J.30).

use time::format_description::well_known::Rfc3339;

use crate::domain::tipos::{AggregateResolution, Device};
use crate::persistence::{repo_agregados, repo_metricas};

pub const SCHEMA_VERSION: &str = "1";
/// El HTML es un resumen legible para personas; su estructura cambia de forma independiente del
/// volcado CSV/JSON (spec 009). Subir esto **no** toca `SCHEMA_VERSION` de CSV/JSON.
pub const SCHEMA_VERSION_HTML: &str = "2";
const NO_DISPONIBLE: &str = "N/A";

#[derive(Debug, Clone, PartialEq)]
pub struct FilaMetrica {
    pub metric_key: String,
    pub unit: String,
    pub resolution: &'static str,
    pub timestamp_utc: String,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct RangoExport {
    pub desde: time::OffsetDateTime,
    pub hasta: time::OffsetDateTime,
}

impl RangoExport {
    pub(crate) fn desde_como_texto(&self) -> String {
        self.desde.format(&Rfc3339).unwrap_or_default()
    }

    pub(crate) fn hasta_como_texto(&self) -> String {
        self.hasta.format(&Rfc3339).unwrap_or_default()
    }
}

pub fn etiqueta_dispositivo(d: &Device) -> String {
    d.alias.clone().unwrap_or_else(|| d.model.clone())
}

/// Resolución servida para una serie del informe y su cadencia en ms, con la **misma cascada** que
/// `filas_dispositivo` y `commands::get_metric_series_impl` (`docs/open-questions.md` E.1). El
/// informe la comparte entre el resumen numérico y las mini-gráficas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolucionInforme {
    Raw,
    FiveMinutes,
    Hourly,
}

impl ResolucionInforme {
    pub fn cadencia_ms(self, metric_key: &str) -> i64 {
        match self {
            // Temperatura sale del ciclo SMART (300 s); actividad y demás rápidas, 30 s.
            Self::Raw if metric_key == "temperature_celsius" => 300_000,
            Self::Raw => 30_000,
            Self::FiveMinutes => 300_000,
            Self::Hourly => 3_600_000,
        }
    }
}

/// Serie `(timestamp_utc, valor)` de una métrica de un dispositivo en el rango, ya elegida la
/// resolución. Devuelve también la resolución y su cadencia, para el eje y los huecos.
pub fn serie_device(
    conn: &rusqlite::Connection,
    device_id: &str,
    metric_key: &str,
    rango: RangoExport,
) -> rusqlite::Result<(Vec<(String, f64)>, ResolucionInforme)> {
    let from_utc = rango.desde_como_texto();
    let to_utc = rango.hasta_como_texto();
    let ahora = time::OffsetDateTime::now_utc();
    let ancho = rango.hasta - rango.desde;
    let dentro_de_siete_dias = rango.desde >= ahora - time::Duration::days(7);

    let (filas, resolucion): (Vec<(String, f64)>, ResolucionInforme) =
        if ancho <= time::Duration::hours(24) && dentro_de_siete_dias {
            (
                repo_metricas::device_series(conn, device_id, metric_key, &from_utc, &to_utc)?
                    .into_iter()
                    .filter_map(|m| {
                        m.value_real
                            .or(m.value_integer.map(|v| v as f64))
                            .map(|v| (m.sampled_at_utc, v))
                    })
                    .collect(),
                ResolucionInforme::Raw,
            )
        } else if ancho <= time::Duration::days(7) {
            (
                agregados_serie(
                    conn,
                    device_id,
                    metric_key,
                    AggregateResolution::FiveMinutes,
                    &from_utc,
                    &to_utc,
                )?,
                ResolucionInforme::FiveMinutes,
            )
        } else if ancho <= time::Duration::days(90) {
            let cinco = agregados_serie(
                conn,
                device_id,
                metric_key,
                AggregateResolution::FiveMinutes,
                &from_utc,
                &to_utc,
            )?;
            if cinco.is_empty() {
                (
                    agregados_serie(
                        conn,
                        device_id,
                        metric_key,
                        AggregateResolution::Hourly,
                        &from_utc,
                        &to_utc,
                    )?,
                    ResolucionInforme::Hourly,
                )
            } else {
                (cinco, ResolucionInforme::FiveMinutes)
            }
        } else {
            (
                agregados_serie(
                    conn,
                    device_id,
                    metric_key,
                    AggregateResolution::Hourly,
                    &from_utc,
                    &to_utc,
                )?,
                ResolucionInforme::Hourly,
            )
        };
    Ok((filas, resolucion))
}

fn agregados_serie(
    conn: &rusqlite::Connection,
    device_id: &str,
    metric_key: &str,
    resolucion: AggregateResolution,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<(String, f64)>> {
    Ok(repo_agregados::device_aggregates(
        conn, device_id, metric_key, resolucion, from_utc, to_utc,
    )?
    .into_iter()
    .filter_map(|a| {
        a.value_avg
            .or(a.value_last)
            .map(|v| (a.bucket_start_utc, v))
    })
    .collect())
}

/// Todas las filas de un dispositivo en el rango, para todas las métricas que de verdad tengan
/// dato (`repo_metricas::distinct_metric_keys`): no hay una lista fija de columnas.
pub fn filas_dispositivo(
    conn: &rusqlite::Connection,
    device_id: &str,
    rango: RangoExport,
) -> rusqlite::Result<Vec<FilaMetrica>> {
    let from_utc = rango.desde_como_texto();
    let to_utc = rango.hasta_como_texto();
    let claves = repo_metricas::distinct_metric_keys(conn, device_id, &from_utc, &to_utc)?;

    let ahora = time::OffsetDateTime::now_utc();
    let ancho = rango.hasta - rango.desde;
    let dentro_de_siete_dias = rango.desde >= ahora - time::Duration::days(7);

    let mut filas = Vec::new();
    for clave in claves {
        if ancho <= time::Duration::hours(24) && dentro_de_siete_dias {
            for m in repo_metricas::device_series(conn, device_id, &clave, &from_utc, &to_utc)? {
                filas.push(FilaMetrica {
                    metric_key: clave.clone(),
                    unit: m.unit,
                    resolution: "raw",
                    timestamp_utc: m.sampled_at_utc,
                    // Ninguna métrica real usa `value_integer` todavía (ningún colector lo llena
                    // hoy), pero el tipo lo admite: la reserva evita que el día que exista, la
                    // exportación la muestre como "N/A" teniendo el dato ahí mismo.
                    value: m.value_real.or(m.value_integer.map(|v| v as f64)),
                });
            }
        } else if ancho <= time::Duration::days(7) {
            filas.extend(filas_agregadas(
                conn,
                device_id,
                &clave,
                AggregateResolution::FiveMinutes,
                "five_minutes",
                &from_utc,
                &to_utc,
            )?);
        } else if ancho <= time::Duration::days(90) {
            let cinco_min = filas_agregadas(
                conn,
                device_id,
                &clave,
                AggregateResolution::FiveMinutes,
                "five_minutes",
                &from_utc,
                &to_utc,
            )?;
            if cinco_min.is_empty() {
                filas.extend(filas_agregadas(
                    conn,
                    device_id,
                    &clave,
                    AggregateResolution::Hourly,
                    "hourly",
                    &from_utc,
                    &to_utc,
                )?);
            } else {
                filas.extend(cinco_min);
            }
        } else {
            filas.extend(filas_agregadas(
                conn,
                device_id,
                &clave,
                AggregateResolution::Hourly,
                "hourly",
                &from_utc,
                &to_utc,
            )?);
        }
    }
    Ok(filas)
}

#[allow(clippy::too_many_arguments)]
fn filas_agregadas(
    conn: &rusqlite::Connection,
    device_id: &str,
    metric_key: &str,
    resolucion: AggregateResolution,
    etiqueta: &'static str,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<FilaMetrica>> {
    Ok(repo_agregados::device_aggregates(
        conn, device_id, metric_key, resolucion, from_utc, to_utc,
    )?
    .into_iter()
    .map(|a| FilaMetrica {
        metric_key: metric_key.to_string(),
        unit: a.unit,
        resolution: etiqueta,
        timestamp_utc: a.bucket_start_utc,
        value: a.value_avg.or(a.value_last),
    })
    .collect())
}

/// Envuelve un campo para CSV (RFC 4180 mínimo): comillas solo si el valor las necesita.
fn csv_campo(valor: &str) -> String {
    if valor.contains(['"', ',', '\n', '\r']) {
        format!("\"{}\"", valor.replace('"', "\"\""))
    } else {
        valor.to_string()
    }
}

/// `schemaVersion` viaja como columna, no como comentario: `product-specification.md` §9 exige
/// que "toda exportación lleve un campo `schemaVersion`, incluidas las cabeceras de CSV".
pub fn generar_csv(
    conn: &rusqlite::Connection,
    dispositivos: &[Device],
    rango: RangoExport,
) -> rusqlite::Result<String> {
    let mut salida = String::from(
        "schemaVersion,deviceId,deviceLabel,metricKey,unit,resolution,timestampUtc,value\n",
    );
    for d in dispositivos {
        let etiqueta = etiqueta_dispositivo(d);
        for fila in filas_dispositivo(conn, &d.id, rango)? {
            let valor = fila
                .value
                .map(|v| v.to_string())
                .unwrap_or_else(|| NO_DISPONIBLE.to_string());
            salida.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                SCHEMA_VERSION,
                csv_campo(&d.id),
                csv_campo(&etiqueta),
                csv_campo(&fila.metric_key),
                csv_campo(&fila.unit),
                fila.resolution,
                fila.timestamp_utc,
                valor
            ));
        }
    }
    Ok(salida)
}

pub fn generar_json(
    conn: &rusqlite::Connection,
    dispositivos: &[Device],
    rango: RangoExport,
    include_serials: bool,
) -> rusqlite::Result<String> {
    let mut lista = Vec::with_capacity(dispositivos.len());
    for d in dispositivos {
        let muestras: Vec<serde_json::Value> = filas_dispositivo(conn, &d.id, rango)?
            .into_iter()
            .map(|f| {
                serde_json::json!({
                    "metricKey": f.metric_key,
                    "unit": f.unit,
                    "resolution": f.resolution,
                    "timestampUtc": f.timestamp_utc,
                    "value": f.value,
                })
            })
            .collect();
        lista.push(serde_json::json!({
            "id": d.id,
            "label": etiqueta_dispositivo(d),
            "serialNumber": if include_serials { d.serial_number.clone() } else { None },
            "samples": muestras,
        }));
    }

    let documento = serde_json::json!({
        "schemaVersion": SCHEMA_VERSION,
        "generatedAtUtc": time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default(),
        "fromUtc": rango.desde_como_texto(),
        "toUtc": rango.hasta_como_texto(),
        "devices": lista,
    });
    Ok(serde_json::to_string_pretty(&documento).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{DeviceType, IdentityConfidence};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("reporting_export");
        db::open(&dir).unwrap().0
    }

    fn dispositivo(id: &str, alias: Option<&str>) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: Some("S123456".to_string()),
            model: "Modelo de prueba".to_string(),
            manufacturer: None,
            firmware: None,
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: None,
            capacity_bytes: Some(1_000_000_000),
            alias: alias.map(str::to_string),
            monitoring_enabled: true,
            first_seen_at: "2026-09-01T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    fn rango_reciente() -> RangoExport {
        RangoExport {
            desde: time::OffsetDateTime::parse("2026-09-04T00:00:00Z", &Rfc3339).unwrap(),
            hasta: time::OffsetDateTime::parse("2026-09-04T10:00:00Z", &Rfc3339).unwrap(),
        }
    }

    #[test]
    fn csv_lleva_la_cabecera_y_la_version_de_esquema_aunque_no_haya_datos() {
        let conn = conn_de_prueba();
        let csv = generar_csv(&conn, &[dispositivo("d1", None)], rango_reciente()).unwrap();
        assert!(csv.starts_with(
            "schemaVersion,deviceId,deviceLabel,metricKey,unit,resolution,timestampUtc,value\n"
        ));
        // Sin muestras: solo la cabecera, ninguna fila inventada.
        assert_eq!(csv.lines().count(), 1);
    }

    #[test]
    fn un_campo_csv_con_coma_se_entrecomilla() {
        assert_eq!(csv_campo("Disco, externo"), "\"Disco, externo\"");
        assert_eq!(csv_campo("Disco externo"), "Disco externo");
    }

    #[test]
    fn json_incluye_el_numero_de_serie_solo_si_se_pide() {
        let conn = conn_de_prueba();
        let dispositivos = [dispositivo("d1", Some("Mi disco"))];

        let con_serie = generar_json(&conn, &dispositivos, rango_reciente(), true).unwrap();
        assert!(con_serie.contains("S123456"));

        let sin_serie = generar_json(&conn, &dispositivos, rango_reciente(), false).unwrap();
        assert!(!sin_serie.contains("S123456"));
    }

    #[test]
    fn json_trae_la_version_de_esquema_y_la_etiqueta_del_dispositivo() {
        let conn = conn_de_prueba();
        let dispositivos = [dispositivo("d1", Some("Mi disco"))];
        let json = generar_json(&conn, &dispositivos, rango_reciente(), false).unwrap();
        assert!(json.contains("\"schemaVersion\": \"1\""));
        assert!(json.contains("\"label\": \"Mi disco\""));
    }

    #[test]
    fn una_muestra_entera_sin_valor_real_se_exporta_por_su_valor_entero() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", None);
        repo_inventario_upsert(&conn, &d);
        repo_metricas::insert_sample(
            &conn,
            &crate::domain::tipos::MetricSample {
                target: crate::domain::tipos::MetricTarget::Device("d1".to_string()),
                metric_key: "power_cycles".to_string(),
                value_real: None,
                value_integer: Some(7),
                unit: "count".to_string(),
                sampled_at_utc: "2026-09-04T05:00:00Z".to_string(),
                source: crate::domain::tipos::MetricSource::Smartctl,
                quality: crate::domain::tipos::MetricQuality::Exact,
                resolution: crate::domain::tipos::Resolution::Raw,
            },
        )
        .unwrap();

        let csv = generar_csv(&conn, &[d], rango_reciente()).unwrap();
        let fila = csv.lines().nth(1).expect("debe haber una fila de datos");
        assert!(
            fila.ends_with(",7"),
            "el valor entero debe aparecer aunque no haya `value_real`: {fila}"
        );
    }

    fn repo_inventario_upsert(conn: &rusqlite::Connection, d: &Device) {
        crate::persistence::repo_inventario::upsert_device(conn, d).unwrap();
    }
}
