<script lang="ts">
  /** Panel general (US-012), v3 (`cambios/01-panel-general.md`). Responde «¿tengo un problema?» de
   *  un vistazo: `HeroPanel` con el disco que necesita atención, rejilla de `DiskCard`, y al pie los
   *  sucesos del sistema y el reparto de estados.
   *
   *  El inventario llega del `load` del layout (constitución §XIV) y vive en el store; las series de
   *  temketaratura se piden **perezosamente** por disco visible tras el primer render (v3, `open-questions.md`):
   *  el panel pinta enseguida y cada sparkline aparece cuando llega su serie. */
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { Card, DiskCard, EmptyState, EventRow, HeroPanel, Icon } from "$lib/components";
  import { getMetricSeries, getSystemEvents, refreshNow } from "$lib/api";
  import { healthToken, selectHeroDisk } from "$lib/design/health";
  import { healthIcon } from "$lib/design/icons";
  import { formatHours, formatPercent } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import type { SystemEventShape } from "$lib/api/schemas";
  import type { HealthState } from "$lib/design/types";

  const devices = $derived(app.devices);
  const ready = $derived(app.loadedAt !== null);

  /** Con más de 12 discos la `DiskCard` pierde la sparkline de cabecera (`ui-design.md` §7): con
   *  tantas tarjetas la miniatura no aporta y el coste de render sí importa (SC-006). */
  const conSparklines = $derived(devices.length <= 12);

  const alerts = $derived(app.alerts);
  const heroDisk = $derived(selectHeroDisk(devices, alerts));

  const heroAlert = $derived(
    heroDisk
      ? (alerts.find((a) => a.deduplicationKey.includes(`device:${heroDisk.id}`) && a.status === "active") ??
          null)
      : null
  );

  const heroFacts = $derived(
    heroDisk
      ? [
          {
            label: t("disk.wear"),
            value: heroDisk.percentageUsed !== null ? formatPercent(heroDisk.percentageUsed) : null,
            icon: "wear" as const
          },
          {
            label: t("disk.activity"),
            value: heroDisk.activityPercent !== null ? formatPercent(heroDisk.activityPercent) : null,
            icon: "pulse" as const
          },
          {
            label: t("disk.powerOnHours"),
            value: heroDisk.powerOnHours !== null ? formatHours(heroDisk.powerOnHours) : null,
            icon: "clock" as const
          },
          {
            label: t("disk.capacity"),
            value:
              heroDisk.volumes[0]?.capacityBytes != null
                ? formatPercent(
                    ((heroDisk.volumes[0].capacityBytes - (heroDisk.volumes[0].freeBytes ?? 0)) /
                      heroDisk.volumes[0].capacityBytes) *
                      100
                  )
                : null,
            icon: "diskStack" as const
          }
        ]
      : []
  );

  /* ---------------------------------------------------------------- series perezosas */

  async function pedirSerie(deviceId: string) {
    if (app.temperatureSeries[deviceId]) return;
    // marca temprana para no lanzar dos peticiones a la vez
    app.temperatureSeries = { ...app.temperatureSeries, [deviceId]: [] };
    try {
      const ahora = new Date();
      const serie = await getMetricSeries({
        deviceId,
        metricKey: "temperature_celsius",
        fromUtc: new Date(ahora.getTime() - 24 * 3600 * 1000).toISOString(),
        toUtc: ahora.toISOString()
      });
      app.temperatureSeries = { ...app.temperatureSeries, [deviceId]: serie.points };
    } catch {
      // Un fallo por disco degrada solo esa sparkline: no se pinta y ya está.
    }
  }

  $effect(() => {
    if (!ready) return;
    const objetivo = new Set<string>();
    if (heroDisk) objetivo.add(heroDisk.id);
    if (conSparklines) for (const d of devices) if (d.state !== "unknown") objetivo.add(d.id);
    for (const id of objetivo) void pedirSerie(id);
  });

  const heroSeries = $derived(heroDisk ? (app.temperatureSeries[heroDisk.id] ?? []) : []);
  const heroThreshold = $derived(heroDisk?.vendorTempLimitC ?? null);

  /* ---------------------------------------------------------------- sucesos y reparto */

  let sucesos = $state<SystemEventShape[]>([]);
  onMount(() => {
    void (async () => {
      try {
        const page = await getSystemEvents({ limit: 3 });
        sucesos = page.events;
      } catch {
        // Sin sucesos: el bloque muestra su estado vacío.
      }
    })();
  });

  const reparto = $derived.by(() => {
    const orden: HealthState[] = ["crit", "warn", "unknown", "ok"];
    const total = devices.length || 1;
    return orden
      .map((estado) => ({
        estado,
        n: devices.filter((d) => d.state === estado).length,
        pct: (devices.filter((d) => d.state === estado).length / total) * 100
      }))
      .filter((x) => x.n > 0);
  });

  function abrir(id: string) {
    void goto(`/disks/${id}`);
  }
</script>

{#if ready && devices.length === 0}
  <div class="mx-auto flex max-w-[520px] flex-1 items-center">
    <EmptyState
      kind="empty"
      title={t("dashboard.noDevices")}
      body={t("dashboard.noDevicesHint")}
      actionLabel={t("dashboard.noDevicesCta")}
      onaction={() => refreshNow("all").catch(() => {})}
    />
  </div>
{:else}
  <div class="flex min-h-0 flex-1 flex-col gap-5">
    {#if heroDisk}
      <HeroPanel
        disk={heroDisk}
        series={heroSeries}
        threshold={heroThreshold}
        loading={!ready}
        alertId={heroAlert?.id ?? null}
        explanation={heroAlert ? t(`alert.rule.${heroAlert.ruleKey}.summary`) : ""}
        facts={heroFacts}
        onopen={abrir}
        onviewalert={() => goto("/alerts")}
      />
    {/if}

    <div class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(272px, 1fr))">
      {#each devices as disk (disk.id)}
        <DiskCard
          {disk}
          href={`/disks/${disk.id}`}
          temperatureSeries={conSparklines ? (app.temperatureSeries[disk.id] ?? []) : []}
        />
      {/each}
    </div>

    <div class="grid gap-5 max-[900px]:grid-cols-1" style="grid-template-columns: 1fr 300px">
      <Card title={t("dashboard.events.title")}>
        {#if sucesos.length}
          {#each sucesos as ev (ev.id)}
            <EventRow
              level={ev.level}
              message={ev.message}
              provider={ev.provider}
              eventId={ev.eventId}
              occurredAt={ev.occurredAt}
              mappingConfidence={ev.mappingConfidence}
            />
          {/each}
        {:else}
          <span class="text-xs text-fg-dim">{t("dashboard.events.empty")}</span>
        {/if}
      </Card>

      <Card title={t("dashboard.spread.title")}>
        {#if reparto.length}
          <div class="flex h-2 overflow-hidden rounded-pill bg-glass-3">
            {#each reparto as r}
              <div style="width: {r.pct}%; background: {healthToken[r.estado].fg}"></div>
            {/each}
          </div>
          <div class="flex flex-col gap-1.5">
            {#each reparto as r}
              <span class="flex items-center gap-2 text-xs" style="color: {healthToken[r.estado].fg}">
                <Icon name={healthIcon[r.estado]} size={13} />
                <span class="flex-1 text-fg-dim">{t(`health.${r.estado}`)}</span>
                <span class="sdm-num font-semibold">{r.n}</span>
              </span>
            {/each}
          </div>
        {:else}
          <span class="text-xs text-fg-dim">{t("common.loading")}</span>
        {/if}
      </Card>
    </div>
  </div>
{/if}
