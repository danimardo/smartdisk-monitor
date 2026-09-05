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

use crate::domain::tipos::AlertSeverity;

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
pub fn evaluar_wear_high(serie: &[f64]) -> Option<AlertSeverity> {
    let v = *serie.first()?;
    if v >= 100.0 {
        Some(AlertSeverity::Critical)
    } else if v >= 90.0 {
        Some(AlertSeverity::Warning)
    } else {
        None
    }
}

/// `temp.above_configured_warn`: sin límite del fabricante, 3 ciclos consecutivos por encima de
/// 70 °C. **Regla independiente** de `temp.above_configured_crit`, no un nivel más bajo de la
/// misma: la tabla normativa les da activación, severidad y resolución propias, así que pueden
/// convivir como dos grupos activos a la vez (`domain::salud::device_state` ya colapsa al peor).
pub fn evaluar_temperatura_configurada_warn(serie: &[f64]) -> Option<AlertSeverity> {
    primeras_n_cumplen(serie, 3, |v| v > 70.0).then_some(AlertSeverity::Warning)
}

/// Resolución de `temp.above_configured_warn`: ≤ 67 °C durante 3 ciclos.
pub fn resuelve_temperatura_configurada_warn(serie: &[f64]) -> bool {
    primeras_n_cumplen(serie, 3, |v| v <= 67.0)
}

/// `temp.above_configured_crit`: crítico **inmediato** en ≥ 80 °C, sin ciclos de histéresis para
/// activarse — a diferencia de `..._warn`, que exige 3 ciclos consecutivos.
pub fn evaluar_temperatura_configurada_crit(serie: &[f64]) -> Option<AlertSeverity> {
    primeras_n_cumplen(serie, 1, |v| v >= 80.0).then_some(AlertSeverity::Critical)
}

/// Resolución de `temp.above_configured_crit`: ≤ 75 °C durante 3 ciclos.
pub fn resuelve_temperatura_configurada_crit(serie: &[f64]) -> bool {
    primeras_n_cumplen(serie, 3, |v| v <= 75.0)
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

/// `smart.media_errors`: crítico inmediato si el contador aumentó respecto a la lectura anterior.
/// Sin lectura previa no hay incremento que medir (`alert-rules.md`, última viñeta de "NO genera
/// alerta"): con menos de dos muestras, no se evalúa.
pub fn evaluar_media_errors(serie: &[f64]) -> Option<AlertSeverity> {
    (serie.len() >= 2 && serie[0] > serie[1]).then_some(AlertSeverity::Critical)
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

#[cfg(test)]
mod tests {
    use super::*;

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

    // ---- smart.wear_high ----

    #[test]
    fn wear_high_advierte_a_partir_de_noventa() {
        assert_eq!(evaluar_wear_high(&[90.0]), Some(AlertSeverity::Warning));
        assert_eq!(evaluar_wear_high(&[89.0]), None);
    }

    #[test]
    fn wear_high_escala_a_critico_en_cien() {
        assert_eq!(evaluar_wear_high(&[100.0]), Some(AlertSeverity::Critical));
    }

    #[test]
    fn wear_high_no_se_activa_sin_lectura() {
        assert_eq!(evaluar_wear_high(&[]), None);
    }

    // ---- temp.above_configured_warn ----

    #[test]
    fn temperatura_warn_advierte_tras_tres_ciclos_por_encima_de_setenta() {
        assert_eq!(
            evaluar_temperatura_configurada_warn(&[71.0, 72.0, 73.0]),
            Some(AlertSeverity::Warning)
        );
    }

    #[test]
    fn temperatura_warn_no_se_activa_con_solo_dos_ciclos() {
        assert_eq!(
            evaluar_temperatura_configurada_warn(&[71.0, 72.0]),
            None,
            "faltan ciclos, dato incompleto"
        );
    }

    #[test]
    fn temperatura_warn_no_se_activa_sin_ninguna_lectura() {
        assert_eq!(evaluar_temperatura_configurada_warn(&[]), None);
    }

    #[test]
    fn temperatura_warn_resuelve_a_67_durante_tres_ciclos() {
        assert!(!resuelve_temperatura_configurada_warn(&[68.0, 68.0, 68.0]));
        assert!(resuelve_temperatura_configurada_warn(&[67.0, 67.0, 67.0]));
    }

    // ---- temp.above_configured_crit ----

    #[test]
    fn temperatura_crit_es_inmediata_una_sola_lectura_basta() {
        assert_eq!(
            evaluar_temperatura_configurada_crit(&[80.0]),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn temperatura_crit_no_se_activa_por_debajo_de_ochenta() {
        assert_eq!(evaluar_temperatura_configurada_crit(&[79.9]), None);
    }

    #[test]
    fn temperatura_crit_no_se_activa_sin_ninguna_lectura() {
        assert_eq!(evaluar_temperatura_configurada_crit(&[]), None);
    }

    #[test]
    fn temperatura_crit_resuelve_a_75_durante_tres_ciclos_no_inmediatamente() {
        assert!(
            !resuelve_temperatura_configurada_crit(&[74.0]),
            "la resolución sí exige histéresis"
        );
        assert!(resuelve_temperatura_configurada_crit(&[74.0, 74.0, 74.0]));
    }

    #[test]
    fn temperatura_warn_y_crit_son_independientes_pueden_coexistir() {
        // Un disco a 85°C cumple ambas: advertencia (>70, 3 ciclos) y crítico (>=80, inmediato).
        // domain::salud colapsa al peor; el motor no decide eso, solo evalúa cada regla.
        let serie = [85.0, 85.0, 85.0];
        assert_eq!(
            evaluar_temperatura_configurada_warn(&serie),
            Some(AlertSeverity::Warning)
        );
        assert_eq!(
            evaluar_temperatura_configurada_crit(&serie),
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

    // ---- smart.media_errors ----

    #[test]
    fn media_errors_se_activa_si_aumenta_respecto_a_la_lectura_anterior() {
        assert_eq!(
            evaluar_media_errors(&[5.0, 3.0]),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn media_errors_no_se_activa_sin_lectura_previa() {
        assert_eq!(
            evaluar_media_errors(&[5.0]),
            None,
            "sin anterior no hay incremento que medir"
        );
    }

    #[test]
    fn media_errors_no_se_activa_si_no_aumenta() {
        assert_eq!(evaluar_media_errors(&[3.0, 3.0]), None);
        assert_eq!(
            evaluar_media_errors(&[3.0, 5.0]),
            None,
            "bajar no es aumentar"
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
}
