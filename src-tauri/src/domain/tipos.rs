//! Vocabulario mínimo de dominio para la persistencia. Sin dependencias de Tauri, Windows ni
//! SQLite: es lo que `persistence/` traduce desde y hacia filas, y lo que el resto de `domain/`
//! usa para razonar. Los campos siguen `docs/data-model.md` §2.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum IdentityConfidence {
    Serial,
    Fingerprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum DeviceType {
    Nvme,
    SataSsd,
    Hdd,
    Usb,
    Virtual,
    RaidLogical,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub fingerprint: String,
    pub identity_confidence: IdentityConfidence,
    pub serial_number: Option<String>,
    pub model: String,
    pub manufacturer: Option<String>,
    pub firmware: Option<String>,
    pub device_type: DeviceType,
    pub bus_type: Option<String>,
    pub smartctl_path: Option<String>,
    pub capacity_bytes: Option<i64>,
    pub alias: Option<String>,
    pub monitoring_enabled: bool,
    /// UTC en ISO-8601 (`docs/data-model.md` §5): la interfaz formatea en local, el dato viaja en UTC.
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub removed_at: Option<String>,
    pub capabilities_json: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum MappingConfidence {
    Exact,
    Inferred,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    pub id: String,
    pub volume_guid: String,
    pub label: Option<String>,
    pub filesystem: Option<String>,
    pub drive_letters_json: Option<String>,
    pub capacity_bytes: Option<i64>,
    pub free_bytes: Option<i64>,
    pub device_mapping_confidence: Option<MappingConfidence>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

/// El wire (`src/lib/design/types.ts` `MetricSource`) usa kebab-case salvo
/// `perf-counter`, que abrevia "performance" — ninguna convención de `rename_all` lo produce sola.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum MetricSource {
    Smartctl,
    WindowsStorage,
    #[serde(rename = "perf-counter")]
    #[ts(rename = "perf-counter")]
    PerformanceCounter,
    Filesystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum MetricQuality {
    Exact,
    Inferred,
    VendorSpecific,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum Resolution {
    Raw,
    FiveMinutes,
    Hourly,
}

/// El objetivo de una muestra: exactamente uno de los dos, nunca ambos ni ninguno — la misma
/// restricción que impone el `CHECK` de la migración, expresada en el tipo para que no se pueda
/// construir un valor inválido en Rust.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricTarget {
    Device(String),
    Volume(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSample {
    pub target: MetricTarget,
    pub metric_key: String,
    /// Un dato ausente **no se almacena como cero**: se omite (`docs/data-model.md` §3). Este tipo
    /// no admite "cero por defecto": el llamante debe decidir entre `value_real` y `value_integer`.
    pub value_real: Option<f64>,
    pub value_integer: Option<i64>,
    pub unit: String,
    pub sampled_at_utc: String,
    pub source: MetricSource,
    pub quality: MetricQuality,
    pub resolution: Resolution,
}

/// El cable usa `warn`/`crit`, no `warning`/`critical`: son las dos únicas cadenas del vocabulario
/// compartido `Severity` (`src/lib/design/types.ts`) que este dato puede tomar. El almacenamiento
/// en SQLite es una representación distinta —`alert_groups.severity` guarda `warning`/`critical`,
/// que es lo que exige el `CHECK` de la migración— y vive aparte en
/// `persistence::repo_alertas::severity_to_str`/`severity_from_str` (mismo patrón que `DeviceType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum AlertSeverity {
    #[serde(rename = "warn")]
    #[ts(rename = "warn")]
    Warning,
    #[serde(rename = "crit")]
    #[ts(rename = "crit")]
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertGroup {
    pub id: String,
    pub deduplication_key: String,
    pub rule_key: String,
    pub target_device_id: Option<String>,
    pub target_volume_id: Option<String>,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    /// Ortogonal al estado: el silencio nunca decide el color (constitución §I, `docs/ui-design.md`).
    pub muted_until: Option<String>,
    pub cycle: i64,
    pub first_occurrence_at_utc: String,
    pub last_occurrence_at_utc: String,
    pub occurrence_count: i64,
    pub acknowledged_at_utc: Option<String>,
    pub resolved_at_utc: Option<String>,
    pub archived_at_utc: Option<String>,
    pub last_value_real: Option<f64>,
    pub context_json: Option<String>,
}

/// Una fila de `alert_occurrences`: la cronología completa de un grupo, más antigua a más
/// reciente en la tabla, pero se consulta de más reciente a más antigua (`repo_alertas::
/// list_occurrences`) porque es como se lee una cronología.
#[derive(Debug, Clone)]
pub struct AlertOccurrence {
    pub occurred_at_utc: String,
    pub cycle: i64,
    pub value_real: Option<f64>,
    pub triggering_event_id: Option<i64>,
    pub context_json: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Critical,
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Autoincremento de `system_events.id`, no la identidad real del evento (que es
    /// `(channel, record_id)`): `0` antes de insertar, poblado al leer de la base.
    pub id: i64,
    pub channel: String,
    pub record_id: i64,
    pub occurred_at_utc: String,
    pub provider: String,
    pub event_id: i64,
    pub level: EventLevel,
    pub message: Option<String>,
    pub raw_xml: Option<String>,
    pub device_id: Option<String>,
    pub volume_id: Option<String>,
    pub mapping_confidence: MappingConfidence,
    pub dedup_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestType {
    Benchmark,
    ChkdskScan,
    SmartShort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Pending,
    Running,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestRun {
    pub id: String,
    pub test_type: TestType,
    pub target_device_id: Option<String>,
    pub target_volume_id: Option<String>,
    pub status: TestStatus,
    pub started_at_utc: Option<String>,
    pub finished_at_utc: Option<String>,
    pub progress_percent: Option<i64>,
    pub result_summary_json: Option<String>,
    pub parameters_json: Option<String>,
    pub temp_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateResolution {
    FiveMinutes,
    Hourly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricAggregate {
    pub target: MetricTarget,
    pub metric_key: String,
    pub bucket_start_utc: String,
    pub bucket_end_utc: String,
    pub resolution: AggregateResolution,
    pub value_min: Option<f64>,
    pub value_max: Option<f64>,
    pub value_avg: Option<f64>,
    pub value_first: Option<f64>,
    pub value_last: Option<f64>,
    pub sample_count: i64,
    pub unit: String,
}

/// Espejo de `HealthState` en `src/lib/design/types.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum HealthState {
    Ok,
    Warn,
    Crit,
    Unknown,
}

/// Espejo de `UnknownReason` en `src/lib/design/types.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum UnknownReason {
    Unsupported,
    Unreadable,
    CollectorError,
    NotYetSampled,
    Paused,
}
