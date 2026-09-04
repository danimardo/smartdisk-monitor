<script lang="ts">
  /** Fila etiqueta / valor / delta para contadores SMART y listas de hechos.
   *  El delta usa color solo cuando significa algo: +N en errores es warn, 0 es neutro. */
  import { healthToken } from "$lib/design/health";
  import type { HealthState } from "$lib/design/types";

  let { label = "", value = "", delta = "", deltaState = null as HealthState | null } = $props();
</script>

<div class="flex items-center gap-3 border-b border-hairline pb-2">
  <span class="flex-1 text-xs text-fg-dim">{label}</span>
  <span class="sdm-num text-sm font-semibold">{value}</span>
  {#if delta}
    <span class="sdm-num w-16 text-right text-2xs font-semibold" style="color: {deltaState ? healthToken[deltaState].fg : 'var(--sdm-text-dim)'}">{delta}</span>
  {/if}
</div>
