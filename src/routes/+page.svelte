<script lang="ts">
  /** Panel general (US-012). El inventario llega listo desde `load` (constitución §XIV); las
   *  actualizaciones posteriores llegan por eventos y viven en el store. */
  import { DiskCard, EmptyState } from "$lib/components";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
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
</script>

{#if devices.length === 0}
  <EmptyState kind="empty" title={t("dashboard.noDevices")} body={t("dashboard.noDevicesHint")} />
{:else}
  <!-- Rejilla fluida desde 460 px: por debajo, las cuatro métricas no caben (AGENTS.md §4.0.bis). -->
  <div class="grid gap-5" style="grid-template-columns: repeat(auto-fill, minmax(460px, 1fr))">
    {#each devices as disk (disk.id)}
      <DiskCard {disk} href={`/disks/${disk.id}`} />
    {/each}
  </div>
{/if}
