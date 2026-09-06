<script lang="ts">
  /** Tarjeta de material: cristal translúcido + filo de 1px + sombra suave, radio 18.
   *  Los bloques internos usan rounded-inner (13) para mantener radios concéntricos.
   *  No apiles materiales: una tarjeta no contiene otra tarjeta. */
  let {
    padding = "md",
    title = "",
    action = undefined,
    /** Contenido opcional a la izquierda del título (v3: cuadrado de icono de las tarjetas de prueba). */
    leading = undefined,
    /** Filo de la tarjeta. `crit` marca una zona destructiva (`06-ajustes.md`); el fondo no cambia. */
    border = "hairline" as "hairline" | "crit",
    /** Utilidades extra sobre la `<section>`: para que el llamante ajuste el encaje (p. ej.
     *  `h-full` en una rejilla que quiere todas las tarjetas iguales), nunca colores ni radios. */
    class: klass = "",
    children
  } = $props();
  const pads: Record<string, string> = { none: "", sm: "p-4", md: "p-5", lg: "p-6" };
</script>

<section
  class="sdm-material flex flex-col gap-3 rounded-card {pads[padding]} {border === 'crit'
    ? 'border-crit'
    : ''} {klass}"
>
  {#if title || action || leading}
    <header class="flex items-center gap-3">
      {@render leading?.()}
      <h2 class="m-0 text-base font-semibold tracking-tight">{title}</h2>
      <div class="flex-1"></div>
      {@render action?.()}
    </header>
  {/if}
  {@render children?.()}
</section>
