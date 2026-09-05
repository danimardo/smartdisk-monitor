<script lang="ts">
  /** Fila del registro de eventos de Windows (US-021). El nivel es una píldora; la asociación
   *  inferida se etiqueta explícitamente — nunca se presenta como certeza. */
  import StatusPill from "./StatusPill.svelte";
  import { formatTime } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import type { HealthState } from "$lib/design/types";

  let {
    level = "info" as "error" | "warning" | "info",
    message = "",
    provider = "",
    eventId = 0,
    occurredAt = "",
    mappingConfidence = "exact" as "exact" | "inferred" | "unknown",
    onselect = undefined as (() => void) | undefined
  } = $props();

  const estadoPorNivel: Record<string, HealthState> = {
    error: "crit",
    warning: "warn",
    info: "unknown"
  };
  const etiquetaNivel = $derived(t(`events.level.${level}`));
</script>

<button
  class="flex w-full items-center gap-3 border-t border-hairline py-2 text-left"
  onclick={() => onselect?.()}
>
  <StatusPill state={estadoPorNivel[level]} label={etiquetaNivel} />
  <span class="flex-1 truncate text-sm">{message}</span>
  {#if mappingConfidence !== "exact"}
    <span class="rounded-pill bg-unknown-soft px-2 py-0.5 text-2xs font-semibold text-unknown"
      >{t("events.inferredMapping")}</span
    >
  {/if}
  <span class="text-xs text-fg-dim">{provider} · {eventId}</span>
  <span class="w-14 text-right text-xs text-fg-faint">{formatTime(occurredAt)}</span>
</button>
