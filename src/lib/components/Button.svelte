<script lang="ts">
  /** Botón cápsula. Primary lleva degradado vertical del acento del sistema y brillo interior de 1px;
   *  secondary y ghost son material translúcido. Una sola primary por pantalla.
   *  Toda acción que escriba datos o genere carga abre ConfirmDialog antes (spec §12).
   *  `hint`: ayuda breve como `title` nativo cuando el botón está activo; si está desactivado,
   *  manda `disabledReason`. */
  type Variant = "primary" | "secondary" | "ghost" | "danger";
  type Size = "sm" | "md" | "lg";

  let {
    variant = "secondary" as Variant,
    size = "md" as Size,
    disabled = false,
    disabledReason = "",
    hint = "",
    loading = false,
    full = false,
    type = "button" as "button" | "submit" | "reset",
    onclick = undefined,
    children
  } = $props();

  const base =
    "inline-flex items-center justify-center gap-2 rounded-pill font-semibold whitespace-nowrap " +
    "transition-all duration-base ease-sdm active:scale-[0.98] disabled:opacity-45 disabled:cursor-not-allowed";

  const variants: Record<Variant, string> = {
    primary:
      "border border-transparent text-fg-onAccent " +
      "bg-[linear-gradient(180deg,var(--sdm-accent-hi),var(--sdm-accent))] " +
      "shadow-[inset_0_1px_0_rgba(255,255,255,.35),0_1px_2px_rgba(0,0,0,.14)] hover:brightness-[1.06]",
    secondary: "border border-hairline bg-glass-2 text-fg shadow-edge backdrop-blur-card hover:bg-glass",
    ghost: "border border-transparent bg-transparent text-fg-dim hover:bg-glass-3 hover:text-fg",
    danger: "border border-crit bg-crit-soft text-crit hover:brightness-105"
  };

  const sizes: Record<Size, string> = {
    sm: "h-control-sm px-3 text-xs",
    md: "h-control-md px-4 text-xs",
    lg: "h-control-lg px-5 text-sm"
  };
</script>

<button
  {type}
  class="{base} {variants[variant]} {sizes[size]} {full ? 'w-full' : ''}"
  disabled={disabled || loading}
  title={disabled && disabledReason ? disabledReason : hint || undefined}
  aria-disabled={disabled || loading}
  {onclick}
>
  {#if loading}<span
      class="size-3 rounded-pill border-2 border-current border-t-transparent animate-spin"
      aria-hidden="true"
    ></span>{/if}
  {@render children?.()}
</button>
