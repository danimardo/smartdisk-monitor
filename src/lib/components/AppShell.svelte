<script lang="ts">
  /** Armazón de la ventana: Sidebar fija + Toolbar fija + región de contenido con scroll propio.
   *  Úsalo como raíz de la app; ninguna pantalla monta su propio chrome.
   *  El contenido pasa por debajo del chrome translúcido: no le pongas fondo opaco.
   *
   *  `transitionKey` distingue un cambio de pantalla real (`docs/ui-design.md`: "Movimiento:
   *  duration-base con ease-sdm en... cambio de pantalla") de un simple cambio de parámetro dentro
   *  de la misma pantalla (un filtro, una página): solo cuando cambia se remonta el contenido y se
   *  repite la animación de entrada. Sin él, cada pantalla aparecía de golpe, sin transición. */
  let { sidebar, toolbar, children, transitionKey = "" } = $props();
</script>

<div
  class="relative flex h-screen overflow-hidden rounded-window bg-[linear-gradient(160deg,var(--sdm-bg),var(--sdm-bg-2))]"
>
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
</style>
