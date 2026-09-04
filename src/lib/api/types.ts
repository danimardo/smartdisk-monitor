/** Tipos del contrato UI ↔ backend (`docs/ui-contract.md`).
 *
 *  **Esto es provisional.** Estos tipos se escriben a mano solo mientras no exista el backend; en
 *  cuanto haya comandos de verdad se generan desde Rust con `ts-rs` y CI comprueba que el fichero
 *  generado coincide con el versionado (`open-questions.md` G.3). Escribirlos dos veces a mano es
 *  exactamente el problema que ese punto pretende evitar.
 *
 *  El vocabulario compartido con los componentes vive en `$lib/design/types`, no se duplica aquí.
 */

import type {
  AlertGroup,
  AlertStatus,
  AppError,
  DiskSummary,
  MetricSource,
  Provenance
} from "$lib/design/types";

export type Resolution = "raw" | "five_minutes" | "hourly";
export type MappingConfidence = "exact" | "inferred" | "unknown";
export type IdentityConfidence = "serial" | "fingerprint";
export type SourceStatus = "ok" | "partial" | "unsupported" | "timeout" | "error";

export interface SourceHealth {
  source: MetricSource;
  status: SourceStatus;
  lastSuccessAt: string | null;
  lastAttemptAt: string | null;
  error?: AppError | null;
}

export interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "es" | "en" | null;
  /** BCP-47 de Windows. Se usa este, no `navigator.language`. */
  systemLocale: string;
  useSystemAccent: boolean;
}

export interface WindowsAccent {
  /** #RRGGBB. El registro lo guarda en ABGR: la conversión es del backend. */
  hex: string;
  /** Tonos de `AccentPalette`, del más claro al más oscuro. */
  palette?: string[];
}

export interface DeviceListResponse {
  devices: DiskSummary[];
  /** Desactivados por el usuario. US-011 exige mostrarlos aparte, no ocultarlos. */
  excluded: DiskSummary[];
  sources: SourceHealth[];
  paused: boolean;
  pausedSince: string | null;
}

export interface DeviceCapability {
  key: "smart" | "nvme_log" | "self_test_short" | "chkdsk_scan" | "temperature";
  available: boolean;
  reasonKey: string | null;
}

export interface SmartCounter {
  metricKey: string;
  value: number | null;
  unit: string | null;
  delta: number | null;
  deltaIsMeaningful: boolean;
  provenance: Provenance;
}

export interface DeviceDetail extends DiskSummary {
  fingerprint: string;
  identityConfidence: IdentityConfidence;
  serialNumber: string | null;
  firmware: string | null;
  busType: string | null;
  capabilities: DeviceCapability[];
  counters: SmartCounter[];
  firstSeenAt: string;
  lastSeenAt: string;
  removedAt: string | null;
}

export interface MetricSeries {
  metricKey: string;
  unit: string;
  resolution: Resolution;
  downsampled: boolean;
  /** El intervalo pedido, devuelto tal cual: el eje lo cubre entero aunque falten datos. */
  fromUtc: string;
  toUtc: string;
  expectedIntervalMs: number;
  points: { t: number; v: number | null }[];
  vendorLimit: number | null;
  vendorCritical: number | null;
}

export interface SystemEvent {
  id: string;
  occurredAt: string;
  provider: string;
  eventId: number;
  level: "error" | "warning" | "info";
  /** En el idioma de Windows, no en el de la aplicación. Se renderiza como texto, nunca como HTML. */
  message: string;
  deviceId: string | null;
  volumeId: string | null;
  mappingConfidence: MappingConfidence;
  hasRawXml: boolean;
}

export interface SystemEventPage {
  events: SystemEvent[];
  nextCursor: string | null;
  total: number | null;
}

export interface AlertDetail extends AlertGroup {
  facts: { labelKey: string; value: string | null }[];
  occurrences: {
    occurredAt: string;
    cycle: number;
    value: number | null;
    eventId: string | null;
    context: string | null;
  }[];
  relatedEvents: SystemEvent[];
}

export type TestType = "benchmark" | "chkdsk_scan" | "smart_short";

export interface TestResult {
  passed: boolean | null;
  readBytesPerSecond: number | null;
  writeBytesPerSecond: number | null;
  readLatencyMs: number | null;
  writeLatencyMs: number | null;
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "space" | "error" | null;
}

export interface TestRun {
  id: string;
  type: TestType;
  deviceId: string | null;
  volumeId: string | null;
  status: "pending" | "running" | "cancelling" | "completed" | "failed" | "cancelled" | "interrupted";
  startedAt: string;
  finishedAt: string | null;
  progressPercent: number | null;
  /** Comando literal ya formado. La UI lo muestra; nunca lo construye. */
  command: string | null;
  parameters: Record<string, unknown>;
  result: TestResult | null;
  output: string | null;
  /** Página de códigos deducida para `output`. Los bytes originales se conservan aparte. */
  outputEncoding: string | null;
  orphanPath: string | null;
}

export interface DiagnosticPreview {
  entries: { path: string; sizeBytes: number; descriptionKey: string }[];
  totalBytes: number;
  redactedFields: string[];
}

export type AlertStatusFilter = AlertStatus[];
