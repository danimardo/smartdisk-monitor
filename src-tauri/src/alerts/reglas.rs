//! Clasificación de reglas de alerta por `rule_key`, sin persistencia ni evaluación.
//!
//! Hoy solo responde a una pregunta: ¿se puede ignorar una alerta de esta regla? El conjunto
//! vetado (ADR-044, enmendado por ADR-045; `docs/alert-rules.md` §1) son las reglas que señalan
//! daño físico o predicción de fallo del propio disco: ignorarlas para siempre convertiría el
//! monitor en algo que oculta su motivo de existir (constitución §I).

/// Reglas para las que la acción «Ignorar» está vetada. Lista cerrada y **no** configurable.
/// Cambiarla exige mover a la vez esta constante, su prueba y `docs/alert-rules.md`.
pub const REGLAS_NO_IGNORABLES: &[&str] = &[
    "smart.health.failed",
    "nvme.critical_warning",
    "smart.wear_high",
    "smart.spare_below_threshold",
    "smart.media_errors",
    "events.disk_predictive",
];

/// ¿Una alerta de `rule_key` se puede llevar al estado `ignored`?
pub fn regla_es_ignorable(rule_key: &str) -> bool {
    !REGLAS_NO_IGNORABLES.contains(&rule_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Espejo de la columna «`rule_key`» de la tabla de `docs/alert-rules.md` §2. Si se añade una
    /// regla nueva al motor, esta lista se queda corta y el test de abajo lo dice.
    const TODAS_LAS_REGLAS: &[&str] = &[
        "smart.health.failed",
        "nvme.critical_warning",
        "smart.media_errors",
        "smart.error_log",
        "smart.spare_below_threshold",
        "smart.wear_high",
        "temp.above_vendor_limit",
        "temp.above_vendor_critical",
        "temp.above_configured_warn",
        "temp.above_configured_crit",
        "capacity.low",
        "capacity.critical",
        "device.removed_unexpected",
        "events.disk_error",
        "events.filesystem_error",
        "events.filesystem_repaired",
        "events.filesystem_repair_storm",
        "events.controller_reset",
        "events.paging_error",
        "events.io_retry",
        "events.delayed_write",
        "events.disk_predictive",
        "events.storage_space_degraded",
        "inventory.duplicate_id",
        "smart.unreadable",
        "collector.stalled",
    ];

    #[test]
    fn exactamente_seis_reglas_estan_vetadas() {
        assert_eq!(REGLAS_NO_IGNORABLES.len(), 6);
    }

    #[test]
    fn el_conjunto_vetado_son_señales_de_fallo_fisico_o_prediccion() {
        for regla in REGLAS_NO_IGNORABLES {
            assert!(!regla_es_ignorable(regla), "{regla} debería estar vetada");
        }
    }

    #[test]
    fn toda_regla_conocida_fuera_del_conjunto_es_ignorable() {
        for regla in TODAS_LAS_REGLAS {
            let esperado = !REGLAS_NO_IGNORABLES.contains(regla);
            assert_eq!(
                regla_es_ignorable(regla),
                esperado,
                "{regla}: ignorable esperado = {esperado}"
            );
        }
    }

    #[test]
    fn eventos_de_sistema_de_archivos_y_de_disco_si_son_ignorables() {
        // Aclaración de 2026-09-08 (spec.md): pueden tener causas ajenas al disco.
        assert!(regla_es_ignorable("events.filesystem_error"));
        assert!(regla_es_ignorable("events.disk_error"));
    }

    #[test]
    fn smart_error_log_es_ignorable() {
        // ADR-045: el contador bruto `num_err_log_entries` en NVMe de consumo lo dominan rechazos
        // de protocolo benignos («Invalid Field in Command»), no daño de medio. El daño real de
        // medio lo cubre `smart.media_errors`, que sí sigue vetada.
        assert!(regla_es_ignorable("smart.error_log"));
        assert!(!regla_es_ignorable("smart.media_errors"));
    }

    #[test]
    fn una_regla_desconocida_se_considera_ignorable_por_defecto() {
        assert!(regla_es_ignorable("regla.que.no.existe"));
    }
}
