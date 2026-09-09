<script lang="ts">
  /** Diálogo «Acerca de» (US-061), abierto desde el riel. Presentacional: la carga de `get_app_info`
   *  y el copiado al portapapeles viven en `+layout.svelte`; este componente solo pinta.
   *
   *  Enriquecido respecto al `ConfirmDialog` genérico que usaba antes: además de versión, licencia y
   *  terceros, presenta al autor con su foto y una biografía breve (ADR-052). Mismo patrón de modal
   *  informativo que `ExplicacionModal`: `role="dialog"`, foco al panel al abrir y de vuelta al
   *  disparador al cerrar, `Escape` y clic fuera cierran. */
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import { t } from "$lib/i18n";
  import type { AppError } from "$lib/design/types";
  import fotoAutor from "$lib/assets/daniel-mardomingo.webp";

  let {
    open = false,
    appInfo = null as { name: string; version: string; author: string } | null,
    error = null as AppError | null,
    oncopy = undefined as (() => void) | undefined,
    onclose = undefined as (() => void) | undefined
  } = $props();

  let panel = $state<HTMLDivElement | undefined>();
  let disparador: HTMLElement | null = null;

  /** El foco entra al panel al abrir (si no, `Escape` —atado al velo— nunca burbujea desde el botón
   *  que abrió el diálogo) y vuelve al disparador al cerrar. Ver docs/known-issues.md #2. */
  $effect(() => {
    if (open) {
      disparador = document.activeElement as HTMLElement | null;
      panel?.focus();
      return () => disparador?.focus();
    }
  });

  const titulo = $derived(appInfo ? `${appInfo.name} ${appInfo.version}` : t("about.title"));
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- El velo captura el clic fuera; el cierre por teclado lo cubre el `onkeydown` de Escape de esta
       capa, alcanzable porque el foco entra al panel al abrir. Ver docs/known-issues.md #2 -->
  <div
    class="fixed inset-0 z-50 grid place-items-center p-8 backdrop-blur-[3px]"
    style="background: var(--sdm-scrim)"
    role="presentation"
    onclick={() => onclose?.()}
    onkeydown={(e) => e.key === "Escape" && onclose?.()}
  >
    <div
      bind:this={panel}
      class="sdm-material-overlay relative flex max-h-[80vh] w-full max-w-xl flex-col gap-4 overflow-y-auto rounded-window p-6"
      style="animation: sdm-dialog var(--sdm-duration-overlay) var(--sdm-ease)"
      role="dialog"
      aria-modal="true"
      aria-label={titulo}
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <button
        type="button"
        class="absolute right-4 top-4 grid size-[30px] place-items-center rounded-nav text-fg-dim hover:bg-glass-3 hover:text-fg"
        aria-label={t("common.close")}
        onclick={() => onclose?.()}
      >
        <Icon name="close" size={16} />
      </button>

      <h2 class="m-0 pr-8 text-xl font-semibold tracking-tight">{titulo}</h2>

      <div class="flex items-center gap-4">
        <img
          src={fotoAutor}
          alt={t("about.photoAlt")}
          width="88"
          height="88"
          class="size-[88px] shrink-0 rounded-inner object-cover"
        />
        <div class="flex min-w-0 flex-col">
          <span class="sdm-display text-lg">{t("about.authorName")}</span>
          <span class="text-2xs text-fg-faint">{t("about.authorRole")}</span>
        </div>
      </div>

      <p class="m-0 text-sm leading-relaxed text-fg-dim" style="text-wrap: pretty">
        {t("about.bio.p1")}
      </p>
      <p class="m-0 text-sm leading-relaxed text-fg-dim" style="text-wrap: pretty">
        {t("about.bio.p2")}
      </p>

      <div class="flex flex-col gap-1 border-t border-hairline pt-4 text-2xs text-fg-faint">
        {#if appInfo}
          <span style="text-wrap: pretty">{t("about.credits", { author: appInfo.author })}</span>
          <span class="break-words">{t("about.links")}</span>
        {:else if error}
          <span>{t(error.messageKey, error.messageVars)}</span>
        {:else}
          <span>{t("common.loading")}</span>
        {/if}
      </div>

      <div class="flex justify-end gap-2">
        <Button variant="ghost" onclick={() => onclose?.()}>{t("common.close")}</Button>
        <Button variant="primary" disabled={!appInfo} onclick={() => oncopy?.()}>
          {t("about.cta.copy")}
        </Button>
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
