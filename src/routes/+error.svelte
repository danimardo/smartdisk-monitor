<script lang="ts">
  /** Página de error de ruta. Un `load` que falla llega aquí, y se presenta con la forma que exige
   *  el principio X: frase humana visible y detalle técnico conservado y copiable. */
  import { page } from "$app/state";
  import { t } from "$lib/i18n";
  import type { AppError } from "$lib/design/types";

  /** SvelteKit envuelve el error; si es un `AppError` nuestro, conserva sus campos. */
  const appError = $derived(page.error as unknown as Partial<AppError> | null);
  const humano = $derived(
    appError?.messageKey ? t(appError.messageKey, appError.messageVars) : t("error.unexpected")
  );
</script>

<div class="flex flex-1 items-center justify-center">
  <div class="sdm-material max-w-lg rounded-card border border-hairline p-6 shadow-card">
    <h1 class="mb-2 text-xl font-semibold">{t("error.screenFailed")}</h1>
    <p class="mb-4 text-sm text-fg-dim">{humano}</p>
    {#if appError?.detail}
      <details class="text-2xs text-fg-faint">
        <summary class="cursor-pointer">{t("common.technicalDetail")}</summary>
        <pre class="mt-2 max-h-48 overflow-auto whitespace-pre-wrap">{appError.detail}</pre>
      </details>
    {/if}
  </div>
</div>
