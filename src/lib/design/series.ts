/** Trazado de series temporales — la regla «un hueco es un hueco» (constitución §I), implementada
 *  **una sola vez** y compartida por `Sparkline` (trazo mini, sin ejes) y `TimeSeriesChart` (con
 *  ejes, umbral, banda de hueco y pie).
 *
 *  El troceo (`tramos`, `huecos`, `cadencia`) trabaja en el dominio (tiempo, valor); el componente
 *  escala. La única excepción son `rutaSuave`/`areaSuave`, que reciben el tramo **ya escalado a
 *  píxeles** y devuelven el `d` de un `<path>` curvo — la curva es un detalle de dibujo, no del
 *  dominio. */

export interface Punto {
  /** ms epoch UTC. La serie debe venir ordenada por `t`. */
  t: number;
  /** `null` = sin dato. Nunca se interpola sobre un `null`. */
  v: number | null;
}

/** Factor sobre la cadencia a partir del cual una separación entre muestras es un hueco real y no
 *  un ciclo de recopilación puntualmente perdido. 2,5× ≈ dos ciclos: un salto suelto no parte la
 *  línea, una parada de minutos u horas sí (`docs/open-questions.md` E.1). Debe coincidir con
 *  `MULTIPLO_HUECO` de `src-tauri/src/domain/series.rs`, que ya inserta los `null` en el backend. */
const FACTOR_HUECO = 2.5;

/** Cadencia de referencia entre muestras (ms): la que indique el llamante, o la mediana de las
 *  separaciones reales. Una separación mayor que `FACTOR_HUECO`× esto se trata como hueco. */
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

/** Tramos continuos. Corta en un `null` explícito **y** en un salto temporal mayor que `FACTOR_HUECO`× la
 *  cadencia: la ausencia de muestra es tan informativa como un `null`. Devuelve los puntos en
 *  dominio (t, v), no en píxeles. */
export function tramos(points: readonly Punto[], step: number): { t: number; v: number }[][] {
  const out: { t: number; v: number }[][] = [];
  let actual: { t: number; v: number }[] = [];
  let prevT: number | null = null;
  for (const p of points) {
    const roto = p.v === null || (prevT !== null && p.t - prevT > step * FACTOR_HUECO);
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

/** Recorta la serie a su **último tramo sin cortes**: desde la última separación grande entre
 *  muestras hasta el final. Para el panel general (v3, `Sparkline.md`): con la app parada a ratos
 *  —se cierra, se reinicia el equipo, se acaba de instalar— la ventana fija de 24 h deja huecos de
 *  horas que se dibujan como rayas sueltas; enseñar solo el tramo en curso devuelve la onda
 *  continua del boceto, y su ancho se adapta a lo que hay (un minuto de datos → ventana de un
 *  minuto).
 *
 *  Un salto se considera «la app estuvo parada» si supera `factorCorte` veces la cadencia normal.
 *  Como referencia de cadencia se usa el **percentil 25** de las separaciones, no la mediana: con
 *  pocas muestras y un parón, la mitad de las separaciones *son* el parón y la mediana se
 *  contamina. `tramos()` sigue partiendo el trazo **dentro** de la ventana ya recortada —dibujar
 *  un hueco es otro trabajo—. Cuenta el tiempo real entre muestras, `null` incluidos. Serie vacía
 *  → todo a cero. */
export function ultimoTramoVisible(
  points: readonly Punto[],
  factorCorte = 4
): { points: Punto[]; desde: number; hasta: number } {
  if (points.length === 0) return { points: [], desde: 0, hasta: 0 };
  const deltas = points
    .slice(1)
    .map((p, i) => p.t - points[i].t)
    .filter((d) => d > 0)
    .sort((a, b) => a - b);
  const cadenciaBase = deltas.length ? deltas[Math.floor(deltas.length * 0.25)] : 1;
  const umbral = cadenciaBase * factorCorte;
  let inicio = 0;
  for (let i = points.length - 1; i > 0; i--) {
    if (points[i].t - points[i - 1].t > umbral) {
      inicio = i;
      break;
    }
  }
  const cola = points.slice(inicio);
  return { points: cola, desde: cola[0].t, hasta: cola.at(-1)!.t };
}

/** Bandas de ausencia de datos, en dominio de tiempo. Es el negativo de `tramos`: el espacio
 *  **entre** dos tramos continuos es un hueco (lo abra un `null` explícito o un salto grande), y si
 *  la serie empieza después de `from` o termina antes de `to`, esos bordes también lo son. Así la
 *  banda gris de `TimeSeriesChart` coincide exactamente con dónde se parte el trazo. */
export function huecos(
  points: readonly Punto[],
  step: number,
  from: number,
  to: number
): { from: number; to: number }[] {
  const runs = tramos(points, step);
  if (!runs.length) return from < to ? [{ from, to }] : [];
  const out: { from: number; to: number }[] = [];
  if (runs[0][0].t - from > step * FACTOR_HUECO) out.push({ from, to: runs[0][0].t });
  for (let i = 1; i < runs.length; i++) {
    out.push({ from: runs[i - 1].at(-1)!.t, to: runs[i][0].t });
  }
  const finTramos = runs.at(-1)!.at(-1)!.t;
  if (to - finTramos > step * FACTOR_HUECO) out.push({ from: finTramos, to });
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

// ---- Curva del trazo (dominio de píxeles, ya escalado por el componente) --------------------------

interface Pt {
  x: number;
  y: number;
}

const redondea = (n: number) => Number(n.toFixed(2));

/** Descarta puntos cuyo `x` no avanza —dos tiempos que caen en el mismo píxel—: dejarían una
 *  secante de pendiente infinita. */
function sinXRepetida(pts: readonly Pt[]): Pt[] {
  const out: Pt[] = [];
  for (const p of pts) {
    if (out.length === 0 || p.x - out[out.length - 1].x > 1e-6) out.push({ x: p.x, y: p.y });
  }
  return out;
}

/** Trazo curvo de un tramo **ya escalado a píxeles** (`x` creciente): spline cúbica de Hermite
 *  **monótona** (Fritsch–Carlson). «Monótona» = entre dos muestras la curva no se sale del rango de
 *  sus valores, así que nunca aparenta cruzar un umbral que los datos no cruzan. Devuelve el `d` de
 *  un `<path>` (empieza en `M`). Dos puntos → una recta; menos de dos → cadena vacía.
 *
 *  Se llama **por tramo** (los `null` y los huecos ya los partió `tramos`), así que la curva jamás
 *  puentea un hueco. */
export function rutaSuave(pts: readonly Pt[]): string {
  const p = sinXRepetida(pts);
  if (p.length === 0) return "";
  if (p.length === 1) return `M ${redondea(p[0].x)},${redondea(p[0].y)}`;
  if (p.length === 2) {
    return `M ${redondea(p[0].x)},${redondea(p[0].y)} L ${redondea(p[1].x)},${redondea(p[1].y)}`;
  }

  const n = p.length;
  const dx: number[] = [];
  const secante: number[] = []; // pendiente del segmento i → i+1
  for (let i = 0; i < n - 1; i++) {
    dx[i] = p[i + 1].x - p[i].x;
    secante[i] = (p[i + 1].y - p[i].y) / dx[i];
  }

  // Tangente en cada punto: media de las secantes vecinas, salvo en un extremo local (cambio de
  // signo) donde se aplana a 0 para no rebasar el pico.
  const m: number[] = Array.from({ length: n }, () => 0);
  m[0] = secante[0];
  m[n - 1] = secante[n - 2];
  for (let i = 1; i < n - 1; i++) {
    m[i] = secante[i - 1] * secante[i] <= 0 ? 0 : (secante[i - 1] + secante[i]) / 2;
  }

  // Restricción de monotonía de Fritsch–Carlson.
  for (let i = 0; i < n - 1; i++) {
    if (secante[i] === 0) {
      m[i] = 0;
      m[i + 1] = 0;
      continue;
    }
    const a = m[i] / secante[i];
    const b = m[i + 1] / secante[i];
    const suma = a * a + b * b;
    if (suma > 9) {
      const tau = 3 / Math.sqrt(suma);
      m[i] = tau * a * secante[i];
      m[i + 1] = tau * b * secante[i];
    }
  }

  let d = `M ${redondea(p[0].x)},${redondea(p[0].y)}`;
  for (let i = 0; i < n - 1; i++) {
    const c1x = p[i].x + dx[i] / 3;
    const c1y = p[i].y + (m[i] * dx[i]) / 3;
    const c2x = p[i + 1].x - dx[i] / 3;
    const c2y = p[i + 1].y - (m[i + 1] * dx[i]) / 3;
    d += ` C ${redondea(c1x)},${redondea(c1y)} ${redondea(c2x)},${redondea(c2y)} ${redondea(p[i + 1].x)},${redondea(p[i + 1].y)}`;
  }
  return d;
}

/** La curva de `rutaSuave` cerrada hasta `base` (la línea de fondo del SVG), para el relleno
 *  degradado bajo el trazo. */
export function areaSuave(pts: readonly Pt[], base: number): string {
  const p = sinXRepetida(pts);
  if (p.length < 2) return "";
  const curva = rutaSuave(p).slice(1).trimStart(); // sin la `M` inicial
  return `M ${redondea(p[0].x)},${redondea(base)} L ${curva} L ${redondea(p[p.length - 1].x)},${redondea(base)} Z`;
}
