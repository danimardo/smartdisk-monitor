<script lang="ts">
  /** Grupo de alertas en la lista (US-030/031): severidad, título, resumen humano y contador de ocurrencias.
   *  Nunca muestres la clave de deduplicación aquí; va en el detalle. */
  import StatusPill from "./StatusPill.svelte";
  import { severityToHealth } from "$lib/design/health";
  import { formatTime } from "$lib/design/format";
  import type { AlertGroup } from "$lib/design/types";

  let { alert = null as AlertGroup | null, selected = false, onselect } = $props();

  const sevLabel: Record<string, string> = { info: "Informativa", warn: "Advertencia", crit: "Crítica" };
  const statusLabel: Record<string, string> = {
    active: "activa", acknowledged: "reconocida", resolved: "resuelta", archived: "archivada"
  };
</script>

{#if alert}
  <button
    class="flex flex-col gap-2 sdm-material rounded-inner border p-4 text-left transition-all duration-base ease-sdm
           {selected ? 'border-accent' : 'border-hairline hover:border-fg-faint'}"
    aria-current={selected}
    onclick={() => onselect?.(alert.id)}
  >
    <div class="flex items-center gap-3">
      <StatusPill state={severityToHealth[alert.severity]} label={sevLabel[alert.severity]} />
      <span class="flex-1 truncate text-sm font-semibold">{alert.title}</span>
      <span class="sdm-num text-xs font-semibold text-fg-dim">×{alert.count}</span>
    </div>
    <p class="m-0 text-xs text-fg-dim">{alert.summary}</p>
    <span class="text-2xs text-fg-faint">
      {alert.target} · {statusLabel[alert.status]} · última {formatTime(alert.lastOccurredAt)}
    </span>
  </button>
{/if}
