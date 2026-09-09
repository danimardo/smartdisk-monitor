<script lang="ts">
  /** Desplegable para listas largas (idioma, retención, volumen objetivo).
   *  Para 2–4 opciones excluyentes usa SegmentedControl o RadioGroup.
   *  `size="sm"`: compacto (misma altura que un `Button size="sm"`), para usarlo en línea junto a
   *  botones; no muestra el rótulo ni el `hint` visibles pero conserva `aria-label={label}`. */
  let {
    value = "",
    label = "",
    hint = "",
    options = [] as { id: string; label: string }[],
    disabled = false,
    size = "lg" as "sm" | "lg",
    /** Callback opcional (`interfaz.md`): el componente funciona sin él. */
    onchange = undefined
  } = $props();

  const compacto = $derived(size === "sm");
</script>

<label class="flex flex-col gap-2">
  {#if label && !compacto}<span class="text-sm font-medium">{label}</span>{/if}
  <select
    class="{compacto
      ? 'h-control-sm px-3'
      : 'h-control-lg px-4'} rounded-pill border border-hairline bg-glass-2 text-xs font-medium text-fg shadow-edge
           transition-all duration-base ease-sdm hover:border-fg-faint hover:bg-glass disabled:opacity-45"
    {value}
    {disabled}
    aria-label={label}
    onchange={(e) => onchange?.(e.currentTarget.value)}
  >
    {#each options as opt (opt.id)}<option value={opt.id}>{opt.label}</option>{/each}
  </select>
  {#if hint && !compacto}<span class="text-xs text-fg-dim" style="text-wrap: pretty">{hint}</span>{/if}
</label>
