<script lang="ts">
  /** Armazón de la ventana: Sidebar fija + Toolbar fija + región de contenido con scroll propio.
   *  Úsalo como raíz de la app; ninguna pantalla monta su propio chrome.
   *  El contenido pasa por debajo del chrome translúcido: no le pongas fondo opaco.
   *
   *  `transitionKey` distingue un cambio de pantalla real (`docs/ui-design.md`: "Movimiento:
   *  duration-base con ease-sdm en... cambio de pantalla") de un simple cambio de parámetro dentro
   *  de la misma pantalla (un filtro, una página): solo cuando cambia se remonta el contenido y se
   *  repite la animación de entrada. Sin él, cada pantalla aparecía de golpe, sin transición. */
  import { navigating } from "$app/state";
  import { t } from "$lib/i18n";

  let { sidebar, toolbar, children, transitionKey = "", busy = false } = $props();

  // Barra fina de navegación (spec 004): la navegación de SvelteKit espera al `load` de la ruta;
  // si algo tarda, esto avisa de que la aplicación está trabajando en vez de parecer congelada. El
  // `animation-delay` la retrasa ~150 ms para que una navegación instantánea no la haga parpadear
  // (a diferencia de `animation-duration`, `prefers-reduced-motion` no anula `animation-delay`).
  // `busy` la reutiliza para una operación global en curso (p. ej. el refresco manual de datos).
  const trabajando = $derived(navigating.to != null || busy);
</script>

<!-- El sprite de iconos (v3, ADR-034) ya no vive aquí: se monta una sola vez en
     `src/routes/+layout.svelte` mediante `<IconSprite />`, fuera de `AppShell`, para que también
     resuelva en rutas sin chrome como el asistente inicial (`/onboarding`). -->

<div
  class="relative flex h-screen overflow-hidden rounded-window bg-[linear-gradient(160deg,var(--sdm-bg),var(--sdm-bg-2))]"
>
  {#if trabajando}
    <div
      class="sdm-nav-progress pointer-events-none absolute inset-x-0 top-0 z-50 h-[2px] bg-accent"
      role="progressbar"
      aria-label={t("common.loading")}
    ></div>
  {/if}
  {@render sidebar?.()}
  <div class="relative flex min-w-0 flex-1 flex-col">
    {@render toolbar?.()}
    <!-- Única región con scroll: la ventana nunca recorta contenido en silencio. -->
    <main class="min-h-0 flex-1 overflow-auto">
      {#key transitionKey}
        <div
          class="box-border flex min-h-full flex-col gap-5 p-6"
          style="animation: sdm-page-in var(--sdm-duration-base) var(--sdm-ease)"
        >
          {@render children?.()}
        </div>
      {/key}
    </main>
  </div>
</div>

<style>
  @keyframes sdm-page-in {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  /* Aparece tras 150 ms (navegación instantánea → no parpadea) y avanza mientras dura. */
  .sdm-nav-progress {
    transform-origin: left;
    animation:
      sdm-nav-appear 1ms linear 150ms both,
      sdm-nav-fill 1.4s ease-out 150ms forwards;
  }
  @keyframes sdm-nav-appear {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes sdm-nav-fill {
    from {
      transform: scaleX(0.05);
    }
    to {
      transform: scaleX(0.9);
    }
  }
</style>
