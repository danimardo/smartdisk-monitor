/** Trazado de series temporales — la regla «un hueco es un hueco» (constitución §I), implementada
 *  **una sola vez** y compartida por `Sparkline` (trazo mini, sin ejes) y `TimeSeriesChart` (con
 *  ejes, umbral, banda de hueco y pie).
 *
 *  Nada aquí sabe de píxeles: todo trabaja en el dominio (tiempo, valor). El componente escala. */

export interface Punto {
  /** ms epoch UTC. La serie debe venir ordenada por `t`. */
  t: number;
  /** `null` = sin dato. Nunca se interpola sobre un `null`. */
  v: number | null;
}

/** Cadencia de referencia entre muestras (ms): la que indique el llamante, o la mediana de las
 *  separaciones reales. Una separación mayor que 1,5× esto se trata como hueco. */
export function cadencia(points: readonly Punto[], expectedIntervalMs: number | null): number {
  if (expectedIntervalMs && expectedIntervalMs > 0) return expectedIntervalMs;
  if (points.length < 2) return 1;
  const deltas = points
    .slice(1)
    .map((p, i) => p.t - points[i].t)
    .filter((d) => d > 0)
    .sort((a, b) => a - b);
  return deltas[Math.floor(deltas.length / 2)] || 1;
}

/** Tramos continuos. Corta en un `null` explícito **y** en un salto temporal mayor que 1,5× la
 *  cadencia: la ausencia de muestra es tan informativa como un `null`. Devuelve los puntos en
 *  dominio (t, v), no en píxeles. */
export function tramos(points: readonly Punto[], step: number): { t: number; v: number }[][] {
  const out: { t: number; v: number }[][] = [];
  let actual: { t: number; v: number }[] = [];
  let prevT: number | null = null;
  for (const p of points) {
    const roto = p.v === null || (prevT !== null && p.t - prevT > step * 1.5);
    if (roto && actual.length) {
      out.push(actual);
      actual = [];
    }
    if (p.v !== null) actual.push({ t: p.t, v: p.v });
    prevT = p.t;
  }
  if (actual.length) out.push(actual);
  return out;
}

/** Bandas de ausencia de datos, en dominio de tiempo, incluidos los extremos: si la serie empieza
 *  después de `from` o termina antes de `to`, esos tramos también son huecos. */
export function huecos(
  points: readonly Punto[],
  step: number,
  from: number,
  to: number
): { from: number; to: number }[] {
  const known = points.filter((p) => p.v !== null);
  if (!known.length) return from < to ? [{ from, to }] : [];
  const out: { from: number; to: number }[] = [];
  const push = (a: number, b: number) => {
    if (b - a > step * 1.5) out.push({ from: a, to: b });
  };
  push(from, known[0].t);
  for (let i = 1; i < known.length; i++) push(known[i - 1].t, known[i].t);
  push(known.at(-1)!.t, to);
  return out;
}

/** Rango [min, max] con un 8 % de aire arriba y abajo para que la curva no toque los bordes. Si la
 *  serie es plana (o de un punto), se centra. Devuelve el rango pedido tal cual si se da explícito. */
export function rangoConAire(
  points: readonly Punto[],
  min: number | null,
  max: number | null
): { min: number; max: number } {
  if (min !== null && max !== null) return { min, max };
  const vals = points.filter((p) => p.v !== null).map((p) => p.v as number);
  if (!vals.length) return { min: min ?? 0, max: max ?? 1 };
  let lo = Math.min(...vals);
  let hi = Math.max(...vals);
  if (lo === hi) {
    lo -= 1;
    hi += 1;
  }
  const aire = (hi - lo) * 0.08;
  return { min: min ?? lo - aire, max: max ?? hi + aire };
}

/** Submuestreo a `columnas` píxeles: un mínimo y un máximo por columna, para no perder los picos.
 *  Solo se aplica por encima de ~400 puntos (30 días de muestras). Conserva los `null`. */
export function submuestrear(points: readonly Punto[], columnas: number): Punto[] {
  if (points.length <= 400 || columnas < 2) return [...points];
  const t0 = points[0].t;
  const t1 = points.at(-1)!.t;
  const span = Math.max(1, t1 - t0);
  const cubos = new Map<number, { min: Punto; max: Punto; huboNull: boolean }>();
  for (const p of points) {
    const col = Math.min(columnas - 1, Math.floor(((p.t - t0) / span) * columnas));
    const c = cubos.get(col);
    if (!c) {
      cubos.set(col, { min: p, max: p, huboNull: p.v === null });
      continue;
    }
    if (p.v === null) c.huboNull = true;
    else if (c.min.v === null || p.v < c.min.v) c.min = p;
    else if (c.max.v === null || p.v > c.max.v) c.max = p;
  }
  const out: Punto[] = [];
  for (const col of [...cubos.keys()].sort((a, b) => a - b)) {
    const c = cubos.get(col)!;
    const a = c.min.t <= c.max.t ? c.min : c.max;
    const b = a === c.min ? c.max : c.min;
    out.push(a);
    if (b.t !== a.t) out.push(b);
    if (c.huboNull) out.push({ t: (a.t + b.t) / 2, v: null });
  }
  return out.sort((p, q) => p.t - q.t);
}
