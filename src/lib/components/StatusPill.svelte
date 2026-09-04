<script lang="ts">
  /** Píldora de estado sobre material: fondo translúcido del token + texto del token.
   *  Es el único indicador textual de salud; el color nunca viaja solo. */
  import { healthToken } from "$lib/design/health";
  import type { HealthState } from "$lib/design/types";

  let { state = "ok" as HealthState, label = "", withDot = false } = $props();
  const tone = $derived(healthToken[state]);
</script>

<span
  class="inline-flex items-center gap-2 rounded-pill px-3 py-1 text-2xs font-semibold whitespace-nowrap"
  style="background: {tone.soft}; color: {tone.fg};"
>
  {#if withDot}<span class="size-1.5 rounded-pill" style="background: {tone.fg}"></span>{/if}
  {label}
</span>
