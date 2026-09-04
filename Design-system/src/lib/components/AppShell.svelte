<script lang="ts">
  /** Armazón de la ventana: Sidebar fija + Toolbar fija + región de contenido con scroll propio.
   *  Úsalo como raíz de la app; ninguna pantalla monta su propio chrome.
   *  El contenido pasa por debajo del chrome translúcido: no le pongas fondo opaco. */
  let { sidebar, toolbar, children } = $props();
</script>

<div class="relative flex h-screen overflow-hidden rounded-window bg-[linear-gradient(160deg,var(--sdm-bg),var(--sdm-bg-2))]">
  {@render sidebar?.()}
  <div class="relative flex min-w-0 flex-1 flex-col">
    {@render toolbar?.()}
    <!-- Única región con scroll: la ventana nunca recorta contenido en silencio. -->
    <main class="min-h-0 flex-1 overflow-auto">
      <div class="box-border flex min-h-full flex-col gap-5 p-6">
        {@render children?.()}
      </div>
    </main>
  </div>
</div>
