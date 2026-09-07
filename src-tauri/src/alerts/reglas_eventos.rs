//! Tabla `(proveedor, id) → regla de alerta` para los eventos del registro de Windows
//! (`docs/alert-rules.md` §3.2). Refleja la especificación normativa fila por fila; la prueba de
//! completitud y fidelidad (`mod tests`) verifica que no se desvía de ella.
//!
//! **Por qué en código y no en `settings`** (`docs/open-questions.md` J.48): la lista de
//! *proveedores* vigilados (`collectors::event_log::PROVEEDORES_VIGILADOS`) es configuración de
//! ingesta y puede ampliarse sin recompilar; la *semántica* de una regla —severidad, resolución,
//! contexto de deduplicación— es dominio normativo y no la edita el usuario.
//!
//! `alert-rules.md` §2 es la fuente normativa por encima de §3.2 (cabecera del documento). Cuando
//! §3.2 da menos detalle que §2 (p. ej. `Microsoft-Windows-Ntfs` 140), manda §2.

use time::Duration;

use crate::domain::tipos::AlertSeverity;

/// A qué apunta la alerta de una regla de evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objetivo {
    /// El disco correlacionado (`device:<id>`), o sin objeto si la atribución es `unknown`.
    Dispositivo,
    /// El volumen (`volume:<guid>`), o sin objeto si no se conoce.
    Volumen,
    /// La regla no apunta a ningún disco ni volumen (`inventory.duplicate_id`,
    /// `events.storage_space_degraded`).
    SinObjeto,
}

/// Discriminante de la clave de deduplicación (`docs/alert-rules.md` §1, columna «Contexto de
/// dedup»).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextoDedup {
    /// `provider:event_id` — un grupo por tipo de evento sobre el mismo objetivo.
    ProviderEventId,
    /// El `volume_guid` — un grupo por volumen, sin importar el id concreto del evento.
    VolumeGuid,
    /// El identificador del disco (`events.paging_error`/`io_retry`).
    DeviceId,
    /// La pareja de discos con identidad compartida (`inventory.duplicate_id`).
    ParDeDiscos,
    /// Sin discriminante: el objetivo ya basta (`events.disk_predictive`,
    /// `device.removed_unexpected`).
    Ninguno,
}

/// Cómo decide la severidad una regla de evento.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evaluacion {
    /// Un solo evento activa la regla, siempre con la misma severidad
    /// (`disk_error`, `filesystem_error`, `storage_space_degraded`, `disk_predictive`,
    /// `filesystem_repaired`, `filesystem_repair_storm`).
    Inmediata { severidad: AlertSeverity },
    /// Activa al alcanzar `min_en_1h` eventos del mismo tipo sobre el mismo disco en una hora.
    /// `escala_crit_en_1h` sube a crítico con esa cuenta (`controller_reset`); `None` si nunca
    /// escala (`paging_error`, `io_retry`).
    PorFrecuencia {
        min_en_1h: u32,
        severidad: AlertSeverity,
        escala_crit_en_1h: Option<u32>,
    },
    /// Un solo evento activa la regla; la severidad depende de si el volumen es extraíble
    /// (`delayed_write`: advertencia siempre, crítico si el volumen **no** es extraíble —
    /// `alert-rules.md` §2, para ambos ids).
    CondicionalNoExtraible {
        base: AlertSeverity,
        si_no_extraible: AlertSeverity,
    },
    /// Un solo evento activa la regla; la severidad depende del bus del disco
    /// (`device.removed_unexpected`: crítico salvo bus USB → advertencia).
    SegunBus {
        fijo: AlertSeverity,
        usb: AlertSeverity,
    },
    /// La regla tiene un disparador o un contexto que no encaja en el evaluador genérico y la
    /// maneja un camino propio (`inventory.duplicate_id`: hay que extraer la pareja de discos del
    /// evento; `device.removed_unexpected` también se dispara desde la baja de inventario).
    Especial,
}

/// Una fila de `docs/alert-rules.md` §3.2: qué combinación de proveedor/id vigila, con qué regla,
/// objetivo, contexto y evaluación.
#[derive(Debug, Clone, Copy)]
pub struct ClasificacionEvento {
    pub provider: &'static str,
    pub event_ids: &'static [i64],
    pub rule_key: &'static str,
    pub objetivo: Objetivo,
    pub dedup_context: ContextoDedup,
    pub evaluacion: Evaluacion,
    /// Ventana «sin repetición» tras la que un grupo activo se resuelve (`docs/alert-rules.md` §2,
    /// columna «Resolución»). `None` cuando la regla no se resuelve por tiempo
    /// (`device.removed_unexpected` resuelve al reaparecer el disco).
    pub ventana_resolucion: Option<Duration>,
    /// Solo eventos sobre discos **no** extraíbles cuentan para esta regla (`events.paging_error`,
    /// `docs/alert-rules.md` §3.3).
    pub solo_no_extraibles: bool,
}

const H24: Option<Duration> = Some(Duration::hours(24));
const D7: Option<Duration> = Some(Duration::days(7));

use AlertSeverity::{Critical, Warning};
use ContextoDedup as C;
use Evaluacion::{CondicionalNoExtraible, Especial, Inmediata, PorFrecuencia, SegunBus};
use Objetivo as O;

/// La tabla completa (`docs/alert-rules.md` §3.2 + §2). Las 10 reglas `events.*`, más `disk` 157
/// (`device.removed_unexpected`, que además se dispara desde la baja de inventario, J.47) y `disk`
/// 158 (`inventory.duplicate_id`).
pub const TABLA: &[ClasificacionEvento] = &[
    // --- events.disk_error (crítico inmediato) ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[7],
        rule_key: "events.disk_error",
        objetivo: O::Dispositivo,
        dedup_context: C::ProviderEventId,
        evaluacion: Inmediata {
            severidad: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    ClasificacionEvento {
        provider: "Microsoft-Windows-NvmeDisk",
        event_ids: &[500],
        rule_key: "events.disk_error",
        objetivo: O::Dispositivo,
        dedup_context: C::ProviderEventId,
        evaluacion: Inmediata {
            severidad: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    ClasificacionEvento {
        provider: "Microsoft-Windows-StorageSpaces-Driver",
        event_ids: &[202, 203, 209],
        rule_key: "events.disk_error",
        objetivo: O::Dispositivo,
        dedup_context: C::ProviderEventId,
        evaluacion: Inmediata {
            severidad: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    // --- events.filesystem_error (crítico inmediato; Ntfs 55 y 131) ---
    ClasificacionEvento {
        provider: "Ntfs",
        event_ids: &[55, 131],
        rule_key: "events.filesystem_error",
        objetivo: O::Volumen,
        dedup_context: C::ProviderEventId,
        evaluacion: Inmediata {
            severidad: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    // --- events.filesystem_repaired (Ntfs 130, advertencia) ---
    ClasificacionEvento {
        provider: "Ntfs",
        event_ids: &[130],
        rule_key: "events.filesystem_repaired",
        objetivo: O::Volumen,
        dedup_context: C::VolumeGuid,
        evaluacion: Inmediata { severidad: Warning },
        ventana_resolucion: D7,
        solo_no_extraibles: false,
    },
    // --- events.filesystem_repair_storm (Ntfs 132, crítico) ---
    ClasificacionEvento {
        provider: "Ntfs",
        event_ids: &[132],
        rule_key: "events.filesystem_repair_storm",
        objetivo: O::Volumen,
        dedup_context: C::VolumeGuid,
        evaluacion: Inmediata {
            severidad: Critical,
        },
        ventana_resolucion: D7,
        solo_no_extraibles: false,
    },
    // --- events.delayed_write (Ntfs 50 / Ms-Windows-Ntfs 140; warn, crit si no extraíble) ---
    ClasificacionEvento {
        provider: "Ntfs",
        event_ids: &[50],
        rule_key: "events.delayed_write",
        objetivo: O::Volumen,
        dedup_context: C::VolumeGuid,
        evaluacion: CondicionalNoExtraible {
            base: Warning,
            si_no_extraible: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    ClasificacionEvento {
        provider: "Microsoft-Windows-Ntfs",
        event_ids: &[140],
        rule_key: "events.delayed_write",
        objetivo: O::Volumen,
        dedup_context: C::VolumeGuid,
        evaluacion: CondicionalNoExtraible {
            base: Warning,
            si_no_extraible: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    // --- events.controller_reset (disk 11 / stornvme|storahci 129; warn, crit ≥3/h) ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[11],
        rule_key: "events.controller_reset",
        objetivo: O::Dispositivo,
        dedup_context: C::ProviderEventId,
        evaluacion: PorFrecuencia {
            min_en_1h: 1,
            severidad: Warning,
            escala_crit_en_1h: Some(3),
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    ClasificacionEvento {
        provider: "stornvme",
        event_ids: &[129],
        rule_key: "events.controller_reset",
        objetivo: O::Dispositivo,
        dedup_context: C::ProviderEventId,
        evaluacion: PorFrecuencia {
            min_en_1h: 1,
            severidad: Warning,
            escala_crit_en_1h: Some(3),
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    ClasificacionEvento {
        provider: "storahci",
        event_ids: &[129],
        rule_key: "events.controller_reset",
        objetivo: O::Dispositivo,
        dedup_context: C::ProviderEventId,
        evaluacion: PorFrecuencia {
            min_en_1h: 1,
            severidad: Warning,
            escala_crit_en_1h: Some(3),
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    // --- events.paging_error (disk 51; ≥10/h, solo discos no extraíbles, nunca crítico) ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[51],
        rule_key: "events.paging_error",
        objetivo: O::Dispositivo,
        dedup_context: C::DeviceId,
        evaluacion: PorFrecuencia {
            min_en_1h: 10,
            severidad: Warning,
            escala_crit_en_1h: None,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: true,
    },
    // --- events.io_retry (disk 153; ≥5/h) ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[153],
        rule_key: "events.io_retry",
        objetivo: O::Dispositivo,
        dedup_context: C::DeviceId,
        evaluacion: PorFrecuencia {
            min_en_1h: 5,
            severidad: Warning,
            escala_crit_en_1h: None,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    // --- events.disk_predictive (disk 52; advertencia) ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[52],
        rule_key: "events.disk_predictive",
        objetivo: O::Dispositivo,
        dedup_context: C::Ninguno,
        evaluacion: Inmediata { severidad: Warning },
        ventana_resolucion: D7,
        solo_no_extraibles: false,
    },
    // --- events.storage_space_degraded (StorageSpaces-Driver 300–311; crítico) ---
    ClasificacionEvento {
        provider: "Microsoft-Windows-StorageSpaces-Driver",
        event_ids: &[300, 301, 302, 303, 304, 305, 306, 307, 308, 309, 310, 311],
        rule_key: "events.storage_space_degraded",
        objetivo: O::SinObjeto,
        dedup_context: C::ProviderEventId,
        evaluacion: Inmediata {
            severidad: Critical,
        },
        ventana_resolucion: H24,
        solo_no_extraibles: false,
    },
    // --- device.removed_unexpected (disk 157; crítico, warn si USB). También desde inventario (J.47). ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[157],
        rule_key: "device.removed_unexpected",
        objetivo: O::Dispositivo,
        dedup_context: C::Ninguno,
        evaluacion: SegunBus {
            fijo: Critical,
            usb: Warning,
        },
        // Resuelve al reaparecer el disco en el inventario, no por tiempo.
        ventana_resolucion: None,
        solo_no_extraibles: false,
    },
    // --- inventory.duplicate_id (disk 158; advertencia, una vez por par) ---
    ClasificacionEvento {
        provider: "disk",
        event_ids: &[158],
        rule_key: "inventory.duplicate_id",
        objetivo: O::SinObjeto,
        dedup_context: C::ParDeDiscos,
        evaluacion: Especial,
        ventana_resolucion: D7,
        solo_no_extraibles: false,
    },
];

/// Ids que el filtro de ingesta deja pasar pero que **nunca** generan alerta (`docs/alert-rules.md`
/// §3.3): la prueba de completitud comprueba que ninguno tiene fila en `TABLA`.
#[cfg(test)]
const EXCLUIDOS_EXPLICITOS: &[(&str, i64)] = &[
    ("Microsoft-Windows-Ntfs", 98),
    ("Microsoft-Windows-NvmeDisk", 501),
    ("Volsnap", 25),
    ("Volsnap", 33),
    ("Volsnap", 36),
    ("volmgr", 161),
    ("Microsoft-Windows-Disk", 1),
];

/// Busca la clasificación de un evento por proveedor e id. `None` = evento no vigilado (la inmensa
/// mayoría): no es un error, es la exclusión por omisión (`docs/alert-rules.md` §3.3, FR-005).
pub fn clasificar(provider: &str, event_id: i64) -> Option<&'static ClasificacionEvento> {
    TABLA
        .iter()
        .find(|c| c.provider.eq_ignore_ascii_case(provider) && c.event_ids.contains(&event_id))
}

/// ¿`rule_key` es una de las reglas cuya fuente es el registro de eventos de Windows (spec 003)?
pub fn es_regla_de_eventos(rule_key: &str) -> bool {
    TABLA.iter().any(|c| c.rule_key == rule_key)
}

/// Ventana de resolución «sin repetición» de una regla, por `rule_key` (`docs/alert-rules.md` §2).
/// `None` si la regla no existe aquí o no se resuelve por tiempo (`device.removed_unexpected`
/// resuelve al reaparecer el disco).
pub fn ventana_resolucion_de(rule_key: &str) -> Option<Duration> {
    TABLA
        .iter()
        .find(|c| c.rule_key == rule_key)
        .and_then(|c| c.ventana_resolucion)
}

/// Todos los pares `(proveedor, event_id)` que activan una misma regla — p. ej.
/// `events.controller_reset` la disparan `disk` 11 y `stornvme`/`storahci` 129. Lo usa el conteo
/// por ventana de frecuencia, que debe sumar todas las variantes.
pub fn pares_de(rule_key: &str) -> Vec<(&'static str, i64)> {
    TABLA
        .iter()
        .filter(|c| c.rule_key == rule_key)
        .flat_map(|c| c.event_ids.iter().map(|id| (c.provider, *id)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valores esperados por `rule_key`, escritos a mano desde `docs/alert-rules.md` §2 (no
    /// derivados de `TABLA`): severidad base, ventana de resolución y contexto de deduplicación.
    /// FR-004: la implementación debe coincidir **exactamente** con la tabla normativa.
    struct Esperado {
        rule_key: &'static str,
        objetivo: Objetivo,
        dedup: ContextoDedup,
        ventana: Option<Duration>,
    }

    const ESPERADOS: &[Esperado] = &[
        Esperado {
            rule_key: "events.disk_error",
            objetivo: Objetivo::Dispositivo,
            dedup: ContextoDedup::ProviderEventId,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "events.filesystem_error",
            objetivo: Objetivo::Volumen,
            dedup: ContextoDedup::ProviderEventId,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "events.filesystem_repaired",
            objetivo: Objetivo::Volumen,
            dedup: ContextoDedup::VolumeGuid,
            ventana: Some(Duration::days(7)),
        },
        Esperado {
            rule_key: "events.filesystem_repair_storm",
            objetivo: Objetivo::Volumen,
            dedup: ContextoDedup::VolumeGuid,
            ventana: Some(Duration::days(7)),
        },
        Esperado {
            rule_key: "events.controller_reset",
            objetivo: Objetivo::Dispositivo,
            dedup: ContextoDedup::ProviderEventId,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "events.paging_error",
            objetivo: Objetivo::Dispositivo,
            dedup: ContextoDedup::DeviceId,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "events.io_retry",
            objetivo: Objetivo::Dispositivo,
            dedup: ContextoDedup::DeviceId,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "events.delayed_write",
            objetivo: Objetivo::Volumen,
            dedup: ContextoDedup::VolumeGuid,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "events.disk_predictive",
            objetivo: Objetivo::Dispositivo,
            dedup: ContextoDedup::Ninguno,
            ventana: Some(Duration::days(7)),
        },
        Esperado {
            rule_key: "events.storage_space_degraded",
            objetivo: Objetivo::SinObjeto,
            dedup: ContextoDedup::ProviderEventId,
            ventana: Some(Duration::hours(24)),
        },
        Esperado {
            rule_key: "device.removed_unexpected",
            objetivo: Objetivo::Dispositivo,
            dedup: ContextoDedup::Ninguno,
            ventana: None,
        },
        Esperado {
            rule_key: "inventory.duplicate_id",
            objetivo: Objetivo::SinObjeto,
            dedup: ContextoDedup::ParDeDiscos,
            ventana: Some(Duration::days(7)),
        },
    ];

    #[test]
    fn cada_regla_de_eventos_de_alert_rules_tiene_al_menos_una_fila() {
        for e in ESPERADOS {
            assert!(
                TABLA.iter().any(|c| c.rule_key == e.rule_key),
                "falta una fila para la regla {}",
                e.rule_key
            );
        }
    }

    #[test]
    fn ninguna_fila_lleva_un_rule_key_fuera_de_alert_rules() {
        for c in TABLA {
            assert!(
                ESPERADOS.iter().any(|e| e.rule_key == c.rule_key),
                "`TABLA` tiene un rule_key desconocido: {}",
                c.rule_key
            );
        }
    }

    #[test]
    fn cada_fila_es_fiel_a_alert_rules_md_2() {
        for c in TABLA {
            let e = ESPERADOS
                .iter()
                .find(|e| e.rule_key == c.rule_key)
                .unwrap_or_else(|| panic!("sin valores esperados para {}", c.rule_key));
            assert_eq!(c.objetivo, e.objetivo, "objetivo de {}", c.rule_key);
            assert_eq!(
                c.dedup_context, e.dedup,
                "contexto de dedup de {}",
                c.rule_key
            );
            assert_eq!(
                c.ventana_resolucion, e.ventana,
                "ventana de resolución de {}",
                c.rule_key
            );
        }
    }

    #[test]
    fn los_eventos_excluidos_de_seccion_3_3_no_tienen_fila() {
        for (provider, id) in EXCLUIDOS_EXPLICITOS {
            assert!(
                clasificar(provider, *id).is_none(),
                "{provider} {id} está en §3.3 (no genera alerta) pero tiene fila en TABLA"
            );
        }
    }

    #[test]
    fn clasificar_ignora_mayusculas_del_proveedor() {
        assert!(clasificar("NTFS", 55).is_some());
        assert!(clasificar("disk", 7).is_some());
        assert!(clasificar("disk", 99999).is_none());
    }

    #[test]
    fn paging_error_es_la_unica_regla_marcada_solo_no_extraibles() {
        for c in TABLA {
            assert_eq!(
                c.solo_no_extraibles,
                c.rule_key == "events.paging_error",
                "solo_no_extraibles mal puesto en {}",
                c.rule_key
            );
        }
    }
}
