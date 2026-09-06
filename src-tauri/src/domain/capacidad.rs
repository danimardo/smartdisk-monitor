//! Nivel de capacidad de un volumen (spec §5, `docs/alert-rules.md` `capacity.low`/`capacity.critical`).
//!
//! **Espejo exacto de `capacityState()` en `src/lib/design/health.ts`** — si una de las dos cambia,
//! la otra miente. La única diferencia: aquí los umbrales llegan como parámetro (de `settings.alerts`,
//! ADR-036), no como constante.
//!
//! Puro: sin Tauri, sin SQLite. Recibe bytes y devuelve un `HealthState`.

use crate::domain::tipos::HealthState;

/// Umbrales efectivos de capacidad, ya resueltos de `settings.alerts` por quien orquesta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UmbralesCapacidad {
    pub warn_percent: f64,
    pub crit_percent: f64,
    /// El suelo absoluto solo se aplica en volúmenes de al menos esta capacidad: en un disco
    /// pequeño, 20 GiB libres pueden ser un tercio del disco y marcarlo sería ruido.
    pub floor_min_capacity_bytes: i64,
    pub floor_warn_bytes: i64,
    pub floor_crit_bytes: i64,
}

impl Default for UmbralesCapacidad {
    fn default() -> Self {
        use crate::domain::ajustes as aj;
        Self {
            warn_percent: aj::CAPACITY_WARN_PERCENT_DEFAULT,
            crit_percent: aj::CAPACITY_CRIT_PERCENT_DEFAULT,
            floor_min_capacity_bytes: aj::CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES,
            floor_warn_bytes: aj::CAPACITY_FLOOR_WARN_DEFAULT_BYTES,
            floor_crit_bytes: aj::CAPACITY_FLOOR_CRIT_DEFAULT_BYTES,
        }
    }
}

/// Nivel de capacidad. Reglas (idénticas a la versión TS):
///  - siempre por porcentaje: `< crit_percent` ⇒ crítico, `< warn_percent` ⇒ advertencia;
///  - además, en volúmenes de al menos `floor_min_capacity_bytes`, por valor absoluto en bytes;
///  - gana el criterio más severo de los dos.
///
/// Sin datos (`free`/`capacity` ausente o capacidad no positiva) ⇒ `Unknown`, nunca un cero inventado.
pub fn estado_capacidad(
    free_bytes: Option<i64>,
    capacity_bytes: Option<i64>,
    u: &UmbralesCapacidad,
) -> HealthState {
    let (Some(free), Some(cap)) = (free_bytes, capacity_bytes) else {
        return HealthState::Unknown;
    };
    if cap <= 0 || free < 0 {
        return HealthState::Unknown;
    }
    let pct = (free as f64 / cap as f64) * 100.0;
    let aplica_absoluto = cap >= u.floor_min_capacity_bytes;

    if pct < u.crit_percent || (aplica_absoluto && free < u.floor_crit_bytes) {
        HealthState::Crit
    } else if pct < u.warn_percent || (aplica_absoluto && free < u.floor_warn_bytes) {
        HealthState::Warn
    } else {
        HealthState::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: i64 = 1024 * 1024 * 1024;

    fn u() -> UmbralesCapacidad {
        UmbralesCapacidad::default() // 10 % / 5 %, suelo en 256 GiB, 20/10 GiB
    }

    #[test]
    fn sin_datos_es_desconocido_nunca_un_cero_inventado() {
        assert_eq!(
            estado_capacidad(None, Some(500 * GIB), &u()),
            HealthState::Unknown
        );
        assert_eq!(
            estado_capacidad(Some(10 * GIB), None, &u()),
            HealthState::Unknown
        );
        assert_eq!(
            estado_capacidad(Some(10 * GIB), Some(0), &u()),
            HealthState::Unknown
        );
    }

    #[test]
    fn por_porcentaje_en_el_umbral_justo_por_encima_y_por_debajo() {
        let cap = 1000 * GIB; // grande, aplica también el absoluto
                              // 10 % exacto no es aviso (operador estricto), 9,99 % sí
        assert_eq!(
            estado_capacidad(Some(100 * GIB), Some(cap), &u()),
            HealthState::Ok
        );
        assert_eq!(
            estado_capacidad(Some(99 * GIB), Some(cap), &u()),
            HealthState::Warn
        );
        // 5 % exacto no es crítico, 4,99 % sí
        assert_eq!(
            estado_capacidad(Some(50 * GIB), Some(cap), &u()),
            HealthState::Warn
        );
        assert_eq!(
            estado_capacidad(Some(49 * GIB), Some(cap), &u()),
            HealthState::Crit
        );
    }

    #[test]
    fn el_suelo_absoluto_no_se_aplica_a_volumenes_pequenos() {
        // 40 GiB de 100 GiB = 40 %: por porcentaje, ok. El absoluto (20 GiB) no aplica: cap < 256 GiB.
        assert_eq!(
            estado_capacidad(Some(40 * GIB), Some(100 * GIB), &u()),
            HealthState::Ok
        );
        // pero si el mismo volumen baja al 8 % (8 GiB), el porcentaje sí manda
        assert_eq!(
            estado_capacidad(Some(8 * GIB), Some(100 * GIB), &u()),
            HealthState::Warn
        );
    }

    #[test]
    fn el_suelo_absoluto_manda_en_un_volumen_grande_con_porcentaje_holgado() {
        // 15 GiB libres de 2 TiB = 0,7 %: crítico por porcentaje ya, pero además < 20 GiB absoluto.
        // Caso más interesante: 25 GiB de 2 TiB = 1,2 % (crítico por %) — comprobamos el absoluto solo:
        // 18 GiB de 4 TiB con umbrales de % relajados a 0/0 → solo el absoluto decide.
        let relajado = UmbralesCapacidad {
            warn_percent: 0.0,
            crit_percent: 0.0,
            ..UmbralesCapacidad::default()
        };
        assert_eq!(
            estado_capacidad(Some(18 * GIB), Some(4096 * GIB), &relajado),
            HealthState::Warn,
            "< 20 GiB en un volumen grande es aviso aunque el porcentaje sea holgado"
        );
        assert_eq!(
            estado_capacidad(Some(9 * GIB), Some(4096 * GIB), &relajado),
            HealthState::Crit
        );
    }

    #[test]
    fn con_espacio_de_sobra_es_correcto() {
        assert_eq!(
            estado_capacidad(Some(500 * GIB), Some(1000 * GIB), &u()),
            HealthState::Ok
        );
    }

    #[test]
    fn umbrales_configurables_mueven_la_frontera() {
        let prudente = UmbralesCapacidad {
            warn_percent: 15.0,
            crit_percent: 8.0,
            ..UmbralesCapacidad::default()
        };
        // 12 % de un volumen pequeño: ok con el perfil de fábrica (10 %), aviso con «Prudente» (15 %)
        assert_eq!(
            estado_capacidad(Some(12 * GIB), Some(100 * GIB), &u()),
            HealthState::Ok
        );
        assert_eq!(
            estado_capacidad(Some(12 * GIB), Some(100 * GIB), &prudente),
            HealthState::Warn
        );
    }
}
