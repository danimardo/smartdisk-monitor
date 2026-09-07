//! Motor de reglas de alerta: activación, histéresis, agrupación, deduplicación y ciclo de vida.
//!
//! El comportamiento normativo está en `docs/alert-rules.md`, regla por regla; este módulo lo
//! implementa. Cada regla exige cinco pruebas sin excepción (`alert-rules.md` §5): activación,
//! **no** activación ante dato ausente, histéresis, deduplicación y ciclo de recaída
//! (constitución §VIII).

use rusqlite::Connection;

use crate::domain::capacidad::UmbralesCapacidad;
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

/// Umbrales de las reglas SMART que el motor parametriza desde `settings.alerts` (v3, ADR-036). Los
/// resuelve quien orquesta la recopilación (`commands::refresh_smart`), no este módulo — así el motor
/// sigue sin leer la base de datos de ajustes.
#[derive(Debug, Clone, Copy)]
pub struct ConfigUmbrales {
    pub temp_warn_c: f64,
    pub temp_crit_c: f64,
    pub wear_warn_pct: f64,
    pub wear_crit_pct: f64,
    pub media_errors_warn: i64,
    pub media_errors_crit: i64,
}

impl Default for ConfigUmbrales {
    fn default() -> Self {
        use crate::domain::ajustes as aj;
        Self {
            temp_warn_c: aj::TEMP_WARN_DEFAULT_C,
            temp_crit_c: aj::TEMP_CRIT_DEFAULT_C,
            wear_warn_pct: aj::WEAR_WARN_PERCENT_DEFAULT,
            wear_crit_pct: aj::WEAR_CRIT_PERCENT_DEFAULT,
            media_errors_warn: aj::MEDIA_ERRORS_WARN_PER24H_DEFAULT,
            media_errors_crit: aj::MEDIA_ERRORS_CRIT_PER24H_DEFAULT,
        }
    }
}

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
    cfg: &ConfigUmbrales,
) -> rusqlite::Result<Vec<(String, Transicion)>> {
    let serie =
        |clave: &str| repo_metricas::latest_n_values(conn, device_id, clave, MUESTRAS_HISTERESIS);
    let cfg = *cfg;

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
        |s| motor::evaluar_media_errors(s, cfg.media_errors_warn, cfg.media_errors_crit),
        None::<fn(&[f64]) -> bool>,
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "smart.error_log",
        &serie("error_log_entries_total")?,
        motor::evaluar_error_log,
        None::<fn(&[f64]) -> bool>,
        ahora_utc,
    )?);
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "smart.wear_high",
        &serie("percentage_used")?,
        |s| motor::evaluar_wear_high(s, cfg.wear_warn_pct, cfg.wear_crit_pct),
        None::<fn(&[f64]) -> bool>,
        ahora_utc,
    )?);
    // Umbral de aviso de temperatura: si el disco declara el límite operativo del fabricante
    // (`temperature.op_limit_max`, serie `vendor_temp_limit_celsius`), esa es la referencia y se
    // evalúa `temp.above_vendor_limit`; si no, el umbral configurado con `temp.above_configured_warn`
    // (`alert-rules.md` §2, `open-questions.md` J.16). El crítico siempre es el configurado: no hay
    // un crítico del fabricante fiable en el JSON de `smartctl`.
    let limite_fabricante = serie("vendor_temp_limit_celsius")?.first().copied();
    match limite_fabricante {
        Some(umbral) => transiciones.push(aplicar_simple(
            conn,
            device_id,
            "temp.above_vendor_limit",
            &serie("temperature_celsius")?,
            move |s| motor::evaluar_temperatura_configurada_warn(s, umbral),
            Some(move |s: &[f64]| motor::resuelve_temperatura_configurada_warn(s, umbral)),
            ahora_utc,
        )?),
        None => transiciones.push(aplicar_simple(
            conn,
            device_id,
            "temp.above_configured_warn",
            &serie("temperature_celsius")?,
            |s| motor::evaluar_temperatura_configurada_warn(s, cfg.temp_warn_c),
            Some(|s: &[f64]| motor::resuelve_temperatura_configurada_warn(s, cfg.temp_warn_c)),
            ahora_utc,
        )?),
    }
    transiciones.push(aplicar_simple(
        conn,
        device_id,
        "temp.above_configured_crit",
        &serie("temperature_celsius")?,
        |s| motor::evaluar_temperatura_configurada_crit(s, cfg.temp_crit_c),
        Some(|s: &[f64]| motor::resuelve_temperatura_configurada_crit(s, cfg.temp_crit_c)),
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

    // Este ciclo produjo una lectura correcta (llegamos aquí desde `persist_smart_reading`): si
    // había un `smart.unreadable` abierto, esto lo resuelve.
    if let Some(t) = evaluar_unreadable(conn, device_id, ahora_utc)? {
        transiciones.push(t);
    }

    Ok(transiciones)
}

/// `smart.unreadable` (`alert-rules.md` §2): la consulta a `smartctl` falla 3 ciclos seguidos en un
/// disco que **sí** daba datos. Se evalúa aparte de `evaluar_smart` porque hay que evaluarla
/// **también en los ciclos que fallan** —`commands::refresh_smart` la llama en sus ramas de error,
/// donde no hay lectura que persistir—. Devuelve `None` si el disco nunca respondió: eso es «no
/// compatible», no «dejó de responder».
pub fn evaluar_unreadable(
    conn: &Connection,
    device_id: &str,
    ahora_utc: &str,
) -> rusqlite::Result<Option<(String, Transicion)>> {
    if !repo_metricas::hubo_lectura_smart_correcta(conn, device_id)? {
        return Ok(None);
    }
    let serie =
        repo_metricas::latest_n_values(conn, device_id, "smart_query_ok", MUESTRAS_HISTERESIS)?;
    Ok(Some(aplicar_simple(
        conn,
        device_id,
        "smart.unreadable",
        &serie,
        motor::evaluar_unreadable,
        Some(motor::resuelve_unreadable),
        ahora_utc,
    )?))
}

/// Una fuente de métricas vigilada por `collector.stalled`: su nombre estable (va en el `context` y
/// en la clave de deduplicación), cuándo completó un ciclo por última vez (`None` = nunca) y cada
/// cuánto se espera que lo complete.
pub struct FuenteVigilada<'a> {
    pub nombre: &'a str,
    pub last_success_at: Option<&'a str>,
    pub intervalo: time::Duration,
}

/// `collector.stalled` (`alert-rules.md` §2): por cada fuente cuyo último ciclo correcto quede más
/// de 3 intervalos atrás, una advertencia con `context = <nombre de la fuente>` y sin objetivo; si
/// vuelve a completar un ciclo, resuelve. Una fuente que **nunca** tuvo éxito (`last_success_at`
/// nulo) no dispara aquí: de eso ya avisa `source:degraded` / el inventario vacío
/// (`open-questions.md` J.16). La llama `commands::post_procesar_ciclo` con el `source_health` del
/// estado y los intervalos de `planificador.rs`.
pub fn evaluar_collector_stalled(
    conn: &Connection,
    ahora_utc: &str,
    fuentes: &[FuenteVigilada<'_>],
) -> rusqlite::Result<Vec<(String, Transicion)>> {
    let rfc = &time::format_description::well_known::Rfc3339;
    let Ok(ahora) = time::OffsetDateTime::parse(ahora_utc, rfc) else {
        return Ok(vec![]);
    };
    let mut transiciones = Vec::with_capacity(fuentes.len());
    for f in fuentes {
        let Some(ultimo_txt) = f.last_success_at else {
            continue;
        };
        let Ok(ultimo) = time::OffsetDateTime::parse(ultimo_txt, rfc) else {
            continue;
        };
        let estancado = motor::colector_estancado(ahora - ultimo, f.intervalo);
        transiciones.push(aplicar_contexto(
            conn,
            "collector.stalled",
            f.nombre,
            estancado,
            ahora_utc,
        )?);
    }
    Ok(transiciones)
}

/// Como `aplicar_simple` pero para una regla que no apunta a un disco ni a un volumen: solo se
/// distingue por su `context` (`collector.stalled` → nombre de la fuente). Recibe ya decidido si la
/// condición se cumple ahora; la resolución es simplemente su negación.
fn aplicar_contexto(
    conn: &Connection,
    rule_key: &str,
    context: &str,
    activa: bool,
    ahora_utc: &str,
) -> rusqlite::Result<(String, Transicion)> {
    let transicion = agrupacion::procesar(
        conn,
        &EvaluacionAlerta {
            rule_key: rule_key.to_string(),
            target_device_id: None,
            target_volume_id: None,
            context: Some(context.to_string()),
            severity_si_activa: activa.then_some(AlertSeverity::Warning),
            resuelto: !activa,
            value: None,
            occurred_at_utc: ahora_utc.to_string(),
        },
    )?;
    let clave = agrupacion::deduplication_key(rule_key, None, None, Some(context));
    Ok((clave, transicion))
}

/// Evalúa `capacity.low` y `capacity.critical` para un volumen, sobre la serie de `volume_free_bytes`
/// ya persistida (v3, ADR-036). `capacity_bytes` es la capacidad actual del volumen (cambia poco);
/// se pasa aparte para no exigir una segunda serie. Se llama tras la reconciliación de inventario,
/// una vez por volumen enlazado a un disco monitorizado.
pub fn evaluar_capacidad(
    conn: &Connection,
    volume_id: &str,
    capacity_bytes: Option<i64>,
    ahora_utc: &str,
    umbrales: &UmbralesCapacidad,
) -> rusqlite::Result<Vec<(String, Transicion)>> {
    let libres = repo_metricas::latest_n_values_volume(
        conn,
        volume_id,
        "volume_free_bytes",
        MUESTRAS_HISTERESIS,
    )?;
    let cap = capacity_bytes.unwrap_or(0) as f64;
    let pares: Vec<(f64, f64)> = libres.iter().map(|&f| (f, cap)).collect();
    let u = *umbrales;

    let low = aplicar_par_volumen(
        conn,
        volume_id,
        "capacity.low",
        &pares,
        |p| motor::evaluar_capacidad_low(p, &u),
        Some(|p: &[(f64, f64)]| motor::resuelve_capacidad_low(p, &u)),
        ahora_utc,
    )?;
    let critical = aplicar_par_volumen(
        conn,
        volume_id,
        "capacity.critical",
        &pares,
        |p| motor::evaluar_capacidad_critical(p, &u),
        Some(|p: &[(f64, f64)]| motor::resuelve_capacidad_critical(p, &u)),
        ahora_utc,
    )?;
    Ok(vec![low, critical])
}

fn aplicar_simple(
    conn: &Connection,
    device_id: &str,
    rule_key: &str,
    serie: &[f64],
    evaluar: impl Fn(&[f64]) -> Option<AlertSeverity>,
    resuelve: Option<impl Fn(&[f64]) -> bool>,
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

/// Como `aplicar_par` pero para una regla que apunta a un **volumen**, no a un dispositivo.
fn aplicar_par_volumen(
    conn: &Connection,
    volume_id: &str,
    rule_key: &str,
    pares: &[(f64, f64)],
    evaluar: impl Fn(&[(f64, f64)]) -> Option<AlertSeverity>,
    resuelve: Option<impl Fn(&[(f64, f64)]) -> bool>,
    ahora_utc: &str,
) -> rusqlite::Result<(String, Transicion)> {
    let severidad = evaluar(pares);
    let resuelto = severidad.is_none() && resuelve.is_some_and(|f| f(pares));
    let transicion = agrupacion::procesar(
        conn,
        &EvaluacionAlerta {
            rule_key: rule_key.to_string(),
            target_device_id: None,
            target_volume_id: Some(volume_id.to_string()),
            context: None,
            severity_si_activa: severidad,
            resuelto,
            value: pares.first().map(|&(libre, _)| libre),
            occurred_at_utc: ahora_utc.to_string(),
        },
    )?;
    let clave = agrupacion::deduplication_key(rule_key, None, Some(volume_id), None);
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

    /// Marca que el disco `d1` sí respondió alguna vez (compuerta de `smart.unreadable`).
    fn snapshot_ok(conn: &Connection, cuando: &str) {
        repo_metricas::insert_smart_snapshot(conn, "d1", cuando, None, Some(0), "ok", None, None)
            .unwrap();
    }

    /// Un ciclo de SMART fallido, tal cual lo hace `commands::refresh_smart`: registra la
    /// ilegibilidad y evalúa la regla.
    fn ciclo_fallido(conn: &Connection, cuando: &str) {
        insertar(conn, "smart_query_ok", 0.0, cuando);
        evaluar_unreadable(conn, "d1", cuando).unwrap();
    }

    #[test]
    fn smart_unreadable_se_activa_tras_tres_ciclos_fallidos_en_un_disco_que_si_respondia() {
        let conn = conn_de_prueba();
        snapshot_ok(&conn, "2026-09-04T09:00:00Z");
        for cuando in [
            "2026-09-04T10:00:00Z",
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
        ] {
            ciclo_fallido(&conn, cuando);
        }
        let g = repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .expect("no se activó smart.unreadable");
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.severity, AlertSeverity::Warning);
    }

    #[test]
    fn smart_unreadable_no_se_activa_sin_ninguna_muestra() {
        let conn = conn_de_prueba();
        snapshot_ok(&conn, "2026-09-04T09:00:00Z");
        evaluar_unreadable(&conn, "d1", "2026-09-04T10:00:00Z").unwrap();
        assert!(repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .is_none());
    }

    #[test]
    fn smart_unreadable_no_se_activa_en_un_disco_que_nunca_respondio() {
        // Sin `snapshot_ok`: el disco tiene ruta de smartctl pero jamás dio datos. Eso es «no
        // compatible», no «dejó de responder».
        let conn = conn_de_prueba();
        for cuando in [
            "2026-09-04T10:00:00Z",
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
        ] {
            ciclo_fallido(&conn, cuando);
        }
        assert!(repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .is_none());
    }

    #[test]
    fn smart_unreadable_resuelve_con_una_lectura_correcta_y_no_reabre_por_oscilar() {
        let conn = conn_de_prueba();
        snapshot_ok(&conn, "2026-09-04T09:00:00Z");
        for cuando in [
            "2026-09-04T10:00:00Z",
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
        ] {
            ciclo_fallido(&conn, cuando);
        }
        // Una lectura correcta: resuelve.
        insertar(&conn, "smart_query_ok", 1.0, "2026-09-04T10:15:00Z");
        evaluar_unreadable(&conn, "d1", "2026-09-04T10:15:00Z").unwrap();
        let g = repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Resolved);

        // Un fallo suelto no reabre (hacen falta 3 seguidos otra vez).
        ciclo_fallido(&conn, "2026-09-04T10:20:00Z");
        let g = repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(
            g.status,
            AlertStatus::Resolved,
            "un fallo aislado no reabre"
        );
    }

    #[test]
    fn smart_unreadable_deduplica_las_evaluaciones_equivalentes_en_un_solo_grupo() {
        let conn = conn_de_prueba();
        snapshot_ok(&conn, "2026-09-04T09:00:00Z");
        for m in 0..5 {
            ciclo_fallido(&conn, &format!("2026-09-04T10:{m:02}:00Z"));
        }
        let g = repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert!(
            g.occurrence_count >= 3,
            "no deduplicó: {}",
            g.occurrence_count
        );
    }

    #[test]
    fn smart_unreadable_al_recaer_incrementa_el_ciclo_y_conserva_el_contador() {
        let conn = conn_de_prueba();
        snapshot_ok(&conn, "2026-09-04T09:00:00Z");
        // episodio 1
        for m in ["00", "05", "10"] {
            ciclo_fallido(&conn, &format!("2026-09-04T10:{m}:00Z"));
        }
        insertar(&conn, "smart_query_ok", 1.0, "2026-09-04T10:15:00Z");
        evaluar_unreadable(&conn, "d1", "2026-09-04T10:15:00Z").unwrap();
        let g1 = repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .unwrap();
        let cuenta_1 = g1.occurrence_count;
        assert_eq!(g1.status, AlertStatus::Resolved);

        // episodio 2: recae
        for m in ["20", "25", "30"] {
            ciclo_fallido(&conn, &format!("2026-09-04T10:{m}:00Z"));
        }
        let g2 = repo_alertas::get_group(&conn, "smart.unreadable|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g2.status, AlertStatus::Active);
        assert_eq!(g2.cycle, 2, "una recaída incrementa el ciclo");
        assert!(
            g2.occurrence_count > cuenta_1,
            "el contador histórico se conserva y sigue subiendo"
        );
    }

    #[test]
    fn una_autoevaluacion_fallida_crea_un_grupo_activo_de_health_failed() {
        let conn = conn_de_prueba();
        insertar(&conn, "health_passed", 0.0, "2026-09-04T10:00:00Z");

        evaluar_smart(
            &conn,
            "d1",
            "2026-09-04T10:00:00Z",
            &ConfigUmbrales::default(),
        )
        .unwrap();

        let grupo = repo_alertas::get_group(&conn, "smart.health.failed|device:d1")
            .unwrap()
            .expect("smartctl_health_failed no creó grupo");
        assert_eq!(grupo.status, AlertStatus::Active);
    }

    #[test]
    fn tres_lecturas_correctas_seguidas_resuelven_el_grupo() {
        let conn = conn_de_prueba();
        insertar(&conn, "health_passed", 0.0, "2026-09-04T10:00:00Z");
        evaluar_smart(
            &conn,
            "d1",
            "2026-09-04T10:00:00Z",
            &ConfigUmbrales::default(),
        )
        .unwrap();

        for (i, cuando) in [
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
            "2026-09-04T10:15:00Z",
        ]
        .into_iter()
        .enumerate()
        {
            insertar(&conn, "health_passed", 1.0, cuando);
            evaluar_smart(&conn, "d1", cuando, &ConfigUmbrales::default()).unwrap();
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
        evaluar_smart(
            &conn,
            "d1",
            "2026-09-04T10:00:00Z",
            &ConfigUmbrales::default(),
        )
        .unwrap();
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
            evaluar_smart(&conn, "d1", cuando, &ConfigUmbrales::default()).unwrap();
        }

        let grupo = repo_alertas::get_group(&conn, "smart.spare_below_threshold|device:d1")
            .unwrap()
            .expect("no se activó smart.spare_below_threshold");
        assert_eq!(grupo.status, AlertStatus::Active);
    }

    // ---- v3: umbrales configurables (ADR-036) ----

    #[test]
    fn la_temperatura_de_aviso_configurada_decide_si_se_activa() {
        let conn = conn_de_prueba();
        for cuando in [
            "2026-09-04T10:00:00Z",
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
        ] {
            insertar(&conn, "temperature_celsius", 62.0, cuando);
        }
        // Con el umbral de fábrica v2 (70) no salta; con el de v3/«Equilibrado» (60) sí.
        let cfg_v2 = ConfigUmbrales {
            temp_warn_c: 70.0,
            ..ConfigUmbrales::default()
        };
        evaluar_smart(&conn, "d1", "2026-09-04T10:10:00Z", &cfg_v2).unwrap();
        assert!(
            repo_alertas::get_group(&conn, "temp.above_configured_warn|device:d1")
                .unwrap()
                .is_none_or(|g| g.status != AlertStatus::Active)
        );

        evaluar_smart(
            &conn,
            "d1",
            "2026-09-04T10:10:00Z",
            &ConfigUmbrales::default(),
        )
        .unwrap();
        let g = repo_alertas::get_group(&conn, "temp.above_configured_warn|device:d1")
            .unwrap()
            .expect("con umbral 60 °C debería activarse a 62 °C");
        assert_eq!(g.status, AlertStatus::Active);
    }

    // ---- capacity.low / capacity.critical (v3) ----

    fn conn_con_volumen() -> Connection {
        let conn = conn_de_prueba();
        conn.execute(
            "INSERT INTO volumes (id, volume_guid, first_seen_at, last_seen_at)
             VALUES ('v1', 'v1', '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn insertar_libres(conn: &Connection, bytes: f64, cuando: &str) {
        use crate::domain::tipos::{
            MetricQuality, MetricSample, MetricSource, MetricTarget, Resolution,
        };
        repo_metricas::insert_sample(
            conn,
            &MetricSample {
                target: MetricTarget::Volume("v1".to_string()),
                metric_key: "volume_free_bytes".to_string(),
                value_real: Some(bytes),
                value_integer: None,
                unit: "bytes".to_string(),
                sampled_at_utc: cuando.to_string(),
                source: MetricSource::WindowsStorage,
                quality: MetricQuality::Exact,
                resolution: Resolution::Raw,
            },
        )
        .unwrap();
    }

    #[test]
    fn un_volumen_por_debajo_del_diez_por_ciento_activa_capacity_low() {
        let conn = conn_con_volumen();
        let cap = 100i64 * 1024 * 1024 * 1024;
        insertar_libres(
            &conn,
            8.0 * 1024.0 * 1024.0 * 1024.0,
            "2026-09-04T10:00:00Z",
        );
        evaluar_capacidad(
            &conn,
            "v1",
            Some(cap),
            "2026-09-04T10:00:00Z",
            &UmbralesCapacidad::default(),
        )
        .unwrap();
        let g = repo_alertas::get_group(&conn, "capacity.low|volume:v1")
            .unwrap()
            .expect("no se activó capacity.low");
        assert_eq!(g.status, AlertStatus::Active);
    }

    #[test]
    fn capacity_low_no_se_activa_sin_ninguna_lectura() {
        let conn = conn_con_volumen();
        evaluar_capacidad(
            &conn,
            "v1",
            Some(1_000),
            "2026-09-04T10:00:00Z",
            &UmbralesCapacidad::default(),
        )
        .unwrap();
        assert!(repo_alertas::get_group(&conn, "capacity.low|volume:v1")
            .unwrap()
            .is_none());
    }

    #[test]
    fn capacity_low_deduplica_y_resuelve_tras_tres_lecturas_en_ok() {
        let conn = conn_con_volumen();
        let cap = 100i64 * 1024 * 1024 * 1024;
        let g = 1024.0 * 1024.0 * 1024.0;
        // dos ciclos en aviso: un solo grupo, con contador 2
        for cuando in ["2026-09-04T10:00:00Z", "2026-09-04T10:05:00Z"] {
            insertar_libres(&conn, 8.0 * g, cuando);
            evaluar_capacidad(
                &conn,
                "v1",
                Some(cap),
                cuando,
                &UmbralesCapacidad::default(),
            )
            .unwrap();
        }
        let dedup = repo_alertas::get_group(&conn, "capacity.low|volume:v1")
            .unwrap()
            .unwrap();
        assert_eq!(dedup.status, AlertStatus::Active);
        assert!(
            dedup.occurrence_count >= 2,
            "no deduplicó: {}",
            dedup.occurrence_count
        );

        // tres lecturas en ok ⇒ resuelve
        for cuando in [
            "2026-09-04T10:10:00Z",
            "2026-09-04T10:15:00Z",
            "2026-09-04T10:20:00Z",
        ] {
            insertar_libres(&conn, 50.0 * g, cuando);
            evaluar_capacidad(
                &conn,
                "v1",
                Some(cap),
                cuando,
                &UmbralesCapacidad::default(),
            )
            .unwrap();
        }
        let g2 = repo_alertas::get_group(&conn, "capacity.low|volume:v1")
            .unwrap()
            .unwrap();
        assert_eq!(g2.status, AlertStatus::Resolved);
    }

    // ---- temp.above_vendor_limit (v3) ----

    /// Inserta el límite del fabricante y N lecturas de temperatura, evaluando `evaluar_smart` en
    /// cada una (como haría `refresh_smart`).
    fn ciclos_de_temperatura(conn: &Connection, limite: Option<f64>, temps: &[(&str, f64)]) {
        for (cuando, t) in temps {
            if let Some(l) = limite {
                insertar(conn, "vendor_temp_limit_celsius", l, cuando);
            }
            insertar(conn, "temperature_celsius", *t, cuando);
            evaluar_smart(conn, "d1", cuando, &ConfigUmbrales::default()).unwrap();
        }
    }

    #[test]
    fn temp_above_vendor_limit_se_activa_tras_tres_ciclos_por_encima_del_limite_del_fabricante() {
        let conn = conn_de_prueba();
        ciclos_de_temperatura(
            &conn,
            Some(65.0),
            &[
                ("2026-09-04T10:00:00Z", 68.0),
                ("2026-09-04T10:05:00Z", 68.0),
                ("2026-09-04T10:10:00Z", 68.0),
            ],
        );
        let g = repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
            .unwrap()
            .expect("no se activó temp.above_vendor_limit");
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.severity, AlertSeverity::Warning);
    }

    #[test]
    fn temp_above_vendor_limit_no_se_activa_sin_ninguna_lectura_de_temperatura() {
        let conn = conn_de_prueba();
        // Solo el límite, ninguna temperatura.
        insertar(
            &conn,
            "vendor_temp_limit_celsius",
            65.0,
            "2026-09-04T10:00:00Z",
        );
        evaluar_smart(
            &conn,
            "d1",
            "2026-09-04T10:00:00Z",
            &ConfigUmbrales::default(),
        )
        .unwrap();
        assert!(
            repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn temp_above_vendor_limit_respeta_la_histeresis_de_tres_ciclos() {
        let conn = conn_de_prueba();
        ciclos_de_temperatura(
            &conn,
            Some(65.0),
            &[
                ("2026-09-04T10:00:00Z", 68.0),
                ("2026-09-04T10:05:00Z", 68.0),
            ],
        );
        assert!(
            repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
                .unwrap()
                .is_none_or(|g| g.status != AlertStatus::Active),
            "con dos ciclos todavía no"
        );
        ciclos_de_temperatura(&conn, Some(65.0), &[("2026-09-04T10:10:00Z", 68.0)]);
        let g = repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
            .unwrap()
            .expect("al tercer ciclo sí");
        assert_eq!(g.status, AlertStatus::Active);
    }

    #[test]
    fn temp_above_vendor_limit_deduplica_en_un_solo_grupo() {
        let conn = conn_de_prueba();
        let temps: Vec<(String, f64)> = (0..5)
            .map(|m| (format!("2026-09-04T10:{m:02}:00Z"), 68.0))
            .collect();
        let refs: Vec<(&str, f64)> = temps.iter().map(|(s, t)| (s.as_str(), *t)).collect();
        ciclos_de_temperatura(&conn, Some(65.0), &refs);
        let g = repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert!(
            g.occurrence_count >= 3,
            "no deduplicó: {}",
            g.occurrence_count
        );
    }

    #[test]
    fn temp_above_vendor_limit_resuelve_con_margen_y_puede_recaer() {
        let conn = conn_de_prueba();
        // episodio 1
        ciclos_de_temperatura(
            &conn,
            Some(65.0),
            &[
                ("2026-09-04T10:00:00Z", 68.0),
                ("2026-09-04T10:05:00Z", 68.0),
                ("2026-09-04T10:10:00Z", 68.0),
            ],
        );
        // 3 ciclos a ≤ (65 − 3) = 62 → resuelve
        ciclos_de_temperatura(
            &conn,
            Some(65.0),
            &[
                ("2026-09-04T10:15:00Z", 60.0),
                ("2026-09-04T10:20:00Z", 60.0),
                ("2026-09-04T10:25:00Z", 60.0),
            ],
        );
        let g = repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Resolved);

        // recae
        ciclos_de_temperatura(
            &conn,
            Some(65.0),
            &[
                ("2026-09-04T10:30:00Z", 70.0),
                ("2026-09-04T10:35:00Z", 70.0),
                ("2026-09-04T10:40:00Z", 70.0),
            ],
        );
        let g = repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.cycle, 2);
    }

    #[test]
    fn un_disco_con_limite_del_fabricante_no_dispara_temp_above_configured_warn() {
        let conn = conn_de_prueba();
        // Límite del fabricante 75; umbral configurado de fábrica 60. Temperatura 68: por encima
        // del configurado, por debajo del fabricante → no debe saltar ninguna de las dos.
        ciclos_de_temperatura(
            &conn,
            Some(75.0),
            &[
                ("2026-09-04T10:00:00Z", 68.0),
                ("2026-09-04T10:05:00Z", 68.0),
                ("2026-09-04T10:10:00Z", 68.0),
            ],
        );
        assert!(
            repo_alertas::get_group(&conn, "temp.above_configured_warn|device:d1")
                .unwrap()
                .is_none(),
            "con límite del fabricante conocido, `temp.above_configured_warn` no se evalúa"
        );
        assert!(
            repo_alertas::get_group(&conn, "temp.above_vendor_limit|device:d1")
                .unwrap()
                .is_none_or(|g| g.status != AlertStatus::Active),
            "68 °C está por debajo del límite del fabricante (75): tampoco salta la del fabricante"
        );
    }

    // ---- collector.stalled (v3) ----

    #[test]
    fn collector_stalled_se_activa_pasados_mas_de_tres_intervalos_desde_el_ultimo_exito() {
        let conn = conn_de_prueba();
        let fuentes = [FuenteVigilada {
            nombre: "smartctl",
            last_success_at: Some("2026-09-04T09:00:00Z"),
            intervalo: time::Duration::minutes(5),
        }];
        // 40 min después: 8× el intervalo → estancado.
        let t = evaluar_collector_stalled(&conn, "2026-09-04T09:40:00Z", &fuentes).unwrap();
        assert_eq!(t.len(), 1);
        let g = repo_alertas::get_group(&conn, "collector.stalled|sin_objetivo|smartctl")
            .unwrap()
            .expect("no se activó collector.stalled");
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.severity, AlertSeverity::Warning);
    }

    #[test]
    fn collector_stalled_no_dispara_si_la_fuente_nunca_tuvo_exito() {
        let conn = conn_de_prueba();
        let fuentes = [FuenteVigilada {
            nombre: "smartctl",
            last_success_at: None,
            intervalo: time::Duration::minutes(5),
        }];
        evaluar_collector_stalled(&conn, "2026-09-04T09:40:00Z", &fuentes).unwrap();
        assert!(
            repo_alertas::get_group(&conn, "collector.stalled|sin_objetivo|smartctl")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn collector_stalled_resuelve_cuando_la_fuente_vuelve_a_completar_un_ciclo() {
        let conn = conn_de_prueba();
        let estancada = [FuenteVigilada {
            nombre: "smartctl",
            last_success_at: Some("2026-09-04T09:00:00Z"),
            intervalo: time::Duration::minutes(5),
        }];
        evaluar_collector_stalled(&conn, "2026-09-04T09:40:00Z", &estancada).unwrap();

        let recuperada = [FuenteVigilada {
            nombre: "smartctl",
            last_success_at: Some("2026-09-04T09:44:00Z"),
            intervalo: time::Duration::minutes(5),
        }];
        evaluar_collector_stalled(&conn, "2026-09-04T09:45:00Z", &recuperada).unwrap();
        let g = repo_alertas::get_group(&conn, "collector.stalled|sin_objetivo|smartctl")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Resolved);
    }

    #[test]
    fn collector_stalled_deduplica_por_fuente() {
        let conn = conn_de_prueba();
        let fuentes = [FuenteVigilada {
            nombre: "perf-counter",
            last_success_at: Some("2026-09-04T09:00:00Z"),
            intervalo: time::Duration::seconds(30),
        }];
        for cuando in [
            "2026-09-04T09:40:00Z",
            "2026-09-04T09:41:00Z",
            "2026-09-04T09:42:00Z",
        ] {
            evaluar_collector_stalled(&conn, cuando, &fuentes).unwrap();
        }
        let g = repo_alertas::get_group(&conn, "collector.stalled|sin_objetivo|perf-counter")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert!(
            g.occurrence_count >= 3,
            "no deduplicó: {}",
            g.occurrence_count
        );
    }

    #[test]
    fn collector_stalled_una_fuente_al_dia_y_otra_recien_leida_solo_activa_la_primera() {
        let conn = conn_de_prueba();
        let fuentes = [
            FuenteVigilada {
                nombre: "smartctl",
                last_success_at: Some("2026-09-04T09:00:00Z"),
                intervalo: time::Duration::minutes(5),
            },
            FuenteVigilada {
                nombre: "perf-counter",
                last_success_at: Some("2026-09-04T09:39:30Z"),
                intervalo: time::Duration::seconds(30),
            },
        ];
        evaluar_collector_stalled(&conn, "2026-09-04T09:40:00Z", &fuentes).unwrap();
        assert!(
            repo_alertas::get_group(&conn, "collector.stalled|sin_objetivo|smartctl")
                .unwrap()
                .is_some_and(|g| g.status == AlertStatus::Active)
        );
        assert!(
            repo_alertas::get_group(&conn, "collector.stalled|sin_objetivo|perf-counter")
                .unwrap()
                .is_none_or(|g| g.status != AlertStatus::Active)
        );
    }
}
