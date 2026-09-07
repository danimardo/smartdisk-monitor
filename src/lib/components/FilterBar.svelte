<script lang="ts">
  /** Barra de filtros multiselección (US-021: filtrar eventos por nivel y proveedor). `Select`
   *  no sirve aquí: solo permite una opción, y aquí varias pueden estar activas a la vez.
   *  Autorizado sin nueva decisión de diseño (`docs/ui-design.md` §3). */
  let {
    groups = [] as { id: string; label: string; options: { id: string; label: string }[] }[],
    selected = {} as Record<string, string[]>,
    onchange = undefined as ((groupId: string, values: string[]) => void) | undefined
  } = $props();

  function alternar(groupId: string, optionId: string) {
    const actuales = selected[groupId] ?? [];
    const nuevos = actuales.includes(optionId)
      ? actuales.filter((v) => v !== optionId)
      : [...actuales, optionId];
    onchange?.(groupId, nuevos);
  }
</script>

<div class="flex flex-wrap items-center gap-4">
  {#each groups as group (group.id)}
    <div class="flex items-center gap-1.5" role="group" aria-label={group.label}>
      {#each group.options as opt (opt.id)}
        {@const activo = (selected[group.id] ?? []).includes(opt.id)}
        <button
          type="button"
          aria-pressed={activo}
          class="rounded-pill px-3 py-1 text-xs font-semibold transition-all duration-base ease-sdm
                 {activo
            ? 'bg-glass text-fg shadow-card shadow-edge'
            : 'text-fg-dim hover:bg-glass-2 hover:text-fg'}"
          onclick={() => alternar(group.id, opt.id)}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  {/each}
</div>
