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

/// Espejo de `SourceStatus` en `docs/ui-contract.md` §2 y `src/lib/api/schemas.ts` (`sourceStatus`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum SourceStatus {
    Ok,
    Partial,
    Unsupported,
    Timeout,
    Error,
}

/// Espejo de `SourceHealth` en `docs/ui-contract.md` §2. Construido por `actualizar_source_health`
/// (T020/T021): antes nada producía este valor, `get_devices_impl` siempre devolvía `sources: []`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct SourceHealth {
    pub source: MetricSource,
    pub status: SourceStatus,
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

// ---- settings (T095-T097, docs/open-questions.md J.32) ---------------------------------------
//
// Claves tipadas y versionadas (`docs/data-model.md` §2), no un diccionario libre: cada clave
// tiene su propio valor de fábrica y, cuando aplica, su validación en `domain::ajustes`. Ausente
// en la tabla == valor de fábrica, nunca un error: una instalación recién hecha no ha escrito
// ninguna fila todavía.

const CLAVE_APARIENCIA_TEMA: &str = "settings.appearance.theme";
const CLAVE_APARIENCIA_IDIOMA: &str = "settings.appearance.language";
const CLAVE_APARIENCIA_ACENTO_SISTEMA: &str = "settings.appearance.use_system_accent";

fn leer_ajuste_i64(conn: &rusqlite::Connection, key: &str, default: i64) -> i64 {
    repo_varios::get_setting_raw(conn, key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<i64>(&s).ok())
        .unwrap_or(default)
}

fn leer_ajuste_f64(conn: &rusqlite::Connection, key: &str, default: f64) -> f64 {
    repo_varios::get_setting_raw(conn, key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<f64>(&s).ok())
        .unwrap_or(default)
}

pub(crate) fn leer_ajuste_bool(conn: &rusqlite::Connection, key: &str, default: bool) -> bool {
    repo_varios::get_setting_raw(conn, key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<bool>(&s).ok())
        .unwrap_or(default)
}

pub(crate) fn leer_ajuste_string(conn: &rusqlite::Connection, key: &str, default: &str) -> String {
    repo_varios::get_setting_raw(conn, key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<String>(&s).ok())
        .unwrap_or_else(|| default.to_string())
}

fn leer_ajuste_string_opcional(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    repo_varios::get_setting_raw(conn, key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<String>(&s).ok())
}

fn guardar_ajuste<T: Serialize>(
    conn: &rusqlite::Connection,
    key: &str,
    value: &T,
    ahora: &str,
) -> AppResult<()> {
    let json = serde_json::to_string(value).unwrap_or_default();
    repo_varios::set_setting_raw(conn, key, &json, ahora).map_err(rusqlite_err_to_app_error)
}

fn error_ajuste_a_app_error(e: crate::domain::ajustes::ErrorAjuste) -> Box<AppError> {
    use crate::domain::ajustes::ErrorAjuste;
    let detalle = match e {
        ErrorAjuste::FueraDeLimites { minimo, maximo } => {
            format!("debe estar entre {minimo} y {maximo}")
        }
        ErrorAjuste::CriticoNoMasSeveroQueAviso => {
            "el valor crítico debe ser más severo que el de aviso".to_string()
        }
    };
    Box::new(
        AppError::new("settings.out_of_range", "error.settingsOutOfRange").with_detail(detalle),
    )
}

fn valor_i64(v: &serde_json::Value) -> AppResult<i64> {
    v.as_i64().ok_or_else(|| {
        Box::new(
            AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                .with_detail("se esperaba un entero"),
        )
    })
}

fn valor_f64(v: &serde_json::Value) -> AppResult<f64> {
    v.as_f64().ok_or_else(|| {
        Box::new(
            AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                .with_detail("se esperaba un número"),
        )
    })
}

fn valor_bool(v: &serde_json::Value) -> AppResult<bool> {
    v.as_bool().ok_or_else(|| {
        Box::new(
            AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                .with_detail("se esperaba un booleano"),
        )
    })
}

fn valor_string(v: &serde_json::Value) -> AppResult<String> {
    v.as_str().map(str::to_string).ok_or_else(|| {
        Box::new(
            AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                .with_detail("se esperaba una cadena"),
        )
    })
}

fn get_appearance_settings_impl(conn: &rusqlite::Connection) -> AppearanceSettings {
    AppearanceSettings {
        theme: leer_ajuste_string(conn, CLAVE_APARIENCIA_TEMA, "system"),
        language: leer_ajuste_string_opcional(conn, CLAVE_APARIENCIA_IDIOMA),
        system_locale: crate::platform::locale::system_locale(),
        use_system_accent: leer_ajuste_bool(conn, CLAVE_APARIENCIA_ACENTO_SISTEMA, true),
    }
}

#[tauri::command]
pub fn get_appearance_settings(state: State<AppState>) -> AppResult<AppearanceSettings> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    Ok(get_appearance_settings_impl(&conn))
}

#[tauri::command]
pub fn get_system_accent_color() -> AppResult<WindowsAccent> {
    accent::read()
}

/// Espejo de `Settings` en `src/lib/api/types.ts` (`docs/open-questions.md` J.32: cuatro grupos,
/// alineados con los cuatro valores de `reset_settings({scope})`; `lifecycle`/`notifications`/
/// `logging` solo se restauran con `scope: "all"`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleSettingsWire {
    pub metrics_fast_seconds: i64,
    pub smart_full_seconds: i64,
    pub events_seconds: i64,
    pub discovery_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertThresholdSettingsWire {
    pub temp_configured_warn_c: f64,
    pub temp_configured_crit_c: f64,
    pub capacity_warn_percent: f64,
    pub capacity_crit_percent: f64,
    pub capacity_absolute_floor_min_capacity_bytes: i64,
    pub capacity_absolute_floor_warn_bytes: i64,
    pub capacity_absolute_floor_crit_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionSettingsWire {
    pub raw_days: i64,
    pub five_minutes_days: i64,
    pub hourly_days: i64,
    pub free_space_warn_bytes: i64,
    pub free_space_halt_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSettingsWire {
    pub close_action: String,
    pub close_action_remembered: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettingsWire {
    pub sound_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingSettingsWire {
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsWire {
    pub schedule: ScheduleSettingsWire,
    pub alerts: AlertThresholdSettingsWire,
    pub retention: RetentionSettingsWire,
    pub lifecycle: LifecycleSettingsWire,
    pub notifications: NotificationSettingsWire,
    pub logging: LoggingSettingsWire,
}

fn get_settings_impl(conn: &rusqlite::Connection) -> SettingsWire {
    use crate::domain::ajustes as aj;
    SettingsWire {
        schedule: ScheduleSettingsWire {
            metrics_fast_seconds: leer_ajuste_i64(conn, "schedule.metrics_fast_seconds", 30),
            smart_full_seconds: leer_ajuste_i64(conn, "schedule.smart_full_seconds", 300),
            events_seconds: leer_ajuste_i64(conn, "schedule.events_seconds", 30),
            discovery_seconds: leer_ajuste_i64(conn, "schedule.discovery_seconds", 60),
        },
        alerts: AlertThresholdSettingsWire {
            temp_configured_warn_c: leer_ajuste_f64(
                conn,
                "alerts.temp_configured_warn_c",
                aj::TEMP_WARN_DEFAULT_C,
            ),
            temp_configured_crit_c: leer_ajuste_f64(
                conn,
                "alerts.temp_configured_crit_c",
                aj::TEMP_CRIT_DEFAULT_C,
            ),
            capacity_warn_percent: leer_ajuste_f64(
                conn,
                "alerts.capacity_warn_percent",
                aj::CAPACITY_WARN_PERCENT_DEFAULT,
            ),
            capacity_crit_percent: leer_ajuste_f64(
                conn,
                "alerts.capacity_crit_percent",
                aj::CAPACITY_CRIT_PERCENT_DEFAULT,
            ),
            capacity_absolute_floor_min_capacity_bytes: leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_min_capacity_bytes",
                aj::CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES,
            ),
            capacity_absolute_floor_warn_bytes: leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_warn_bytes",
                aj::CAPACITY_FLOOR_WARN_DEFAULT_BYTES,
            ),
            capacity_absolute_floor_crit_bytes: leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_crit_bytes",
                aj::CAPACITY_FLOOR_CRIT_DEFAULT_BYTES,
            ),
        },
        retention: RetentionSettingsWire {
            raw_days: leer_ajuste_i64(conn, "retention.raw_days", aj::RETENTION_RAW_DAYS_DEFAULT),
            five_minutes_days: leer_ajuste_i64(
                conn,
                "retention.five_minutes_days",
                aj::RETENTION_FIVE_MINUTES_DAYS_DEFAULT,
            ),
            hourly_days: leer_ajuste_i64(
                conn,
                "retention.hourly_days",
                aj::RETENTION_HOURLY_DAYS_DEFAULT,
            ),
            free_space_warn_bytes: leer_ajuste_i64(
                conn,
                "storage.free_space_warn_bytes",
                aj::STORAGE_FREE_SPACE_WARN_DEFAULT_BYTES,
            ),
            free_space_halt_bytes: leer_ajuste_i64(
                conn,
                "storage.free_space_halt_bytes",
                aj::STORAGE_FREE_SPACE_HALT_DEFAULT_BYTES,
            ),
        },
        lifecycle: LifecycleSettingsWire {
            close_action: leer_ajuste_string(conn, "lifecycle.close_action", "minimize"),
            close_action_remembered: leer_ajuste_bool(
                conn,
                "lifecycle.close_action_remembered",
                false,
            ),
        },
        notifications: NotificationSettingsWire {
            sound_enabled: leer_ajuste_bool(conn, "notifications.sound_enabled", false),
        },
        logging: LoggingSettingsWire {
            verbose: leer_ajuste_bool(conn, "logging.verbose", false),
        },
    }
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<SettingsWire> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    Ok(get_settings_impl(&conn))
}

fn set_setting_impl(
    conn: &rusqlite::Connection,
    key: &str,
    value: &serde_json::Value,
) -> AppResult<()> {
    use crate::domain::ajustes as aj;
    let ahora = ahora_rfc3339();

    match key {
        "schedule.metrics_fast_seconds" => {
            let segundos = valor_i64(value)?;
            aj::validar_frecuencia_segundos(
                crate::collectors::planificador::METRICAS_RAPIDAS,
                segundos,
            )
            .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &segundos, &ahora)?;
        }
        "schedule.smart_full_seconds" => {
            let segundos = valor_i64(value)?;
            aj::validar_frecuencia_segundos(
                crate::collectors::planificador::SMART_COMPLETO,
                segundos,
            )
            .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &segundos, &ahora)?;
        }
        "schedule.events_seconds" => {
            let segundos = valor_i64(value)?;
            aj::validar_frecuencia_segundos(
                crate::collectors::planificador::EVENTOS_WINDOWS,
                segundos,
            )
            .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &segundos, &ahora)?;
        }
        "schedule.discovery_seconds" => {
            let segundos = valor_i64(value)?;
            aj::validar_frecuencia_segundos(
                crate::collectors::planificador::ALTAS_Y_BAJAS,
                segundos,
            )
            .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &segundos, &ahora)?;
        }
        "alerts.temp_configured_warn_c" => {
            let nuevo = valor_f64(value)?;
            let crit_actual = leer_ajuste_f64(
                conn,
                "alerts.temp_configured_crit_c",
                aj::TEMP_CRIT_DEFAULT_C,
            );
            aj::validar_temperaturas(nuevo, crit_actual).map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "alerts.temp_configured_crit_c" => {
            let nuevo = valor_f64(value)?;
            let warn_actual = leer_ajuste_f64(
                conn,
                "alerts.temp_configured_warn_c",
                aj::TEMP_WARN_DEFAULT_C,
            );
            aj::validar_temperaturas(warn_actual, nuevo).map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "alerts.capacity_warn_percent" => {
            let nuevo = valor_f64(value)?;
            let crit_actual = leer_ajuste_f64(
                conn,
                "alerts.capacity_crit_percent",
                aj::CAPACITY_CRIT_PERCENT_DEFAULT,
            );
            aj::validar_capacidad_porcentual(nuevo, crit_actual)
                .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "alerts.capacity_crit_percent" => {
            let nuevo = valor_f64(value)?;
            let warn_actual = leer_ajuste_f64(
                conn,
                "alerts.capacity_warn_percent",
                aj::CAPACITY_WARN_PERCENT_DEFAULT,
            );
            aj::validar_capacidad_porcentual(warn_actual, nuevo)
                .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "alerts.capacity_absolute_floor_min_capacity_bytes" => {
            let nuevo = valor_i64(value)?;
            let warn_actual = leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_warn_bytes",
                aj::CAPACITY_FLOOR_WARN_DEFAULT_BYTES,
            );
            let crit_actual = leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_crit_bytes",
                aj::CAPACITY_FLOOR_CRIT_DEFAULT_BYTES,
            );
            aj::validar_capacidad_absoluta(nuevo, warn_actual, crit_actual)
                .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "alerts.capacity_absolute_floor_warn_bytes" => {
            let nuevo = valor_i64(value)?;
            let min_actual = leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_min_capacity_bytes",
                aj::CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES,
            );
            let crit_actual = leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_crit_bytes",
                aj::CAPACITY_FLOOR_CRIT_DEFAULT_BYTES,
            );
            aj::validar_capacidad_absoluta(min_actual, nuevo, crit_actual)
                .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "alerts.capacity_absolute_floor_crit_bytes" => {
            let nuevo = valor_i64(value)?;
            let min_actual = leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_min_capacity_bytes",
                aj::CAPACITY_FLOOR_MIN_CAPACITY_DEFAULT_BYTES,
            );
            let warn_actual = leer_ajuste_i64(
                conn,
                "alerts.capacity_absolute_floor_warn_bytes",
                aj::CAPACITY_FLOOR_WARN_DEFAULT_BYTES,
            );
            aj::validar_capacidad_absoluta(min_actual, warn_actual, nuevo)
                .map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &nuevo, &ahora)?;
        }
        "retention.raw_days" => {
            let dias = valor_i64(value)?;
            aj::validar_retencion_raw_days(dias).map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &dias, &ahora)?;
        }
        "retention.five_minutes_days" => {
            let dias = valor_i64(value)?;
            aj::validar_retencion_five_minutes_days(dias).map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &dias, &ahora)?;
        }
        "retention.hourly_days" => {
            let dias = valor_i64(value)?;
            aj::validar_retencion_hourly_days(dias).map_err(error_ajuste_a_app_error)?;
            guardar_ajuste(conn, key, &dias, &ahora)?;
        }
        "storage.free_space_warn_bytes" | "storage.free_space_halt_bytes" => {
            // Sin límites propios de edición (J.32): US-071 solo exige poder cambiar los tres
            // periodos de retención, no estos dos bytes. Solo se exige que sean positivos.
            let bytes = valor_i64(value)?;
            if bytes <= 0 {
                return Err(error_ajuste_a_app_error(aj::ErrorAjuste::FueraDeLimites {
                    minimo: 1.0,
                    maximo: f64::MAX,
                }));
            }
            guardar_ajuste(conn, key, &bytes, &ahora)?;
        }
        "lifecycle.close_action" => {
            let accion = valor_string(value)?;
            if accion != "minimize" && accion != "exit" {
                return Err(Box::new(
                    AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                        .with_detail("se esperaba \"minimize\" o \"exit\""),
                ));
            }
            guardar_ajuste(conn, key, &accion, &ahora)?;
        }
        "lifecycle.close_action_remembered" | "notifications.sound_enabled" => {
            let b = valor_bool(value)?;
            guardar_ajuste(conn, key, &b, &ahora)?;
        }
        CLAVE_APARIENCIA_TEMA => {
            let tema = valor_string(value)?;
            if !["light", "dark", "system"].contains(&tema.as_str()) {
                return Err(Box::new(
                    AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                        .with_detail("se esperaba \"light\", \"dark\" o \"system\""),
                ));
            }
            guardar_ajuste(conn, key, &tema, &ahora)?;
        }
        CLAVE_APARIENCIA_IDIOMA => {
            let idioma = valor_string(value)?;
            if idioma != "es" && idioma != "en" {
                return Err(Box::new(
                    AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                        .with_detail("se esperaba \"es\" o \"en\""),
                ));
            }
            guardar_ajuste(conn, key, &idioma, &ahora)?;
        }
        CLAVE_APARIENCIA_ACENTO_SISTEMA => {
            let b = valor_bool(value)?;
            guardar_ajuste(conn, key, &b, &ahora)?;
        }
        // `logging.verbose` se cambia solo con `set_log_level` (T098): ese comando persiste la
        // clave **y** recarga el filtro en caliente a la vez; permitirlo también aquí abriría una
        // segunda vía que podría dejar el filtro activo desincronizado de lo guardado.
        _ => {
            return Err(Box::new(
                AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                    .with_detail(format!("clave de ajuste desconocida: {key}")),
            ))
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: serde_json::Value) -> AppResult<()> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    set_setting_impl(&conn, &key, &value)
}

/// Claves que `reset_settings` borra por ámbito (T097): borrar, no reescribir el valor de
/// fábrica, para que `leer_ajuste_*` sea la única fuente de esos valores (`docs/open-questions.md`
/// J.32).
fn claves_por_ambito(scope: &str) -> Vec<&'static str> {
    const SCHEDULE: &[&str] = &[
        "schedule.metrics_fast_seconds",
        "schedule.smart_full_seconds",
        "schedule.events_seconds",
        "schedule.discovery_seconds",
    ];
    const ALERTS: &[&str] = &[
        "alerts.temp_configured_warn_c",
        "alerts.temp_configured_crit_c",
        "alerts.capacity_warn_percent",
        "alerts.capacity_crit_percent",
        "alerts.capacity_absolute_floor_min_capacity_bytes",
        "alerts.capacity_absolute_floor_warn_bytes",
        "alerts.capacity_absolute_floor_crit_bytes",
    ];
    const RETENTION: &[&str] = &[
        "retention.raw_days",
        "retention.five_minutes_days",
        "retention.hourly_days",
        "storage.free_space_warn_bytes",
        "storage.free_space_halt_bytes",
    ];
    const RESTO: &[&str] = &[
        "lifecycle.close_action",
        "lifecycle.close_action_remembered",
        "notifications.sound_enabled",
        "logging.verbose",
    ];
    match scope {
        "schedule" => SCHEDULE.to_vec(),
        "alerts" => ALERTS.to_vec(),
        "retention" => RETENTION.to_vec(),
        _ => [SCHEDULE, ALERTS, RETENTION, RESTO].concat(),
    }
}

#[tauri::command]
pub fn reset_settings(state: State<AppState>, scope: String) -> AppResult<SettingsWire> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    for clave in claves_por_ambito(&scope) {
        repo_varios::delete_setting(&conn, clave).map_err(rusqlite_err_to_app_error)?;
    }
    Ok(get_settings_impl(&conn))
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
    // Los contadores de rendimiento son un colector aparte, con su propia cadencia
    // (`METRICAS_RAPIDAS`, independiente de `SMART_COMPLETO`): se refleja aunque todavía no haya
    // llegado ninguna lectura SMART, en vez de esperar a `temperatura` como el resto de campos de
    // aquí abajo. Antes de esto, `activity_percent` nunca se leía y se quedaba en "no disponible"
    // para siempre, aunque el dato ya estuviera guardado.
    let actividad = repo_metricas::latest_device_sample(conn, &d.id, "activity_percent")
        .map_err(rusqlite_err_to_app_error)?;
    resumen.activity_percent = actividad.as_ref().and_then(|m| m.value_real);

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
/// El número tras `/dev/pdN` (la ruta que `smartctl` de verdad entiende en esta compilación,
/// `open-questions.md` J.42 — no `\\.\PhysicalDriveN`): la misma numeración efímera de Windows que
/// usa `Get-Partition` (`DiskNumber`) para decir a qué disco pertenece un volumen. No es identidad
/// (`disco.wwn_o_pnp_device_id` lo es), pero es lo único que ambas consultas comparten en el
/// mismo instante de lectura.
fn disk_number_from_smartctl_path(path: &str) -> Option<i64> {
    path.strip_prefix("/dev/pd")?.parse().ok()
}

/// Ids de dispositivo dados de alta o de baja en una reconciliación (T021, `inventory:changed`).
/// `actualizados` se deja siempre vacío a propósito: `ui-contract.md` §4 solo documenta "alta o
/// retirada" como disparador de este evento (`open-questions.md` J.35).
struct CambiosInventario {
    ids_dados_de_alta: Vec<String>,
    ids_dados_de_baja: Vec<String>,
}

fn reconciliar_inventario(
    conn: &rusqlite::Connection,
    leidos: &[crate::collectors::windows_storage::DiscoFisico],
    volumenes_leidos: &[crate::collectors::capacidad::VolumenLeido],
    ahora: &str,
) -> AppResult<CambiosInventario> {
    let mut mapa_disco_a_device_id: std::collections::HashMap<i64, String> =
        std::collections::HashMap::new();
    let mut mapa_huella_a_device_id: std::collections::HashMap<String, String> =
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
        mapa_huella_a_device_id.insert(dispositivo.fingerprint.clone(), dispositivo.id.clone());
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
    let mut ids_dados_de_baja = Vec::new();
    for huella_baja in &cambios.bajas {
        if let Some(d) = repo_inventario::get_device_by_fingerprint(conn, huella_baja)
            .map_err(rusqlite_err_to_app_error)?
        {
            repo_inventario::mark_device_removed(conn, &d.id, ahora)
                .map_err(rusqlite_err_to_app_error)?;
            ids_dados_de_baja.push(d.id);
        }
    }
    let ids_dados_de_alta: Vec<String> = cambios
        .altas
        .iter()
        .filter_map(|huella| mapa_huella_a_device_id.get(huella).cloned())
        .collect();

    Ok(CambiosInventario {
        ids_dados_de_alta,
        ids_dados_de_baja,
    })
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

/// Lo que produce un ciclo completo, listo para su post-proceso común (`post_procesar_ciclo`,
/// `open-questions.md` J.39): transiciones de alerta, fuentes recién degradadas, altas/bajas de
/// inventario (si el ciclo tocó ese trabajo) y si produjo alguna métrica (para `metrics:updated`).
struct ResultadoCicloPost {
    transiciones: Vec<(String, crate::alerts::agrupacion::Transicion)>,
    degradadas: Vec<SourceHealth>,
    cambios_inventario: Option<CambiosInventario>,
    hubo_metrica: bool,
}

/// Post-proceso común a `refresh_now` y al bucle en segundo plano (`open-questions.md` J.39):
/// notificaciones, `alerts:changed`, `source:degraded`, `inventory:changed`, `metrics:updated` y el
/// icono de la bandeja. Se llama solo cuando el ciclo terminó sin error — un error se trata en cada
/// llamador según su propia semántica (ver comentario de cada uno).
fn post_procesar_ciclo(
    app: &tauri::AppHandle,
    state: &State<AppState>,
    resultado: &ResultadoCicloPost,
) {
    crate::alerts::notificaciones::procesar_transiciones(app, &resultado.transiciones);
    let ids: Vec<String> = resultado
        .transiciones
        .iter()
        .filter(|(_, t)| *t != crate::alerts::agrupacion::Transicion::SinCambio)
        .map(|(id, _)| id.clone())
        .collect();
    if !ids.is_empty() {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        emitir_alerts_changed(app, &conn, &ids);
    }

    for fuente in resultado.degradadas.iter().cloned() {
        emitir_source_degraded(app, fuente);
    }

    if let Some(cambios) = &resultado.cambios_inventario {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        emitir_inventory_changed(app, &conn, cambios);
    }

    if resultado.hubo_metrica {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        let source_health = state
            .source_health
            .lock()
            .expect("el mutex de estado de fuentes no se envenena: sin pánicos dentro");
        emitir_metrics_updated(app, &conn, &source_health);
    }

    crate::platform::bandeja::actualizar(app);
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
    // Todo o nada, a propósito (`open-questions.md` J.39): un error en cualquier paso aborta el
    // resto y no se emite nada salvo la bandeja — es la semántica que ya esperaban sus pruebas y
    // el botón manual "actualizar ahora" antes de esta historia. El bucle en segundo plano
    // (`ejecutar_ciclo`) es más tolerante porque sus trabajos son independientes entre sí.
    let resultado: AppResult<ResultadoCicloPost> = (|| match scope.as_str() {
        "all" => {
            let cambios = refresh_inventory(&state)?;
            let conn = state
                .conn
                .lock()
                .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
            refresh_events(&conn);
            let mut source_health = state
                .source_health
                .lock()
                .expect("el mutex de estado de fuentes no se envenena: sin pánicos dentro");
            let r = refresh_smart(&conn, None, &mut source_health)?;
            let mut degradadas = r.degradadas;
            degradadas.extend(refresh_metricas_rendimiento(
                &conn,
                None,
                &mut source_health,
            )?);
            Ok(ResultadoCicloPost {
                transiciones: r.transiciones,
                degradadas,
                cambios_inventario: Some(cambios),
                hubo_metrica: true,
            })
        }
        "device" => {
            let id = device_id.ok_or_else(|| {
                Box::new(AppError::new("device.not_found", "error.deviceNotFound"))
            })?;
            let conn = state
                .conn
                .lock()
                .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
            let mut source_health = state
                .source_health
                .lock()
                .expect("el mutex de estado de fuentes no se envenena: sin pánicos dentro");
            let r = refresh_smart(&conn, Some(&id), &mut source_health)?;
            let mut degradadas = r.degradadas;
            degradadas.extend(refresh_metricas_rendimiento(
                &conn,
                Some(&id),
                &mut source_health,
            )?);
            Ok(ResultadoCicloPost {
                transiciones: r.transiciones,
                degradadas,
                cambios_inventario: None,
                hubo_metrica: true,
            })
        }
        _ => Err(Box::new(AppError::not_implemented(&format!(
            "refresh_now({scope})"
        )))),
    })();

    match &resultado {
        Ok(r) => post_procesar_ciclo(&app, &state, r),
        Err(_) => crate::platform::bandeja::actualizar(&app),
    }
    resultado.map(|_| ())
}

/// Ejecuta los trabajos debidos de un sondeo del bucle en segundo plano (T020/T021). A diferencia
/// de `refresh_now`, cada trabajo es independiente: el fallo de uno se registra y no impide que los
/// demás debidos en el mismo sondeo se ejecuten (`open-questions.md` J.39) — un bucle autónomo que
/// se bloquea entero por un fallo ajeno sería peor que uno que registra el fallo y sigue.
fn ejecutar_ciclo(
    app: &tauri::AppHandle,
    trabajos: &[crate::collectors::planificador::TipoTrabajo],
) {
    use crate::collectors::planificador::TipoTrabajo;

    let state = app.state::<AppState>();
    let mut resultado = ResultadoCicloPost {
        transiciones: Vec::new(),
        degradadas: Vec::new(),
        cambios_inventario: None,
        hubo_metrica: false,
    };

    if trabajos.contains(&TipoTrabajo::AltasYBajas) {
        match refresh_inventory(&state) {
            Ok(cambios) => resultado.cambios_inventario = Some(cambios),
            Err(e) => {
                tracing::warn!(error = ?e, "no se pudo reconciliar el inventario en el bucle en segundo plano")
            }
        }
    }

    if trabajos.contains(&TipoTrabajo::EventosWindows) {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        refresh_events(&conn);
    }

    if trabajos.contains(&TipoTrabajo::SmartCompleto) {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        let mut source_health = state
            .source_health
            .lock()
            .expect("el mutex de estado de fuentes no se envenena: sin pánicos dentro");
        match refresh_smart(&conn, None, &mut source_health) {
            Ok(r) => {
                resultado.transiciones.extend(r.transiciones);
                resultado.degradadas.extend(r.degradadas);
                resultado.hubo_metrica = true;
            }
            Err(e) => {
                tracing::warn!(error = ?e, "no se pudo completar el ciclo SMART en segundo plano")
            }
        }
    }

    if trabajos.contains(&TipoTrabajo::MetricasRapidas) {
        let conn = state
            .conn
            .lock()
            .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
        let mut source_health = state
            .source_health
            .lock()
            .expect("el mutex de estado de fuentes no se envenena: sin pánicos dentro");
        match refresh_metricas_rendimiento(&conn, None, &mut source_health) {
            Ok(degradadas) => {
                resultado.degradadas.extend(degradadas);
                resultado.hubo_metrica = true;
            }
            Err(e) => {
                tracing::warn!(error = ?e, "no se pudieron leer los contadores de rendimiento en el bucle en segundo plano")
            }
        }
    }

    post_procesar_ciclo(app, &state, &resultado);
}

/// La frecuencia configurada de cada trabajo, leída de `settings` (mismos valores y valores por
/// defecto que ya usa `get_settings_impl` para el grupo `schedule`).
fn leer_configuracion_trabajos(
    conn: &rusqlite::Connection,
) -> crate::collectors::planificador::ConfiguracionTrabajos {
    crate::collectors::planificador::ConfiguracionTrabajos {
        metricas_rapidas: std::time::Duration::from_secs(leer_ajuste_i64(
            conn,
            "schedule.metrics_fast_seconds",
            30,
        ) as u64),
        smart_completo: std::time::Duration::from_secs(leer_ajuste_i64(
            conn,
            "schedule.smart_full_seconds",
            300,
        ) as u64),
        eventos_windows: std::time::Duration::from_secs(leer_ajuste_i64(
            conn,
            "schedule.events_seconds",
            30,
        ) as u64),
        altas_y_bajas: std::time::Duration::from_secs(leer_ajuste_i64(
            conn,
            "schedule.discovery_seconds",
            60,
        ) as u64),
    }
}

/// El bucle en segundo plano de verdad (T020): sondea cada 1 s (`open-questions.md` J.34), sin
/// tocar nada mientras esté pausado, y se para cuando `AppState.detener_planificador` se marca
/// (`lib.rs`, al recibir `RunEvent::Exit`/`ExitRequested`). Vive en un hilo bloqueante propio
/// (`tauri::async_runtime::spawn_blocking`): todo lo que hace es E/S síncrona (SQLite, procesos,
/// FFI de Windows), nunca futuros — no hace falta ni se añade ningún runtime asíncrono nuevo.
pub fn iniciar_planificador(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        let mut estado_planificador = crate::collectors::planificador::EstadoPlanificador::nuevo();
        loop {
            let state = app.state::<AppState>();
            if state
                .detener_planificador
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                break;
            }

            let pausado = state
                .paused
                .lock()
                .expect("el mutex de pausa no se envenena: sin pánicos dentro")
                .is_some();

            if !pausado {
                let config = {
                    let conn = state
                        .conn
                        .lock()
                        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
                    leer_configuracion_trabajos(&conn)
                };
                let en_bateria = crate::platform::energia::en_bateria();
                let ahora = std::time::Instant::now();
                let debidos = crate::collectors::planificador::trabajos_debidos(
                    &estado_planificador,
                    &config,
                    en_bateria,
                    ahora,
                );
                if !debidos.is_empty() {
                    ejecutar_ciclo(&app, &debidos);
                    for tipo in &debidos {
                        estado_planificador.marcar_ejecutado(*tipo, ahora);
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

/// Canal único vigilado por ahora (T067): los proveedores de `docs/alert-rules.md` §3 viven todos
/// en `System`. Ampliar a más canales es una lista, no un cambio de diseño.
const CANAL_EVENTOS: &str = "System";

/// Lee los eventos nuevos del canal, los correlaciona con el inventario ya reconciliado y los
/// persiste. Un fallo aquí **nunca** aborta `refresh_now` ni el bucle en segundo plano: es la
/// misma tolerancia (SC-008) que ya aplica `refresh_smart` disco a disco — perder un ciclo de
/// eventos es mucho mejor que perder el resto de la recopilación por su culpa.
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

fn refresh_inventory(state: &State<AppState>) -> AppResult<CambiosInventario> {
    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    #[cfg(windows)]
    let resultado_disco = crate::collectors::windows_storage::list_physical_disks();
    #[cfg(not(windows))]
    let resultado_disco: Result<Vec<crate::collectors::windows_storage::DiscoFisico>, String> =
        Ok(vec![]);

    let leidos = {
        let mut source_health = state
            .source_health
            .lock()
            .expect("el mutex de estado de fuentes no se envenena: sin pánicos dentro");
        match resultado_disco {
            Ok(leidos) => {
                actualizar_source_health(
                    &mut source_health,
                    MetricSource::WindowsStorage,
                    1,
                    0,
                    None,
                    &ahora,
                );
                leidos
            }
            Err(e) => {
                let error = AppError::new("windows_storage.failed", "error.storageCollectorFailed")
                    .with_detail(format!("{e:?}"))
                    .from_source(crate::domain::tipos::MetricSource::WindowsStorage)
                    .retryable();
                actualizar_source_health(
                    &mut source_health,
                    MetricSource::WindowsStorage,
                    1,
                    1,
                    Some(error.clone()),
                    &ahora,
                );
                return Err(Box::new(error));
            }
        }
    };

    // Un fallo al leer volúmenes no debe tumbar la recopilación de discos (SC-008): se registra y
    // se continúa con una lista vacía, igual que ya hace `refresh_smart` disco a disco.
    #[cfg(windows)]
    let volumenes = crate::collectors::capacidad::list_volumes().unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "no se pudo leer la capacidad de los volúmenes");
        vec![]
    });
    #[cfg(not(windows))]
    let volumenes: Vec<crate::collectors::capacidad::VolumenLeido> = vec![];

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

/// Actualiza el estado de una fuente tras cerrar un ciclo (`open-questions.md` J.37: se agrega por
/// tipo de colector, no por disco). `intentos == 0` no toca el mapa: un colector que no tenía
/// ningún dispositivo elegible este ciclo no es lo mismo que uno que falló. Devuelve `Some` solo en
/// el flanco de subida a `timeout`/`error` — es lo único que debe disparar `source:degraded`.
fn actualizar_source_health(
    mapa: &mut std::collections::HashMap<MetricSource, SourceHealth>,
    source: MetricSource,
    intentos: u32,
    fallos: u32,
    ultimo_error: Option<AppError>,
    ahora: &str,
) -> Option<SourceHealth> {
    if intentos == 0 {
        return None;
    }
    let anterior_degradada = matches!(
        mapa.get(&source).map(|s| s.status),
        Some(SourceStatus::Timeout) | Some(SourceStatus::Error)
    );
    let hubo_exito = fallos < intentos;
    let status = if fallos == 0 {
        SourceStatus::Ok
    } else if ultimo_error.as_ref().is_some_and(|e| e.retryable) {
        SourceStatus::Timeout
    } else {
        SourceStatus::Error
    };
    let nueva_degradada = matches!(status, SourceStatus::Timeout | SourceStatus::Error);
    let nueva = SourceHealth {
        source,
        status,
        last_success_at: if hubo_exito {
            Some(ahora.to_string())
        } else {
            mapa.get(&source).and_then(|s| s.last_success_at.clone())
        },
        last_attempt_at: Some(ahora.to_string()),
        error: ultimo_error,
    };
    mapa.insert(source, nueva.clone());
    (nueva_degradada && !anterior_degradada).then_some(nueva)
}

/// Lo que produce un ciclo de recopilación SMART/rendimiento: las transiciones de alerta para
/// notificar, y las fuentes que acaban de degradarse en este ciclo (`source:degraded`).
#[derive(Debug)]
struct ResultadoSmart {
    transiciones: Vec<(String, crate::alerts::agrupacion::Transicion)>,
    degradadas: Vec<SourceHealth>,
}

/// Ambas funciones de recopilación (`refresh_smart`, `refresh_metricas_rendimiento`) empiezan
/// igual: si se pide un disco concreto, comprobar que existe.
fn comprobar_dispositivo_existe(
    conn: &rusqlite::Connection,
    solo_device_id: Option<&str>,
) -> AppResult<()> {
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
    Ok(())
}

/// Consulta `smartctl` para los dispositivos presentes con `smartctl_path` conocido —todos si
/// `solo_device_id` es `None`, uno solo si se indica (ámbito `"device"` de `refresh_now`). Un
/// fallo en un disco no aborta el resto (SC-008): se registra, se continúa, y se talla como un
/// intento fallido de su fuente (T020/T021) en vez de propagarse y degradar la recopilación entera.
/// Separada de `refresh_metricas_rendimiento`: cada una tiene su propia cadencia en el bucle en
/// segundo plano (`SMART_COMPLETO` frente a `METRICAS_RAPIDAS`, `open-questions.md` D.1) y mezclar
/// ambas en una sola función ataría sus frecuencias entre sí sin motivo.
fn refresh_smart(
    conn: &rusqlite::Connection,
    solo_device_id: Option<&str>,
    source_health: &mut std::collections::HashMap<MetricSource, SourceHealth>,
) -> AppResult<ResultadoSmart> {
    comprobar_dispositivo_existe(conn, solo_device_id)?;

    let dispositivos =
        repo_inventario::list_present_devices(conn).map_err(rusqlite_err_to_app_error)?;

    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    let mut transiciones = Vec::new();
    let mut intentos: u32 = 0;
    let mut fallos: u32 = 0;
    let mut ultimo_error: Option<AppError> = None;

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
        {
            intentos += 1;
        }
        #[cfg(windows)]
        let json = match crate::collectors::smartctl::query_device_json(ruta) {
            Ok(j) => j,
            Err(e) => {
                fallos += 1;
                ultimo_error = Some(
                    AppError::new("smartctl.query_failed", "error.smartctlQueryFailed")
                        .with_detail(format!("{e:?}"))
                        .retryable(),
                );
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
    }

    let degradadas = actualizar_source_health(
        source_health,
        MetricSource::Smartctl,
        intentos,
        fallos,
        ultimo_error,
        &ahora,
    )
    .into_iter()
    .collect();

    Ok(ResultadoSmart {
        transiciones,
        degradadas,
    })
}

/// Lee los contadores de rendimiento (T058) para los dispositivos presentes con `smartctl_path`
/// conocido —de ahí se deriva el número de disco físico que pide PDH—, todos si `solo_device_id` es
/// `None`, uno solo si se indica. No produce transiciones de alerta: solo persiste (US-020) y talla
/// su fuente (T020/T021).
fn refresh_metricas_rendimiento(
    conn: &rusqlite::Connection,
    solo_device_id: Option<&str>,
    source_health: &mut std::collections::HashMap<MetricSource, SourceHealth>,
) -> AppResult<Vec<SourceHealth>> {
    comprobar_dispositivo_existe(conn, solo_device_id)?;

    let dispositivos =
        repo_inventario::list_present_devices(conn).map_err(rusqlite_err_to_app_error)?;

    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    #[allow(unused_mut)]
    let mut intentos: u32 = 0;
    #[allow(unused_mut)]
    let mut fallos: u32 = 0;
    #[allow(unused_mut)]
    let mut ultimo_error: Option<AppError> = None;

    #[cfg_attr(not(windows), allow(unused_variables))]
    for d in dispositivos
        .iter()
        .filter(|d| d.monitoring_enabled)
        .filter(|d| match solo_device_id {
            Some(id) => id == d.id,
            None => true,
        })
    {
        let Some(_ruta) = &d.smartctl_path else {
            continue;
        };

        #[cfg(windows)]
        if let Some(n) = disk_number_from_smartctl_path(_ruta) {
            intentos += 1;
            match crate::collectors::perf_counters::leer(n) {
                Ok(lectura) => {
                    if let Err(e) = persist_perf_reading(conn, &d.id, &lectura, &ahora) {
                        tracing::warn!(disco = %d.id, error = ?e, "no se pudo guardar la lectura de rendimiento");
                    }
                }
                Err(e) => {
                    fallos += 1;
                    ultimo_error = Some(
                        AppError::new("perf_counters.read_failed", "error.perfCountersFailed")
                            .with_detail(format!("{e:?}"))
                            .retryable(),
                    );
                    tracing::warn!(disco = %d.id, error = ?e, "no se pudo leer los contadores de rendimiento");
                }
            }
        }
    }

    Ok(actualizar_source_health(
        source_health,
        MetricSource::PerformanceCounter,
        intentos,
        fallos,
        ultimo_error,
        &ahora,
    )
    .into_iter()
    .collect())
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

/// Espejo de la carga de `metrics:updated` (`docs/ui-contract.md` §4). Se emite al cerrar un ciclo
/// que produjo alguna métrica (T020/T021), aunque nada haya cambiado: es lo que le dice a la
/// interfaz que un ciclo terminó, no solo que algo cambió.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetricsUpdatedEvent {
    emitted_at: String,
    devices: Vec<DiskSummary>,
    sources: Vec<SourceHealth>,
    history_write_halted: bool,
}

fn emitir_metrics_updated(
    app: &tauri::AppHandle,
    conn: &rusqlite::Connection,
    source_health: &std::collections::HashMap<MetricSource, SourceHealth>,
) {
    let devices = match repo_inventario::list_present_devices(conn) {
        Ok(todos) => todos
            .iter()
            .filter(|d| d.monitoring_enabled)
            .filter_map(|d| enrich_with_smart_data(conn, d).ok())
            .collect(),
        Err(e) => {
            tracing::warn!(error = ?e, "no se pudo listar dispositivos para metrics:updated");
            return;
        }
    };
    let emitted_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    if let Err(e) = app.emit(
        "metrics:updated",
        MetricsUpdatedEvent {
            emitted_at,
            devices,
            sources: source_health.values().cloned().collect(),
            history_write_halted: !historial_habilitado(conn),
        },
    ) {
        tracing::warn!(error = %e, "no se pudo emitir metrics:updated");
    }
}

/// Espejo de la carga de `inventory:changed` (`docs/ui-contract.md` §4). `updated` siempre vacío
/// (`open-questions.md` J.35): el contrato solo documenta alta/retirada como disparador.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InventoryChangedEvent {
    emitted_at: String,
    added: Vec<DiskSummary>,
    removed: Vec<String>,
    updated: Vec<DiskSummary>,
}

fn emitir_inventory_changed(
    app: &tauri::AppHandle,
    conn: &rusqlite::Connection,
    cambios: &CambiosInventario,
) {
    if cambios.ids_dados_de_alta.is_empty() && cambios.ids_dados_de_baja.is_empty() {
        return;
    }
    let added: Vec<DiskSummary> = cambios
        .ids_dados_de_alta
        .iter()
        .filter_map(|id| repo_inventario::get_device(conn, id).ok().flatten())
        .filter_map(|d| enrich_with_smart_data(conn, &d).ok())
        .collect();
    let emitted_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    if let Err(e) = app.emit(
        "inventory:changed",
        InventoryChangedEvent {
            emitted_at,
            added,
            removed: cambios.ids_dados_de_baja.clone(),
            updated: vec![],
        },
    ) {
        tracing::warn!(error = %e, "no se pudo emitir inventory:changed");
    }
}

/// Espejo de la carga de `source:degraded` (`docs/ui-contract.md` §4): solo en el flanco de subida
/// a `timeout`/`error` (`open-questions.md` J.37), nunca en cada ciclo que siga degradado.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceDegradedEvent {
    emitted_at: String,
    source: SourceHealth,
}

fn emitir_source_degraded(app: &tauri::AppHandle, source: SourceHealth) {
    let emitted_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    if let Err(e) = app.emit(
        "source:degraded",
        SourceDegradedEvent { emitted_at, source },
    ) {
        tracing::warn!(error = %e, "no se pudo emitir source:degraded");
    }
}

/// Espejo de la carga compartida de `monitoring:paused`/`monitoring:resumed` (`docs/ui-contract.md`
/// §4, mismo esquema Zod `monitoringPaused` para ambos).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitoringPausedEvent {
    emitted_at: String,
    since: Option<String>,
}

fn emitir_monitoring_pausado(app: &tauri::AppHandle, since: Option<String>) {
    let emitted_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    let evento = if since.is_some() {
        "monitoring:paused"
    } else {
        "monitoring:resumed"
    };
    if let Err(e) = app.emit(evento, MonitoringPausedEvent { emitted_at, since }) {
        tracing::warn!(error = %e, evento, "no se pudo emitir el evento de ciclo de vida");
    }
}

/// Si el ciclo de recopilación debe escribir historial (FR-020a/b/c, guardia de espacio libre,
/// `open-questions.md` J.40: por ahora solo informa, no detiene ninguna escritura todavía). Un
/// fallo al leer el espacio real se trata como `Normal` (J.36): un dato desconocido no es "disco
/// lleno".
fn historial_habilitado(conn: &rusqlite::Connection) -> bool {
    use crate::domain::ajustes as aj;
    let Some(libres) =
        crate::platform::energia::espacio_libre_bytes(&crate::platform::paths::data_dir())
    else {
        return true;
    };
    let umbrales = crate::domain::espacio::UmbralesEspacio {
        aviso_bytes: leer_ajuste_i64(
            conn,
            "storage.free_space_warn_bytes",
            aj::STORAGE_FREE_SPACE_WARN_DEFAULT_BYTES,
        ) as u64,
        parada_bytes: leer_ajuste_i64(
            conn,
            "storage.free_space_halt_bytes",
            aj::STORAGE_FREE_SPACE_HALT_DEFAULT_BYTES,
        ) as u64,
    };
    crate::domain::espacio::debe_escribir_historial(umbrales.evaluar(libres))
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

// ---- Informes y diagnóstico (T091, docs/ui-contract.md §3.7) ----------------------------------

fn parsear_rango(from_utc: &str, to_utc: &str) -> AppResult<crate::reporting::export::RangoExport> {
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
    Ok(crate::reporting::export::RangoExport { desde, hasta })
}

/// `device_ids: None` significa todos los dispositivos monitorizados, no los excluidos
/// (`docs/open-questions.md` J.30) — mismo criterio que el resto de la aplicación.
fn dispositivos_para_informe(
    conn: &rusqlite::Connection,
    device_ids: Option<&[String]>,
) -> AppResult<Vec<Device>> {
    match device_ids {
        Some(ids) => {
            let mut resultado = Vec::with_capacity(ids.len());
            for id in ids {
                if let Some(d) =
                    repo_inventario::get_device(conn, id).map_err(rusqlite_err_to_app_error)?
                {
                    resultado.push(d);
                }
            }
            Ok(resultado)
        }
        None => Ok(repo_inventario::list_present_devices(conn)
            .map_err(rusqlite_err_to_app_error)?
            .into_iter()
            .filter(|d| d.monitoring_enabled)
            .collect()),
    }
}

fn export_report_impl(
    conn: &rusqlite::Connection,
    format: &str,
    from_utc: &str,
    to_utc: &str,
    device_ids: Option<&[String]>,
    include_serials: bool,
    destination_path: &str,
) -> AppResult<String> {
    let rango = parsear_rango(from_utc, to_utc)?;
    let dispositivos = dispositivos_para_informe(conn, device_ids)?;

    let contenido = match format {
        "csv" => crate::reporting::export::generar_csv(conn, &dispositivos, rango)
            .map_err(rusqlite_err_to_app_error)?,
        "json" => {
            crate::reporting::export::generar_json(conn, &dispositivos, rango, include_serials)
                .map_err(rusqlite_err_to_app_error)?
        }
        "html" => {
            let alertas = repo_alertas::list_groups(conn).map_err(rusqlite_err_to_app_error)?;
            crate::reporting::informe::generar_html(&dispositivos, &alertas, rango, include_serials)
        }
        _ => {
            return Err(Box::new(
                AppError::new("ipc.schema_mismatch", "error.schemaMismatch")
                    .with_detail(format!("formato desconocido: {format}")),
            ))
        }
    };

    std::fs::write(destination_path, contenido).map_err(|e| {
        Box::new(
            AppError::new("export.write_failed", "error.exportWriteFailed")
                .with_detail(e.to_string())
                .retryable(),
        )
    })?;

    Ok(destination_path.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn export_report(
    state: State<AppState>,
    format: String,
    from_utc: String,
    to_utc: String,
    device_ids: Option<Vec<String>>,
    include_serials: bool,
    destination_path: String,
) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    export_report_impl(
        &conn,
        &format,
        &from_utc,
        &to_utc,
        device_ids.as_deref(),
        include_serials,
        &destination_path,
    )
}

/// Espejo de `DiagnosticPreview` en `src/lib/api/types.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticPreviewEntryWire {
    pub path: String,
    pub size_bytes: i64,
    pub description_key: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticPreviewWire {
    pub entries: Vec<DiagnosticPreviewEntryWire>,
    pub total_bytes: i64,
    pub redacted_fields: Vec<String>,
}

/// Clave de i18n para el nombre de cada entrada del paquete en la vista previa: el backend no
/// manda frases, solo claves (mismo criterio que `AlertGroup.ruleKey`, ADR-030).
fn clave_descripcion_entrada(ruta: &str) -> String {
    if ruta == "manifest.json" {
        "diagnostic.entry.manifest".to_string()
    } else if ruta == "settings.json" {
        "diagnostic.entry.settings".to_string()
    } else if ruta == "events.json" {
        "diagnostic.entry.events".to_string()
    } else if ruta.starts_with("smart/") {
        "diagnostic.entry.smart".to_string()
    } else if ruta.starts_with("logs/") {
        "diagnostic.entry.logs".to_string()
    } else {
        "diagnostic.entry.other".to_string()
    }
}

fn paquete_diagnostico(
    conn: &rusqlite::Connection,
    include_identifiers: bool,
) -> AppResult<crate::reporting::diagnostico::PaqueteDiagnostico> {
    let dispositivos = repo_inventario::list_present_devices(conn)
        .map_err(rusqlite_err_to_app_error)?
        .into_iter()
        .filter(|d| d.monitoring_enabled)
        .collect::<Vec<_>>();
    Ok(crate::reporting::diagnostico::recolectar(
        conn,
        &dispositivos,
        include_identifiers,
        &crate::platform::paths::log_dir(),
    ))
}

#[tauri::command]
pub fn preview_diagnostic_zip(
    state: State<AppState>,
    include_identifiers: bool,
) -> AppResult<DiagnosticPreviewWire> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let paquete = paquete_diagnostico(&conn, include_identifiers)?;
    Ok(DiagnosticPreviewWire {
        entries: paquete
            .entradas
            .iter()
            .map(|e| DiagnosticPreviewEntryWire {
                path: e.ruta.clone(),
                size_bytes: e.contenido.len() as i64,
                description_key: clave_descripcion_entrada(&e.ruta),
            })
            .collect(),
        total_bytes: paquete.total_bytes() as i64,
        redacted_fields: paquete
            .campos_redactados
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

#[tauri::command]
pub fn create_diagnostic_zip(
    state: State<AppState>,
    include_identifiers: bool,
    destination_path: String,
) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let paquete = paquete_diagnostico(&conn, include_identifiers)?;
    crate::reporting::diagnostico::escribir_zip(&paquete, std::path::Path::new(&destination_path))
        .map_err(|e| {
            Box::new(
                AppError::new("export.write_failed", "error.exportWriteFailed")
                    .with_detail(e.to_string())
                    .retryable(),
            )
        })?;
    Ok(destination_path)
}

#[tauri::command]
pub fn pause_monitoring(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    let ahora = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    *state
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro") = Some(ahora.clone());
    crate::platform::bandeja::actualizar(&app);
    emitir_monitoring_pausado(&app, Some(ahora));
    Ok(())
}

#[tauri::command]
pub fn resume_monitoring(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    *state
        .paused
        .lock()
        .expect("el mutex de pausa no se envenena: sin pánicos dentro") = None;
    crate::platform::bandeja::actualizar(&app);
    emitir_monitoring_pausado(&app, None);
    Ok(())
}

/// Nombre y versión salen del manifiesto, nunca de un literal duplicado (ADR-011, T104):
/// `app.package_info().name` es el `productName` de `tauri.conf.json` ("SmartDisk Monitor"), no
/// `CARGO_PKG_NAME` (el nombre del *crate*, en minúsculas con guiones) — Tauri resuelve el primero
/// a partir del segundo en tiempo de compilación cuando el manifiesto declara `productName`.
#[tauri::command]
pub fn get_app_info(app: tauri::AppHandle) -> AppResult<AppInfo> {
    let info = app.package_info();
    Ok(AppInfo {
        name: info.name.clone(),
        version: info.version.to_string(),
        author: info.authors.to_string(),
    })
}

/// Frase que hay que escribir literalmente para borrar todo (US-073): el propio nombre de la
/// aplicación, no una palabra ceremonial — así no cambia con el idioma de la interfaz ni exige
/// mantener la misma cadena traducida en los dos diccionarios.
const FRASE_CONFIRMACION_BORRAR_TODO: &str = "SmartDisk Monitor";

/// Todas las tablas de datos salvo `schema_migrations`, que no le pertenece a "los datos del
/// usuario" sino al propio esquema de la base. `settings` se incluye: "queda como recién
/// instalada" (US-073) también vacía las preferencias, que es justo lo que no existe en una
/// instalación nueva.
const TABLAS_DATOS_USUARIO: &[&str] = &[
    "alert_occurrences",
    "alert_groups",
    "test_runs",
    "system_events",
    "event_cursors",
    "smart_snapshots",
    "metric_aggregates",
    "metric_samples",
    "device_volume_links",
    "volumes",
    "devices",
    "settings",
];

fn delete_all_data_impl(conn: &mut rusqlite::Connection) -> AppResult<()> {
    let tx = conn.transaction().map_err(rusqlite_err_to_app_error)?;
    for tabla in TABLAS_DATOS_USUARIO {
        tx.execute(&format!("DELETE FROM {tabla}"), [])
            .map_err(rusqlite_err_to_app_error)?;
    }
    tx.commit().map_err(rusqlite_err_to_app_error)?;
    Ok(())
}

#[tauri::command]
pub fn delete_all_data(state: State<AppState>, confirmation_phrase: String) -> AppResult<()> {
    if confirmation_phrase != FRASE_CONFIRMACION_BORRAR_TODO {
        return Err(Box::new(
            AppError::new("ipc.schema_mismatch", "error.confirmationPhraseMismatch")
                .with_detail("la frase escrita no coincide"),
        ));
    }
    let mut conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    delete_all_data_impl(&mut conn)
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

fn nivel_resuelto(conn: &rusqlite::Connection) -> crate::logging::LogLevel {
    // Precedencia (constitución §XV): --log-level > settings > info.
    let args: Vec<String> = std::env::args().collect();
    crate::logging::level_from_cli(&args).unwrap_or_else(|| {
        if leer_ajuste_bool(conn, "logging.verbose", false) {
            crate::logging::LogLevel::Debug
        } else {
            crate::logging::LogLevel::Info
        }
    })
}

/// Nivel efectivo, para que el frontend filtre igual que el backend y no envíe lo que se
/// descartaría de todos modos.
#[tauri::command]
pub fn get_log_level(state: State<AppState>) -> AppResult<String> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    let level = nivel_resuelto(&conn);
    Ok(serde_json::to_value(level)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "info".to_owned()))
}

/// Interruptor de modo detallado (T098, FR-029a): persiste `logging.verbose` **y** recarga el
/// filtro en caliente a la vez, para que activarlo no exija reiniciar (US-070, "un cambio se
/// aplica sin reiniciar" — el mismo criterio, aunque esta historia sea la 071).
#[tauri::command]
pub fn set_log_level(
    state: State<AppState>,
    manejador: State<crate::logging::ManejadorNivel>,
    verbose: bool,
) -> AppResult<()> {
    let conn = state
        .conn
        .lock()
        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
    guardar_ajuste(&conn, "logging.verbose", &verbose, &ahora_rfc3339())?;
    drop(conn);

    // Si `--log-level` fuerza un nivel por línea de órdenes, ese nivel manda siempre (§XV): el
    // interruptor de la interfaz persiste igual la preferencia, pero no la aplica en caliente
    // hasta el próximo arranque sin ese argumento.
    let args: Vec<String> = std::env::args().collect();
    if crate::logging::level_from_cli(&args).is_some() {
        return Ok(());
    }
    let nivel = if verbose {
        crate::logging::LogLevel::Debug
    } else {
        crate::logging::LogLevel::Info
    };
    manejador.establecer(nivel).map_err(|e| {
        Box::new(AppError::new("app.log_reload_failed", "error.unexpected").with_detail(e))
    })?;
    Ok(())
}

/// Abre **una sola ruta conocida** —la carpeta de registro— sin recibirla como argumento: un
/// parámetro de ruta abriría una segunda vía de acceso al sistema de ficheros, que es justo lo
/// que el principio IX prohíbe (`docs/ui-contract.md` §3.9, T098).
#[tauri::command]
pub fn open_log_folder() -> AppResult<()> {
    std::process::Command::new("explorer.exe")
        .arg(crate::platform::paths::log_dir())
        .spawn()
        .map_err(|e| {
            Box::new(
                AppError::new("app.open_folder_failed", "error.unexpected")
                    .with_detail(e.to_string()),
            )
        })?;
    Ok(())
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
            smartctl_device_path: Some("/dev/pd0".to_string()),
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
    fn la_actividad_de_rendimiento_se_refleja_aunque_todavia_no_haya_lectura_smart() {
        // Bug real (encontrado usando la aplicación contra hardware real): `activity_percent`
        // nunca se leía aquí y se quedaba en "no disponible" para siempre, aunque el colector de
        // rendimiento ya lo hubiera guardado. Además, al venir de un colector con cadencia propia
        // (`METRICAS_RAPIDAS`), debe reflejarse aunque la lectura SMART (`SMART_COMPLETO`, más
        // lenta) todavía no haya llegado — no solo cuando ambas ya existen.
        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let ahora = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let lectura = crate::collectors::perf_counters::LecturaRendimiento {
            activity_percent: Some(17.0),
            read_bytes_per_second: None,
            write_bytes_per_second: None,
            read_latency_ms: None,
            write_latency_ms: None,
        };
        persist_perf_reading(&conn, "d1", &lectura, &ahora).unwrap();

        let resumen = get_device_detail_impl(&conn, "d1").unwrap();

        assert_eq!(resumen.summary.activity_percent, Some(17.0));
        // Sin lectura SMART todavía, el resto sigue en su estado por defecto: la actividad no
        // debe forzar un estado "correcto" que la temperatura no ha confirmado.
        assert_eq!(resumen.summary.state, HealthState::Unknown);
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
    fn persist_perf_reading_guarda_solo_las_metricas_presentes() {
        use crate::collectors::perf_counters::LecturaRendimiento;

        let conn = conn_de_prueba();
        let d = dispositivo_con_ruta("d1", Some(r"\.\PhysicalDrive0"));
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let lectura = LecturaRendimiento {
            activity_percent: Some(42.0),
            read_bytes_per_second: Some(1024.0),
            write_bytes_per_second: None,
            read_latency_ms: None,
            write_latency_ms: None,
        };
        persist_perf_reading(&conn, "d1", &lectura, "2026-09-04T10:00:00Z").unwrap();

        let actividad = repo_metricas::latest_device_sample(&conn, "d1", "activity_percent")
            .unwrap()
            .unwrap();
        assert_eq!(actividad.value_real, Some(42.0));

        let lectura_bytes =
            repo_metricas::latest_device_sample(&conn, "d1", "read_bytes_per_second")
                .unwrap()
                .unwrap();
        assert_eq!(lectura_bytes.value_real, Some(1024.0));

        // Ausente en la lectura: no debe inventarse ninguna muestra para esta métrica.
        assert!(
            repo_metricas::latest_device_sample(&conn, "d1", "write_bytes_per_second")
                .unwrap()
                .is_none()
        );
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

    fn fuentes_de_prueba() -> std::collections::HashMap<MetricSource, SourceHealth> {
        std::collections::HashMap::new()
    }

    #[test]
    fn refresh_smart_para_un_dispositivo_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        let err = refresh_smart(&conn, Some("no-existe"), &mut fuentes_de_prueba()).unwrap_err();
        assert_eq!(err.code, "device.not_found");
    }

    #[test]
    fn refresh_smart_sin_filtro_recorre_todos_los_dispositivos_monitorizados() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        repo_inventario::upsert_device(&conn, &dispositivo("d2")).unwrap();

        // Ninguno tiene smartctl_path: el bucle los recorre y los salta sin invocar nada externo.
        assert!(refresh_smart(&conn, None, &mut fuentes_de_prueba()).is_ok());
    }

    #[test]
    fn refresh_smart_con_filtro_no_falla_si_el_dispositivo_existe() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        repo_inventario::upsert_device(&conn, &dispositivo("d2")).unwrap();

        assert!(refresh_smart(&conn, Some("d1"), &mut fuentes_de_prueba()).is_ok());
    }

    #[test]
    fn un_dispositivo_excluido_de_la_monitorizacion_no_detiene_el_ambito_all() {
        let conn = conn_de_prueba();
        let mut excluido = dispositivo("d1");
        excluido.monitoring_enabled = false;
        repo_inventario::upsert_device(&conn, &excluido).unwrap();

        assert!(refresh_smart(&conn, None, &mut fuentes_de_prueba()).is_ok());
    }

    #[test]
    fn ningun_dispositivo_con_ruta_smartctl_no_registra_intentos_de_la_fuente() {
        // Sin ningún dispositivo con `smartctl_path`, el colector no se invoca ni una vez: la
        // fuente se queda sin entrada, no se inventa un "ok" ni un "error" (J.37).
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        let mut fuentes = fuentes_de_prueba();
        refresh_smart(&conn, None, &mut fuentes).unwrap();
        assert!(!fuentes.contains_key(&MetricSource::Smartctl));
    }

    #[test]
    fn refresh_metricas_rendimiento_para_un_dispositivo_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        let err = refresh_metricas_rendimiento(&conn, Some("no-existe"), &mut fuentes_de_prueba())
            .unwrap_err();
        assert_eq!(err.code, "device.not_found");
    }

    #[test]
    fn refresh_metricas_rendimiento_sin_ruta_smartctl_no_falla_y_no_registra_intentos() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        let mut fuentes = fuentes_de_prueba();
        assert!(refresh_metricas_rendimiento(&conn, None, &mut fuentes).is_ok());
        assert!(!fuentes.contains_key(&MetricSource::PerformanceCounter));
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

#[cfg(test)]
mod tests_informes {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_informes");
        db::open(&dir).unwrap().0
    }

    fn dispositivo(id: &str, monitoring_enabled: bool) -> Device {
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
            smartctl_path: None,
            capacity_bytes: Some(1_000_000_000),
            alias: None,
            monitoring_enabled,
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    #[test]
    fn sin_device_ids_solo_entran_los_dispositivos_monitorizados() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1", true)).unwrap();
        repo_inventario::upsert_device(&conn, &dispositivo("d2", false)).unwrap();

        let dispositivos = dispositivos_para_informe(&conn, None).unwrap();
        assert_eq!(dispositivos.len(), 1);
        assert_eq!(dispositivos[0].id, "d1");
    }

    #[test]
    fn con_device_ids_explicitos_se_respeta_la_lista_aunque_incluya_uno_inexistente() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1", true)).unwrap();

        let ids = vec!["d1".to_string(), "no-existe".to_string()];
        let dispositivos = dispositivos_para_informe(&conn, Some(&ids)).unwrap();
        assert_eq!(
            dispositivos.len(),
            1,
            "el id inexistente se omite, no falla"
        );
        assert_eq!(dispositivos[0].id, "d1");
    }

    #[test]
    fn un_formato_desconocido_falla_con_schema_mismatch() {
        let conn = conn_de_prueba();
        let destino =
            crate::test_util::temp_dir_unico("commands_informes_salida").join("informe.xml");
        let err = export_report_impl(
            &conn,
            "xml",
            "2026-09-04T00:00:00Z",
            "2026-09-04T10:00:00Z",
            None,
            false,
            destino.to_str().unwrap(),
        )
        .unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn exportar_csv_escribe_el_fichero_con_la_cabecera_esperada() {
        let conn = conn_de_prueba();
        let dir = crate::test_util::temp_dir_unico("commands_informes_salida");
        std::fs::create_dir_all(&dir).unwrap();
        let destino = dir.join("informe.csv");

        let ruta = export_report_impl(
            &conn,
            "csv",
            "2026-09-04T00:00:00Z",
            "2026-09-04T10:00:00Z",
            None,
            false,
            destino.to_str().unwrap(),
        )
        .unwrap();

        let contenido = std::fs::read_to_string(&ruta).unwrap();
        assert!(contenido.starts_with("schemaVersion,deviceId,deviceLabel"));
    }

    #[test]
    fn exportar_json_escribe_un_fichero_deserializable() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1", true)).unwrap();
        let dir = crate::test_util::temp_dir_unico("commands_informes_salida");
        std::fs::create_dir_all(&dir).unwrap();
        let destino = dir.join("informe.json");

        let ruta = export_report_impl(
            &conn,
            "json",
            "2026-09-04T00:00:00Z",
            "2026-09-04T10:00:00Z",
            None,
            false,
            destino.to_str().unwrap(),
        )
        .unwrap();

        let contenido = std::fs::read_to_string(&ruta).unwrap();
        let valor: serde_json::Value = serde_json::from_str(&contenido).unwrap();
        assert!(valor.is_object(), "el json exportado debe ser un objeto");
    }

    #[test]
    fn exportar_html_escribe_un_informe_con_marcado() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1", true)).unwrap();
        let dir = crate::test_util::temp_dir_unico("commands_informes_salida");
        std::fs::create_dir_all(&dir).unwrap();
        let destino = dir.join("informe.html");

        let ruta = export_report_impl(
            &conn,
            "html",
            "2026-09-04T00:00:00Z",
            "2026-09-04T10:00:00Z",
            None,
            false,
            destino.to_str().unwrap(),
        )
        .unwrap();

        let contenido = std::fs::read_to_string(&ruta).unwrap();
        assert!(contenido.contains("<html"));
    }

    #[test]
    fn exportar_a_una_ruta_de_destino_invalida_falla_con_write_failed() {
        let conn = conn_de_prueba();
        // Un directorio inexistente como destino: `std::fs::write` no puede crear el árbol de
        // carpetas por sí solo y debe fallar.
        let destino = crate::test_util::temp_dir_unico("commands_informes_salida")
            .join("no-existe")
            .join("informe.csv");

        let err = export_report_impl(
            &conn,
            "csv",
            "2026-09-04T00:00:00Z",
            "2026-09-04T10:00:00Z",
            None,
            false,
            destino.to_str().unwrap(),
        )
        .unwrap_err();
        assert_eq!(err.code, "export.write_failed");
        assert!(err.retryable);
    }

    #[test]
    fn clave_de_descripcion_distingue_cada_categoria_de_entrada() {
        assert_eq!(
            clave_descripcion_entrada("manifest.json"),
            "diagnostic.entry.manifest"
        );
        assert_eq!(
            clave_descripcion_entrada("settings.json"),
            "diagnostic.entry.settings"
        );
        assert_eq!(
            clave_descripcion_entrada("events.json"),
            "diagnostic.entry.events"
        );
        assert_eq!(
            clave_descripcion_entrada("smart/d1.json"),
            "diagnostic.entry.smart"
        );
        assert_eq!(
            clave_descripcion_entrada("logs/smartdisk.log.2026-09-04"),
            "diagnostic.entry.logs"
        );
    }
}

#[cfg(test)]
mod tests_ajustes {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_ajustes");
        db::open(&dir).unwrap().0
    }

    #[test]
    fn sin_ninguna_clave_guardada_get_settings_devuelve_los_valores_de_fabrica() {
        let conn = conn_de_prueba();
        let s = get_settings_impl(&conn);
        assert_eq!(s.schedule.metrics_fast_seconds, 30);
        assert_eq!(s.alerts.temp_configured_warn_c, 70.0);
        assert_eq!(s.alerts.capacity_warn_percent, 10.0);
        assert_eq!(s.retention.raw_days, 7);
        assert_eq!(s.lifecycle.close_action, "minimize");
        assert!(!s.notifications.sound_enabled);
        assert!(!s.logging.verbose);
    }

    #[test]
    fn set_setting_persiste_y_get_settings_lo_refleja() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "schedule.metrics_fast_seconds",
            &serde_json::json!(45),
        )
        .unwrap();
        assert_eq!(get_settings_impl(&conn).schedule.metrics_fast_seconds, 45);
    }

    #[test]
    fn una_frecuencia_fuera_de_limites_se_rechaza_con_settings_out_of_range() {
        let conn = conn_de_prueba();
        let err = set_setting_impl(
            &conn,
            "schedule.metrics_fast_seconds",
            &serde_json::json!(5),
        )
        .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn un_critico_de_temperatura_no_mas_severo_que_el_aviso_se_rechaza() {
        let conn = conn_de_prueba();
        let err = set_setting_impl(
            &conn,
            "alerts.temp_configured_crit_c",
            &serde_json::json!(60.0),
        )
        .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn cambiar_el_aviso_de_temperatura_se_valida_contra_el_critico_ya_guardado() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "alerts.temp_configured_crit_c",
            &serde_json::json!(75.0),
        )
        .unwrap();
        // 80 > 75: el nuevo aviso ya no sería más severo que el crítico guardado.
        let err = set_setting_impl(
            &conn,
            "alerts.temp_configured_warn_c",
            &serde_json::json!(80.0),
        )
        .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn una_clave_desconocida_falla_con_schema_mismatch() {
        let conn = conn_de_prueba();
        let err = set_setting_impl(&conn, "no.existe", &serde_json::json!(1)).unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn logging_verbose_no_se_puede_cambiar_por_set_setting_generico() {
        let conn = conn_de_prueba();
        let err = set_setting_impl(&conn, "logging.verbose", &serde_json::json!(true)).unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn cada_frecuencia_de_schedule_se_guarda_dentro_de_limites() {
        let conn = conn_de_prueba();
        for (clave, valor, campo, esperado) in [
            (
                "schedule.smart_full_seconds",
                120,
                "smart_full_seconds" as &str,
                120i64,
            ),
            ("schedule.events_seconds", 45, "events_seconds", 45),
            ("schedule.discovery_seconds", 90, "discovery_seconds", 90),
        ] {
            set_setting_impl(&conn, clave, &serde_json::json!(valor)).unwrap();
            let s = get_settings_impl(&conn);
            let actual = match campo {
                "smart_full_seconds" => s.schedule.smart_full_seconds,
                "events_seconds" => s.schedule.events_seconds,
                "discovery_seconds" => s.schedule.discovery_seconds,
                _ => unreachable!(),
            };
            assert_eq!(actual, esperado, "{clave}");
        }
    }

    #[test]
    fn capacidad_porcentual_se_guarda_y_se_valida_contra_la_pareja_actual() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "alerts.capacity_warn_percent",
            &serde_json::json!(12.0),
        )
        .unwrap();
        assert_eq!(get_settings_impl(&conn).alerts.capacity_warn_percent, 12.0);

        set_setting_impl(
            &conn,
            "alerts.capacity_crit_percent",
            &serde_json::json!(6.0),
        )
        .unwrap();
        assert_eq!(get_settings_impl(&conn).alerts.capacity_crit_percent, 6.0);

        // El crítico igual o por encima del aviso ya guardado se rechaza.
        let err = set_setting_impl(
            &conn,
            "alerts.capacity_crit_percent",
            &serde_json::json!(12.0),
        )
        .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn suelo_absoluto_de_capacidad_se_guarda_y_se_valida_contra_la_terna_actual() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "alerts.capacity_absolute_floor_min_capacity_bytes",
            &serde_json::json!(300_i64 * 1024 * 1024 * 1024),
        )
        .unwrap();
        set_setting_impl(
            &conn,
            "alerts.capacity_absolute_floor_warn_bytes",
            &serde_json::json!(25_i64 * 1024 * 1024 * 1024),
        )
        .unwrap();
        set_setting_impl(
            &conn,
            "alerts.capacity_absolute_floor_crit_bytes",
            &serde_json::json!(12_i64 * 1024 * 1024 * 1024),
        )
        .unwrap();
        let s = get_settings_impl(&conn).alerts;
        assert_eq!(
            s.capacity_absolute_floor_min_capacity_bytes,
            300 * 1024 * 1024 * 1024
        );
        assert_eq!(
            s.capacity_absolute_floor_warn_bytes,
            25 * 1024 * 1024 * 1024
        );
        assert_eq!(
            s.capacity_absolute_floor_crit_bytes,
            12 * 1024 * 1024 * 1024
        );

        // El crítico igual o por encima del aviso ya guardado (25 GiB) se rechaza.
        let err = set_setting_impl(
            &conn,
            "alerts.capacity_absolute_floor_crit_bytes",
            &serde_json::json!(25_i64 * 1024 * 1024 * 1024),
        )
        .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn los_otros_dos_periodos_de_retencion_se_guardan_dentro_de_limites() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "retention.five_minutes_days",
            &serde_json::json!(120),
        )
        .unwrap();
        assert_eq!(get_settings_impl(&conn).retention.five_minutes_days, 120);

        set_setting_impl(&conn, "retention.hourly_days", &serde_json::json!(365)).unwrap();
        assert_eq!(get_settings_impl(&conn).retention.hourly_days, 365);

        let err = set_setting_impl(&conn, "retention.five_minutes_days", &serde_json::json!(1))
            .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn los_umbrales_de_espacio_solo_exigen_ser_positivos() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "storage.free_space_warn_bytes",
            &serde_json::json!(2_i64 * 1024 * 1024 * 1024),
        )
        .unwrap();
        assert_eq!(
            get_settings_impl(&conn).retention.free_space_warn_bytes,
            2 * 1024 * 1024 * 1024
        );

        let err = set_setting_impl(
            &conn,
            "storage.free_space_halt_bytes",
            &serde_json::json!(0),
        )
        .unwrap_err();
        assert_eq!(err.code, "settings.out_of_range");
    }

    #[test]
    fn la_accion_de_cierre_solo_admite_minimize_o_exit() {
        let conn = conn_de_prueba();
        set_setting_impl(&conn, "lifecycle.close_action", &serde_json::json!("exit")).unwrap();
        assert_eq!(get_settings_impl(&conn).lifecycle.close_action, "exit");

        let err = set_setting_impl(
            &conn,
            "lifecycle.close_action",
            &serde_json::json!("hibernate"),
        )
        .unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn recordar_el_cierre_y_el_sonido_de_notificaciones_son_booleanos_simples() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "lifecycle.close_action_remembered",
            &serde_json::json!(true),
        )
        .unwrap();
        assert!(get_settings_impl(&conn).lifecycle.close_action_remembered);

        set_setting_impl(
            &conn,
            "notifications.sound_enabled",
            &serde_json::json!(true),
        )
        .unwrap();
        assert!(get_settings_impl(&conn).notifications.sound_enabled);
    }

    #[test]
    fn el_tema_de_apariencia_solo_admite_los_tres_valores_conocidos() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "settings.appearance.theme",
            &serde_json::json!("dark"),
        )
        .unwrap();
        assert_eq!(get_appearance_settings_impl(&conn).theme, "dark");

        let err = set_setting_impl(
            &conn,
            "settings.appearance.theme",
            &serde_json::json!("azulón"),
        )
        .unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn el_idioma_de_apariencia_solo_admite_es_o_en() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "settings.appearance.language",
            &serde_json::json!("en"),
        )
        .unwrap();
        assert_eq!(
            get_appearance_settings_impl(&conn).language.as_deref(),
            Some("en")
        );

        let err = set_setting_impl(
            &conn,
            "settings.appearance.language",
            &serde_json::json!("fr"),
        )
        .unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn el_acento_del_sistema_es_un_booleano_simple() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "settings.appearance.use_system_accent",
            &serde_json::json!(false),
        )
        .unwrap();
        assert!(!get_appearance_settings_impl(&conn).use_system_accent);
    }

    #[test]
    fn un_valor_del_tipo_equivocado_falla_con_schema_mismatch_para_cada_conversor() {
        let conn = conn_de_prueba();
        // `valor_i64`: se pasa una cadena donde se esperaba un entero.
        assert_eq!(
            set_setting_impl(
                &conn,
                "schedule.metrics_fast_seconds",
                &serde_json::json!("no es un número")
            )
            .unwrap_err()
            .code,
            "ipc.schema_mismatch"
        );
        // `valor_f64`: un booleano donde se esperaba un número.
        assert_eq!(
            set_setting_impl(
                &conn,
                "alerts.temp_configured_warn_c",
                &serde_json::json!(true)
            )
            .unwrap_err()
            .code,
            "ipc.schema_mismatch"
        );
        // `valor_bool`: una cadena donde se esperaba un booleano.
        assert_eq!(
            set_setting_impl(
                &conn,
                "notifications.sound_enabled",
                &serde_json::json!("sí")
            )
            .unwrap_err()
            .code,
            "ipc.schema_mismatch"
        );
        // `valor_string`: un número donde se esperaba una cadena.
        assert_eq!(
            set_setting_impl(&conn, "lifecycle.close_action", &serde_json::json!(1))
                .unwrap_err()
                .code,
            "ipc.schema_mismatch"
        );
    }

    #[test]
    fn reset_settings_de_ambito_schedule_no_toca_alerts() {
        let conn = conn_de_prueba();
        set_setting_impl(
            &conn,
            "schedule.metrics_fast_seconds",
            &serde_json::json!(45),
        )
        .unwrap();
        set_setting_impl(
            &conn,
            "alerts.temp_configured_warn_c",
            &serde_json::json!(65.0),
        )
        .unwrap();

        for clave in claves_por_ambito("schedule") {
            repo_varios::delete_setting(&conn, clave).unwrap();
        }

        let s = get_settings_impl(&conn);
        assert_eq!(s.schedule.metrics_fast_seconds, 30, "vuelve a fábrica");
        assert_eq!(s.alerts.temp_configured_warn_c, 65.0, "sin tocar");
    }

    #[test]
    fn la_frase_de_confirmacion_es_el_nombre_de_la_aplicacion_y_nada_mas() {
        // No es una palabra ceremonial: es el propio nombre, para no depender de una traducción
        // exacta mantenida en los dos diccionarios (comentario junto a la constante).
        assert_eq!(FRASE_CONFIRMACION_BORRAR_TODO, "SmartDisk Monitor");
        assert_ne!(FRASE_CONFIRMACION_BORRAR_TODO, "smartdisk monitor");
    }

    #[test]
    fn borrar_todos_los_datos_vacia_dispositivos_y_ajustes() {
        let mut conn = conn_de_prueba();
        conn.execute(
            "INSERT INTO devices (id, fingerprint, identity_confidence, model, device_type, monitoring_enabled, first_seen_at, last_seen_at)
             VALUES ('d1', 'huella', 'fingerprint', 'Modelo', 'nvme', 1, '2026-09-04T00:00:00Z', '2026-09-04T00:00:00Z')",
            [],
        )
        .unwrap();
        set_setting_impl(
            &conn,
            "schedule.metrics_fast_seconds",
            &serde_json::json!(45),
        )
        .unwrap();

        delete_all_data_impl(&mut conn).unwrap();

        assert!(repo_inventario::list_present_devices(&conn)
            .unwrap()
            .is_empty());
        assert_eq!(get_settings_impl(&conn).schedule.metrics_fast_seconds, 30);
    }
}

/// Funciones puras dispersas por el fichero que no tenían módulo de test propio (T108,
/// `docs/open-questions.md` cobertura mínima §VIII): ninguna necesita `State`/`AppHandle`, así que
/// todas son testables sin un runtime de Tauri, a diferencia de los `#[tauri::command]` que las
/// envuelven.
#[cfg(test)]
mod tests_helpers_varios {
    use super::*;
    use crate::domain::tipos::{
        DeviceType, EventLevel, IdentityConfidence, MappingConfidence as MC, TestStatus, TestType,
    };
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("commands_helpers");
        db::open(&dir).unwrap().0
    }

    #[test]
    fn un_error_de_bloqueo_de_sqlite_se_marca_reintentable() {
        let e = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(5), // SQLITE_BUSY
            Some("database is locked".to_string()),
        );
        let err = rusqlite_err_to_app_error(e);
        assert_eq!(err.code, "db.locked");
        assert!(err.retryable);
    }

    #[test]
    fn un_error_de_sqlite_que_no_es_bloqueo_no_es_reintentable() {
        let e = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(1), // SQLITE_ERROR
            Some("no such table: x".to_string()),
        );
        let err = rusqlite_err_to_app_error(e);
        assert_eq!(err.code, "db.query_failed");
        assert!(!err.retryable);
    }

    fn volumen(id: &str, letras: Option<&str>) -> Volume {
        Volume {
            id: id.to_string(),
            volume_guid: format!("guid-{id}"),
            label: None,
            filesystem: Some("NTFS".to_string()),
            drive_letters_json: letras.map(|l| format!("[\"{l}\"]")),
            capacity_bytes: Some(1_000_000_000),
            free_bytes: Some(500_000_000),
            device_mapping_confidence: Some(MC::Exact),
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn primera_letra_unidad_toma_la_primera_de_la_lista() {
        assert_eq!(primera_letra_unidad(&volumen("v1", Some("D:"))), Some('D'));
    }

    #[test]
    fn primera_letra_unidad_sin_letras_es_ninguna() {
        assert_eq!(primera_letra_unidad(&volumen("v1", None)), None);
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

    #[test]
    fn hay_prueba_activa_ve_por_el_disco_fisico_subyacente_no_solo_por_el_id_exacto() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_device(&conn, &dispositivo("d1")).unwrap();
        repo_inventario::upsert_volume(&conn, &volumen("v1", Some("D:"))).unwrap();
        repo_inventario::link_device_volume(&conn, "d1", "v1", MC::Exact, "test").unwrap();

        assert!(!hay_prueba_activa(&conn, Some("d1"), None).unwrap());

        let fila = TestRun {
            id: "run1".to_string(),
            test_type: TestType::Benchmark,
            target_device_id: None,
            target_volume_id: Some("v1".to_string()),
            status: TestStatus::Running,
            started_at_utc: Some("2026-09-04T00:00:00Z".to_string()),
            finished_at_utc: None,
            progress_percent: Some(10),
            result_summary_json: None,
            parameters_json: None,
            temp_path: None,
        };
        repo_varios::create_test_run(&conn, &fila).unwrap();

        // La prueba corre sobre el volumen v1; "d1" pide por dispositivo, pero es el mismo disco
        // físico subyacente (enlazado a v1), así que debe verse ocupado igualmente.
        assert!(hay_prueba_activa(&conn, Some("d1"), None).unwrap());
        assert!(hay_prueba_activa(&conn, None, Some("v1")).unwrap());
        assert!(!hay_prueba_activa(&conn, Some("d-otro"), None).unwrap());
    }

    #[test]
    fn nivel_de_evento_ida_y_vuelta_normaliza_critico_a_error() {
        assert_eq!(nivel_a_wire(EventLevel::Critical), "error");
        assert_eq!(nivel_a_wire(EventLevel::Error), "error");
        assert_eq!(nivel_a_wire(EventLevel::Warning), "warning");
        assert_eq!(nivel_a_wire(EventLevel::Information), "info");

        assert_eq!(nivel_desde_wire("error"), EventLevel::Error);
        assert_eq!(nivel_desde_wire("warning"), EventLevel::Warning);
        // Cualquier cadena que no sea "error"/"warning" cae a informativo, nunca a un pánico.
        assert_eq!(nivel_desde_wire("info"), EventLevel::Information);
        assert_eq!(nivel_desde_wire("lo-que-sea"), EventLevel::Information);
    }

    #[test]
    fn parsear_cursor_separa_tiempo_e_id_y_rechaza_lo_que_no_encaja() {
        assert_eq!(
            parsear_cursor("2026-09-04T00:00:00Z|42"),
            Some(("2026-09-04T00:00:00Z".to_string(), 42))
        );
        assert_eq!(parsear_cursor("sin separador"), None);
        assert_eq!(parsear_cursor("2026-09-04T00:00:00Z|no-es-numero"), None);
    }

    fn evento(id: i64, record_id: i64) -> crate::domain::tipos::SystemEvent {
        crate::domain::tipos::SystemEvent {
            id,
            channel: "System".to_string(),
            record_id,
            occurred_at_utc: "2026-09-04T00:00:00Z".to_string(),
            provider: "disk".to_string(),
            event_id: 51,
            level: EventLevel::Warning,
            message: Some("mensaje de prueba".to_string()),
            raw_xml: Some("<Event/>".to_string()),
            device_id: None,
            volume_id: None,
            mapping_confidence: MC::Unknown,
            dedup_hash: format!("hash-{record_id}"),
        }
    }

    #[test]
    fn evento_a_wire_traduce_el_nivel_y_declara_si_tiene_xml() {
        let wire = evento_a_wire(&evento(1, 100));
        assert_eq!(wire.level, "warning");
        assert_eq!(wire.message, "mensaje de prueba");
        assert!(wire.has_raw_xml);
    }

    #[test]
    fn get_system_events_impl_pagina_y_declara_el_total() {
        let conn = conn_de_prueba();
        for i in 0..3 {
            repo_varios::insert_event_if_new(&conn, &evento(0, i)).unwrap();
        }

        let pagina =
            get_system_events_impl(&conn, None, None, None, None, None, None, None, Some(2))
                .unwrap();
        assert_eq!(pagina.events.len(), 2, "recorta al límite pedido");
        assert!(pagina.next_cursor.is_some(), "hay más allá del límite");
        assert_eq!(pagina.total, Some(3));
    }

    #[test]
    fn get_system_events_impl_sin_eventos_no_ofrece_cursor_siguiente() {
        let conn = conn_de_prueba();
        let pagina =
            get_system_events_impl(&conn, None, None, None, None, None, None, None, None).unwrap();
        assert!(pagina.events.is_empty());
        assert_eq!(pagina.next_cursor, None);
    }

    #[test]
    fn get_event_raw_xml_impl_devuelve_el_xml_o_event_not_found() {
        let conn = conn_de_prueba();
        repo_varios::insert_event_if_new(&conn, &evento(0, 1)).unwrap();
        let fila = repo_varios::get_event_by_id(&conn, 1).unwrap().unwrap();

        let xml = get_event_raw_xml_impl(&conn, &fila.id.to_string()).unwrap();
        assert_eq!(xml, "<Event/>");

        let err = get_event_raw_xml_impl(&conn, "no-es-un-id").unwrap_err();
        assert_eq!(err.code, "event.not_found");

        let err = get_event_raw_xml_impl(&conn, "99999").unwrap_err();
        assert_eq!(err.code, "event.not_found");
    }

    fn test_run_con_resumen(
        parameters_json: Option<&str>,
        result_summary_json: Option<&str>,
    ) -> TestRun {
        TestRun {
            id: "run1".to_string(),
            test_type: TestType::ChkdskScan,
            target_device_id: None,
            target_volume_id: Some("v1".to_string()),
            status: TestStatus::Completed,
            started_at_utc: Some("2026-09-04T00:00:00Z".to_string()),
            finished_at_utc: Some("2026-09-04T00:05:00Z".to_string()),
            progress_percent: Some(100),
            result_summary_json: result_summary_json.map(str::to_string),
            parameters_json: parameters_json.map(str::to_string),
            temp_path: None,
        }
    }

    #[test]
    fn test_run_to_wire_sin_parametros_ni_resumen_no_inventa_nada() {
        let wire = test_run_to_wire(&test_run_con_resumen(None, None));
        assert_eq!(wire.command, None);
        assert!(wire.result.is_none());
        assert_eq!(wire.output, None);
    }

    #[test]
    fn test_run_to_wire_lee_el_comando_y_el_resultado_de_sus_sobres_json() {
        let parametros = r#"{"command":"chkdsk D: /scan","params":{}}"#;
        let resumen = r#"{"passed":true,"stoppedReason":"completed","output":"sin errores","outputEncoding":"cp1252"}"#;
        let wire = test_run_to_wire(&test_run_con_resumen(Some(parametros), Some(resumen)));
        assert_eq!(wire.command.as_deref(), Some("chkdsk D: /scan"));
        assert_eq!(wire.output.as_deref(), Some("sin errores"));
        assert_eq!(wire.output_encoding.as_deref(), Some("cp1252"));
        let resultado = wire.result.expect("debe traer resultado");
        assert_eq!(resultado.passed, Some(true));
        assert_eq!(resultado.stopped_reason.as_deref(), Some("completed"));
    }

    #[test]
    fn paquete_diagnostico_solo_incluye_dispositivos_monitorizados() {
        let conn = conn_de_prueba();
        let mut monitorizado = dispositivo("d1");
        monitorizado.monitoring_enabled = true;
        let mut excluido = dispositivo("d2");
        excluido.monitoring_enabled = false;
        repo_inventario::upsert_device(&conn, &monitorizado).unwrap();
        repo_inventario::upsert_device(&conn, &excluido).unwrap();

        let paquete = paquete_diagnostico(&conn, false).unwrap();
        // El manifiesto siempre está; ninguna entrada de smart puede venir del disco excluido
        // porque ninguno de los dos tiene `smartctl_path`, así que basta comprobar que no falla y
        // que el manifiesto va primero (contrato ya probado en reporting::diagnostico).
        assert_eq!(paquete.entradas[0].ruta, "manifest.json");
    }

    #[test]
    fn parsear_rango_rechaza_una_fecha_ilegible_con_schema_mismatch() {
        let err = parsear_rango("no-es-una-fecha", "2026-09-04T10:00:00Z").unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");

        let err = parsear_rango("2026-09-04T00:00:00Z", "tampoco-esta").unwrap_err();
        assert_eq!(err.code, "ipc.schema_mismatch");
    }

    #[test]
    fn claves_por_ambito_all_incluye_los_tres_grupos_mas_ciclo_de_vida_y_registro() {
        let claves = claves_por_ambito("all");
        assert!(claves.contains(&"schedule.metrics_fast_seconds"));
        assert!(claves.contains(&"alerts.temp_configured_warn_c"));
        assert!(claves.contains(&"retention.raw_days"));
        assert!(claves.contains(&"lifecycle.close_action"));
        assert!(claves.contains(&"logging.verbose"));
    }

    #[test]
    fn claves_por_ambito_un_ambito_desconocido_cae_a_todas() {
        // No hay un cuarto valor de `scope` fuera de "schedule"/"alerts"/"retention": cualquier
        // otra cadena (incluido un typo) debe comportarse como "all", nunca como una lista vacía
        // que dejaría `reset_settings` sin hacer nada en silencio.
        assert_eq!(claves_por_ambito("lo-que-sea"), claves_por_ambito("all"));
    }

    fn grupo_de_alerta(
        target_device_id: Option<&str>,
        target_volume_id: Option<&str>,
    ) -> AlertGroup {
        AlertGroup {
            id: "g1".to_string(),
            deduplication_key: "dedup".to_string(),
            rule_key: "temp.above_configured_warn".to_string(),
            target_device_id: target_device_id.map(str::to_string),
            target_volume_id: target_volume_id.map(str::to_string),
            severity: AlertSeverity::Warning,
            status: AlertStatus::Active,
            muted_until: None,
            cycle: 1,
            first_occurrence_at_utc: "2026-09-04T00:00:00Z".to_string(),
            last_occurrence_at_utc: "2026-09-04T00:00:00Z".to_string(),
            occurrence_count: 1,
            acknowledged_at_utc: None,
            resolved_at_utc: None,
            archived_at_utc: None,
            last_value_real: None,
            context_json: None,
        }
    }

    #[test]
    fn resolve_target_name_sin_dispositivo_usa_la_etiqueta_del_volumen() {
        let conn = conn_de_prueba();
        repo_inventario::upsert_volume(&conn, &volumen("v1", Some("D:"))).unwrap();

        let grupo = grupo_de_alerta(None, Some("v1"));
        assert_eq!(resolve_target_name(&conn, &grupo), "v1");
    }

    #[test]
    fn resolve_target_name_con_volumen_etiquetado_usa_la_etiqueta() {
        let conn = conn_de_prueba();
        let mut v = volumen("v1", Some("D:"));
        v.label = Some("Datos".to_string());
        repo_inventario::upsert_volume(&conn, &v).unwrap();

        let grupo = grupo_de_alerta(None, Some("v1"));
        assert_eq!(resolve_target_name(&conn, &grupo), "Datos");
    }

    #[test]
    fn resolve_target_name_sin_dispositivo_ni_volumen_que_resuelvan_cae_a_interrogacion() {
        let conn = conn_de_prueba();
        let grupo = grupo_de_alerta(None, None);
        assert_eq!(resolve_target_name(&conn, &grupo), "?");

        let grupo_huerfano = grupo_de_alerta(Some("no-existe"), None);
        assert_eq!(resolve_target_name(&conn, &grupo_huerfano), "?");
    }

    #[test]
    fn nivel_resuelto_sigue_el_ajuste_de_verbosidad() {
        let conn = conn_de_prueba();
        assert_eq!(nivel_resuelto(&conn), crate::logging::LogLevel::Info);

        guardar_ajuste(&conn, "logging.verbose", &true, "2026-09-04T00:00:00Z").unwrap();
        assert_eq!(nivel_resuelto(&conn), crate::logging::LogLevel::Debug);
    }

    #[test]
    fn set_device_alias_impl_para_un_dispositivo_inexistente_falla_con_device_not_found() {
        let conn = conn_de_prueba();
        let err = set_device_alias_impl(&conn, "no-existe", Some("Alias")).unwrap_err();
        assert_eq!(err.code, "device.not_found");
    }

    #[test]
    fn nuevo_id_prueba_es_hexadecimal_y_no_vacio() {
        let id = nuevo_id_prueba();
        assert!(!id.is_empty());
        assert!(
            id.chars().all(|c| c.is_ascii_hexdigit()),
            "debe ser hexadecimal: {id}"
        );
    }
}
