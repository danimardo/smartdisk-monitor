import { describe, expect, it } from "vitest";
import { cadencia, huecos, rangoConAire, submuestrear, tramos, type Punto } from "./series";

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

  it("un salto mayor que 1,5× la cadencia también parte la serie, aunque no haya null", () => {
    const p: Punto[] = [
      { t: 0, v: 1 },
      { t: 10, v: 2 },
      { t: 100, v: 3 },
      { t: 110, v: 4 }
    ];
    const r = tramos(p, 10);
    expect(r).toHaveLength(2);
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
