<script lang="ts">
  /** Cifra destacada + icono + evolución + procedencia (v3, `MetricCard.md`).
   *
   *  Dos reglas que no son estéticas:
   *   1. **El peso 800 desaparece** (`font-black` → `.sdm-display`, que es 600). El sistema declara
   *      600 como máximo y esta era la única pieza que se lo saltaba.
   *   2. **Un valor que no es una cifra no se compone como cifra.** `value === null` ⇒ «No disponible»
   *      a `text-lg` en gris tenue, sin sparkline, con la procedencia explicando por qué falta.
   *   3. **Un bloque interno no es una tarjeta.** Va sobre `bg-glass-3` con `rounded-inner`. */
  import Icon from "./Icon.svelte";
  import Sparkline from "./Sparkline.svelte";
  import Tooltip from "./Tooltip.svelte";
  import { NOT_AVAILABLE } from "$lib/design/format";
  import { healthToken } from "$lib/design/health";
  import type { AyudaMetrica } from "$lib/design/metricHelp";
  import type { IconName } from "$lib/design/icons";
  import type { HealthState } from "$lib/design/types";
  import type { Punto } from "$lib/design/series";

  let {
    label = "",
    value = null as string | null,
    icon = "pulse" as IconName,
    state = null as HealthState | null,
    series = [] as Punto[],
    /** Unidad de la serie (p. ej. "°C", "%"): el globo de lectura de la sparkline la muestra. */
    unidad = "",
    /** Ayuda contextual (qué es la métrica + veredicto del valor). Si se pasa, la cabecera —icono y
     *  etiqueta— es un objetivo enfocable con un `Tooltip`. */
    ayuda = undefined as AyudaMetrica | undefined,
    provenance = "",
    /** Marca de dato obsoleto, p. ej. "hace 12 min". Se muestra junto a la procedencia. */
    age = null as string | null,
    /** Cifra secundaria bajo la principal, p. ej. "Pico 80 %" en la actividad (spec 007). */
    secondary = null as string | null
  } = $props();

  const missing = $derived(value === null || value === "");
  const tone = $derived(
    state ? healthToken[state] : { fg: "var(--sdm-accent-fg)", soft: "var(--sdm-accent-soft)" }
  );
  const cifraColor = $derived(
    missing ? "var(--sdm-text-faint)" : state ? healthToken[state].fg : "var(--sdm-text)"
  );
  const hayCurva = $derived(!missing && series.some((p) => p.v !== null));
</script>

<div class="flex min-w-0 flex-col gap-2 rounded-inner bg-glass-3 p-4">
  {#snippet cabecera()}
    <span
      class="grid size-7 shrink-0 place-items-center rounded-nav"
      style="background: {tone.soft}; color: {tone.fg}"
    >
      <Icon name={icon} size={15} />
    </span>
    <span class="truncate text-2xs text-fg-faint">{label}</span>
  {/snippet}

  {#if ayuda}
    <Tooltip
      titulo={ayuda.titulo}
      text={ayuda.texto}
      acento={ayuda.estado}
      disparador="flex w-full items-center gap-2 rounded-nav text-left"
    >
      {@render cabecera()}
    </Tooltip>
  {:else}
    <div class="flex items-center gap-2">{@render cabecera()}</div>
  {/if}

  {#if missing}
    <span class="text-lg font-medium" style="color: {cifraColor}">{NOT_AVAILABLE()}</span>
  {:else}
    <span class="sdm-num sdm-display truncate text-metric" style="color: {cifraColor}" title={value}>
      {value}
    </span>
    {#if secondary}
      <span class="sdm-num truncate text-2xs text-fg-dim">{secondary}</span>
    {/if}
    {#if hayCurva}
      <Sparkline
        points={series}
        color={state ? healthToken[state].fg : "var(--sdm-accent)"}
        height={22}
        interactivo
        {unidad}
      />
    {/if}
  {/if}

  {#if provenance || age}
    <span class="truncate text-2xs text-fg-faint">
      {provenance}{provenance && age ? " · " : ""}{age ?? ""}
    </span>
  {/if}
</div>
