<script lang="ts">
  /** Tarjeta de material: cristal translúcido + filo de 1px + sombra suave, radio 18.
   *  Los bloques internos usan rounded-inner (13) para mantener radios concéntricos.
   *  No apiles materiales: una tarjeta no contiene otra tarjeta. */
  let { padding = "md", title = "", action = undefined, children } = $props();
  const pads: Record<string, string> = { none: "", sm: "p-4", md: "p-5", lg: "p-6" };
</script>

<section class="sdm-material flex flex-col gap-3 rounded-card {pads[padding]}">
  {#if title || action}
    <header class="flex items-center gap-3">
      <h2 class="m-0 text-base font-semibold tracking-tight">{title}</h2>
      <div class="flex-1"></div>
      {@render action?.()}
    </header>
  {/if}
  {@render children?.()}
</section>
