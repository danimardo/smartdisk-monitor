<script lang="ts">
  /** Tarjeta de disco del panel general (US-012). Muestra alias sobre modelo, estado, tres métricas
   *  y la capacidad del volumen principal. Un disco sin SMART se rotula "Sin datos SMART" en gris.
   *
   *  Navega con **un enlace real**, no con un callback (constitución §XIV): así conserva ctrl+clic,
   *  clic central, menú contextual, foco y el anuncio como enlace de un lector de pantalla. Sin
   *  `href` se renderiza como bloque no interactivo, que es lo correcto cuando no lleva a ninguna
   *  parte. */
  import Card from "./Card.svelte";
  import StatusPill from "./StatusPill.svelte";
  import CapacityBar from "./CapacityBar.svelte";
  import { formatTemperature, formatPercent, formatBytes } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import type { DiskSummary } from "$lib/design/types";

  let { disk = null as DiskSummary | null, href = undefined as string | undefined } = $props();

  const label = $derived(
    disk?.state === "unknown" ? t("disk.noSmartData") : t(`health.${disk?.state ?? "unknown"}`)
  );
  const overTempLimit = $derived(
    !!disk?.temperatureC && !!disk?.vendorTempLimitC && disk.temperatureC >= disk.vendorTempLimitC
  );
  const volume = $derived(disk?.volumes?.[0] ?? null);
</script>

{#if disk}
  <svelte:element
    this={href ? "a" : "div"}
    href={href || undefined}
    class="block text-left"
    aria-label={href ? t("disk.open", { name: disk.alias ?? disk.model }) : undefined}
  >
    <Card padding="sm">
      <div class="flex items-start gap-3">
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="truncate text-base font-semibold">{disk.alias ?? disk.model}</span>
          <span class="truncate text-xs text-fg-dim">{disk.model} · {disk.deviceType}</span>
        </div>
        <div class="flex-1"></div>
        <StatusPill state={disk.state} {label} />
      </div>

      <div class="flex gap-5">
        <div class="flex flex-col">
          <span class="text-2xs font-medium text-fg-faint">{t("disk.temperature")}</span>
          <span class="sdm-num text-xl font-semibold" style={overTempLimit ? "color: var(--sdm-warn)" : ""}>
            {formatTemperature(disk.temperatureC)}
          </span>
        </div>
        <div class="flex flex-col">
          <span class="text-2xs font-medium text-fg-faint">{t("disk.wear")}</span>
          <span class="sdm-num text-xl font-semibold">{formatPercent(disk.percentageUsed)}</span>
        </div>
        <div class="flex flex-col">
          <span class="text-2xs font-medium text-fg-faint">{t("disk.activity")}</span>
          <span class="sdm-num text-xl font-semibold">{formatPercent(disk.activityPercent)}</span>
        </div>
      </div>

      {#if volume}
        <CapacityBar
          label={`${volume.driveLetters.join(", ") || volume.label} · ${formatBytes(volume.capacityBytes)}`}
          capacityBytes={volume.capacityBytes}
          freeBytes={volume.freeBytes}
        />
      {:else}
        <span class="text-xs text-fg-faint">{t("disk.noVolumes")}</span>
      {/if}
    </Card>
  </svelte:element>
{/if}
