<script lang="ts">
  /** Punto de estado de 9 px, con etiqueta accesible obligatoria (el color no es el único portador). */
  import { healthToken } from "$lib/design/health";
  import type { HealthState } from "$lib/design/types";

  let { state = "ok" as HealthState, label = "", size = 9 } = $props();
</script>

<span class="inline-flex items-center gap-2">
  <span
    class="rounded-pill shrink-0"
    style="width: {size}px; height: {size}px; background: {healthToken[state].fg};"
    role="img"
    aria-label={label}
  ></span>
  {#if label}<span class="text-xs text-fg-dim">{label}</span>{/if}
</span>
