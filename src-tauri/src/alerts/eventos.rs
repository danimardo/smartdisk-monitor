//! Puente del registro de eventos de Windows al motor de alertas (spec `003-puente-eventos-alertas`).
//!
//! La ingesta (`collectors::event_log`), la correlación evento→disco (`domain::correlacion`) y la
//! persistencia (`system_events`) ya existen. Este módulo añade la capa que **evalúa** esos eventos
//! contra las reglas de `docs/alert-rules.md` §2 cuya fuente es el registro de eventos, más
//! `device.removed_unexpected` e `inventory.duplicate_id`.
//!
//! - `evaluar_eventos`: clasifica cada evento nuevo del ciclo (`reglas_eventos::clasificar`) y lo
//!   aplica con `agrupacion::procesar`, estampando `triggering_event_id`.
//! - `resolver_grupos_de_eventos_vencidos`: barrido de resolución **por tiempo** (24 h / 7 días sin
//!   repetición, `alert-rules.md` §2) que corre cada ciclo, haya o no eventos nuevos.
//!
//! «Solo hacia delante» (`open-questions.md` J.51): quien llama pasa únicamente los eventos que
//! `repo_varios::insert_event_if_new` marcó como nuevos; los históricos ya ingeridos no se re-evalúan.

use rusqlite::Connection;
use time::{Duration, OffsetDateTime};

use crate::alerts::agrupacion::{self, EvaluacionAlerta, Transicion};
use crate::alerts::correlacion_rafaga::{self, EventoEnVentana, ResultadoCorrelacion, VENTANA};
use crate::alerts::reglas_eventos::{
    self, ClasificacionEvento, ContextoDedup, Evaluacion, Objetivo,
};
use crate::domain::tipos::{AlertSeverity, AlertStatus, EventLevel, MappingConfidence};
use crate::persistence::{repo_alertas, repo_varios};

/// Ventana de conteo de las reglas por frecuencia (`docs/alert-rules.md` §2: «≥ N en 1 h»).
const VENTANA_FRECUENCIA: Duration = Duration::hours(1);

const RFC3339: &time::format_description::well_known::Rfc3339 =
    &time::format_description::well_known::Rfc3339;

/// Vista mínima de un evento del sistema para decidir su regla, sin arrastrar la fila entera
/// (`docs/data-model.md` §2, spec 003). La construye `evaluar_eventos` a partir de `SystemEvent`
/// más el inventario (`es_extraible`).
#[derive(Debug, Clone)]
pub struct EventoParaRegla {
    /// `system_events.id`, para `alert_occurrences.triggering_event_id`.
    pub id: i64,
    pub provider: String,
    pub event_id: i64,
    pub occurred_at: OffsetDateTime,
    pub level: EventLevel,
    /// Disco correlacionado (`device:<id>`), o `None` si la atribución es `unknown`.
    pub device_id: Option<String>,
    /// Volumen correlacionado, o `None` (hoy `SystemEvent.volume_id` casi siempre es `None`:
    /// la correlación por volumen es trabajo futuro del colector, `domain::correlacion`).
    pub volume_id: Option<String>,
    pub mapping_confidence: MappingConfidence,
    /// El disco correlacionado es de medio extraíble / bus USB. Decide `events.paging_error` (que
    /// solo cuenta discos no extraíbles) y la severidad de `events.delayed_write` y
    /// `device.removed_unexpected`. `false` cuando no hay disco correlacionado.
    pub es_extraible: bool,
    /// Texto humano del evento, para las reglas que necesitan mirar dentro (`inventory.duplicate_id`
    /// busca los dos discos de la colisión).
    pub message: Option<String>,
}

/// ¿El disco es de medio extraíble? Se decide por el tipo de bus (`Device.bus_type`, tal cual lo
/// da `Get-PhysicalDisk` / `windows_storage`). USB y SD son extraíbles; SATA, NVMe, RAID, SAS no.
/// Un bus desconocido se trata como **no** extraíble: el lado prudente para `events.paging_error`
/// (que solo alerta de discos fijos) y para `device.removed_unexpected` (crítico salvo USB).
pub fn es_disco_extraible(bus_type: Option<&str>) -> bool {
    matches!(
        bus_type.map(|b| b.to_ascii_lowercase()).as_deref(),
        Some("usb") | Some("sd") | Some("mmc")
    )
}

fn es_proveedor_disk(provider: &str) -> bool {
    provider.eq_ignore_ascii_case("disk")
}

/// Evalúa los eventos **nuevos** de un ciclo contra las reglas de `docs/alert-rules.md` §2 cuya
/// fuente es el registro de eventos, y aplica cada resultado con `agrupacion::procesar`. Devuelve,
/// por regla tocada, `(deduplication_key, Transicion)` para que `commands::post_procesar_ciclo` las
/// notifique y emita como las demás.
///
/// Las extracciones imprevistas (`disk` 157) se procesan **primero**: así, dentro de una misma
/// ráfaga (mismo ciclo), los eventos derivados encuentran ya creado el grupo
/// `device.removed_unexpected` y se pliegan como ocurrencias suyas (`docs/alert-rules.md` §3.5).
pub fn evaluar_eventos(
    conn: &Connection,
    eventos_nuevos: &[EventoParaRegla],
) -> rusqlite::Result<Vec<(String, Transicion)>> {
    let mut transiciones = Vec::new();

    let (causas, resto): (Vec<&EventoParaRegla>, Vec<&EventoParaRegla>) =
        eventos_nuevos.iter().partition(|e| {
            correlacion_rafaga::es_extraccion_imprevista(es_proveedor_disk(&e.provider), e.event_id)
        });

    for e in causas.into_iter().chain(resto) {
        if let Some(t) = evaluar_un_evento(conn, e)? {
            transiciones.push(t);
        }
    }
    Ok(transiciones)
}

fn evaluar_un_evento(
    conn: &Connection,
    e: &EventoParaRegla,
) -> rusqlite::Result<Option<(String, Transicion)>> {
    let Some(clas) = reglas_eventos::clasificar(&e.provider, e.event_id) else {
        return Ok(None); // evento no vigilado: la inmensa mayoría (FR-005)
    };
    // Un evento de nivel Información con fila en la tabla no debería ocurrir; si ocurre, la tabla y
    // §3.3 mandan sobre el nivel: se registra y no se evalúa (`data-model.md` §5).
    if e.level == EventLevel::Information {
        tracing::warn!(
            provider = %e.provider, event_id = e.event_id,
            "evento de nivel Información con fila en reglas_eventos; no se evalúa"
        );
        return Ok(None);
    }
    // `events.paging_error` solo cuenta discos no extraíbles (`docs/alert-rules.md` §3.3).
    if clas.solo_no_extraibles && e.es_extraible {
        return Ok(None);
    }

    // Ventana de correlación de ráfaga (`docs/alert-rules.md` §3.5): solo entre eventos del mismo
    // disco.
    if let Some(device_id) = e.device_id.as_deref() {
        match correlacionar(conn, e, device_id)? {
            ResultadoCorrelacion::OcurrenciaDeExtraccion { .. } => {
                // Este derivado se pliega como ocurrencia de `device.removed_unexpected` del disco,
                // no crea su propio grupo (FR-009).
                return aplicar_como_ocurrencia_de_extraccion(conn, e, device_id).map(Some);
            }
            ResultadoCorrelacion::EsCausaConDerivadosPrevios => {
                // El `disk` 157 llegó tarde: los grupos de evento recientes de este disco son
                // síntomas suyos → se resuelven, y el 157 crea su `device.removed_unexpected`.
                reasignar_derivados_previos(conn, e, device_id)?;
            }
            ResultadoCorrelacion::SiguePropiaRegla => {}
        }
    }

    let severidad = match clas.evaluacion {
        Evaluacion::Inmediata { severidad } => severidad,
        Evaluacion::CondicionalNoExtraible {
            base,
            si_no_extraible,
        } => {
            if e.es_extraible {
                base
            } else {
                si_no_extraible
            }
        }
        Evaluacion::SegunBus { fijo, usb } => {
            if e.es_extraible {
                usb
            } else {
                fijo
            }
        }
        Evaluacion::PorFrecuencia {
            min_en_1h,
            severidad,
            escala_crit_en_1h,
        } => {
            let Some(device_id) = e.device_id.as_deref() else {
                // Sin disco no hay a qué acumular la frecuencia (contexto `device_id`).
                return Ok(None);
            };
            let cuenta = contar_en_ventana_de_frecuencia(conn, clas.rule_key, device_id, e)?;
            if cuenta < min_en_1h as i64 {
                return Ok(None); // aún por debajo del umbral
            }
            match escala_crit_en_1h {
                Some(umbral_crit) if cuenta >= umbral_crit as i64 => AlertSeverity::Critical,
                _ => severidad,
            }
        }
        Evaluacion::Especial => {
            // `inventory.duplicate_id`: una advertencia sin objeto, deduplicada por el par de
            // discos si se puede leer del mensaje, si no por `provider:event_id` (una sola vez).
            return aplicar_duplicate_id(conn, e).map(Some);
        }
    };

    let (target_device_id, target_volume_id, context) = resolver_objetivo(clas, e);
    Ok(Some(aplicar(
        conn,
        clas.rule_key,
        target_device_id,
        target_volume_id,
        context,
        Some(severidad),
        false,
        e,
    )?))
}

/// Aplica una `EvaluacionAlerta` de un evento y devuelve `(deduplication_key, Transicion)`.
#[allow(clippy::too_many_arguments)]
fn aplicar(
    conn: &Connection,
    rule_key: &str,
    target_device_id: Option<String>,
    target_volume_id: Option<String>,
    context: Option<String>,
    severity_si_activa: Option<AlertSeverity>,
    resuelto: bool,
    e: &EventoParaRegla,
) -> rusqlite::Result<(String, Transicion)> {
    aplicar_directo(
        conn,
        rule_key,
        target_device_id,
        target_volume_id,
        context,
        severity_si_activa,
        resuelto,
        &e.occurred_at.format(RFC3339).unwrap_or_default(),
        Some(e.id),
    )
}

#[allow(clippy::too_many_arguments)]
fn aplicar_directo(
    conn: &Connection,
    rule_key: &str,
    target_device_id: Option<String>,
    target_volume_id: Option<String>,
    context: Option<String>,
    severity_si_activa: Option<AlertSeverity>,
    resuelto: bool,
    ahora_utc: &str,
    triggering_event_id: Option<i64>,
) -> rusqlite::Result<(String, Transicion)> {
    let ev = EvaluacionAlerta {
        rule_key: rule_key.to_string(),
        target_device_id,
        target_volume_id,
        context,
        severity_si_activa,
        resuelto,
        value: None,
        occurred_at_utc: ahora_utc.to_string(),
        triggering_event_id,
    };
    let t = agrupacion::procesar(conn, &ev)?;
    let clave = agrupacion::deduplication_key(
        rule_key,
        ev.target_device_id.as_deref(),
        ev.target_volume_id.as_deref(),
        ev.context.as_deref(),
    );
    Ok((clave, t))
}

/// `device.removed_unexpected` disparado por la **desaparición de un disco del inventario** (no por
/// un `disk` 157): un disco fijo no desaparece en operación normal (`docs/open-questions.md` J.47).
/// Un disco USB sin `disk` 157 se trata como retirada esperada y **no** alerta. La deduplicación por
/// `device.removed_unexpected|device:<id>` hace que, si el `disk` 157 ya creó el grupo en el mismo
/// ciclo, esto solo registre una ocurrencia.
pub fn device_removed_unexpected_por_baja(
    conn: &Connection,
    device_id: &str,
    es_usb: bool,
    ahora_utc: &str,
) -> rusqlite::Result<Option<(String, Transicion)>> {
    if es_usb {
        return Ok(None);
    }
    aplicar_directo(
        conn,
        "device.removed_unexpected",
        Some(device_id.to_string()),
        None,
        None,
        Some(AlertSeverity::Critical),
        false,
        ahora_utc,
        None,
    )
    .map(Some)
}

/// Resuelve el `device.removed_unexpected` de un disco cuando **reaparece** en el inventario
/// (`docs/alert-rules.md` §2: «reaparece el mismo `fingerprint`»).
pub fn resolver_device_removed_por_reaparicion(
    conn: &Connection,
    device_id: &str,
    ahora_utc: &str,
) -> rusqlite::Result<Option<(String, Transicion)>> {
    let clave =
        agrupacion::deduplication_key("device.removed_unexpected", Some(device_id), None, None);
    let Some(g) = repo_alertas::get_group(conn, &clave)? else {
        return Ok(None);
    };
    if matches!(g.status, AlertStatus::Active | AlertStatus::Acknowledged) {
        repo_alertas::set_status(conn, &g.id, AlertStatus::Resolved, ahora_utc)?;
        return Ok(Some((clave, Transicion::Resuelta)));
    }
    Ok(None)
}

/// Registra `e` como ocurrencia del grupo `device.removed_unexpected` del disco (creado ya por el
/// `disk` 157 de la ráfaga). Mismo objetivo/contexto/severidad que produciría el 157.
fn aplicar_como_ocurrencia_de_extraccion(
    conn: &Connection,
    e: &EventoParaRegla,
    device_id: &str,
) -> rusqlite::Result<(String, Transicion)> {
    let severidad = if e.es_extraible {
        AlertSeverity::Warning
    } else {
        AlertSeverity::Critical
    };
    aplicar(
        conn,
        "device.removed_unexpected",
        Some(device_id.to_string()),
        None,
        None,
        Some(severidad),
        false,
        e,
    )
}

/// El `disk` 157 llegó después de que sus síntomas ya crearan grupos: los resuelve (son
/// consecuencia, no problemas propios). El propio 157 sigue su curso y crea el
/// `device.removed_unexpected` en `evaluar_un_evento`.
fn reasignar_derivados_previos(
    conn: &Connection,
    e: &EventoParaRegla,
    device_id: &str,
) -> rusqlite::Result<()> {
    let ahora = e.occurred_at.format(RFC3339).unwrap_or_default();
    for g in grupos_de_evento_recientes(conn, device_id, e.occurred_at)? {
        if g.rule_key == "device.removed_unexpected" {
            continue;
        }
        repo_alertas::set_status(conn, &g.id, AlertStatus::Resolved, &ahora)?;
    }
    Ok(())
}

/// `inventory.duplicate_id`: advertencia sin objeto. Contexto = los dos números de disco del
/// mensaje, ordenados, si se pueden leer; si no, `provider:event_id` (una sola vez).
fn aplicar_duplicate_id(
    conn: &Connection,
    e: &EventoParaRegla,
) -> rusqlite::Result<(String, Transicion)> {
    let context = par_de_discos_del_mensaje(e.message.as_deref())
        .unwrap_or_else(|| format!("{}:{}", e.provider, e.event_id));
    aplicar(
        conn,
        "inventory.duplicate_id",
        None,
        None,
        Some(context),
        Some(AlertSeverity::Warning),
        false,
        e,
    )
}

/// Extrae hasta dos números de disco de un texto («disco 1… disco 3», «Harddisk1… Harddisk3»),
/// ordenados, como `"1+3"`. `None` si no encuentra dos.
fn par_de_discos_del_mensaje(mensaje: Option<&str>) -> Option<String> {
    let m = mensaje?;
    let bajo = m.to_lowercase();
    let mut nums: Vec<i64> = Vec::new();
    for marcador in ["disco ", "disk ", "harddisk"] {
        let mut resto = bajo.as_str();
        while let Some(pos) = resto.find(marcador) {
            let tras = &resto[pos + marcador.len()..];
            let fin = tras
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(tras.len());
            if fin > 0 {
                if let Ok(n) = tras[..fin].parse::<i64>() {
                    nums.push(n);
                }
            }
            resto = &tras[fin..];
        }
    }
    nums.sort_unstable();
    nums.dedup();
    if nums.len() >= 2 {
        Some(format!("{}+{}", nums[0], nums[1]))
    } else {
        None
    }
}

fn correlacionar(
    conn: &Connection,
    e: &EventoParaRegla,
    device_id: &str,
) -> rusqlite::Result<ResultadoCorrelacion> {
    let desde = (e.occurred_at - VENTANA)
        .format(RFC3339)
        .unwrap_or_default();
    let ventana: Vec<EventoEnVentana> =
        repo_varios::eventos_de_dispositivo_desde(conn, device_id, &desde)?
            .into_iter()
            .filter(|(id, ..)| *id != e.id)
            .filter_map(|(id, provider, event_id, cuando)| {
                let cuando = OffsetDateTime::parse(&cuando, RFC3339).ok()?;
                Some(EventoEnVentana {
                    id,
                    provider_es_disk: es_proveedor_disk(&provider),
                    event_id,
                    antiguedad: (e.occurred_at - cuando).abs(),
                })
            })
            .collect();

    // ¿Hay síntomas ya agrupados de este disco dentro de la ventana? (grupos de evento que no sean
    // la propia extracción imprevista).
    let hay_derivados_previos = grupos_de_evento_recientes(conn, device_id, e.occurred_at)?
        .iter()
        .any(|g| g.rule_key != "device.removed_unexpected");

    Ok(correlacion_rafaga::correlacionar_rafaga(
        e.event_id,
        es_proveedor_disk(&e.provider),
        &ventana,
        hay_derivados_previos,
    ))
}

/// Grupos de reglas `events.*` de un disco cuya primera ocurrencia cae dentro de la ventana de
/// correlación (para la reasignación del `disk` 157 tardío).
fn grupos_de_evento_recientes(
    conn: &Connection,
    device_id: &str,
    ahora: OffsetDateTime,
) -> rusqlite::Result<Vec<crate::domain::tipos::AlertGroup>> {
    Ok(repo_alertas::list_groups(conn)?
        .into_iter()
        .filter(|g| {
            g.target_device_id.as_deref() == Some(device_id)
                && matches!(g.status, AlertStatus::Active | AlertStatus::Acknowledged)
                && reglas_eventos::es_regla_de_eventos(&g.rule_key)
                && OffsetDateTime::parse(&g.first_occurrence_at_utc, RFC3339)
                    .map(|t| (ahora - t).abs() <= VENTANA)
                    .unwrap_or(false)
        })
        .collect())
}

/// Cuenta los eventos de las reglas equivalentes a `rule_key` sobre `device_id` en la última hora
/// (por `occurred_at_utc`, no por hora de ingesta — `data-model.md` §5), incluido el evento actual.
fn contar_en_ventana_de_frecuencia(
    conn: &Connection,
    rule_key: &str,
    device_id: &str,
    e: &EventoParaRegla,
) -> rusqlite::Result<i64> {
    let pares = reglas_eventos::pares_de(rule_key);
    let desde = (e.occurred_at - VENTANA_FRECUENCIA)
        .format(RFC3339)
        .unwrap_or_default();
    let cuenta = repo_varios::eventos_de_dispositivo_desde(conn, device_id, &desde)?
        .into_iter()
        .filter(|(_, provider, event_id, _)| {
            pares
                .iter()
                .any(|(p, id)| p.eq_ignore_ascii_case(provider) && id == event_id)
        })
        .count();
    Ok(cuenta as i64)
}

/// Decide `(target_device_id, target_volume_id, context)` de un evento según su regla
/// (`docs/data-model.md` §3). Si la regla apuntaba a un disco o volumen y la correlación no lo pudo
/// determinar (`unknown`), la alerta se crea **sin objeto** y se deduplica por `provider:event_id`
/// (FR-004a, Q2 → A).
fn resolver_objetivo(
    clas: &ClasificacionEvento,
    e: &EventoParaRegla,
) -> (Option<String>, Option<String>, Option<String>) {
    let provider_event_id = || Some(format!("{}:{}", e.provider, e.event_id));

    let contexto_normal = || match clas.dedup_context {
        ContextoDedup::ProviderEventId => provider_event_id(),
        // `DeviceId`/`VolumeGuid` = un grupo por objetivo, sin discriminante extra.
        ContextoDedup::DeviceId | ContextoDedup::VolumeGuid | ContextoDedup::Ninguno => None,
        ContextoDedup::ParDeDiscos => None, // `Especial`, no llega aquí
    };

    match clas.objetivo {
        Objetivo::SinObjeto => (None, None, contexto_normal()),
        Objetivo::Dispositivo => match &e.device_id {
            Some(id) => (Some(id.clone()), None, contexto_normal()),
            None => (None, None, provider_event_id()),
        },
        Objetivo::Volumen => match &e.volume_id {
            Some(id) => (None, Some(id.clone()), contexto_normal()),
            None => (None, None, provider_event_id()),
        },
    }
}

/// Barrido de resolución **por tiempo** de los grupos de reglas de eventos (`docs/alert-rules.md`
/// §2: «24 h / 7 días sin repetición»). Corre en cada ciclo del recopilador, haya o no eventos
/// nuevos — igual que `collector.stalled` se evalúa en `post_procesar_ciclo` sin depender de un
/// ciclo de recopilación concreto. Un grupo `active`/`acknowledged` cuya última ocurrencia queda más
/// atrás que la ventana de su regla pasa a `resolved`. La recaída (evento nuevo tras resolverse) la
/// maneja `evaluar_eventos`, no este barrido.
pub fn resolver_grupos_de_eventos_vencidos(
    conn: &Connection,
    ahora_utc: &str,
) -> rusqlite::Result<Vec<(String, Transicion)>> {
    let Ok(ahora) = OffsetDateTime::parse(ahora_utc, RFC3339) else {
        return Ok(vec![]);
    };
    let mut transiciones = Vec::new();
    for g in repo_alertas::list_groups(conn)? {
        if !matches!(g.status, AlertStatus::Active | AlertStatus::Acknowledged) {
            continue;
        }
        if !reglas_eventos::es_regla_de_eventos(&g.rule_key) {
            continue;
        }
        let Some(ventana) = reglas_eventos::ventana_resolucion_de(&g.rule_key) else {
            // `device.removed_unexpected`: no se resuelve por tiempo.
            continue;
        };
        let Ok(ultima) = OffsetDateTime::parse(&g.last_occurrence_at_utc, RFC3339) else {
            continue;
        };
        if ahora - ultima > ventana {
            repo_alertas::set_status(conn, &g.id, AlertStatus::Resolved, ahora_utc)?;
            transiciones.push((g.deduplication_key.clone(), Transicion::Resuelta));
        }
    }
    Ok(transiciones)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::agrupacion::{self, EvaluacionAlerta};
    use crate::domain::tipos::AlertSeverity;
    use crate::persistence::db;

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("alerts_eventos");
        let conn = db::open(&dir).unwrap().0;
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella-d1', 'fingerprint', 'M', 'nvme', 1, '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    /// Persiste un `system_events` (como haría `refresh_events`) y devuelve su vista para el motor.
    /// El `id` real es imprescindible: `alert_occurrences.triggering_event_id` tiene FK.
    fn evento(conn: &Connection, provider: &str, event_id: i64, cuando: &str) -> EventoParaRegla {
        evento_de(conn, provider, event_id, cuando, None, false)
    }

    fn evento_de(
        conn: &Connection,
        provider: &str,
        event_id: i64,
        cuando: &str,
        device_id: Option<&str>,
        es_extraible: bool,
    ) -> EventoParaRegla {
        let id: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(id), 0) + 1 FROM system_events",
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            "INSERT INTO system_events (id, channel, record_id, occurred_at_utc, provider, event_id, level, device_id, mapping_confidence, dedup_hash)
             VALUES (?1, 'System', ?1, ?2, ?3, ?4, 'error', ?5, 'unknown', ?6)",
            rusqlite::params![id, cuando, provider, event_id, device_id, format!("h{id}")],
        )
        .unwrap();
        EventoParaRegla {
            id,
            provider: provider.to_string(),
            event_id,
            occurred_at: OffsetDateTime::parse(cuando, RFC3339).unwrap(),
            level: EventLevel::Error,
            device_id: device_id.map(String::from),
            volume_id: None,
            mapping_confidence: MappingConfidence::Unknown,
            es_extraible,
            message: None,
        }
    }

    #[test]
    fn una_regla_inmediata_activa_un_grupo_con_la_severidad_de_la_tabla() {
        let conn = conn_de_prueba();
        let e = evento(&conn, "disk", 7, "2026-09-04T10:00:00Z");
        let t = evaluar_eventos(&conn, &[e]).unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].1, Transicion::CreadaActiva);
        let g = repo_alertas::get_group(&conn, "events.disk_error|sin_objetivo|disk:7")
            .unwrap()
            .expect("no se creó el grupo");
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.severity, AlertSeverity::Critical);
    }

    #[test]
    fn un_evento_no_vigilado_no_crea_nada() {
        let conn = conn_de_prueba();
        // `Microsoft-Windows-Ntfs` 98 está en §3.3: no genera alerta.
        let mut e = evento(&conn, "Microsoft-Windows-Ntfs", 98, "2026-09-04T10:00:00Z");
        e.level = EventLevel::Information;
        assert!(evaluar_eventos(&conn, &[e]).unwrap().is_empty());
        assert!(repo_alertas::list_groups(&conn).unwrap().is_empty());
    }

    #[test]
    fn atribucion_desconocida_crea_alerta_sin_objeto_deduplicada_por_provider_event_id() {
        let conn = conn_de_prueba();
        // `Ntfs` 131 apunta a un volumen; sin `volume_id` correlacionado → sin objeto (FR-004a).
        let e = evento(&conn, "Ntfs", 131, "2026-09-04T10:00:00Z");
        evaluar_eventos(&conn, &[e]).unwrap();
        let g = repo_alertas::get_group(&conn, "events.filesystem_error|sin_objetivo|Ntfs:131")
            .unwrap()
            .expect("no se creó el grupo sin objeto");
        assert_eq!(g.target_device_id, None);
        assert_eq!(g.target_volume_id, None);
        assert_eq!(g.severity, AlertSeverity::Critical);
    }

    #[test]
    fn delayed_write_es_critico_en_volumen_fijo_y_advertencia_en_extraible() {
        let conn = conn_de_prueba();
        let mut fijo = evento(&conn, "Ntfs", 50, "2026-09-04T10:00:00Z");
        fijo.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[fijo]).unwrap();
        let g = repo_alertas::get_group(&conn, "events.delayed_write|sin_objetivo|Ntfs:50")
            .unwrap()
            .unwrap();
        assert_eq!(g.severity, AlertSeverity::Critical, "volumen fijo");

        let conn2 = conn_de_prueba();
        let mut ext = evento(&conn2, "Ntfs", 50, "2026-09-04T10:00:00Z");
        ext.level = EventLevel::Warning;
        ext.es_extraible = true;
        evaluar_eventos(&conn2, &[ext]).unwrap();
        let g2 = repo_alertas::get_group(&conn2, "events.delayed_write|sin_objetivo|Ntfs:50")
            .unwrap()
            .unwrap();
        assert_eq!(g2.severity, AlertSeverity::Warning, "volumen extraíble");
    }

    #[test]
    fn device_removed_unexpected_por_disk_157_es_critico_en_disco_fijo_y_advertencia_en_usb() {
        let conn = conn_de_prueba();
        let mut fijo = evento(&conn, "disk", 157, "2026-09-04T10:00:00Z");
        fijo.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[fijo]).unwrap();
        assert_eq!(
            repo_alertas::get_group(&conn, "device.removed_unexpected|sin_objetivo|disk:157")
                .unwrap()
                .unwrap()
                .severity,
            AlertSeverity::Critical
        );

        let conn2 = conn_de_prueba();
        let mut usb = evento(&conn2, "disk", 157, "2026-09-04T10:00:00Z");
        usb.level = EventLevel::Warning;
        usb.es_extraible = true;
        evaluar_eventos(&conn2, &[usb]).unwrap();
        assert_eq!(
            repo_alertas::get_group(&conn2, "device.removed_unexpected|sin_objetivo|disk:157")
                .unwrap()
                .unwrap()
                .severity,
            AlertSeverity::Warning
        );
    }

    #[test]
    fn paging_error_sobre_un_medio_extraible_no_cuenta() {
        let conn = conn_de_prueba();
        let mut e = evento(&conn, "disk", 51, "2026-09-04T10:00:00Z");
        e.level = EventLevel::Warning;
        e.es_extraible = true;
        assert!(evaluar_eventos(&conn, &[e]).unwrap().is_empty());
    }

    #[test]
    fn una_regla_por_frecuencia_todavia_no_produce_grupo_fase_2() {
        // `events.paging_error` sobre un disco fijo: se reconoce pero no crea grupo hasta US3.
        let conn = conn_de_prueba();
        let mut e = evento(&conn, "disk", 51, "2026-09-04T10:00:00Z");
        e.level = EventLevel::Warning;
        assert!(evaluar_eventos(&conn, &[e]).unwrap().is_empty());
    }

    #[test]
    fn solo_hacia_delante_ningun_evento_nuevo_no_crea_ni_toca_nada() {
        // FR-018 / SC-007: `evaluar_eventos` solo mira la lista que se le pasa. Los eventos
        // históricos ya en `system_events` no se re-evalúan porque nunca llegan aquí.
        let conn = conn_de_prueba();
        conn.execute(
            "INSERT INTO system_events (id, channel, record_id, occurred_at_utc, provider, event_id, level, mapping_confidence, dedup_hash)
             VALUES (1, 'System', 1, '2020-01-01T00:00:00Z', 'disk', 7, 'error', 'unknown', 'h1'),
                    (2, 'System', 2, '2020-01-01T00:00:01Z', 'Ntfs', 131, 'error', 'unknown', 'h2')",
            [],
        )
        .unwrap();
        let t = evaluar_eventos(&conn, &[]).unwrap();
        assert!(t.is_empty());
        assert!(repo_alertas::list_groups(&conn).unwrap().is_empty());
    }

    /// Crea un grupo `active` de una regla de evento con la última ocurrencia en `cuando`.
    fn grupo_de_evento(conn: &Connection, rule_key: &str, cuando: &str) {
        let ev = EvaluacionAlerta {
            rule_key: rule_key.to_string(),
            target_device_id: None,
            target_volume_id: None,
            context: Some("disk:7".to_string()),
            severity_si_activa: Some(AlertSeverity::Critical),
            resuelto: false,
            value: None,
            occurred_at_utc: cuando.to_string(),
            triggering_event_id: None,
        };
        agrupacion::procesar(conn, &ev).unwrap();
    }

    #[test]
    fn un_grupo_de_evento_sin_repetirse_pasada_su_ventana_se_resuelve() {
        let conn = conn_de_prueba();
        grupo_de_evento(&conn, "events.disk_error", "2026-09-04T10:00:00Z");

        // 23 h después: dentro de la ventana de 24 h → sigue activo.
        let t = resolver_grupos_de_eventos_vencidos(&conn, "2026-09-05T09:00:00Z").unwrap();
        assert!(t.is_empty());

        // 25 h después: fuera de la ventana → resuelto.
        let t = resolver_grupos_de_eventos_vencidos(&conn, "2026-09-05T11:00:00Z").unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].1, Transicion::Resuelta);
        let g = repo_alertas::get_group(&conn, "events.disk_error|sin_objetivo|disk:7")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Resolved);
    }

    #[test]
    fn filesystem_repaired_usa_la_ventana_larga_de_siete_dias() {
        let conn = conn_de_prueba();
        grupo_de_evento(&conn, "events.filesystem_repaired", "2026-09-04T10:00:00Z");
        // 5 días: sigue activo.
        assert!(
            resolver_grupos_de_eventos_vencidos(&conn, "2026-09-09T10:00:00Z")
                .unwrap()
                .is_empty()
        );
        // 8 días: resuelto.
        assert_eq!(
            resolver_grupos_de_eventos_vencidos(&conn, "2026-09-12T11:00:00Z")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn device_removed_unexpected_no_se_resuelve_por_tiempo() {
        let conn = conn_de_prueba();
        grupo_de_evento(&conn, "device.removed_unexpected", "2026-09-04T10:00:00Z");
        // Un mes después: sigue activo (resuelve al reaparecer el disco, no por tiempo).
        assert!(
            resolver_grupos_de_eventos_vencidos(&conn, "2026-10-04T10:00:00Z")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn el_barrido_no_toca_grupos_de_smart_ni_capacidad() {
        let conn = conn_de_prueba();
        grupo_de_evento(&conn, "smart.health.failed", "2026-09-04T10:00:00Z");
        assert!(
            resolver_grupos_de_eventos_vencidos(&conn, "2026-12-01T10:00:00Z")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn tras_resolverse_por_tiempo_un_evento_nuevo_reabre_el_grupo_en_un_ciclo_nuevo() {
        let conn = conn_de_prueba();
        grupo_de_evento(&conn, "events.disk_error", "2026-09-04T10:00:00Z");
        resolver_grupos_de_eventos_vencidos(&conn, "2026-09-05T11:00:00Z").unwrap();

        // El evento vuelve: `agrupacion::procesar` lo reabre (cycle + 1).
        grupo_de_evento(&conn, "events.disk_error", "2026-09-06T10:00:00Z");
        let g = repo_alertas::get_group(&conn, "events.disk_error|sin_objetivo|disk:7")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.cycle, 2);
        assert!(g.occurrence_count >= 2, "el contador histórico se conserva");
    }

    #[test]
    fn el_bus_usb_es_extraible_y_los_fijos_no() {
        assert!(es_disco_extraible(Some("USB")));
        assert!(es_disco_extraible(Some("usb")));
        assert!(es_disco_extraible(Some("SD")));
        assert!(!es_disco_extraible(Some("SATA")));
        assert!(!es_disco_extraible(Some("NVMe")));
        assert!(!es_disco_extraible(Some("RAID")));
    }

    #[test]
    fn un_bus_desconocido_o_ausente_se_trata_como_no_extraible() {
        assert!(!es_disco_extraible(None));
        assert!(!es_disco_extraible(Some("")));
        assert!(!es_disco_extraible(Some("algo-raro")));
    }

    // ---- US1: ventana de correlación de ráfaga ----

    #[test]
    fn una_rafaga_con_disk_157_produce_un_solo_grupo_con_los_derivados_como_ocurrencias() {
        let conn = conn_de_prueba();
        let e157 = evento_de(
            &conn,
            "disk",
            157,
            "2026-09-04T10:00:00Z",
            Some("d1"),
            false,
        );
        let e51 = evento_de(&conn, "disk", 51, "2026-09-04T10:00:03Z", Some("d1"), false);
        let e50 = evento_de(&conn, "Ntfs", 50, "2026-09-04T10:00:05Z", Some("d1"), false);

        let t = evaluar_eventos(&conn, &[e51, e50, e157]).unwrap();
        // Solo hay un grupo (device.removed_unexpected); los otros no crearon grupo propio.
        let grupos = repo_alertas::list_groups(&conn).unwrap();
        assert_eq!(grupos.len(), 1, "una ráfaga = un grupo");
        assert_eq!(grupos[0].rule_key, "device.removed_unexpected");
        assert!(
            grupos[0].occurrence_count >= 2,
            "los derivados figuran en su cronología: {}",
            grupos[0].occurrence_count
        );
        assert!(t
            .iter()
            .all(|(k, _)| k.starts_with("device.removed_unexpected")));
    }

    #[test]
    fn una_rafaga_sin_disk_157_deja_que_cada_regla_cree_su_grupo() {
        let conn = conn_de_prueba();
        let e55 = evento_de(&conn, "Ntfs", 55, "2026-09-04T10:00:00Z", Some("d1"), false);
        let e7 = evento_de(&conn, "disk", 7, "2026-09-04T10:00:02Z", Some("d1"), false);
        evaluar_eventos(&conn, &[e55, e7]).unwrap();
        let claves: Vec<String> = repo_alertas::list_groups(&conn)
            .unwrap()
            .into_iter()
            .map(|g| g.rule_key)
            .collect();
        assert!(claves.contains(&"events.filesystem_error".to_string()));
        assert!(claves.contains(&"events.disk_error".to_string()));
        assert_eq!(
            claves.len(),
            2,
            "dos problemas distintos, dos grupos (FR-009a)"
        );
    }

    #[test]
    fn el_disk_157_que_llega_en_un_ciclo_posterior_absorbe_los_grupos_derivados() {
        let conn = conn_de_prueba();
        // Ciclo 1: llega el derivado y crea su grupo.
        let e7 = evento_de(&conn, "disk", 7, "2026-09-04T10:00:00Z", Some("d1"), false);
        evaluar_eventos(&conn, &[e7]).unwrap();
        assert_eq!(
            repo_alertas::list_groups(&conn).unwrap()[0].status,
            AlertStatus::Active
        );
        // Ciclo 2: llega el disk 157 dentro de los 60 s → el grupo derivado se resuelve.
        let e157 = evento_de(
            &conn,
            "disk",
            157,
            "2026-09-04T10:00:20Z",
            Some("d1"),
            false,
        );
        evaluar_eventos(&conn, &[e157]).unwrap();
        let d = repo_alertas::get_group(&conn, "events.disk_error|device:d1|disk:7")
            .unwrap()
            .unwrap();
        assert_eq!(d.status, AlertStatus::Resolved, "el síntoma se resuelve");
        assert!(
            repo_alertas::get_group(&conn, "device.removed_unexpected|device:d1")
                .unwrap()
                .is_some_and(|g| g.status == AlertStatus::Active)
        );
    }

    // ---- US1: device.removed_unexpected vía correlación de disco ----

    #[test]
    fn device_removed_unexpected_por_disk_157_correlacionado_apunta_al_disco() {
        let conn = conn_de_prueba();
        let e = evento_de(
            &conn,
            "disk",
            157,
            "2026-09-04T10:00:00Z",
            Some("d1"),
            false,
        );
        evaluar_eventos(&conn, &[e]).unwrap();
        let g = repo_alertas::get_group(&conn, "device.removed_unexpected|device:d1")
            .unwrap()
            .expect("no se creó el grupo con objetivo de disco");
        assert_eq!(g.severity, AlertSeverity::Critical);
        assert_eq!(g.target_device_id.as_deref(), Some("d1"));
    }

    #[test]
    fn device_removed_unexpected_por_baja_de_inventario_de_disco_fijo_es_critico() {
        let conn = conn_de_prueba();
        let t = device_removed_unexpected_por_baja(&conn, "d1", false, "2026-09-04T10:00:00Z")
            .unwrap()
            .expect("un disco fijo que desaparece debe alertar");
        assert_eq!(t.1, Transicion::CreadaActiva);
        let g = repo_alertas::get_group(&conn, "device.removed_unexpected|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.severity, AlertSeverity::Critical);
    }

    #[test]
    fn una_baja_de_disco_usb_sin_disk_157_no_alerta() {
        let conn = conn_de_prueba();
        assert!(
            device_removed_unexpected_por_baja(&conn, "d1", true, "2026-09-04T10:00:00Z")
                .unwrap()
                .is_none()
        );
        assert!(repo_alertas::list_groups(&conn).unwrap().is_empty());
    }

    #[test]
    fn device_removed_unexpected_se_resuelve_al_reaparecer_el_disco_y_recae_si_vuelve_a_irse() {
        let conn = conn_de_prueba();
        device_removed_unexpected_por_baja(&conn, "d1", false, "2026-09-04T10:00:00Z").unwrap();
        let r = resolver_device_removed_por_reaparicion(&conn, "d1", "2026-09-04T10:05:00Z")
            .unwrap()
            .unwrap();
        assert_eq!(r.1, Transicion::Resuelta);
        assert_eq!(
            repo_alertas::get_group(&conn, "device.removed_unexpected|device:d1")
                .unwrap()
                .unwrap()
                .status,
            AlertStatus::Resolved
        );
        // Recae.
        device_removed_unexpected_por_baja(&conn, "d1", false, "2026-09-04T11:00:00Z").unwrap();
        let g = repo_alertas::get_group(&conn, "device.removed_unexpected|device:d1")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.cycle, 2);
    }

    #[test]
    fn reaparecer_un_disco_sin_alerta_previa_no_hace_nada() {
        let conn = conn_de_prueba();
        assert!(
            resolver_device_removed_por_reaparicion(&conn, "d1", "2026-09-04T10:00:00Z")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn una_baja_repetida_del_mismo_disco_no_crea_un_segundo_grupo() {
        let conn = conn_de_prueba();
        device_removed_unexpected_por_baja(&conn, "d1", false, "2026-09-04T10:00:00Z").unwrap();
        device_removed_unexpected_por_baja(&conn, "d1", false, "2026-09-04T10:01:00Z").unwrap();
        assert_eq!(repo_alertas::list_groups(&conn).unwrap().len(), 1);
    }

    // ---- US3: reglas por frecuencia ----

    #[test]
    fn paging_error_no_alerta_bajo_el_umbral_y_si_al_alcanzarlo() {
        let conn = conn_de_prueba();
        for i in 0..9 {
            let e = evento_de(
                &conn,
                "disk",
                51,
                &format!("2026-09-04T10:{i:02}:00Z"),
                Some("d1"),
                false,
            );
            let mut e = e;
            e.level = EventLevel::Warning;
            evaluar_eventos(&conn, &[e]).unwrap();
        }
        assert!(
            repo_alertas::list_groups(&conn).unwrap().is_empty(),
            "9 en 1 h: aún sin alerta"
        );
        let mut e = evento_de(&conn, "disk", 51, "2026-09-04T10:09:30Z", Some("d1"), false);
        e.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[e]).unwrap();
        let g = repo_alertas::get_group(&conn, "events.paging_error|device:d1")
            .unwrap()
            .expect("el décimo debería activar");
        assert_eq!(
            g.severity,
            AlertSeverity::Warning,
            "paging_error nunca es crítico"
        );
    }

    #[test]
    fn io_retry_activa_al_quinto_en_una_hora() {
        let conn = conn_de_prueba();
        for i in 0..4 {
            let mut e = evento_de(
                &conn,
                "disk",
                153,
                &format!("2026-09-04T10:{i:02}:00Z"),
                Some("d1"),
                false,
            );
            e.level = EventLevel::Warning;
            evaluar_eventos(&conn, &[e]).unwrap();
        }
        assert!(repo_alertas::list_groups(&conn).unwrap().is_empty());
        let mut e = evento_de(
            &conn,
            "disk",
            153,
            "2026-09-04T10:05:00Z",
            Some("d1"),
            false,
        );
        e.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[e]).unwrap();
        assert!(repo_alertas::get_group(&conn, "events.io_retry|device:d1")
            .unwrap()
            .is_some());
    }

    #[test]
    fn controller_reset_avisa_al_primero_y_escala_a_critico_al_tercero_en_una_hora() {
        let conn = conn_de_prueba();
        let mut e1 = evento_de(&conn, "disk", 11, "2026-09-04T10:00:00Z", Some("d1"), false);
        e1.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[e1]).unwrap();
        assert_eq!(
            repo_alertas::get_group(&conn, "events.controller_reset|device:d1|disk:11")
                .unwrap()
                .unwrap()
                .severity,
            AlertSeverity::Warning
        );
        for i in 1..3 {
            let mut e = evento_de(
                &conn,
                "disk",
                11,
                &format!("2026-09-04T10:{:02}:00Z", i * 10),
                Some("d1"),
                false,
            );
            e.level = EventLevel::Warning;
            evaluar_eventos(&conn, &[e]).unwrap();
        }
        assert_eq!(
            repo_alertas::get_group(&conn, "events.controller_reset|device:d1|disk:11")
                .unwrap()
                .unwrap()
                .severity,
            AlertSeverity::Critical,
            "≥ 3 en 1 h → crítico"
        );
    }

    #[test]
    fn una_regla_por_frecuencia_no_cuenta_eventos_de_hace_mas_de_una_hora() {
        let conn = conn_de_prueba();
        // 8 hace más de una hora + 2 ahora: solo 2 en ventana, por debajo del 10 de paging_error.
        for i in 0..8 {
            let mut e = evento_de(
                &conn,
                "disk",
                51,
                &format!("2026-09-04T08:{i:02}:00Z"),
                Some("d1"),
                false,
            );
            e.level = EventLevel::Warning;
            evaluar_eventos(&conn, &[e]).unwrap();
        }
        for cuando in ["2026-09-04T10:00:00Z", "2026-09-04T10:01:00Z"] {
            let mut e = evento_de(&conn, "disk", 51, cuando, Some("d1"), false);
            e.level = EventLevel::Warning;
            evaluar_eventos(&conn, &[e]).unwrap();
        }
        assert!(repo_alertas::list_groups(&conn).unwrap().is_empty());
    }

    // ---- US4: advertencias leves (activación + histéresis de 7 días) ----

    #[test]
    fn filesystem_repaired_y_disk_predictive_son_advertencias_con_ventana_de_siete_dias() {
        let conn = conn_de_prueba();
        let mut r = evento_de(
            &conn,
            "Ntfs",
            130,
            "2026-09-04T10:00:00Z",
            Some("d1"),
            false,
        );
        r.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[r]).unwrap();
        let g = repo_alertas::get_group(&conn, "events.filesystem_repaired|sin_objetivo|Ntfs:130")
            .unwrap()
            .unwrap();
        assert_eq!(g.severity, AlertSeverity::Warning);

        // 8 días sin repetición → resuelto (barrido).
        resolver_grupos_de_eventos_vencidos(&conn, "2026-09-12T11:00:00Z").unwrap();
        assert_eq!(
            repo_alertas::get_group(&conn, "events.filesystem_repaired|sin_objetivo|Ntfs:130")
                .unwrap()
                .unwrap()
                .status,
            AlertStatus::Resolved
        );
    }

    #[test]
    fn filesystem_repair_storm_es_critico() {
        let conn = conn_de_prueba();
        let mut e = evento_de(
            &conn,
            "Ntfs",
            132,
            "2026-09-04T10:00:00Z",
            Some("d1"),
            false,
        );
        e.level = EventLevel::Warning;
        evaluar_eventos(&conn, &[e]).unwrap();
        assert_eq!(
            repo_alertas::get_group(
                &conn,
                "events.filesystem_repair_storm|sin_objetivo|Ntfs:132"
            )
            .unwrap()
            .unwrap()
            .severity,
            AlertSeverity::Critical
        );
    }

    // ---- US5: inventory.duplicate_id ----

    #[test]
    fn duplicate_id_crea_una_advertencia_una_sola_vez_por_par() {
        let conn = conn_de_prueba();
        let mut e = evento(&conn, "disk", 158, "2026-09-04T10:00:00Z");
        e.level = EventLevel::Warning;
        e.message = Some("Los discos disco 1 y disco 3 tienen los mismos identificadores".into());
        evaluar_eventos(&conn, &[e]).unwrap();
        let g = repo_alertas::get_group(&conn, "inventory.duplicate_id|sin_objetivo|1+3")
            .unwrap()
            .expect("no se creó el grupo por par de discos");
        assert_eq!(g.severity, AlertSeverity::Warning);
        assert_eq!(g.occurrence_count, 1);

        // Un segundo evento del mismo par: ocurrencia, no grupo nuevo.
        let mut e2 = evento(&conn, "disk", 158, "2026-09-04T11:00:00Z");
        e2.level = EventLevel::Warning;
        e2.message = Some("disco 3 y disco 1 duplicados".into());
        evaluar_eventos(&conn, &[e2]).unwrap();
        assert_eq!(repo_alertas::list_groups(&conn).unwrap().len(), 1);
        assert_eq!(
            repo_alertas::get_group(&conn, "inventory.duplicate_id|sin_objetivo|1+3")
                .unwrap()
                .unwrap()
                .occurrence_count,
            2
        );
    }

    #[test]
    fn duplicate_id_sin_par_legible_cae_a_un_grupo_unico() {
        let conn = conn_de_prueba();
        let mut e = evento(&conn, "disk", 158, "2026-09-04T10:00:00Z");
        e.level = EventLevel::Warning;
        e.message = None;
        evaluar_eventos(&conn, &[e]).unwrap();
        assert!(
            repo_alertas::get_group(&conn, "inventory.duplicate_id|sin_objetivo|disk:158")
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn cero_falsos_positivos_sobre_el_ruido_de_fondo_del_registro_de_referencia() {
        // SC-001: un subconjunto representativo del ruido de `docs/alert-rules.md` §3.
        let conn = conn_de_prueba();
        let mut lote = Vec::new();
        // `disk` 51 ×9 sobre un disco fijo: bajo el umbral de 10, no debe alertar.
        for i in 0..9 {
            let mut fijo = evento_de(
                &conn,
                "disk",
                51,
                &format!("2026-09-04T10:{i:02}:00Z"),
                Some("d1"),
                false,
            );
            fijo.level = EventLevel::Warning;
            lote.push(fijo);
        }
        // Eventos de §3.3 que nunca generan alerta.
        for (provider, id, lvl) in [
            ("Microsoft-Windows-Ntfs", 98, EventLevel::Information),
            ("Microsoft-Windows-Ntfs", 98, EventLevel::Information),
            ("Microsoft-Windows-NvmeDisk", 501, EventLevel::Warning),
            ("Volsnap", 25, EventLevel::Information),
            ("volmgr", 161, EventLevel::Error),
            ("Microsoft-Windows-Disk", 1, EventLevel::Information),
        ] {
            let mut e = evento_de(
                &conn,
                provider,
                id,
                "2026-09-04T10:30:00Z",
                Some("d1"),
                false,
            );
            e.level = lvl;
            lote.push(e);
        }
        // `disk` 11 ×2 (bajo el umbral de escalada de 3).
        for i in 0..2 {
            let mut e = evento_de(
                &conn,
                "disk",
                11,
                &format!("2026-09-04T11:{i:02}:00Z"),
                Some("d1"),
                false,
            );
            e.level = EventLevel::Warning;
            lote.push(e);
        }

        evaluar_eventos(&conn, &lote).unwrap();
        // `disk` 11 ×2 sí crea el grupo de `events.controller_reset` (advertencia al primero); lo
        // que NO debe haber es ningún crítico ni ninguna regla por frecuencia disparada.
        let grupos = repo_alertas::list_groups(&conn).unwrap();
        assert!(
            grupos.iter().all(|g| g.severity != AlertSeverity::Critical),
            "cero críticos falsos sobre el ruido de fondo (SC-001)"
        );
        assert!(
            !grupos.iter().any(|g| g.rule_key == "events.paging_error"),
            "`disk` 51 bajo umbral no debe alertar"
        );
    }

    // ---- US2: reglas críticas — deduplicación y ciclo de recaída ----

    #[test]
    fn disk_error_deduplica_y_recae_conservando_el_contador() {
        let conn = conn_de_prueba();
        for cuando in [
            "2026-09-04T10:00:00Z",
            "2026-09-04T10:05:00Z",
            "2026-09-04T10:10:00Z",
        ] {
            let e = evento_de(&conn, "disk", 7, cuando, Some("d1"), false);
            evaluar_eventos(&conn, &[e]).unwrap();
        }
        let g = repo_alertas::get_group(&conn, "events.disk_error|device:d1|disk:7")
            .unwrap()
            .unwrap();
        assert_eq!(g.occurrence_count, 3, "3 evaluaciones, 1 grupo");

        resolver_grupos_de_eventos_vencidos(&conn, "2026-09-05T11:00:00Z").unwrap();
        let e = evento_de(&conn, "disk", 7, "2026-09-06T10:00:00Z", Some("d1"), false);
        evaluar_eventos(&conn, &[e]).unwrap();
        let g = repo_alertas::get_group(&conn, "events.disk_error|device:d1|disk:7")
            .unwrap()
            .unwrap();
        assert_eq!(g.status, AlertStatus::Active);
        assert_eq!(g.cycle, 2);
        assert!(g.occurrence_count >= 4);
    }
}
