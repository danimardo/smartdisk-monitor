<script lang="ts">
  /** Selector excluyente en cápsula sobre material (intervalos 24 h / 7 d / 30 d / personalizado,
   *  filtros de alertas). El segmento activo se eleva con material, no con color de relleno. */
  let { options = [] as { id: string; label: string }[], value = "", onchange } = $props();
</script>

<div class="inline-flex gap-0.5 rounded-pill bg-glass-3 p-[3px]" role="radiogroup">
  {#each options as opt (opt.id)}
    <button
      role="radio"
      aria-checked={value === opt.id}
      class="rounded-pill px-3 py-1 text-xs font-semibold transition-all duration-base ease-sdm
             {value === opt.id
        ? 'bg-glass text-fg shadow-card shadow-edge'
        : 'text-fg-dim hover:bg-glass-2 hover:text-fg'}"
      onclick={() => onchange?.(opt.id)}
    >
      {opt.label}
    </button>
  {/each}
</div>
