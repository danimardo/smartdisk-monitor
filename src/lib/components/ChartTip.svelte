<script lang="ts">
  /** Globo flotante con el valor de un punto de una gráfica, al pasar el ratón o al recorrerla con
   *  el teclado. Presentacional: no calcula nada, lo posiciona el llamante en píxeles dentro de su
   *  contenedor (que debe ser `position: relative`). Lo usan `TimeSeriesChart` y la `Sparkline`
   *  interactiva, y cualquier gráfica futura, para que la lectura se vea igual en todas.
   *
   *  `aria-hidden`: el valor lo anuncia la región viva de la gráfica (`aria-live`), este globo es
   *  solo el adorno visual. */
  let {
    /** Posición del punto, en píxeles dentro del contenedor de la gráfica. */
    x = 0,
    y = 0,
    /** Ancho del contenedor: se usa para voltear el globo antes de que se salga por la derecha. */
    anchoContenedor = 0,
    texto = ""
  } = $props();

  /** Estimación grosera del ancho del globo (≈7 px por carácter + relleno) para decidir el volteo
   *  sin medir el DOM: no hay que acertar el ancho, solo evitar que se recorte. */
  const anchoEstimado = $derived(texto.length * 7 + 20);
  const aLaIzquierda = $derived(x + anchoEstimado > anchoContenedor);
  /** Cerca del borde superior no cabe encima del punto: se pone debajo. */
  const abajo = $derived(y < 34);

  const desplazX = $derived(aLaIzquierda ? "calc(-100% - 8px)" : "8px");
  const desplazY = $derived(abajo ? "8px" : "calc(-100% - 8px)");
</script>

<div
  class="sdm-material pointer-events-none absolute z-10 whitespace-nowrap rounded-inner px-2 py-1 text-2xs"
  style="left: {x}px; top: {y}px; transform: translate({desplazX}, {desplazY})"
  aria-hidden="true"
>
  <span class="sdm-num text-fg">{texto}</span>
</div>
