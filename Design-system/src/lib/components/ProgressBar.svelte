<script lang="ts">
  /** Progreso de una operación en curso (benchmark, chkdsk, autotest). Siempre con texto de estado y restante. */
  let { value = 0, caption = "", trailing = "", indeterminate = false } = $props();
</script>

<div class="flex flex-col gap-2">
  <div class="h-2.5 overflow-hidden rounded-pill bg-glass-3" role="progressbar" aria-valuenow={indeterminate ? undefined : value} aria-valuemin="0" aria-valuemax="100">
    <div
      class="h-full rounded-pill bg-accent transition-all duration-base ease-sdm {indeterminate ? 'animate-pulse w-1/3' : ''}"
      style={indeterminate ? "" : `width: ${Math.min(100, Math.max(0, value))}%`}
    ></div>
  </div>
  <div class="flex justify-between text-xs text-fg-dim">
    <span>{caption}</span>
    <span class="font-semibold text-fg">{trailing}</span>
  </div>
</div>
