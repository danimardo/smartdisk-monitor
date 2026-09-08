<script lang="ts">
  /** Render de un subconjunto de Markdown como **marcado Svelte**, nunca `{@html}` (principio XVI:
   *  la respuesta de un LLM es contenido no confiable). El análisis vive en `$lib/design/markdown`;
   *  aquí solo se pinta el árbol. Los enlaces se muestran como texto + URL entre paréntesis, sin
   *  `href`: la aplicación no navega a nada que diga un tercero. */
  import { parseMarkdown, type Inline } from "$lib/design/markdown";

  let { source = "" }: { source?: string } = $props();

  const bloques = $derived(parseMarkdown(source));
</script>

{#snippet enLinea(nodos: Inline[])}
  {#each nodos as n}
    {#if n.tipo === "texto"}{n.valor}
    {:else if n.tipo === "codigo"}<code
        class="rounded-inner bg-glass-3 px-1.5 py-0.5 font-mono text-2xs text-fg">{n.valor}</code
      >
    {:else if n.tipo === "negrita"}<strong class="font-semibold text-fg">{@render enLinea(n.hijos)}</strong>
    {:else if n.tipo === "cursiva"}<em>{@render enLinea(n.hijos)}</em>
    {:else if n.tipo === "enlace"}<span>{n.texto} ({n.url})</span>
    {/if}
  {/each}
{/snippet}

<div class="flex flex-col gap-3 text-sm leading-relaxed text-fg-dim">
  {#each bloques as b}
    {#if b.tipo === "encabezado"}
      <h3 class="m-0 font-semibold tracking-tight text-fg {b.nivel === 1 ? 'text-base' : 'text-sm'}">
        {@render enLinea(b.hijos)}
      </h3>
    {:else if b.tipo === "parrafo"}
      <p class="m-0" style="text-wrap: pretty">{@render enLinea(b.hijos)}</p>
    {:else if b.tipo === "lista"}
      {#if b.ordenada}
        <ol class="m-0 flex list-decimal flex-col gap-1 pl-5">
          {#each b.items as item}<li>{@render enLinea(item)}</li>{/each}
        </ol>
      {:else}
        <ul class="m-0 flex list-disc flex-col gap-1 pl-5">
          {#each b.items as item}<li>{@render enLinea(item)}</li>{/each}
        </ul>
      {/if}
    {:else if b.tipo === "codigo"}
      <pre class="m-0 overflow-x-auto rounded-inner bg-glass-3 p-3 font-mono text-2xs text-fg">{b.valor}</pre>
    {:else if b.tipo === "cita"}
      <blockquote class="m-0 border-l-2 border-hairline pl-3 italic">
        {@render enLinea(b.hijos)}
      </blockquote>
    {/if}
  {/each}
</div>
