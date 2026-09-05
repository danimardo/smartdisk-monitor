//! Comandos Tauri: la superficie completa que la interfaz puede invocar.
//!
//! Cada comando es una función fina que valida sus argumentos y delega en `domain/`. El grueso del
//! esqueleto devuelve `not_implemented` a propósito: es preferible que la aplicación diga en voz
//! alta lo que falta a que devuelva datos inventados que alguien confunda con reales.
//!
//! El contrato normativo está en `docs/ui-contract.md`. La lista de comandos que se registran en
//! `lib.rs` es la lista cerrada: la interfaz no puede invocar nada que no esté aquí.

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use ts_rs::TS;

use crate::domain::identidad;
use crate::domain::tipos::{
    AggregateResolution, AlertGroup, AlertSeverity, AlertStatus, Device, DeviceType, HealthState,
    IdentityConfidence, MappingConfidence, MetricSource, Resolution, TestRun, TestStatus, TestType,
    UnknownReason, Volume,
};
use crate::error::{AppError, AppResult};
use crate::persistence::db::AppState;
use crate::persistence::{
    repo_agregados, repo_alertas, repo_inventario, repo_metricas, repo_varios,
};
use crate::platform::accent::{self, WindowsAccent};
use crate::tests;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct AppearanceSettings {
    pub theme: String,
    pub language: Option<String>,
    /// BCP-47 de Windows. La interfaz usa este, no `navigator.language`: el formato de números
    /// debe seguir al idioma de la aplicación (`open-questions.md` A.6).
    pub system_locale: String,
    pub use_system_accent: bool,
}

/// Espejo de `VolumeSummary` en `src/lib/design/types.ts`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct VolumeSummary {
    pub id: String,
    pub label: String,
    pub drive_letters: Vec<String>,
    pub capacity_bytes: Option<i64>,
    pub free_bytes: Option<i64>,
    /// `chkdsk /scan` solo existe en NTFS (T083): la decisión vive en el backend, no se le pide al
    /// frontend que repita el criterio comparando `filesystem` a mano.
    pub chkdsk_available: bool,
    pub mapping_confidence: MappingConfidence,
}

/// Espejo de `Provenance` en `src/lib/design/types.ts`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct Provenance {
    pub source: MetricSource,
    /// `MetricQuality` en Rust, pero como texto aquí: nada la usa todavía y tipar ahora una unión
    /// que ninguna prueba ejercita es más ceremonia que garantía. Se tipa igual que `source` en
    /// cuanto `domain::salud` (T036) produzca el primer valor real.
    pub quality: String,
    pub read_at: Option<String>,
}

/// Espejo de `DiskSummary` en `src/lib/design/types.ts`. `enrich_with_smart_data` lo completa con
/// las últimas muestras SMART persistidas; sin ellas (o sin `smartctl_path`) `state` es `unknown`
/// con un motivo explícito, nunca un relleno inventado (FR-004, FR-006).
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct DiskSummary {
    pub id: String,
    pub alias: Option<String>,
    pub model: String,
    pub device_type: DeviceType,
    pub state: HealthState,
    pub temperature_c: Option<f64>,
    pub percentage_used: Option<f64>,
    pub activity_percent: Option<f64>,
    pub power_on_hours: Option<f64>,
    pub vendor_temp_limit_c: Option<f64>,
    pub vendor_temp_critical_c: Option<f64>,
    pub unknown_reason: Option<UnknownReason>,
    pub last_read_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional = nullable)]
    pub provenance: Option<Provenance>,
    pub volumes: Vec<VolumeSummary>,
}

/// Espejo de `DeviceCapability` en `docs/ui-contract.md` §3.2. Solo `Smart` se evalúa hoy, con
/// `available = smartctl_path.is_some()`: las otras cuatro claves existen en el contrato pero
/// ningún colector actual sabe responder por ellas, así que no aparecen en la lista en vez de
/// fabricar un `false` que nadie ha comprobado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum DeviceCapabilityKey {
    Smart,
    NvmeLog,
    SelfTestShort,
    ChkdskScan,
    Temperature,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct DeviceCapability {
    pub key: DeviceCapabilityKey,
    pub available: bool,
    pub reason_key: Option<String>,
}

/// Espejo de `SmartCounter` en `docs/ui-contract.md` §3.2. `delta`/`delta_is_meaningful` quedan en
/// `None`/`false`: calcular un incremento con sentido (y decidir si empeora o mejora) es trabajo de
/// `alerts/`, que todavía no existe. Mostrar un delta inventado sería peor que no mostrar ninguno.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct SmartCounter {
    pub metric_key: String,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub delta: Option<f64>,
    pub delta_is_meaningful: bool,
    pub provenance: Provenance,
}

/// Espejo de `DeviceDetail` en `docs/ui-contract.md` §3.2: `DiskSummary` con los campos propios de
/// identidad y ciclo de vida. `#[serde(flatten)]` evita duplicar los dieciséis campos de
/// `DiskSummary` a mano, que es justo el riesgo de divergencia que `ts-rs` existe para cerrar.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct DeviceDetail {
    #[serde(flatten)]
    #[ts(flatten)]
    pub summary: DiskSummary,
    pub fingerprint: String,
    pub identity_confidence: IdentityConfidence,
    pub serial_number: Option<String>,
    pub firmware: Option<String>,
    pub bus_type: Option<String>,
    pub capabilities: Vec<DeviceCapability>,
    pub counters: Vec<SmartCounter>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub removed_at: Option<String>,
}

/// Espejo de `SourceHealth` en `docs/ui-contract.md` §2.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct SourceHealth {
    pub source: MetricSource,
    /// `SourceStatus` en `docs/ui-contract.md` §2, sin tipar todavía: nada produce este valor
    /// hasta que exista el seguimiento de estado por fuente (T020/T021).
    pub status: String,
    pub last_success_at: Option<String>,
    pub last_attempt_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional = nullable)]
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct DeviceListResponse {
    pub devices: Vec<DiskSummary>,
    pub excluded: Vec<DiskSummary>,
    pub sources: Vec<SourceHealth>,
    pub paused: bool,
    pub paused_since: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub author: String,
}

/// Espejo de `AlertGroup` en `src/lib/design/types.ts`, tras ADR-030: **sin** `title` ni
/// `summary` — el frontend los resuelve desde `ruleKey` por i18n. `target` sí es del backend:
/// no es texto de interfaz, es el alias o modelo del disco (dato del usuario).
///
/// Sin `ts-rs`: `AlertGroup` es vocabulario compartido de `design/types.ts`, no un tipo de
/// `api/generated/` — mezclar ambos rompería esa separación. El contrato en tiempo de ejecución
/// lo verifica el esquema Zod ya actualizado (`schemas.ts`), como con `SystemEvent` o `TestRun`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertGroupWire {
    pub id: String,
    pub rule_key: String,
    pub deduplication_key: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub count: i64,
    pub first_occurred_at: String,
    pub last_occurred_at: String,
    pub target: String,
    pub muted_until: Option<String>,
    pub cycle: Option<i64>,
}

/// Espejo de `AlertDetail` (`docs/ui-contract.md` §3.4): `AlertGroup` con hechos, cronología de
/// ocurrencias y eventos relacionados. `related_events` queda **siempre vacío**: la correlación
/// con el registro de eventos es la Historia 4, todavía sin construir — una lista vacía es la
/// verdad, no falta nada que se pueda mostrar hoy.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertFact {
    pub label_key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertOccurrenceWire {
    pub occurred_at: String,
    pub cycle: i64,
    pub value: Option<f64>,
    pub event_id: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertDetail {
    #[serde(flatten)]
    pub group: AlertGroupWire,
    pub facts: Vec<AlertFact>,
    pub occurrences: Vec<AlertOccurrenceWire>,
    pub related_events: Vec<serde_json::Value>,
}

/// Un `Device` de dominio, presentado como `DiskSummary` **sin** enriquecer con salud: siempre
/// `unknown` con `not-yet-sampled`. Es la base neutra que `enrich_with_smart_data` completa con
/// las muestras reales; no se usa sola salvo cuando de verdad no hay ninguna lectura que mostrar.
fn device_to_summary(d: &Device) -> DiskSummary {
    DiskSummary {
        id: d.id.clone(),
        alias: d.alias.clone(),
        model: d.model.clone(),
        device_type: d.device_type,
        state: HealthState::Unknown,
        temperature_c: None,
        percentage_used: None,
        activity_percent: None,
        power_on_hours: None,
        vendor_temp_limit_c: None,
        vendor_temp_critical_c: None,
        unknown_reason: Some(UnknownReason::NotYetSampled),
        last_read_at: None,
        provenance: None,
        volumes: vec![],
    }
}

fn rusqlite_err_to_app_error(e: rusqlite::Error) -> Box<AppError> {
    let mensaje = e.to_string();
    let bloqueado = mensaje.contains("database is locked") || mensaje.contains("busy");
    let mut err = AppError::new(
        if bloqueado {
            "db.locked"
        } else {
            "db.query_failed"
        },
        if bloqueado {
            "error.dbLocked"
        } else {
            "error.dbQueryFailed"
        },
    )
    .with_detail(mensaje);
    if bloqueado {
        err = err.retryable();
    }
    Box::new(err)
}

/// Apariencia persistida. Mientras no exista la tabla `settings`, devuelve los valores de arranque
/// que la especificación define: seguir al sistema en tema e idioma.
#[tauri::command]
pub fn get_appearance_settings() -> AppResult<AppearanceSettings> {
    Ok(AppearanceSettings {
        theme: "system".into(),
        language: None,
        system_locale: crate::platform::locale::system_locale(),
        use_system_accent: true,
    })
}

#[tauri::command]
pub fn get_system_accent_color() -> AppResult<WindowsAccent> {
    accent::read()
}

#[tauri::command]
pub fn set_setting(key: String, _value: serde_json::Value) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "set_setting({key})"
    ))))
}

/// Inventario. Una lista vacía no es un error: no tener discos todavía es un estado legítimo, y la
/// interfaz lo distingue de un fallo de fuente por `sources` (`docs/ui-contract.md` §3.2).
pub(crate) fn get_devices_impl(conn: &rusqlite::Connection) -> AppResult<DeviceListResponse> {
    let todos = repo_inventario::list_present_devices(conn).map_err(rusqlite_err_to_app_error)?;
    let (incluidos, excluidos): (Vec<_>, Vec<_>) = todos.iter().partition(|d| d.monitoring_enabled);

    Ok(DeviceListResponse {
        devices: incluidos
            .iter()
            .map(|d| enrich_with_smart_data(conn, d))
            .collect::<Result<_, _>>()?,
        excluded: excluidos
            .iter()
            .map(|d| enrich_with_smart_data(conn, d))
            .collect::<Result<_, _>>()?,
        sources: vec![],
        paused: false,
        paused_since: None,
    })
}

#[tauri::command]
pub fn get_devices(state: State<AppState>) -> AppResult<DeviceListResponse> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let mut respuesta = get_devices_impl(&conn)?;

    let pausado_desde = state
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro")
        .clone();
    respuesta.paused = pausado_desde.is_some();
    respuesta.paused_since = pausado_desde;
    Ok(respuesta)
}

/// Cadencia esperada de la lectura SMART completa (`docs/product-specification.md` §4): 5 minutos.
/// Es la base del margen de frescura (`domain::salud::es_dato_caduco`).
const CADENCIA_SMART_SEGUNDOS: i64 = 5 * 60;

fn segundos_desde(sampled_at_utc: &str, ahora: time::OffsetDateTime) -> Option<i64> {
    time::OffsetDateTime::parse(
        sampled_at_utc,
        &time::format_description::well_known::Rfc3339,
    )
    .ok()
    .map(|t| (ahora - t).whole_seconds())
}

/// Completa un `DiskSummary` con las últimas muestras SMART persistidas.
///
/// La distinción "sin compatibilidad" / "aún sin leer" sigue el criterio provisional de
/// `docs/open-questions.md` J.15: sin `smartctl_path` no hay forma de consultar el dispositivo
/// (`unsupported`); con `smartctl_path` pero sin ninguna muestra `smartctl`, todavía no se ha
/// leído (`not-yet-sampled`). Una muestra existente pero caduca reutiliza el mismo motivo: el
/// vocabulario de `UnknownReason` no distingue hoy "nunca leído" de "dejó de leerse", y J.15 lo
/// deja como provisional hasta medir.
/// Los volúmenes enlazados a un dispositivo, listos para `DiskSummary.volumes`. Independiente de
/// si el disco tiene lectura SMART: un disco sin SMART puede perfectamente tener volúmenes.
fn build_volume_summaries(
    conn: &rusqlite::Connection,
    device_id: &str,
) -> AppResult<Vec<VolumeSummary>> {
    let ids =
        repo_inventario::volumes_for_device(conn, device_id).map_err(rusqlite_err_to_app_error)?;
    let mut resumenes = Vec::with_capacity(ids.len());
    for id in ids {
        let Some(v) = repo_inventario::get_volume(conn, &id).map_err(rusqlite_err_to_app_error)?
        else {
            continue;
        };
        let drive_letters: Vec<String> = v
            .drive_letters_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();
        let label = v
            .label
            .clone()
            .unwrap_or_else(|| drive_letters.first().cloned().unwrap_or_default());
        let chkdsk_available = tests::chkdsk::admite_scan(v.filesystem.as_deref());
        resumenes.push(VolumeSummary {
            id: v.id,
            label,
            drive_letters,
            capacity_bytes: v.capacity_bytes,
            free_bytes: v.free_bytes,
            mapping_confidence: v
                .device_mapping_confidence
                .unwrap_or(MappingConfidence::Unknown),
            chkdsk_available,
        });
    }
    Ok(resumenes)
}

fn enrich_with_smart_data(conn: &rusqlite::Connection, d: &Device) -> AppResult<DiskSummary> {
    let mut resumen = device_to_summary(d);
    resumen.volumes = build_volume_summaries(conn, &d.id)?;

    let Some(_ruta) = &d.smartctl_path else {
        return Ok(resumen);
    };

    let temperatura = repo_metricas::latest_device_sample(conn, &d.id, "temperature_celsius")
        .map_err(rusqlite_err_to_app_error)?;
    let desgaste = repo_metricas::latest_device_sample(conn, &d.id, "percentage_used")
        .map_err(rusqlite_err_to_app_error)?;
    let horas_encendido = repo_metricas::latest_device_sample(conn, &d.id, "power_on_hours")
        .map_err(rusqlite_err_to_app_error)?;

    let Some(principal) = &temperatura else {
        // `not-yet-sampled` es el motivo por defecto que ya trae `device_to_summary`.
        return Ok(resumen);
    };

    let ahora = time::OffsetDateTime::now_utc();
    let antiguedad = segundos_desde(&principal.sampled_at_utc, ahora).unwrap_or(i64::MAX);
    let fresco = !crate::domain::salud::es_dato_caduco(antiguedad, CADENCIA_SMART_SEGUNDOS);

    resumen.temperature_c = temperatura.as_ref().and_then(|m| m.value_real);
    resumen.percentage_used = desgaste.as_ref().and_then(|m| m.value_real);
    resumen.power_on_hours = horas_encendido.as_ref().and_then(|m| m.value_real);
    resumen.last_read_at = Some(principal.sampled_at_utc.clone());
    resumen.provenance = Some(Provenance {
        source: MetricSource::Smartctl,
        quality: "exact".to_string(),
        read_at: Some(principal.sampled_at_utc.clone()),
    });

    resumen.state = crate::domain::salud::device_state(true, fresco, None);
    resumen.unknown_reason = if fresco {
        None
    } else {
        Some(UnknownReason::NotYetSampled)
    };

    Ok(resumen)
}

/// Construye los contadores SMART a partir de las últimas muestras de cada clave, con su
/// procedencia. Vacío si nunca se ha leído el dispositivo: una lista vacía es la verdad, no un
/// relleno (FR-004).
fn build_smart_counters(
    conn: &rusqlite::Connection,
    device_id: &str,
) -> AppResult<Vec<SmartCounter>> {
    let muestras = repo_metricas::latest_samples_by_source(conn, device_id, MetricSource::Smartctl)
        .map_err(rusqlite_err_to_app_error)?;

    Ok(muestras
        .into_iter()
        .map(|m| SmartCounter {
            metric_key: m.metric_key,
            value: m.value_real,
            unit: Some(m.unit),
            delta: None,
            delta_is_meaningful: false,
            provenance: Provenance {
                source: m.source,
                quality: "exact".to_string(),
                read_at: Some(m.sampled_at_utc),
            },
        })
        .collect())
}

fn get_device_detail_impl(conn: &rusqlite::Connection, device_id: &str) -> AppResult<DeviceDetail> {
    let dispositivo = repo_inventario::get_device(conn, device_id)
        .map_err(rusqlite_err_to_app_error)?
        .ok_or_else(|| Box::new(AppError::new("device.not_found", "error.deviceNotFound")))?;

    let summary = enrich_with_smart_data(conn, &dispositivo)?;
    let counters = build_smart_counters(conn, device_id)?;

    // "nvme_log" y "temperature" quedan fuera de la lista en vez de fabricar un `false` sin
    // comprobar: ningún colector actual sabe responder por ellas.
    let admite_autotest = tests::autotest::admite_autotest(dispositivo.smartctl_path.as_deref());
    let admite_chkdsk = repo_inventario::volumes_for_device(conn, device_id)
        .map_err(rusqlite_err_to_app_error)?
        .iter()
        .any(|volume_id| {
            repo_inventario::get_volume(conn, volume_id)
                .ok()
                .flatten()
                .is_some_and(|v| tests::chkdsk::admite_scan(v.filesystem.as_deref()))
        });
    let capabilities = vec![
        DeviceCapability {
            key: DeviceCapabilityKey::Smart,
            available: dispositivo.smartctl_path.is_some(),
            reason_key: if dispositivo.smartctl_path.is_some() {
                None
            } else {
                Some("disk.noSmartData".to_string())
            },
        },
        DeviceCapability {
            key: DeviceCapabilityKey::SelfTestShort,
            available: admite_autotest,
            reason_key: if admite_autotest {
                None
            } else {
                Some("disk.noSmartData".to_string())
            },
        },
        DeviceCapability {
            key: DeviceCapabilityKey::ChkdskScan,
            available: admite_chkdsk,
            reason_key: if admite_chkdsk {
                None
            } else {
                Some("tests.chkdskUnsupportedReason".to_string())
            },
        },
    ];

    Ok(DeviceDetail {
        summary,
        fingerprint: dispositivo.fingerprint,
        identity_confidence: dispositivo.identity_confidence,
        serial_number: dispositivo.serial_number,
        firmware: dispositivo.firmware,
        bus_type: dispositivo.bus_type,
        capabilities,
        counters,
        first_seen_at: dispositivo.first_seen_at,
        last_seen_at: dispositivo.last_seen_at,
        removed_at: dispositivo.removed_at,
    })
}

#[tauri::command]
pub fn get_device_detail(state: State<AppState>, device_id: String) -> AppResult<DeviceDetail> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    get_device_detail_impl(&conn, &device_id)
}

fn set_device_monitoring_impl(
    conn: &rusqlite::Connection,
    device_id: &str,
    enabled: bool,
) -> AppResult<()> {
    if repo_inventario::get_device(conn, device_id)
        .map_err(rusqlite_err_to_app_error)?
        .is_none()
    {
        return Err(Box::new(AppError::new(
            "device.not_found",
            "error.deviceNotFound",
        )));
    }
    repo_inventario::set_monitoring(conn, device_id, enabled).map_err(rusqlite_err_to_app_error)
}

#[tauri::command]
pub fn set_device_monitoring(
    state: State<AppState>,
    device_id: String,
    enabled: bool,
) -> AppResult<()> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    set_device_monitoring_impl(&conn, &device_id, enabled)
}

fn set_device_alias_impl(
    conn: &rusqlite::Connection,
    device_id: &str,
    alias: Option<&str>,
) -> AppResult<()> {
    if repo_inventario::get_device(conn, device_id)
        .map_err(rusqlite_err_to_app_error)?
        .is_none()
    {
        return Err(Box::new(AppError::new(
            "device.not_found",
            "error.deviceNotFound",
        )));
    }
    repo_inventario::set_alias(conn, device_id, alias).map_err(rusqlite_err_to_app_error)
}

#[tauri::command]
pub fn set_device_alias(
    state: State<AppState>,
    device_id: String,
    alias: Option<String>,
) -> AppResult<()> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    set_device_alias_impl(&conn, &device_id, alias.as_deref())
}

/// Reconcilia el inventario ya leído (`leidos`) contra lo persistido: alta o actualización de cada
/// disco, y marca como retirado lo que ya no aparece. Recibe la lectura ya hecha, no la ejecuta
/// ella misma, para que la lógica de reconciliación se pueda probar sin PowerShell de por medio.
/// El número tras `\\.\PhysicalDriveN`: la misma numeración efímera de Windows que usa
/// `Get-Partition` (`DiskNumber`) para decir a qué disco pertenece un volumen. No es identidad
/// (`disco.wwn_o_pnp_device_id` lo es), pero es lo único que ambas consultas comparten en el
/// mismo instante de lectura.
fn disk_number_from_smartctl_path(path: &str) -> Option<i64> {
    path.strip_prefix(r"\\.\PhysicalDrive")?.parse().ok()
}

fn reconciliar_inventario(
    conn: &rusqlite::Connection,
    leidos: &[crate::collectors::windows_storage::DiscoFisico],
    volumenes_leidos: &[crate::collectors::capacidad::VolumenLeido],
    ahora: &str,
) -> AppResult<()> {
    let mut mapa_disco_a_device_id: std::collections::HashMap<i64, String> =
        std::collections::HashMap::new();

    for disco in leidos {
        let huella = identidad::compute_fingerprint(
            &disco.model,
            disco.capacity_bytes,
            &disco.bus_type,
            &disco.wwn_o_pnp_device_id,
        );
        let existente = repo_inventario::get_device_by_fingerprint(conn, &huella)
            .map_err(rusqlite_err_to_app_error)?;

        let dispositivo = Device {
            id: existente
                .as_ref()
                .map(|d| d.id.clone())
                .unwrap_or_else(|| huella.clone()),
            fingerprint: huella,
            identity_confidence: identidad::identity_confidence(disco.serial_number.as_deref()),
            serial_number: disco.serial_number.clone(),
            model: disco.model.clone(),
            manufacturer: disco.manufacturer.clone(),
            firmware: disco.firmware.clone(),
            device_type: disco.device_type,
            bus_type: Some(disco.bus_type.clone()),
            // El índice de Windows puede cambiar entre arranques (no es identidad, la huella lo
            // es), así que se refresca en cada reconciliación en vez de conservar el valor previo.
            smartctl_path: disco.smartctl_device_path.clone(),
            capacity_bytes: disco.capacity_bytes,
            alias: existente.as_ref().and_then(|d| d.alias.clone()),
            monitoring_enabled: existente
                .as_ref()
                .map(|d| d.monitoring_enabled)
                .unwrap_or(true),
            first_seen_at: existente
                .as_ref()
                .map(|d| d.first_seen_at.clone())
                .unwrap_or_else(|| ahora.to_string()),
            last_seen_at: ahora.to_string(),
            removed_at: None,
            capabilities_json: None,
        };
        if let Some(n) = disco
            .smartctl_device_path
            .as_deref()
            .and_then(disk_number_from_smartctl_path)
        {
            mapa_disco_a_device_id.insert(n, dispositivo.id.clone());
        }
        repo_inventario::upsert_device(conn, &dispositivo).map_err(rusqlite_err_to_app_error)?;
    }

    reconciliar_volumenes(conn, volumenes_leidos, &mapa_disco_a_device_id, ahora)?;

    // Bajas: lo que estaba presente y no vino en esta lectura. La clasificación de "expulsión
    // segura" frente a "retirada sin aviso" (T026) exige el colector de eventos de la Historia 4;
    // hasta entonces se marca como retirado sin generar la distinción de severidad de alerta.
    let presentes_antes: Vec<String> = repo_inventario::list_present_devices(conn)
        .map_err(rusqlite_err_to_app_error)?
        .into_iter()
        .map(|d| d.fingerprint)
        .collect();
    let leidas_ahora: Vec<String> = leidos
        .iter()
        .map(|d| {
            identidad::compute_fingerprint(
                &d.model,
                d.capacity_bytes,
                &d.bus_type,
                &d.wwn_o_pnp_device_id,
            )
        })
        .collect();
    let cambios = identidad::reconcile(&presentes_antes, &leidas_ahora);
    for huella_baja in &cambios.bajas {
        if let Some(d) = repo_inventario::get_device_by_fingerprint(conn, huella_baja)
            .map_err(rusqlite_err_to_app_error)?
        {
            repo_inventario::mark_device_removed(conn, &d.id, ahora)
                .map_err(rusqlite_err_to_app_error)?;
        }
    }

    Ok(())
}

/// Persiste y enlaza los volúmenes leídos (T059). La confianza del enlace es `Exact` cuando el
/// número de disco de la lectura coincide con un dispositivo ya reconciliado en esta misma pasada
/// —ambos vienen del mismo instante—, `Unknown` si no hay ningún dispositivo con ese número
/// (por ejemplo, un volumen sobre un bus que `Get-PhysicalDisk` no expone). Ninguna otra confianza
/// tiene sentido aquí: no hay una vía "inferida" real todavía.
fn reconciliar_volumenes(
    conn: &rusqlite::Connection,
    leidos: &[crate::collectors::capacidad::VolumenLeido],
    mapa_disco_a_device_id: &std::collections::HashMap<i64, String>,
    ahora: &str,
) -> AppResult<()> {
    for vol in leidos {
        let existente = repo_inventario::get_volume(conn, &vol.volume_guid)
            .map_err(rusqlite_err_to_app_error)?;

        // Se recalcula en cada pasada, nunca se conserva la anterior: igual que `smartctl_path`
        // en el dispositivo, el número de disco de hoy es lo único que se puede confirmar hoy.
        let enlace = vol
            .disk_number
            .and_then(|n| mapa_disco_a_device_id.get(&n))
            .map(|id| (id.clone(), MappingConfidence::Exact));
        let confianza = enlace
            .as_ref()
            .map(|(_, c)| *c)
            .unwrap_or(MappingConfidence::Unknown);

        let volumen = Volume {
            id: vol.volume_guid.clone(),
            volume_guid: vol.volume_guid.clone(),
            label: vol.label.clone(),
            filesystem: vol.filesystem.clone(),
            drive_letters_json: vol
                .drive_letter
                .map(|c| serde_json::json!([format!("{c}:")]).to_string()),
            capacity_bytes: vol.capacity_bytes,
            free_bytes: vol.free_bytes,
            device_mapping_confidence: Some(confianza),
            first_seen_at: existente
                .as_ref()
                .map(|v| v.first_seen_at.clone())
                .unwrap_or_else(|| ahora.to_string()),
            last_seen_at: ahora.to_string(),
        };
        repo_inventario::upsert_volume(conn, &volumen).map_err(rusqlite_err_to_app_error)?;

        if let Some((device_id, confianza)) = enlace {
            repo_inventario::link_device_volume(
                conn,
                &device_id,
                &volumen.id,
                confianza,
                "disk_number",
            )
            .map_err(rusqlite_err_to_app_error)?;
        }
    }
    Ok(())
}

/// Fuerza una recopilación (US-013). El contrato normativo es `docs/ui-contract.md` §3.2:
/// `scope: "all" | "device"`, con `deviceId` obligatorio en el segundo caso.
///
/// - `"all"`: relee el inventario físico completo y, con lo que quede tras reconciliar, la salud
///   SMART de cada disco monitorizado.
/// - `"device"`: releé solo la salud SMART del disco indicado, sin volver a escanear el
///   inventario entero — es la vía barata para "he tocado este disco y quiero verlo ya".
///
/// Los ámbitos de métricas de rendimiento y eventos de Windows llegan con sus colectores
/// (Historias 3 y 4); mientras tanto no son parte del contrato de `refresh_now`, así que no hay
/// ninguna rama `not_implemented` que mantener para ellos aquí.
#[tauri::command]
pub fn refresh_now(
    app: tauri::AppHandle,
    state: State<AppState>,
    scope: String,
    device_id: Option<String>,
) -> AppResult<()> {
    let resultado = match scope.as_str() {
        "all" => {
            refresh_inventory(&state)?;
            let conn = state
                .conn
                .lock()
                .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
            refresh_events(&conn);
            refresh_smart(&conn, None)
        }
        "device" => {
            let id = device_id.ok_or_else(|| {
                Box::new(AppError::new("device.not_found", "error.deviceNotFound"))
            })?;
            let conn = state
                .conn
                .lock()
                .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
            refresh_smart(&conn, Some(&id))
        }
        _ => Err(Box::new(AppError::not_implemented(&format!(
            "refresh_now({scope})"
        )))),
    };
    // Fuera del bloqueo de la conexión que usó `refresh_smart`: notificar puede volver a abrirla.
    if let Ok(transiciones) = &resultado {
        crate::alerts::notificaciones::procesar_transiciones(&app, transiciones);
        let ids: Vec<String> = transiciones
            .iter()
            .filter(|(_, t)| *t != crate::alerts::agrupacion::Transicion::SinCambio)
            .map(|(id, _)| id.clone())
            .collect();
        if !ids.is_empty() {
            let conn = state
                .conn
                .lock()
                .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
            emitir_alerts_changed(&app, &conn, &ids);
        }
    }
    crate::platform::bandeja::actualizar(&app);
    resultado.map(|_| ())
}

/// Canal único vigilado por ahora (T067): los proveedores de `docs/alert-rules.md` §3 viven todos
/// en `System`. Ampliar a más canales es una lista, no un cambio de diseño.
const CANAL_EVENTOS: &str = "System";

/// Lee los eventos nuevos del canal, los correlaciona con el inventario ya reconciliado y los
/// persiste. Un fallo aquí **nunca** aborta `refresh_now`: es la misma tolerancia (SC-008) que ya
/// aplica `refresh_smart` disco a disco — sin colector de rendimiento (T020/T021) que reintente
/// por su cuenta, perder un ciclo de eventos es mucho mejor que perder el resto de la
/// recopilación por su culpa.
#[cfg(windows)]
fn refresh_events(conn: &rusqlite::Connection) {
    use crate::domain::tipos::SystemEvent;

    let bookmark_previo = match repo_varios::get_cursor(conn, CANAL_EVENTOS) {
        Ok(b) => b.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) => {
            tracing::warn!(canal = CANAL_EVENTOS, error = ?e, "no se pudo leer el cursor de eventos");
            None
        }
    };

    let (eventos, bookmark_nuevo) = match crate::collectors::event_log::leer_eventos_nuevos(
        CANAL_EVENTOS,
        bookmark_previo.as_deref(),
    ) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(canal = CANAL_EVENTOS, error = ?e, "no se pudieron leer los eventos del sistema");
            return;
        }
    };

    if eventos.is_empty() {
        return;
    }

    let dispositivos = repo_inventario::list_present_devices(conn).unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "no se pudo leer el inventario para correlacionar eventos");
        vec![]
    });
    let discos_conocidos: Vec<crate::domain::correlacion::DiscoConocido> = dispositivos
        .iter()
        .filter_map(|d| {
            d.smartctl_path
                .as_deref()
                .and_then(disk_number_from_smartctl_path)
                .map(|n| crate::domain::correlacion::DiscoConocido {
                    disk_number: n,
                    device_id: &d.id,
                })
        })
        .collect();

    for evento in &eventos {
        let (device_id, confianza) = crate::domain::correlacion::correlacionar(
            &evento.raw_xml,
            evento.message.as_deref(),
            &discos_conocidos,
        );
        let dedup_hash = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(evento.provider.as_bytes());
            hasher.update(b"|");
            hasher.update(evento.event_id.to_string().as_bytes());
            hasher.update(b"|");
            hasher.update(evento.occurred_at_utc.as_bytes());
            hasher.update(b"|");
            hasher.update(evento.message.as_deref().unwrap_or_default().as_bytes());
            format!("{:x}", hasher.finalize())
        };
        let evento_dominio = SystemEvent {
            id: 0,
            channel: evento.channel.clone(),
            record_id: evento.record_id,
            occurred_at_utc: evento.occurred_at_utc.clone(),
            provider: evento.provider.clone(),
            event_id: evento.event_id,
            level: evento.level,
            message: evento.message.clone(),
            raw_xml: Some(evento.raw_xml.clone()),
            device_id,
            volume_id: None,
            mapping_confidence: confianza,
            dedup_hash,
        };
        if let Err(e) = repo_varios::insert_event_if_new(conn, &evento_dominio) {
            tracing::warn!(
                canal = %evento.channel, record_id = evento.record_id, error = ?e,
                "no se pudo persistir un evento del sistema"
            );
        }
    }

    if let Some(bookmark) = bookmark_nuevo {
        let ahora = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        if let Err(e) = repo_varios::set_cursor(conn, CANAL_EVENTOS, bookmark.as_bytes(), &ahora) {
            tracing::warn!(canal = CANAL_EVENTOS, error = ?e, "no se pudo guardar el cursor de eventos");
        }
    }
}

#[cfg(not(windows))]
fn refresh_events(_conn: &rusqlite::Connection) {}

fn refresh_inventory(state: &State<AppState>) -> AppResult<()> {
    #[cfg(windows)]
    let leidos = crate::collectors::windows_storage::list_physical_disks().map_err(|e| {
        Box::new(
            AppError::new("windows_storage.failed", "error.storageCollectorFailed")
                .with_detail(format!("{e:?}"))
                .from_source(crate::domain::tipos::MetricSource::WindowsStorage)
                .retryable(),
        )
    })?;
    #[cfg(not(windows))]
    let leidos: Vec<crate::collectors::windows_storage::DiscoFisico> = vec![];

    // Un fallo al leer volúmenes no debe tumbar la recopilación de discos (SC-008): se registra y
    // se continúa con una lista vacía, igual que ya hace `refresh_smart` disco a disco.
    #[cfg(windows)]
    let volumenes = crate::collectors::capacidad::list_volumes().unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "no se pudo leer la capacidad de los volúmenes");
        vec![]
    });
    #[cfg(not(windows))]
    let volumenes: Vec<crate::collectors::capacidad::VolumenLeido> = vec![];

    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    reconciliar_inventario(&conn, &leidos, &volumenes, &ahora)
}

/// Persiste una lectura ya parseada de `smartctl`: cada campo presente como muestra, y la
/// instantánea completa para `smart_snapshots`. Puro respecto al proceso externo — no invoca
/// `smartctl`, solo guarda lo que ya se leyó —, así que se puede probar sin hardware.
fn persist_smart_reading(
    conn: &rusqlite::Connection,
    device_id: &str,
    resultado: &crate::collectors::smartctl_parser::SmartctlResult,
    sampled_at_utc: &str,
) -> rusqlite::Result<()> {
    use crate::domain::tipos::{
        MetricQuality, MetricSample, MetricSource, MetricTarget, Resolution,
    };

    for metrica in &resultado.metrics {
        let unidad = if metrica.metric_key == "temperature_celsius" {
            "celsius"
        } else if metrica.metric_key.ends_with("_bytes_total") {
            "bytes"
        } else if metrica.metric_key == "power_on_hours" {
            "hours"
        } else {
            "count"
        };
        repo_metricas::insert_sample(
            conn,
            &MetricSample {
                target: MetricTarget::Device(device_id.to_string()),
                metric_key: metrica.metric_key.to_string(),
                value_real: Some(metrica.value),
                value_integer: None,
                unit: unidad.to_string(),
                sampled_at_utc: sampled_at_utc.to_string(),
                source: MetricSource::Smartctl,
                quality: MetricQuality::Exact,
                resolution: Resolution::Raw,
            },
        )?;
    }

    repo_metricas::insert_smart_snapshot(
        conn,
        device_id,
        sampled_at_utc,
        resultado.smartctl_version.as_deref(),
        Some(resultado.exit_status as i64),
        if resultado.health_passed.is_some() {
            "ok"
        } else {
            "no_health_field"
        },
        None,
        None,
    )
}

/// Persiste una lectura de contadores de rendimiento (T058): un campo ausente se omite, nunca se
/// guarda como cero (`docs/data-model.md` §3, "los campos no disponibles se omiten").
#[cfg(windows)]
fn persist_perf_reading(
    conn: &rusqlite::Connection,
    device_id: &str,
    lectura: &crate::collectors::perf_counters::LecturaRendimiento,
    sampled_at_utc: &str,
) -> rusqlite::Result<()> {
    use crate::domain::tipos::{
        MetricQuality, MetricSample, MetricSource, MetricTarget, Resolution,
    };

    for (metric_key, unidad, valor) in [
        ("activity_percent", "percent", lectura.activity_percent),
        (
            "read_bytes_per_second",
            "bytes_per_second",
            lectura.read_bytes_per_second,
        ),
        (
            "write_bytes_per_second",
            "bytes_per_second",
            lectura.write_bytes_per_second,
        ),
        ("read_latency_ms", "milliseconds", lectura.read_latency_ms),
        ("write_latency_ms", "milliseconds", lectura.write_latency_ms),
    ] {
        let Some(v) = valor else { continue };
        repo_metricas::insert_sample(
            conn,
            &MetricSample {
                target: MetricTarget::Device(device_id.to_string()),
                metric_key: metric_key.to_string(),
                value_real: Some(v),
                value_integer: None,
                unit: unidad.to_string(),
                sampled_at_utc: sampled_at_utc.to_string(),
                source: MetricSource::PerformanceCounter,
                quality: MetricQuality::Exact,
                resolution: Resolution::Raw,
            },
        )?;
    }
    Ok(())
}

/// Consulta `smartctl` para los dispositivos presentes con `smartctl_path` conocido —todos si
/// `solo_device_id` es `None`, uno solo si se indica (ámbito `"device"` de `refresh_now`). Un
/// fallo en un disco no aborta el resto (SC-008): sin seguimiento de estado por fuente todavía
/// (T020/T021), el fallo se registra y se continúa, en vez de propagarse y degradar la recopilación
/// entera.
fn refresh_smart(
    conn: &rusqlite::Connection,
    solo_device_id: Option<&str>,
) -> AppResult<Vec<(String, crate::alerts::agrupacion::Transicion)>> {
    if let Some(id) = solo_device_id {
        if repo_inventario::get_device(conn, id)
            .map_err(rusqlite_err_to_app_error)?
            .is_none()
        {
            return Err(Box::new(AppError::new(
                "device.not_found",
                "error.deviceNotFound",
            )));
        }
    }

    let dispositivos =
        repo_inventario::list_present_devices(conn).map_err(rusqlite_err_to_app_error)?;

    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    let mut transiciones = Vec::new();

    for d in dispositivos
        .iter()
        .filter(|d| d.monitoring_enabled)
        .filter(|d| match solo_device_id {
            Some(id) => id == d.id,
            None => true,
        })
    {
        let Some(ruta) = &d.smartctl_path else {
            continue;
        };

        #[cfg(windows)]
        let json = match crate::collectors::smartctl::query_device_json(ruta) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!(disco = %d.id, error = ?e, "no se pudo consultar smartctl");
                continue;
            }
        };
        #[cfg(not(windows))]
        let json = continue;

        match crate::collectors::smartctl_parser::parse_smartctl_json(&json) {
            Ok(resultado) => {
                if let Err(e) = persist_smart_reading(conn, &d.id, &resultado, &ahora) {
                    tracing::warn!(disco = %d.id, error = ?e, "no se pudo guardar la lectura SMART");
                } else {
                    match crate::alerts::evaluar_smart(conn, &d.id, &ahora) {
                        Ok(mut t) => transiciones.append(&mut t),
                        // La lectura ya quedó guardada: un fallo al evaluar alertas no debe hacer
                        // parecer que la lectura en sí falló.
                        Err(e) => {
                            tracing::warn!(disco = %d.id, error = ?e, "no se pudo evaluar el motor de alertas")
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(disco = %d.id, error = ?e, "smartctl devolvió un JSON irreconocible");
            }
        }

        #[cfg(windows)]
        if let Some(n) = disk_number_from_smartctl_path(ruta) {
            match crate::collectors::perf_counters::leer(n) {
                Ok(lectura) => {
                    if let Err(e) = persist_perf_reading(conn, &d.id, &lectura, &ahora) {
                        tracing::warn!(disco = %d.id, error = ?e, "no se pudo guardar la lectura de rendimiento");
                    }
                }
                Err(e) => {
                    tracing::warn!(disco = %d.id, error = ?e, "no se pudo leer los contadores de rendimiento");
                }
            }
        }
    }

    Ok(transiciones)
}

/// Espejo de `MetricSeries` en `src/lib/api/types.ts` (T062, `docs/open-questions.md` E.1).
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct PuntoSerieWire {
    // `f64`, no `i64`: `ts-rs` mapea `i64` a `bigint`, y el contrato con la interfaz
    // (`TimeSeriesChart.svelte`) es un `number` — milisegundos epoch caben enteros de sobra en
    // la mantisa de 53 bits de un `f64`.
    pub t: f64,
    pub v: Option<f64>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct MetricSeriesWire {
    pub metric_key: String,
    pub unit: String,
    pub resolution: Resolution,
    pub downsampled: bool,
    pub from_utc: String,
    pub to_utc: String,
    // `f64` por el mismo motivo que `PuntoSerieWire.t`: sin él, `ts-rs` mapea el `i64` a `bigint`.
    pub expected_interval_ms: f64,
    pub points: Vec<PuntoSerieWire>,
    pub vendor_limit: Option<f64>,
    pub vendor_critical: Option<f64>,
}

/// Cadencia esperada por métrica (`docs/product-specification.md` §4): 30 s para temperatura,
/// actividad y latencias; 5 min para el resto de SMART completo. Decide tanto el hueco como la
/// resolución `raw`.
fn cadencia_esperada_ms(metric_key: &str) -> i64 {
    match metric_key {
        "temperature_celsius"
        | "activity_percent"
        | "read_bytes_per_second"
        | "write_bytes_per_second"
        | "read_latency_ms"
        | "write_latency_ms"
        | "volume_free_bytes"
        | "volume_free_percent" => 30_000,
        _ => 300_000,
    }
}

/// Unidad por métrica, para cuando la serie no tiene ninguna muestra real de la que tomarla
/// (un intervalo enteramente hueco no deja de necesitar declarar su unidad).
fn unidad_para(metric_key: &str) -> &'static str {
    match metric_key {
        "temperature_celsius" => "celsius",
        "activity_percent" | "percentage_used" | "volume_free_percent" => "percent",
        "read_bytes_per_second" | "write_bytes_per_second" => "bytes_per_second",
        "read_latency_ms" | "write_latency_ms" => "milliseconds",
        "power_on_hours" => "hours",
        k if k.ends_with("_bytes_total") || k == "volume_free_bytes" => "bytes",
        _ => "count",
    }
}

/// Tope de puntos por serie (`docs/open-questions.md` E.1): por encima, se submuestrea
/// conservando mínimo y máximo de cada cubo.
const TOPE_PUNTOS_SERIE: usize = 1500;

fn leer_agregados_dispositivo(
    conn: &rusqlite::Connection,
    device_id: &str,
    metric_key: &str,
    resolution: AggregateResolution,
    from_utc: &str,
    to_utc: &str,
) -> rusqlite::Result<Vec<(String, f64)>> {
    Ok(repo_agregados::device_aggregates(
        conn, device_id, metric_key, resolution, from_utc, to_utc,
    )?
    .into_iter()
    .filter_map(|a| {
        a.value_avg
            .or(a.value_last)
            .map(|v| (a.bucket_start_utc, v))
    })
    .collect())
}

fn get_metric_series_impl(
    conn: &rusqlite::Connection,
    device_id: Option<&str>,
    volume_id: Option<&str>,
    metric_key: &str,
    from_utc: &str,
    to_utc: &str,
) -> AppResult<MetricSeriesWire> {
    let rfc3339 = &time::format_description::well_known::Rfc3339;
    let desde = time::OffsetDateTime::parse(from_utc, rfc3339).map_err(|e| {
        Box::new(
            AppError::new("ipc.schema_mismatch", "error.schemaMismatch").with_detail(e.to_string()),
        )
    })?;
    let hasta = time::OffsetDateTime::parse(to_utc, rfc3339).map_err(|e| {
        Box::new(
            AppError::new("ipc.schema_mismatch", "error.schemaMismatch").with_detail(e.to_string()),
        )
    })?;

    let ahora = time::OffsetDateTime::now_utc();
    let ancho = hasta - desde;
    let dentro_de_siete_dias = desde >= ahora - time::Duration::days(7);

    // Solo los dispositivos tienen serie real hoy: la capacidad de volumen se guarda en
    // `volumes.capacity_bytes`/`free_bytes` (T059), todavía no como muestra periódica
    // (`docs/open-questions.md`). Un volumen pedido devuelve honestamente un hueco, no un error.
    let (muestras, resolucion): (Vec<(String, f64)>, Resolution) = match device_id {
        Some(id) => {
            if ancho <= time::Duration::hours(24) && dentro_de_siete_dias {
                let filas = repo_metricas::device_series(conn, id, metric_key, from_utc, to_utc)
                    .map_err(rusqlite_err_to_app_error)?;
                (
                    filas
                        .into_iter()
                        .filter_map(|m| m.value_real.map(|v| (m.sampled_at_utc, v)))
                        .collect(),
                    Resolution::Raw,
                )
            } else if ancho <= time::Duration::days(7) {
                (
                    leer_agregados_dispositivo(
                        conn,
                        id,
                        metric_key,
                        AggregateResolution::FiveMinutes,
                        from_utc,
                        to_utc,
                    )
                    .map_err(rusqlite_err_to_app_error)?,
                    Resolution::FiveMinutes,
                )
            } else if ancho <= time::Duration::days(90) {
                let cinco_min = leer_agregados_dispositivo(
                    conn,
                    id,
                    metric_key,
                    AggregateResolution::FiveMinutes,
                    from_utc,
                    to_utc,
                )
                .map_err(rusqlite_err_to_app_error)?;
                if cinco_min.is_empty() {
                    (
                        leer_agregados_dispositivo(
                            conn,
                            id,
                            metric_key,
                            AggregateResolution::Hourly,
                            from_utc,
                            to_utc,
                        )
                        .map_err(rusqlite_err_to_app_error)?,
                        Resolution::Hourly,
                    )
                } else {
                    (cinco_min, Resolution::FiveMinutes)
                }
            } else {
                (
                    leer_agregados_dispositivo(
                        conn,
                        id,
                        metric_key,
                        AggregateResolution::Hourly,
                        from_utc,
                        to_utc,
                    )
                    .map_err(rusqlite_err_to_app_error)?,
                    Resolution::Hourly,
                )
            }
        }
        None => (vec![], Resolution::Raw),
    };
    let _ = volume_id; // aceptado por contrato; sin serie real todavía (ver comentario arriba).

    let cadencia_ms = match resolucion {
        Resolution::Raw => cadencia_esperada_ms(metric_key),
        Resolution::FiveMinutes => 5 * 60 * 1000,
        Resolution::Hourly => 60 * 60 * 1000,
    };

    let puntos_completos =
        crate::domain::series::completar_serie(&muestras, desde, hasta, cadencia_ms);
    let (puntos_finales, downsampled) =
        crate::domain::series::submuestrear(puntos_completos, TOPE_PUNTOS_SERIE);

    Ok(MetricSeriesWire {
        metric_key: metric_key.to_string(),
        unit: unidad_para(metric_key).to_string(),
        resolution: resolucion,
        downsampled,
        from_utc: from_utc.to_string(),
        to_utc: to_utc.to_string(),
        expected_interval_ms: cadencia_ms as f64,
        points: puntos_finales
            .into_iter()
            .map(|p| PuntoSerieWire {
                t: p.t_epoch_ms as f64,
                v: p.v,
            })
            .collect(),
        vendor_limit: None,
        vendor_critical: None,
    })
}

#[tauri::command]
pub fn get_metric_series(
    state: State<AppState>,
    device_id: Option<String>,
    volume_id: Option<String>,
    metric_key: String,
    from_utc: String,
    to_utc: String,
) -> AppResult<MetricSeriesWire> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    get_metric_series_impl(
        &conn,
        device_id.as_deref(),
        volume_id.as_deref(),
        &metric_key,
        &from_utc,
        &to_utc,
    )
}

#[cfg(test)]
mod tests_series {
    use super::*;
    use crate::domain::tipos::{MetricAggregate, MetricQuality, MetricSample, MetricTarget};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_series");
        let (conn, _) = db::open(&dir).unwrap();
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn muestra(cuando: &str, valor: f64) -> MetricSample {
        MetricSample {
            target: MetricTarget::Device("d1".to_string()),
            metric_key: "temperature_celsius".to_string(),
            value_real: Some(valor),
            value_integer: None,
            unit: "celsius".to_string(),
            sampled_at_utc: cuando.to_string(),
            source: MetricSource::Smartctl,
            quality: MetricQuality::Exact,
            resolution: Resolution::Raw,
        }
    }

    #[test]
    fn un_rango_reciente_y_corto_usa_muestras_crudas() {
        let conn = conn_de_prueba();
        let ahora = time::OffsetDateTime::now_utc();
        let hace_una_hora = ahora - time::Duration::hours(1);
        let fmt = |t: time::OffsetDateTime| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        };
        repo_metricas::insert_sample(&conn, &muestra(&fmt(hace_una_hora), 40.0)).unwrap();

        let serie = get_metric_series_impl(
            &conn,
            Some("d1"),
            None,
            "temperature_celsius",
            &fmt(hace_una_hora),
            &fmt(ahora),
        )
        .unwrap();

        assert_eq!(serie.resolution, Resolution::Raw);
        assert!(serie.points.iter().any(|p| p.v == Some(40.0)));
    }

    #[test]
    fn un_rango_de_mas_de_noventa_dias_usa_horario() {
        let conn = conn_de_prueba();
        let ahora = time::OffsetDateTime::now_utc();
        let hace_cien_dias = ahora - time::Duration::days(100);
        let fmt = |t: time::OffsetDateTime| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        };

        let serie = get_metric_series_impl(
            &conn,
            Some("d1"),
            None,
            "temperature_celsius",
            &fmt(hace_cien_dias),
            &fmt(ahora),
        )
        .unwrap();

        assert_eq!(serie.resolution, Resolution::Hourly);
        assert_eq!(serie.expected_interval_ms, 3_600_000.0);
    }

    #[test]
    fn sin_agregados_de_cinco_minutos_en_un_rango_de_treinta_dias_cae_a_horario() {
        let conn = conn_de_prueba();
        let ahora = time::OffsetDateTime::now_utc();
        let hace_treinta_dias = ahora - time::Duration::days(30);
        let fmt = |t: time::OffsetDateTime| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        };
        repo_agregados::insert_aggregate(
            &conn,
            &MetricAggregate {
                target: MetricTarget::Device("d1".to_string()),
                metric_key: "temperature_celsius".to_string(),
                bucket_start_utc: fmt(hace_treinta_dias),
                bucket_end_utc: fmt(hace_treinta_dias + time::Duration::hours(1)),
                resolution: crate::domain::tipos::AggregateResolution::Hourly,
                value_min: Some(38.0),
                value_max: Some(42.0),
                value_avg: Some(40.0),
                value_first: Some(38.0),
                value_last: Some(42.0),
                sample_count: 12,
                unit: "celsius".to_string(),
            },
        )
        .unwrap();

        let serie = get_metric_series_impl(
            &conn,
            Some("d1"),
            None,
            "temperature_celsius",
            &fmt(hace_treinta_dias),
            &fmt(ahora),
        )
        .unwrap();

        assert_eq!(serie.resolution, Resolution::Hourly);
        assert!(serie.points.iter().any(|p| p.v == Some(40.0)));
    }

    #[test]
    fn un_volumen_pedido_devuelve_un_hueco_honesto_no_un_error() {
        let conn = conn_de_prueba();
        let ahora = time::OffsetDateTime::now_utc();
        let fmt = |t: time::OffsetDateTime| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        };

        let serie = get_metric_series_impl(
            &conn,
            None,
            Some("vol-1"),
            "volume_free_bytes",
            &fmt(ahora - time::Duration::hours(1)),
            &fmt(ahora),
        )
        .unwrap();

        assert!(serie.points.iter().all(|p| p.v.is_none()));
    }

    #[test]
    fn una_fecha_ilegible_falla_con_schema_mismatch() {
        let conn = conn_de_prueba();
        let err = get_metric_series_impl(
            &conn,
            Some("d1"),
            None,
            "temperature_celsius",
            "no-es-una-fecha",
            "tampoco",
        )
        .unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn un_intervalo_por_encima_del_tope_se_submuestrea_y_lo_declara() {
        let conn = conn_de_prueba();
        let ahora = time::OffsetDateTime::now_utc();
        let hace_un_dia = ahora - time::Duration::hours(24);
        let fmt = |t: time::OffsetDateTime| {
            t.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        };
        // Más muestras que TOPE_PUNTOS_SERIE, una por segundo.
        for i in 0..(TOPE_PUNTOS_SERIE + 200) {
            let cuando = hace_un_dia + time::Duration::seconds(i as i64);
            repo_metricas::insert_sample(&conn, &muestra(&fmt(cuando), i as f64)).unwrap();
        }

        let serie = get_metric_series_impl(
            &conn,
            Some("d1"),
            None,
            "temperature_celsius",
            &fmt(hace_un_dia),
            &fmt(hace_un_dia + time::Duration::seconds((TOPE_PUNTOS_SERIE + 200) as i64)),
        )
        .unwrap();

        assert!(serie.downsampled);
        assert!(serie.points.len() <= TOPE_PUNTOS_SERIE);
    }
}

/// El nombre que la interfaz muestra para el objetivo de una alerta: alias o modelo del disco, o
/// la etiqueta del volumen. No es texto traducible — es dato del usuario, igual que en `DiskCard`.
fn resolve_target_name(conn: &rusqlite::Connection, g: &AlertGroup) -> String {
    if let Some(device_id) = &g.target_device_id {
        if let Ok(Some(d)) = repo_inventario::get_device(conn, device_id) {
            return d.alias.unwrap_or(d.model);
        }
    }
    if let Some(volume_id) = &g.target_volume_id {
        if let Ok(Some(v)) = repo_inventario::get_volume(conn, volume_id) {
            return v.label.unwrap_or(volume_id.clone());
        }
    }
    "?".to_string()
}

fn alert_group_to_wire(conn: &rusqlite::Connection, g: &AlertGroup) -> AlertGroupWire {
    AlertGroupWire {
        id: g.id.clone(),
        rule_key: g.rule_key.clone(),
        deduplication_key: g.deduplication_key.clone(),
        severity: g.severity,
        status: g.status,
        count: g.occurrence_count,
        first_occurred_at: g.first_occurrence_at_utc.clone(),
        last_occurred_at: g.last_occurrence_at_utc.clone(),
        target: resolve_target_name(conn, g),
        muted_until: g.muted_until.clone(),
        cycle: Some(g.cycle),
    }
}

/// Espejo de la carga de `alerts:changed` (`docs/ui-contract.md` §2). `removed` queda siempre
/// vacío: no existe todavía ningún camino que borre un grupo (solo lo archiva), así que no hay
/// nada real que poner ahí — inventarlo sería peor que dejarlo vacío.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AlertsChangedEvent {
    emitted_at: String,
    changed: Vec<AlertGroupWire>,
    removed: Vec<String>,
}

/// Emite `alerts:changed` con los grupos indicados, ya convertidos. Un id que ya no exista se
/// omite en vez de fallar: puede pasar si dos hilos compitieran por el mismo grupo, aunque hoy la
/// conexión es única y no debería ocurrir.
fn emitir_alerts_changed(app: &tauri::AppHandle, conn: &rusqlite::Connection, ids: &[String]) {
    let changed: Vec<AlertGroupWire> = ids
        .iter()
        .filter_map(|id| repo_alertas::get_group(conn, id).ok().flatten())
        .map(|g| alert_group_to_wire(conn, &g))
        .collect();
    if changed.is_empty() {
        return;
    }
    let emitted_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    if let Err(e) = app.emit(
        "alerts:changed",
        AlertsChangedEvent {
            emitted_at,
            changed,
            removed: vec![],
        },
    ) {
        tracing::warn!(error = %e, "no se pudo emitir alerts:changed");
    }
}

fn get_alert_groups_impl(
    conn: &rusqlite::Connection,
    status: Option<&[AlertStatus]>,
    device_id: Option<&str>,
) -> AppResult<Vec<AlertGroupWire>> {
    let todos = repo_alertas::list_groups(conn).map_err(rusqlite_err_to_app_error)?;
    Ok(todos
        .iter()
        .filter(|g| match status {
            Some(filtro) => filtro.contains(&g.status),
            None => true,
        })
        .filter(|g| match device_id {
            Some(id) => g.target_device_id.as_deref() == Some(id),
            None => true,
        })
        .map(|g| alert_group_to_wire(conn, g))
        .collect())
}

#[tauri::command]
pub fn get_alert_groups(
    state: State<AppState>,
    status: Option<Vec<AlertStatus>>,
    device_id: Option<String>,
) -> AppResult<Vec<AlertGroupWire>> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    get_alert_groups_impl(&conn, status.as_deref(), device_id.as_deref())
}

fn get_alert_detail_impl(
    conn: &rusqlite::Connection,
    alert_group_id: &str,
) -> AppResult<AlertDetail> {
    let grupo = repo_alertas::get_group(conn, alert_group_id)
        .map_err(rusqlite_err_to_app_error)?
        .ok_or_else(|| Box::new(AppError::new("device.not_found", "error.deviceNotFound")))?;

    // Hechos mínimos y honestos: regla y valor de la última lectura. Sin colector de eventos
    // (Historia 4) no hay más contexto que ofrecer sin inventarlo.
    let facts = vec![
        AlertFact {
            label_key: "alert.fact.ruleKey".to_string(),
            value: Some(grupo.rule_key.clone()),
        },
        AlertFact {
            label_key: "alert.fact.lastValue".to_string(),
            value: grupo.last_value_real.map(|v| v.to_string()),
        },
    ];

    let occurrences = repo_alertas::list_occurrences(conn, alert_group_id)
        .map_err(rusqlite_err_to_app_error)?
        .into_iter()
        .map(|o| AlertOccurrenceWire {
            occurred_at: o.occurred_at_utc,
            cycle: o.cycle,
            value: o.value_real,
            event_id: o.triggering_event_id.map(|id| id.to_string()),
            context: o.context_json,
        })
        .collect();

    Ok(AlertDetail {
        group: alert_group_to_wire(conn, &grupo),
        facts,
        occurrences,
        related_events: vec![],
    })
}

#[tauri::command]
pub fn get_alert_detail(state: State<AppState>, alert_group_id: String) -> AppResult<AlertDetail> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    get_alert_detail_impl(&conn, &alert_group_id)
}

fn existe_grupo(conn: &rusqlite::Connection, id: &str) -> AppResult<()> {
    if repo_alertas::get_group(conn, id)
        .map_err(rusqlite_err_to_app_error)?
        .is_none()
    {
        return Err(Box::new(AppError::new(
            "device.not_found",
            "error.deviceNotFound",
        )));
    }
    Ok(())
}

#[tauri::command]
pub fn acknowledge_alert(
    app: tauri::AppHandle,
    state: State<AppState>,
    alert_group_id: String,
) -> AppResult<()> {
    let resultado = {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        existe_grupo(&conn, &alert_group_id)?;
        let ahora = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let resultado = crate::alerts::ciclo::reconocer(&conn, &alert_group_id, &ahora)
            .map_err(rusqlite_err_to_app_error);
        if resultado.is_ok() {
            emitir_alerts_changed(&app, &conn, std::slice::from_ref(&alert_group_id));
        }
        resultado
    };
    // Reconocer no cambia el color de la bandeja (constitución §I): se recalcula igual, por
    // coherencia con el resto de acciones, y sale gratis porque nada cambió.
    crate::platform::bandeja::actualizar(&app);
    resultado
}

#[tauri::command]
pub fn mute_alert(
    app: tauri::AppHandle,
    state: State<AppState>,
    alert_group_id: String,
    minutes: Option<u32>,
) -> AppResult<()> {
    let resultado = {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        existe_grupo(&conn, &alert_group_id)?;
        let resultado = crate::alerts::ciclo::silenciar(
            &conn,
            &alert_group_id,
            minutes,
            time::OffsetDateTime::now_utc(),
        )
        .map_err(rusqlite_err_to_app_error);
        if resultado.is_ok() {
            emitir_alerts_changed(&app, &conn, std::slice::from_ref(&alert_group_id));
        }
        resultado
    };
    crate::platform::bandeja::actualizar(&app);
    resultado
}

#[tauri::command]
pub fn unmute_alert(
    app: tauri::AppHandle,
    state: State<AppState>,
    alert_group_id: String,
) -> AppResult<()> {
    let resultado = {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        existe_grupo(&conn, &alert_group_id)?;
        let resultado = crate::alerts::ciclo::reanudar_notificaciones(&conn, &alert_group_id)
            .map_err(rusqlite_err_to_app_error);
        if resultado.is_ok() {
            emitir_alerts_changed(&app, &conn, std::slice::from_ref(&alert_group_id));
        }
        resultado
    };
    crate::platform::bandeja::actualizar(&app);
    resultado
}

#[tauri::command]
pub fn archive_alert(
    app: tauri::AppHandle,
    state: State<AppState>,
    alert_group_id: String,
) -> AppResult<()> {
    let resultado = {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        existe_grupo(&conn, &alert_group_id)?;
        let ahora = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let resultado = crate::alerts::ciclo::archivar(&conn, &alert_group_id, &ahora)
            .map_err(rusqlite_err_to_app_error);
        if resultado.is_ok() {
            emitir_alerts_changed(&app, &conn, std::slice::from_ref(&alert_group_id));
        }
        resultado
    };
    // Archivar sí puede cambiar el color: era la peor alerta activa y deja de contar.
    crate::platform::bandeja::actualizar(&app);
    resultado
}

/// Espejo de `SystemEvent` en `docs/ui-contract.md` §3.5. Sin `ts-rs`: mezclarlo con
/// `api/generated/` rompería la separación ya establecida para `AlertGroup`/`SystemEvent`
/// (vocabulario compartido de `design/types.ts`), como razona `AlertGroupWire`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemEventWire {
    id: String,
    occurred_at: String,
    provider: String,
    event_id: i64,
    level: String,
    message: String,
    device_id: Option<String>,
    volume_id: Option<String>,
    mapping_confidence: MappingConfidence,
    has_raw_xml: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemEventPageWire {
    events: Vec<SystemEventWire>,
    next_cursor: Option<String>,
    total: Option<i64>,
}

/// El contrato solo admite tres niveles (`docs/ui-contract.md` §3.5): `Critical` se pliega en
/// `"error"`, no hay un cuarto nivel en la interfaz para distinguirlo.
fn nivel_a_wire(v: crate::domain::tipos::EventLevel) -> &'static str {
    use crate::domain::tipos::EventLevel;
    match v {
        EventLevel::Critical | EventLevel::Error => "error",
        EventLevel::Warning => "warning",
        EventLevel::Information => "info",
    }
}

fn nivel_desde_wire(s: &str) -> crate::domain::tipos::EventLevel {
    use crate::domain::tipos::EventLevel;
    match s {
        "error" => EventLevel::Error,
        "warning" => EventLevel::Warning,
        _ => EventLevel::Information,
    }
}

fn evento_a_wire(e: &crate::domain::tipos::SystemEvent) -> SystemEventWire {
    SystemEventWire {
        id: e.id.to_string(),
        occurred_at: e.occurred_at_utc.clone(),
        provider: e.provider.clone(),
        event_id: e.event_id,
        level: nivel_a_wire(e.level).to_string(),
        message: e.message.clone().unwrap_or_default(),
        device_id: e.device_id.clone(),
        volume_id: e.volume_id.clone(),
        mapping_confidence: e.mapping_confidence,
        has_raw_xml: e.raw_xml.is_some(),
    }
}

/// `"occurred_at_utc|id"`: par ordenable que identifica el último evento de una página
/// (paginación por conjunto de claves, `repo_varios::FiltroEventos.cursor`).
fn parsear_cursor(cursor: &str) -> Option<(String, i64)> {
    let (tiempo, id) = cursor.split_once('|')?;
    Some((tiempo.to_string(), id.parse().ok()?))
}

#[allow(clippy::too_many_arguments)]
fn get_system_events_impl(
    conn: &rusqlite::Connection,
    device_id: Option<&str>,
    volume_id: Option<&str>,
    levels: Option<&[String]>,
    providers: Option<&[String]>,
    from_utc: Option<&str>,
    to_utc: Option<&str>,
    cursor: Option<&str>,
    limit: Option<i64>,
) -> AppResult<SystemEventPageWire> {
    let niveles_dominio: Option<Vec<crate::domain::tipos::EventLevel>> =
        levels.map(|ls| ls.iter().map(|s| nivel_desde_wire(s)).collect());
    let cursor_parseado = cursor.and_then(parsear_cursor);
    let limite = limit.unwrap_or(200).clamp(1, 1000);

    let filtro = repo_varios::FiltroEventos {
        device_id,
        volume_id,
        levels: niveles_dominio.as_deref(),
        providers,
        from_utc,
        to_utc,
        cursor: cursor_parseado.as_ref().map(|(t, id)| (t.as_str(), *id)),
        limit: limite,
    };

    let mut filas = repo_varios::list_events(conn, &filtro).map_err(rusqlite_err_to_app_error)?;
    let hay_mas = filas.len() as i64 > limite;
    if hay_mas {
        filas.truncate(limite as usize);
    }
    let next_cursor = if hay_mas {
        filas
            .last()
            .map(|e| format!("{}|{}", e.occurred_at_utc, e.id))
    } else {
        None
    };
    let total = repo_varios::count_events(conn, &filtro)
        .map_err(rusqlite_err_to_app_error)
        .ok();

    Ok(SystemEventPageWire {
        events: filas.iter().map(evento_a_wire).collect(),
        next_cursor,
        total,
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn get_system_events(
    state: State<AppState>,
    device_id: Option<String>,
    volume_id: Option<String>,
    levels: Option<Vec<String>>,
    providers: Option<Vec<String>>,
    from_utc: Option<String>,
    to_utc: Option<String>,
    cursor: Option<String>,
    limit: Option<i64>,
) -> AppResult<SystemEventPageWire> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    get_system_events_impl(
        &conn,
        device_id.as_deref(),
        volume_id.as_deref(),
        levels.as_deref(),
        providers.as_deref(),
        from_utc.as_deref(),
        to_utc.as_deref(),
        cursor.as_deref(),
        limit,
    )
}

fn get_event_raw_xml_impl(conn: &rusqlite::Connection, event_id: &str) -> AppResult<String> {
    let id: i64 = event_id
        .parse()
        .map_err(|_| Box::new(AppError::new("event.not_found", "error.eventNotFound")))?;
    let evento = repo_varios::get_event_by_id(conn, id)
        .map_err(rusqlite_err_to_app_error)?
        .ok_or_else(|| Box::new(AppError::new("event.not_found", "error.eventNotFound")))?;
    evento
        .raw_xml
        .ok_or_else(|| Box::new(AppError::new("event.not_found", "error.eventNotFound")))
}

#[tauri::command]
pub fn get_event_raw_xml(state: State<AppState>, event_id: String) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    get_event_raw_xml_impl(&conn, &event_id)
}

// ---- Pruebas (T083, docs/ui-contract.md §3.6) ------------------------------------------------
//
// `test_runs` no tiene columna propia para `command`/`output`/`outputEncoding`
// (`docs/open-questions.md` J.29): viajan dentro de `parameters_json` y `result_summary_json`.
// `orphanPath` reutiliza `temp_path`: solo tiene valor mientras el archivo del benchmark sigue en
// disco, se limpia a `NULL` en cuanto el borrado tiene éxito.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParametrosPruebaJson {
    command: Option<String>,
    #[serde(default)]
    params: serde_json::Value,
}

impl Default for ParametrosPruebaJson {
    fn default() -> Self {
        Self {
            command: None,
            params: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResumenPruebaJson {
    passed: Option<bool>,
    read_bytes_per_second: Option<f64>,
    write_bytes_per_second: Option<f64>,
    read_latency_ms: Option<f64>,
    write_latency_ms: Option<f64>,
    max_temperature_c: Option<f64>,
    stopped_reason: Option<String>,
    output: Option<String>,
    output_encoding: Option<String>,
}

/// Espejo de `TestResult` en `src/lib/api/types.ts` (hoy declarado a mano allí; T084 lo sustituye
/// por la reexportación de este tipo generado, igual que ya se hizo con `MetricSeriesWire`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultWire {
    pub passed: Option<bool>,
    pub read_bytes_per_second: Option<f64>,
    pub write_bytes_per_second: Option<f64>,
    pub read_latency_ms: Option<f64>,
    pub write_latency_ms: Option<f64>,
    pub max_temperature_c: Option<f64>,
    pub stopped_reason: Option<String>,
}

/// Espejo de `TestRun` en `src/lib/api/types.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunWire {
    pub id: String,
    #[serde(rename = "type")]
    pub test_type: String,
    pub device_id: Option<String>,
    pub volume_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub progress_percent: Option<i64>,
    pub command: Option<String>,
    pub parameters: serde_json::Value,
    pub result: Option<TestResultWire>,
    pub output: Option<String>,
    pub output_encoding: Option<String>,
    pub orphan_path: Option<String>,
}

fn ahora_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Nanosegundos UTC en hexadecimal: identificador de `test_run` y sufijo del archivo del benchmark
/// (`docs/open-questions.md` J.29), sin añadir una dependencia de aleatoriedad. La unicidad real
/// la da `tests::rutas::confirmar_no_sobrescribe`, no la improbabilidad de colisión de este valor.
fn nuevo_id_prueba() -> String {
    format!(
        "{:x}",
        time::OffsetDateTime::now_utc().unix_timestamp_nanos()
    )
}

fn test_run_to_wire(t: &TestRun) -> TestRunWire {
    let envelope: ParametrosPruebaJson = t
        .parameters_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    let resumen: Option<ResumenPruebaJson> = t
        .result_summary_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    TestRunWire {
        id: t.id.clone(),
        test_type: repo_varios::test_type_to_str(t.test_type).to_string(),
        device_id: t.target_device_id.clone(),
        volume_id: t.target_volume_id.clone(),
        status: repo_varios::test_status_to_str(t.status).to_string(),
        started_at: t.started_at_utc.clone().unwrap_or_default(),
        finished_at: t.finished_at_utc.clone(),
        progress_percent: t.progress_percent,
        command: envelope.command,
        parameters: envelope.params,
        result: resumen.as_ref().map(|r| TestResultWire {
            passed: r.passed,
            read_bytes_per_second: r.read_bytes_per_second,
            write_bytes_per_second: r.write_bytes_per_second,
            read_latency_ms: r.read_latency_ms,
            write_latency_ms: r.write_latency_ms,
            max_temperature_c: r.max_temperature_c,
            stopped_reason: r.stopped_reason.clone(),
        }),
        output: resumen.as_ref().and_then(|r| r.output.clone()),
        output_encoding: resumen.as_ref().and_then(|r| r.output_encoding.clone()),
        orphan_path: t.temp_path.clone(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TestProgressEvent {
    emitted_at: String,
    test_run: TestRunWire,
}

fn emitir_test_progress(app: &tauri::AppHandle, conn: &rusqlite::Connection, test_run_id: &str) {
    let Ok(Some(t)) = repo_varios::get_test_run(conn, test_run_id) else {
        return;
    };
    if let Err(e) = app.emit(
        "test:progress",
        TestProgressEvent {
            emitted_at: ahora_rfc3339(),
            test_run: test_run_to_wire(&t),
        },
    ) {
        tracing::warn!(error = %e, "no se pudo emitir test:progress");
    }
}

/// Primera letra de unidad de un volumen ("C:", con los dos puntos, tal como se persiste), o
/// `None` si no tiene ninguna asignada — un volumen así no puede alojar el benchmark ni `chkdsk`.
fn primera_letra_unidad(v: &Volume) -> Option<char> {
    let letras: Vec<String> = v
        .drive_letters_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    letras.first().and_then(|s| s.chars().next())
}

/// `test.busy` (`docs/ui-contract.md` §1, `docs/open-questions.md` J.29): por disco físico
/// subyacente, no solo por el id exacto recibido — cubre a la vez la exclusión general y "el
/// autotest no corre junto al benchmark de la aplicación" (`product-specification.md` §6).
fn hay_prueba_activa(
    conn: &rusqlite::Connection,
    device_id: Option<&str>,
    volume_id: Option<&str>,
) -> AppResult<bool> {
    let mut device_ids: Vec<String> = device_id.map(str::to_string).into_iter().collect();
    let mut volume_ids: Vec<String> = volume_id.map(str::to_string).into_iter().collect();
    if let Some(id) = device_id {
        volume_ids.extend(
            repo_inventario::volumes_for_device(conn, id).map_err(rusqlite_err_to_app_error)?,
        );
    }
    if let Some(id) = volume_id {
        device_ids.extend(
            repo_inventario::devices_for_volume(conn, id).map_err(rusqlite_err_to_app_error)?,
        );
    }
    repo_varios::has_active_test_run(conn, &device_ids, &volume_ids)
        .map_err(rusqlite_err_to_app_error)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn start_benchmark(
    app: tauri::AppHandle,
    state: State<AppState>,
    volume_id: String,
    size_bytes: i64,
    block_size_bytes: i64,
    mode: String,
    passes: i64,
) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");

    let volumen = repo_inventario::get_volume(&conn, &volume_id)
        .map_err(rusqlite_err_to_app_error)?
        .ok_or_else(|| Box::new(AppError::new("volume.not_found", "error.volumeNotFound")))?;

    if hay_prueba_activa(&conn, None, Some(&volume_id))? {
        return Err(Box::new(AppError::new("test.busy", "error.testBusy")));
    }

    let letra = primera_letra_unidad(&volumen)
        .ok_or_else(|| Box::new(AppError::new("path.invalid", "error.pathInvalid")))?;

    let tamano_bytes = tests::rutas::resolver_tamano_bytes(
        size_bytes,
        volumen.free_bytes.unwrap_or(0),
        volumen.capacity_bytes.unwrap_or(0),
    )
    .map_err(|_| {
        Box::new(AppError::new(
            "test.insufficient_space",
            "error.testInsufficientSpace",
        ))
    })? as u64;

    let bloque_bytes = if block_size_bytes > 0 {
        block_size_bytes as u64
    } else {
        1024 * 1024
    };
    let pasadas = passes.clamp(1, 10) as u32;
    let modo = if mode == "random" {
        tests::benchmark::ModoAcceso::Random
    } else {
        tests::benchmark::ModoAcceso::Sequential
    };

    let raiz = std::path::PathBuf::from(format!("{letra}:\\"));
    let carpeta = tests::rutas::carpeta_benchmark(&raiz);
    if let Err(e) = std::fs::create_dir_all(&carpeta) {
        return Err(Box::new(
            AppError::new("test.io_failed", "error.testIoFailed").with_detail(e.to_string()),
        ));
    }

    let id = nuevo_id_prueba();
    let nombre_archivo = format!("benchmark-{id}.tmp");
    let ruta_archivo = tests::rutas::resolver_ruta_archivo(&raiz, &nombre_archivo)
        .map_err(|_| Box::new(AppError::new("path.invalid", "error.pathInvalid")))?;
    tests::rutas::confirmar_no_sobrescribe(&ruta_archivo)
        .map_err(|_| Box::new(AppError::new("path.invalid", "error.pathInvalid")))?;

    let ahora = ahora_rfc3339();
    let envelope = ParametrosPruebaJson {
        command: None,
        params: serde_json::json!({
            "sizeBytes": tamano_bytes,
            "blockSizeBytes": bloque_bytes,
            "mode": if modo == tests::benchmark::ModoAcceso::Random { "random" } else { "sequential" },
            "passes": pasadas,
        }),
    };
    let fila = TestRun {
        id: id.clone(),
        test_type: TestType::Benchmark,
        target_device_id: None,
        target_volume_id: Some(volume_id.clone()),
        status: TestStatus::Running,
        started_at_utc: Some(ahora),
        finished_at_utc: None,
        progress_percent: Some(0),
        result_summary_json: None,
        parameters_json: serde_json::to_string(&envelope).ok(),
        temp_path: Some(ruta_archivo.to_string_lossy().into_owned()),
    };
    repo_varios::create_test_run(&conn, &fila).map_err(rusqlite_err_to_app_error)?;
    emitir_test_progress(&app, &conn, &id);

    // El disco que respalda el volumen, para leer su temperatura durante la ejecución (guardia
    // térmica, `product-specification.md` §6). Puede no haber ninguno enlazado todavía: el
    // benchmark sigue adelante sin guardia térmica activa en ese caso, nunca se rechaza por esto.
    let device_id_para_temp = repo_inventario::devices_for_volume(&conn, &volume_id)
        .unwrap_or_default()
        .into_iter()
        .next();

    let bandera = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state
        .test_cancel_flags
        .lock()
        .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
        .insert(id.clone(), bandera.clone());

    drop(conn);

    let parametros = tests::benchmark::ParametrosBenchmark {
        size_bytes: tamano_bytes,
        block_size_bytes: bloque_bytes,
        mode: modo,
        passes: pasadas,
    };
    let limite_termico = tests::guardia::limite_critico_efectivo(None, 80.0);

    let app_temp = app.clone();
    let mut ultimo_chequeo_temp = std::time::Instant::now() - std::time::Duration::from_secs(10);
    let mut ultima_temp: Option<f64> = None;
    let leer_temperatura = move || -> Option<f64> {
        if ultimo_chequeo_temp.elapsed() >= std::time::Duration::from_secs(2) {
            ultimo_chequeo_temp = std::time::Instant::now();
            ultima_temp = app_temp
                .state::<AppState>()
                .conn
                .lock()
                .ok()
                .and_then(|conn| {
                    device_id_para_temp.as_deref().and_then(|id| {
                        repo_metricas::latest_device_sample(&conn, id, "temperature_celsius")
                            .ok()
                            .flatten()
                    })
                })
                .and_then(|m| m.value_real);
        }
        ultima_temp
    };

    let bandera_hilo = bandera.clone();
    let cancelado = move || bandera_hilo.load(std::sync::atomic::Ordering::Relaxed);

    let app_progreso = app.clone();
    let id_progreso = id.clone();
    let mut ultimo_progreso = std::time::Instant::now() - std::time::Duration::from_secs(10);
    let on_progreso = move |hecho: u64, total: u64| {
        if ultimo_progreso.elapsed() < std::time::Duration::from_millis(300) && hecho < total {
            return;
        }
        ultimo_progreso = std::time::Instant::now();
        let porcentaje = if total == 0 {
            100
        } else {
            ((hecho as f64 / total as f64) * 100.0).min(100.0) as i64
        };
        if let Ok(conn) = app_progreso.state::<AppState>().conn.lock() {
            let _ = repo_varios::update_test_run_status(
                &conn,
                &id_progreso,
                TestStatus::Running,
                Some(porcentaje),
            );
            emitir_test_progress(&app_progreso, &conn, &id_progreso);
        }
    };

    let app_final = app.clone();
    let id_final = id.clone();
    let ruta_hilo = ruta_archivo.clone();
    std::thread::spawn(move || {
        let resultado = tests::benchmark::ejecutar(
            &ruta_hilo,
            &parametros,
            limite_termico,
            cancelado,
            leer_temperatura,
            on_progreso,
        );

        // El archivo se borra siempre que se pueda, se sepa o no el resultado (product-specification.md
        // §6): si el borrado falla, `temp_path` queda con la ruta para limpieza manual posterior.
        let borrado_ok = std::fs::remove_file(&ruta_hilo).is_ok();
        let temp_path_final = if borrado_ok {
            None
        } else {
            Some(ruta_hilo.to_string_lossy().into_owned())
        };

        let (estado, resumen) = match resultado {
            Ok(r) => {
                let estado = match r.stopped_reason {
                    tests::benchmark::RazonParada::Completed => TestStatus::Completed,
                    tests::benchmark::RazonParada::Cancelled => TestStatus::Cancelled,
                    tests::benchmark::RazonParada::Thermal
                    | tests::benchmark::RazonParada::Space
                    | tests::benchmark::RazonParada::Error => TestStatus::Failed,
                };
                let stopped_reason = match r.stopped_reason {
                    tests::benchmark::RazonParada::Completed => "completed",
                    tests::benchmark::RazonParada::Cancelled => "cancelled",
                    tests::benchmark::RazonParada::Thermal => "thermal",
                    tests::benchmark::RazonParada::Space => "space",
                    tests::benchmark::RazonParada::Error => "error",
                };
                (
                    estado,
                    ResumenPruebaJson {
                        passed: r.passed,
                        read_bytes_per_second: r.read_bytes_per_second,
                        write_bytes_per_second: r.write_bytes_per_second,
                        read_latency_ms: r.read_latency_ms,
                        write_latency_ms: r.write_latency_ms,
                        max_temperature_c: r.max_temperature_c,
                        stopped_reason: Some(stopped_reason.to_string()),
                        output: None,
                        output_encoding: None,
                    },
                )
            }
            Err(e) => (
                TestStatus::Failed,
                ResumenPruebaJson {
                    stopped_reason: Some("error".to_string()),
                    output: Some(e.to_string()),
                    ..Default::default()
                },
            ),
        };

        if let Ok(conn) = app_final.state::<AppState>().conn.lock() {
            let _ = repo_varios::finish_test_run(
                &conn,
                &id_final,
                estado,
                &ahora_rfc3339(),
                serde_json::to_string(&resumen).ok().as_deref(),
                temp_path_final.as_deref(),
            );
            emitir_test_progress(&app_final, &conn, &id_final);
        }
        app_final
            .state::<AppState>()
            .test_cancel_flags
            .lock()
            .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
            .remove(&id_final);
    });

    Ok(id)
}

#[tauri::command]
pub fn run_chkdsk_scan(
    app: tauri::AppHandle,
    state: State<AppState>,
    volume_id: String,
) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");

    let volumen = repo_inventario::get_volume(&conn, &volume_id)
        .map_err(rusqlite_err_to_app_error)?
        .ok_or_else(|| Box::new(AppError::new("volume.not_found", "error.volumeNotFound")))?;

    if !tests::chkdsk::admite_scan(volumen.filesystem.as_deref()) {
        return Err(Box::new(AppError::new(
            "test.unsupported",
            "error.testUnsupported",
        )));
    }
    let letra = primera_letra_unidad(&volumen)
        .ok_or_else(|| Box::new(AppError::new("path.invalid", "error.pathInvalid")))?;

    if hay_prueba_activa(&conn, None, Some(&volume_id))? {
        return Err(Box::new(AppError::new("test.busy", "error.testBusy")));
    }

    let id = nuevo_id_prueba();
    let ahora = ahora_rfc3339();
    let envelope = ParametrosPruebaJson {
        command: Some(tests::chkdsk::comando_literal(letra)),
        params: serde_json::json!({}),
    };
    let fila = TestRun {
        id: id.clone(),
        test_type: TestType::ChkdskScan,
        target_device_id: None,
        target_volume_id: Some(volume_id.clone()),
        status: TestStatus::Running,
        started_at_utc: Some(ahora),
        finished_at_utc: None,
        progress_percent: None,
        result_summary_json: None,
        parameters_json: serde_json::to_string(&envelope).ok(),
        temp_path: None,
    };
    repo_varios::create_test_run(&conn, &fila).map_err(rusqlite_err_to_app_error)?;
    emitir_test_progress(&app, &conn, &id);

    let bandera = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state
        .test_cancel_flags
        .lock()
        .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
        .insert(id.clone(), bandera.clone());
    drop(conn);

    let app_hilo = app.clone();
    let id_hilo = id.clone();
    std::thread::spawn(move || {
        let cancelado = move || bandera.load(std::sync::atomic::Ordering::Relaxed);
        let resultado = tests::chkdsk::ejecutar_scan(letra, cancelado);

        let (estado, resumen) = match resultado {
            Ok(salida) => {
                let deteccion = tests::codificacion::detectar(&salida.stdout_crudo);
                let aprobado = salida.exit_status == 0;
                (
                    TestStatus::Completed,
                    ResumenPruebaJson {
                        passed: Some(aprobado),
                        stopped_reason: Some(
                            if aprobado { "completed" } else { "error" }.to_string(),
                        ),
                        output: Some(deteccion.texto),
                        output_encoding: Some(deteccion.codificacion),
                        ..Default::default()
                    },
                )
            }
            Err(tests::chkdsk::ErrorChkdsk::Cancelado) => (
                TestStatus::Cancelled,
                ResumenPruebaJson {
                    stopped_reason: Some("cancelled".to_string()),
                    ..Default::default()
                },
            ),
            Err(tests::chkdsk::ErrorChkdsk::Io(e)) => (
                TestStatus::Failed,
                ResumenPruebaJson {
                    stopped_reason: Some("error".to_string()),
                    output: Some(e.to_string()),
                    ..Default::default()
                },
            ),
        };

        if let Ok(conn) = app_hilo.state::<AppState>().conn.lock() {
            let _ = repo_varios::finish_test_run(
                &conn,
                &id_hilo,
                estado,
                &ahora_rfc3339(),
                serde_json::to_string(&resumen).ok().as_deref(),
                None,
            );
            emitir_test_progress(&app_hilo, &conn, &id_hilo);
        }
        app_hilo
            .state::<AppState>()
            .test_cancel_flags
            .lock()
            .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
            .remove(&id_hilo);
    });

    Ok(id)
}

/// Arranca el autotest corto y sondea su estado cada 5 s hasta que `smartctl` informe de un
/// resultado final (`tests::autotest::parse_estado_json`, **sin verificar contra hardware real**,
/// `docs/open-questions.md` J.28). Límite de sondeo de 30 min como salvaguarda defensiva: un
/// autotest corto real dura minutos, nunca debería alcanzarlo, pero un hilo que sondea para
/// siempre si el disco no informa nunca de un final sí sería un fallo real.
fn ejecutar_autotest_corto(
    ruta_dispositivo: &str,
    mut cancelado: impl FnMut() -> bool,
    mut on_progreso: impl FnMut(i64),
) -> (TestStatus, ResumenPruebaJson) {
    let ejecutable = crate::collectors::smartctl::resolve_smartctl_path();
    if let Err(e) = std::process::Command::new(&ejecutable)
        .args(tests::autotest::argumentos_iniciar(ruta_dispositivo))
        .output()
    {
        return (
            TestStatus::Failed,
            ResumenPruebaJson {
                stopped_reason: Some("error".to_string()),
                output: Some(e.to_string()),
                ..Default::default()
            },
        );
    }

    let inicio = std::time::Instant::now();
    let mut minutos_estimados: Option<i64> = None;
    loop {
        if cancelado() {
            let _ = std::process::Command::new(&ejecutable)
                .args(tests::autotest::argumentos_cancelar(ruta_dispositivo))
                .output();
            return (
                TestStatus::Cancelled,
                ResumenPruebaJson {
                    stopped_reason: Some("cancelled".to_string()),
                    ..Default::default()
                },
            );
        }
        if let Ok(json) = crate::collectors::smartctl::query_device_json(ruta_dispositivo) {
            if let Some(estado) = tests::autotest::parse_estado_json(&json) {
                if minutos_estimados.is_none() {
                    minutos_estimados = estado.minutos_estimados;
                }
                if let Some(aprobado) = estado.passed {
                    return (
                        TestStatus::Completed,
                        ResumenPruebaJson {
                            passed: Some(aprobado),
                            stopped_reason: Some(
                                if aprobado { "completed" } else { "error" }.to_string(),
                            ),
                            output: estado.descripcion,
                            ..Default::default()
                        },
                    );
                }
                if let Some(mins) = minutos_estimados {
                    let total_segundos = (mins.max(1) as f64) * 60.0;
                    let fraccion = (inicio.elapsed().as_secs_f64() / total_segundos).min(0.99);
                    on_progreso((fraccion * 100.0) as i64);
                }
            }
        }
        if inicio.elapsed() >= std::time::Duration::from_secs(30 * 60) {
            return (
                TestStatus::Failed,
                ResumenPruebaJson {
                    stopped_reason: Some("error".to_string()),
                    output: Some(
                        "El autotest no informó de un resultado final dentro del tiempo máximo de sondeo."
                            .to_string(),
                    ),
                    ..Default::default()
                },
            );
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[tauri::command]
pub fn run_smart_short_test(
    app: tauri::AppHandle,
    state: State<AppState>,
    device_id: String,
) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");

    let dispositivo = repo_inventario::get_device(&conn, &device_id)
        .map_err(rusqlite_err_to_app_error)?
        .ok_or_else(|| Box::new(AppError::new("device.not_found", "error.deviceNotFound")))?;

    if !tests::autotest::admite_autotest(dispositivo.smartctl_path.as_deref()) {
        return Err(Box::new(AppError::new(
            "test.unsupported",
            "error.testUnsupported",
        )));
    }
    if hay_prueba_activa(&conn, Some(&device_id), None)? {
        return Err(Box::new(AppError::new("test.busy", "error.testBusy")));
    }

    let ruta_dispositivo = dispositivo
        .smartctl_path
        .clone()
        .expect("admite_autotest ya comprobó que existe");

    let id = nuevo_id_prueba();
    let ahora = ahora_rfc3339();
    let comando = format!(
        "smartctl.exe {}",
        tests::autotest::argumentos_iniciar(&ruta_dispositivo).join(" ")
    );
    let envelope = ParametrosPruebaJson {
        command: Some(comando),
        params: serde_json::json!({}),
    };
    let fila = TestRun {
        id: id.clone(),
        test_type: TestType::SmartShort,
        target_device_id: Some(device_id.clone()),
        target_volume_id: None,
        status: TestStatus::Running,
        started_at_utc: Some(ahora),
        finished_at_utc: None,
        progress_percent: Some(0),
        result_summary_json: None,
        parameters_json: serde_json::to_string(&envelope).ok(),
        temp_path: None,
    };
    repo_varios::create_test_run(&conn, &fila).map_err(rusqlite_err_to_app_error)?;
    emitir_test_progress(&app, &conn, &id);

    let bandera = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state
        .test_cancel_flags
        .lock()
        .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
        .insert(id.clone(), bandera.clone());
    drop(conn);

    let app_hilo = app.clone();
    let id_hilo = id.clone();
    std::thread::spawn(move || {
        let app_progreso = app_hilo.clone();
        let id_progreso = id_hilo.clone();
        let on_progreso = move |porcentaje: i64| {
            if let Ok(conn) = app_progreso.state::<AppState>().conn.lock() {
                let _ = repo_varios::update_test_run_status(
                    &conn,
                    &id_progreso,
                    TestStatus::Running,
                    Some(porcentaje),
                );
                emitir_test_progress(&app_progreso, &conn, &id_progreso);
            }
        };
        let cancelado = move || bandera.load(std::sync::atomic::Ordering::Relaxed);
        let (estado, resumen) = ejecutar_autotest_corto(&ruta_dispositivo, cancelado, on_progreso);

        if let Ok(conn) = app_hilo.state::<AppState>().conn.lock() {
            let _ = repo_varios::finish_test_run(
                &conn,
                &id_hilo,
                estado,
                &ahora_rfc3339(),
                serde_json::to_string(&resumen).ok().as_deref(),
                None,
            );
            emitir_test_progress(&app_hilo, &conn, &id_hilo);
        }
        app_hilo
            .state::<AppState>()
            .test_cancel_flags
            .lock()
            .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
            .remove(&id_hilo);
    });

    Ok(id)
}

#[tauri::command]
pub fn cancel_test(
    app: tauri::AppHandle,
    state: State<AppState>,
    test_run_id: String,
) -> AppResult<()> {
    let bandera = state
        .test_cancel_flags
        .lock()
        .expect("el mutex de cancelación no se envenena: sin pánicos dentro")
        .get(&test_run_id)
        .cloned();
    // Una cancelación sobre una prueba que ya ha terminado (o que nunca existió) no es un error:
    // es una carrera benigna entre la UI y el hilo que la ejecutaba, se ignora en silencio.
    if let Some(bandera) = bandera {
        bandera.store(true, std::sync::atomic::Ordering::Relaxed);
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        let progreso_actual = repo_varios::get_test_run(&conn, &test_run_id)
            .ok()
            .flatten()
            .and_then(|t| t.progress_percent);
        let _ = repo_varios::update_test_run_status(
            &conn,
            &test_run_id,
            TestStatus::Cancelling,
            progreso_actual,
        );
        emitir_test_progress(&app, &conn, &test_run_id);
    }
    Ok(())
}

#[tauri::command]
pub fn get_test_runs(
    state: State<AppState>,
    device_id: Option<String>,
    limit: Option<i64>,
) -> AppResult<Vec<TestRunWire>> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let limite = limit.unwrap_or(50).clamp(1, 500);
    let filas = repo_varios::list_test_runs(&conn, device_id.as_deref(), limite)
        .map_err(rusqlite_err_to_app_error)?;
    Ok(filas.iter().map(test_run_to_wire).collect())
}

#[tauri::command]
pub fn export_report(format: String) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented(&format!(
        "export_report({format})"
    ))))
}

#[tauri::command]
pub fn preview_diagnostic_zip(_include_identifiers: bool) -> AppResult<serde_json::Value> {
    Err(Box::new(AppError::not_implemented(
        "preview_diagnostic_zip",
    )))
}

#[tauri::command]
pub fn create_diagnostic_zip(
    _include_identifiers: bool,
    _destination_path: String,
) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented("create_diagnostic_zip")))
}

#[tauri::command]
pub fn pause_monitoring(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    *state
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro") = Some(ahora);
    crate::platform::bandeja::actualizar(&app);
    Ok(())
}

#[tauri::command]
pub fn resume_monitoring(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    *state
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro") = None;
    crate::platform::bandeja::actualizar(&app);
    Ok(())
}

/// Nombre y versión salen del manifiesto de compilación, nunca de un literal duplicado (ADR-011).
#[tauri::command]
pub fn get_app_info() -> AppResult<AppInfo> {
    Ok(AppInfo {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        author: env!("CARGO_PKG_AUTHORS").into(),
    })
}

#[tauri::command]
pub fn delete_all_data(_confirmation_phrase: String) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented("delete_all_data")))
}

/* ------------------------------------------------------------------ registro */

/// Recibe del frontend las entradas que deben quedar en el fichero.
///
/// Solo llegan `warn` y `error` (constitución §XV): `debug` e `info` se quedan en la consola del
/// WebView, porque enviarlos por IPC costaría más que el valor que aportan. Así un fallo de
/// interfaz en el equipo de un usuario aparece en el ZIP de diagnóstico sin pedirle que abra las
/// herramientas de desarrollo.
#[tauri::command]
pub fn log_from_ui(
    level: String,
    scope: String,
    message: String,
    context: Option<serde_json::Value>,
) -> AppResult<()> {
    let ctx = context
        .as_ref()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "-".to_owned());

    match level.as_str() {
        "error" => tracing::error!(target: "ui", scope, contexto = %ctx, "{message}"),
        "warn" => tracing::warn!(target: "ui", scope, contexto = %ctx, "{message}"),
        // Un nivel inesperado se registra igual, pero degradado: perder la entrada sería peor.
        other => {
            tracing::info!(target: "ui", scope, nivel_recibido = other, contexto = %ctx, "{message}")
        }
    }
    Ok(())
}

/// Nivel efectivo, para que el frontend filtre igual que el backend y no envíe lo que se
/// descartaría de todos modos.
#[tauri::command]
pub fn get_log_level() -> AppResult<String> {
    // De momento solo manda la línea de órdenes; cuando exista `settings`, esta función devolverá
    // el valor resuelto con la precedencia completa (§XV).
    let args: Vec<String> = std::env::args().collect();
    let level = crate::logging::level_from_cli(&args).unwrap_or(crate::logging::LogLevel::Info);
    Ok(serde_json::to_value(level)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "info".to_owned()))
}

#[cfg(test)]
mod tests_inventario {
    use super::*;
    use crate::collectors::capacidad::VolumenLeido;
    use crate::collectors::windows_storage::DiscoFisico;
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_inventario");
        db::open(&dir).unwrap().0
    }

    fn disco_leido(model: &str, serial: Option<&str>) -> DiscoFisico {
        DiscoFisico {
            model: model.to_string(),
            manufacturer: Some("Fabricante".to_string()),
            serial_number: serial.map(str::to_string),
            firmware: Some("1.0".to_string()),
            capacity_bytes: Some(1_000_000_000),
            bus_type: "NVMe".to_string(),
            device_type: DeviceType::Nvme,
            wwn_o_pnp_device_id: format!("pnp-{model}"),
            smartctl_device_path: Some(r"\\.\PhysicalDrive0".to_string()),
        }
    }

    fn volumen_leido(guid: &str, disk_number: Option<i64>) -> VolumenLeido {
        VolumenLeido {
            volume_guid: guid.to_string(),
            label: Some("Datos".to_string()),
            filesystem: Some("NTFS".to_string()),
            drive_letter: Some('C'),
            capacity_bytes: Some(500_000_000),
            free_bytes: Some(200_000_000),
            disk_number,
        }
    }

    #[test]
    fn reconciliar_da_de_alta_un_disco_nuevo() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert_eq!(respuesta.devices.len(), 1);
        assert_eq!(respuesta.devices[0].model, "Disco A");
        assert_eq!(respuesta.devices[0].state, HealthState::Unknown);
        assert_eq!(
            respuesta.devices[0].unknown_reason,
            Some(UnknownReason::NotYetSampled)
        );
    }

    #[test]
    fn reconciliar_dos_veces_no_duplica_el_mismo_disco() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:01:00Z",
        )
        .unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert_eq!(
            respuesta.devices.len(),
            1,
            "la misma huella no crea una segunda entidad"
        );
    }

    #[test]
    fn un_disco_que_desaparece_se_marca_retirado_y_deja_de_listarse() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();
        reconciliar_inventario(&conn, &[], &[], "2026-09-04T10:05:00Z").unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert!(respuesta.devices.is_empty());
    }

    #[test]
    fn un_disco_retirado_que_vuelve_continua_su_historial() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();
        let id_original = get_devices_impl(&conn).unwrap().devices[0].id.clone();

        reconciliar_inventario(&conn, &[], &[], "2026-09-04T10:05:00Z").unwrap();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:10:00Z",
        )
        .unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert_eq!(respuesta.devices.len(), 1);
        assert_eq!(
            respuesta.devices[0].id, id_original,
            "reconecta el mismo id, no crea uno nuevo"
        );
    }

    #[test]
    fn un_disco_sin_numero_de_serie_no_rompe_la_reconciliacion() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco USB", None)],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert_eq!(respuesta.devices.len(), 1);
    }

    #[test]
    fn excluir_de_la_monitorizacion_lo_mueve_a_excluded() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();
        let id = get_devices_impl(&conn).unwrap().devices[0].id.clone();

        set_device_monitoring_impl(&conn, &id, false).unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert!(respuesta.devices.is_empty());
        assert_eq!(respuesta.excluded.len(), 1);
    }

    #[test]
    fn operar_sobre_un_disco_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        let err = set_device_monitoring_impl(&conn, "no-existe", true).unwrap_err();
        assert_eq!(err.code, "device.not_found");
    }

    #[test]
    fn poner_un_alias_se_refleja_en_el_resumen() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();
        let id = get_devices_impl(&conn).unwrap().devices[0].id.clone();

        set_device_alias_impl(&conn, &id, Some("Mi disco")).unwrap();

        let detalle = get_device_detail_impl(&conn, &id).unwrap();
        assert_eq!(detalle.summary.alias.as_deref(), Some("Mi disco"));
    }

    #[test]
    fn un_volumen_con_el_mismo_numero_de_disco_se_enlaza_con_confianza_exacta() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[volumen_leido("vol-1", Some(0))],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert_eq!(respuesta.devices[0].volumes.len(), 1);
        let volumen = &respuesta.devices[0].volumes[0];
        assert_eq!(volumen.label, "Datos");
        assert_eq!(volumen.drive_letters, vec!["C:".to_string()]);
        assert_eq!(volumen.capacity_bytes, Some(500_000_000));
        assert_eq!(volumen.mapping_confidence, MappingConfidence::Exact);
    }

    #[test]
    fn un_volumen_sin_disco_correspondiente_no_se_enlaza_a_nadie() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[volumen_leido("vol-huerfano", Some(99))],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();

        let respuesta = get_devices_impl(&conn).unwrap();
        assert!(respuesta.devices[0].volumes.is_empty());
        // El volumen sigue persistido, solo que sin ningún dispositivo enlazado.
        assert_eq!(repo_inventario::list_volumes(&conn).unwrap().len(), 1);
    }

    #[test]
    fn reconciliar_volumenes_dos_veces_actualiza_el_espacio_libre_sin_duplicar() {
        let conn = conn_de_prueba();
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[volumen_leido("vol-1", Some(0))],
            "2026-09-04T10:00:00Z",
        )
        .unwrap();
        let mut segunda_lectura = volumen_leido("vol-1", Some(0));
        segunda_lectura.free_bytes = Some(100_000_000);
        reconciliar_inventario(
            &conn,
            &[disco_leido("Disco A", Some("S1"))],
            &[segunda_lectura],
            "2026-09-04T10:05:00Z",
        )
        .unwrap();

        assert_eq!(repo_inventario::list_volumes(&conn).unwrap().len(), 1);
        let respuesta = get_devices_impl(&conn).unwrap();
        assert_eq!(
            respuesta.devices[0].volumes[0].free_bytes,
            Some(100_000_000)
        );
    }
}

#[cfg(test)]
mod tests_salud {
    use super::*;
    use crate::collectors::smartctl_parser::{parse_smartctl_json, MetricaLeida, SmartctlResult};
    use crate::domain::tipos::{DeviceType, HealthState, IdentityConfidence, UnknownReason};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_salud");
        db::open(&dir).unwrap().0
    }

    fn dispositivo_con_ruta(id: &str, smartctl_path: Option<&str>) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: None,
            model: "Modelo".to_string(),
            manufacturer: None,
            firmware: None,
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: smartctl_path.map(str::to_string),
            capacity_bytes: Some(1_000_000_000),
            alias: None,
            monitoring_enabled: true,
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    fn resultado_de_prueba(temperatura: f64) -> SmartctlResult {
        SmartctlResult {
            exit_status: 0,
            smartctl_version: Some("7.5".to_string()),
            device_type: Some("nvme".to_string()),
            model_name: Some("Modelo".to_string()),
            serial_number: Some("S1".to_string()),
            firmware_version: Some("1.0".to_string()),
            user_capacity_bytes: Some(1_000_000_000),
            health_passed: Some(true),
            metrics: vec![MetricaLeida {
                metric_key: "temperature_celsius",
                value: temperatura,
            }],
        }
    }

    #[test]
    fn sin_ruta_de_smartctl_el_disco_queda_desconocido_por_no_soportado() {
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", None);
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let resumen = get_device_detail_impl(&conn, "d1").unwrap();

        assert_eq!(resumen.summary.state, HealthState::Unknown);
        assert_eq!(
            resumen.summary.unknown_reason,
            Some(UnknownReason::NotYetSampled)
        );
        assert_eq!(
            resumen.summary.temperature_c, None,
            "sin ruta no se inventa un valor"
        );
        assert_eq!(
            resumen.capabilities,
            vec![
                DeviceCapability {
                    key: DeviceCapabilityKey::Smart,
                    available: false,
                    reason_key: Some("disk.noSmartData".to_string()),
                },
                DeviceCapability {
                    key: DeviceCapabilityKey::SelfTestShort,
                    available: false,
                    reason_key: Some("disk.noSmartData".to_string()),
                },
                DeviceCapability {
                    key: DeviceCapabilityKey::ChkdskScan,
                    available: false,
                    reason_key: Some("tests.chkdskUnsupportedReason".to_string()),
                },
            ]
        );
    }

    #[test]
    fn con_ruta_pero_sin_lectura_previa_sigue_sin_muestrear() {
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let resumen = get_device_detail_impl(&conn, "d1").unwrap();

        assert_eq!(resumen.summary.state, HealthState::Unknown);
        assert_eq!(
            resumen.summary.unknown_reason,
            Some(UnknownReason::NotYetSampled)
        );
    }

    #[test]
    fn una_lectura_reciente_persistida_se_refleja_como_correcto_con_dato_real() {
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let ahora = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        persist_smart_reading(&conn, "d1", &resultado_de_prueba(42.0), &ahora).unwrap();

        let resumen = get_device_detail_impl(&conn, "d1").unwrap();

        assert_eq!(resumen.summary.state, HealthState::Ok);
        assert_eq!(resumen.summary.unknown_reason, None);
        assert_eq!(resumen.summary.temperature_c, Some(42.0));
        assert!(resumen.summary.provenance.is_some());
        assert_eq!(
            resumen.summary.provenance.unwrap().source,
            MetricSource::Smartctl
        );
        assert!(
            resumen.capabilities[0].available,
            "con ruta, la capacidad smart está disponible"
        );
        assert_eq!(
            resumen.counters.len(),
            1,
            "una muestra persistida produce un contador"
        );
        assert_eq!(resumen.counters[0].metric_key, "temperature_celsius");
    }

    #[test]
    fn el_autotest_sigue_a_la_ruta_de_smartctl_y_chkdsk_a_un_volumen_ntfs_enlazado() {
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let sin_volumenes = get_device_detail_impl(&conn, "d1").unwrap();
        assert!(
            sin_volumenes.capabilities[1].available,
            "con ruta de smartctl, el autotest corto se ofrece"
        );
        assert!(
            !sin_volumenes.capabilities[2].available,
            "sin ningún volumen enlazado, chkdsk no tiene dónde ejecutarse"
        );

        let volumen = Volume {
            id: "v1".to_string(),
            volume_guid: "guid-v1".to_string(),
            label: None,
            filesystem: Some("NTFS".to_string()),
            drive_letters_json: Some(r#"["D:"]"#.to_string()),
            capacity_bytes: Some(1_000_000_000),
            free_bytes: Some(500_000_000),
            device_mapping_confidence: Some(MappingConfidence::Exact),
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
        };
        repo_inventario::upsert_volume(&conn, &volumen).unwrap();
        repo_inventario::link_device_volume(&conn, "d1", "v1", MappingConfidence::Exact, "test")
            .unwrap();

        let con_volumen_ntfs = get_device_detail_impl(&conn, "d1").unwrap();
        assert!(
            con_volumen_ntfs.capabilities[2].available,
            "con un volumen NTFS enlazado, chkdsk se ofrece"
        );
    }

    #[test]
    fn una_lectura_caducada_vuelve_a_quedar_desconocida_sin_borrar_el_dato() {
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        // Una lectura de hace dos días, muy por encima del margen de frescura (10 minutos).
        let hace_dos_dias = (time::OffsetDateTime::now_utc() - time::Duration::days(2))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        persist_smart_reading(&conn, "d1", &resultado_de_prueba(40.0), &hace_dos_dias).unwrap();

        let resumen = get_device_detail_impl(&conn, "d1").unwrap();

        assert_eq!(
            resumen.summary.state,
            HealthState::Unknown,
            "un dato viejo no puede decir que todo va bien"
        );
        assert_eq!(
            resumen.summary.unknown_reason,
            Some(UnknownReason::NotYetSampled)
        );
        // El dato sigue disponible para mostrarlo con su antigüedad, aunque el estado sea unknown.
        assert_eq!(resumen.summary.temperature_c, Some(40.0));
        assert_eq!(
            resumen.summary.last_read_at.as_deref(),
            Some(hace_dos_dias.as_str())
        );
    }

    #[test]
    fn persist_smart_reading_guarda_una_instantanea_ademas_de_las_muestras() {
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        persist_smart_reading(
            &conn,
            "d1",
            &resultado_de_prueba(35.0),
            "2026-09-04T10:00:00Z",
        )
        .unwrap();

        let cuenta_snapshots: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM smart_snapshots WHERE device_id = 'd1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cuenta_snapshots, 1);

        let ultima = repo_metricas::latest_device_sample(&conn, "d1", "temperature_celsius")
            .unwrap()
            .unwrap();
        assert_eq!(ultima.value_real, Some(35.0));
    }

    #[test]
    fn el_json_de_smartctl_de_fixture_se_parsea_y_persiste_de_extremo_a_extremo() {
        // No es una invocación real de smartctl (eso necesita hardware); confirma que el parser
        // (T031) y la persistencia (este módulo) encajan con la misma forma de datos.
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let json = r#"{
            "smartctl": { "version": [7, 5], "exit_status": 0 },
            "device": { "type": "nvme" },
            "temperature": { "current": 38 },
            "power_on_time": { "hours": 500 },
            "smart_status": { "passed": true }
        }"#;
        let resultado = parse_smartctl_json(json).unwrap();
        persist_smart_reading(&conn, "d1", &resultado, "2026-09-04T10:00:00Z").unwrap();

        let resumen = get_device_detail_impl(&conn, "d1").unwrap();
        assert_eq!(resumen.summary.temperature_c, Some(38.0));
        assert_eq!(resumen.summary.power_on_hours, Some(500.0));
    }
}

#[cfg(test)]
mod tests_refresh_now {
    use super::*;
    use crate::domain::tipos::{DeviceType, IdentityConfidence};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_refresh_now");
        db::open(&dir).unwrap().0
    }

    fn dispositivo(id: &str) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: None,
            model: "Modelo".to_string(),
            manufacturer: None,
            firmware: None,
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            // Sin ruta: la consulta real a smartctl (que necesita hardware) nunca se intenta, y
            // estas pruebas verifican solo el filtrado y la validación previas a esa consulta.
            smartctl_path: None,
            capacity_bytes: Some(1_000_000_000),
            alias: None,
            monitoring_enabled: true,
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    /// El contrato normativo (`docs/ui-contract.md` §3.2) es `scope: "all" | "device"`. Cualquier
    /// otro valor —incluidos `"inventory"` y `"smart"`, que este módulo usó por error antes de
    /// esta corrección— debe seguir sin implementarse en voz alta, nunca en silencio.
    #[test]
    fn un_ambito_fuera_del_contrato_normativo_no_se_implementa_en_silencio() {
        fn clasificar(scope: &str) -> Result<(), ()> {
            match scope {
                "all" | "device" => Ok(()),
                _ => Err(()),
            }
        }
        for ambito_invalido in ["inventory", "smart", "metrics", "events", ""] {
            assert!(
                clasificar(ambito_invalido).is_err(),
                "«{ambito_invalido}» no es del contrato"
            );
        }
        assert!(clasificar("all").is_ok());
        assert!(clasificar("device").is_ok());
    }

    #[test]
    fn refresh_smart_para_un_dispositivo_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        let err = refresh_smart(&conn, Some("no-existe")).unwrap_err();
        assert_eq!(err.code, "device.not_found");
    }

    #[test]
    fn refresh_smart_sin_filtro_recorre_todos_los_dispositivos_monitorizados() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        repo_inventario::upsert_device(&conn, &dispositivo("d2")).unwrap();

        // Ninguno tiene smartctl_path: el bucle los recorre y los salta sin invocar nada externo.
        assert!(refresh_smart(&conn, None).is_ok());
    }

    #[test]
    fn refresh_smart_con_filtro_no_falla_si_el_dispositivo_existe() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        repo_inventario::upsert_device(&conn, &dispositivo("d2")).unwrap();

        assert!(refresh_smart(&conn, Some("d1")).is_ok());
    }

    #[test]
    fn un_dispositivo_excluido_de_la_monitorizacion_no_detiene_el_ambito_all() {
        let conn = conn_de_prueba();
        let mut excluido = dispositivo("d1");
        excluido.monitoring_enabled = false;
        repo_inventario::upsert_device(&conn, &excluido).unwrap();

        assert!(refresh_smart(&conn, None).is_ok());
    }
}

#[cfg(test)]
mod tests_comandos_alertas {
    use super::*;
    use crate::alerts::agrupacion::{procesar, EvaluacionAlerta};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_alertas");
        let conn = db::open(&dir).unwrap().0;
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, alias, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, 'Mi disco', '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        conn
    }

    fn crear_grupo_de_prueba(conn: &rusqlite::Connection) -> String {
        let ev = EvaluacionAlerta {
            rule_key: "temp.above_configured_warn".to_string(),
            target_device_id: Some("d1".to_string()),
            target_volume_id: None,
            context: None,
            severity_si_activa: Some(AlertSeverity::Warning),
            resuelto: false,
            value: Some(75.0),
            occurred_at_utc: "2026-09-04T10:00:00Z".to_string(),
        };
        procesar(conn, &ev).unwrap();
        repo_alertas::list_groups(conn).unwrap()[0].id.clone()
    }

    #[test]
    fn get_alert_groups_usa_el_alias_del_disco_como_target() {
        let conn = conn_de_prueba();
        crear_grupo_de_prueba(&conn);

        let grupos = get_alert_groups_impl(&conn, None, None).unwrap();
        assert_eq!(grupos.len(), 1);
        assert_eq!(grupos[0].target, "Mi disco");
        assert_eq!(grupos[0].severity, AlertSeverity::Warning);
    }

    #[test]
    fn get_alert_groups_filtra_por_estado() {
        let conn = conn_de_prueba();
        crear_grupo_de_prueba(&conn);

        let activas = get_alert_groups_impl(&conn, Some(&[AlertStatus::Active]), None).unwrap();
        assert_eq!(activas.len(), 1);

        let archivadas =
            get_alert_groups_impl(&conn, Some(&[AlertStatus::Archived]), None).unwrap();
        assert!(archivadas.is_empty());
    }

    #[test]
    fn get_alert_groups_filtra_por_dispositivo() {
        let conn = conn_de_prueba();
        crear_grupo_de_prueba(&conn);

        assert_eq!(
            get_alert_groups_impl(&conn, None, Some("d1"))
                .unwrap()
                .len(),
            1
        );
        assert!(get_alert_groups_impl(&conn, None, Some("otro-disco"))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn get_alert_detail_incluye_la_clave_de_regla_como_hecho() {
        let conn = conn_de_prueba();
        let id = crear_grupo_de_prueba(&conn);

        let detalle = get_alert_detail_impl(&conn, &id).unwrap();
        assert_eq!(detalle.group.rule_key, "temp.above_configured_warn");
        assert!(detalle
            .facts
            .iter()
            .any(|f| f.value.as_deref() == Some("temp.above_configured_warn")));
        assert!(
            detalle.related_events.is_empty(),
            "sin colector de eventos, la lista es honesta: vacía"
        );
        assert_eq!(
            detalle.occurrences.len(),
            1,
            "el primer episodio ya tiene su fila en la cronología (no una lista vacía)"
        );
        assert_eq!(detalle.occurrences[0].value, Some(75.0));
    }

    #[test]
    fn get_alert_detail_de_un_id_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        let err = get_alert_detail_impl(&conn, "no-existe").unwrap_err();
        assert_eq!(err.code, "device.not_found");
    }

    #[test]
    fn acknowledge_alert_no_cambia_la_severidad_ni_lo_saca_de_la_lista_de_activas() {
        let conn = conn_de_prueba();
        let id = crear_grupo_de_prueba(&conn);

        existe_grupo(&conn, &id).unwrap();
        crate::alerts::ciclo::reconocer(&conn, &id, "2026-09-04T11:00:00Z").unwrap();

        let grupo = repo_alertas::get_group(&conn, &id).unwrap().unwrap();
        assert_eq!(grupo.status, AlertStatus::Acknowledged);
        assert_eq!(
            repo_alertas::list_groups_counting_toward_health(&conn)
                .unwrap()
                .len(),
            1,
            "reconocida sigue contando para la salud"
        );
    }

    #[test]
    fn una_accion_sobre_un_grupo_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        assert_eq!(
            existe_grupo(&conn, "no-existe").unwrap_err().code,
            "device.not_found"
        );
    }
}
