<script lang="ts">
  /** Grupo de alertas en la lista (US-030/031): severidad, título, resumen humano y contador de ocurrencias.
   *  Nunca muestres la clave de deduplicación aquí; va en el detalle. */
  import StatusPill from "./StatusPill.svelte";
  import { severityToHealth } from "$lib/design/health";
  import { severityIcon } from "$lib/design/icons";
  import { formatTime } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import type { AlertGroup } from "$lib/design/types";

  let {
    alert = null as AlertGroup | null,
    selected = false,
    onselect = undefined as ((id: string) => void) | undefined
  } = $props();

  /** El backend no manda texto de interfaz (ADR-030): título, resumen, severidad y estado se
   *  resuelven aquí a partir de `ruleKey`/`severity`/`status`, que sí son datos estables. */
  const titulo = $derived(alert ? t(`alert.rule.${alert.ruleKey}.title`) : "");
  const resumen = $derived(alert ? t(`alert.rule.${alert.ruleKey}.summary`) : "");
  const etiquetaSeveridad = $derived(alert ? t(`health.${severityToHealth[alert.severity]}`) : "");
  const etiquetaEstado = $derived(alert ? t(`alert.status.${alert.status}`) : "");
</script>

{#if alert}
  <button
    class="flex flex-col gap-2 sdm-material rounded-inner border p-4 text-left transition-all duration-base ease-sdm
           {selected ? 'border-accent' : 'border-hairline hover:border-fg-faint'}"
    aria-current={selected}
    onclick={() => onselect?.(alert.id)}
  >
    <div class="flex items-center gap-3">
      <StatusPill
        state={severityToHealth[alert.severity]}
        label={etiquetaSeveridad}
        icon={severityIcon[alert.severity]}
      />
      <span class="flex-1 truncate text-sm font-semibold">{titulo}</span>
      <span class="sdm-num text-xs font-semibold text-fg-dim">×{alert.count}</span>
    </div>
    <p class="m-0 text-xs text-fg-dim">{resumen}</p>
    <span class="text-2xs text-fg-faint">
      {alert.target} · {etiquetaEstado} · última {formatTime(alert.lastOccurredAt)}
    </span>
  </button>
{/if}
