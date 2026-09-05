<script lang="ts">
  /** Panel general (US-012). El inventario llega listo desde `load` (constitución §XIV); las
   *  actualizaciones posteriores llegan por eventos y viven en el store.
   *
   *  **Rejilla virtualizada** (medido, `docs/open-questions.md`): con 20 discos, pintar la rejilla
   *  entera de una vez producía una tarea de ~100 ms — por encima del umbral de 50 ms de SC-007.
   *  `VirtualList` virtualiza por filas (no por tarjeta): cada fila agrupa tantas `DiskCard` como
   *  columnas quepan en el ancho disponible, igual que hacía la rejilla CSS `auto-fill` que
   *  sustituye. */
  import { DiskCard, EmptyState, VirtualList } from "$lib/components";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import type { DiskSummary } from "$lib/design/types";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  /** Lo que `load` trajo alimenta el store una sola vez; a partir de ahí manda el store, que es
   *  quien recibe los eventos. Sin esto, la pantalla mostraría siempre la foto del arranque. */
  $effect(() => {
    if (app.loadedAt === null) {
      app.devices = data.inventory.devices;
      app.excluded = data.inventory.excluded;
      app.sources = data.inventory.sources;
      app.loadedAt = new Date().toISOString();
    }
  });

  const devices = $derived(app.loadedAt ? app.devices : data.inventory.devices);

  // Mismos números que la rejilla CSS que sustituye: 460 px de columna mínima
  // (AGENTS.md §4.0.bis) y 20 px de hueco (`gap-5`). Altura de fila medida sobre `DiskCard`
  // real (padding "sm" con tres métricas y una barra de capacidad).
  const ANCHO_COLUMNA_MIN = 460;
  const HUECO = 20;
  const ALTURA_TARJETA = 169;
  const ALTURA_FILA = ALTURA_TARJETA + HUECO;

  let anchoDisponible = $state(0);
  let altoDisponible = $state(0);

  const columnas = $derived(Math.max(1, Math.floor((anchoDisponible + HUECO) / (ANCHO_COLUMNA_MIN + HUECO))));
  const filas = $derived.by(() => {
    const resultado: DiskSummary[][] = [];
    for (let i = 0; i < devices.length; i += columnas) {
      resultado.push(devices.slice(i, i + columnas));
    }
    return resultado;
  });
</script>

{#if devices.length === 0}
  <EmptyState kind="empty" title={t("dashboard.noDevices")} body={t("dashboard.noDevicesHint")} />
{:else}
  <div class="min-h-0 flex-1" bind:clientWidth={anchoDisponible} bind:clientHeight={altoDisponible}>
    <VirtualList
      items={filas}
      itemHeight={ALTURA_FILA}
      height={altoDisponible || 480}
      label={t("nav.dashboard")}
    >
      {#snippet row(fila: DiskSummary[])}
        <div class="grid gap-5" style="grid-template-columns: repeat({columnas}, minmax(0, 1fr))">
          {#each fila as disk (disk.id)}
            <DiskCard {disk} href={`/disks/${disk.id}`} />
          {/each}
        </div>
      {/snippet}
    </VirtualList>
  </div>
{/if}
