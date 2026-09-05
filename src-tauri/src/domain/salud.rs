//! Cálculo de estado de salud y frescura por métrica (FR-003 a FR-006, constitución §I y §VI).
//!
//! **No decide alertas.** Recibe la peor severidad ya calculada por `alerts/` y la traduce a color;
//! nunca vuelve a evaluar reglas. Esa separación es la que permite que "reconocer no apaga el
//! color" sea una propiedad de este módulo y no algo que cada pantalla tenga que recordar.

use crate::domain::tipos::{AlertSeverity, HealthState};

/// El estado de un dispositivo, en el orden exacto que exige la constitución:
///
/// 1. Sin compatibilidad SMART ⇒ `unknown`, nunca rojo, nunca alerta (FR-003).
/// 2. Sin lecturas frescas ⇒ `unknown`, nunca `ok` — no saber que algo está bien no es saber que
///    está bien (FR-006).
/// 3. En otro caso, la peor severidad de las alertas `active` **o** `acknowledged`. Reconocer o
///    silenciar no participa aquí: ya salieron de esa lista en el repositorio, o no.
pub fn device_state(
    smart_supported: bool,
    has_fresh_data: bool,
    peor_alerta_activa_o_reconocida: Option<AlertSeverity>,
) -> HealthState {
    if !smart_supported {
        return HealthState::Unknown;
    }
    if !has_fresh_data {
        return HealthState::Unknown;
    }
    match peor_alerta_activa_o_reconocida {
        Some(AlertSeverity::Critical) => HealthState::Crit,
        Some(AlertSeverity::Warning) => HealthState::Warn,
        None => HealthState::Ok,
    }
}

/// Color agregado de la bandeja del sistema (`docs/product-specification.md` §3,
/// `docs/open-questions.md` B.5). Espejo exacto de `trayState()` en `src/lib/design/health.ts`:
/// **no es un tipo nuevo**, es el mismo `HealthState` que usa el resto de la interfaz — el "gris"
/// del icono es su `Unknown`. Un crítico vigente manda sobre la pausa: la condición sigue siendo
/// cierta aunque hayamos dejado de mirar; la pausa se comunica con el texto del menú de la
/// bandeja, no apagando la señal.
pub fn tray_state(
    pausado: bool,
    fallo_recopilador: bool,
    estados_monitorizados: &[HealthState],
) -> HealthState {
    if estados_monitorizados.contains(&HealthState::Crit) {
        return HealthState::Crit;
    }
    if pausado || fallo_recopilador || estados_monitorizados.is_empty() {
        return HealthState::Unknown;
    }
    if estados_monitorizados.contains(&HealthState::Warn) {
        return HealthState::Warn;
    }
    if estados_monitorizados.contains(&HealthState::Ok) {
        HealthState::Ok
    } else {
        HealthState::Unknown
    }
}

/// Cuánto puede envejecer una muestra antes de dejar de contar como "reciente", en función de la
/// cadencia esperada de su métrica (`docs/product-specification.md` §4). El múltiplo (2×) es el
/// mismo criterio de margen que usan los huecos de una serie (`domain::retencion`, con su propio
/// 1,5× para gráficas): aquí se da algo más de margen porque un ciclo perdido por una fuente lenta
/// no debe declarar "sin datos" antes de que el siguiente ciclo tenga ocasión de llegar.
pub fn es_dato_caduco(antiguedad_segundos: i64, cadencia_esperada_segundos: i64) -> bool {
    antiguedad_segundos > cadencia_esperada_segundos * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_soporte_smart_es_desconocido_aunque_haya_alertas() {
        let estado = device_state(false, true, Some(AlertSeverity::Critical));
        assert_eq!(estado, HealthState::Unknown, "no compatible nunca es rojo");
    }

    #[test]
    fn sin_datos_frescos_es_desconocido_no_correcto() {
        let estado = device_state(true, false, None);
        assert_eq!(
            estado,
            HealthState::Unknown,
            "no saber que algo está bien no es saber que está bien"
        );
    }

    #[test]
    fn con_datos_frescos_y_sin_alertas_es_correcto() {
        assert_eq!(device_state(true, true, None), HealthState::Ok);
    }

    #[test]
    fn una_alerta_critica_activa_o_reconocida_pone_el_dispositivo_en_critico() {
        assert_eq!(
            device_state(true, true, Some(AlertSeverity::Critical)),
            HealthState::Crit
        );
    }

    #[test]
    fn una_advertencia_pone_el_dispositivo_en_advertencia() {
        assert_eq!(
            device_state(true, true, Some(AlertSeverity::Warning)),
            HealthState::Warn
        );
    }

    #[test]
    fn dentro_de_dos_cadencias_no_esta_caduco() {
        assert!(!es_dato_caduco(59, 30));
        assert!(!es_dato_caduco(60, 30));
    }

    #[test]
    fn mas_de_dos_cadencias_si_esta_caduco() {
        assert!(es_dato_caduco(61, 30));
    }

    // ---- tray_state — espejo de trayState() en health.test.ts (§B.5) ----

    #[test]
    fn un_critico_vigente_manda_sobre_la_pausa() {
        assert_eq!(
            tray_state(true, false, &[HealthState::Crit]),
            HealthState::Crit
        );
    }

    #[test]
    fn en_pausa_sin_criticos_el_icono_es_gris() {
        assert_eq!(
            tray_state(true, false, &[HealthState::Ok, HealthState::Warn]),
            HealthState::Unknown
        );
    }

    #[test]
    fn sin_discos_monitorizados_gris() {
        assert_eq!(tray_state(false, false, &[]), HealthState::Unknown);
    }

    #[test]
    fn un_recopilador_caido_degrada_el_icono() {
        assert_eq!(
            tray_state(false, true, &[HealthState::Ok]),
            HealthState::Unknown
        );
    }

    #[test]
    fn alguna_advertencia_sin_criticos_es_ambar() {
        assert_eq!(
            tray_state(false, false, &[HealthState::Ok, HealthState::Warn]),
            HealthState::Warn
        );
    }

    #[test]
    fn todo_correcto_es_verde() {
        assert_eq!(
            tray_state(false, false, &[HealthState::Ok, HealthState::Ok]),
            HealthState::Ok
        );
    }

    #[test]
    fn desconocido_no_gana_a_un_estado_conocido() {
        assert_eq!(
            tray_state(false, false, &[HealthState::Unknown, HealthState::Ok]),
            HealthState::Ok
        );
    }
}
