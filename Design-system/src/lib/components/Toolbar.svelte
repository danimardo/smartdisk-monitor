<script lang="ts">
  /** Barra de herramientas unificada (v2): título y subtítulo de la pantalla a la izquierda,
   *  controles contextuales, estado global, frescura del dato y acción primaria a la derecha.
   *  Es material de chrome: translúcida, sin sombra propia, con filo superior. */
  import Button from "./Button.svelte";
  import StatusPill from "./StatusPill.svelte";
  import type { HealthState } from "$lib/design/types";

  let {
    title = "",
    subtitle = "",
    globalState = "ok" as HealthState,
    globalLabel = "",
    freshness = "",
    primaryLabel = "Actualizar",
    onprimary = undefined,
    onabout = undefined,
    /** Controles propios de la pantalla: SegmentedControl de intervalo, filtros, etc. */
    controls = undefined
  } = $props();
</script>

<header class="sdm-material-chrome z-10 flex h-13 flex-none items-center gap-3 border-b border-hairline px-5" style="height: 52px">
  <div class="flex min-w-0 flex-col">
    <span class="truncate text-lg font-semibold tracking-tight">{title}</span>
    {#if subtitle}<span class="truncate text-2xs text-fg-faint">{subtitle}</span>{/if}
  </div>
  <div class="flex-1"></div>
  {@render controls?.()}
  <StatusPill state={globalState} label={globalLabel} />
  {#if freshness}<span class="text-2xs text-fg-faint">{freshness}</span>{/if}
  <Button variant="primary" onclick={onprimary}>{primaryLabel}</Button>
  <button
    class="grid size-control-md place-items-center rounded-pill border border-hairline bg-glass-2 text-xs font-semibold text-fg-dim shadow-edge"
    aria-label="Acerca de"
    onclick={onabout}
  >?</button>
</header>
