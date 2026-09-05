//! Cálculo de los intervalos de recopilación: frecuencias configurables y su reducción en batería
//! (FR-030, `product-specification.md` §4), y qué trabajos tocan ejecutarse ahora (T020). El bucle
//! que de verdad duerme y despierta vive en `lib.rs` (sondea esto cada 1 s, `open-questions.md`
//! J.34); este módulo solo decide, nunca ejecuta ni bloquea.

use std::time::{Duration, Instant};

/// Un trabajo de recopilación con su intervalo por defecto y sus límites válidos.
#[derive(Debug, Clone, Copy)]
pub struct Trabajo {
    pub por_defecto: Duration,
    pub minimo: Duration,
    pub maximo: Duration,
    /// Si es `true`, la batería no lo altera: alimenta alertas graves (FR-030).
    pub inmune_a_bateria: bool,
}

pub const METRICAS_RAPIDAS: Trabajo = Trabajo {
    por_defecto: Duration::from_secs(30),
    minimo: Duration::from_secs(10),
    maximo: Duration::from_secs(5 * 60),
    inmune_a_bateria: false,
};

pub const SMART_COMPLETO: Trabajo = Trabajo {
    por_defecto: Duration::from_secs(5 * 60),
    minimo: Duration::from_secs(60),
    maximo: Duration::from_secs(60 * 60),
    inmune_a_bateria: true,
};

pub const EVENTOS_WINDOWS: Trabajo = Trabajo {
    por_defecto: Duration::from_secs(30),
    minimo: Duration::from_secs(15),
    maximo: Duration::from_secs(5 * 60),
    inmune_a_bateria: true,
};

pub const ALTAS_Y_BAJAS: Trabajo = Trabajo {
    por_defecto: Duration::from_secs(60),
    minimo: Duration::from_secs(30),
    maximo: Duration::from_secs(10 * 60),
    inmune_a_bateria: false,
};

/// Multiplicador que se aplica en batería a los trabajos no inmunes (FR-030).
const MULTIPLICADOR_BATERIA: u32 = 4;

/// El intervalo configurado, ajustado por batería si corresponde. `configurado` debe venir ya
/// validado contra `minimo`/`maximo` (eso lo hace `domain::ajustes`, no este módulo).
pub fn intervalo_efectivo(trabajo: Trabajo, configurado: Duration, en_bateria: bool) -> Duration {
    if en_bateria && !trabajo.inmune_a_bateria {
        configurado * MULTIPLICADOR_BATERIA
    } else {
        configurado
    }
}

/// Identifica cada uno de los cuatro trabajos del bucle en segundo plano (T020). No es
/// intercambiable con `Trabajo`: `Trabajo` es la configuración (límites, inmunidad a batería),
/// esto es la etiqueta que permite recordar cuándo se ejecutó cada uno por última vez.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoTrabajo {
    MetricasRapidas,
    SmartCompleto,
    EventosWindows,
    AltasYBajas,
}

/// La frecuencia configurada de cada trabajo, ya validada y resuelta desde `settings`
/// (`domain::ajustes`, no este módulo).
#[derive(Debug, Clone, Copy)]
pub struct ConfiguracionTrabajos {
    pub metricas_rapidas: Duration,
    pub smart_completo: Duration,
    pub eventos_windows: Duration,
    pub altas_y_bajas: Duration,
}

/// Cuándo se ejecutó cada trabajo por última vez. Un trabajo ausente del mapa no se ha ejecutado
/// nunca en este proceso (arranque): siempre toca.
#[derive(Debug, Default)]
pub struct EstadoPlanificador {
    ultima_ejecucion: std::collections::HashMap<TipoTrabajo, Instant>,
}

impl EstadoPlanificador {
    pub fn nuevo() -> Self {
        Self::default()
    }

    pub fn marcar_ejecutado(&mut self, tipo: TipoTrabajo, ahora: Instant) {
        self.ultima_ejecucion.insert(tipo, ahora);
    }
}

/// Qué trabajos tocan ejecutarse ahora mismo, dado el estado de la última ejecución de cada uno,
/// la configuración vigente y si el equipo está en batería. Pura: sondear con qué frecuencia
/// llamarla es responsabilidad de quien la use (T020, `open-questions.md` J.34: cada 1 s).
pub fn trabajos_debidos(
    estado: &EstadoPlanificador,
    config: &ConfiguracionTrabajos,
    en_bateria: bool,
    ahora: Instant,
) -> Vec<TipoTrabajo> {
    let candidatos = [
        (
            TipoTrabajo::MetricasRapidas,
            METRICAS_RAPIDAS,
            config.metricas_rapidas,
        ),
        (
            TipoTrabajo::SmartCompleto,
            SMART_COMPLETO,
            config.smart_completo,
        ),
        (
            TipoTrabajo::EventosWindows,
            EVENTOS_WINDOWS,
            config.eventos_windows,
        ),
        (
            TipoTrabajo::AltasYBajas,
            ALTAS_Y_BAJAS,
            config.altas_y_bajas,
        ),
    ];

    candidatos
        .into_iter()
        .filter_map(|(tipo, trabajo, configurado)| {
            let intervalo = intervalo_efectivo(trabajo, configurado, en_bateria);
            match estado.ultima_ejecucion.get(&tipo) {
                None => Some(tipo),
                Some(ultima) => {
                    (ahora.saturating_duration_since(*ultima) >= intervalo).then_some(tipo)
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_red_electrica_no_cambia_el_intervalo() {
        let efectivo = intervalo_efectivo(METRICAS_RAPIDAS, Duration::from_secs(30), false);
        assert_eq!(efectivo, Duration::from_secs(30));
    }

    #[test]
    fn en_bateria_las_metricas_rapidas_se_multiplican_por_cuatro() {
        let efectivo = intervalo_efectivo(METRICAS_RAPIDAS, Duration::from_secs(30), true);
        assert_eq!(efectivo, Duration::from_secs(120));
    }

    #[test]
    fn en_bateria_smart_completo_no_se_altera() {
        // SMART completo alimenta alertas graves: la batería no lo toca (FR-030).
        let efectivo = intervalo_efectivo(SMART_COMPLETO, Duration::from_secs(300), true);
        assert_eq!(efectivo, Duration::from_secs(300));
    }

    #[test]
    fn en_bateria_eventos_de_windows_no_se_altera() {
        let efectivo = intervalo_efectivo(EVENTOS_WINDOWS, Duration::from_secs(30), true);
        assert_eq!(efectivo, Duration::from_secs(30));
    }

    #[test]
    fn en_bateria_altas_y_bajas_si_se_multiplica() {
        let efectivo = intervalo_efectivo(ALTAS_Y_BAJAS, Duration::from_secs(60), true);
        assert_eq!(efectivo, Duration::from_secs(240));
    }

    fn config_de_prueba() -> ConfiguracionTrabajos {
        ConfiguracionTrabajos {
            metricas_rapidas: Duration::from_secs(30),
            smart_completo: Duration::from_secs(300),
            eventos_windows: Duration::from_secs(30),
            altas_y_bajas: Duration::from_secs(60),
        }
    }

    #[test]
    fn un_trabajo_nunca_ejecutado_siempre_toca_arranque() {
        let estado = EstadoPlanificador::nuevo();
        let debidos = trabajos_debidos(&estado, &config_de_prueba(), false, Instant::now());
        assert_eq!(debidos.len(), 4, "al arrancar, los cuatro trabajos tocan");
    }

    #[test]
    fn un_trabajo_ejecutado_hace_menos_que_su_intervalo_no_toca() {
        let mut estado = EstadoPlanificador::nuevo();
        let ahora = Instant::now();
        estado.marcar_ejecutado(TipoTrabajo::MetricasRapidas, ahora);
        // Los otros tres siguen sin ejecutarse nunca: solo se comprueba que MetricasRapidas no está.
        let debidos = trabajos_debidos(&estado, &config_de_prueba(), false, ahora);
        assert!(!debidos.contains(&TipoTrabajo::MetricasRapidas));
        assert!(debidos.contains(&TipoTrabajo::SmartCompleto));
    }

    #[test]
    fn un_trabajo_ejecutado_hace_mas_que_su_intervalo_vuelve_a_tocar() {
        let mut estado = EstadoPlanificador::nuevo();
        let hace_un_minuto = Instant::now();
        estado.marcar_ejecutado(TipoTrabajo::AltasYBajas, hace_un_minuto);
        let ahora = hace_un_minuto + Duration::from_secs(61);
        let debidos = trabajos_debidos(&estado, &config_de_prueba(), false, ahora);
        assert!(debidos.contains(&TipoTrabajo::AltasYBajas));
    }

    #[test]
    fn en_bateria_un_trabajo_no_inmune_tarda_mas_en_volver_a_tocar() {
        let mut estado = EstadoPlanificador::nuevo();
        let hace_35_segundos = Instant::now();
        estado.marcar_ejecutado(TipoTrabajo::MetricasRapidas, hace_35_segundos);
        let ahora = hace_35_segundos + Duration::from_secs(35);

        // En red, 35 s ya superan el intervalo configurado de 30 s: toca.
        let en_red = trabajos_debidos(&estado, &config_de_prueba(), false, ahora);
        assert!(en_red.contains(&TipoTrabajo::MetricasRapidas));

        // En batería, el intervalo efectivo es 30 s × 4 = 120 s: 35 s no bastan.
        let en_bateria = trabajos_debidos(&estado, &config_de_prueba(), true, ahora);
        assert!(!en_bateria.contains(&TipoTrabajo::MetricasRapidas));
    }

    #[test]
    fn en_bateria_un_trabajo_inmune_no_cambia_su_cadencia() {
        let mut estado = EstadoPlanificador::nuevo();
        let hace_31_segundos = Instant::now();
        estado.marcar_ejecutado(TipoTrabajo::EventosWindows, hace_31_segundos);
        let ahora = hace_31_segundos + Duration::from_secs(31);

        // EVENTOS_WINDOWS es inmune a batería: 31 s ya superan los 30 s configurados, en red o no.
        let en_bateria = trabajos_debidos(&estado, &config_de_prueba(), true, ahora);
        assert!(en_bateria.contains(&TipoTrabajo::EventosWindows));
    }
}
