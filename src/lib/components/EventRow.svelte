<script lang="ts">
  /** Fila del registro de eventos de Windows (US-021). Va dentro de una `VirtualList` de miles de
   *  filas, así que la altura **no cambia** (42 px): el cuadrado de nivel de 26 px cabe dentro.
   *
   *  v3 (`03-eventos.md`): el nivel pasa de píldora de texto a cuadrado con icono en el color del
   *  token. El color nunca viaja solo — el cuadrado lleva `aria-label` con el nombre del nivel. La
   *  píldora de texto solo aparece en el panel de detalle, no en la fila. */
  import Icon from "./Icon.svelte";
  import { healthToken } from "$lib/design/health";
  import { eventLevelIcon } from "$lib/design/icons";
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
    highlighted = false,
    /** Si se pasa, la fila es un enlace a esa ruta (panel general → detalle del suceso). Sin él,
     *  es un botón que abre el panel de detalle en la propia pantalla de eventos. */
    href = undefined as string | undefined,
    onselect = undefined as (() => void) | undefined
  } = $props();

  const estadoPorNivel: Record<string, HealthState> = {
    error: "crit",
    warning: "warn",
    info: "unknown"
  };
  const etiquetaNivel = $derived(t(`events.level.${level}`));
  const tono = $derived(healthToken[estadoPorNivel[level]]);

  const clase = $derived(
    `flex w-full items-center gap-3 border-t border-hairline py-2 text-left${
      href ? " sdm-block-link" : ""
    }${highlighted ? " bg-accent-soft" : ""}`
  );
</script>

{#snippet contenido()}
  <span
    class="grid size-[26px] shrink-0 place-items-center rounded-nav"
    style="background: {tono.soft}; color: {tono.fg}"
  >
    <Icon name={eventLevelIcon[level]} size={15} label={etiquetaNivel} />
  </span>
  <span class="min-w-0 flex-1 truncate text-sm">{message}</span>
  {#if mappingConfidence !== "exact"}
    <span class="rounded-pill bg-unknown-soft px-2 py-0.5 text-2xs text-unknown"
      >{t("events.inferredMapping")}</span
    >
  {/if}
  <span class="text-xs text-fg-dim">{provider} · {eventId}</span>
  <span class="w-14 text-right text-xs text-fg-faint">{formatTime(occurredAt)}</span>
{/snippet}

{#if href}
  <a {href} class={clase}>{@render contenido()}</a>
{:else}
  <button type="button" class={clase} onclick={() => onselect?.()}>{@render contenido()}</button>
{/if}
