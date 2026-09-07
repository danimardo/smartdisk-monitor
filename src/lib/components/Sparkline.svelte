<script lang="ts">
  /** Trazo de una serie temporal **sin ejes ni etiquetas** (v3, ADR-034). Por defecto es contexto,
   *  no lectura (fondo de `HeroPanel` y cabecera de `DiskCard`). Con `interactivo` (lo usa
   *  `MetricCard`) gana el mismo cursor de lectura que `TimeSeriesChart`: ratón **y** teclado, con
   *  el valor del punto en un globo (`ChartTip`).
   *
   *  Las cuatro reglas de trazado (`Sparkline.md` §2), todas heredadas de `$lib/design/series.ts`
   *  para implementarlas una sola vez:
   *   1. un `<path>` curvo (spline monótona, `rutaSuave`) por tramo continuo; los `null` y los
   *      saltos grandes parten la serie y **nunca** se interpola sobre un hueco;
   *   2. `vector-effect="non-scaling-stroke"` en todo trazo — con `preserveAspectRatio="none"`
   *      (necesario para estirar el SVG al ancho del contenedor) el grosor se deforma y a alturas
   *      pequeñas el trazo desaparece;
   *   3. eje X por tiempo real, no por índice;
   *   4. rango con 8 % de aire, salvo que se pase `min`/`max`. */
  import {
    areaSuave,
    cadencia,
    masCercano,
    rangoConAire,
    rutaSuave,
    submuestrear,
    tramos,
    type Punto
  } from "$lib/design/series";
  import { formatDateTime } from "$lib/design/format";
  import { i18n, t } from "$lib/i18n";
  import ChartTip from "./ChartTip.svelte";

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
    label = undefined as string | undefined,
    /** Cursor de lectura (ratón + teclado) con el valor del punto en un globo. Por defecto no: el
     *  fondo de `HeroPanel`/`DiskCard` es decorativo. */
    interactivo = false,
    /** Unidad para el texto del globo cuando `interactivo` (p. ej. "°C", "%"). */
    unidad = ""
  } = $props();

  /** El `id` del degradado tiene que ser único por instancia: con varias sparklines en pantalla, un
   *  `id` repetido hace que todas usen el primero. */
  const gradId = `spark-${crypto.randomUUID()}`;

  const W = 100; // el viewBox; `preserveAspectRatio="none"` lo estira al ancho real

  const formatNumber = (v: number) => v.toLocaleString(i18n.formatLocale, { maximumFractionDigits: 2 });

  const muestras = $derived(submuestrear(points, W));
  const step = $derived(cadencia(muestras, expectedIntervalMs));
  const rango = $derived(rangoConAire(muestras, min, max));
  const t0 = $derived(muestras.at(0)?.t ?? 0);
  const t1 = $derived(muestras.at(-1)?.t ?? 1);
  const span = $derived(Math.max(1, t1 - t0));

  const sx = (t: number) => ((t - t0) / span) * W;
  const sy = (v: number) => height - ((v - rango.min) / (rango.max - rango.min || 1)) * height;

  const runs = $derived(tramos(muestras, step).map((run) => run.map((p) => ({ x: sx(p.t), y: sy(p.v) }))));

  const hayAlgo = $derived(runs.some((r) => r.length > 0));

  /* ---- Cursor de lectura (solo con `interactivo`) ------------------------------------------------ */
  let anchoPx = $state(0);
  let cursorIdx = $state<number | null>(null);
  const legibles = $derived(muestras.filter((p): p is { t: number; v: number } => p.v !== null));
  const hovered = $derived(cursorIdx === null ? null : (legibles[cursorIdx] ?? null));
  const puntoPx = $derived(hovered ? { x: (sx(hovered.t) / W) * anchoPx, y: sy(hovered.v) } : null);

  function alPuntero(e: PointerEvent) {
    if (!interactivo || !legibles.length) return;
    const rect = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    const time = t0 + ((e.clientX - rect.left) / rect.width) * span;
    const p = masCercano(legibles, time);
    cursorIdx = p ? legibles.indexOf(p) : null;
  }

  function moverCursor(delta: number) {
    if (!legibles.length) return;
    cursorIdx = Math.max(0, Math.min(legibles.length - 1, (cursorIdx ?? 0) + delta));
  }

  function alTeclado(e: KeyboardEvent) {
    if (!interactivo) return;
    if (e.key === "ArrowRight") {
      moverCursor(1);
      e.preventDefault();
    } else if (e.key === "ArrowLeft") {
      moverCursor(-1);
      e.preventDefault();
    } else if (e.key === "Home") {
      cursorIdx = 0;
      e.preventDefault();
    } else if (e.key === "End") {
      cursorIdx = legibles.length - 1;
      e.preventDefault();
    } else if (e.key === "Escape") {
      cursorIdx = null;
    }
  }

  /** Lectura textual equivalente (constitución §VII) cuando es interactiva. */
  const resumen = $derived.by(() => {
    if (!interactivo) return label;
    const vals = legibles.map((p) => p.v);
    if (!vals.length) return label ?? t("chart.emptyLabel");
    return t("chart.summaryLabel", {
      unit: unidad,
      min: Math.min(...vals),
      max: Math.max(...vals),
      last: vals.at(-1)!,
      from: formatDateTime(new Date(t0).toISOString()),
      to: formatDateTime(new Date(t1).toISOString())
    });
  });

  const lectura = $derived(
    hovered
      ? t("chart.readout", {
          value: formatNumber(hovered.v),
          unit: unidad,
          when: formatDateTime(new Date(hovered.t).toISOString())
        })
      : ""
  );
</script>

<div class="relative" bind:clientWidth={anchoPx}>
  {#if hayAlgo}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- Con `interactivo` el SVG es un objetivo de lectura (ratón + teclado): `role="img"` +
         `tabindex="0"` + `aria-label` + flechas, el mismo patrón que `TimeSeriesChart`.
         Ver docs/known-issues.md #3 -->
    <svg
      viewBox="0 0 {W} {height}"
      preserveAspectRatio="none"
      class="block w-full"
      style="height: {height}px; color: {color}"
      role={interactivo || label ? "img" : undefined}
      aria-label={interactivo ? resumen : label}
      aria-hidden={interactivo || label ? undefined : "true"}
      tabindex={interactivo ? 0 : undefined}
      onpointermove={alPuntero}
      onpointerleave={() => (cursorIdx = null)}
      onkeydown={alTeclado}
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
            <path d={areaSuave(run, height)} fill="url(#{gradId})" />
          {/if}
          <path
            d={rutaSuave(run)}
            fill="none"
            stroke="currentColor"
            stroke-width={strokeWidth}
            stroke-linejoin="round"
            stroke-linecap="round"
            vector-effect="non-scaling-stroke"
          />
        {:else if run.length === 1}
          <circle
            cx={run[0].x}
            cy={run[0].y}
            r="1.5"
            fill="currentColor"
            vector-effect="non-scaling-stroke"
          />
        {/if}
      {/each}
      {#if hovered}
        <circle
          cx={sx(hovered.t)}
          cy={sy(hovered.v)}
          r="2.5"
          fill="currentColor"
          stroke="var(--sdm-solid)"
          stroke-width="1.5"
          vector-effect="non-scaling-stroke"
        />
      {/if}
    </svg>

    {#if hovered && puntoPx}
      <ChartTip x={puntoPx.x} y={puntoPx.y} anchoContenedor={anchoPx} texto={lectura} />
    {/if}
    {#if interactivo}
      <span class="sr-only" aria-live="polite">{lectura}</span>
    {/if}
  {:else}
    <!-- Serie vacía o toda `null`: el hueco se queda en blanco. NO se dibuja una línea a cero. -->
    <div style="height: {height}px" aria-hidden="true"></div>
  {/if}
</div>
