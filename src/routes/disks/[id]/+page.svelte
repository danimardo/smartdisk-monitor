<script lang="ts">
  /** Detalle de disco (US-012, US-020), v3 (`cambios/07-detalle-disco.md`): cabecera de identidad,
   *  cuatro `MetricCard` con icono y evolución, `TimeSeriesChart` con eje y huecos, y contadores
   *  con delta. El intervalo vive **junto a la gráfica** que modifica, ya no en la barra de
   *  herramientas (v3). */
  import {
    Button,
    Card,
    DataRow,
    DateRangePicker,
    EmptyState,
    ExplicacionModal,
    Icon,
    MetricCard,
    SegmentedControl,
    StatusPill,
    TimeSeriesChart
  } from "$lib/components";
  import {
    addSmartctlDefenderException,
    checkSmartctlDefenderException,
    getMetricSeries,
    toAppError
  } from "$lib/api";
  import { healthToken } from "$lib/design/health";
  import { busIcon } from "$lib/design/icons";
  import {
    formatBytes,
    formatHours,
    formatLatency,
    formatPercent,
    formatTemperature,
    formatThroughput,
    formatAge,
    maskSerial
  } from "$lib/design/format";
  import { classifyAgainstThresholds, temperatureThresholds } from "$lib/design/health";
  import { actividadDetalle, ayudaMetrica } from "$lib/design/metricHelp";
  import { i18n, t } from "$lib/i18n";
  import { ia } from "$lib/stores/ia.svelte";
  import { explicacion } from "$lib/stores/explicacion.svelte";
  import { goto } from "$app/navigation";
  import type { AppError } from "$lib/design/types";
  import type { MetricSeries } from "$lib/api";
  import type { Punto } from "$lib/design/series";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();
  const disk = $derived(data.disk);
  const settings = $derived(data.settings);
  const tone = $derived(healthToken[disk.state]);

  /** «Explícamelo» sobre los contadores SMART (spec 005, US4). Reusa la máquina de estados del
   *  modal, igual que `/alerts`. */
  function explicarSmart() {
    void explicacion.lanzar({
      tipo: "smart",
      deviceId: disk.id,
      alertGroupId: null,
      eventId: null,
      idioma: i18n.locale
    });
  }

  const tempThresholds = $derived(
    temperatureThresholds(
      disk.vendorTempLimitC,
      disk.vendorTempCriticalC,
      settings.alerts.tempConfiguredWarnC,
      settings.alerts.tempConfiguredCritC
    )
  );
  const tempState = $derived(
    classifyAgainstThresholds(disk.temperatureC, tempThresholds.warn, tempThresholds.crit)
  );

  /** Ayuda contextual de las cuatro métricas: qué es cada una y si el valor actual preocupa. Los
   *  umbrales salen de `settings.alerts`, así el veredicto nunca contradice al color ni a una alerta. */
  const ctxAyuda = $derived({
    tempWarnC: settings.alerts.tempConfiguredWarnC,
    tempCritC: settings.alerts.tempConfiguredCritC,
    vendorLimitC: disk.vendorTempLimitC,
    wearWarnPct: settings.alerts.wearWarnPercent,
    wearCritPct: settings.alerts.wearCritPercent
  });
  const ayudaTemp = $derived(ayudaMetrica("temperature", disk.temperatureC, ctxAyuda));
  const ayudaActividad = $derived(ayudaMetrica("activity", null, ctxAyuda, disk.activity));
  const ayudaDesgaste = $derived(ayudaMetrica("wear", disk.percentageUsed, ctxAyuda));
  const ayudaHoras = $derived(ayudaMetrica("powerOnHours", disk.powerOnHours, ctxAyuda));
  const actividad = $derived(actividadDetalle(disk.activity));

  /** Un disco "unreadable" que no sea NVMe puede deberse a que Windows Defender bloquea el comando
   *  de bajo nivel que `smartctl` necesita (J.56/ADR-043) — se comprueba solo en ese caso, nunca en
   *  cada carga de pantalla, porque implica lanzar PowerShell. */
  let defenderBloqueado = $state(false);
  let defenderReintentando = $state(false);
  let defenderResultado = $state<{ added: boolean; detail: string | null } | null>(null);

  $effect(() => {
    const podriaSerDefender = disk.unknownReason === "unreadable" && disk.deviceType !== "nvme";
    if (!podriaSerDefender) {
      defenderBloqueado = false;
      return;
    }
    let cancelado = false;
    void (async () => {
      try {
        const permitido = await checkSmartctlDefenderException();
        if (!cancelado) defenderBloqueado = !permitido;
      } catch {
        // Sin dato no se acusa a Defender: se deja el aviso genérico de "unreadable".
      }
    })();
    return () => {
      cancelado = true;
    };
  });

  async function reintentarDefender() {
    defenderReintentando = true;
    defenderResultado = null;
    try {
      const resultado = await addSmartctlDefenderException();
      defenderResultado = resultado;
      // El aviso sigue visible con el resultado aunque `added` sea `true`: `defenderBloqueado`
      // solo lo apaga la siguiente lectura SMART real que llegue fresca, no este resultado en sí
      // — si se apagara aquí, el mensaje de éxito desaparecería antes de que nadie lo leyera.
    } catch (cause) {
      defenderResultado = { added: false, detail: toAppError(cause).detail ?? null };
    } finally {
      defenderReintentando = false;
    }
  }

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
  /** Series mini de 24 h para las sparklines de las `MetricCard` (`07-detalle-disco.md` §3). */
  let mini = $state<Record<string, Punto[]>>({});

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

  $effect(() => {
    const id = disk.id;
    const ahora = new Date();
    const from = new Date(ahora.getTime() - 24 * 3600 * 1000).toISOString();
    const to = ahora.toISOString();
    for (const clave of ["temperature_celsius", "activity_percent", "percentage_used", "power_on_hours"]) {
      void (async () => {
        try {
          const s = await getMetricSeries({ deviceId: id, metricKey: clave, fromUtc: from, toUtc: to });
          mini = { ...mini, [clave]: s.points };
        } catch {
          // sin serie: la MetricCard no pinta sparkline
        }
      })();
    }
  });

  const resolutionLabel = $derived(serie ? t(`chart.resolution.${serie.resolution}`) : "");
  const frescura = $derived(formatAge(disk.lastReadAt));

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

  const ETIQUETA_REUTILIZADA: Record<string, string> = {
    temperature_celsius: "disk.temperature",
    activity_percent: "disk.activity",
    percentage_used: "disk.wear",
    power_on_hours: "disk.powerOnHours"
  };
  const etiquetaContador = (k: string) => t(ETIQUETA_REUTILIZADA[k] ?? `smart.counter.${k}`);
  const formatDelta = (d: number) => (d > 0 ? `+${d.toLocaleString(i18n.formatLocale)}` : String(d));
</script>

<div class="flex flex-col gap-5">
  <Card>
    <div class="flex flex-wrap items-center gap-4">
      <span
        class="grid size-14 shrink-0 place-items-center rounded-inner text-fg-onAccent"
        style="background: {tone.fg}"
      >
        <Icon name={busIcon(disk.deviceType)} size={24} />
      </span>
      <div class="flex min-w-0 flex-col gap-1">
        <div class="flex items-center gap-2">
          <h1 class="sdm-selectable sdm-display m-0 text-2xl">{disk.alias ?? disk.model}</h1>
          <StatusPill state={disk.state} label={t(`health.${disk.state}`)} icon="auto" />
        </div>
        <span class="sdm-selectable text-2xs text-fg-dim">
          {disk.model} · {disk.busType ?? disk.deviceType} · {maskSerial(disk.serialNumber)} · {disk.firmware ??
            t("common.notAvailable")}{#if frescura}
            · {t("common.updatedAgo", { value: frescura })}{/if}
        </span>
      </div>
      <div class="flex flex-1"></div>
      <Button variant="primary" onclick={() => goto("/tests")}>{t("disk.testThisDisk")}</Button>
    </div>
  </Card>

  {#if defenderBloqueado}
    <div class="flex flex-wrap items-center gap-3 rounded-inner bg-warn-soft p-4">
      <span
        class="grid size-[18px] shrink-0 place-items-center rounded-pill bg-warn text-2xs font-semibold text-white"
        >!</span
      >
      <span class="flex-1 text-xs leading-normal" style="text-wrap: pretty"
        >{t("disk.noSmartBlockedByDefender")}</span
      >
      <Button size="sm" loading={defenderReintentando} onclick={reintentarDefender}
        >{t("disk.retryFolderProtection")}</Button
      >
      {#if defenderResultado}
        <span class="basis-full text-2xs {defenderResultado.added ? 'text-ok' : 'text-warn'}">
          {defenderResultado.added
            ? t("disk.retryFolderProtectionSuccess")
            : t("disk.retryFolderProtectionFailed", { detail: defenderResultado.detail ?? "" })}
        </span>
      {/if}
    </div>
  {/if}

  <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
    <MetricCard
      label={t("disk.temperature")}
      icon="temp"
      value={disk.temperatureC !== null ? formatTemperature(disk.temperatureC) : null}
      state={tempState}
      series={mini["temperature_celsius"] ?? []}
      unidad="°C"
      ayuda={ayudaTemp}
      provenance={disk.provenance?.source ?? ""}
    />
    <MetricCard
      label={t("disk.activity")}
      icon="pulse"
      value={actividad.valor}
      secondary={actividad.secundario}
      series={mini["activity_percent"] ?? []}
      unidad="%"
      ayuda={ayudaActividad}
    />
    <MetricCard
      label={t("disk.wear")}
      icon="wear"
      value={disk.percentageUsed !== null ? formatPercent(disk.percentageUsed) : null}
      series={mini["percentage_used"] ?? []}
      unidad="%"
      ayuda={ayudaDesgaste}
    />
    <MetricCard
      label={t("disk.powerOnHours")}
      icon="clock"
      value={disk.powerOnHours !== null ? formatHours(disk.powerOnHours) : null}
      series={mini["power_on_hours"] ?? []}
      unidad="h"
      ayuda={ayudaHoras}
    />
  </div>

  <div class="grid gap-5 lg:grid-cols-[1.6fr_1fr]">
    <div class="flex min-h-[240px] flex-col gap-3">
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
          color={tempState === "warn" || tempState === "crit" ? "var(--sdm-warn)" : "var(--sdm-accent)"}
          warnThreshold={tempThresholds.warn}
          warnLabel={t("chart.tempWarnLabel", { value: formatTemperature(tempThresholds.warn) })}
          critThreshold={tempThresholds.crit}
          critLabel={t("chart.tempCritLabel", { value: formatTemperature(tempThresholds.crit) })}
          {resolutionLabel}
        />
      {:else if serieLoading}
        <span class="text-xs text-fg-dim">{t("common.loading")}</span>
      {/if}
    </div>

    <Card title={t("disk.counters")}>
      {#each disk.counters as counter (counter.metricKey)}
        <DataRow
          label={etiquetaContador(counter.metricKey)}
          value={formatearContador(counter.unit, counter.value)}
          delta={counter.delta != null ? formatDelta(counter.delta) : ""}
          deltaState={counter.deltaIsMeaningful ? "warn" : null}
        />
      {/each}

      {#if ia.activa && disk.counters.length > 0}
        <div>
          <Button
            variant="feature"
            full={false}
            disabled={ia.estaEnCurso(`smart:${disk.id}`)}
            onclick={explicarSmart}
          >
            <Icon name="sparkles" />
            {t("alerts.explainCta")}
          </Button>
        </div>
      {/if}
    </Card>
  </div>
</div>

<ExplicacionModal
  open={explicacion.open}
  fase={explicacion.fase}
  markdown={explicacion.markdown}
  modeloUsado={explicacion.modeloUsado}
  detalleRecortado={explicacion.detalleRecortado}
  sinVolcado={explicacion.sinVolcado}
  sinSuceso={explicacion.sinSuceso}
  error={explicacion.error}
  textoRevision={explicacion.textoRevision}
  fragmentos={explicacion.fragmentos}
  onclose={explicacion.cerrar}
  oncancel={explicacion.cerrar}
  onretry={() => void explicacion.reintentar()}
  onconfirmar={() => void explicacion.confirmarPreview()}
  onenviarigual={() => void explicacion.enviarIgual()}
  onquitarfragmentos={() => void explicacion.quitarFragmentos()}
/>
