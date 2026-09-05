//! Validación tipada de `settings` (T095/T096, `docs/open-questions.md` J.32).
//!
//! Puro: sin Tauri, sin SQLite, sin persistencia — cada función valida un valor contra sus límites
//! y devuelve el valor validado o el motivo del rechazo. `commands::mod` decide qué hacer con el
//! valor anterior (conservarlo) y cómo persistir el nuevo.

use crate::collectors::planificador::Trabajo;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorAjuste {
    /// `valor` cayó fuera de `[minimo, maximo]`.
    FueraDeLimites { minimo: f64, maximo: f64 },
    /// El valor crítico no es más severo que el de aviso (temperatura: no es mayor; capacidad:
    /// no es menor). Sin esto, un aviso y un crítico invertidos dejarían el segundo nivel
    /// inalcanzable en silencio.
    CriticoNoMasSeveroQueAviso,
}

/// Límites de temperatura configurada (J.32): mismos literales que hoy tiene hardcodeados
/// `alerts::motor` para `temp.above_configured_warn/crit` (70/80), con límites de edición nuevos:
/// 40-95 °C para el aviso, el crítico entre el aviso y 100 °C.
pub const TEMP_WARN_MIN_C: f64 = 40.0;
pub const TEMP_WARN_MAX_C: f64 = 95.0;
pub const TEMP_CRIT_MAX_C: f64 = 100.0;

pub const TEMP_WARN_DEFAULT_C: f64 = 70.0;
pub const TEMP_CRIT_DEFAULT_C: f64 = 80.0;

/// Límites de capacidad porcentual (C.1/ADR-019): 10 % aviso, 5 % crítico de fábrica.
pub const CAPACITY_PERCENT_MIN: f64 = 1.0;
pub const CAPACITY_PERCENT_MAX: f64 = 50.0;
pub const CAPACITY_WARN_PERCENT_DEFAULT: f64 = 10.0;
pub const CAPACITY_CRIT_PERCENT_DEFAULT: f64 = 5.0;

/// Suelo absoluto de capacidad (C.1): solo se aplica en volúmenes de al menos
/// `capacity_absolute_floor_min_capacity_bytes` (256 GiB de fábrica); por debajo de ese umbral de
/// aviso/crítico en bytes, el disco es demasiado pequeño para que 20/10 GiB libres sean una señal.
pub const CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES: i64 = 256 * 1024 * 1024 * 1024;
pub const CAPACITY_FLOOR_WARN_DEFAULT_BYTES: i64 = 20 * 1024 * 1024 * 1024;
pub const CAPACITY_FLOOR_CRIT_DEFAULT_BYTES: i64 = 10 * 1024 * 1024 * 1024;

/// Periodos de retención (J.14): 7/90/730 días de fábrica, límites de edición nuevos.
pub const RETENTION_RAW_DAYS_MIN: i64 = 1;
pub const RETENTION_RAW_DAYS_MAX: i64 = 30;
pub const RETENTION_RAW_DAYS_DEFAULT: i64 = 7;

pub const RETENTION_FIVE_MINUTES_DAYS_MIN: i64 = 7;
pub const RETENTION_FIVE_MINUTES_DAYS_MAX: i64 = 365;
pub const RETENTION_FIVE_MINUTES_DAYS_DEFAULT: i64 = 90;

pub const RETENTION_HOURLY_DAYS_MIN: i64 = 90;
pub const RETENTION_HOURLY_DAYS_MAX: i64 = 1825;
pub const RETENTION_HOURLY_DAYS_DEFAULT: i64 = 730;

/// Guardia de espacio del historial (J.13): 1 GiB aviso, 256 MiB parada. Sin límites de edición
/// propios: US-071 solo exige poder cambiar los tres periodos de retención, no estos dos valores.
pub const STORAGE_FREE_SPACE_WARN_DEFAULT_BYTES: i64 = 1024 * 1024 * 1024;
pub const STORAGE_FREE_SPACE_HALT_DEFAULT_BYTES: i64 = 256 * 1024 * 1024;

fn en_rango(valor: f64, minimo: f64, maximo: f64) -> Result<f64, ErrorAjuste> {
    if valor < minimo || valor > maximo {
        Err(ErrorAjuste::FueraDeLimites { minimo, maximo })
    } else {
        Ok(valor)
    }
}

/// Una de las cuatro frecuencias de `collectors::planificador` (D.1): reutiliza los límites que
/// ese módulo ya declaraba para sí mismo, no unos nuevos.
pub fn validar_frecuencia_segundos(trabajo: Trabajo, segundos: i64) -> Result<i64, ErrorAjuste> {
    let minimo = trabajo.minimo.as_secs() as f64;
    let maximo = trabajo.maximo.as_secs() as f64;
    en_rango(segundos as f64, minimo, maximo)?;
    Ok(segundos)
}

/// Valida el par aviso/crítico de temperatura configurada. `warn_c` y `crit_c` son los valores
/// candidatos completos (el llamante ya sustituyó el que cambió), no un delta.
pub fn validar_temperaturas(warn_c: f64, crit_c: f64) -> Result<(), ErrorAjuste> {
    en_rango(warn_c, TEMP_WARN_MIN_C, TEMP_WARN_MAX_C)?;
    en_rango(crit_c, TEMP_WARN_MIN_C, TEMP_CRIT_MAX_C)?;
    if crit_c <= warn_c {
        return Err(ErrorAjuste::CriticoNoMasSeveroQueAviso);
    }
    Ok(())
}

/// Valida el par aviso/crítico de capacidad porcentual: el crítico es un porcentaje **menor**
/// (menos espacio libre es más grave), así que la relación se invierte frente a temperatura.
pub fn validar_capacidad_porcentual(warn_pct: f64, crit_pct: f64) -> Result<(), ErrorAjuste> {
    en_rango(warn_pct, CAPACITY_PERCENT_MIN, CAPACITY_PERCENT_MAX)?;
    en_rango(crit_pct, CAPACITY_PERCENT_MIN, CAPACITY_PERCENT_MAX)?;
    if crit_pct >= warn_pct {
        return Err(ErrorAjuste::CriticoNoMasSeveroQueAviso);
    }
    Ok(())
}

/// Valida el suelo absoluto de capacidad completo: la capacidad mínima de volumen a la que se
/// aplica, y el par aviso/crítico en bytes (igual relación que el porcentual: crítico menor).
pub fn validar_capacidad_absoluta(
    min_capacity_bytes: i64,
    warn_bytes: i64,
    crit_bytes: i64,
) -> Result<(), ErrorAjuste> {
    if min_capacity_bytes < 0 || warn_bytes <= 0 || crit_bytes <= 0 {
        return Err(ErrorAjuste::FueraDeLimites {
            minimo: 0.0,
            maximo: f64::MAX,
        });
    }
    if crit_bytes >= warn_bytes {
        return Err(ErrorAjuste::CriticoNoMasSeveroQueAviso);
    }
    Ok(())
}

pub fn validar_retencion_raw_days(dias: i64) -> Result<i64, ErrorAjuste> {
    en_rango(
        dias as f64,
        RETENTION_RAW_DAYS_MIN as f64,
        RETENTION_RAW_DAYS_MAX as f64,
    )?;
    Ok(dias)
}

pub fn validar_retencion_five_minutes_days(dias: i64) -> Result<i64, ErrorAjuste> {
    en_rango(
        dias as f64,
        RETENTION_FIVE_MINUTES_DAYS_MIN as f64,
        RETENTION_FIVE_MINUTES_DAYS_MAX as f64,
    )?;
    Ok(dias)
}

pub fn validar_retencion_hourly_days(dias: i64) -> Result<i64, ErrorAjuste> {
    en_rango(
        dias as f64,
        RETENTION_HOURLY_DAYS_MIN as f64,
        RETENTION_HOURLY_DAYS_MAX as f64,
    )?;
    Ok(dias)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::planificador::METRICAS_RAPIDAS;

    #[test]
    fn una_frecuencia_dentro_de_limites_se_acepta() {
        assert_eq!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 30), Ok(30));
    }

    #[test]
    fn una_frecuencia_por_debajo_del_minimo_se_rechaza() {
        assert!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 5).is_err());
    }

    #[test]
    fn una_frecuencia_por_encima_del_maximo_se_rechaza() {
        assert!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 3600).is_err());
    }

    #[test]
    fn justo_en_los_extremos_de_la_frecuencia_se_acepta() {
        assert!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 10).is_ok());
        assert!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 300).is_ok());
        assert!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 9).is_err());
        assert!(validar_frecuencia_segundos(METRICAS_RAPIDAS, 301).is_err());
    }

    #[test]
    fn temperaturas_de_fabrica_son_validas() {
        assert!(validar_temperaturas(TEMP_WARN_DEFAULT_C, TEMP_CRIT_DEFAULT_C).is_ok());
    }

    #[test]
    fn el_critico_de_temperatura_igual_al_aviso_se_rechaza() {
        assert_eq!(
            validar_temperaturas(70.0, 70.0),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
    }

    #[test]
    fn el_critico_de_temperatura_por_debajo_del_aviso_se_rechaza() {
        assert_eq!(
            validar_temperaturas(70.0, 60.0),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
    }

    #[test]
    fn un_aviso_de_temperatura_fuera_de_limites_se_rechaza() {
        assert!(validar_temperaturas(30.0, 80.0).is_err());
        assert!(validar_temperaturas(96.0, 97.0).is_err());
    }

    #[test]
    fn capacidad_porcentual_de_fabrica_es_valida() {
        assert!(validar_capacidad_porcentual(
            CAPACITY_WARN_PERCENT_DEFAULT,
            CAPACITY_CRIT_PERCENT_DEFAULT
        )
        .is_ok());
    }

    #[test]
    fn el_critico_de_capacidad_igual_o_por_encima_del_aviso_se_rechaza() {
        assert_eq!(
            validar_capacidad_porcentual(10.0, 10.0),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
        assert_eq!(
            validar_capacidad_porcentual(10.0, 15.0),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
    }

    #[test]
    fn capacidad_absoluta_de_fabrica_es_valida() {
        assert!(validar_capacidad_absoluta(
            CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES,
            CAPACITY_FLOOR_WARN_DEFAULT_BYTES,
            CAPACITY_FLOOR_CRIT_DEFAULT_BYTES
        )
        .is_ok());
    }

    #[test]
    fn capacidad_absoluta_con_critico_no_mas_severo_se_rechaza() {
        assert_eq!(
            validar_capacidad_absoluta(
                CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES,
                10 * 1024 * 1024 * 1024,
                10 * 1024 * 1024 * 1024
            ),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
    }

    #[test]
    fn retencion_de_fabrica_es_valida_en_los_tres_periodos() {
        assert_eq!(
            validar_retencion_raw_days(RETENTION_RAW_DAYS_DEFAULT),
            Ok(RETENTION_RAW_DAYS_DEFAULT)
        );
        assert_eq!(
            validar_retencion_five_minutes_days(RETENTION_FIVE_MINUTES_DAYS_DEFAULT),
            Ok(RETENTION_FIVE_MINUTES_DAYS_DEFAULT)
        );
        assert_eq!(
            validar_retencion_hourly_days(RETENTION_HOURLY_DAYS_DEFAULT),
            Ok(RETENTION_HOURLY_DAYS_DEFAULT)
        );
    }

    #[test]
    fn retencion_justo_en_los_extremos() {
        assert!(validar_retencion_raw_days(RETENTION_RAW_DAYS_MIN).is_ok());
        assert!(validar_retencion_raw_days(RETENTION_RAW_DAYS_MAX).is_ok());
        assert!(validar_retencion_raw_days(RETENTION_RAW_DAYS_MIN - 1).is_err());
        assert!(validar_retencion_raw_days(RETENTION_RAW_DAYS_MAX + 1).is_err());
    }

    #[test]
    fn retencion_fuera_de_limites_se_rechaza_en_los_tres_periodos() {
        assert!(validar_retencion_raw_days(0).is_err());
        assert!(validar_retencion_five_minutes_days(1).is_err());
        assert!(validar_retencion_hourly_days(10).is_err());
    }
}
