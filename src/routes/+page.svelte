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
  import { getMetricSeries, getSettings, getSystemEvents, refreshNow } from "$lib/api";
  import { estadoConAlertas, healthToken, selectHeroDisk } from "$lib/design/health";
  import { healthIcon } from "$lib/design/icons";
  import { formatHours, formatPercent, formatSpanShort } from "$lib/design/format";
  import { ultimoTramoVisible, type Punto } from "$lib/design/series";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import type { SystemEventShape } from "$lib/api/schemas";
  import type { HealthState } from "$lib/design/types";

  /** El `state` que manda el backend solo mira la frescura de SMART; las alertas vigentes se funden
   *  aquí (`estadoConAlertas`, `B.1`) antes de que nada lo lea —Hero, tarjetas, reparto—. Un
   *  `unknown` sigue siendo `unknown` (gris «sin datos SMART»); que cuente para «necesitan
   *  atención» lo decide `estadoParaRecuento` en el chrome, no aquí. */
  const devices = $derived(app.devices.map((d) => ({ ...d, state: estadoConAlertas(d, app.alerts) })));
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

  /** Sin lectura SMART fresca los hechos derivados de SMART no se enseñan (serían valores viejos):
   *  misma regla que la `DiskCard` (boceto §4). */
  const heroSinSmartFresco = $derived(heroDisk?.unknownReason != null);

  const heroFacts = $derived.by(() => {
    if (!heroDisk) return [];
    const smart = <T,>(v: T | null): T | null => (heroSinSmartFresco ? null : v);
    return [
      {
        // Boceto: la salud del firmware (autoevaluación SMART) es el primer hecho del Hero (ADR-041).
        label: t("disk.firmwareHealth"),
        value: smart(
          heroDisk.smartHealthPassed == null
            ? null
            : t(heroDisk.smartHealthPassed ? "disk.firmwareHealthOk" : "disk.firmwareHealthFail")
        ),
        state: heroDisk.smartHealthPassed === false ? ("crit" as const) : null,
        icon: "shield" as const
      },
      {
        label: t("disk.wear"),
        value: smart(heroDisk.percentageUsed !== null ? formatPercent(heroDisk.percentageUsed) : null),
        icon: "wear" as const
      },
      {
        label: t("disk.activity"),
        value: smart(heroDisk.activityPercent !== null ? formatPercent(heroDisk.activityPercent) : null),
        icon: "pulse" as const
      },
      {
        label: t("disk.powerOnHours"),
        value: smart(heroDisk.powerOnHours !== null ? formatHours(heroDisk.powerOnHours) : null),
        icon: "clock" as const
      }
    ];
  });

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

  /** El gráfico del panel enseña **solo el último tramo sin cortes** (`ultimoTramoVisible`): con la
   *  app parada a ratos, la ventana fija de 24 h sale a rayas; así vuelve a ser una onda y su ancho
   *  se adapta a lo que hay. El pie del Hero dice cuánto abarca. */
  const heroVentana = $derived(
    ultimoTramoVisible(heroDisk ? (app.temperatureSeries[heroDisk.id] ?? []) : [])
  );
  const heroSeries = $derived(heroVentana.points);
  const heroWindowLabel = $derived(
    heroVentana.points.length >= 2
      ? t("dashboard.hero.window", { span: formatSpanShort(heroVentana.hasta - heroVentana.desde) })
      : ""
  );
  const heroThreshold = $derived(heroDisk?.vendorTempLimitC ?? null);

  function serieVisibleTarjeta(deviceId: string): Punto[] {
    return conSparklines ? ultimoTramoVisible(app.temperatureSeries[deviceId] ?? []).points : [];
  }

  /** Umbrales para el veredicto del tooltip de cada métrica de la tarjeta. Se piden una vez, sin
   *  bloquear el pintado: hasta que llegan, `metricHelp` usa los de fábrica (que son los que casi
   *  todo el mundo tiene), y como el tooltip solo aparece al pasar el ratón, no se ve el cambio. */
  let umbrales = $state<
    { tempWarnC: number; tempCritC: number; wearWarnPct: number; wearCritPct: number } | undefined
  >();
  onMount(() => {
    void getSettings()
      .then((s) => {
        umbrales = {
          tempWarnC: s.alerts.tempConfiguredWarnC,
          tempCritC: s.alerts.tempConfiguredCritC,
          wearWarnPct: s.alerts.wearWarnPercent,
          wearCritPct: s.alerts.wearCritPercent
        };
      })
      .catch(() => {});
  });

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

  /** Las cuatro categorías, **siempre las cuatro** aunque alguna esté a cero (boceto): un «críticos: 0»
   *  informa. Orden del boceto: correctos, con advertencia, críticos, sin datos SMART. */
  const reparto = $derived.by(() => {
    const orden: HealthState[] = ["ok", "warn", "crit", "unknown"];
    const total = devices.length || 1;
    return orden.map((estado) => {
      const n = devices.filter((d) => d.state === estado).length;
      return { estado, n, pct: (n / total) * 100 };
    });
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
        windowLabel={heroWindowLabel}
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
          {umbrales}
          href={`/disks/${disk.id}`}
          temperatureSeries={serieVisibleTarjeta(disk.id)}
        />
      {/each}
    </div>

    <!-- `minmax(0, 1fr)` y no `1fr`: con `1fr` (= `minmax(auto, 1fr)`) el `min-content` de un mensaje
         de suceso largo fuerza la columna más ancha que la ventana y empuja «Reparto de estados»
         fuera de la vista (obligaba a scroll horizontal). Con `minmax(0, …)` la columna cede. -->
    <div class="grid gap-5 max-[900px]:grid-cols-1" style="grid-template-columns: minmax(0, 1fr) 300px">
      <Card title={t("dashboard.events.title")}>
        {#snippet action()}
          <a href="/events" class="text-xs font-medium text-accent hover:underline">{t("common.viewAll")}</a>
        {/snippet}
        {#if sucesos.length}
          {#each sucesos as ev (ev.id)}
            <EventRow
              level={ev.level}
              message={ev.message}
              provider={ev.provider}
              eventId={ev.eventId}
              occurredAt={ev.occurredAt}
              mappingConfidence={ev.mappingConfidence}
              href="/events?focus={ev.id}"
            />
          {/each}
        {:else}
          <span class="text-xs text-fg-dim">{t("dashboard.events.empty")}</span>
        {/if}
      </Card>

      <Card title={t("dashboard.spread.title")}>
        {#if ready}
          <div class="flex flex-col gap-1.5">
            {#each reparto as r (r.estado)}
              <span class="flex items-center gap-2 text-xs" style="color: {healthToken[r.estado].fg}">
                <Icon name={healthIcon[r.estado]} size={13} />
                <span class="flex-1 text-fg-dim">{t(`health.${r.estado}`)}</span>
                <span class="sdm-num font-semibold">{r.n}</span>
              </span>
            {/each}
          </div>
          <!-- Barra de proporción al pie (boceto): la fila inferior estira las dos tarjetas a la
               misma altura, así que `mt-auto` la ancla abajo. Solo tramos con algún disco. -->
          <div class="mt-auto flex h-2 gap-0.5 overflow-hidden rounded-pill bg-glass-3">
            {#each reparto.filter((r) => r.n > 0) as r (r.estado)}
              <div style="width: {r.pct}%; background: {healthToken[r.estado].fg}"></div>
            {/each}
          </div>
        {:else}
          <span class="text-xs text-fg-dim">{t("common.loading")}</span>
        {/if}
      </Card>
    </div>
  </div>
{/if}
