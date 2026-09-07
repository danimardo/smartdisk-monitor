import * as S from "../../../src/lib/api/schemas";

/** Respuestas del backend para el plano de interfaz.
 *
 *  Se validan contra los **mismos esquemas Zod** que usa la aplicación en tiempo de ejecución
 *  (`validar()`, más abajo). Sin eso, un fixture podría quedarse desfasado del contrato y las
 *  pruebas seguirían en verde probando una forma de datos que ya no existe — que es exactamente
 *  la clase de fallo silencioso que el principio XI trata de impedir.
 */

const AHORA = "2026-09-04T10:00:00Z";

export const apariencia = {
  theme: "light",
  language: "es",
  systemLocale: "es-ES",
  // De fábrica apagado (v3, ADR-035): una instalación nueva estrena la paleta Ciruela.
  useSystemAccent: false
};

export const acento = { hex: "#0067c0", palette: ["#0067c0"] };

/** Un NVMe sano y un USB sin SMART. El segundo importa: comprueba que «no compatible» se pinta en
 *  gris y no en rojo, que es la confusión que la especificación §12 prohíbe expresamente. */
export const inventario = {
  devices: [
    {
      id: "disk-0",
      alias: null,
      model: "Samsung SSD 990 PRO 2TB",
      deviceType: "nvme",
      state: "ok",
      temperatureC: 41,
      percentageUsed: 3,
      activityPercent: 12,
      powerOnHours: 1840,
      lastReadAt: AHORA,
      volumes: [
        {
          id: "vol-c",
          label: "Sistema",
          driveLetters: ["C:"],
          capacityBytes: 2_000_398_934_016,
          freeBytes: 1_204_000_000_000,
          mappingConfidence: "exact",
          chkdskAvailable: true,
          isSystemVolume: false
        }
      ]
    },
    {
      id: "disk-1",
      alias: "Copia externa",
      model: "WD Elements 25A3",
      deviceType: "usb",
      state: "unknown",
      temperatureC: null,
      percentageUsed: null,
      activityPercent: null,
      powerOnHours: null,
      unknownReason: "unsupported",
      lastReadAt: AHORA,
      volumes: []
    }
  ],
  excluded: [],
  sources: [
    { source: "smartctl", status: "ok", lastSuccessAt: AHORA, lastAttemptAt: AHORA },
    { source: "windows-storage", status: "ok", lastSuccessAt: AHORA, lastAttemptAt: AHORA }
  ],
  paused: false,
  pausedSince: null
};

/** Un grupo de alerta activo, con desgaste alto: ninguna acción de ciclo de vida lo cambia solo
 *  (`smart.wear_high` no se resuelve sola, se archiva a mano). Sirve para probar la lista, el
 *  detalle y el filtro sin depender de la histéresis del motor. */
export const alertaActiva = {
  id: "alert-wear",
  ruleKey: "smart.wear_high",
  deduplicationKey: "smart.wear_high|device:disk-0",
  severity: "warn",
  status: "active",
  count: 3,
  firstOccurredAt: AHORA,
  lastOccurredAt: AHORA,
  target: "Samsung SSD 990 PRO 2TB",
  mutedUntil: null,
  cycle: 1
};

export const alertas = [alertaActiva];

export const detalleAlertaActiva = {
  ...alertaActiva,
  facts: [
    { labelKey: "alert.fact.ruleKey", value: "smart.wear_high" },
    { labelKey: "alert.fact.lastValue", value: "92" }
  ],
  occurrences: [{ occurredAt: AHORA, cycle: 1, value: 92, eventId: null, context: null }],
  relatedEvents: []
};

/** Detalle del primer disco del inventario, con un par de contadores reales — cubre las dos
 *  unidades más comunes (temperatura y porcentaje) sin listar los treinta y tantos posibles. */
export const detalleDisco0 = {
  ...inventario.devices[0],
  fingerprint: "huella-disk-0",
  identityConfidence: "fingerprint",
  serialNumber: "S6B2NS0T123456",
  firmware: "GXA7801Q",
  busType: "NVMe",
  capabilities: [
    { key: "smart", available: true, reasonKey: null },
    { key: "self_test_short", available: true, reasonKey: null },
    { key: "chkdsk_scan", available: true, reasonKey: null }
  ],
  counters: [
    {
      metricKey: "temperature_celsius",
      value: 41,
      unit: "celsius",
      delta: null,
      deltaIsMeaningful: false,
      provenance: { source: "smartctl", quality: "exact", readAt: AHORA }
    },
    {
      metricKey: "power_cycles",
      value: 812,
      unit: "count",
      delta: null,
      deltaIsMeaningful: false,
      provenance: { source: "smartctl", quality: "exact", readAt: AHORA }
    }
  ],
  firstSeenAt: AHORA,
  lastSeenAt: AHORA,
  removedAt: null
};

/** Serie de temperatura con un hueco explícito, para probar que la gráfica lo distingue de un
 *  valor real (`docs/open-questions.md` E.1). */
export const serieTemperatura = {
  metricKey: "temperature_celsius",
  unit: "celsius",
  resolution: "raw",
  downsampled: false,
  fromUtc: "2026-09-04T08:00:00Z",
  toUtc: AHORA,
  expectedIntervalMs: 30_000,
  points: [
    { t: new Date("2026-09-04T08:00:00Z").getTime(), v: 40 },
    { t: new Date("2026-09-04T09:00:00Z").getTime(), v: null },
    { t: new Date(AHORA).getTime(), v: 41 }
  ],
  vendorLimit: null,
  vendorCritical: null
};

/** Un evento con asociación exacta y otro con inferida, para probar que `EventRow` etiqueta la
 *  inferencia y nunca la presenta como certeza (`docs/alert-rules.md` §3.6). */
export const eventos = [
  {
    id: "1",
    occurredAt: AHORA,
    provider: "Microsoft-Windows-Ntfs",
    eventId: 98,
    level: "info",
    message: "Volumen C: es correcto. No se requiere ninguna acción.",
    deviceId: "disk-0",
    volumeId: null,
    mappingConfidence: "exact",
    hasRawXml: true
  },
  {
    id: "2",
    occurredAt: AHORA,
    provider: "disk",
    eventId: 157,
    level: "error",
    message: "El disco 1 se ha extraído de forma imprevista del sistema.",
    deviceId: "disk-0",
    volumeId: null,
    mappingConfidence: "inferred",
    hasRawXml: true
  }
];

export const paginaEventos = { events: eventos, nextCursor: null, total: 2 };
export const xmlEjemplo = "<Event><System><Provider Name='disk'/></System></Event>";

/** Historial vacío: la pantalla de pruebas debe aguantarlo sin lista ni tarjeta activa. */
export const testRunsVacio: unknown[] = [];

/** Un benchmark en curso, para probar la tarjeta de progreso y el evento `test:progress` sin
 *  esperar a que una prueba real termine. */
export const testRunActivo = {
  id: "run-benchmark-1",
  type: "benchmark",
  deviceId: null,
  volumeId: "vol-c",
  status: "running",
  startedAt: AHORA,
  finishedAt: null,
  progressPercent: 40,
  command: null,
  parameters: { sizeBytes: 1_073_741_824, blockSizeBytes: 1_048_576, mode: "sequential", passes: 1 },
  result: null,
  output: null,
  outputEncoding: null,
  orphanPath: null
};

/** Vista previa del ZIP de diagnóstico, anonimizada por defecto. */
export const vistaPreviaDiagnostico = {
  entries: [
    { path: "manifest.json", sizeBytes: 128, descriptionKey: "diagnostic.entry.manifest" },
    { path: "settings.json", sizeBytes: 256, descriptionKey: "diagnostic.entry.settings" },
    { path: "smart/disk-0.json", sizeBytes: 4096, descriptionKey: "diagnostic.entry.smart" }
  ],
  totalBytes: 4480,
  redactedFields: ["diagnostic.redacted.serialNumber", "diagnostic.redacted.computerName"]
};

/** Ajustes de fábrica (`docs/open-questions.md` J.32), para la pantalla `/settings`. */
export const settingsDeFabrica = {
  schedule: {
    metricsFastSeconds: 30,
    smartFullSeconds: 300,
    eventsSeconds: 30,
    discoverySeconds: 60
  },
  alerts: {
    profile: "balanced" as const,
    tempConfiguredWarnC: 60,
    tempConfiguredCritC: 70,
    wearWarnPercent: 80,
    wearCritPercent: 90,
    capacityWarnPercent: 10,
    capacityCritPercent: 5,
    capacityAbsoluteFloorMinCapacityBytes: 274_877_906_944,
    capacityAbsoluteFloorWarnBytes: 21_474_836_480,
    capacityAbsoluteFloorCritBytes: 10_737_418_240,
    mediaErrorsWarnPer24h: 1,
    mediaErrorsCritPer24h: 5,
    driverRetryWarnPer24h: 5,
    driverRetryCritPer24h: 12
  },
  onboarding: {
    completedAt: "2026-01-01T00:00:00Z"
  },
  retention: {
    rawDays: 7,
    fiveMinutesDays: 90,
    hourlyDays: 730,
    freeSpaceWarnBytes: 1_073_741_824,
    freeSpaceHaltBytes: 268_435_456
  },
  lifecycle: {
    closeAction: "minimize" as const,
    closeActionRemembered: false,
    startWithSystem: false
  },
  notifications: {
    soundEnabled: false,
    enabled: true
  },
  logging: {
    verbose: false
  }
};

export const appInfoDePrueba = {
  name: "SmartDisk Monitor",
  version: "0.1.1",
  author: "Daniel Diez Mardomingo"
};

/** Comando → respuesta. Lo que no esté aquí devuelve `null`, y la pantalla debe aguantarlo. */
export const RESPUESTAS: Record<string, unknown> = {
  get_appearance_settings: apariencia,
  get_system_accent_color: acento,
  get_devices: inventario,
  get_device_detail: detalleDisco0,
  get_metric_series: serieTemperatura,
  get_alert_groups: alertas,
  get_alert_detail: detalleAlertaActiva,
  get_system_events: paginaEventos,
  get_event_raw_xml: xmlEjemplo,
  get_log_level: "info",
  get_test_runs: testRunsVacio,
  start_benchmark: testRunActivo.id,
  run_chkdsk_scan: "run-chkdsk-1",
  run_smart_short_test: "run-autotest-1",
  cancel_test: null,
  export_report: "C:\\destino\\de\\prueba\\informe.csv",
  preview_diagnostic_zip: vistaPreviaDiagnostico,
  create_diagnostic_zip: "C:\\destino\\de\\prueba\\diagnostico.zip",
  get_settings: settingsDeFabrica,
  reset_settings: settingsDeFabrica,
  set_setting: null,
  set_log_level: null,
  open_log_folder: null,
  delete_all_data: null,
  get_app_info: appInfoDePrueba,
  // El diálogo nativo de guardado (ADR-031): el harness simula que el usuario ya eligió un
  // destino, sin abrir ningún selector real.
  "plugin:dialog|save": "C:\\destino\\de\\prueba\\elegido.tmp"
};

/** Falla en cuanto un fixture deja de cumplir el contrato, no cuando una pantalla se rompe. */
export function validar(): void {
  S.appearanceSettings.parse(apariencia);
  S.settings.parse(settingsDeFabrica);
  S.windowsAccent.parse(acento);
  S.deviceListResponse.parse(inventario);
  S.deviceDetail.parse(detalleDisco0);
  S.metricSeries.parse(serieTemperatura);
  S.alertGroup.array().parse(alertas);
  S.alertDetail.parse(detalleAlertaActiva);
  S.systemEventPage.parse(paginaEventos);
  S.testRun.array().parse(testRunsVacio);
  S.testRun.parse(testRunActivo);
  S.diagnosticPreview.parse(vistaPreviaDiagnostico);
  S.appInfo.parse(appInfoDePrueba);
}
