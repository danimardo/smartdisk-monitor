//! Estado de ejecución del ciclo de recopilación: pausa y escritura de historial detenida
//! (delta D2/C1 de `specs/001-monitor-discos-windows`).
//!
//! No decide salud de dispositivos ni evalúa reglas: solo si el ciclo escribe y si está pausado.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EstadoEjecucion {
    pub paused: bool,
    pub history_write_halted: bool,
}

impl EstadoEjecucion {
    /// El estado al arrancar. **Siempre reanuda**, con independencia de cómo quedara la pausa
    /// persistida: una pausa olvidada es un monitor que no vigila y no lo dice (FR-031, US-074).
    pub fn al_arrancar() -> Self {
        Self {
            paused: false,
            history_write_halted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn al_arrancar_nunca_esta_pausado() {
        assert!(!EstadoEjecucion::al_arrancar().paused);
    }

    #[test]
    fn al_arrancar_la_escritura_de_historial_no_esta_detenida() {
        // Se reevalúa contra el espacio libre real en el primer ciclo (domain::espacio); al
        // arrancar no se asume un fallo que todavía no se ha medido.
        assert!(!EstadoEjecucion::al_arrancar().history_write_halted);
    }
}
