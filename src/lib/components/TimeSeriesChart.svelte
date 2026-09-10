<script lang="ts">
  /** Serie temporal (US-020), v3. Reglas duras:
   *  - **el eje X es tiempo real, no el índice de la muestra**: dos muestras separadas dos horas
   *    ocupan dos horas de ancho. Las series de esta aplicación no son equiespaciadas, así que
   *    repartir por índice falsearía la cronología, que es justo lo que se está investigando;
   *  - un hueco de datos se dibuja como hueco (banda gris **con su leyenda `sin datos HH:MM–HH:MM`**),
   *    nunca como 0 ni interpolado;
   *  - el dominio del eje es el intervalo **pedido** (`from`/`to`), no el de los datos recibidos;
   *  - **eje Y** de 34 px con cuatro marcas, fuera del área de trazo, para saber a qué altura está
   *    cada valor y dónde queda el límite;
   *  - `warnThreshold`/`critThreshold` se pintan como zonas de fondo **y** como discontinua con
   *    leyenda propia;
   *  - el trazo comparte con `Sparkline` la lógica de tramos y huecos (`$lib/design/series.ts`): «un
   *    hueco es un hueco» se implementa una sola vez;
   *  - el eje X y las leyendas van en hora local y todo texto pasa por i18n.
   */
  import {
    areaSuave,
    cadencia,
    huecos,
    masCercano,
    rangoConAire,
    rutaSuave,
    tramos
  } from "$lib/design/series";
  import { formatDateTime, formatTime } from "$lib/design/format";
  import { i18n, t } from "$lib/i18n";
  import ChartTip from "./ChartTip.svelte";

  const formatNumber = (v: number) => v.toLocaleString(i18n.formatLocale, { maximumFractionDigits: 2 });

  let {
    /** `t` en milisegundos epoch UTC; `v` null = sin dato. Debe venir ordenado por `t`. */
    points = [] as { t: number; v: number | null }[],
    min = null as number | null,
    max = null as number | null,
    /** Dominio temporal pedido. Si se omite, se usa el que cubran los datos. */
    from = null as number | null,
    to = null as number | null,
    /** Cadencia esperada entre muestras (ms). Una separación mayor que 2,5× se dibuja como hueco. */
    expectedIntervalMs = null as number | null,
    warnThreshold = null as number | null,
    warnLabel = "",
    critThreshold = null as number | null,
    critLabel = "",
    height = 220,
    unit = "",
    /** Color de la serie (`var(--sdm-*)`). Sigue el estado: `--sdm-warn` en advertencia térmica,
     *  `--sdm-accent` cuando el disco está correcto. */
    color = "var(--sdm-accent)",
    /** Resolución servida por el backend; se muestra para que el usuario sepa que ve agregados. */
    resolutionLabel = "" as string,
    /** Prefijo de la etiqueta accesible (`role="img"`), **no** un título visible: el llamante pone
     *  el encabezado (p. ej. la cabecera de su panel). Obligatorio cuando la pantalla apila varias
     *  gráficas (detalle de disco: temperatura y actividad) para que cada `role="img"` se
     *  distinga con un lector de pantalla. */
    titulo = "" as string,
    /** Texto del estado vacío. Por defecto «Sin muestras en el intervalo»; una pantalla que sabe
     *  que el historial aún se está poblando (actividad recién arrancada) pasa aquí su propio
     *  mensaje de «recopilando». */
    textoVacio = "" as string
  } = $props();

  const GUTTER = 34; // ancho del eje Y, fuera del área de trazo
  const PAD_R = 2;

  /** Ancho real en píxeles CSS: el SVG no se deforma, se redibuja. */
  let width = $state(780);
  const plotW = $derived(Math.max(1, width - GUTTER - PAD_R));

  const t0 = $derived(from ?? points.at(0)?.t ?? Date.now());
  const t1 = $derived(to ?? points.at(-1)?.t ?? Date.now());
  const span = $derived(Math.max(1, t1 - t0));

  const rango = $derived(rangoConAire(points, min, max));
  const step = $derived(cadencia(points, expectedIntervalMs));

  const scaleX = (time: number) => GUTTER + ((time - t0) / span) * plotW;
  const scaleY = (v: number) => height - ((v - rango.min) / (rango.max - rango.min || 1)) * height;
  const scaleYClamped = (v: number) => Math.max(0, Math.min(height, scaleY(v)));

  const warnZoneY = $derived(warnThreshold !== null ? scaleYClamped(warnThreshold) : null);
  const critZoneY = $derived(critThreshold !== null ? scaleYClamped(critThreshold) : null);

  /** Cuatro marcas del eje Y, repartidas por el rango real. */
  const yTicks = $derived(
    [0, 1, 2, 3].map((i) => {
      const v = rango.min + ((rango.max - rango.min) * (3 - i)) / 3;
      return { v, y: scaleY(v) };
    })
  );

  const runs = $derived(
    tramos(points, step).map((run) => run.map((p) => ({ x: scaleX(p.t), y: scaleY(p.v) })))
  );

  const gaps = $derived(
    huecos(points, step, t0, t1).map((g) => ({
      x: scaleX(g.from),
      w: scaleX(g.to) - scaleX(g.from),
      from: g.from,
      to: g.to
    }))
  );

  /* ---- Cursor de lectura: ratón y teclado (constitución §VII exige teclado en toda interacción) -- */
  let cursor = $state<number | null>(null);
  const readable = $derived(points.map((p, i) => ({ ...p, i })).filter((p) => p.v !== null));
  const hovered = $derived(cursor === null ? null : (points[cursor] ?? null));

  function nearestIndex(clientX: number, el: SVGSVGElement): number | null {
    if (!readable.length) return null;
    const rect = el.getBoundingClientRect();
    const time = t0 + ((clientX - rect.left - GUTTER) / (rect.width - GUTTER)) * span;
    return masCercano(readable, time)?.i ?? null;
  }

  function moveCursor(delta: number) {
    if (!readable.length) return;
    const pos = cursor === null ? 0 : readable.findIndex((p) => p.i === cursor);
    const next = Math.min(readable.length - 1, Math.max(0, (pos < 0 ? 0 : pos) + delta));
    cursor = readable[next].i;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowRight") {
      moveCursor(1);
      e.preventDefault();
    } else if (e.key === "ArrowLeft") {
      moveCursor(-1);
      e.preventDefault();
    } else if (e.key === "Home") {
      cursor = readable[0]?.i ?? null;
      e.preventDefault();
    } else if (e.key === "End") {
      cursor = readable.at(-1)?.i ?? null;
      e.preventDefault();
    } else if (e.key === "Escape") cursor = null;
  }

  /** Lectura textual equivalente exigida por la constitución §VII: rango, extremos y huecos. */
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

  /** Etiqueta accesible: la lectura textual equivalente, precedida del título cuando lo hay. */
  const etiqueta = $derived(titulo ? t("chart.titledSummary", { title: titulo, body: summary }) : summary);

  const hayMuestras = $derived(readable.length > 0);
  const gid = `tsc-${crypto.randomUUID()}`;

  /** Texto del globo (`ChartTip`) y de la región viva: valor + instante del punto señalado. */
  const lectura = $derived(
    hovered && hovered.v !== null
      ? t("chart.readout", {
          value: formatNumber(hovered.v),
          unit,
          when: formatDateTime(new Date(hovered.t).toISOString())
        })
      : ""
  );
</script>

<figure class="relative m-0 flex flex-col gap-3">
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- La gráfica es interactiva a propósito: cursor de lectura con ratón y teclado, como exige la
       constitución §VII. Ver docs/known-issues.md #3 -->
  <svg
    bind:clientWidth={width}
    viewBox="0 0 {width} {height}"
    class="block w-full rounded-inner"
    style="height: {height}px; color: {color}"
    role="img"
    tabindex="0"
    aria-label={etiqueta}
    onmousemove={(e) => (cursor = nearestIndex(e.clientX, e.currentTarget))}
    onmouseleave={() => (cursor = null)}
    onkeydown={onKey}
  >
    <defs>
      <linearGradient id={gid} x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="currentColor" stop-opacity="0.32" />
        <stop offset="1" stop-color="currentColor" stop-opacity="0" />
      </linearGradient>
    </defs>

    <!-- Eje Y: cuatro marcas y sus líneas de rejilla, fuera del área de trazo -->
    {#each yTicks as tick}
      <line x1={GUTTER} y1={tick.y} x2={width} y2={tick.y} stroke="var(--sdm-hairline)" stroke-width="1" />
      <text
        x={GUTTER - 6}
        y={Math.max(9, Math.min(height - 2, tick.y + 3))}
        text-anchor="end"
        class="sdm-num text-2xs"
        style="fill: var(--sdm-text-faint)"
      >
        {formatNumber(tick.v)}
      </text>
    {/each}

    {#if critZoneY !== null}
      <rect x={GUTTER} y="0" width={width - GUTTER} height={critZoneY} fill="var(--sdm-crit-soft)" />
    {/if}
    {#if warnZoneY !== null}
      <rect
        x={GUTTER}
        y={critZoneY ?? 0}
        width={width - GUTTER}
        height={Math.max(0, warnZoneY - (critZoneY ?? 0))}
        fill="var(--sdm-warn-soft)"
      />
    {/if}

    {#each gaps as gap}
      <rect x={gap.x} y="0" width={gap.w} {height} fill="var(--sdm-unknown)" opacity="0.14" />
    {/each}

    {#if warnThreshold !== null}
      <line
        x1={GUTTER}
        y1={scaleYClamped(warnThreshold)}
        x2={width}
        y2={scaleYClamped(warnThreshold)}
        stroke="var(--sdm-warn)"
        stroke-width="1.5"
        stroke-dasharray="6 6"
      />
    {/if}
    {#if critThreshold !== null}
      <line
        x1={GUTTER}
        y1={scaleYClamped(critThreshold)}
        x2={width}
        y2={scaleYClamped(critThreshold)}
        stroke="var(--sdm-crit)"
        stroke-width="1.5"
        stroke-dasharray="6 6"
      />
    {/if}

    {#each runs as run}
      {#if run.length > 1}
        <path d={areaSuave(run, height)} fill="url(#{gid})" />
        <path
          d={rutaSuave(run)}
          fill="none"
          stroke="currentColor"
          stroke-width="2.6"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
      {:else if run.length === 1}
        <circle cx={run[0].x} cy={run[0].y} r="2.6" fill="currentColor" />
      {/if}
    {/each}

    {#if !hayMuestras}
      <text
        x={GUTTER + (width - GUTTER) / 2}
        y={height / 2}
        text-anchor="middle"
        class="text-xs"
        style="fill: var(--sdm-text-dim)"
      >
        {textoVacio || t("chart.noSamples")}
      </text>
    {/if}

    {#if hovered && hovered.v !== null}
      <line
        x1={scaleX(hovered.t)}
        y1="0"
        x2={scaleX(hovered.t)}
        y2={height}
        stroke="currentColor"
        stroke-width="1"
        opacity="0.45"
      />
      <circle
        cx={scaleX(hovered.t)}
        cy={scaleY(hovered.v)}
        r="4"
        fill="currentColor"
        stroke="var(--sdm-solid)"
        stroke-width="2"
      />
    {/if}
  </svg>

  {#if hovered && hovered.v !== null}
    <ChartTip x={scaleX(hovered.t)} y={scaleY(hovered.v)} anchoContenedor={width} texto={lectura} />
  {/if}
  <!-- El valor lo lleva el globo (visual); esto lo anuncia al recorrer la serie con el teclado. -->
  <span class="sr-only" aria-live="polite">{lectura}</span>

  <figcaption class="flex flex-wrap items-center justify-between gap-2 text-2xs text-fg-faint">
    <span>{formatDateTime(new Date(t0).toISOString())}</span>

    <span class="flex flex-wrap gap-3">
      {#each gaps.slice(0, 2) as gap}
        <span>
          {t("chart.gapRange", {
            from: formatTime(new Date(gap.from).toISOString()),
            to: formatTime(new Date(gap.to).toISOString())
          })}
        </span>
      {/each}
      {#if resolutionLabel}<span>{resolutionLabel}</span>{/if}
    </span>

    {#if warnLabel || critLabel}
      <span class="flex gap-3">
        {#if warnLabel}<span class="text-warn">{warnLabel}</span>{/if}
        {#if critLabel}<span class="text-crit">{critLabel}</span>{/if}
      </span>
    {/if}
    <span>{formatDateTime(new Date(t1).toISOString())}</span>
  </figcaption>
</figure>
