import { beforeEach, describe, expect, it } from "vitest";
import { app, VENTANA_ACTIVIDAD_TARJETA_MS } from "./app.svelte";
import type { AlertGroup, DiskSummary } from "$lib/design/types";

/** El estado se alimenta de eventos que traen el objeto completo, no un parche (ADR-015).
 *  Estas pruebas fijan que reemplazar por identificador funciona y no duplica. */

const disk = (id: string, over: Partial<DiskSummary> = {}): DiskSummary => ({
  id,
  model: `Modelo ${id}`,
  deviceType: "NVMe",
  state: "ok",
  temperatureC: 40,
  percentageUsed: 5,
  activity: { estado: "valido", mediaPercent: 0, picoPercent: 0, muestras: 30, ventanaSegundos: 30 },
  powerOnHours: 100,
  volumes: [],
  ...over
});

const alert = (id: string, over: Partial<AlertGroup> = {}): AlertGroup => ({
  id,
  ruleKey: "temp.above_vendor_limit",
  deduplicationKey: `k-${id}`,
  severity: "warn",
  status: "active",
  count: 1,
  firstOccurredAt: "2026-09-04T10:00:00Z",
  lastOccurredAt: "2026-09-04T10:00:00Z",
  target: "dev-1",
  ...over
});

describe("upsertDevices", () => {
  beforeEach(() => {
    app.devices = [];
  });

  it("añade discos nuevos", () => {
    app.upsertDevices([disk("a"), disk("b")]);
    expect(app.devices.map((d) => d.id)).toEqual(["a", "b"]);
  });

  it("reemplaza por identificador en vez de duplicar", () => {
    app.upsertDevices([disk("a", { temperatureC: 40 })]);
    app.upsertDevices([disk("a", { temperatureC: 52 })]);
    expect(app.devices).toHaveLength(1);
    expect(app.devices[0].temperatureC).toBe(52);
  });

  it("conserva los discos que el lote no menciona", () => {
    app.upsertDevices([disk("a"), disk("b")]);
    app.upsertDevices([disk("b", { state: "crit" })]);
    expect(app.devices).toHaveLength(2);
    expect(app.devices.find((d) => d.id === "b")?.state).toBe("crit");
    expect(app.devices.find((d) => d.id === "a")?.state).toBe("ok");
  });
});

describe("upsertAlerts", () => {
  beforeEach(() => {
    app.alerts = [];
  });

  it("añade y actualiza grupos", () => {
    app.upsertAlerts([alert("g1")]);
    app.upsertAlerts([alert("g1", { count: 7, severity: "crit" })]);
    expect(app.alerts).toHaveLength(1);
    expect(app.alerts[0].count).toBe(7);
    expect(app.alerts[0].severity).toBe("crit");
  });

  it("retira los grupos que el backend marca como eliminados", () => {
    app.upsertAlerts([alert("g1"), alert("g2")]);
    app.upsertAlerts([], ["g1"]);
    expect(app.alerts.map((a) => a.id)).toEqual(["g2"]);
  });

  it("puede añadir y retirar en la misma tanda", () => {
    app.upsertAlerts([alert("g1")]);
    app.upsertAlerts([alert("g2")], ["g1"]);
    expect(app.alerts.map((a) => a.id)).toEqual(["g2"]);
  });
});

describe("onda de actividad de la tarjeta (ADR-051)", () => {
  const T0 = Date.parse("2026-09-09T10:00:00Z");
  const iso = (ms: number) => new Date(ms).toISOString();
  const conActividad = (
    id: string,
    estado: DiskSummary["activity"]["estado"],
    media: number | null
  ): DiskSummary =>
    disk(id, {
      activity: { estado, mediaPercent: media, picoPercent: media, muestras: 30, ventanaSegundos: 30 }
    });

  beforeEach(() => {
    app.devices = [];
    app.activitySeries = {};
  });

  it("añade un punto por disco en cada evento", () => {
    app.pushActivitySamples([conActividad("a", "valido", 20)], iso(T0));
    app.pushActivitySamples(
      [conActividad("a", "valido", 30), conActividad("b", "valido", 5)],
      iso(T0 + 30_000)
    );
    expect(app.activitySeries["a"]).toEqual([
      { t: T0, v: 20 },
      { t: T0 + 30_000, v: 30 }
    ]);
    expect(app.activitySeries["b"]).toEqual([{ t: T0 + 30_000, v: 5 }]);
  });

  it("pinta la media parcial, pero deja hueco (v null) cuando no hay dato", () => {
    app.pushActivitySamples([conActividad("a", "parcial", 40)], iso(T0));
    app.pushActivitySamples([conActividad("a", "no_disponible", null)], iso(T0 + 30_000));
    expect(app.activitySeries["a"].map((p) => p.v)).toEqual([40, null]);
  });

  it("ignora un evento a menos de 15 s del último punto (reemisión del ciclo SMART)", () => {
    app.pushActivitySamples([conActividad("a", "valido", 20)], iso(T0));
    app.pushActivitySamples([conActividad("a", "valido", 99)], iso(T0 + 5_000));
    expect(app.activitySeries["a"]).toEqual([{ t: T0, v: 20 }]);
  });

  it("recorta los puntos que salen de la ventana relativa al más reciente", () => {
    app.pushActivitySamples([conActividad("a", "valido", 10)], iso(T0));
    app.pushActivitySamples(
      [conActividad("a", "valido", 50)],
      iso(T0 + VENTANA_ACTIVIDAD_TARJETA_MS + 60_000)
    );
    expect(app.activitySeries["a"]).toEqual([{ t: T0 + VENTANA_ACTIVIDAD_TARJETA_MS + 60_000, v: 50 }]);
  });

  it("la siembra se fusiona con lo que ya llegó en vivo, sin pisarlo", () => {
    app.pushActivitySamples([conActividad("a", "valido", 30)], iso(T0 + 60_000));
    app.seedActivitySeries("a", [
      { t: T0, v: 10 },
      { t: T0 + 30_000, v: 20 }
    ]);
    expect(app.activitySeries["a"]).toEqual([
      { t: T0, v: 10 },
      { t: T0 + 30_000, v: 20 },
      { t: T0 + 60_000, v: 30 }
    ]);
  });

  it("prunearSeriesActividad descarta los discos que ya no están presentes", () => {
    app.pushActivitySamples([conActividad("a", "valido", 1), conActividad("b", "valido", 2)], iso(T0));
    app.devices = [disk("a")];
    app.prunearSeriesActividad();
    expect(Object.keys(app.activitySeries)).toEqual(["a"]);
  });
});

describe("estado inicial", () => {
  it("loadedAt empieza en null: no es lo mismo que no tener discos", () => {
    // Distinguir "todavía no se ha cargado nada" de "se cargó y no hay discos" es lo que permite
    // que la interfaz no muestre un estado vacío engañoso durante el arranque.
    const fresh = Object.getPrototypeOf(app).constructor;
    const instance = new fresh();
    expect(instance.loadedAt).toBeNull();
    expect(instance.devices).toEqual([]);
  });
});
