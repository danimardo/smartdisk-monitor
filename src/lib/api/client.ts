/** Único punto por el que la interfaz habla con el backend.
 *
 *  **Ninguna pantalla llama a `invoke` directamente** (`docs/engineering-conventions.md` §2). Todo
 *  pasa por aquí, que es donde viven los tipos, la normalización de errores y —cuando se generen
 *  desde Rust— los DTO. Así, cuando una firma cambia, rompe en un sitio y no en once.
 */

import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { z, type ZodType } from "zod";
import * as S from "./schemas";
import type { AppError, DiskSummary } from "$lib/design/types";
import type { AlertStatusFilter } from "./types";

/** Convierte cualquier rechazo en un `AppError` con forma completa.
 *
 *  El backend siempre debería devolver uno, pero un fallo de serialización, un comando que no
 *  existe o un pánico en Rust llegan aquí como cadena o como `Error`. La interfaz exige frase
 *  humana **y** detalle técnico (`AGENTS.md` §5), así que nunca se deja pasar un error sin forma:
 *  una pantalla no debería tener que comprobar de qué tipo es lo que ha capturado.
 */
export function toAppError(cause: unknown): AppError {
  if (cause && typeof cause === "object" && "code" in cause && "messageKey" in cause) {
    return cause as AppError;
  }
  const detail =
    cause instanceof Error
      ? `${cause.name}: ${cause.message}`
      : typeof cause === "string"
        ? cause
        : JSON.stringify(cause);
  return {
    code: "ipc.unexpected",
    messageKey: "error.unexpected",
    detail,
    retryable: false
  };
}

/** Convierte un fallo de esquema en `AppError`.
 *
 *  Un dato con forma inesperada es un dato desconocido, y el principio I dice qué hacer con eso:
 *  decirlo, no rellenarlo con un valor por defecto. El detalle técnico lleva la ruta del campo y lo
 *  que se esperaba, que es lo único que permite diagnosticar una divergencia de contrato.
 */
function schemaError(command: string, issues: { path: PropertyKey[]; message: string }[]): AppError {
  const detalle = issues
    .slice(0, 5)
    .map((i) => `${i.path.join(".") || "(raíz)"}: ${i.message}`)
    .join("; ");
  return {
    code: "ipc.schema_mismatch",
    messageKey: "error.schemaMismatch",
    messageVars: { command },
    detail: `${command} -> ${detalle}`,
    retryable: false
  };
}

/** `invoke` validado. Lanza siempre un `AppError`, nunca otra cosa.
 *
 *  El esquema es obligatorio salvo para los comandos que no devuelven nada: `invoke<T>` es una
 *  aserción de tipo, no una comprobación (constitución §XI). */
async function call<T>(command: string, schema: ZodType<T>, args?: Record<string, unknown>): Promise<T> {
  let raw: unknown;
  try {
    raw = await invoke(command, args);
  } catch (cause) {
    throw toAppError(cause);
  }
  const result = schema.safeParse(raw);
  if (!result.success) {
    throw schemaError(command, result.error.issues);
  }
  return result.data;
}

/** Comandos que no devuelven dato. No hay nada que validar, pero el error sí se normaliza. */
async function callVoid(command: string, args?: Record<string, unknown>): Promise<void> {
  try {
    await invoke(command, args);
  } catch (cause) {
    throw toAppError(cause);
  }
}

/* ---------------------------------------------------------------- apariencia */

export const getAppearanceSettings = () => call("get_appearance_settings", S.appearanceSettings);
export const getSystemAccentColor = () => call("get_system_accent_color", S.windowsAccent);

/** Guarda una clave tipada de `settings`. El backend valida el rango: si no cabe, `AppError`. */
export const setSetting = (key: string, value: unknown) => callVoid("set_setting", { key, value });

export const getSettings = () => call("get_settings", S.settings);

/** Vuelve a los valores de fábrica de un ámbito. `"all"` incluye ciclo de vida, notificaciones y
 *  registro, que no tienen ámbito propio; nunca toca la apariencia. */
export const resetSettings = (scope: "all" | "alerts" | "schedule" | "retention") =>
  call("reset_settings", S.settings, { scope });

/* ---------------------------------------------------------------- inventario */

export const getDevices = () => call("get_devices", S.deviceListResponse);
export const getDeviceDetail = (deviceId: string) => call("get_device_detail", S.deviceDetail, { deviceId });
export const setDeviceMonitoring = (deviceId: string, enabled: boolean) =>
  callVoid("set_device_monitoring", { deviceId, enabled });
export const setDeviceAlias = (deviceId: string, alias: string | null) =>
  callVoid("set_device_alias", { deviceId, alias });

/** Idempotente: si ya hay una recopilación igual en curso no encola otra (US-013). */
export const refreshNow = (scope: "all" | "device", deviceId?: string) =>
  callVoid("refresh_now", { scope, deviceId });

/* ------------------------------------------------------------------- series */

export const getMetricSeries = (params: {
  deviceId?: string;
  volumeId?: string;
  metricKey: string;
  fromUtc: string;
  toUtc: string;
}) => call("get_metric_series", S.metricSeries, params);

/* ------------------------------------------------------------------ alertas */

export const getAlertGroups = (filter?: { status?: AlertStatusFilter; deviceId?: string }) =>
  call("get_alert_groups", S.alertGroup.array(), filter ?? {});
export const getAlertDetail = (alertGroupId: string) =>
  call("get_alert_detail", S.alertDetail, { alertGroupId });
export const acknowledgeAlert = (alertGroupId: string) => callVoid("acknowledge_alert", { alertGroupId });
/** `minutes` null = silencio indefinido hasta reactivación manual. Nunca afecta al color. */
export const muteAlert = (alertGroupId: string, minutes: 15 | 60 | 480 | null) =>
  callVoid("mute_alert", { alertGroupId, minutes });
export const unmuteAlert = (alertGroupId: string) => callVoid("unmute_alert", { alertGroupId });
export const archiveAlert = (alertGroupId: string) => callVoid("archive_alert", { alertGroupId });

/* ------------------------------------------------------------------ eventos */

export const getSystemEvents = (params: {
  deviceId?: string;
  volumeId?: string;
  levels?: ("error" | "warning" | "info")[];
  providers?: string[];
  fromUtc?: string;
  toUtc?: string;
  cursor?: string;
  limit?: number;
}) => call("get_system_events", S.systemEventPage, params);

export const getEventRawXml = (eventId: string) => call("get_event_raw_xml", z.string(), { eventId });

/* ------------------------------------------------------------------ pruebas */

export const startBenchmark = (params: {
  volumeId: string;
  sizeBytes: number;
  blockSizeBytes: number;
  mode: "sequential" | "random";
  passes: number;
}) => call("start_benchmark", z.string(), params);

export const runChkdskScan = (volumeId: string) => call("run_chkdsk_scan", z.string(), { volumeId });
export const runSmartShortTest = (deviceId: string) => call("run_smart_short_test", z.string(), { deviceId });
export const cancelTest = (testRunId: string) => callVoid("cancel_test", { testRunId });
export const getTestRuns = (deviceId?: string, limit?: number) =>
  call("get_test_runs", S.testRun.array(), { deviceId, limit });

/* ----------------------------------------------------------------- informes */

export const exportReport = (params: {
  format: "csv" | "json" | "html";
  fromUtc: string;
  toUtc: string;
  deviceIds: string[] | null;
  includeSerials: boolean;
  destinationPath: string;
}) => call("export_report", z.string(), params);

/** No escribe nada: US-051 exige enseñar el contenido antes de guardar. */
export const previewDiagnosticZip = (includeIdentifiers: boolean) =>
  call("preview_diagnostic_zip", S.diagnosticPreview, { includeIdentifiers });

export const createDiagnosticZip = (includeIdentifiers: boolean, destinationPath: string) =>
  call("create_diagnostic_zip", z.string(), { includeIdentifiers, destinationPath });

/** Diálogo nativo de guardado (ADR-031): la interfaz nunca construye una ruta, la pide siempre al
 *  sistema. `null` significa que el usuario canceló el diálogo, no un error. */
export const chooseSavePath = async (params: {
  defaultFileName: string;
  filterName: string;
  extensions: string[];
}): Promise<string | null> => {
  try {
    return await save({
      defaultPath: params.defaultFileName,
      filters: [{ name: params.filterName, extensions: params.extensions }]
    });
  } catch (cause) {
    throw toAppError(cause);
  }
};

/* ------------------------------------------------------------ ciclo de vida */

export const pauseMonitoring = () => callVoid("pause_monitoring");
export const resumeMonitoring = () => callVoid("resume_monitoring");
export const getAppInfo = () => call("get_app_info", S.appInfo);

/** Irreversible: exige que el usuario escriba la frase de confirmación (US-073). */
export const deleteAllData = (confirmationPhrase: string) =>
  callVoid("delete_all_data", { confirmationPhrase });

/** J.56/ADR-043: si `smartctl.exe` ya está permitido en Control de acceso a carpetas de Defender. */
export const checkSmartctlDefenderException = () =>
  call("check_smartctl_defender_exception", S.smartctlDefenderAllowed);

/** Reintenta añadir la excepción (el instalador ya lo intenta al instalar); puede fallar si la
 *  Protección contra alteraciones de Defender lo bloquea incluso con privilegios de administrador. */
export const addSmartctlDefenderException = () =>
  call("add_smartctl_defender_exception", S.defenderExceptionResult);

/* ------------------------------------------------------------------ registro */

/** Mismos seis niveles que `$lib/logger`'s `LEVELS`, escritos aquí a mano porque ese módulo no
 *  puede depender de Zod (constitución §XV: cuarenta líneas sin dependencias externas). */
export const getLogLevel = () =>
  call("get_log_level", z.enum(["trace", "debug", "info", "warn", "error", "silent"]));

/** Persiste `logging.verbose` **y** lo aplica en caliente en el mismo comando (T098): no hace
 *  falta reiniciar para que el modo detallado surta efecto. */
export const setLogLevel = (verbose: boolean) => callVoid("set_log_level", { verbose });

/** Abre el explorador de archivos en la carpeta de registro. Sin parámetro de ruta: es **una
 *  sola ruta conocida** que decide el backend, nunca una que construya la interfaz (principio IX). */
export const openLogFolder = () => callVoid("open_log_folder");

export type { DiskSummary };
