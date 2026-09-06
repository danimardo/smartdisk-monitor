<script lang="ts">
  /** Progreso de una operación en curso (benchmark, chkdsk, autotest). Siempre con texto de estado
   *  y restante: una barra sin leyenda no dice nada.
   *
   *  v3 (`ProgressBar.md`): prop `emphasis`. `inline` (por defecto) es la barra de 10 px de las
   *  listas y el historial; `display` es la de la prueba en curso — 12 px, relleno con degradado
   *  del acento y filo interior. La **cifra grande no va aquí**: la pone la pantalla, porque su
   *  posición depende de la composición de la cabecera. */
  let {
    value = 0,
    caption = "",
    trailing = "",
    indeterminate = false,
    emphasis = "inline" as "inline" | "display"
  } = $props();

  const pct = $derived(Math.min(100, Math.max(0, value)));
</script>

<div class="flex flex-col gap-2">
  <div
    class="{emphasis === 'display' ? 'h-3' : 'h-2.5'} overflow-hidden rounded-pill bg-glass-3"
    role="progressbar"
    aria-valuenow={indeterminate ? undefined : value}
    aria-valuemin="0"
    aria-valuemax="100"
  >
    <div
      class="h-full rounded-pill transition-all duration-base ease-sdm {indeterminate
        ? 'animate-pulse w-1/3'
        : ''} {emphasis === 'display' ? 'shadow-edge' : 'bg-accent'}"
      style="{indeterminate ? '' : `width: ${pct}%;`}{emphasis === 'display'
        ? ' background: linear-gradient(90deg, var(--sdm-accent), var(--sdm-accent-hi));'
        : ''}"
    ></div>
  </div>
  <div class="flex justify-between text-xs text-fg-dim">
    <span>{caption}</span>
    <span class="font-semibold text-fg">{trailing}</span>
  </div>
</div>
