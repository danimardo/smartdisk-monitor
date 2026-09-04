<script lang="ts">
  /** Confirmación obligatoria antes de cualquier operación que escriba datos o genere carga (spec §12).
   *  Diálogo de material sobre velo desenfocado; declara qué hará, dónde, el impacto y el comando literal. */
  import Button from "./Button.svelte";

  let {
    open = false,
    title = "",
    body = "",
    command = "",
    impact = "",
    confirmLabel = "Continuar",
    destructive = false,
    onconfirm = undefined,
    oncancel = undefined} = $props();
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- El velo solo captura el clic fuera del diálogo; el cierre por teclado lo cubre el `onkeydown`
       de Escape de esta misma capa. Ver docs/known-issues.md #2 -->
  <div
    class="fixed inset-0 z-50 grid place-items-center p-8 backdrop-blur-[3px]"
    style="background: var(--sdm-scrim)"
    role="presentation"
    onclick={oncancel}
    onkeydown={(e) => e.key === "Escape" && oncancel?.()}
  >
    <div
      class="sdm-material-overlay flex w-full max-w-lg flex-col gap-4 rounded-window p-6"
      style="animation: sdm-dialog var(--sdm-duration-overlay) var(--sdm-ease)"
      role="dialog" aria-modal="true" aria-label={title}
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 class="m-0 text-xl font-semibold tracking-tight">{title}</h2>
      <p class="m-0 text-sm leading-relaxed text-fg-dim" style="text-wrap: pretty">{body}</p>

      {#if command}
        <code class="rounded-inner bg-glass-3 px-4 py-3 font-mono text-xs text-fg">{command}</code>
      {/if}

      {#if impact}
        <div class="flex gap-3 rounded-inner bg-warn-soft p-4">
          <span class="grid size-[18px] shrink-0 place-items-center rounded-pill bg-warn text-2xs font-semibold text-white">!</span>
          <span class="text-xs leading-normal" style="text-wrap: pretty">{impact}</span>
        </div>
      {/if}

      <div class="flex justify-end gap-2">
        <Button variant="ghost" onclick={oncancel}>Cancelar</Button>
        <Button variant={destructive ? "danger" : "primary"} onclick={onconfirm}>{confirmLabel}</Button>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes sdm-dialog {
    from { opacity: 0; transform: scale(0.96) translateY(8px); }
    to { opacity: 1; transform: none; }
  }
</style>
