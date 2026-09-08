<script lang="ts">
  /** Modal de la ayuda con IA (spec 005-explicacion-ia, US2). Presentacional: toda la orquestación
   *  (llamadas, reintentos, revisión) vive en la pantalla; este componente solo pinta la fase que
   *  se le pasa y emite intenciones (constitución §IV).
   *
   *  La respuesta del modelo es **contenido no confiable** (principio XVI): se renderiza con
   *  `<Markdown>`, que nunca usa `{@html}`. */
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";
  import Markdown from "./Markdown.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import { t } from "$lib/i18n";
  import type { AppError } from "$lib/design/types";

  type Fragmento = { texto: string; motivoKey: string };

  let {
    open = false,
    /** `progreso` | `resultado` | `error` | `vistaPrevia` | `revision` */
    fase = "progreso" as "progreso" | "resultado" | "error" | "vistaPrevia" | "revision",
    markdown = "",
    modeloUsado = "",
    detalleRecortado = false,
    error = null as AppError | null,
    textoRevision = "",
    fragmentos = [] as Fragmento[],
    oncancel = undefined as (() => void) | undefined,
    onclose = undefined as (() => void) | undefined,
    onretry = undefined as (() => void) | undefined,
    onconfirmar = undefined as (() => void) | undefined,
    onenviarigual = undefined as (() => void) | undefined,
    onquitarfragmentos = undefined as (() => void) | undefined
  } = $props();

  let panel = $state<HTMLDivElement | undefined>();
  let disparador: HTMLElement | null = null;

  $effect(() => {
    if (open) {
      disparador = document.activeElement as HTMLElement | null;
      panel?.focus();
      return () => disparador?.focus();
    }
  });

  const cerrar = () => (fase === "progreso" ? oncancel?.() : onclose?.());
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- El velo captura el clic fuera; el cierre por teclado lo cubre el `onkeydown` de Escape de
       esta capa, alcanzable porque el foco entra al panel al abrir. Ver docs/known-issues.md #2 -->
  <div
    class="fixed inset-0 z-50 grid place-items-center p-8 backdrop-blur-[3px]"
    style="background: var(--sdm-scrim)"
    role="presentation"
    onclick={cerrar}
    onkeydown={(e) => e.key === "Escape" && cerrar()}
  >
    <div
      bind:this={panel}
      class="sdm-material-overlay relative flex max-h-[80vh] w-full max-w-xl flex-col gap-4 overflow-y-auto rounded-window p-6"
      style="animation: sdm-dialog var(--sdm-duration-overlay) var(--sdm-ease)"
      role="dialog"
      aria-modal="true"
      aria-label={t("ai.modal.title")}
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center gap-3">
        <h2 class="m-0 flex-1 text-lg font-semibold tracking-tight">{t("ai.modal.title")}</h2>
        {#if fase !== "progreso"}
          <button
            type="button"
            class="grid size-[30px] place-items-center rounded-nav text-fg-dim hover:bg-glass-3 hover:text-fg"
            aria-label={t("common.close")}
            onclick={() => onclose?.()}
          >
            <Icon name="close" size={16} />
          </button>
        {/if}
      </div>

      {#if fase === "progreso"}
        <ProgressBar indeterminate caption={t("ai.modal.progress")} trailing="" />
        <div class="flex justify-end">
          <Button variant="ghost" onclick={() => oncancel?.()}>{t("common.cancel")}</Button>
        </div>
      {:else if fase === "error" && error}
        <p class="m-0 text-sm text-crit" style="text-wrap: pretty">{t(error.messageKey)}</p>
        {#if error.detail}
          <details>
            <summary class="cursor-pointer text-xs font-semibold text-fg-dim">
              {t("common.technicalDetail")}
            </summary>
            <pre
              class="mt-2 overflow-x-auto whitespace-pre-wrap rounded-inner bg-glass-3 p-3 text-xs text-fg-dim">{error.detail}</pre>
          </details>
        {/if}
        <div class="flex justify-end gap-2">
          <Button variant="ghost" onclick={() => onclose?.()}>{t("common.close")}</Button>
          {#if error.retryable}
            <Button variant="primary" onclick={() => onretry?.()}>{t("ai.modal.retry")}</Button>
          {/if}
        </div>
      {:else if fase === "resultado"}
        <Markdown source={markdown} />
        {#if detalleRecortado}
          <p class="m-0 text-2xs text-fg-faint" style="text-wrap: pretty">
            {t("ai.modal.truncated")}
          </p>
        {/if}
        <div class="flex flex-col gap-2 border-t border-hairline pt-3">
          <p class="m-0 text-2xs text-fg-faint">{t("ai.modal.model", { model: modeloUsado })}</p>
          <p class="m-0 text-2xs text-fg-faint" style="text-wrap: pretty">
            {t("ai.modal.disclaimer")}
          </p>
        </div>
      {:else}
        <!-- vistaPrevia | revision -->
        <p class="m-0 text-sm text-fg-dim" style="text-wrap: pretty">
          {fase === "revision" ? t("ai.review.body") : t("ai.preview.body")}
        </p>
        {#if fase === "revision" && fragmentos.length}
          <div class="flex flex-col gap-1 rounded-inner bg-warn-soft p-3">
            <p class="m-0 text-xs font-semibold text-warn">{t("ai.review.flaggedTitle")}</p>
            <ul class="m-0 flex list-disc flex-col gap-0.5 pl-5 text-xs text-fg-dim">
              {#each fragmentos as f}
                <li><code class="font-mono">{f.texto}</code> — {t(f.motivoKey)}</li>
              {/each}
            </ul>
          </div>
        {/if}
        <pre
          class="m-0 max-h-64 overflow-y-auto whitespace-pre-wrap rounded-inner bg-glass-3 p-3 text-xs text-fg-dim">{textoRevision}</pre>
        <div class="flex flex-wrap justify-end gap-2">
          <Button variant="ghost" onclick={() => oncancel?.()}>{t("common.cancel")}</Button>
          {#if fase === "revision" && fragmentos.length}
            <Button variant="secondary" onclick={() => onquitarfragmentos?.()}>
              {t("ai.review.stripFragments")}
            </Button>
            <Button variant="primary" onclick={() => onenviarigual?.()}>
              {t("ai.review.sendAnyway")}
            </Button>
          {:else}
            <Button variant="primary" onclick={() => onconfirmar?.()}>
              {t("ai.preview.confirm")}
            </Button>
          {/if}
        </div>
      {/if}
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
