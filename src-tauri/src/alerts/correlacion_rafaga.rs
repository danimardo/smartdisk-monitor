//! Ventana de correlación de ráfagas (`docs/alert-rules.md` §3.5, `docs/open-questions.md` D4).
//!
//! Un solo hecho físico —una desconexión en caliente— produce varios eventos distintos del mismo
//! disco en segundos (`disk` 157 + `disk` 51 + `Ntfs` 50 + `Microsoft-Windows-Ntfs` 140). Sin esta
//! capa, cada uno crearía su propio grupo de alerta. La regla: los eventos del mismo disco dentro
//! de **60 segundos** se tratan como un suceso, y si entre ellos hay un `disk` 157 (extracción
//! imprevista) **ese es la causa** y los demás pasan a ser ocurrencias suyas.
//!
//! Función pura: recibe el evento nuevo y los eventos del mismo disco ya persistidos en la ventana
//! de 60 s hacia atrás, y decide qué hacer con el evento nuevo.

use time::Duration;

/// El id de evento de «El disco N se ha extraído de forma imprevista» (proveedor clásico `disk`).
pub const EVENT_ID_EXTRACCION_IMPREVISTA: i64 = 157;

/// Ancho de la ventana de correlación (`docs/alert-rules.md` §3.5).
pub const VENTANA: Duration = Duration::seconds(60);

/// Un evento mínimo para decidir correlación: id y momento. El resto (regla, severidad) lo decide
/// `reglas_eventos` aparte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EventoEnVentana {
    /// `system_events.id`.
    pub id: i64,
    pub provider_es_disk: bool,
    pub event_id: i64,
    /// Segundos entre este evento y el evento nuevo que se está evaluando (≥ 0).
    pub antiguedad: Duration,
}

/// Qué hacer con el evento nuevo tras mirar su ventana de 60 s.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultadoCorrelacion {
    /// No hay causa en la ventana: el evento sigue su propia regla y crea/alimenta su grupo.
    SiguePropiaRegla,
    /// Hay un `disk` 157 en la ventana: este evento es una ocurrencia de `device.removed_unexpected`
    /// del mismo disco, no un grupo propio. Lleva el `system_events.id` de la causa.
    OcurrenciaDeExtraccion { causa_event_id: i64 },
    /// El evento nuevo **es** el `disk` 157 y en su ventana hay grupos de evento del mismo disco
    /// creados justo antes: hay que reasignarlos (resolverlos con nota de causa y re-registrar sus
    /// eventos bajo `device.removed_unexpected`).
    EsCausaConDerivadosPrevios,
}

/// ¿Es este `(provider, event_id)` la causa de una ráfaga (una extracción imprevista)?
pub fn es_extraccion_imprevista(provider_es_disk: bool, event_id: i64) -> bool {
    provider_es_disk && event_id == EVENT_ID_EXTRACCION_IMPREVISTA
}

/// Decide qué hacer con `evento_nuevo` (id de evento de Windows + si es del proveedor `disk`) dada
/// su ventana de 60 s de eventos del mismo disco ya persistidos.
///
/// `hay_grupos_de_evento_recientes` lo aporta quien llama consultando `alert_groups` (no es asunto
/// de esta función pura): ¿tiene este disco grupos de reglas `events.*` creados dentro de la
/// ventana? Solo importa cuando el evento nuevo **es** el `disk` 157 que llega tarde.
pub fn correlacionar_rafaga(
    evento_nuevo_event_id: i64,
    evento_nuevo_es_disk: bool,
    ventana: &[EventoEnVentana],
    hay_grupos_de_evento_recientes: bool,
) -> ResultadoCorrelacion {
    let nuevo_es_causa = es_extraccion_imprevista(evento_nuevo_es_disk, evento_nuevo_event_id);

    let causa_en_ventana = ventana
        .iter()
        .filter(|e| e.antiguedad <= VENTANA)
        .find(|e| es_extraccion_imprevista(e.provider_es_disk, e.event_id));

    match (
        nuevo_es_causa,
        causa_en_ventana,
        hay_grupos_de_evento_recientes,
    ) {
        // El evento nuevo es el `disk` 157 y llega **después** de derivados ya agrupados.
        (true, _, true) => ResultadoCorrelacion::EsCausaConDerivadosPrevios,
        // El evento nuevo es el `disk` 157 y no hay derivados previos: crea su propio grupo.
        (true, _, false) => ResultadoCorrelacion::SiguePropiaRegla,
        // El evento nuevo es un derivado y la causa (`disk` 157) ya está en su ventana.
        (false, Some(causa), _) => ResultadoCorrelacion::OcurrenciaDeExtraccion {
            causa_event_id: causa.id,
        },
        // Sin causa en la ventana: cada regla sigue su camino (FR-009a).
        (false, None, _) => ResultadoCorrelacion::SiguePropiaRegla,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn en_ventana(id: i64, es_disk: bool, event_id: i64, hace_s: i64) -> EventoEnVentana {
        EventoEnVentana {
            id,
            provider_es_disk: es_disk,
            event_id,
            antiguedad: Duration::seconds(hace_s),
        }
    }

    #[test]
    fn sin_causa_en_la_ventana_cada_evento_sigue_su_regla() {
        // Un Ntfs 55 y un disk 7 a la vez, sin disk 157: dos grupos (FR-009a).
        let ventana = [en_ventana(1, false, 55, 3)];
        assert_eq!(
            correlacionar_rafaga(7, true, &ventana, false),
            ResultadoCorrelacion::SiguePropiaRegla
        );
    }

    #[test]
    fn un_derivado_con_disk_157_en_la_ventana_es_ocurrencia_de_la_extraccion() {
        let ventana = [en_ventana(99, true, 157, 5), en_ventana(50, false, 50, 2)];
        assert_eq!(
            correlacionar_rafaga(51, true, &ventana, false),
            ResultadoCorrelacion::OcurrenciaDeExtraccion { causa_event_id: 99 }
        );
    }

    #[test]
    fn el_disk_157_sin_derivados_previos_crea_su_propio_grupo() {
        assert_eq!(
            correlacionar_rafaga(157, true, &[], false),
            ResultadoCorrelacion::SiguePropiaRegla
        );
    }

    #[test]
    fn el_disk_157_que_llega_tarde_reasigna_los_grupos_derivados() {
        let ventana = [en_ventana(50, false, 50, 10)];
        assert_eq!(
            correlacionar_rafaga(157, true, &ventana, true),
            ResultadoCorrelacion::EsCausaConDerivadosPrevios
        );
    }

    #[test]
    fn un_disk_157_fuera_de_la_ventana_de_60s_ya_no_es_causa() {
        let ventana = [en_ventana(99, true, 157, 75)];
        assert_eq!(
            correlacionar_rafaga(50, false, &ventana, false),
            ResultadoCorrelacion::SiguePropiaRegla
        );
    }

    #[test]
    fn el_157_de_otro_proveedor_no_cuenta_como_extraccion() {
        // Solo el proveedor clásico `disk` emite el 157.
        assert!(!es_extraccion_imprevista(false, 157));
        let ventana = [en_ventana(1, false, 157, 3)];
        assert_eq!(
            correlacionar_rafaga(50, false, &ventana, false),
            ResultadoCorrelacion::SiguePropiaRegla
        );
    }
}
