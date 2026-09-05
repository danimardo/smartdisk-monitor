<script lang="ts">
  /** Estado vacío o no disponible. Diferencia "sin datos todavía", "no compatible" y "error de fuente":
   *  un dispositivo no compatible NO es un fallo (spec §12). */
  import Button from "./Button.svelte";
  import { t } from "$lib/i18n";

  let {
    kind = "empty" as "empty" | "unsupported" | "error",
    title = "",
    body = "",
    /** Detalle técnico conservado, colapsado por defecto. */
    detail = "",
    actionLabel = "",
    onaction = undefined
  } = $props();

  const tint: Record<string, string> = {
    empty: "var(--sdm-unknown-soft)",
    unsupported: "var(--sdm-unknown-soft)",
    error: "var(--sdm-crit-soft)"
  };
  const fg: Record<string, string> = {
    empty: "var(--sdm-unknown)",
    unsupported: "var(--sdm-unknown)",
    error: "var(--sdm-crit)"
  };
</script>

<div class="flex flex-col items-start gap-3 sdm-material rounded-card border-dashed p-6">
  <span
    class="rounded-pill px-3 py-1 text-2xs font-semibold"
    style="background: {tint[kind]}; color: {fg[kind]}"
  >
    {kind === "unsupported"
      ? t("common.unsupported")
      : kind === "error"
        ? t("common.sourceError")
        : t("common.noData")}
  </span>
  <h3 class="m-0 text-base font-semibold">{title}</h3>
  <p class="m-0 max-w-prose text-sm text-fg-dim" style="text-wrap: pretty">{body}</p>
  {#if detail}
    <details class="w-full">
      <summary class="cursor-pointer text-xs font-semibold text-fg-dim">{t("common.technicalDetail")}</summary
      >
      <pre
        class="m-0 mt-2 overflow-auto rounded-inner bg-glass-2 p-3 font-mono text-2xs text-fg-dim">{detail}</pre>
    </details>
  {/if}
  {#if actionLabel}<Button size="sm" onclick={onaction}>{actionLabel}</Button>{/if}
</div>
