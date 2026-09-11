/** Tipos del contrato UI ↔ backend (`docs/ui-contract.md`).
 *
 *  Los que ya tienen un comando Rust real detrás se generan con `ts-rs` en `./generated/` y se
 *  reexportan aquí (`open-questions.md` G.3): `cargo test` los regenera, y una diferencia con lo
 *  versionado es una señal de que el contrato cambió sin actualizar este fichero. **No se declaran
 *  a mano en paralelo** — es justo lo que ese punto existe para evitar.
 *
 *  Lo que todavía no tiene comando conectado sigue a mano más abajo, con la misma advertencia:
 *  en cuanto su comando exista, se genera y se reexporta igual que los de arriba.
 *
 *  El vocabulario compartido con los componentes vive en `$lib/design/types`, no se duplica aquí.
 */

import type { AlertGroup, AlertStatus } from "$lib/design/types";

export type { AppError } from "./generated/AppError";
export type { AppearanceSettings } from "./generated/AppearanceSettings";
export type { AppInfo } from "./generated/AppInfo";
export type { DeviceListResponse } from "./generated/DeviceListResponse";
export type { SourceHealth } from "./generated/SourceHealth";
export type { WindowsAccent } from "./generated/WindowsAccent";
export type { DeviceCapability } from "./generated/DeviceCapability";
export type { SmartCounter } from "./generated/SmartCounter";
export type { DeviceDetail } from "./generated/DeviceDetail";
export type { IdentityConfidence } from "./generated/IdentityConfidence";
export type { Resolution } from "./generated/Resolution";
/** El intervalo pedido, devuelto tal cual: el eje lo cubre entero aunque falten datos. */
export type { MetricSeriesWire as MetricSeries } from "./generated/MetricSeriesWire";
export type { PuntoSerieWire } from "./generated/PuntoSerieWire";

// Ayuda con IA (spec 005-explicacion-ia). Comandos: estado_ia, guardar_clave_ia, borrar_clave_ia,
// probar_clave_ia, listar_modelos_ia, explicar_detalle_tecnico.
export type { EstadoIaWire } from "./generated/EstadoIaWire";
export type { AiSettingsWire } from "./generated/AiSettingsWire";
export type { ModeloIaWire } from "./generated/ModeloIaWire";
export type { ExplicacionIaWire } from "./generated/ExplicacionIaWire";
export type { RevisionAnonimizacionWire } from "./generated/RevisionAnonimizacionWire";
export type { FragmentoDudosoWire } from "./generated/FragmentoDudosoWire";
export type { ResultadoExplicacion } from "./generated/ResultadoExplicacion";
export type { OrigenExplicacion } from "./generated/OrigenExplicacion";
export type { TipoOrigen } from "./generated/TipoOrigen";
export type { RevisionEnvio } from "./generated/RevisionEnvio";

export type MappingConfidence = "exact" | "inferred" | "unknown";
export type SourceStatus = "ok" | "partial" | "unsupported" | "timeout" | "error";

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
  /** `false` para las reglas de daño físico / predicción de fallo (ADR-044): la acción «Ignorar»
   *  se presenta deshabilitada con su motivo. La lista canónica vive solo en el backend. */
  ruleIgnorable: boolean;
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

export type BenchmarkProfile = "seq1m_q8" | "seq1m_q1" | "rnd4k_q32" | "rnd4k_q1";

export interface BenchmarkRow {
  profile: BenchmarkProfile;
  direction: "read" | "write";
  /** MB decimales por segundo, como CrystalDiskMark. */
  mbPerSecond: number;
  iops: number;
  avgLatencyMs: number;
  actualDurationS: number;
  bytesMoved: number;
  /** `true` si la duración se recortó por el tope de datos de escritura (ADR-053, D3). */
  dataCapHit: boolean;
}

export interface BenchmarkResult {
  tool: "diskspd";
  toolVersion: string;
  fileSizeBytes: number;
  rows: BenchmarkRow[];
  notRun: { profile: BenchmarkProfile; direction: "read" | "write" }[];
}

export interface TestResult {
  passed: boolean | null;
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "error" | null;
  /** Solo en la prueba de Rendimiento (`type === "benchmark"`). */
  benchmark: BenchmarkResult | null;
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

/** Vista previa de `preview_informe_ia` (spec 009): el texto exacto y anonimizado por disco. */
export interface PreviewInformeIa {
  discos: {
    deviceId: string;
    deviceLabel: string;
    textoEnviado: string;
    fragmentos: { texto: string; motivoKey: string }[];
    recortado: boolean;
  }[];
  redactedFields: string[];
  totalLlamadas: number;
}

export type AlertStatusFilter = AlertStatus[];
