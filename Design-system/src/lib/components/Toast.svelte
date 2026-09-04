<script lang="ts">
  /** Aviso efímero dentro de la app (prueba terminada, exportación guardada, fuente recuperada).
   *  NO sustituye al centro de alertas ni a la notificación nativa de Windows: es solo confirmación
   *  de una acción del usuario. Nunca lo uses para comunicar una alerta de salud. */
  import { healthToken } from "$lib/design/health";
  import type { HealthState } from "$lib/design/types";

  let { open = false, state = "ok" as HealthState, message = "", action = "", onaction, onclose } = $props();
  const tone = $derived(healthToken[state]);
</script>

{#if open}
  <div
    class="sdm-material-overlay pointer-events-auto fixed bottom-6 left-1/2 z-40 flex max-w-md -translate-x-1/2 items-center gap-3 rounded-pill py-2 pl-4 pr-2"
    style="animation: sdm-toast var(--sdm-duration-overlay) var(--sdm-ease)"
    role="status"
    aria-live="polite"
  >
    <span class="size-2 shrink-0 rounded-pill" style="background: {tone.fg}"></span>
    <span class="text-xs font-medium">{message}</span>
    {#if action}
      <button class="rounded-pill px-3 py-1 text-xs font-semibold text-accent hover:bg-accent-soft" onclick={onaction}>{action}</button>
    {/if}
    <button class="grid size-6 shrink-0 place-items-center rounded-pill text-fg-faint hover:bg-glass-3" aria-label="Cerrar" onclick={onclose}>×</button>
  </div>
{/if}

<style>
  @keyframes sdm-toast {
    from { opacity: 0; transform: translate(-50%, 12px) scale(0.97); }
    to { opacity: 1; transform: translate(-50%, 0) scale(1); }
  }
</style>
