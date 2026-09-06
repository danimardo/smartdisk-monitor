/** Tailwind mapeado 1:1 sobre design-system/tokens.css (v2 material translúcido).
 *  Si una utilidad no existe aquí, el valor no está en el sistema: no inventes clases arbitrarias.
 *  Las capas de material se aplican con las clases .sdm-material / -chrome / -overlay de tokens.css.
 *  @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,svelte,ts,js}"],
  theme: {
    extend: {
      colors: {
        bg: { DEFAULT: "var(--sdm-bg)", 2: "var(--sdm-bg-2)" },
        glass: { DEFAULT: "var(--sdm-glass)", 2: "var(--sdm-glass-2)", 3: "var(--sdm-glass-3)" },
        solid: "var(--sdm-solid)",
        hairline: "var(--sdm-hairline)",
        scrim: "var(--sdm-scrim)",
        fg: {
          DEFAULT: "var(--sdm-text)",
          dim: "var(--sdm-text-dim)",
          faint: "var(--sdm-text-faint)",
          onAccent: "var(--sdm-on-accent)"
        },
        accent: {
          DEFAULT: "var(--sdm-accent)",
          hi: "var(--sdm-accent-hi)",
          soft: "var(--sdm-accent-soft)",
          fg: "var(--sdm-accent-fg)"
        },
        ok: { DEFAULT: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)" },
        warn: { DEFAULT: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)" },
        crit: { DEFAULT: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)" },
        unknown: { DEFAULT: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)" }
      },
      fontFamily: {
        sans: "var(--sdm-font-sans)",
        mono: "var(--sdm-font-mono)",
        display: "var(--sdm-font-display)"
      },
      fontSize: {
        "2xs": ["var(--sdm-text-2xs)", { lineHeight: "1.35" }],
        xs: ["var(--sdm-text-xs)", { lineHeight: "1.45" }],
        sm: ["var(--sdm-text-sm)", { lineHeight: "1.5" }],
        base: ["var(--sdm-text-base)", { lineHeight: "1.4" }],
        lg: ["var(--sdm-text-lg)", { lineHeight: "1.3" }],
        xl: ["var(--sdm-text-xl)", { lineHeight: "1.2" }],
        "2xl": ["var(--sdm-text-2xl)", { lineHeight: "1.2" }],
        metric: ["var(--sdm-text-metric)", { lineHeight: "1.05" }],
        display: ["var(--sdm-text-display)", { lineHeight: "1" }],
        hero: ["var(--sdm-text-hero)", { lineHeight: "1" }]
      },
      fontWeight: { regular: "400", medium: "500", semibold: "600" },
      spacing: {
        1: "var(--sdm-space-1)",
        2: "var(--sdm-space-2)",
        3: "var(--sdm-space-3)",
        4: "var(--sdm-space-4)",
        5: "var(--sdm-space-5)",
        6: "var(--sdm-space-6)",
        8: "var(--sdm-space-8)"
      },
      borderRadius: {
        window: "var(--sdm-radius-window)",
        card: "var(--sdm-radius-card)",
        inner: "var(--sdm-radius-inner)",
        nav: "var(--sdm-radius-nav)",
        pill: "var(--sdm-radius-pill)"
      },
      height: {
        "control-sm": "var(--sdm-control-sm)",
        "control-md": "var(--sdm-control-md)",
        "control-lg": "var(--sdm-control-lg)",
        hero: "var(--sdm-hero-height)"
      },
      width: {
        rail: "var(--sdm-rail-width)"
      },
      boxShadow: {
        card: "var(--sdm-shadow)",
        lift: "var(--sdm-shadow-lift)",
        edge: "inset 0 1px 0 var(--sdm-highlight)",
        focus: "var(--sdm-focus-ring)"
      },
      backdropBlur: {
        chrome: "var(--sdm-blur-chrome)",
        card: "var(--sdm-blur-card)",
        overlay: "var(--sdm-blur-overlay)"
      },
      transitionTimingFunction: { sdm: "var(--sdm-ease)" },
      transitionDuration: {
        fast: "var(--sdm-duration-fast)",
        base: "var(--sdm-duration-base)",
        overlay: "var(--sdm-duration-overlay)"
      }
    }
  },
  plugins: []
};
