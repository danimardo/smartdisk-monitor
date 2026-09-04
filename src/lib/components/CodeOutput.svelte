<script lang="ts">
  /** Salida literal de un proceso auxiliar: JSON de smartctl (US-043) o salida de chkdsk (US-041).
   *  Se renderiza SIEMPRE como texto, nunca como HTML. Incluye copia al portapapeles y
   *  la procedencia exacta (comando lógico, versión, fecha). La vista no permite ejecutar nada. */
  import Button from "./Button.svelte";

  let {
    content = "",
    /** "smartctl 7.4 · smartctl -j -a /dev/sda · 04/09/2026 12:41:07" */
    provenance = "",
    maxHeight = 280,
    oncopy = undefined
  } = $props();

  let copied = $state(false);

  async function copy() {
    // Solo se confirma si el portapapeles aceptó de verdad: marcar "Copiado" cuando la escritura
    // falló haría creer al usuario que tiene la salida y no la tendría.
    try {
      await navigator.clipboard?.writeText(content);
    } catch {
      return;
    }
    oncopy?.();
    copied = true;
    setTimeout(() => (copied = false), 1600);
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center gap-3">
    {#if provenance}<span class="flex-1 truncate font-mono text-2xs text-fg-faint">{provenance}</span>{/if}
    <Button size="sm" onclick={copy}>{copied ? "Copiado" : "Copiar"}</Button>
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- Región desplazable: el `tabindex` es deliberado y lo exige WCAG 2.1.1 para que una salida
       larga se pueda recorrer solo con el teclado. Ver docs/known-issues.md #1 -->
  <pre
    class="m-0 overflow-auto rounded-inner bg-glass-3 p-4 font-mono text-2xs leading-relaxed text-fg-dim"
    style="max-height: {maxHeight}px"
    role="region"
    aria-label={provenance || "Salida"}
    tabindex="0">{content}</pre>
</div>
