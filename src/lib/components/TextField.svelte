<script lang="ts">
  /** Campo de texto o numérico (alias de disco, tamaño del archivo de prueba, retención).
   *  `error` se muestra en lenguaje comprensible; el detalle técnico va aparte. */
  let {
    value = "",
    label = "",
    hint = "",
    error = "",
    type = "text" as "text" | "number",
    suffix = "",
    min = undefined as number | undefined,
    max = undefined as number | undefined,
    step = undefined as number | undefined,
    placeholder = "",
    disabled = false,
    oninput = undefined
  } = $props();
</script>

<label class="flex flex-col gap-2">
  {#if label}<span class="text-sm font-medium">{label}</span>{/if}
  <span
    class="flex h-control-lg items-center gap-2 rounded-pill border bg-glass-2 px-4 shadow-edge transition-all duration-base ease-sdm
           {error ? 'border-crit' : 'border-hairline'}"
  >
    <input
      class="sdm-num w-full border-none bg-transparent text-xs font-medium text-fg outline-none placeholder:text-fg-faint"
      {type}
      {value}
      {placeholder}
      {disabled}
      {min}
      {max}
      {step}
      aria-label={label}
      aria-invalid={!!error}
      oninput={(e) => oninput?.(e.currentTarget.value)}
    />
    {#if suffix}<span class="shrink-0 text-xs text-fg-faint">{suffix}</span>{/if}
  </span>
  {#if error}
    <span class="text-xs font-medium text-crit" style="text-wrap: pretty">{error}</span>
  {:else if hint}
    <span class="text-xs text-fg-dim" style="text-wrap: pretty">{hint}</span>
  {/if}
</label>
