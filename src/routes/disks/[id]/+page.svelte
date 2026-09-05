<script lang="ts">
  /** Detalle de disco (US-012, US-020). Composición aprobada (`docs/ui-design.md` §7.2): cabecera
   *  con alias y estado, fila de `MetricCard`, `TimeSeriesChart` de temperatura y panel de
   *  contadores. El `SegmentedControl` de intervalo vive en el contenido de la pantalla, no en la
   *  `Toolbar` compartida: el layout raíz no tiene todavía un mecanismo para que una pantalla le
   *  inyecte controles propios (`docs/open-questions.md`), y construirlo es un cambio de
   *  arquitectura más allá de esta tarea. */
  import {
    DataRow,
    DateRangePicker,
    EmptyState,
    MetricCard,
    SegmentedControl,
    StatusPill,
    TimeSeriesChart
  } from "$lib/components";
  import { getMetricSeries, toAppError } from "$lib/api";
  import {
    formatBytes,
    formatHours,
    formatLatency,
    formatPercent,
    formatTemperature,
    formatThroughput
  } from "$lib/design/format";
  import { i18n, t } from "$lib/i18n";
  import type { AppError } from "$lib/design/types";
  import type { MetricSeries } from "$lib/api";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();
  const disk = $derived(data.disk);

  type Rango = "24h" | "7d" | "30d" | "custom";
  let rango = $state<Rango>("24h");
  let customFrom = $state("");
  let customTo = $state("");

  const opcionesRango = [
    { id: "24h", label: t("range.24h") },
    { id: "7d", label: t("range.7d") },
    { id: "30d", label: t("range.30d") },
    { id: "custom", label: t("range.custom") }
  ];

  function calcularIntervalo(r: Rango, desde: string, hasta: string): { from: string; to: string } {
    const ahora = new Date();
    if (r === "custom" && desde && hasta) {
      return {
        from: new Date(`${desde}T00:00:00.000Z`).toISOString(),
        to: new Date(`${hasta}T23:59:59.999Z`).toISOString()
      };
    }
    const horas = r === "24h" ? 24 : r === "7d" ? 24 * 7 : 24 * 30;
    return {
      from: new Date(ahora.getTime() - horas * 60 * 60 * 1000).toISOString(),
      to: ahora.toISOString()
    };
  }

  let serie = $state<MetricSeries | null>(null);
  let serieError = $state<AppError | null>(null);
  let serieLoading = $state(false);

  $effect(() => {
    const id = disk.id;
    const { from, to } = calcularIntervalo(rango, customFrom, customTo);
    let cancelado = false;
    serieLoading = true;
    void (async () => {
      try {
        const s = await getMetricSeries({
          deviceId: id,
          metricKey: "temperature_celsius",
          fromUtc: from,
          toUtc: to
        });
        if (!cancelado) {
          serie = s;
          serieError = null;
        }
      } catch (cause) {
        if (!cancelado) {
          serie = null;
          serieError = toAppError(cause);
        }
      } finally {
        if (!cancelado) serieLoading = false;
      }
    })();
    return () => {
      cancelado = true;
    };
  });

  const resolutionLabel = $derived(serie ? t(`chart.resolution.${serie.resolution}`) : "");

  const ETIQUETA_REUTILIZADA: Record<string, string> = {
    temperature_celsius: "disk.temperature",
    activity_percent: "disk.activity",
    percentage_used: "disk.wear",
    power_on_hours: "disk.powerOnHours"
  };

  function etiquetaContador(metricKey: string): string {
    return t(ETIQUETA_REUTILIZADA[metricKey] ?? `smart.counter.${metricKey}`);
  }

  function formatearContador(unit: string | null, value: number | null): string {
    switch (unit) {
      case "celsius":
        return formatTemperature(value);
      case "percent":
        return formatPercent(value);
      case "hours":
        return formatHours(value);
      case "bytes":
        return formatBytes(value);
      case "bytes_per_second":
        return formatThroughput(value);
      case "milliseconds":
        return formatLatency(value);
      default:
        return value === null ? t("common.notAvailable") : value.toLocaleString(i18n.formatLocale);
    }
  }
</script>

<div class="flex flex-col gap-5">
  <div class="flex items-center gap-3">
    <StatusPill state={disk.state} label={t(`health.${disk.state}`)} />
    <h1 class="m-0 text-lg font-semibold">{disk.alias ?? disk.model}</h1>
  </div>

  <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
    <MetricCard
      label={t("disk.temperature")}
      value={disk.temperatureC !== null ? formatTemperature(disk.temperatureC) : null}
    />
    <MetricCard
      label={t("disk.activity")}
      value={disk.activityPercent !== null ? formatPercent(disk.activityPercent) : null}
    />
    <MetricCard
      label={t("disk.wear")}
      value={disk.percentageUsed !== null ? formatPercent(disk.percentageUsed) : null}
    />
    <MetricCard
      label={t("disk.powerOnHours")}
      value={disk.powerOnHours !== null ? formatHours(disk.powerOnHours) : null}
    />
  </div>

  <div class="flex flex-col gap-3">
    <div class="flex flex-wrap items-center gap-3">
      <SegmentedControl options={opcionesRango} value={rango} onchange={(v: Rango) => (rango = v)} />
      {#if rango === "custom"}
        <DateRangePicker
          from={customFrom}
          to={customTo}
          fromLabel={t("dateRange.from")}
          toLabel={t("dateRange.to")}
          onchange={(r: { from: string; to: string }) => {
            customFrom = r.from;
            customTo = r.to;
          }}
        />
      {/if}
    </div>

    {#if serieError}
      <EmptyState
        kind="error"
        title={t("error.screenFailed")}
        body={t(serieError.messageKey, serieError.messageVars)}
        detail={serieError.detail ?? ""}
      />
    {:else if serie}
      <TimeSeriesChart
        points={serie.points}
        from={new Date(serie.fromUtc).getTime()}
        to={new Date(serie.toUtc).getTime()}
        expectedIntervalMs={serie.expectedIntervalMs}
        unit="°C"
        min={0}
        max={100}
        {resolutionLabel}
      />
    {:else if serieLoading}
      <span class="text-xs text-fg-dim">{t("common.loading")}</span>
    {/if}
  </div>

  <div class="flex flex-col gap-1">
    <h2 class="m-0 text-sm font-semibold">{t("disk.counters")}</h2>
    {#each disk.counters as counter (counter.metricKey)}
      <DataRow
        label={etiquetaContador(counter.metricKey)}
        value={formatearContador(counter.unit, counter.value)}
      />
    {/each}
  </div>
</div>
