//! Detección de altas y bajas en caliente sobre el inventario de discos (FR-001, US-014).
//!
//! Combina `windows_storage::list_physical_disks()` con `domain::identidad::reconcile` para saber
//! qué apareció y qué desapareció desde la última lectura. Distinguir una baja por **expulsión
//! segura** de una **retirada sin aviso** exige buscar el evento `disk` 157 del registro de
//! Windows junto a esa desaparición (`docs/alert-rules.md`, regla `device.removed_unexpected`).
//!
//! **Esa correlación todavía no está disponible aquí.** El colector del registro de eventos y la
//! atribución evento→dispositivo son la Historia 4 (`collectors::event_log`, T067;
//! `domain::correlacion`, T069), que no existe todavía. Lo que sí puede hacerse ya —y hace este
//! módulo— es la parte independiente de esa dependencia: reconciliar el inventario y dejar la
//! clasificación de la baja como una decisión pura que toma como entrada si se encontró o no el
//! evento, para que conectarla, cuando exista el colector de eventos, sea sustituir el `bool`
//! fijo de la llamada por una consulta real — no reescribir esta lógica.

use crate::domain::identidad::{reconcile, Reconciliacion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoBaja {
    /// Precedida de una expulsión segura: solo un apunte de inventario, sin alerta (US-014).
    Segura,
    /// Sin expulsión previa: alerta crítica, o advertencia si el bus es USB
    /// (`docs/alert-rules.md`, `device.removed_unexpected`).
    SinAviso,
}

/// `true` si la desaparición del disco fue precedida de una solicitud de expulsión segura, tal
/// como la refleja (cuando exista) la correlación con el evento `disk` 157 del registro de
/// Windows: su **ausencia** es la señal de expulsión segura, no su presencia — el 157 es
/// justamente el aviso de que *no* hubo expulsión previa.
pub fn clasificar_baja(se_encontro_evento_extraccion_imprevista: bool) -> TipoBaja {
    if se_encontro_evento_extraccion_imprevista {
        TipoBaja::SinAviso
    } else {
        TipoBaja::Segura
    }
}

/// Detecta altas y bajas comparando el inventario ya presente contra el recién leído. No decide
/// severidad de alerta —eso es `alerts/`—, solo qué cambió.
pub fn detectar_cambios(huellas_presentes: &[String], huellas_leidas: &[String]) -> Reconciliacion {
    reconcile(huellas_presentes, huellas_leidas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_evento_de_extraccion_imprevista_la_baja_es_segura() {
        assert_eq!(clasificar_baja(false), TipoBaja::Segura);
    }

    #[test]
    fn con_evento_de_extraccion_imprevista_la_baja_es_sin_aviso() {
        assert_eq!(clasificar_baja(true), TipoBaja::SinAviso);
    }

    #[test]
    fn detectar_cambios_delega_en_la_reconciliacion_de_dominio() {
        let antes = vec!["a".to_string()];
        let ahora = vec!["b".to_string()];
        let r = detectar_cambios(&antes, &ahora);
        assert_eq!(r.altas, vec!["b".to_string()]);
        assert_eq!(r.bajas, vec!["a".to_string()]);
    }
}
