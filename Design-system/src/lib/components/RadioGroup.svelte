<script lang="ts">
  /** Elección excluyente con descripción por opción (tema, comportamiento al cerrar,
   *  nivel de anonimización del ZIP). Cuando las etiquetas son cortas, SegmentedControl es mejor. */
  let { value = "", label = "", options = [] as { id: string; label: string; hint?: string }[], onchange } = $props();
</script>

<fieldset class="m-0 flex flex-col gap-2 border-none p-0" role="radiogroup" aria-label={label}>
  {#if label}<legend class="mb-1 p-0 text-sm font-medium">{label}</legend>{/if}
  {#each options as opt (opt.id)}
    <button
      role="radio"
      aria-checked={value === opt.id}
      class="flex items-start gap-3 rounded-inner border p-3 text-left transition-all duration-base ease-sdm
             {value === opt.id ? 'border-accent bg-accent-soft' : 'border-hairline bg-glass-2 hover:bg-glass'}"
      onclick={() => onchange?.(opt.id)}
    >
      <span class="mt-0.5 grid size-4 shrink-0 place-items-center rounded-pill border {value === opt.id ? 'border-accent' : 'border-hairline'}">
        {#if value === opt.id}<span class="size-2 rounded-pill bg-accent"></span>{/if}
      </span>
      <span class="flex flex-col gap-0.5">
        <span class="text-sm font-medium">{opt.label}</span>
        {#if opt.hint}<span class="text-xs text-fg-dim" style="text-wrap: pretty">{opt.hint}</span>{/if}
      </span>
    </button>
  {/each}
</fieldset>
