<script lang="ts">
  /** Globo de ayuda al pasar el ratón o al enfocar (patrón WAI-ARIA «tooltip»). Genérico: el
   *  llamante da el contenido visible (`children`), un `titulo` y el `text` de ayuda. El globo de
   *  lectura de una gráfica es `ChartTip`, que es otra cosa (sigue al puntero sobre un lienzo).
   *
   *  Dos modos:
   *   - `focusable` (por defecto): el disparador es un `<button>`; el globo aparece al pasar el ratón
   *     **y al enfocar con el teclado**, y se cierra con `Escape`, al perder el foco o al salir con
   *     el ratón. Cumple WCAG 1.4.13 (descartable, persistente, se puede llevar el ratón al globo:
   *     va pegado al disparador y ambos cuelgan del mismo contenedor).
   *   - `focusable={false}`: el disparador es un `<span>` no enfocable; **solo con el ratón**. Para
   *     cuando va dentro de un `<a>` (la `DiskCard` del panel), que no puede contener un elemento
   *     tabulable. */
  import { healthToken } from "$lib/design/health";
  import type { HealthState } from "$lib/design/types";

  let {
    titulo = "",
    text = "",
    /** Filo izquierdo del globo con el color del estado; el veredicto lo remata visualmente. */
    acento = undefined as HealthState | undefined,
    /** Clases del disparador, para que se mimetice con su sitio. */
    disparador = "",
    focusable = true,
    children
  } = $props();

  let abierto = $state(false);
  let debajo = $state(false);
  let alineacion = $state<"centro" | "izquierda" | "derecha">("centro");
  let raiz = $state<HTMLElement>();

  const id = `tt-${Math.random().toString(36).slice(2, 9)}`;
  /** Aprox. la mitad de `max-w-xs` (20rem): basta para decidir si el globo centrado se saldría. */
  const MEDIA_GLOBO = 160;

  function abrir() {
    const r = raiz?.getBoundingClientRect();
    if (r) {
      debajo = r.top < 170;
      const centro = r.left + r.width / 2;
      alineacion =
        centro - MEDIA_GLOBO < 8
          ? "izquierda"
          : centro + MEDIA_GLOBO > window.innerWidth - 8
            ? "derecha"
            : "centro";
    }
    abierto = true;
  }
  const cerrar = () => (abierto = false);
</script>

<span
  bind:this={raiz}
  class="relative flex min-w-0"
  role="none"
  onmouseenter={abrir}
  onmouseleave={cerrar}
  onkeydown={(e) => e.key === "Escape" && cerrar()}
>
  {#if focusable}
    <button
      type="button"
      class={disparador}
      onfocus={abrir}
      onblur={cerrar}
      aria-describedby={abierto ? id : undefined}
    >
      {@render children?.()}
    </button>
  {:else}
    <span class={disparador}>{@render children?.()}</span>
  {/if}

  {#if abierto}
    <div
      {id}
      role="tooltip"
      class="sdm-material-overlay absolute z-50 w-max max-w-xs whitespace-pre-line rounded-inner p-3 text-2xs leading-normal
             {debajo ? 'top-full' : 'bottom-full'}
             {alineacion === 'centro'
        ? 'left-1/2 -translate-x-1/2'
        : alineacion === 'izquierda'
          ? 'left-0'
          : 'right-0'}"
      style="background: var(--sdm-glass-strong){acento
        ? `; border-left: 3px solid ${healthToken[acento].fg}`
        : ''}"
    >
      {#if titulo}<span class="mb-1 block font-semibold text-fg">{titulo}</span>{/if}
      <span class="block text-fg-dim">{text}</span>
    </div>
  {/if}
</span>
