<script lang="ts" generics="T">
  /** Lista virtualizada (US-021: un servidor genera miles de eventos). Renderiza solo las filas
   *  visibles + un margen de reserva; el resto se representa como un hueco de la altura correcta,
   *  para que la barra de desplazamiter sea la del total, no la de lo renderizado. Autorizado sin
   *  nueva decisión de diseño (`docs/ui-design.md` §3). */
  import type { Snippet } from "svelte";

  let {
    items = [] as T[],
    itemHeight = 44,
    height = 480,
    overscan = 8,
    row,
    label = ""
  }: {
    items: T[];
    itemHeight?: number;
    height?: number;
    overscan?: number;
    row: Snippet<[T, number]>;
    label?: string;
  } = $props();

  let contenedor = $state<HTMLDivElement | undefined>(undefined);
  let scrollTop = $state(0);
  let alturaVisible = $state(0);

  const total = $derived(items.length);
  const alturaTotal = $derived(total * itemHeight);

  const primeraVisible = $derived(Math.max(0, Math.floor(scrollTop / itemHeight) - overscan));
  const ultimaVisible = $derived(
    Math.min(total, Math.ceil((scrollTop + alturaVisible) / itemHeight) + overscan)
  );
  const visibles = $derived(items.slice(primeraVisible, ultimaVisible));

  function alDesplazar() {
    if (contenedor) scrollTop = contenedor.scrollTop;
  }
</script>

<div
  bind:this={contenedor}
  bind:clientHeight={alturaVisible}
  onscroll={alDesplazar}
  class="overflow-y-auto"
  style="height: {height}px"
>
  <div role="list" aria-label={label} style="position: relative; height: {alturaTotal}px">
    {#each visibles as item, i (primeraVisible + i)}
      <div
        role="listitem"
        style="position: absolute; top: {(primeraVisible + i) *
          itemHeight}px; left: 0; right: 0; height: {itemHeight}px"
      >
        {@render row(item, primeraVisible + i)}
      </div>
    {/each}
  </div>
</div>
