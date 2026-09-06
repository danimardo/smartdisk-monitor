import { describe, expect, it } from "vitest";
import {
  capacityState,
  classifyAgainstThresholds,
  deviceState,
  estadoConAlertas,
  globalStatus,
  selectHeroDisk,
  temperatureThresholds,
  trayState,
  worstState
} from "./health";
import type { HealthState } from "./types";

/** Estas pruebas fijan decisiones de producto, no detalles de implementación. Cada una
 *  corresponde a una entrada de `docs/open-questions.md`: si alguna falla, es que se ha cambiado
 *  una decisión, y eso debe hacerse en el documento antes que en el código. */

describe("deviceState — reconocer no apaga el color (ADR-016, §B.1)", () => {
  it("una alerta crítica reconocida sigue pintando el disco en rojo", () => {
    expect(deviceState([{ severity: "crit", status: "acknowledged" }], true)).toBe("crit");
  });

  it("una alerta resuelta deja de contar", () => {
    expect(deviceState([{ severity: "crit", status: "resolved" }], true)).toBe("ok");
  });

  it("una alerta archivada deja de contar", () => {
    expect(deviceState([{ severity: "crit", status: "archived" }], true)).toBe("ok");
  });

  it("manda la peor severidad vigente", () => {
    expect(
      deviceState(
        [
          { severity: "warn", status: "active" },
          { severity: "crit", status: "acknowledged" }
        ],
        true
      )
    ).toBe("crit");
  });

  it("sin datos frescos es desconocido, no correcto: no saber que algo va bien no es saberlo", () => {
    expect(deviceState([], false)).toBe("unknown");
    expect(deviceState([], true)).toBe("ok");
  });
});

describe("estadoConAlertas — B.1 conectado de punta a punta (el backend no funde alertas)", () => {
  const disco = (
    state: HealthState,
    volIds: string[] = [],
    unknownReason: "unsupported" | "unreadable" | "collector-error" | "not-yet-sampled" | null = null
  ) => ({
    id: "d1",
    state,
    unknownReason,
    volumes: volIds.map((id) => ({ id }))
  });
  const alerta = (
    severity: "warn" | "crit",
    key: string,
    status: "active" | "acknowledged" | "resolved" | "archived" = "active"
  ) => ({ severity, status, deduplicationKey: key });

  it("sin alertas conserva el estado de SMART tal cual", () => {
    expect(estadoConAlertas(disco("ok"), [])).toBe("ok");
    expect(estadoConAlertas(disco("unknown"), [])).toBe("unknown");
  });

  it("una alerta de dispositivo vigente eleva un disco correcto a su severidad", () => {
    expect(estadoConAlertas(disco("ok"), [alerta("warn", "temp.high|device:d1|")])).toBe("warn");
    expect(estadoConAlertas(disco("ok"), [alerta("crit", "temp.crit|device:d1|")])).toBe("crit");
  });

  it("una alerta de un volumen del disco también cuenta: un volumen lleno es problema del disco", () => {
    expect(estadoConAlertas(disco("ok", ["v9"]), [alerta("warn", "capacity.low|volume:v9")])).toBe(
      "warn"
    );
  });

  it("manda la peor severidad, y reconocer no la apaga (ADR-016)", () => {
    expect(
      estadoConAlertas(disco("ok", ["v9"]), [
        alerta("warn", "capacity.low|volume:v9"),
        alerta("crit", "smart.fail|device:d1|", "acknowledged")
      ])
    ).toBe("crit");
  });

  it("una alerta resuelta o archivada deja de contar", () => {
    expect(estadoConAlertas(disco("ok"), [alerta("crit", "x|device:d1|", "resolved")])).toBe("ok");
    expect(estadoConAlertas(disco("ok"), [alerta("crit", "x|device:d1|", "archived")])).toBe("ok");
  });

  it("una alerta de otro disco no tiñe este", () => {
    expect(estadoConAlertas(disco("ok"), [alerta("crit", "x|device:otro|")])).toBe("ok");
  });

  it("una alerta vigente sobre un disco sin SMART fresco lo saca de «unknown»", () => {
    expect(estadoConAlertas(disco("unknown"), [alerta("warn", "x|device:d1|")])).toBe("warn");
  });

  it("un disco que dejó de responder a SMART (unreadable) cuenta como advertencia, sin alerta", () => {
    expect(estadoConAlertas(disco("unknown", [], "unreadable"), [])).toBe("warn");
    expect(estadoConAlertas(disco("unknown", [], "collector-error"), [])).toBe("warn");
  });

  it("un disco sin SMART por diseño (unsupported) o aún sin medir NO cuenta como advertencia", () => {
    expect(estadoConAlertas(disco("unknown", [], "unsupported"), [])).toBe("unknown");
    expect(estadoConAlertas(disco("unknown", [], "not-yet-sampled"), [])).toBe("unknown");
  });

  it("con la monitorización en pausa, un unreadable no salta a ámbar (el estado de pausa manda)", () => {
    expect(estadoConAlertas(disco("unknown", [], "unreadable"), [], { paused: true })).toBe("unknown");
  });

  it("una alerta crítica gana a la promoción por unreadable", () => {
    expect(
      estadoConAlertas(disco("unknown", [], "unreadable"), [alerta("crit", "x|device:d1|")])
    ).toBe("crit");
  });
});

describe("capacityState — el suelo absoluto solo aplica a volúmenes grandes (ADR-019, §C.1)", () => {
  const GB = 1024 ** 3;

  it("un volumen de 64 GB con 15 GB libres (23 %) NO es crítico", () => {
    expect(capacityState(15 * GB, 64 * GB)).toBe("ok");
  });

  it("un volumen de 300 GB con 18 GB libres (6 %) es advertencia por el suelo absoluto", () => {
    // 6 % no dispararía nada por porcentaje; lo que manda aquí es el suelo de 20 GB.
    expect(capacityState(18 * GB, 300 * GB)).toBe("warn");
  });

  it("un volumen de 300 GB con 9 GB libres (3 %) es crítico", () => {
    expect(capacityState(9 * GB, 300 * GB)).toBe("crit");
  });

  it("el mismo espacio libre en un volumen pequeño no dispara el suelo absoluto", () => {
    // 18 GB libres de 200 GB son el 9 %: advertencia por porcentaje, no por el suelo.
    // De 100 GB son el 18 %: nada. El suelo absoluto no aplica por debajo de 256 GB.
    expect(capacityState(18 * GB, 100 * GB)).toBe("ok");
  });

  it("el porcentaje manda siempre, también en volúmenes pequeños", () => {
    expect(capacityState(2 * GB, 64 * GB)).toBe("crit"); // 3 %
    expect(capacityState(5 * GB, 64 * GB)).toBe("warn"); // 7,8 %
  });

  it("un dato ausente es desconocido, nunca cero", () => {
    expect(capacityState(null, 100 * GB)).toBe("unknown");
    expect(capacityState(10 * GB, null)).toBe("unknown");
  });
});

describe("trayState — prioridad del icono de bandeja (§B.5)", () => {
  it("un crítico vigente manda sobre la pausa", () => {
    expect(trayState({ paused: true, collectorFailure: false, monitoredStates: ["crit"] })).toBe("crit");
  });

  it("en pausa sin críticos, el icono es gris", () => {
    expect(trayState({ paused: true, collectorFailure: false, monitoredStates: ["ok", "warn"] })).toBe(
      "unknown"
    );
  });

  it("sin discos monitorizados, gris", () => {
    expect(trayState({ paused: false, collectorFailure: false, monitoredStates: [] })).toBe("unknown");
  });

  it("un recopilador caído degrada el icono", () => {
    expect(trayState({ paused: false, collectorFailure: true, monitoredStates: ["ok"] })).toBe("unknown");
  });
});

describe("temperatureThresholds — el límite del fabricante manda si existe (§alert-rules temp.*)", () => {
  it("sin ningún límite del fabricante, usa los configurados", () => {
    expect(temperatureThresholds(null, null, 70, 80)).toEqual({ warn: 70, crit: 80 });
  });

  it("con ambos límites del fabricante, los usa en vez de los configurados", () => {
    expect(temperatureThresholds(55, 65, 70, 80)).toEqual({ warn: 55, crit: 65 });
  });

  it("un límite parcial del fabricante solo sustituye el suyo, el otro sigue siendo el configurado", () => {
    expect(temperatureThresholds(55, null, 70, 80)).toEqual({ warn: 55, crit: 80 });
    expect(temperatureThresholds(null, 65, 70, 80)).toEqual({ warn: 70, crit: 65 });
  });
});

describe("classifyAgainstThresholds — mismos operadores que el motor de alertas", () => {
  it("justo en el umbral de aviso (estrictamente mayor) todavía no avisa", () => {
    expect(classifyAgainstThresholds(70, 70, 80)).toBe("ok");
    expect(classifyAgainstThresholds(70.1, 70, 80)).toBe("warn");
  });

  it("justo en el umbral crítico (mayor o igual) ya es crítico", () => {
    expect(classifyAgainstThresholds(80, 70, 80)).toBe("crit");
    expect(classifyAgainstThresholds(79.9, 70, 80)).toBe("warn");
  });

  it("un valor ausente es desconocido, nunca ok", () => {
    expect(classifyAgainstThresholds(null, 70, 80)).toBe("unknown");
  });
});

describe("worstState", () => {
  it("desconocido no gana a un estado conocido", () => {
    expect(worstState(["unknown", "ok"])).toBe("ok");
  });

  it("sin nada que mostrar, desconocido", () => {
    expect(worstState([])).toBe("unknown");
  });
});

describe("globalStatus — una sola fuente, dos presentaciones (09-chrome-y-estados)", () => {
  it("mientras no ha cargado el inventario, no dice 'sin discos'", () => {
    const r = globalStatus({ loaded: false, paused: false, monitoredStates: [] });
    expect(r.kind).toBe("loading");
  });

  it("cargado y sin discos: 'sin discos monitorizados', nunca en rojo", () => {
    const r = globalStatus({ loaded: true, paused: false, monitoredStates: [] });
    expect(r.kind).toBe("noDevices");
    expect(r.state).toBe("unknown");
  });

  it("en pausa manda sobre el estado de los discos para el texto", () => {
    const r = globalStatus({ loaded: true, paused: true, monitoredStates: ["warn"] });
    expect(r.kind).toBe("paused");
  });

  it("ningún disco en warn/crit: todo en orden", () => {
    const r = globalStatus({ loaded: true, paused: false, monitoredStates: ["ok", "ok", "unknown"] });
    expect(r).toEqual({ kind: "ok", state: "ok", count: 0 });
  });

  it("N en warn o crit: cuenta y peor estado", () => {
    const r = globalStatus({
      loaded: true,
      paused: false,
      monitoredStates: ["ok", "warn", "crit", "unknown"]
    });
    expect(r).toEqual({ kind: "attention", state: "crit", count: 2 });
  });

  it("un disco sin SMART (unknown) no cuenta como que necesita atención", () => {
    const r = globalStatus({ loaded: true, paused: false, monitoredStates: ["ok", "unknown"] });
    expect(r.kind).toBe("ok");
    expect(r.count).toBe(0);
  });
});

describe("selectHeroDisk — quién protagoniza el panel (HeroPanel.md)", () => {
  const disco = (id: string, over: Partial<Parameters<typeof selectHeroDisk>[0][number]> = {}) => ({
    id,
    state: "ok" as HealthState,
    unknownReason: null,
    volumes: [{ isSystemVolume: false }],
    ...over
  });
  const alerta = (deviceId: string, severity: "warn" | "crit", when: string) => ({
    severity,
    status: "active" as const,
    deduplicationKey: `temp.x|device:${deviceId}|`,
    lastOccurredAt: when
  });

  it("sin discos devuelve null", () => {
    expect(selectHeroDisk([], [])).toBeNull();
  });

  it("elige el disco con la alerta de mayor severidad", () => {
    const r = selectHeroDisk(
      [disco("a"), disco("b"), disco("c")],
      [alerta("a", "warn", "2026-09-06T10:00:00Z"), alerta("c", "crit", "2026-09-06T09:00:00Z")]
    );
    expect(r?.id).toBe("c");
  });

  it("empate de severidad: gana la ocurrencia más reciente", () => {
    const r = selectHeroDisk(
      [disco("a"), disco("b")],
      [alerta("a", "warn", "2026-09-06T08:00:00Z"), alerta("b", "warn", "2026-09-06T11:00:00Z")]
    );
    expect(r?.id).toBe("b");
  });

  it("sin alertas, elige el disco de sistema", () => {
    const r = selectHeroDisk(
      [disco("a"), disco("sys", { volumes: [{ isSystemVolume: true }] }), disco("c")],
      []
    );
    expect(r?.id).toBe("sys");
  });

  it("sin alertas y sin disco de sistema conocido, el primero del inventario", () => {
    const r = selectHeroDisk([disco("a"), disco("b")], []);
    expect(r?.id).toBe("a");
  });

  it("un disco sin SMART nunca protagoniza si hay otro", () => {
    const r = selectHeroDisk(
      [disco("usb", { state: "unknown", unknownReason: "unsupported" }), disco("nvme")],
      []
    );
    expect(r?.id).toBe("nvme");
  });

  it("salvo que el sin-SMART sea el único disco", () => {
    const r = selectHeroDisk([disco("usb", { state: "unknown", unknownReason: "unsupported" })], []);
    expect(r?.id).toBe("usb");
  });

  it("sin alerta de dispositivo pero con un disco en warn (volumen lleno o SMART ilegible), protagoniza ese, no el de sistema", () => {
    const r = selectHeroDisk(
      [
        disco("sys", { volumes: [{ isSystemVolume: true }] }),
        disco("lleno", { state: "warn" }),
        disco("ilegible", { state: "warn", unknownReason: "unreadable" })
      ],
      []
    );
    expect(r?.id).toBe("lleno");
  });

  it("un crit sin alerta de dispositivo gana a un warn", () => {
    const r = selectHeroDisk([disco("w", { state: "warn" }), disco("c", { state: "crit" })], []);
    expect(r?.id).toBe("c");
  });

  it("una alerta solo reconocida sigue eligiendo su disco (cuenta para la salud)", () => {
    const r = selectHeroDisk(
      [disco("a"), disco("b", { volumes: [{ isSystemVolume: true }] })],
      [{ ...alerta("a", "warn", "2026-09-06T10:00:00Z"), status: "acknowledged" as const }]
    );
    expect(r?.id).toBe("a");
  });
});
