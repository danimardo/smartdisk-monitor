import { describe, expect, it } from "vitest";
import * as S from "./schemas";

/** La constitución §XI exige que **todo esquema tenga su prueba de rechazo**. Un esquema que solo
 *  se prueba con datos buenos no demuestra nada: lo que hay que verificar es que rechaza lo que
 *  debe rechazar, porque de eso depende que un cambio de contrato en el backend se detecte en vez
 *  de convertirse en `undefined` mostrado como dato válido. */

const actividadValida = {
  estado: "valido",
  mediaPercent: 12,
  picoPercent: 38,
  muestras: 30,
  ventanaSegundos: 30
};

const discoValido = {
  id: "dev-1",
  model: "Samsung 990 PRO 2TB",
  deviceType: "NVMe",
  state: "ok",
  temperatureC: 47,
  percentageUsed: 3,
  activity: actividadValida,
  powerOnHours: 1200,
  volumes: []
};

describe("diskSummary", () => {
  it("acepta un disco completo", () => {
    expect(S.diskSummary.safeParse(discoValido).success).toBe(true);
  });

  it("acepta null en toda métrica opcional: un dato ausente es null explícito", () => {
    const sinDatos = {
      ...discoValido,
      temperatureC: null,
      percentageUsed: null,
      activity: {
        estado: "no_disponible",
        mediaPercent: null,
        picoPercent: null,
        muestras: 0,
        ventanaSegundos: 30
      }
    };
    expect(S.diskSummary.safeParse(sinDatos).success).toBe(true);
  });

  it("RECHAZA un estado de salud que no existe", () => {
    const r = S.diskSummary.safeParse({ ...discoValido, state: "degradado" });
    expect(r.success).toBe(false);
  });

  it("RECHAZA una temperatura como cadena, que es lo que produce un cambio de serialización", () => {
    const r = S.diskSummary.safeParse({ ...discoValido, temperatureC: "47" });
    expect(r.success).toBe(false);
  });

  it("RECHAZA que falte un campo obligatorio, y dice cuál", () => {
    // Se descarta `model` a propósito para construir un objeto al que le falta un obligatorio.
    const { model: _descartado, ...sinModelo } = discoValido;
    const r = S.diskSummary.safeParse(sinModelo);
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].path).toContain("model");
  });

  it("RECHAZA que `volumes` no sea un array", () => {
    expect(S.diskSummary.safeParse({ ...discoValido, volumes: null }).success).toBe(false);
  });

  it("acepta `smartHealthPassed` booleano o null, RECHAZA otra cosa (ADR-041)", () => {
    expect(S.diskSummary.safeParse({ ...discoValido, smartHealthPassed: true }).success).toBe(true);
    expect(S.diskSummary.safeParse({ ...discoValido, smartHealthPassed: null }).success).toBe(true);
    expect(S.diskSummary.safeParse({ ...discoValido, smartHealthPassed: 1 }).success).toBe(false);
  });

  it("RECHAZA `activity` como número suelto (el contrato viejo, spec 007 FR-012a)", () => {
    const r = S.diskSummary.safeParse({ ...discoValido, activity: 12 });
    expect(r.success).toBe(false);
  });

  it("RECHAZA un `estado` de actividad fuera del enum", () => {
    const r = S.diskSummary.safeParse({
      ...discoValido,
      activity: { ...actividadValida, estado: "desconocido" }
    });
    expect(r.success).toBe(false);
  });

  it("RECHAZA una actividad sin `ventanaSegundos`, y dice cuál falta", () => {
    const { ventanaSegundos: _fuera, ...sinVentana } = actividadValida;
    const r = S.diskSummary.safeParse({ ...discoValido, activity: sinVentana });
    expect(r.success).toBe(false);
    if (!r.success) expect(r.error.issues[0].path).toContain("ventanaSegundos");
  });
});

describe("actividadDisco", () => {
  it("acepta media y pico null solo con estado no_disponible", () => {
    expect(
      S.actividadDisco.safeParse({
        estado: "no_disponible",
        mediaPercent: null,
        picoPercent: null,
        muestras: 0,
        ventanaSegundos: 30
      }).success
    ).toBe(true);
  });

  it("RECHAZA `muestras` negativas y `ventanaSegundos` de cero", () => {
    expect(S.actividadDisco.safeParse({ ...actividadValida, muestras: -1 }).success).toBe(false);
    expect(S.actividadDisco.safeParse({ ...actividadValida, ventanaSegundos: 0 }).success).toBe(false);
  });
});

describe("metricSeries", () => {
  const serie = {
    metricKey: "temperature_celsius",
    unit: "°C",
    resolution: "raw",
    downsampled: false,
    fromUtc: "2026-09-04T10:00:00.000Z",
    toUtc: "2026-09-04T12:00:00.000Z",
    expectedIntervalMs: 30000,
    points: [
      { t: 1, v: 40 },
      { t: 2, v: null }
    ],
    vendorLimit: 70,
    vendorCritical: 80
  };

  it("acepta huecos explícitos: `v: null` es un hueco, no un error", () => {
    expect(S.metricSeries.safeParse(serie).success).toBe(true);
  });

  it("RECHAZA una cadencia de cero o negativa", () => {
    expect(S.metricSeries.safeParse({ ...serie, expectedIntervalMs: 0 }).success).toBe(false);
  });

  it("RECHAZA una resolución desconocida", () => {
    expect(S.metricSeries.safeParse({ ...serie, resolution: "diaria" }).success).toBe(false);
  });
});

describe("windowsAccent", () => {
  it("acepta un hex de seis dígitos", () => {
    expect(S.windowsAccent.safeParse({ hex: "#0078d4" }).success).toBe(true);
  });

  it("RECHAZA un hex de tres dígitos", () => {
    // Importa: el backend convierte desde ABGR y debe entregar siempre seis dígitos.
    expect(S.windowsAccent.safeParse({ hex: "#08d" }).success).toBe(false);
  });

  it("RECHAZA un color sin almohadilla", () => {
    expect(S.windowsAccent.safeParse({ hex: "0078d4" }).success).toBe(false);
  });

  it("la paleta es opcional: si Windows no la expone, se deriva", () => {
    expect(S.windowsAccent.safeParse({ hex: "#0078d4" }).success).toBe(true);
  });
});

describe("appearanceSettings", () => {
  const ajustes = { theme: "system", language: null, systemLocale: "es-ES", useSystemAccent: true };

  it("acepta idioma null, que significa seguir al sistema", () => {
    expect(S.appearanceSettings.safeParse(ajustes).success).toBe(true);
  });

  it("RECHAZA un idioma no soportado", () => {
    expect(S.appearanceSettings.safeParse({ ...ajustes, language: "fr" }).success).toBe(false);
  });

  it("RECHAZA un locale vacío: sin él, el formato de números no tiene base", () => {
    expect(S.appearanceSettings.safeParse({ ...ajustes, systemLocale: "" }).success).toBe(false);
  });
});

describe("defenderExceptionResult", () => {
  it("acepta un fallo con su detalle", () => {
    expect(S.defenderExceptionResult.safeParse({ added: false, detail: "bloqueado" }).success).toBe(true);
  });

  it("acepta un éxito sin detalle (null)", () => {
    expect(S.defenderExceptionResult.safeParse({ added: true, detail: null }).success).toBe(true);
  });

  it("RECHAZA que falte el detalle: `null` es explícito, no opcional", () => {
    expect(S.defenderExceptionResult.safeParse({ added: true }).success).toBe(false);
  });
});

describe("alertGroup", () => {
  const grupo = {
    id: "g1",
    ruleKey: "temp.above_vendor_limit",
    deduplicationKey: "k1",
    severity: "warn",
    status: "active",
    title: "Temperatura alta",
    summary: "Supera el límite del fabricante",
    count: 3,
    firstOccurredAt: "2026-09-04T10:00:00.000Z",
    lastOccurredAt: "2026-09-04T11:00:00.000Z",
    target: "dev-1"
  };

  it("acepta un grupo sin silencio ni ciclo", () => {
    expect(S.alertGroup.safeParse(grupo).success).toBe(true);
  });

  it("acepta el silencio indefinido como literal 'infinite'", () => {
    expect(S.alertGroup.safeParse({ ...grupo, mutedUntil: "infinite" }).success).toBe(true);
  });

  it("RECHAZA un contador negativo", () => {
    expect(S.alertGroup.safeParse({ ...grupo, count: -1 }).success).toBe(false);
  });

  it("RECHAZA un estado que no está en el ciclo de vida", () => {
    expect(S.alertGroup.safeParse({ ...grupo, status: "silenciada" }).success).toBe(false);
  });
});

describe("eventSchemas", () => {
  it("cubre exactamente los nueve eventos del contrato", () => {
    expect(Object.keys(S.eventSchemas).sort()).toEqual(
      [
        "alerts:changed",
        "inventory:changed",
        "metrics:updated",
        "monitoring:paused",
        "monitoring:resumed",
        "source:degraded",
        "system:accent-changed",
        "system:theme-changed",
        "test:progress"
      ].sort()
    );
  });

  it("metrics:updated RECHAZA una carga sin fecha de emisión", () => {
    const r = S.eventSchemas["metrics:updated"].safeParse({ devices: [], sources: [] });
    expect(r.success).toBe(false);
  });

  it("metrics:updated RECHAZA un disco malformado dentro del lote", () => {
    const r = S.eventSchemas["metrics:updated"].safeParse({
      emittedAt: "2026-09-04T12:00:00.000Z",
      devices: [{ id: "x" }],
      sources: []
    });
    expect(r.success).toBe(false);
  });
});

describe("settings — perfiles de alerta v3 (ADR-036)", () => {
  const base = {
    schedule: { metricsFastSeconds: 30, smartFullSeconds: 300, eventsSeconds: 30, discoverySeconds: 60 },
    alerts: {
      profile: "balanced",
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
    onboarding: { completedAt: null },
    retention: {
      rawDays: 7,
      fiveMinutesDays: 90,
      hourlyDays: 730,
      freeSpaceWarnBytes: 1_073_741_824,
      freeSpaceHaltBytes: 268_435_456
    },
    lifecycle: { closeAction: "minimize", closeActionRemembered: false, startWithSystem: false },
    notifications: { soundEnabled: false, enabled: true },
    logging: { verbose: false },
    ai: { enabled: false, model: "openrouter/free", previewAcknowledged: false, sendWithoutReview: false }
  };

  it("acepta el settings de fábrica v3", () => {
    expect(S.settings.safeParse(base).success).toBe(true);
  });

  it("RECHAZA un settings sin el grupo ai (spec 005)", () => {
    const sinIa = { ...base } as Record<string, unknown>;
    delete sinIa.ai;
    expect(S.settings.safeParse(sinIa).success).toBe(false);
  });

  it("RECHAZA un grupo ai con enabled que no es booleano", () => {
    expect(S.settings.safeParse({ ...base, ai: { ...base.ai, enabled: "sí" } }).success).toBe(false);
  });

  it("RECHAZA un perfil de alerta que no está en el enum", () => {
    expect(S.settings.safeParse({ ...base, alerts: { ...base.alerts, profile: "agresivo" } }).success).toBe(
      false
    );
  });

  it("RECHAZA un settings al que le falta un umbral nuevo (cambio de contrato del backend)", () => {
    const alerts = { ...base.alerts };
    delete (alerts as Record<string, unknown>).wearWarnPercent;
    expect(S.settings.safeParse({ ...base, alerts }).success).toBe(false);
  });

  it("RECHAZA una marca de asistente que no es fecha ni null", () => {
    expect(S.settings.safeParse({ ...base, onboarding: { completedAt: "ayer" } }).success).toBe(false);
  });

  it("RECHAZA un settings sin los ajustes nuevos de notificación y autoarranque (v3 ADR-037/038)", () => {
    const sinToast = { ...base, notifications: { soundEnabled: false } };
    expect(S.settings.safeParse(sinToast).success).toBe(false);
    const sinAutoarranque = {
      ...base,
      lifecycle: { closeAction: "minimize", closeActionRemembered: false }
    };
    expect(S.settings.safeParse(sinAutoarranque).success).toBe(false);
  });
});

describe("ayuda con IA (spec 005-explicacion-ia)", () => {
  it("estadoIa acepta una respuesta completa y con claveValida null", () => {
    expect(
      S.estadoIa.safeParse({
        activa: true,
        modelo: "openrouter/free",
        previewAcknowledged: false,
        sendWithoutReview: false,
        claveValida: null,
        claveCompartidaDisponible: false,
        usandoClaveCompartida: false
      }).success
    ).toBe(true);
  });

  it("estadoIa RECHAZA claveValida como cadena", () => {
    expect(
      S.estadoIa.safeParse({
        activa: true,
        modelo: "x",
        previewAcknowledged: false,
        claveValida: "sí",
        claveCompartidaDisponible: false,
        usandoClaveCompartida: false
      }).success
    ).toBe(false);
  });

  it("estadoIa RECHAZA usandoClaveCompartida como cadena (ADR-054)", () => {
    expect(
      S.estadoIa.safeParse({
        activa: true,
        modelo: "openrouter/free",
        previewAcknowledged: false,
        sendWithoutReview: false,
        claveValida: true,
        claveCompartidaDisponible: true,
        usandoClaveCompartida: "sí"
      }).success
    ).toBe(false);
  });

  it("estadoIa RECHAZA claveCompartidaDisponible ausente (cambio de contrato del backend)", () => {
    expect(
      S.estadoIa.safeParse({
        activa: false,
        modelo: "openrouter/free",
        previewAcknowledged: false,
        sendWithoutReview: false,
        claveValida: null,
        usandoClaveCompartida: false
      }).success
    ).toBe(false);
  });

  it("modeloIa RECHAZA esDePago ausente (cambio de contrato del backend)", () => {
    expect(S.modeloIa.safeParse({ id: "a/b", nombre: "A B" }).success).toBe(false);
  });

  it("explicacionIa RECHAZA markdown como número", () => {
    expect(
      S.explicacionIa.safeParse({
        markdown: 42,
        modeloUsado: "x",
        detalleRecortado: false,
        sinVolcado: false,
        sinSuceso: false
      }).success
    ).toBe(false);
  });

  it("explicacionIa RECHAZA sinVolcado/sinSuceso ausentes (cambio de contrato del backend)", () => {
    expect(
      S.explicacionIa.safeParse({ markdown: "x", modeloUsado: "y", detalleRecortado: false }).success
    ).toBe(false);
  });

  it("resultadoExplicacion acepta la variante ok y la variante revision", () => {
    expect(
      S.resultadoExplicacion.safeParse({
        estado: "ok",
        markdown: "## Hola",
        modeloUsado: "vendor/model:free",
        detalleRecortado: false,
        sinVolcado: false,
        sinSuceso: false
      }).success
    ).toBe(true);
    expect(
      S.resultadoExplicacion.safeParse({
        estado: "revision",
        textoCompleto: "…",
        fragmentos: [{ texto: "D:\\datos", motivoKey: "ia.review.path" }]
      }).success
    ).toBe(true);
  });

  it("resultadoExplicacion RECHAZA un estado desconocido", () => {
    expect(S.resultadoExplicacion.safeParse({ estado: "otro", markdown: "x" }).success).toBe(false);
  });

  it("origenExplicacion acepta el tipo `evento` con `eventId`", () => {
    expect(
      S.origenExplicacion.safeParse({
        tipo: "evento",
        deviceId: null,
        alertGroupId: null,
        eventId: "42",
        idioma: "es",
        revision: "ninguna",
        previewConfirmada: false
      }).success
    ).toBe(true);
  });

  it("origenExplicacion RECHAZA `eventId` ausente (cambio de contrato del backend)", () => {
    expect(
      S.origenExplicacion.safeParse({
        tipo: "evento",
        deviceId: null,
        alertGroupId: null,
        idioma: "es",
        revision: "ninguna",
        previewConfirmada: false
      }).success
    ).toBe(false);
  });

  it("origenExplicacion RECHAZA un tipo que no existe", () => {
    expect(
      S.origenExplicacion.safeParse({
        tipo: "informe",
        deviceId: null,
        alertGroupId: null,
        eventId: null,
        idioma: "es",
        revision: "ninguna",
        previewConfirmada: false
      }).success
    ).toBe(false);
  });

  it("revisionAnonimizacion RECHAZA un fragmento sin motivoKey", () => {
    expect(
      S.revisionAnonimizacion.safeParse({
        textoCompleto: "x",
        fragmentos: [{ texto: "algo" }]
      }).success
    ).toBe(false);
  });
});

describe("benchmark de Rendimiento (spec 008 / ADR-053)", () => {
  const filaValida = {
    profile: "seq1m_q8",
    direction: "read",
    mbPerSecond: 2680,
    iops: 2556,
    avgLatencyMs: 2.97,
    actualDurationS: 1.0,
    bytesMoved: 2680160256,
    dataCapHit: false
  };

  it("testResult acepta un benchmark completo y también chkdsk sin el campo `benchmark`", () => {
    expect(
      S.testResult.safeParse({
        passed: null,
        maxTemperatureC: 44,
        stoppedReason: "completed",
        benchmark: {
          tool: "diskspd",
          toolVersion: "2.3.0",
          fileSizeBytes: 1073741824,
          rows: [filaValida],
          notRun: []
        }
      }).success
    ).toBe(true);
    // chkdsk / autotest: el backend omite `benchmark` (skip_serializing_if) → default a null.
    const chkdsk = S.testResult.safeParse({
      passed: true,
      maxTemperatureC: null,
      stoppedReason: "completed"
    });
    expect(chkdsk.success).toBe(true);
    expect(chkdsk.success && chkdsk.data.benchmark).toBe(null);
  });

  it("RECHAZA una fila sin `mbPerSecond`", () => {
    const { mbPerSecond: _omitido, ...incompleta } = filaValida;
    expect(S.benchmarkRow.safeParse(incompleta).success).toBe(false);
  });

  it("RECHAZA un `profile` fuera del enum y un `direction` inválido", () => {
    expect(S.benchmarkRow.safeParse({ ...filaValida, profile: "seq2m_q8" }).success).toBe(false);
    expect(S.benchmarkRow.safeParse({ ...filaValida, direction: "readwrite" }).success).toBe(false);
  });

  it('RECHAZA `stoppedReason: "space"` (ya no aparece en ejecución, ADR-053)', () => {
    expect(
      S.testResult.safeParse({ passed: null, maxTemperatureC: null, stoppedReason: "space" }).success
    ).toBe(false);
  });
});
