//! Cálculo de los intervalos de recopilación: frecuencias configurables y su reducción en batería
//! (FR-030, `product-specification.md` §4). Solo el cálculo puro: qué intervalo toca usar. El
//! bucle que de verdad duerme y despierta necesita un colector real, que todavía no existe
//! (llega con la Historia 1); esa parte queda para entonces, documentada en `tasks.md` T020.

use std::time::Duration;

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
}
