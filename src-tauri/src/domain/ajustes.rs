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

/// Límites de temperatura configurada (J.32). El motor **lee** estos valores desde v3 (ADR-036):
/// `temp.above_configured_warn/crit` ya no llevan `70/80` literal. Límites de edición: 40-95 °C para
/// el aviso, el crítico entre el aviso y 100 °C. Fábrica: 60/70 (perfil «Equilibrado», clarify Q2).
pub const TEMP_WARN_MIN_C: f64 = 40.0;
pub const TEMP_WARN_MAX_C: f64 = 95.0;
pub const TEMP_CRIT_MAX_C: f64 = 100.0;

pub const TEMP_WARN_DEFAULT_C: f64 = 60.0;
pub const TEMP_CRIT_DEFAULT_C: f64 = 70.0;

/// Desgaste (`percentage_used`) configurable (ADR-036): `smart.wear_high` deja de llevar `90/100`
/// literal. Fábrica 80/90 (perfil «Equilibrado»).
pub const WEAR_WARN_PERCENT_MIN: f64 = 50.0;
pub const WEAR_WARN_PERCENT_MAX: f64 = 99.0;
pub const WEAR_CRIT_PERCENT_MAX: f64 = 100.0;
pub const WEAR_WARN_PERCENT_DEFAULT: f64 = 80.0;
pub const WEAR_CRIT_PERCENT_DEFAULT: f64 = 90.0;

/// Umbral de errores de medios: la **magnitud del incremento** de `media_errors_total` entre dos
/// lecturas consecutivas que basta para avisar (clarify Q1 — el nombre lleva `Per24h` por
/// continuidad con la propuesta del diseñador, pero no es una ventana de 24 h). Fábrica 1/5.
pub const MEDIA_ERRORS_MIN: i64 = 1;
pub const MEDIA_ERRORS_MAX: i64 = 1000;
pub const MEDIA_ERRORS_WARN_PER24H_DEFAULT: i64 = 1;
pub const MEDIA_ERRORS_CRIT_PER24H_DEFAULT: i64 = 5;

/// Umbral de reintentos del controlador. **Aún sin consumidor**: la regla
/// `events.controller_reset`/`io_retry` necesita el colector de eventos (Historia 4). Se guarda para
/// que un perfil escriba el juego completo de 12 valores (`open-questions.md`). Fábrica 5/12.
pub const DRIVER_RETRY_MIN: i64 = 1;
pub const DRIVER_RETRY_MAX: i64 = 1000;
pub const DRIVER_RETRY_WARN_PER24H_DEFAULT: i64 = 5;
pub const DRIVER_RETRY_CRIT_PER24H_DEFAULT: i64 = 12;

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

/// Valida el par aviso/crítico de desgaste (%): mismo criterio que temperatura — el crítico es un
/// porcentaje **mayor** (más desgaste es más grave).
pub fn validar_desgaste(warn_pct: f64, crit_pct: f64) -> Result<(), ErrorAjuste> {
    en_rango(warn_pct, WEAR_WARN_PERCENT_MIN, WEAR_WARN_PERCENT_MAX)?;
    en_rango(crit_pct, WEAR_WARN_PERCENT_MIN, WEAR_CRIT_PERCENT_MAX)?;
    if crit_pct <= warn_pct {
        return Err(ErrorAjuste::CriticoNoMasSeveroQueAviso);
    }
    Ok(())
}

/// Valida el par aviso/crítico de errores de medios (incremento por lectura): crítico **mayor**.
pub fn validar_media_errors(warn: i64, crit: i64) -> Result<(), ErrorAjuste> {
    en_rango(
        warn as f64,
        MEDIA_ERRORS_MIN as f64,
        MEDIA_ERRORS_MAX as f64,
    )?;
    en_rango(
        crit as f64,
        MEDIA_ERRORS_MIN as f64,
        MEDIA_ERRORS_MAX as f64,
    )?;
    if crit <= warn {
        return Err(ErrorAjuste::CriticoNoMasSeveroQueAviso);
    }
    Ok(())
}

/// Valida el par aviso/crítico de reintentos del controlador: crítico **mayor**.
pub fn validar_reintentos_controlador(warn: i64, crit: i64) -> Result<(), ErrorAjuste> {
    en_rango(
        warn as f64,
        DRIVER_RETRY_MIN as f64,
        DRIVER_RETRY_MAX as f64,
    )?;
    en_rango(
        crit as f64,
        DRIVER_RETRY_MIN as f64,
        DRIVER_RETRY_MAX as f64,
    )?;
    if crit <= warn {
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

const GIB: i64 = 1024 * 1024 * 1024;

/// Los doce umbrales que un perfil de alerta escribe de golpe (`docs/data-model.md` §3.3,
/// `cambios/08b-perfiles-de-alerta.md`). `capacity_absolute_floor_min_capacity_bytes` (256 GiB) **no**
/// forma parte de un perfil: se queda constante.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UmbralesPerfil {
    pub temp_warn_c: f64,
    pub temp_crit_c: f64,
    pub wear_warn_percent: f64,
    pub wear_crit_percent: f64,
    pub capacity_warn_percent: f64,
    pub capacity_crit_percent: f64,
    pub capacity_absolute_floor_warn_bytes: i64,
    pub capacity_absolute_floor_crit_bytes: i64,
    pub media_errors_warn_per24h: i64,
    pub media_errors_crit_per24h: i64,
    pub driver_retry_warn_per24h: i64,
    pub driver_retry_crit_per24h: i64,
}

/// Perfil de alerta. El identificador que cruza a la interfaz (`settings.alerts.profile`) es el
/// texto en inglés; `Personalizado` es el estado tras editar un umbral a mano.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerfilAlerta {
    Prudente,
    Equilibrado,
    SoloLoGrave,
    Personalizado,
}

impl PerfilAlerta {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "cautious" => Some(Self::Prudente),
            "balanced" => Some(Self::Equilibrado),
            "quiet" => Some(Self::SoloLoGrave),
            "custom" => Some(Self::Personalizado),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Prudente => "cautious",
            Self::Equilibrado => "balanced",
            Self::SoloLoGrave => "quiet",
            Self::Personalizado => "custom",
        }
    }

    /// Los doce valores del perfil. `Personalizado` no tiene: sus valores son los que haya en
    /// `settings` en cada momento.
    pub fn umbrales(&self) -> Option<UmbralesPerfil> {
        let u = match self {
            Self::Prudente => UmbralesPerfil {
                temp_warn_c: 55.0,
                temp_crit_c: 65.0,
                wear_warn_percent: 70.0,
                wear_crit_percent: 85.0,
                capacity_warn_percent: 15.0,
                capacity_crit_percent: 8.0,
                capacity_absolute_floor_warn_bytes: 30 * GIB,
                capacity_absolute_floor_crit_bytes: 15 * GIB,
                media_errors_warn_per24h: 1,
                media_errors_crit_per24h: 3,
                driver_retry_warn_per24h: 2,
                driver_retry_crit_per24h: 6,
            },
            Self::Equilibrado => UmbralesPerfil {
                temp_warn_c: TEMP_WARN_DEFAULT_C,
                temp_crit_c: TEMP_CRIT_DEFAULT_C,
                wear_warn_percent: WEAR_WARN_PERCENT_DEFAULT,
                wear_crit_percent: WEAR_CRIT_PERCENT_DEFAULT,
                capacity_warn_percent: CAPACITY_WARN_PERCENT_DEFAULT,
                capacity_crit_percent: CAPACITY_CRIT_PERCENT_DEFAULT,
                capacity_absolute_floor_warn_bytes: CAPACITY_FLOOR_WARN_DEFAULT_BYTES,
                capacity_absolute_floor_crit_bytes: CAPACITY_FLOOR_CRIT_DEFAULT_BYTES,
                media_errors_warn_per24h: MEDIA_ERRORS_WARN_PER24H_DEFAULT,
                media_errors_crit_per24h: MEDIA_ERRORS_CRIT_PER24H_DEFAULT,
                driver_retry_warn_per24h: DRIVER_RETRY_WARN_PER24H_DEFAULT,
                driver_retry_crit_per24h: DRIVER_RETRY_CRIT_PER24H_DEFAULT,
            },
            Self::SoloLoGrave => UmbralesPerfil {
                temp_warn_c: 70.0,
                temp_crit_c: 80.0,
                wear_warn_percent: 90.0,
                wear_crit_percent: 95.0,
                capacity_warn_percent: 5.0,
                capacity_crit_percent: 2.0,
                capacity_absolute_floor_warn_bytes: 10 * GIB,
                capacity_absolute_floor_crit_bytes: 5 * GIB,
                media_errors_warn_per24h: 5,
                media_errors_crit_per24h: 15,
                driver_retry_warn_per24h: 12,
                driver_retry_crit_per24h: 30,
            },
            Self::Personalizado => return None,
        };
        Some(u)
    }
}

/// El perfil de fábrica: «Equilibrado». Es lo que aplica «Omitir» en el asistente (PR 9) y lo que
/// devuelve `get_settings` cuando `settings.alerts.profile` no está guardado.
pub const PERFIL_DEFECTO: PerfilAlerta = PerfilAlerta::Equilibrado;

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
    fn el_umbral_termico_de_fabrica_es_60_70() {
        // clarify Q2: baja de 70/80. Si esto cambia, hay que revisar alert-rules.md §2 y sus pruebas.
        assert_eq!(TEMP_WARN_DEFAULT_C, 60.0);
        assert_eq!(TEMP_CRIT_DEFAULT_C, 70.0);
    }

    #[test]
    fn desgaste_de_fabrica_valido_y_critico_no_mas_severo_se_rechaza() {
        assert!(validar_desgaste(WEAR_WARN_PERCENT_DEFAULT, WEAR_CRIT_PERCENT_DEFAULT).is_ok());
        assert_eq!(
            validar_desgaste(80.0, 80.0),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
        assert!(
            validar_desgaste(40.0, 90.0).is_err(),
            "aviso por debajo del mínimo"
        );
        assert!(validar_desgaste(50.0, 50.5).is_ok());
    }

    #[test]
    fn media_errors_y_reintentos_validan_par_y_rango() {
        assert!(validar_media_errors(
            MEDIA_ERRORS_WARN_PER24H_DEFAULT,
            MEDIA_ERRORS_CRIT_PER24H_DEFAULT
        )
        .is_ok());
        assert_eq!(
            validar_media_errors(5, 3),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
        assert!(validar_media_errors(0, 5).is_err());
        assert!(validar_reintentos_controlador(
            DRIVER_RETRY_WARN_PER24H_DEFAULT,
            DRIVER_RETRY_CRIT_PER24H_DEFAULT
        )
        .is_ok());
        assert_eq!(
            validar_reintentos_controlador(12, 12),
            Err(ErrorAjuste::CriticoNoMasSeveroQueAviso)
        );
    }

    #[test]
    fn cada_perfil_produce_los_valores_de_la_tabla() {
        let p = PerfilAlerta::Prudente.umbrales().unwrap();
        assert_eq!((p.temp_warn_c, p.temp_crit_c), (55.0, 65.0));
        assert_eq!(p.driver_retry_crit_per24h, 6);

        let e = PerfilAlerta::Equilibrado.umbrales().unwrap();
        assert_eq!((e.temp_warn_c, e.temp_crit_c), (60.0, 70.0));
        assert_eq!((e.wear_warn_percent, e.wear_crit_percent), (80.0, 90.0));

        let q = PerfilAlerta::SoloLoGrave.umbrales().unwrap();
        assert_eq!((q.temp_warn_c, q.temp_crit_c), (70.0, 80.0));
        assert_eq!(q.capacity_crit_percent, 2.0);

        assert!(PerfilAlerta::Personalizado.umbrales().is_none());
    }

    #[test]
    fn los_valores_de_cada_perfil_pasan_su_propia_validacion() {
        for perfil in [
            PerfilAlerta::Prudente,
            PerfilAlerta::Equilibrado,
            PerfilAlerta::SoloLoGrave,
        ] {
            let u = perfil.umbrales().unwrap();
            assert!(
                validar_temperaturas(u.temp_warn_c, u.temp_crit_c).is_ok(),
                "{perfil:?}"
            );
            assert!(
                validar_desgaste(u.wear_warn_percent, u.wear_crit_percent).is_ok(),
                "{perfil:?}"
            );
            assert!(
                validar_capacidad_porcentual(u.capacity_warn_percent, u.capacity_crit_percent)
                    .is_ok(),
                "{perfil:?}"
            );
            assert!(
                validar_media_errors(u.media_errors_warn_per24h, u.media_errors_crit_per24h)
                    .is_ok(),
                "{perfil:?}"
            );
        }
    }

    #[test]
    fn from_id_ida_y_vuelta() {
        for id in ["cautious", "balanced", "quiet", "custom"] {
            assert_eq!(PerfilAlerta::from_id(id).unwrap().id(), id);
        }
        assert!(PerfilAlerta::from_id("otro").is_none());
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
