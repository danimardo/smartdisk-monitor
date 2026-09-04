<script lang="ts">
  /** Barra de ocupación de un volumen. El color lo decide capacityState(), no el llamante. */
  import { capacityState } from "$lib/design/health";
  import { healthToken } from "$lib/design/health";
  import { formatBytes, usedPercent, NOT_AVAILABLE } from "$lib/design/format";

  let { label = "", capacityBytes = null as number | null, freeBytes = null as number | null } = $props();

  const pct = $derived(usedPercent(capacityBytes, freeBytes));
  const tone = $derived(healthToken[capacityState(freeBytes, capacityBytes)]);
</script>

<div class="flex flex-col gap-2">
  <div class="flex justify-between text-xs text-fg-dim">
    <span>{label}</span>
    <span>{freeBytes === null ? NOT_AVAILABLE() : formatBytes(freeBytes) + " libres"}</span>
  </div>
  <div
    class="h-2 overflow-hidden rounded-pill bg-glass-3"
    role="progressbar"
    aria-valuenow={pct ?? undefined}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-label={label}
  >
    {#if pct !== null}
      <div class="h-full rounded-pill transition-all duration-base ease-sdm" style="width: {pct}%; background: {tone.fg}"></div>
    {/if}
  </div>
</div>
