<script lang="ts">
  /** Icono de línea (v3, ADR-034). Referencia un `<symbol>` del sprite montado en `AppShell` y
   *  hereda `currentColor` del contenedor — nunca fija un color.
   *
   *  Accesibilidad: si el icono es el **único** portador de un significado se pasa `label` y el SVG
   *  sale como `role="img"` con nombre accesible; si acompaña a un texto que ya lo dice se pasa sin
   *  `label` y queda `aria-hidden`. Un `role="img"` sin nombre es peor que no ponerlo
   *  (`ui-design.md` §6). */
  import type { IconName } from "$lib/design/icons";

  let { name, size = 16, label = undefined }: { name: IconName; size?: number; label?: string } = $props();
</script>

{#if label}
  <svg width={size} height={size} viewBox="0 0 24 24" role="img" aria-label={label}>
    <use href="#i-{name}" />
  </svg>
{:else}
  <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true" focusable="false">
    <use href="#i-{name}" />
  </svg>
{/if}

<style>
  svg {
    display: inline-block;
    flex: none;
    vertical-align: text-bottom;
  }
</style>
