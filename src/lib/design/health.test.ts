import { describe, expect, it } from "vitest";
import {
  capacityState,
  classifyAgainstThresholds,
  deviceState,
  temperatureThresholds,
  trayState,
  worstState
} from "./health";

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
