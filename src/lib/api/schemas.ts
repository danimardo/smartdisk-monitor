/** Esquemas de validación de la frontera IPC (constitución §XI).
 *
 *  `invoke<T>()` y `listen<T>()` **no validan nada**: son aserciones de tipo sobre datos que vienen
 *  de otro proceso. Si el backend cambia un campo, TypeScript no se entera y la interfaz muestra
 *  `undefined` como si fuera un dato, que es justo lo que prohíbe el principio I.
 *
 *  Los tipos se infieren de estos esquemas con `z.infer<>`: **no se declaran a mano en paralelo**.
 *  Dos declaraciones de la misma forma acaban divergiendo; una sola no puede.
 */

import { z } from "zod";

/* ------------------------------------------------------------------ básicos */

/** Fecha ISO en UTC. El backend siempre persiste en UTC (principio V). */
const isoUtc = z.string().min(20, "se esperaba una fecha ISO en UTC");

export const healthState = z.enum(["ok", "warn", "crit", "unknown"]);
export const severity = z.enum(["info", "warn", "crit"]);
export const alertStatus = z.enum(["active", "acknowledged", "resolved", "archived"]);
export const metricSource = z.enum(["smartctl", "windows-storage", "perf-counter", "filesystem"]);
export const metricQuality = z.enum(["exact", "inferred", "vendor_specific", "stale"]);
export const mappingConfidence = z.enum(["exact", "inferred", "unknown"]);
export const identityConfidence = z.enum(["serial", "fingerprint"]);
export const sourceStatus = z.enum(["ok", "partial", "unsupported", "timeout", "error"]);
export const resolution = z.enum(["raw", "five_minutes", "hourly"]);
export const unknownReason = z.enum([
  "unsupported",
  "unreadable",
  "collector-error",
  "not-yet-sampled",
  "paused"
]);
export const testStatus = z.enum([
  "pending",
  "running",
  "cancelling",
  "completed",
  "failed",
  "cancelled",
  "interrupted"
]);

/** `.nullable()` y no `.optional()` a propósito: el contrato dice que un dato que no se ha podido
 *  obtener viaja como `null` explícito, nunca como campo ausente (principio I). */
const nullableNumber = z.number().nullable();

export const appError = z.object({
  code: z.string(),
  messageKey: z.string(),
  messageVars: z.record(z.string(), z.union([z.string(), z.number()])).optional(),
  detail: z.string().nullable().optional(),
  source: metricSource.nullable().optional(),
  retryable: z.boolean()
});

export const provenance = z.object({
  source: metricSource,
  quality: metricQuality,
  readAt: isoUtc.optional()
});

/* --------------------------------------------------------------- inventario */

export const volumeSummary = z.object({
  id: z.string(),
  label: z.string(),
  driveLetters: z.array(z.string()),
  capacityBytes: nullableNumber,
  freeBytes: nullableNumber,
  mappingConfidence
});

export const diskSummary = z.object({
  id: z.string(),
  alias: z.string().nullable().optional(),
  model: z.string(),
  deviceType: z.string(),
  state: healthState,
  temperatureC: nullableNumber,
  percentageUsed: nullableNumber,
  activityPercent: nullableNumber,
  powerOnHours: nullableNumber,
  vendorTempLimitC: nullableNumber.optional(),
  vendorTempCriticalC: nullableNumber.optional(),
  unknownReason: unknownReason.nullable().optional(),
  lastReadAt: isoUtc.nullable().optional(),
  provenance: provenance.nullable().optional(),
  volumes: z.array(volumeSummary)
});

export const sourceHealth = z.object({
  source: metricSource,
  status: sourceStatus,
  lastSuccessAt: isoUtc.nullable(),
  lastAttemptAt: isoUtc.nullable(),
  error: appError.nullable().optional()
});

export const deviceListResponse = z.object({
  devices: z.array(diskSummary),
  excluded: z.array(diskSummary),
  sources: z.array(sourceHealth),
  paused: z.boolean(),
  pausedSince: isoUtc.nullable()
});

export const deviceCapability = z.object({
  key: z.enum(["smart", "nvme_log", "self_test_short", "chkdsk_scan", "temperature"]),
  available: z.boolean(),
  reasonKey: z.string().nullable()
});

export const smartCounter = z.object({
  metricKey: z.string(),
  value: nullableNumber,
  unit: z.string().nullable(),
  delta: nullableNumber,
  deltaIsMeaningful: z.boolean(),
  provenance
});

export const deviceDetail = diskSummary.extend({
  fingerprint: z.string(),
  identityConfidence,
  serialNumber: z.string().nullable(),
  firmware: z.string().nullable(),
  busType: z.string().nullable(),
  capabilities: z.array(deviceCapability),
  counters: z.array(smartCounter),
  firstSeenAt: isoUtc,
  lastSeenAt: isoUtc,
  removedAt: isoUtc.nullable()
});

/* -------------------------------------------------------------------- series */

export const metricSeries = z.object({
  metricKey: z.string(),
  unit: z.string(),
  resolution,
  downsampled: z.boolean(),
  fromUtc: isoUtc,
  toUtc: isoUtc,
  expectedIntervalMs: z.number().positive(),
  // `v: null` es un hueco explícito y debe llegar como tal: el gráfico lo dibuja como hueco.
  points: z.array(z.object({ t: z.number(), v: nullableNumber })),
  vendorLimit: nullableNumber,
  vendorCritical: nullableNumber
});

/* ------------------------------------------------------------------ alertas */

export const alertGroup = z.object({
  id: z.string(),
  ruleKey: z.string(),
  deduplicationKey: z.string(),
  severity,
  status: alertStatus,
  title: z.string(),
  summary: z.string(),
  count: z.number().int().nonnegative(),
  firstOccurredAt: isoUtc,
  lastOccurredAt: isoUtc,
  target: z.string(),
  mutedUntil: z
    .union([isoUtc, z.literal("infinite")])
    .nullable()
    .optional(),
  cycle: z.number().int().nonnegative().optional()
});

export const alertDetail = alertGroup.extend({
  facts: z.array(z.object({ labelKey: z.string(), value: z.string().nullable() })),
  occurrences: z.array(
    z.object({
      occurredAt: isoUtc,
      cycle: z.number().int().nonnegative(),
      value: nullableNumber,
      eventId: z.string().nullable(),
      context: z.string().nullable()
    })
  ),
  relatedEvents: z.array(z.lazy(() => systemEvent))
});

/* ------------------------------------------------------------------ eventos */

export const systemEvent = z.object({
  id: z.string(),
  occurredAt: isoUtc,
  provider: z.string(),
  eventId: z.number().int(),
  level: z.enum(["error", "warning", "info"]),
  message: z.string(),
  deviceId: z.string().nullable(),
  volumeId: z.string().nullable(),
  mappingConfidence,
  hasRawXml: z.boolean()
});

export const systemEventPage = z.object({
  events: z.array(systemEvent),
  nextCursor: z.string().nullable(),
  total: z.number().int().nonnegative().nullable()
});

/* ------------------------------------------------------------------ pruebas */

export const testResult = z.object({
  passed: z.boolean().nullable(),
  readBytesPerSecond: nullableNumber,
  writeBytesPerSecond: nullableNumber,
  readLatencyMs: nullableNumber,
  writeLatencyMs: nullableNumber,
  maxTemperatureC: nullableNumber,
  stoppedReason: z.enum(["completed", "cancelled", "thermal", "space", "error"]).nullable()
});

export const testRun = z.object({
  id: z.string(),
  type: z.enum(["benchmark", "chkdsk_scan", "smart_short"]),
  deviceId: z.string().nullable(),
  volumeId: z.string().nullable(),
  status: testStatus,
  startedAt: isoUtc,
  finishedAt: isoUtc.nullable(),
  progressPercent: nullableNumber,
  command: z.string().nullable(),
  parameters: z.record(z.string(), z.unknown()),
  result: testResult.nullable(),
  output: z.string().nullable(),
  outputEncoding: z.string().nullable(),
  orphanPath: z.string().nullable()
});

/* ------------------------------------------------------- apariencia y varios */

export const appearanceSettings = z.object({
  theme: z.enum(["light", "dark", "system"]),
  language: z.enum(["es", "en"]).nullable(),
  systemLocale: z.string().min(2),
  useSystemAccent: z.boolean()
});

export const windowsAccent = z.object({
  hex: z.string().regex(/^#[0-9a-fA-F]{6}$/, "se esperaba #RRGGBB"),
  palette: z.array(z.string().regex(/^#[0-9a-fA-F]{6}$/)).optional()
});

export const diagnosticPreview = z.object({
  entries: z.array(
    z.object({ path: z.string(), sizeBytes: z.number().nonnegative(), descriptionKey: z.string() })
  ),
  totalBytes: z.number().nonnegative(),
  redactedFields: z.array(z.string())
});

export const appInfo = z.object({
  name: z.string(),
  version: z.string(),
  author: z.string()
});

/* ------------------------------------------------------ eventos que empujan */

const emitted = { emittedAt: isoUtc };

export const metricsUpdated = z.object({
  ...emitted,
  devices: z.array(diskSummary),
  sources: z.array(sourceHealth)
});

export const alertsChanged = z.object({
  ...emitted,
  changed: z.array(alertGroup),
  removed: z.array(z.string())
});

export const inventoryChanged = z.object({
  ...emitted,
  added: z.array(diskSummary),
  removed: z.array(z.string()),
  updated: z.array(diskSummary)
});

export const testProgress = z.object({ ...emitted, testRun });
export const sourceDegraded = z.object({ ...emitted, source: sourceHealth });
export const monitoringPaused = z.object({ ...emitted, since: isoUtc.nullable() });
export const accentChanged = z.object({ hex: z.string().regex(/^#[0-9a-fA-F]{6}$/) });
export const themeChanged = z.object({ dark: z.boolean() });

/** Esquema por evento. La clave es el nombre exacto del contrato: si el backend emite un evento que
 *  no está aquí, no hay forma de suscribirse a él, que es justo lo que se quiere. */
export const eventSchemas = {
  "metrics:updated": metricsUpdated,
  "alerts:changed": alertsChanged,
  "inventory:changed": inventoryChanged,
  "test:progress": testProgress,
  "source:degraded": sourceDegraded,
  "system:accent-changed": accentChanged,
  "system:theme-changed": themeChanged,
  "monitoring:paused": monitoringPaused,
  "monitoring:resumed": monitoringPaused
} as const;

/* ---------------------------------------------------------- tipos inferidos */

export type AppErrorShape = z.infer<typeof appError>;
export type DiskSummaryShape = z.infer<typeof diskSummary>;
export type DeviceListResponseShape = z.infer<typeof deviceListResponse>;
export type DeviceDetailShape = z.infer<typeof deviceDetail>;
export type MetricSeriesShape = z.infer<typeof metricSeries>;
export type AlertGroupShape = z.infer<typeof alertGroup>;
export type AlertDetailShape = z.infer<typeof alertDetail>;
export type SystemEventShape = z.infer<typeof systemEvent>;
export type SystemEventPageShape = z.infer<typeof systemEventPage>;
export type TestRunShape = z.infer<typeof testRun>;
export type AppearanceSettingsShape = z.infer<typeof appearanceSettings>;
export type WindowsAccentShape = z.infer<typeof windowsAccent>;
export type EventName = keyof typeof eventSchemas;
export type EventPayload<K extends EventName> = z.infer<(typeof eventSchemas)[K]>;
