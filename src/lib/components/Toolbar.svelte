<script lang="ts">
  /** Barra de herramientas unificada (v3). Título y subtítulo **de la ruta** a la izquierda; estado
   *  global, frescura del dato y acción primaria a la derecha. Es material de chrome: translúcida,
   *  sin sombra propia, con filo superior.
   *
   *  v3: alto 56 px; el título usa la familia de display; la píldora de estado global es la **única**
   *  fuente de ese dato (el pie del riel es solo un icono y no puede contradecirla); fuera el botón
   *  «?» (Acerca de vive en el riel) y fuera la ranura de controles contextuales (van junto a lo que
   *  modifican, p. ej. el intervalo del detalle de disco). */
  import Button from "./Button.svelte";
  import StatusPill from "./StatusPill.svelte";
  import type { HealthState } from "$lib/design/types";

  let {
    title = "",
    subtitle = "",
    globalState = "unknown" as HealthState,
    globalLabel = "",
    freshness = "",
    /** Pinta la frescura en `text-warn` cuando el último dato válido pasa del intervalo de muestreo. */
    stale = false,
    primaryLabel = "",
    primaryLoading = false,
    onprimary = undefined as (() => void) | undefined
  } = $props();
</script>

<header class="sdm-material-chrome z-10 flex h-14 flex-none items-center gap-3 border-b border-hairline px-5">
  <div class="flex min-w-0 flex-col">
    <span class="sdm-display truncate text-xl">{title}</span>
    {#if subtitle}<span class="truncate text-2xs text-fg-faint">{subtitle}</span>{/if}
  </div>
  <div class="flex-1"></div>
  <StatusPill state={globalState} label={globalLabel} icon="auto" />
  {#if freshness}
    <span class="text-2xs {stale ? 'text-warn' : 'text-fg-faint'}">{freshness}</span>
  {/if}
  {#if primaryLabel}
    <Button variant="primary" loading={primaryLoading} onclick={onprimary}>{primaryLabel}</Button>
  {/if}
</header>
