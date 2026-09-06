//! Evaluación pura de reglas de alerta: activación e histéresis por valor, sin persistencia.
//!
//! **Alcance de esta primera versión** (`docs/open-questions.md` J.16): solo las reglas que se
//! evalúan sobre datos de `smartctl` ya persistidos. Las que dependen de un colector todavía sin
//! construir —eventos de Windows, capacidad de volumen, límite del fabricante, estado del
//! recopilador— quedan fuera a propósito, no simuladas con datos inventados.
//!
//! Cada función recibe la serie de valores **ya leída de `metric_samples`, ordenada de más
//! reciente a más antigua**, y no toca la base de datos: eso es responsabilidad de
//! `alerts::agrupacion`, que sabe además de deduplicación y de cuándo una resolución depende del
//! tiempo transcurrido en vez de del valor.

use crate::domain::capacidad::{estado_capacidad, UmbralesCapacidad};
use crate::domain::tipos::{AlertSeverity, HealthState};

/// Las N muestras más recientes cumplen todas el predicado. Si hay menos de N, no cumple: un
/// contador que falta no se evalúa (`alert-rules.md`, "Lo que explícitamente NO genera alerta").
fn primeras_n_cumplen(serie: &[f64], n: usize, cumple: impl Fn(f64) -> bool) -> bool {
    serie.len() >= n && serie[..n].iter().all(|&v| cumple(v))
}

/// `smart.health.failed`: inmediato, sin histéresis de activación.
pub fn evaluar_health_failed(serie: &[f64]) -> Option<AlertSeverity> {
    primeras_n_cumplen(serie, 1, |v| v == 0.0).then_some(AlertSeverity::Critical)
}

/// Resolución de `smart.health.failed`: `health_passed = true` durante 3 ciclos.
pub fn resuelve_health_failed(serie: &[f64]) -> bool {
    primeras_n_cumplen(serie, 3, |v| v == 1.0)
}

/// `nvme.critical_warning`: inmediato, cualquier bit activo.
pub fn evaluar_critical_warning(serie: &[f64]) -> Option<AlertSeverity> {
    primeras_n_cumplen(serie, 1, |v| v != 0.0).then_some(AlertSeverity::Critical)
}

/// Resolución de `nvme.critical_warning`: `= 0` durante 3 ciclos.
pub fn resuelve_critical_warning(serie: &[f64]) -> bool {
    primeras_n_cumplen(serie, 3, |v| v == 0.0)
}

/// `smart.wear_high`: inmediato, dos niveles. **No se resuelve sola** — el desgaste no baja — se
/// archiva a mano (`alert-rules.md`), así que no existe una función `resuelve_wear_high`.
/// v3 (ADR-036): los umbrales llegan de `settings.alerts` (fábrica 80/90), ya no `90/100` literal.
pub fn evaluar_wear_high(serie: &[f64], warn_pct: f64, crit_pct: f64) -> Option<AlertSeverity> {
    let v = *serie.first()?;
    if v >= crit_pct {
        Some(AlertSeverity::Critical)
    } else if v >= warn_pct {
        Some(AlertSeverity::Warning)
    } else {
        None
    }
}

/// `temp.above_configured_warn`: sin límite del fabricante, 3 ciclos consecutivos por encima del
/// umbral configurado (`settings.alerts.temp_configured_warn_c`, fábrica 60 °C — ADR-036).
/// **Regla independiente** de `temp.above_configured_crit`: la tabla normativa les da activación,
/// severidad y resolución propias (`domain::salud::device_state` colapsa al peor).
pub fn evaluar_temperatura_configurada_warn(serie: &[f64], warn_c: f64) -> Option<AlertSeverity> {
    primeras_n_cumplen(serie, 3, |v| v > warn_c).then_some(AlertSeverity::Warning)
}

/// Resolución de `temp.above_configured_warn`: ≤ (umbral − 3 °C) durante 3 ciclos. El margen de 3 °C
/// es el mismo que tenía la histéresis literal (70 → 67).
pub fn resuelve_temperatura_configurada_warn(serie: &[f64], warn_c: f64) -> bool {
    primeras_n_cumplen(serie, 3, |v| v <= warn_c - 3.0)
}

/// `temp.above_configured_crit`: crítico **inmediato** en ≥ umbral configurado
/// (`temp_configured_crit_c`, fábrica 70 °C), sin ciclos de histéresis para activarse.
pub fn evaluar_temperatura_configurada_crit(serie: &[f64], crit_c: f64) -> Option<AlertSeverity> {
    primeras_n_cumplen(serie, 1, |v| v >= crit_c).then_some(AlertSeverity::Critical)
}

/// Resolución de `temp.above_configured_crit`: ≤ (umbral − 5 °C) durante 3 ciclos (margen de 80 → 75).
pub fn resuelve_temperatura_configurada_crit(serie: &[f64], crit_c: f64) -> bool {
    primeras_n_cumplen(serie, 3, |v| v <= crit_c - 5.0)
}

/// `smart.spare_below_threshold`: 3 ciclos consecutivos con la reserva por debajo de su propio
/// umbral. `pares` son `(available_spare_percent, available_spare_threshold_percent)` de la misma
/// lectura, más reciente primero — dos métricas que solo tienen sentido comparadas juntas.
pub fn evaluar_spare_below_threshold(pares: &[(f64, f64)]) -> Option<AlertSeverity> {
    (pares.len() >= 3 && pares[..3].iter().all(|&(spare, umbral)| spare < umbral))
        .then_some(AlertSeverity::Critical)
}

/// Resolución: por encima del umbral **más 2 puntos**, durante 3 ciclos.
pub fn resuelve_spare_below_threshold(pares: &[(f64, f64)]) -> bool {
    pares.len() >= 3
        && pares[..3]
            .iter()
            .all(|&(spare, umbral)| spare >= umbral + 2.0)
}

/// `smart.media_errors`: se activa según la **magnitud del incremento** de `media_errors_total`
/// entre las dos lecturas más recientes (clarify Q1, ADR-036): `>= crit` ⇒ crítico, `>= warn` ⇒
/// advertencia. Sin lectura previa no hay incremento que medir (`alert-rules.md`, última viñeta de
/// "NO genera alerta"): con menos de dos muestras, no se evalúa. Un incremento nulo o negativo (el
/// contador no baja salvo que se sustituya el disco) tampoco.
pub fn evaluar_media_errors(serie: &[f64], warn: i64, crit: i64) -> Option<AlertSeverity> {
    if serie.len() < 2 {
        return None;
    }
    let incremento = serie[0] - serie[1];
    if incremento >= crit as f64 {
        Some(AlertSeverity::Critical)
    } else if incremento >= warn as f64 {
        Some(AlertSeverity::Warning)
    } else {
        None
    }
}

/// `smart.error_log`: aumenta ⇒ advertencia; si son **tres aumentos consecutivos** ⇒ crítico.
/// "Tres aumentos consecutivos" se interpreta como tres deltas positivos seguidos entre lecturas
/// consecutivas (`serie[0]>serie[1]>serie[2]>serie[3]`), no como "más alto que hace tres ciclos":
/// la tabla normativa no lo precisa más, y esta es la lectura literal de "aumenta … seguidos".
pub fn evaluar_error_log(serie: &[f64]) -> Option<AlertSeverity> {
    if serie.len() < 2 || serie[0] <= serie[1] {
        return None;
    }
    let tres_aumentos_seguidos =
        serie.len() >= 4 && serie[0] > serie[1] && serie[1] > serie[2] && serie[2] > serie[3];
    Some(if tres_aumentos_seguidos {
        AlertSeverity::Critical
    } else {
        AlertSeverity::Warning
    })
}

/// `capacity.low` / `capacity.critical` (v3, ADR-036 — antes no implementadas en el motor).
/// `pares` son `(free_bytes, capacity_bytes)` de cada lectura de `volume_free_bytes`, más reciente
/// primero. La capacidad se repite en cada par (cambia poco); se pasa así para reutilizar el patrón
/// `aplicar_par`. Sin ninguna lectura, no se evalúa.
pub fn evaluar_capacidad_low(pares: &[(f64, f64)], u: &UmbralesCapacidad) -> Option<AlertSeverity> {
    let (free, cap) = *pares.first()?;
    match estado_capacidad(Some(free as i64), Some(cap as i64), u) {
        HealthState::Warn => Some(AlertSeverity::Warning),
        // Cuando la capacidad está en crítico, `capacity.low` **no** se activa: es `capacity.critical`
        // quien manda. Dos grupos independientes, como temperatura (`domain::salud` colapsa al peor).
        _ => None,
    }
}

/// Resolución de `capacity.low`: las 3 lecturas más recientes dan `ok` (`alert-rules.md`: «vuelve a
/// `ok` **y** se mantiene 3 ciclos»).
pub fn resuelve_capacidad_low(pares: &[(f64, f64)], u: &UmbralesCapacidad) -> bool {
    pares.len() >= 3
        && pares[..3]
            .iter()
            .all(|&(f, c)| estado_capacidad(Some(f as i64), Some(c as i64), u) == HealthState::Ok)
}

/// `capacity.critical`: crítico inmediato cuando `estado_capacidad` da `crit`.
pub fn evaluar_capacidad_critical(
    pares: &[(f64, f64)],
    u: &UmbralesCapacidad,
) -> Option<AlertSeverity> {
    let (free, cap) = *pares.first()?;
    (estado_capacidad(Some(free as i64), Some(cap as i64), u) == HealthState::Crit)
        .then_some(AlertSeverity::Critical)
}

/// Resolución de `capacity.critical`: las 3 lecturas más recientes dan `warn` u `ok` (`alert-rules.md`:
/// «sube a `warn` u `ok` y se mantiene 3 ciclos»).
pub fn resuelve_capacidad_critical(pares: &[(f64, f64)], u: &UmbralesCapacidad) -> bool {
    pares.len() >= 3
        && pares[..3].iter().all(|&(f, c)| {
            !matches!(
                estado_capacidad(Some(f as i64), Some(c as i64), u),
                HealthState::Crit
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: f64 = (1024 * 1024 * 1024) as f64;

    // ---- smart.health.failed ----

    #[test]
    fn health_failed_se_activa_con_una_sola_lectura_en_falso() {
        assert_eq!(evaluar_health_failed(&[0.0]), Some(AlertSeverity::Critical));
    }

    #[test]
    fn health_failed_no_se_activa_sin_ninguna_lectura() {
        assert_eq!(
            evaluar_health_failed(&[]),
            None,
            "un contador ausente no se evalúa"
        );
    }

    #[test]
    fn health_failed_no_se_activa_si_la_autoevaluacion_es_correcta() {
        assert_eq!(evaluar_health_failed(&[1.0]), None);
    }

    #[test]
    fn health_failed_resuelve_solo_tras_tres_lecturas_correctas_seguidas() {
        assert!(!resuelve_health_failed(&[1.0, 1.0]), "dos no bastan");
        assert!(resuelve_health_failed(&[1.0, 1.0, 1.0]));
    }

    #[test]
    fn health_failed_no_resuelve_si_la_mas_reciente_vuelve_a_fallar() {
        assert!(!resuelve_health_failed(&[0.0, 1.0, 1.0]));
    }

    // ---- nvme.critical_warning ----

    #[test]
    fn critical_warning_se_activa_con_cualquier_bit_no_nulo() {
        assert_eq!(
            evaluar_critical_warning(&[4.0]),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn critical_warning_no_se_activa_sin_datos() {
        assert_eq!(evaluar_critical_warning(&[]), None);
    }

    #[test]
    fn critical_warning_resuelve_tras_tres_ceros_seguidos() {
        assert!(!resuelve_critical_warning(&[0.0, 0.0]));
        assert!(resuelve_critical_warning(&[0.0, 0.0, 0.0]));
    }

    // ---- smart.wear_high (umbrales de settings, fábrica 80/90) ----

    #[test]
    fn wear_high_advierte_en_el_umbral_justo_por_encima_y_por_debajo() {
        assert_eq!(
            evaluar_wear_high(&[80.0], 80.0, 90.0),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(
            evaluar_wear_high(&[80.1], 80.0, 90.0),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(evaluar_wear_high(&[79.9], 80.0, 90.0), None);
    }

    #[test]
    fn wear_high_escala_a_critico_en_su_umbral() {
        assert_eq!(
            evaluar_wear_high(&[90.0], 80.0, 90.0),
            Some(AlertSeverity::Critical)
        );
        assert_eq!(
            evaluar_wear_high(&[89.9], 80.0, 90.0),
            Some(AlertSeverity::Warning)
        );
    }

    #[test]
    fn wear_high_no_se_activa_sin_lectura() {
        assert_eq!(evaluar_wear_high(&[], 80.0, 90.0), None);
    }

    #[test]
    fn wear_high_respeta_umbrales_configurables() {
        // 88 %: crítico con «Solo lo grave» (90/95)? no, aviso. Con «Prudente» (70/85): crítico.
        assert_eq!(evaluar_wear_high(&[88.0], 90.0, 95.0), None);
        assert_eq!(
            evaluar_wear_high(&[88.0], 70.0, 85.0),
            Some(AlertSeverity::Critical)
        );
    }

    // ---- temp.above_configured_warn (umbral de settings, fábrica 60 °C) ----

    #[test]
    fn temperatura_warn_advierte_en_el_umbral_justo_por_encima_y_por_debajo() {
        assert_eq!(
            evaluar_temperatura_configurada_warn(&[61.0, 62.0, 63.0], 60.0),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(
            evaluar_temperatura_configurada_warn(&[60.0, 60.0, 60.0], 60.0),
            None,
            "en el umbral exacto no (operador estricto)"
        );
    }

    #[test]
    fn temperatura_warn_no_se_activa_con_solo_dos_ciclos() {
        assert_eq!(
            evaluar_temperatura_configurada_warn(&[61.0, 62.0], 60.0),
            None,
            "faltan ciclos, dato incompleto"
        );
    }

    #[test]
    fn temperatura_warn_no_se_activa_sin_ninguna_lectura() {
        assert_eq!(evaluar_temperatura_configurada_warn(&[], 60.0), None);
    }

    #[test]
    fn temperatura_warn_resuelve_tres_grados_por_debajo_del_umbral_durante_tres_ciclos() {
        assert!(!resuelve_temperatura_configurada_warn(
            &[58.0, 58.0, 58.0],
            60.0
        ));
        assert!(resuelve_temperatura_configurada_warn(
            &[57.0, 57.0, 57.0],
            60.0
        ));
    }

    // ---- temp.above_configured_crit (umbral de settings, fábrica 70 °C) ----

    #[test]
    fn temperatura_crit_es_inmediata_en_su_umbral() {
        assert_eq!(
            evaluar_temperatura_configurada_crit(&[70.0], 70.0),
            Some(AlertSeverity::Critical)
        );
        assert_eq!(evaluar_temperatura_configurada_crit(&[69.9], 70.0), None);
    }

    #[test]
    fn temperatura_crit_no_se_activa_sin_ninguna_lectura() {
        assert_eq!(evaluar_temperatura_configurada_crit(&[], 70.0), None);
    }

    #[test]
    fn temperatura_crit_resuelve_cinco_grados_por_debajo_durante_tres_ciclos_no_inmediatamente() {
        assert!(
            !resuelve_temperatura_configurada_crit(&[64.0], 70.0),
            "la resolución sí exige histéresis"
        );
        assert!(resuelve_temperatura_configurada_crit(
            &[64.0, 64.0, 64.0],
            70.0
        ));
    }

    #[test]
    fn temperatura_warn_y_crit_son_independientes_pueden_coexistir() {
        // Un disco a 75°C con fábrica 60/70 cumple ambas: advertencia (>60, 3 ciclos) y crítico (>=70).
        let serie = [75.0, 75.0, 75.0];
        assert_eq!(
            evaluar_temperatura_configurada_warn(&serie, 60.0),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(
            evaluar_temperatura_configurada_crit(&serie, 70.0),
            Some(AlertSeverity::Critical)
        );
    }

    // ---- smart.spare_below_threshold ----

    #[test]
    fn spare_below_threshold_se_activa_tras_tres_lecturas_por_debajo() {
        let pares = [(8.0, 10.0), (7.0, 10.0), (9.0, 10.0)];
        assert_eq!(
            evaluar_spare_below_threshold(&pares),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn spare_below_threshold_no_se_activa_si_una_lectura_reciente_esta_por_encima() {
        let pares = [(11.0, 10.0), (7.0, 10.0), (9.0, 10.0)];
        assert_eq!(evaluar_spare_below_threshold(&pares), None);
    }

    #[test]
    fn spare_below_threshold_no_se_activa_sin_datos() {
        assert_eq!(evaluar_spare_below_threshold(&[]), None);
    }

    #[test]
    fn spare_below_threshold_resuelve_con_dos_puntos_de_margen() {
        assert!(
            !resuelve_spare_below_threshold(&[(11.0, 10.0), (11.0, 10.0), (11.0, 10.0)]),
            "no llega al margen de 2"
        );
        assert!(resuelve_spare_below_threshold(&[
            (12.0, 10.0),
            (12.0, 10.0),
            (12.0, 10.0)
        ]));
    }

    // ---- smart.media_errors (umbral sobre la magnitud del incremento, fábrica 1/5) ----

    #[test]
    fn media_errors_advierte_en_el_umbral_del_incremento_y_escala_a_critico() {
        // incremento de 1 = aviso; de 5 = crítico; de 0 = nada.
        assert_eq!(
            evaluar_media_errors(&[4.0, 3.0], 1, 5),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(evaluar_media_errors(&[3.0, 3.0], 1, 5), None);
        assert_eq!(
            evaluar_media_errors(&[8.0, 3.0], 1, 5),
            Some(AlertSeverity::Critical)
        );
        assert_eq!(
            evaluar_media_errors(&[7.0, 3.0], 1, 5),
            Some(AlertSeverity::Warning)
        );
    }

    #[test]
    fn media_errors_no_se_activa_sin_lectura_previa() {
        assert_eq!(
            evaluar_media_errors(&[5.0], 1, 5),
            None,
            "sin anterior no hay incremento que medir"
        );
    }

    #[test]
    fn media_errors_un_incremento_negativo_no_activa_nada() {
        // El contador solo baja si se sustituye el disco: no es un error.
        assert_eq!(evaluar_media_errors(&[3.0, 5.0], 1, 5), None);
    }

    #[test]
    fn media_errors_respeta_umbrales_configurables() {
        // incremento de 3: nada con «Solo lo grave» (5/15), crítico con «Prudente» (1/3).
        assert_eq!(evaluar_media_errors(&[6.0, 3.0], 5, 15), None);
        assert_eq!(
            evaluar_media_errors(&[6.0, 3.0], 1, 3),
            Some(AlertSeverity::Critical)
        );
    }

    // ---- smart.error_log ----

    #[test]
    fn error_log_advierte_con_un_solo_aumento() {
        assert_eq!(evaluar_error_log(&[2.0, 1.0]), Some(AlertSeverity::Warning));
    }

    #[test]
    fn error_log_no_se_activa_sin_lectura_previa() {
        assert_eq!(evaluar_error_log(&[2.0]), None);
    }

    #[test]
    fn error_log_escala_a_critico_tras_tres_aumentos_consecutivos() {
        assert_eq!(
            evaluar_error_log(&[4.0, 3.0, 2.0, 1.0]),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn error_log_se_queda_en_advertencia_si_el_tercer_paso_no_aumento() {
        assert_eq!(
            evaluar_error_log(&[4.0, 3.0, 2.0, 2.0]),
            Some(AlertSeverity::Warning)
        );
    }

    // ---- capacity.low / capacity.critical (nuevas en v3) ----

    fn cap() -> UmbralesCapacidad {
        UmbralesCapacidad::default() // 10 % / 5 %
    }

    #[test]
    fn capacidad_low_advierte_por_debajo_del_diez_por_ciento_no_en_el_umbral() {
        let vol = 100.0 * G; // pequeño: solo manda el porcentaje
        assert_eq!(
            evaluar_capacidad_low(&[(9.0 * G, vol)], &cap()),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(evaluar_capacidad_low(&[(10.0 * G, vol)], &cap()), None);
        assert_eq!(evaluar_capacidad_low(&[(11.0 * G, vol)], &cap()), None);
    }

    #[test]
    fn capacidad_low_no_se_activa_cuando_ya_es_critico_ese_es_capacity_critical() {
        let vol = 100.0 * G;
        assert_eq!(
            evaluar_capacidad_low(&[(3.0 * G, vol)], &cap()),
            None,
            "3 % es crítico, no low"
        );
        assert_eq!(
            evaluar_capacidad_critical(&[(3.0 * G, vol)], &cap()),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn capacidad_no_se_activa_sin_ninguna_lectura() {
        assert_eq!(evaluar_capacidad_low(&[], &cap()), None);
        assert_eq!(evaluar_capacidad_critical(&[], &cap()), None);
    }

    #[test]
    fn capacidad_low_resuelve_tras_tres_lecturas_en_ok() {
        let vol = 100.0 * G;
        assert!(!resuelve_capacidad_low(
            &[(20.0 * G, vol), (9.0 * G, vol), (20.0 * G, vol)],
            &cap()
        ));
        assert!(resuelve_capacidad_low(
            &[(20.0 * G, vol), (20.0 * G, vol), (20.0 * G, vol)],
            &cap()
        ));
    }

    #[test]
    fn capacidad_critical_resuelve_cuando_sube_a_warn_u_ok_tres_ciclos() {
        let vol = 100.0 * G;
        // 3 lecturas en aviso (8 %): ya no es crítico ⇒ resuelve el crítico
        assert!(resuelve_capacidad_critical(
            &[(8.0 * G, vol), (8.0 * G, vol), (8.0 * G, vol)],
            &cap()
        ));
        // una lectura reciente vuelve a crítico ⇒ no resuelve
        assert!(!resuelve_capacidad_critical(
            &[(3.0 * G, vol), (8.0 * G, vol), (8.0 * G, vol)],
            &cap()
        ));
    }
}
