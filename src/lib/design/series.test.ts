import { describe, expect, it } from "vitest";
import {
  areaSuave,
  cadencia,
  huecos,
  rangoConAire,
  rutaSuave,
  submuestrear,
  tramos,
  ultimoTramoVisible,
  type Punto
} from "./series";

/** «Un hueco es un hueco» (constitución §I): estas pruebas fijan que la serie nunca se interpola
 *  sobre un `null` ni sobre un salto temporal, y que el rango deja aire. Es lógica compartida por
 *  `Sparkline` y `TimeSeriesChart`; si falla, las dos mienten a la vez. */

describe("tramos — corta en null y en salto temporal", () => {
  it("un null explícito parte la serie en dos tramos", () => {
    const p: Punto[] = [
      { t: 0, v: 10 },
      { t: 1, v: 11 },
      { t: 2, v: null },
      { t: 3, v: 20 },
      { t: 4, v: 21 }
    ];
    const r = tramos(p, 1);
    expect(r).toHaveLength(2);
    expect(r[0].map((x) => x.v)).toEqual([10, 11]);
    expect(r[1].map((x) => x.v)).toEqual([20, 21]);
  });

  it("un salto grande (varios ciclos) también parte la serie, aunque no haya null", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 10, v: 2 },
      { t: 100, v: 3 }, // 90 ms de salto con cadencia 10: 9× → hueco
      { t: 110, v: 4 }
    ];
    const r = tramos(p, 10);
    expect(r).toHaveLength(2);
  });

  it("un ciclo suelto perdido (≤ 2,5× la cadencia) NO parte la serie", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 10, v: 2 },
      { t: 32, v: 3 }, // 22 ms con cadencia 10: 2,2× → aún continuo
      { t: 42, v: 4 }
    ];
    expect(tramos(p, 10)).toHaveLength(1);
  });

  it("nunca inserta un punto en el hueco: los valores de salida son exactamente los de entrada", () => {
    const p: Punto[] = [
      { t: 0, v: 5 },
      { t: 1, v: null },
      { t: 2, v: 9 }
    ];
    const planos = tramos(p, 1).flat().map((x) => x.v);
    expect(planos).toEqual([5, 9]);
  });

  it("serie vacía o toda null: ningún tramo", () => {
    expect(tramos([], 1)).toEqual([]);
    expect(tramos([{ t: 0, v: null }], 1)).toEqual([]);
  });
});

describe("ultimoTramoVisible — la ventana del panel se adapta al último tramo sin cortes", () => {
  const MIN = 60_000;

  it("serie con un parón largo en medio: solo el tramo posterior", () => {
    const p: Punto[] = [
      { t: 0, v: 40 },
      { t: 5 * MIN, v: 41 },
      { t: 10 * MIN, v: 42 },
      // parón de 17 h
      { t: 17 * 60 * MIN, v: 45 },
      { t: 17 * 60 * MIN + 5 * MIN, v: 46 },
      { t: 17 * 60 * MIN + 10 * MIN, v: 47 }
    ];
    const r = ultimoTramoVisible(p);
    expect(r.points.map((x) => x.v)).toEqual([45, 46, 47]);
    expect(r.desde).toBe(17 * 60 * MIN);
    expect(r.hasta).toBe(17 * 60 * MIN + 10 * MIN);
  });

  it("un ciclo perdido no abre tramo nuevo: el hueco pequeño no corta la ventana", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 5 * MIN, v: 2 },
      { t: 15 * MIN, v: 3 }, // 10 min: solo dos veces la cadencia
      { t: 20 * MIN, v: 4 }
    ];
    expect(ultimoTramoVisible(p).points).toHaveLength(4);
  });

  it("serie densa y entera: no la toca", () => {
    const p: Punto[] = Array.from({ length: 50 }, (_, i) => ({ t: i * MIN, v: i }));
    expect(ultimoTramoVisible(p).points).toHaveLength(50);
  });

  it("un solo punto tras el parón: ese punto (sin mínimo de zoom)", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 5 * MIN, v: 2 },
      { t: 300 * MIN, v: 9 }
    ];
    const r = ultimoTramoVisible(p);
    expect(r.points).toEqual([{ t: 300 * MIN, v: 9 }]);
    expect(r.desde).toBe(r.hasta);
  });

  it("serie vacía: todo a cero", () => {
    expect(ultimoTramoVisible([])).toEqual({ points: [], desde: 0, hasta: 0 });
  });
});

describe("huecos — incluye los extremos", () => {
  it("si la serie empieza después de `from`, ese tramo inicial es un hueco", () => {
    const p: Punto[] = [
      { t: 100, v: 1 },
      { t: 110, v: 2 }
    ];
    const g = huecos(p, 10, 0, 120);
    expect(g).toEqual([{ from: 0, to: 100 }]);
  });

  it("serie sin ningún dato: todo el dominio pedido es un hueco", () => {
    expect(huecos([], 1, 0, 1000)).toEqual([{ from: 0, to: 1000 }]);
  });

  it("datos densos y completos: sin huecos", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 10, v: 2 },
      { t: 20, v: 3 }
    ];
    expect(huecos(p, 10, 0, 20)).toEqual([]);
  });
});

describe("cadencia", () => {
  it("usa el valor del llamante si es válido", () => {
    expect(cadencia([{ t: 0, v: 1 }], 42)).toBe(42);
  });
  it("si no, la mediana de las separaciones reales", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 10, v: 2 },
      { t: 20, v: 3 },
      { t: 200, v: 4 }
    ];
    expect(cadencia(p, null)).toBe(10);
  });
});

describe("rangoConAire", () => {
  it("respeta min/max si se dan los dos", () => {
    expect(rangoConAire([{ t: 0, v: 50 }], 0, 100)).toEqual({ min: 0, max: 100 });
  });
  it("deja un 8 % de aire cuando se calcula de la serie", () => {
    const r = rangoConAire(
      [
        { t: 0, v: 40 },
        { t: 1, v: 60 }
      ],
      null,
      null
    );
    expect(r.min).toBeCloseTo(40 - 1.6);
    expect(r.max).toBeCloseTo(60 + 1.6);
  });
  it("serie plana: se centra, no colapsa a una línea en el borde", () => {
    const r = rangoConAire([{ t: 0, v: 30 }, { t: 1, v: 30 }], null, null);
    expect(r.min).toBeLessThan(30);
    expect(r.max).toBeGreaterThan(30);
  });
});

describe("submuestrear", () => {
  it("por debajo de 400 puntos no toca nada", () => {
    const p: Punto[] = Array.from({ length: 100 }, (_, i) => ({ t: i, v: i }));
    expect(submuestrear(p, 50)).toHaveLength(100);
  });
  it("por encima de 400 reduce, conservando el pico máximo de la serie", () => {
    const p: Punto[] = Array.from({ length: 1000 }, (_, i) => ({ t: i, v: i === 500 ? 9999 : 1 }));
    const out = submuestrear(p, 80);
    expect(out.length).toBeLessThan(1000);
    expect(Math.max(...out.map((x) => x.v ?? 0))).toBe(9999);
  });
});

describe("rutaSuave / areaSuave — curva del trazo (dominio de píxeles)", () => {
  const ys = (d: string) =>
    [...d.matchAll(/(-?\d+(?:\.\d+)?),(-?\d+(?:\.\d+)?)/g)].map((m) => Number(m[2]));

  it("empieza en el primer punto del tramo", () => {
    const d = rutaSuave([
      { x: 0, y: 5 },
      { x: 10, y: 8 },
      { x: 20, y: 3 }
    ]);
    expect(d.startsWith("M 0,5")).toBe(true);
  });

  it("dos puntos: una recta (L), sin curva (C)", () => {
    const d = rutaSuave([
      { x: 0, y: 0 },
      { x: 10, y: 10 }
    ]);
    expect(d).toContain(" L ");
    expect(d).not.toContain(" C ");
  });

  it("tres o más puntos: curva cúbica (C)", () => {
    const d = rutaSuave([
      { x: 0, y: 0 },
      { x: 10, y: 5 },
      { x: 20, y: 2 }
    ]);
    expect(d).toContain(" C ");
  });

  it("un pico: la curva no se pasa del valor del pico (spline monótona, sin overshoot)", () => {
    const d = rutaSuave([
      { x: 0, y: 0 },
      { x: 10, y: 10 },
      { x: 20, y: 0 }
    ]);
    const todas = ys(d);
    expect(Math.max(...todas)).toBeLessThanOrEqual(10 + 1e-6);
    expect(Math.min(...todas)).toBeGreaterThanOrEqual(0 - 1e-6);
  });

  it("un tramo plano queda plano: ninguna coordenada y se desvía", () => {
    const d = rutaSuave([
      { x: 0, y: 7 },
      { x: 10, y: 7 },
      { x: 20, y: 7 },
      { x: 30, y: 7 }
    ]);
    for (const y of ys(d)) expect(y).toBeCloseTo(7);
  });

  it("descarta puntos cuyo x no avanza (dos tiempos en el mismo píxel)", () => {
    const d = rutaSuave([
      { x: 0, y: 0 },
      { x: 0, y: 100 },
      { x: 10, y: 10 }
    ]);
    // Se queda con (0,0) y (10,10): una recta, no una vertical imposible.
    expect(d).toBe("M 0,0 L 10,10");
  });

  it("menos de dos puntos: cadena vacía para el área", () => {
    expect(areaSuave([{ x: 0, y: 0 }], 50)).toBe("");
  });

  it("el área cierra contra la base y termina en Z", () => {
    const d = areaSuave(
      [
        { x: 0, y: 10 },
        { x: 10, y: 20 },
        { x: 20, y: 15 }
      ],
      100
    );
    expect(d.startsWith("M 0,100")).toBe(true);
    expect(d.trimEnd().endsWith("Z")).toBe(true);
    expect(d).toContain("20,100");
  });
});
