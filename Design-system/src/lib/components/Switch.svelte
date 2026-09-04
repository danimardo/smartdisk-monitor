<script lang="ts">
  /** Conmutador para preferencias booleanas (Ajustes, alertas por volumen).
   *  Etiqueta obligatoria; `hint` explica la consecuencia, no repite la etiqueta. */
  let { checked = false, label = "", hint = "", disabled = false, disabledReason = "", onchange } = $props();
</script>

<label class="flex items-start gap-3 {disabled ? 'opacity-45' : ''}" title={disabled && disabledReason ? disabledReason : undefined}>
  <button
    role="switch"
    aria-checked={checked}
    aria-label={label}
    {disabled}
    class="mt-0.5 h-[22px] w-[38px] shrink-0 rounded-pill border border-hairline p-[2px] transition-all duration-base ease-sdm
           {checked ? 'bg-[linear-gradient(180deg,var(--sdm-accent-hi),var(--sdm-accent))] shadow-[inset_0_1px_0_rgba(255,255,255,.35)]' : 'bg-glass-3'}"
    onclick={() => !disabled && onchange?.(!checked)}
  >
    <span
      class="block size-4 rounded-pill bg-white shadow-card transition-transform duration-base ease-sdm"
      style="transform: translateX({checked ? 16 : 0}px)"
    ></span>
  </button>
  <span class="flex flex-col gap-0.5">
    <span class="text-sm font-medium">{label}</span>
    {#if hint}<span class="text-xs text-fg-dim" style="text-wrap: pretty">{hint}</span>{/if}
  </span>
</label>
