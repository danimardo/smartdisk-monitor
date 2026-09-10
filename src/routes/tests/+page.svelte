<script lang="ts">
  /** Pantalla de pruebas y diagnóstico (US-040/041/042, `docs/product-specification.md` §6).
   *
   *  Tres tarjetas (una por tipo de prueba), la prueba en curso si la hay, y el historial. El
   *  historial llega listo desde `load` (constitución §XIV); las actualizaciones posteriores
   *  llegan por `test:progress` (ADR-015) — esta pantalla es la única que lo escucha, porque es la
   *  única que lo necesita.
   *
   *  El objetivo (disco/volumen) se elige aquí: el backend decide la exclusión mutua real por
   *  disco físico subyacente (`docs/open-questions.md` J.29), esta pantalla solo refleja lo que ve
   *  en el historial para no ofrecer una acción que el backend va a rechazar.
   */
  import { onMount } from "svelte";
  import {
    BenchmarkResults,
    Button,
    Card,
    ConfirmDialog,
    EmptyState,
    Icon,
    ProgressBar,
    Select,
    StatusPill
  } from "$lib/components";
  import {
    cancelTest,
    getDeviceDetail,
    on,
    runChkdskScan,
    runSmartShortTest,
    startBenchmark,
    toAppError
  } from "$lib/api";
  import { formatDateTime, formatIops, formatMbPerSecond } from "$lib/design/format";
  import { healthToken } from "$lib/design/health";
  import { healthIcon, testIcon, type IconName } from "$lib/design/icons";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import type { TestRun } from "$lib/api";
  import type { AppError, HealthState } from "$lib/design/types";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  /** Mismo guardián que usa `/` (constitución §XIV): esta pantalla puede ser la primera que se
   *  visite, así que no puede asumir que el panel general ya rellenó `app.devices`. */
  $effect(() => {
    if (app.loadedAt === null) {
      app.devices = data.inventory.devices;
      app.excluded = data.inventory.excluded;
      app.sources = data.inventory.sources;
      app.loadedAt = new Date().toISOString();
    }
  });

  let testRuns = $state<TestRun[]>([]);
  let testRunsLoadedAt = $state<string | null>(null);
  $effect(() => {
    if (testRunsLoadedAt === null) {
      testRuns = data.testRuns;
      testRunsLoadedAt = new Date().toISOString();
    }
  });

  onMount(() => {
    let unsubscribe: (() => void) | undefined;
    void (async () => {
      unsubscribe = await on("test:progress", (p) => {
        const existe = testRuns.some((r) => r.id === p.testRun.id);
        testRuns = existe
          ? testRuns.map((r) => (r.id === p.testRun.id ? p.testRun : r))
          : [p.testRun, ...testRuns];
      });
    })();
    return () => unsubscribe?.();
  });

  const ESTADOS_ACTIVOS: readonly string[] = ["pending", "running", "cancelling"];
  const historialOrdenado = $derived([...testRuns].sort((a, b) => b.startedAt.localeCompare(a.startedAt)));
  const pruebaActiva = $derived(historialOrdenado.find((r) => ESTADOS_ACTIVOS.includes(r.status)) ?? null);

  function estaOcupado(deviceIdObjetivo: string | null, volumeIdObjetivo: string | null): boolean {
    return testRuns.some(
      (r) =>
        ESTADOS_ACTIVOS.includes(r.status) &&
        ((deviceIdObjetivo && r.deviceId === deviceIdObjetivo) ||
          (volumeIdObjetivo && r.volumeId === volumeIdObjetivo))
    );
  }

  /* -------------------------------------------------------------- selección de disco y volumen */

  let deviceId = $state<string | null>(null);
  $effect(() => {
    if (deviceId === null || !app.devices.some((d) => d.id === deviceId)) {
      deviceId = app.devices[0]?.id ?? null;
    }
  });
  const dispositivo = $derived(app.devices.find((d) => d.id === deviceId) ?? null);

  let volumeId = $state<string | null>(null);
  const volumenes = $derived(dispositivo?.volumes ?? []);
  $effect(() => {
    if (!volumenes.some((v) => v.id === volumeId)) volumeId = volumenes[0]?.id ?? null;
  });
  const volumen = $derived(volumenes.find((v) => v.id === volumeId) ?? null);

  const opcionesDispositivo = $derived(app.devices.map((d) => ({ id: d.id, label: d.alias || d.model })));
  const opcionesVolumen = $derived(
    volumenes.map((v) => ({
      id: v.id,
      label: `${v.label} (${v.driveLetters.join(", ") || t("common.notAvailable")})`
    }))
  );

  /* -------------------------------------------------------- compatibilidad del autotest SMART */

  let autotestDisponible = $state(false);
  let autotestMotivoClave = $state<string | null>(null);
  $effect(() => {
    const id = deviceId;
    if (!id) {
      autotestDisponible = false;
      autotestMotivoClave = null;
      return;
    }
    let cancelado = false;
    void (async () => {
      try {
        const detalle = await getDeviceDetail(id);
        if (cancelado) return;
        const capacidad = detalle.capabilities.find((c) => c.key === "self_test_short");
        autotestDisponible = capacidad?.available ?? false;
        autotestMotivoClave = capacidad?.reasonKey ?? null;
      } catch {
        if (!cancelado) {
          autotestDisponible = false;
          autotestMotivoClave = null;
        }
      }
    })();
    return () => {
      cancelado = true;
    };
  });

  /* -------------------------------------------------------------------------- diálogo y arranque */

  type TipoPrueba = "benchmark" | "chkdsk" | "autotest";
  let dialogoAbierto = $state<TipoPrueba | null>(null);
  let iniciando = $state(false);
  let accionError = $state<AppError | null>(null);

  const letraVolumen = $derived(volumen?.driveLetters[0]?.replace(":", "") ?? "");
  const comandoChkdsk = $derived(letraVolumen ? `chkdsk ${letraVolumen}: /scan` : "");

  async function confirmar() {
    const tipo = dialogoAbierto;
    if (!tipo) return;
    iniciando = true;
    try {
      if (tipo === "benchmark" && volumen) {
        await startBenchmark({ volumeId: volumen.id });
      } else if (tipo === "chkdsk" && volumen) {
        await runChkdskScan(volumen.id);
      } else if (tipo === "autotest" && deviceId) {
        await runSmartShortTest(deviceId);
      }
      accionError = null;
    } catch (cause) {
      accionError = toAppError(cause);
    } finally {
      iniciando = false;
      dialogoAbierto = null;
    }
  }

  async function cancelar(testRunId: string) {
    try {
      await cancelTest(testRunId);
      accionError = null;
    } catch (cause) {
      accionError = toAppError(cause);
    }
  }

  /* ---------------------------------------------------------------------------- presentación */

  const ESTADO_A_SALUD: Record<string, HealthState> = {
    pending: "unknown",
    running: "unknown",
    cancelling: "unknown",
    completed: "ok",
    failed: "crit",
    cancelled: "warn",
    interrupted: "warn"
  };

  const TIPO_LABEL_KEY: Record<string, string> = {
    benchmark: "tests.type.benchmark",
    chkdsk_scan: "tests.type.chkdsk",
    smart_short: "tests.type.autotest"
  };

  /** Icono del cuadrado de cada tarjeta de prueba (`04-pruebas.md` §3). Las tarjetas se distinguen
   *  hoy solo por el título; el cuadrado da un ancla visual. */
  const ICONO_TARJETA: Record<TipoPrueba, IconName> = {
    benchmark: "flask",
    chkdsk: "shield",
    autotest: "bolt"
  };

  function objetivoLabel(r: TestRun): string {
    if (r.volumeId) {
      const v = app.devices.flatMap((d) => d.volumes).find((vol) => vol.id === r.volumeId);
      return v
        ? `${v.label} (${v.driveLetters.join(", ") || t("common.notAvailable")})`
        : t("common.notAvailable");
    }
    if (r.deviceId) {
      const d = app.devices.find((dev) => dev.id === r.deviceId);
      return d ? d.alias || d.model : t("common.notAvailable");
    }
    return t("common.notAvailable");
  }

  function resultadoLabel(r: TestRun): string {
    if (ESTADOS_ACTIVOS.includes(r.status)) {
      return r.progressPercent === null
        ? t("tests.history.inProgressIndeterminate")
        : t("tests.history.inProgress", { percent: r.progressPercent });
    }
    if (r.result?.stoppedReason) {
      return t(`tests.stoppedReason.${r.result.stoppedReason}`);
    }
    return t("common.notAvailable");
  }

  const progresoActivoTexto = $derived.by(() => {
    if (!pruebaActiva) return "";
    return pruebaActiva.progressPercent === null
      ? t("tests.active.indeterminate")
      : t("tests.active.progress", { percent: pruebaActiva.progressPercent });
  });

  /** Resumen de una línea para el historial: el caudal secuencial de lectura y los IOPS 4K de
   *  lectura, que es lo que la gente compara de un vistazo. */
  function resumenBenchmark(b: NonNullable<NonNullable<TestRun["result"]>["benchmark"]>): string {
    const seq = b.rows.find((f) => f.profile === "seq1m_q8" && f.direction === "read");
    const rnd = b.rows.find((f) => f.profile === "rnd4k_q32" && f.direction === "read");
    const partes: string[] = [];
    if (seq) partes.push(`${t("tests.benchmark.col.throughput")} ${formatMbPerSecond(seq.mbPerSecond)}`);
    if (rnd) partes.push(`${t("tests.benchmark.col.iops")} ${formatIops(rnd.iops)}`);
    return partes.join(" · ");
  }
</script>

<div class="flex flex-col gap-5 p-5">
  <h1 class="m-0 text-2xl font-semibold tracking-tight">{t("nav.tests")}</h1>

  {#if app.devices.length === 0}
    <EmptyState kind="empty" title={t("dashboard.noDevices")} body={t("dashboard.noDevicesHint")} />
  {:else}
    <div class="flex flex-wrap gap-4">
      <div class="w-64">
        <Select
          label={t("tests.picker.device")}
          value={deviceId ?? ""}
          options={opcionesDispositivo}
          onchange={(v: string) => (deviceId = v)}
        />
      </div>
      <div class="w-64">
        <Select
          label={t("tests.picker.volume")}
          value={volumeId ?? ""}
          options={opcionesVolumen}
          disabled={opcionesVolumen.length === 0}
          onchange={(v: string) => (volumeId = v)}
        />
      </div>
    </div>

    {#if accionError}
      <div class="rounded-inner bg-crit-soft p-4 text-sm text-crit" role="alert">
        {t(accionError.messageKey, accionError.messageVars)}
      </div>
    {/if}

    {#if pruebaActiva}
      <Card padding="md">
        <div class="flex flex-wrap items-center gap-3">
          <span
            class="grid size-[38px] shrink-0 place-items-center rounded-inner bg-accent-soft text-accent-fg"
          >
            <Icon name={testIcon[pruebaActiva.type] ?? "flask"} size={20} />
          </span>
          <div class="flex min-w-0 flex-col gap-0.5">
            <StatusPill state="unknown" label={t("tests.active.title")} />
            <span class="sdm-display truncate text-2xl"
              >{t(TIPO_LABEL_KEY[pruebaActiva.type] ?? "common.notAvailable")}</span
            >
          </div>
          <div class="flex-1"></div>
          {#if pruebaActiva.progressPercent !== null}
            <span class="sdm-num sdm-display text-metric leading-none min-[1100px]:text-display">
              {pruebaActiva.progressPercent}<span class="text-xl text-fg-faint">%</span>
            </span>
          {/if}
          <Button
            variant="danger"
            size="sm"
            disabled={pruebaActiva.status === "cancelling"}
            disabledReason={pruebaActiva.status === "cancelling" ? t("tests.status.cancelling") : ""}
            onclick={() => cancelar(pruebaActiva.id)}
          >
            {t("tests.active.cancel")}
          </Button>
        </div>

        <span class="text-xs text-fg-dim">{objetivoLabel(pruebaActiva)}</span>

        <ProgressBar
          emphasis="display"
          value={pruebaActiva.progressPercent ?? 0}
          indeterminate={pruebaActiva.progressPercent === null}
          caption={pruebaActiva.status === "cancelling"
            ? t("tests.status.cancelling")
            : t(TIPO_LABEL_KEY[pruebaActiva.type] ?? "common.notAvailable")}
          trailing={progresoActivoTexto}
        />

        {#if pruebaActiva.type === "benchmark" && pruebaActiva.result?.benchmark}
          <BenchmarkResults result={pruebaActiva.result.benchmark} running />
        {/if}

        <div class="flex gap-3 rounded-inner bg-warn-soft p-4">
          <span class="shrink-0 text-warn"><Icon name="alert" size={16} /></span>
          <span class="text-xs leading-normal" style="text-wrap: pretty">{t("tests.active.warning")}</span>
        </div>
      </Card>
    {/if}

    <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
      <Card title={t("tests.cards.benchmark.title")}>
        {#snippet leading()}
          <span class="grid size-[38px] shrink-0 place-items-center rounded-inner bg-glass-3 text-fg-dim">
            <Icon name={ICONO_TARJETA.benchmark} size={19} />
          </span>
        {/snippet}
        <p class="m-0 min-h-[58px] text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
          {t("tests.cards.benchmark.desc")}
        </p>
        {#if volumen && estaOcupado(null, volumen.id)}
          <StatusPill state="unknown" label={t("tests.chip.running")} />
        {:else}
          <StatusPill state="ok" label={t("tests.chip.available")} />
        {/if}
        <Button
          variant="secondary"
          disabled={!volumen || iniciando || estaOcupado(null, volumen?.id ?? null)}
          onclick={() => (dialogoAbierto = "benchmark")}
        >
          {t("tests.cta.configure")}
        </Button>
      </Card>

      <Card title={t("tests.cards.chkdsk.title")}>
        {#snippet leading()}
          <span class="grid size-[38px] shrink-0 place-items-center rounded-inner bg-glass-3 text-fg-dim">
            <Icon name={ICONO_TARJETA.chkdsk} size={19} />
          </span>
        {/snippet}
        <p class="m-0 min-h-[58px] text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
          {t("tests.cards.chkdsk.desc")}
        </p>
        {#if volumen && estaOcupado(null, volumen.id)}
          <StatusPill state="unknown" label={t("tests.chip.running")} />
        {:else if volumen && volumen.chkdskAvailable}
          <StatusPill state="ok" label={t("tests.chip.available")} />
        {:else}
          <StatusPill state="unknown" label={t("tests.chip.unsupported")} />
        {/if}
        <Button
          variant="secondary"
          disabled={!volumen ||
            !volumen.chkdskAvailable ||
            iniciando ||
            estaOcupado(null, volumen?.id ?? null)}
          disabledReason={volumen && !volumen.chkdskAvailable ? t("tests.chkdskUnsupportedReason") : ""}
          onclick={() => (dialogoAbierto = "chkdsk")}
        >
          {t("tests.cta.configure")}
        </Button>
      </Card>

      <Card title={t("tests.cards.autotest.title")}>
        {#snippet leading()}
          <span class="grid size-[38px] shrink-0 place-items-center rounded-inner bg-glass-3 text-fg-dim">
            <Icon name={ICONO_TARJETA.autotest} size={19} />
          </span>
        {/snippet}
        <p class="m-0 min-h-[58px] text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
          {t("tests.cards.autotest.desc")}
        </p>
        {#if deviceId && estaOcupado(deviceId, null)}
          <StatusPill state="unknown" label={t("tests.chip.running")} />
        {:else if autotestDisponible}
          <StatusPill state="ok" label={t("tests.chip.available")} />
        {:else}
          <StatusPill state="unknown" label={t("tests.chip.unsupported")} />
        {/if}
        <Button
          variant="secondary"
          disabled={!deviceId || !autotestDisponible || iniciando || estaOcupado(deviceId, null)}
          disabledReason={autotestMotivoClave ? t(autotestMotivoClave) : ""}
          onclick={() => (dialogoAbierto = "autotest")}
        >
          {t("tests.cta.configure")}
        </Button>
      </Card>
    </div>

    <Card title={t("tests.history.title")}>
      {#if historialOrdenado.length === 0}
        <span class="text-xs text-fg-dim">{t("tests.history.empty")}</span>
      {:else}
        <div class="flex flex-col">
          {#each historialOrdenado as run (run.id)}
            {@const salud = ESTADO_A_SALUD[run.status] ?? "unknown"}
            {@const tabla = run.type === "benchmark" ? (run.result?.benchmark ?? null) : null}
            <div class="border-t border-hairline first:border-t-0">
              <div class="grid grid-cols-[28px_118px_1fr_1fr_1fr_auto] items-center gap-4 py-2">
                <span
                  class="grid size-[28px] shrink-0 place-items-center rounded-nav"
                  style="background: {healthToken[salud].soft}; color: {healthToken[salud].fg}"
                >
                  <Icon name={healthIcon[salud]} size={14} label={t(`tests.status.${run.status}`)} />
                </span>
                <span class="text-xs tabular-nums text-fg-dim">{formatDateTime(run.startedAt)}</span>
                <span class="text-xs font-semibold"
                  >{t(TIPO_LABEL_KEY[run.type] ?? "common.notAvailable")}</span
                >
                <span class="text-xs text-fg-dim">{objetivoLabel(run)}</span>
                <span class="text-xs text-fg-dim">
                  {#if tabla && tabla.rows.length > 0}
                    {resumenBenchmark(tabla)}
                  {:else}
                    {resultadoLabel(run)}
                  {/if}
                </span>
                <StatusPill state={salud} label={t(`tests.status.${run.status}`)} />
              </div>
              {#if tabla && (tabla.rows.length > 0 || tabla.notRun.length > 0)}
                <details class="pb-2" open={run.id === historialOrdenado[0].id}>
                  <summary class="cursor-pointer text-2xs font-semibold text-fg-dim">
                    {t("tests.benchmark.showTable")}
                  </summary>
                  <div class="mt-2">
                    <BenchmarkResults result={tabla} />
                  </div>
                </details>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </Card>
  {/if}
</div>

<ConfirmDialog
  open={dialogoAbierto === "benchmark"}
  title={t("tests.confirm.benchmark.title")}
  body={t("tests.confirm.benchmark.body")}
  impact={t("tests.confirm.benchmark.impact")}
  confirmLabel={t("tests.confirm.benchmark.confirmLabel")}
  onconfirm={confirmar}
  oncancel={() => (dialogoAbierto = null)}
/>

<ConfirmDialog
  open={dialogoAbierto === "chkdsk"}
  title={t("tests.confirm.chkdsk.title", { letter: letraVolumen })}
  body={t("tests.confirm.chkdsk.body")}
  command={comandoChkdsk}
  impact={t("tests.confirm.chkdsk.impact")}
  confirmLabel={t("tests.confirm.chkdsk.confirmLabel")}
  onconfirm={confirmar}
  oncancel={() => (dialogoAbierto = null)}
/>

<ConfirmDialog
  open={dialogoAbierto === "autotest"}
  title={t("tests.confirm.autotest.title")}
  body={t("tests.confirm.autotest.body")}
  impact={t("tests.confirm.autotest.impact")}
  confirmLabel={t("tests.confirm.autotest.confirmLabel")}
  onconfirm={confirmar}
  oncancel={() => (dialogoAbierto = null)}
/>
