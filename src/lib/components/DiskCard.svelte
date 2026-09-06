<script lang="ts">
  /** Tarjeta de disco del panel general (US-012), recompuesta en v3 (`DiskCard.md`): cabecera de
   *  52 px que **hereda el color del estado** con la sparkline de temperatura de fondo, cifras de
   *  display, y el dato ausente como «—» discreto —no «No disponible» a 23 px, que pesaba más que
   *  el dato presente—.
   *
   *  Navega con **un enlace real** (constitución §XIV): conserva ctrl+clic, clic central, menú
   *  contextual, foco y el anuncio como enlace. Sin `href` se renderiza como bloque no interactivo. */
  import Card from "./Card.svelte";
  import CapacityBar from "./CapacityBar.svelte";
  import Icon from "./Icon.svelte";
  import Sparkline from "./Sparkline.svelte";
  import StatusPill from "./StatusPill.svelte";
  import { healthToken } from "$lib/design/health";
  import { busIcon } from "$lib/design/icons";
  import { formatTemperature, formatPercent, formatBytes } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import type { DiskSummary, HealthState } from "$lib/design/types";
  import type { Punto } from "$lib/design/series";

  let {
    disk = null as DiskSummary | null,
    href = undefined as string | undefined,
    /** Serie de temperatura de 24 h para el fondo de la cabecera. Sin ella, cabecera plana. */
    temperatureSeries = [] as Punto[]
  } = $props();

  const state = $derived<HealthState>(disk?.state ?? "unknown");
  const tone = $derived(healthToken[state]);
  /** No hay lectura SMART reciente: el bus no la expone, dejó de responder, o aún no ha llegado la
   *  primera. El backend lo marca poniendo `unknownReason` (lo deja en `null` en cuanto la lectura
   *  es fresca). Independiente del color: un disco puede contar como advertencia (`state` ya
   *  elevado por `estadoConAlertas`) *y* no tener SMART fresco a la vez. */
  const sinSmartFresco = $derived(disk?.unknownReason != null);
  /** El texto de la píldora dice **por qué** (sin datos SMART); el color lo pone `state`. */
  const label = $derived(sinSmartFresco ? t("disk.noSmartData") : t(`health.${state}`));
  const overTempLimit = $derived(
    !!disk?.temperatureC && !!disk?.vendorTempLimitC && disk.temperatureC >= disk.vendorTempLimitC
  );
  const volume = $derived(disk?.volumes?.[0] ?? null);
  const hayCurva = $derived(!sinSmartFresco && temperatureSeries.some((p) => p.v !== null));

  /** Un dato ausente se compone como «—» a text-xs en gris, con el texto completo en el `title`
   *  (`DiskCard.md`): así el disco sin SMART deja de pesar más que el que tiene datos. */
  const magnitudes = $derived([
    {
      icon: "temp" as const,
      label: t("disk.temperature"),
      value: disk?.temperatureC ?? null,
      text: disk ? formatTemperature(disk.temperatureC) : "",
      color: overTempLimit ? "var(--sdm-warn)" : "var(--sdm-text)"
    },
    {
      icon: "wear" as const,
      label: t("disk.wear"),
      value: disk?.percentageUsed ?? null,
      text: disk ? formatPercent(disk.percentageUsed) : "",
      color: "var(--sdm-text)"
    },
    {
      icon: "pulse" as const,
      label: t("disk.activity"),
      value: disk?.activityPercent ?? null,
      text: disk ? formatPercent(disk.activityPercent) : "",
      color: "var(--sdm-text)"
    }
  ]);
</script>

{#if disk}
  <svelte:element
    this={href ? "a" : "div"}
    href={href || undefined}
    class="block text-left"
    aria-label={href ? t("disk.open", { name: disk.alias ?? disk.model }) : undefined}
  >
    <Card padding="none">
      <div class="relative flex h-13 items-center gap-3 overflow-hidden px-4" style="background: {tone.soft}">
        {#if hayCurva}
          <div class="pointer-events-none absolute inset-0 opacity-[0.85]">
            <Sparkline points={temperatureSeries} color={tone.fg} height={52} strokeWidth={1.6} />
          </div>
        {/if}
        <span
          class="relative grid size-[30px] shrink-0 place-items-center rounded-nav text-fg-onAccent"
          style="background: {tone.fg}"
        >
          <Icon name={busIcon(disk.deviceType)} size={16} />
        </span>
        <div class="relative flex-1"></div>
        <span class="relative sdm-material rounded-pill">
          <StatusPill {state} {label} icon="auto" />
        </span>
      </div>

      <div class="flex flex-col gap-3 px-4 pb-4 pt-3">
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="sdm-selectable sdm-display truncate text-base">{disk.alias ?? disk.model}</span>
          <span class="sdm-selectable truncate text-2xs text-fg-dim">{disk.model} · {disk.deviceType}</span>
        </div>

        <div class="grid grid-cols-3 gap-2">
          {#each magnitudes as m}
            <div class="flex min-w-0 flex-col gap-0.5">
              <span class="flex items-center gap-1 text-2xs font-medium text-fg-faint">
                <Icon name={m.icon} size={12} />
                <span class="truncate">{m.label}</span>
              </span>
              <!-- Sin SMART fresco las tres magnitudes van a «—», aunque quede en la base una
                   lectura vieja o un contador de rendimiento en vivo (boceto §4): un dato caduco
                   presentado como actual engaña. -->
              {#if m.value === null || sinSmartFresco}
                <span class="text-xs text-fg-dim" title={t("common.notAvailable")}>—</span>
              {:else}
                <span class="sdm-num sdm-display text-xl" style="color: {m.color}">{m.text}</span>
              {/if}
            </div>
          {/each}
        </div>

        {#if volume}
          <CapacityBar
            label={`${volume.driveLetters.join(", ") || volume.label} · ${formatBytes(volume.capacityBytes)}`}
            capacityBytes={volume.capacityBytes}
            freeBytes={volume.freeBytes}
          />
        {:else}
          <!-- Misma altura que la barra, para que la rejilla no se descuadre (`DiskCard.md`). -->
          <span class="flex h-[38px] items-center text-2xs text-fg-faint">{t("disk.noVolumes")}</span>
        {/if}
      </div>
    </Card>
  </svelte:element>
{/if}
