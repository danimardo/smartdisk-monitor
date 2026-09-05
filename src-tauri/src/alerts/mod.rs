//! Motor de reglas de alerta: activación, histéresis, agrupación, deduplicación y ciclo de vida.
//!
//! El comportamiento normativo está en `docs/alert-rules.md`, regla por regla; este módulo lo
//! implementa. Cada regla exige cinco pruebas sin excepción (`alert-rules.md` §5): activación,
//! **no** activación ante dato ausente, histéresis, deduplicación y ciclo de recaída
//! (constitución §VIII).

use rusqlite::Connection;

use crate::domain::tipos::AlertSeverity;
use crate::persistence::repo_metricas;
use agrupacion::{EvaluacionAlerta, Transicion};

pub mod agrupacion;
pub mod ciclo;
pub mod motor;
pub mod notificaciones;

/// Muestras que se piden por métrica: la histéresis más larga en alcance pide 3 ciclos; una de
/// margen basta para decidir a la vez activación y resolución sin dos consultas.
const MUESTRAS_HISTERESIS: u32 = 4;

/// Evalúa, sobre `metric_samples` ya persistido, las reglas en alcance de la primera versión del
/// motor (`docs/open-questions.md` J.16) para un dispositivo, y aplica el resultado con
/// `agrupacion::procesar`. Se llama tras cada `persist_smart_reading` real
/// (`commands::refresh_smart`): sin este paso el motor nunca produce ningún `alert_group`, por
/// probado que esté en aislamiento — es lo que conecta el ciclo 5 del plan con datos reales.
///
/// `smart.media_errors` y `smart.error_log` **no llevan `resuelve_*`**: `alert-rules.md` las
/// resuelve por tiempo transcurrido sin incremento (24 h), no por N ciclos de histéresis sobre el
/// valor, y ese seguimiento temporal todavía no existe. Quedan activas hasta archivarse a mano,
/// igual que `smart.wear_high` por diseño. Documentado en `docs/open-questions.md`.
///
/// Devuelve, por regla activada/tocada, el par `(deduplication_key, Transicion)` — es lo que
/// `alerts::notificaciones` necesita para decidir si avisa, sin que este módulo (ni `agrupacion`)
/// tenga que saber nada de notificaciones (`agrupacion` lo dice explícito en su cabecera).
pub fn evaluar_smart(
    conn: &Connection,
    device_id: &str,
    ahora_utc: &str,
) -> rusqlite::Result<Vec<(String, Transicion)>> {
    let serie =
        |clave: &str| repo_metricas::latest_n_values(conn, device_id, clave, MUESTRAS_HISTERESIS);

    let mut transiciones = Vec::with_capacity(8);

    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "smart.health.failed",
        &serie("health_passed")?,
        motor::evaluar_health_failed,
        Some(motor::resuelve_health_failed),
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "nvme.critical_warning",
        &serie("critical_warning")?,
        motor::evaluar_critical_warning,
        Some(motor::resuelve_critical_warning),
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "smart.media_errors",
        &serie("media_errors_total")?,
        motor::evaluar_media_errors,
        None,
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "smart.error_log",
        &serie("error_log_entries_total")?,
        motor::evaluar_error_log,
        None,
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "smart.wear_high",
        &serie("percentage_used")?,
        motor::evaluar_wear_high,
        None,
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "temp.above_configured_warn",
        &serie("temperature_celsius")?,
        motor::evaluar_temperatura_configurada_warn,
        Some(motor::resuelve_temperatura_configurada_warn),
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "temp.above_configured_crit",
        &serie("temperature_celsius")?,
        motor::evaluar_temperatura_configurada_crit,
        Some(motor::resuelve_temperatura_configurada_crit),
        ahora_utc,
    )?);

    let spare = serie("available_spare_percent")?;
    let umbral = serie("available_spare_threshold_percent")?;
    let pares: Vec<(f64, f64)> = spare
        .iter()
        .zip(umbral.iter())
        .map(|(&s, &u)| (s, u))
        .collect();
    transiciones.push(aplicar_par(
        conn,
        device_id,
        "smart.spare_below_threshold",
        &pares,
        motor::evaluar_spare_below_threshold,
        Some(motor::resuelve_spare_below_threshold),
        ahora_utc,
    )?);

    Ok(transiciones)
}

fn aplicar_simple(
    conn: &Connection,
    device_id: &str,
    rule_key: &str,
    serie: &[f64],
    evaluar: fn(&[f64]) -> Option<AlertSeverity>,
    resuelve: Option<fn(&[f64]) -> bool>,
    ahora_utc: &str,
) -> rusqlite::Result<(String, Transicion)> {
    let severidad = evaluar(serie);
    let resuelto = severidad.is_none() && resuelve.is_some_and(|f| f(serie));
    let transicion = agrupacion::procesar(
        conn,
        &EvaluacionAlerta {
            rule_key: rule_key.to_string(),
            target_device_id: Some(device_id.to_string()),
            target_volume_id: None,
            context: None,
            severity_si_activa: severidad,
            resuelto,
            value: serie.first().copied(),
            occurred_at_utc: ahora_utc.to_string(),
        },
    )?;
    let clave = agrupacion::deduplication_key(rule_key, Some(device_id), None, None);
    Ok((clave, transicion))
}

/// Par `(available_spare_percent, available_spare_threshold_percent)` de una misma lectura.
type Par = (f64, f64);

fn aplicar_par(
    conn: &Connection,
    device_id: &str,
    rule_key: &str,
    pares: &[Par],
    evaluar: fn(&[Par]) -> Option<AlertSeverity>,
    resuelve: Option<fn(&[Par]) -> bool>,
    ahora_utc: &str,
) -> rusqlite::Result<(String, Transicion)> {
    let severidad = evaluar(pares);
    let resuelto = severidad.is_none() && resuelve.is_some_and(|f| f(pares));
    let transicion = agrupacion::procesar(
        conn,
        &EvaluacionAlerta {
            rule_key: rule_key.to_string(),
            target_device_id: Some(device_id.to_string()),
            target_volume_id: None,
            context: None,
            severity_si_activa: severidad,
            resuelto,
            value: pares.first().map(|&(spare, _)| spare),
            occurred_at_utc: ahora_utc.to_string(),
        },
    )?;
    let clave = agrupacion::deduplication_key(rule_key, Some(device_id), None, None);
    Ok((clave, transicion))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::AlertStatus;
    use crate::persistence::{db, repo_alertas};

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("alerts_mod");
        let (conn, _) = db::open(&dir).unwrap();
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn insertar(conn: &Connection, clave: &str, valor: f64, cuando: &str) {
        use crate::domain::tipos::{
            MetricQuality, MetricSample, MetricSource, MetricTarget, Resolution,
        };
        repo_metricas::insert_sample(
            conn,
            &MetricSample {
                target: MetricTarget::Device("d1".to_string()),
                metric_key: clave.to_string(),
                value_real: Some(valor),
                value_integer: None,
                unit: "count".to_string(),
                sampled_at_utc: cuando.to_string(),
                source: MetricSource::Smartctl,
                quality: MetricQuality::Exact,
                resolution: Resolution::Raw,
            },
        )
        .unwrap();
    }

    #[test]
    fn una_autoevaluacion_fallida_crea_un_grupo_activo_de_health_failed() {
        let conn = conn_de_prueba();
        insertar(&conn, "health_passed", 0.0, "2026-09-04T10:00:00Z");

        evaluar_smart(&conn, "d1", "2026-09-04T10:00:00Z").unwrap();

        let grupo = repo_alertas::get_group(&conn, "smart.health.failed|device:d1")
            .unwrap()
            .expect("smartctl_health_failed no creó grupo");
        assert_eq!(grupo.status, AlertStatus::Active);
    }

    #[test]
    fn tres_lecturas_correctas_seguidas_resuelven_el_grupo() {
        let conn = conn_de_prueba();
        insertar(&conn, "health_passed", 0.0, "2026-09-04T10:00:00Z");
        evaluar_smart(&conn, "d1", "2026-09-04T10:00:00Z").unwrap();

        for (i, cuando) in [
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
            "2026-09-04T10:15:00Z",
        ]
        .into_iter()
        .enumerate()
        {
            insertar(&conn, "health_passed", 1.0, cuando);
            evaluar_smart(&conn, "d1", cuando).unwrap();
            let _ = i;
        }

        let grupo = repo_alertas::get_group(&conn, "smart.health.failed|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(grupo.status, AlertStatus::Resolved);
    }

    #[test]
    fn sin_ninguna_muestra_no_crea_ningun_grupo() {
        let conn = conn_de_prueba();
        evaluar_smart(&conn, "d1", "2026-09-04T10:00:00Z").unwrap();
        assert!(repo_alertas::list_groups_counting_toward_health(&conn)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn spare_por_debajo_del_umbral_tres_ciclos_activa_el_grupo_por_pares() {
        let conn = conn_de_prueba();
        for cuando in [
            "2026-09-04T10:00:00Z",
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
        ] {
            insertar(&conn, "available_spare_percent", 5.0, cuando);
            insertar(&conn, "available_spare_threshold_percent", 10.0, cuando);
            evaluar_smart(&conn, "d1", cuando).unwrap();
        }

        let grupo = repo_alertas::get_group(&conn, "smart.spare_below_threshold|device:d1")
            .unwrap()
            .expect("no se activó smart.spare_below_threshold");
        assert_eq!(grupo.status, AlertStatus::Active);
    }
}
