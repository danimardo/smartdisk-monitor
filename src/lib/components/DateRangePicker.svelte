<script lang="ts">
  /** Selector de intervalo personalizado (US-020, US-050): dos fechas, sin hora — la resolución
   *  de la serie ya declara su propia cadencia, no hace falta que el usuario afine a la hora.
   *  Autorizado sin nueva decisión de diseño (`docs/ui-design.md` §3, "Autorizados y pendientes
   *  de construir").
   *
   *  Las etiquetas «Desde»/«Hasta» van **en línea** con su campo, no encima: así el control es de
   *  una sola altura y se alinea con el `SegmentedControl` cuando comparten fila (detalle de disco). */
  let {
    from = "" as string,
    to = "" as string,
    label = "",
    fromLabel = "",
    toLabel = "",
    max = "" as string,
    onchange = undefined as ((range: { from: string; to: string }) => void) | undefined
  } = $props();
</script>

<div class="flex flex-col gap-2">
  {#if label}<span class="text-sm font-medium">{label}</span>{/if}
  <div class="flex flex-wrap items-center gap-3">
    <label class="flex items-center gap-2">
      {#if fromLabel}<span class="text-2xs text-fg-dim">{fromLabel}</span>{/if}
      <input
        type="date"
        class="h-control-lg rounded-pill border border-hairline bg-glass-2 px-4 text-xs font-medium text-fg shadow-edge
               transition-all duration-base ease-sdm"
        value={from}
        {max}
        aria-label={fromLabel || label}
        onchange={(e) => onchange?.({ from: e.currentTarget.value, to })}
      />
    </label>
    <label class="flex items-center gap-2">
      {#if toLabel}<span class="text-2xs text-fg-dim">{toLabel}</span>{/if}
      <input
        type="date"
        class="h-control-lg rounded-pill border border-hairline bg-glass-2 px-4 text-xs font-medium text-fg shadow-edge
               transition-all duration-base ease-sdm"
        value={to}
        min={from}
        {max}
        aria-label={toLabel || label}
        onchange={(e) => onchange?.({ from, to: e.currentTarget.value })}
      />
    </label>
  </div>
</div>
