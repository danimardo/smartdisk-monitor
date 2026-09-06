<script lang="ts">
  /** Confirmación obligatoria antes de cualquier operación que escriba datos o genere carga (spec §12).
   *  Diálogo de material sobre velo desenfocado; declara qué hará, dónde, el impacto y el comando literal. */
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import { t } from "$lib/i18n";

  let {
    open = false,
    title = "",
    body = "",
    command = "",
    impact = "",
    confirmLabel = "",
    destructive = false,
    onconfirm = undefined,
    oncancel = undefined
  } = $props();

  let panel = $state<HTMLDivElement | undefined>();

  /** El foco tiene que entrar al abrir: sin esto, `Escape` (atado al velo) nunca llega a
   *  disparar, porque el foco se queda en el botón que abrió el diálogo, fuera del árbol del
   *  velo, y un `keydown` no baja desde ahí (`docs/known-issues.md` #2, corregido). */
  $effect(() => {
    if (open) panel?.focus();
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- El velo solo captura el clic fuera del diálogo; el cierre por teclado lo cubre el `onkeydown`
       de Escape de esta misma capa, alcanzable porque el foco entra al panel al abrir.
       Ver docs/known-issues.md #2 -->
  <div
    class="fixed inset-0 z-50 grid place-items-center p-8 backdrop-blur-[3px]"
    style="background: var(--sdm-scrim)"
    role="presentation"
    onclick={oncancel}
    onkeydown={(e) => e.key === "Escape" && oncancel?.()}
  >
    <div
      bind:this={panel}
      class="sdm-material-overlay flex w-full max-w-lg flex-col gap-4 rounded-window p-6"
      style="animation: sdm-dialog var(--sdm-duration-overlay) var(--sdm-ease)"
      role="dialog"
      aria-modal="true"
      aria-label={title}
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
          <span class="shrink-0 text-warn"><Icon name="alert" size={16} /></span>
          <span class="text-xs leading-normal" style="text-wrap: pretty">{impact}</span>
        </div>
      {/if}

      <div class="flex justify-end gap-2">
        <Button variant="ghost" onclick={oncancel}>{t("common.cancel")}</Button>
        <Button variant={destructive ? "danger" : "primary"} onclick={onconfirm}
          >{confirmLabel || t("common.continue")}</Button
        >
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes sdm-dialog {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
</style>
