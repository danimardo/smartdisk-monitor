<script lang="ts">
  /** Salida literal de un proceso auxiliar: JSON de smartctl (US-043), salida de chkdsk (US-041)
   *  o el XML de un suceso de Windows. Se renderiza SIEMPRE como texto, nunca como HTML — con
   *  `lang="xml"` se reindenta y se colorea por fragmento (`$lib/design/xml`), pero cada fragmento
   *  sigue siendo una interpolación de texto normal de Svelte dentro de un `<span>`, jamás
   *  `{@html}`. Incluye copia al portapapeles (del texto tal cual se ve, ya reindentado si aplica)
   *  y la procedencia exacta (comando lógico, versión, fecha). La vista no permite ejecutar nada. */
  import Button from "./Button.svelte";
  import { formatXml, tokenizeXml, type TipoTokenXml } from "$lib/design/xml";

  let {
    content = "",
    /** "smartctl 7.4 · smartctl -j -a /dev/sda · 04/09/2026 12:41:07" */
    provenance = "",
    maxHeight = 280,
    /** "xml" reindenta y colorea; por defecto el contenido se muestra tal cual llega (JSON, texto
     *  plano de chkdsk). */
    lang = "text" as "text" | "xml",
    oncopy = undefined
  } = $props();

  const mostrado = $derived(lang === "xml" ? formatXml(content) : content);
  const tokens = $derived(lang === "xml" ? tokenizeXml(mostrado) : null);

  /** Reutiliza únicamente tonos ya existentes del sistema de diseño (`tokens.css`): ninguno nuevo,
   *  y ninguno de los tokens de salud (ok/warn/crit/unknown) — esto es resaltado de sintaxis, no
   *  un estado del disco. */
  const CLASE_TOKEN: Record<TipoTokenXml, string> = {
    tagName: "text-accent-fg",
    attrName: "text-fg-dim",
    attrValue: "text-fg",
    punct: "text-fg-faint",
    comment: "italic text-fg-faint",
    text: "text-fg"
  };

  let copied = $state(false);

  async function copy() {
    // Solo se confirma si el portapapeles aceptó de verdad: marcar "Copiado" cuando la escritura
    // falló haría creer al usuario que tiene la salida y no la tendría.
    try {
      await navigator.clipboard?.writeText(mostrado);
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
  <!-- Sin espacio entre las etiquetas y las interpolaciones: dentro de un `<pre>` Svelte conserva
       literalmente cualquier espacio en blanco del propio código fuente de la plantilla, y
       añadiría una línea en blanco al principio de la salida. -->
  <pre
    class="m-0 overflow-auto rounded-inner bg-glass-3 p-4 font-mono text-2xs leading-relaxed text-fg-dim"
    style="max-height: {maxHeight}px"
    role="region"
    aria-label={provenance || "Salida"}
    tabindex="0">{#if tokens}{#each tokens as tok, i (i)}<span class={CLASE_TOKEN[tok.tipo]}>{tok.texto}</span
        >{/each}{:else}{mostrado}{/if}</pre>
</div>
