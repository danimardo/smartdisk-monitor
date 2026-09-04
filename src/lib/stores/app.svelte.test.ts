import { beforeEach, describe, expect, it } from "vitest";
import { app } from "./app.svelte";
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
  activityPercent: 0,
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
  title: "Temperatura alta",
  summary: "El disco supera el límite del fabricante",
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
