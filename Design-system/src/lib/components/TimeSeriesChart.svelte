<script lang="ts">
  /** Serie temporal (US-020). Reglas duras:
   *  - **el eje X es tiempo real, no el índice de la muestra**: dos muestras separadas dos horas
   *    ocupan dos horas de ancho. Las series de esta aplicación no son equiespaciadas (huecos,
   *    pausa, reducción de frecuencia en batería, cambios de ajustes), así que repartir por índice
   *    falsearía la cronología, que es justo lo que se está investigando;
   *  - un hueco de datos se dibuja como hueco (banda gris), nunca como 0 ni interpolado;
   *  - el dominio del eje es el intervalo **pedido** (`from`/`to`), no el de los datos recibidos:
   *    si se piden 7 días y solo hay 2, los 5 restantes se ven vacíos;
   *  - el umbral del fabricante se pinta como línea discontinua ámbar;
   *  - el eje X va en hora local y todo texto pasa por i18n.
   */
  import { formatDateTime, formatTime } from "$lib/design/format";
  import { i18n, t, tp } from "$lib/i18n";

  const formatNumber = (v: number) => v.toLocaleString(i18n.formatLocale, { maximumFractionDigits: 2 });

  let {
    /** `t` en milisegundos epoch UTC; `v` null = sin dato. Debe venir ordenado por `t`. */
    points = [] as { t: number; v: number | null }[],
    min = 0,
    max = 100,
    /** Dominio temporal pedido. Si se omite, se usa el que cubran los datos. */
    from = null as number | null,
    to = null as number | null,
    /** Cadencia esperada entre muestras (ms). Una separación mayor que 1,5× se dibuja como hueco.
     *  Si se omite, se infiere de la mediana de las separaciones observadas. */
    expectedIntervalMs = null as number | null,
    threshold = null as number | null,
    thresholdLabel = "",
    height = 220,
    unit = "",
    /** Resolución servida por el backend; se muestra al usuario para que sepa que está viendo
     *  agregados y no muestras crudas (véase la tabla intervalo→resolución de la especificación). */
    resolutionLabel = "" as string
  } = $props();

  /** Ancho real en píxeles CSS: el SVG no se deforma, se redibuja. */
  let width = $state(780);

  const PAD_R = 1; // medio trazo, para que la línea no se corte en el borde
  const t0 = $derived(from ?? points.at(0)?.t ?? Date.now());
  const t1 = $derived(to ?? points.at(-1)?.t ?? Date.now());
  const span = $derived(Math.max(1, t1 - t0));

  const scaleX = (time: number) => ((time - t0) / span) * (width - PAD_R * 2) + PAD_R;
  const scaleY = (v: number) => height - ((v - min) / (max - min || 1)) * height;

  /** Cadencia de referencia: la mediana de las separaciones reales, salvo que la indique el llamante. */
  const step = $derived.by(() => {
    if (expectedIntervalMs) return expectedIntervalMs;
    if (points.length < 2) return span;
    const deltas = points.slice(1).map((p, i) => p.t - points[i].t).sort((a, b) => a - b);
    return deltas[Math.floor(deltas.length / 2)] || span;
  });

  /** Tramos continuos. Corta la línea en un `null` explícito y también en un salto temporal
   *  mayor que 1,5× la cadencia: la ausencia de muestra es tan informativa como un null. */
  const runs = $derived.by(() => {
    const out: { x: number; y: number }[][] = [];
    let current: { x: number; y: number }[] = [];
    let prevT: number | null = null;
    for (const p of points) {
      const broken = p.v === null || (prevT !== null && p.t - prevT > step * 1.5);
      if (broken && current.length) {
        out.push(current);
        current = [];
      }
      if (p.v !== null) {
        current.push({ x: scaleX(p.t), y: scaleY(p.v) });
        prevT = p.t;
      } else {
        prevT = p.t;
      }
    }
    if (current.length) out.push(current);
    return out;
  });

  /** Bandas de ausencia de datos, en coordenadas de tiempo real (incluidos los extremos:
   *  si la serie empieza después de `from`, ese tramo inicial también es un hueco). */
  const gaps = $derived.by(() => {
    const known = points.filter((p) => p.v !== null);
    if (!known.length) return [{ x: scaleX(t0), w: scaleX(t1) - scaleX(t0) }];
    const out: { x: number; w: number }[] = [];
    const push = (a: number, b: number) => {
      if (b - a > step * 1.5) out.push({ x: scaleX(a), w: scaleX(b) - scaleX(a) });
    };
    push(t0, known[0].t);
    for (let i = 1; i < known.length; i++) push(known[i - 1].t, known[i].t);
    push(known.at(-1)!.t, t1);
    return out;
  });

  const path = (run: { x: number; y: number }[]) => run.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");
  const area = (run: { x: number; y: number }[]) =>
    `${run[0].x.toFixed(1)},${height} ${path(run)} ${run.at(-1)!.x.toFixed(1)},${height}`;

  /* ---- Cursor de lectura: ratón y teclado (AGENTS.md §6 exige teclado en toda interacción) ---- */
  let cursor = $state<number | null>(null); // índice dentro de `points`
  const readable = $derived(points.map((p, i) => ({ ...p, i })).filter((p) => p.v !== null));
  const hovered = $derived(cursor === null ? null : (points[cursor] ?? null));

  function nearestIndex(clientX: number, el: SVGSVGElement): number | null {
    if (!readable.length) return null;
    const rect = el.getBoundingClientRect();
    const time = t0 + ((clientX - rect.left) / rect.width) * span;
    let best = readable[0];
    for (const p of readable) if (Math.abs(p.t - time) < Math.abs(best.t - time)) best = p;
    return best.i;
  }

  function moveCursor(delta: number) {
    if (!readable.length) return;
    const pos = cursor === null ? 0 : readable.findIndex((p) => p.i === cursor);
    const next = Math.min(readable.length - 1, Math.max(0, (pos < 0 ? 0 : pos) + delta));
    cursor = readable[next].i;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowRight") { moveCursor(1); e.preventDefault(); }
    else if (e.key === "ArrowLeft") { moveCursor(-1); e.preventDefault(); }
    else if (e.key === "Home") { cursor = readable[0]?.i ?? null; e.preventDefault(); }
    else if (e.key === "End") { cursor = readable.at(-1)?.i ?? null; e.preventDefault(); }
    else if (e.key === "Escape") cursor = null;
  }

  /** Lectura textual equivalente exigida por AGENTS.md §6: rango, extremos y huecos. */
  const summary = $derived.by(() => {
    const vals = readable.map((p) => p.v as number);
    if (!vals.length) return t("chart.emptyLabel");
    return t("chart.summaryLabel", {
      unit,
      min: Math.min(...vals),
      max: Math.max(...vals),
      last: vals.at(-1)!,
      from: formatDateTime(new Date(t0).toISOString()),
      to: formatDateTime(new Date(t1).toISOString())
    });
  });
</script>

<figure class="m-0 flex flex-col gap-3">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- La gráfica es interactiva a propósito: cursor de lectura con ratón y teclado, como exige
       AGENTS.md §6. Ver docs/known-issues.md #3 -->
  <svg
    bind:clientWidth={width}
    viewBox="0 0 {width} {height}"
    class="block w-full rounded-inner"
    style="height: {height}px"
    role="img"
    tabindex="0"
    aria-label={summary}
    onmousemove={(e) => (cursor = nearestIndex(e.clientX, e.currentTarget))}
    onmouseleave={() => (cursor = null)}
    onkeydown={onKey}
  >
    {#each [0.25, 0.5, 0.75] as g}
      <line x1="0" y1={height * g} x2={width} y2={height * g} stroke="var(--sdm-hairline)" stroke-width="1" />
    {/each}

    {#each gaps as gap}
      <rect x={gap.x} y="0" width={gap.w} height={height} fill="var(--sdm-unknown)" opacity="0.14" />
    {/each}

    {#if threshold !== null}
      <line x1="0" y1={scaleY(threshold)} x2={width} y2={scaleY(threshold)}
            stroke="var(--sdm-warn)" stroke-width="1.5" stroke-dasharray="6 6" />
    {/if}

    {#each runs as run}
      {#if run.length > 1}
        <polygon points={area(run)} fill="var(--sdm-accent)" opacity="0.12" />
        <polyline points={path(run)} fill="none" stroke="var(--sdm-accent)" stroke-width="2.5"
                  stroke-linejoin="round" stroke-linecap="round" />
      {:else}
        <circle cx={run[0].x} cy={run[0].y} r="2.5" fill="var(--sdm-accent)" />
      {/if}
    {/each}

    {#if hovered && hovered.v !== null}
      <line x1={scaleX(hovered.t)} y1="0" x2={scaleX(hovered.t)} y2={height}
            stroke="var(--sdm-accent)" stroke-width="1" opacity="0.45" />
      <circle cx={scaleX(hovered.t)} cy={scaleY(hovered.v)} r="4"
              fill="var(--sdm-accent)" stroke="var(--sdm-solid)" stroke-width="2" />
    {/if}
  </svg>

  <figcaption class="flex flex-wrap items-center justify-between gap-2 text-2xs text-fg-faint">
    <span>{formatDateTime(new Date(t0).toISOString())}</span>

    {#if hovered && hovered.v !== null}
      <span class="sdm-num text-fg">{formatTime(new Date(hovered.t).toISOString())} · {formatNumber(hovered.v)} {unit}</span>
    {:else}
      <span class="flex gap-3">
        {#if gaps.length}<span>{tp("chart.gaps", gaps.length)}</span>{/if}
        {#if resolutionLabel}<span>{resolutionLabel}</span>{/if}
      </span>
    {/if}

    {#if thresholdLabel}<span class="text-warn">{thresholdLabel}</span>{/if}
    <span>{formatDateTime(new Date(t1).toISOString())}</span>
  </figcaption>
</figure>
