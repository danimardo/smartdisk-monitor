/** Vocabulario compartido de la UI. Refleja el modelo de datos (docs/data-model.md). */

/** Estado de salud presentable. `unknown` cubre "no compatible" y "sin datos": nunca se pinta en rojo. */
export type HealthState = "ok" | "warn" | "crit" | "unknown";

/** Severidad de una alerta (alert_groups.severity). */
export type Severity = "info" | "warn" | "crit";

/** Ciclo de vida de un grupo de alertas (spec §5). El silencio **no** es un estado: es ortogonal
 *  y vive en `mutedUntil`. Una alerta puede estar activa y silenciada a la vez. */
export type AlertStatus = "active" | "acknowledged" | "resolved" | "archived" | "ignored";

/** Por qué un dato es desconocido. Distingue lo normal de lo averiado: un USB que no expone SMART
 *  es `unsupported` y no ensucia el estado global; un disco que debería responder y no responde es
 *  `unreadable` y sí cuenta como advertencia (véase `unknownContributesWarning`). */
export type UnknownReason = "unsupported" | "unreadable" | "collector-error" | "not-yet-sampled" | "paused";

/** Estado de una ejecución de prueba (test_runs.status). */
export type TestStatus =
  "pending" | "running" | "cancelling" | "completed" | "failed" | "cancelled" | "interrupted";

/** Procedencia y confianza de una métrica (metric_samples.source/quality). */
export type MetricSource = "smartctl" | "windows-storage" | "perf-counter" | "filesystem";
export type MetricQuality = "exact" | "inferred" | "vendor_specific" | "stale";

export type ThemePreference = "light" | "dark" | "system";

export interface Provenance {
  source: MetricSource;
  quality: MetricQuality;
  /** ISO UTC; la UI la presenta en hora local. */
  readAt?: string;
}

/** Error devuelto por cualquier comando Tauri. `AGENTS.md` §5 exige frase humana visible y detalle
 *  técnico conservado: `messageKey` es la clave i18n de la frase, `detail` el texto técnico crudo
 *  que se muestra dentro de un `<details>` y se copia al portapapeles. Nunca se enseña `detail` solo. */
export interface AppError {
  /** Identificador estable, apto para ramificar en la UI: "smartctl.timeout", "db.locked", … */
  code: string;
  /** Clave i18n de la explicación en lenguaje humano. */
  messageKey: string;
  /** Interpolaciones de `messageKey` (nombre de disco, ruta, número de intentos…). */
  messageVars?: Record<string, string | number>;
  /** Texto técnico literal: stderr, código de salida, mensaje de SQLite. Nunca traducido. */
  detail?: string | null;
  /** Qué fuente falló, cuando aplique: permite degradar una tarjeta y no la aplicación entera. */
  source?: MetricSource | null;
  /** true si repetir la misma acción tiene sentido (timeout, bloqueo temporal). */
  retryable: boolean;
}

export interface DiskSummary {
  id: string;
  alias?: string | null;
  model: string;
  deviceType: string;
  state: HealthState;
  /** null = no disponible. Nunca 0 inventado. */
  temperatureC: number | null;
  percentageUsed: number | null;
  activityPercent: number | null;
  powerOnHours: number | null;
  vendorTempLimitC?: number | null;
  /** Umbral crítico del fabricante, si lo declara; por debajo de él manda el configurado en ajustes. */
  vendorTempCriticalC?: number | null;
  /** Autoevaluación SMART global (`smart_status.passed`): `true` superada, `false` fallida, `null`
   *  sin dato o disco sin SMART. La consume el primer hecho del `HeroPanel` (ADR-041). */
  smartHealthPassed?: boolean | null;
  /** Presente solo cuando `state === "unknown"`: explica por qué y decide si cuenta como advertencia. */
  unknownReason?: UnknownReason | null;
  /** Última lectura válida de cualquier fuente. Alimenta la marca de dato obsoleto. */
  lastReadAt?: string | null;
  /** Fuente y calidad del bloque principal de métricas. */
  provenance?: Provenance | null;
  volumes: VolumeSummary[];
}

export interface VolumeSummary {
  id: string;
  label: string;
  driveLetters: string[];
  capacityBytes: number | null;
  freeBytes: number | null;
  mappingConfidence: "exact" | "inferred" | "unknown";
  /** `chkdsk /scan` solo existe en NTFS: lo decide el backend, no se repite el criterio aquí. */
  chkdskAvailable: boolean;
  /** `true` para el volumen donde vive Windows (v3, ADR-036). Lo calcula el backend; la interfaz no
   *  lo infiere. Lo consume `selectHeroDisk()`. */
  isSystemVolume: boolean;
}

export interface AlertGroup {
  id: string;
  /** El backend no manda texto de interfaz (ADR-030): el título y el resumen se resuelven en el
   *  componente con `t(\`alert.rule.${ruleKey}.title\`)` / `.summary`, una clave por regla. */
  ruleKey: string;
  deduplicationKey: string;
  severity: Severity;
  status: AlertStatus;
  count: number;
  firstOccurredAt: string;
  lastOccurredAt: string;
  target: string;
  /** Silencio de la notificación. `null` = no silenciada; una fecha ISO UTC = silenciada hasta
   *  ese momento; `"infinite"` = hasta reactivación manual. **Nunca afecta al color** (ADR-016).
   *
   *  El tipo es `string | null` y no `string | "infinite" | null` porque el literal quedaría
   *  absorbido por `string` sin aportar nada; el valor especial se documenta aquí y lo valida el
   *  esquema Zod, que sí puede distinguirlos. */
  mutedUntil?: string | null;
  /** Ciclo de recaída: se incrementa cada vez que el grupo se resuelve y vuelve a activarse,
   *  para que la cronología distinga episodios (US-030). */
  cycle?: number;
}
