<script lang="ts">
  /** Trazo de una serie temporal **sin ejes ni etiquetas** (v3, ADR-034): contexto, no lectura.
   *  Va dentro de `MetricCard`, de la cabecera de `DiskCard` y de fondo del `HeroPanel`.
   *
   *  Las cuatro reglas de trazado (`Sparkline.md` §2), todas heredadas de `$lib/design/series.ts`
   *  para implementarlas una sola vez:
   *   1. un `<polyline>` por tramo continuo; los `null` parten la serie y **nunca** se interpola;
   *   2. `vector-effect="non-scaling-stroke"` en todo trazo — con `preserveAspectRatio="none"`
   *      (necesario para estirar el SVG al ancho del contenedor) el grosor se deforma y a alturas
   *      pequeñas el trazo desaparece;
   *   3. eje X por tiempo real, no por índice;
   *   4. rango con 8 % de aire, salvo que se pase `min`/`max`. */
  import { cadencia, rangoConAire, submuestrear, tramos, type Punto } from "$lib/design/series";

  let {
    points = [] as Punto[],
    min = null as number | null,
    max = null as number | null,
    color = "var(--sdm-accent)",
    height = 22,
    fill = false,
    strokeWidth = 1.8,
    expectedIntervalMs = null as number | null,
    /** Línea discontinua horizontal (umbral del fabricante). No es un eje: una sola referencia. */
    threshold = null as number | null,
    label = undefined as string | undefined
  } = $props();

  /** El `id` del degradado tiene que ser único por instancia: con varias sparklines en pantalla, un
   *  `id` repetido hace que todas usen el primero. */
  const gradId = `spark-${crypto.randomUUID()}`;

  const W = 100; // el viewBox; `preserveAspectRatio="none"` lo estira al ancho real

  const muestras = $derived(submuestrear(points, W));
  const step = $derived(cadencia(muestras, expectedIntervalMs));
  const rango = $derived(rangoConAire(muestras, min, max));
  const t0 = $derived(muestras.at(0)?.t ?? 0);
  const t1 = $derived(muestras.at(-1)?.t ?? 1);
  const span = $derived(Math.max(1, t1 - t0));

  const sx = (t: number) => ((t - t0) / span) * W;
  const sy = (v: number) => height - ((v - rango.min) / (rango.max - rango.min || 1)) * height;

  const runs = $derived(tramos(muestras, step).map((run) => run.map((p) => ({ x: sx(p.t), y: sy(p.v) }))));

  const linea = (run: { x: number; y: number }[]) =>
    run.map((p) => `${p.x.toFixed(2)},${p.y.toFixed(2)}`).join(" ");
  const relleno = (run: { x: number; y: number }[]) =>
    `${run[0].x.toFixed(2)},${height} ${linea(run)} ${run.at(-1)!.x.toFixed(2)},${height}`;

  const hayAlgo = $derived(runs.some((r) => r.length > 0));
</script>

{#if hayAlgo}
  <svg
    viewBox="0 0 {W} {height}"
    preserveAspectRatio="none"
    class="block w-full"
    style="height: {height}px; color: {color}"
    role={label ? "img" : undefined}
    aria-label={label}
    aria-hidden={label ? undefined : "true"}
  >
    {#if fill}
      <defs>
        <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="currentColor" stop-opacity="0.32" />
          <stop offset="1" stop-color="currentColor" stop-opacity="0" />
        </linearGradient>
      </defs>
    {/if}
    {#if threshold != null && threshold >= rango.min && threshold <= rango.max}
      <line
        x1="0"
        y1={sy(threshold)}
        x2={W}
        y2={sy(threshold)}
        stroke="currentColor"
        stroke-width="1"
        stroke-dasharray="3 3"
        opacity="0.5"
        vector-effect="non-scaling-stroke"
      />
    {/if}
    {#each runs as run}
      {#if run.length > 1}
        {#if fill}
          <polygon points={relleno(run)} fill="url(#{gradId})" />
        {/if}
        <polyline
          points={linea(run)}
          fill="none"
          stroke="currentColor"
          stroke-width={strokeWidth}
          stroke-linejoin="round"
          stroke-linecap="round"
          vector-effect="non-scaling-stroke"
        />
      {:else if run.length === 1}
        <circle cx={run[0].x} cy={run[0].y} r="1.5" fill="currentColor" vector-effect="non-scaling-stroke" />
      {/if}
    {/each}
  </svg>
{:else}
  <!-- Serie vacía o toda `null`: el hueco se queda en blanco. NO se dibuja una línea a cero. -->
  <div style="height: {height}px" aria-hidden="true"></div>
{/if}
