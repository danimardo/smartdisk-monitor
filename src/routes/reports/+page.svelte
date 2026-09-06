<script lang="ts">
  /** Pantalla de informes y diagnóstico (US-050/051, `docs/product-specification.md` §9).
   *
   *  Sin boceto aprobado (a diferencia de la de pruebas): compuesta con el catálogo actual según
   *  `docs/ui-design.md` Apéndice C, con autorización explícita del usuario para este enfoque.
   *
   *  El destino de cada exportación lo elige el usuario con el diálogo nativo de guardado
   *  (ADR-031, `chooseSavePath`), nunca una ruta que construya esta pantalla.
   */
  import { Button, Card, DateRangePicker, EmptyState, Icon, SegmentedControl, Switch } from "$lib/components";
  import {
    chooseSavePath,
    createDiagnosticZip,
    exportReport,
    previewDiagnosticZip,
    toAppError
  } from "$lib/api";
  import { formatBytes } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import type { DiagnosticPreview } from "$lib/api";
  import type { AppError } from "$lib/design/types";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  $effect(() => {
    if (app.loadedAt === null) {
      app.devices = data.inventory.devices;
      app.excluded = data.inventory.excluded;
      app.sources = data.inventory.sources;
      app.loadedAt = new Date().toISOString();
    }
  });

  /* -------------------------------------------------------------------------------- intervalo */

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

  /* -------------------------------------------------------------------------- discos incluidos */

  let seleccion = $state<Record<string, boolean>>({});
  $effect(() => {
    for (const d of app.devices) {
      if (!(d.id in seleccion)) seleccion[d.id] = true;
    }
  });

  function deviceIdsActuales(): string[] | null {
    const activos = app.devices.filter((d) => seleccion[d.id] ?? true).map((d) => d.id);
    return activos.length === app.devices.length ? null : activos;
  }

  let includeSerials = $state(false);

  /* ---------------------------------------------------------------------------- exportaciones */

  type Formato = "csv" | "json" | "html";
  const EXTENSION: Record<Formato, string> = { csv: "csv", json: "json", html: "html" };
  const NOMBRE_FILTRO: Record<Formato, string> = {
    csv: t("reports.format.csv"),
    json: t("reports.format.json"),
    html: t("reports.format.html")
  };

  let exportando = $state<Formato | null>(null);
  let exportError = $state<AppError | null>(null);
  let ultimaRutaExportada = $state<string | null>(null);

  async function exportar(formato: Formato) {
    exportando = formato;
    exportError = null;
    try {
      const destino = await chooseSavePath({
        defaultFileName: `informe-smartdisk.${EXTENSION[formato]}`,
        filterName: NOMBRE_FILTRO[formato],
        extensions: [EXTENSION[formato]]
      });
      if (!destino) return;
      const { from, to } = calcularIntervalo(rango, customFrom, customTo);
      const ruta = await exportReport({
        format: formato,
        fromUtc: from,
        toUtc: to,
        deviceIds: deviceIdsActuales(),
        includeSerials,
        destinationPath: destino
      });
      ultimaRutaExportada = ruta;
    } catch (cause) {
      exportError = toAppError(cause);
    } finally {
      exportando = null;
    }
  }

  /* ------------------------------------------------------------------------------ diagnóstico */

  let includeIdentifiers = $state(false);
  let vistaPrevia = $state<DiagnosticPreview | null>(null);
  let previaError = $state<AppError | null>(null);
  let cargandoPrevia = $state(false);
  let guardandoZip = $state(false);
  let ultimaRutaZip = $state<string | null>(null);

  async function verContenido() {
    cargandoPrevia = true;
    previaError = null;
    try {
      vistaPrevia = await previewDiagnosticZip(includeIdentifiers);
    } catch (cause) {
      previaError = toAppError(cause);
      vistaPrevia = null;
    } finally {
      cargandoPrevia = false;
    }
  }

  async function guardarZip() {
    guardandoZip = true;
    try {
      const destino = await chooseSavePath({
        defaultFileName: "diagnostico-smartdisk.zip",
        filterName: t("reports.diagnostic.zipFilter"),
        extensions: ["zip"]
      });
      if (!destino) return;
      ultimaRutaZip = await createDiagnosticZip(includeIdentifiers, destino);
      vistaPrevia = null;
    } catch (cause) {
      previaError = toAppError(cause);
    } finally {
      guardandoZip = false;
    }
  }
</script>

<div class="flex flex-col gap-5 p-5">
  <h1 class="m-0 text-2xl font-semibold tracking-tight">{t("nav.reports")}</h1>

  {#if app.devices.length === 0}
    <EmptyState kind="empty" title={t("dashboard.noDevices")} body={t("dashboard.noDevicesHint")} />
  {:else}
    <Card title={t("reports.range.title")}>
      <SegmentedControl options={opcionesRango} value={rango} onchange={(v: Rango) => (rango = v)} />
      {#if rango === "custom"}
        <DateRangePicker
          from={customFrom}
          to={customTo}
          fromLabel={t("dateRange.from")}
          toLabel={t("dateRange.to")}
          max={new Date().toISOString().slice(0, 10)}
          onchange={(r: { from: string; to: string }) => {
            customFrom = r.from;
            customTo = r.to;
          }}
        />
      {/if}
    </Card>

    <Card title={t("reports.devices.title")}>
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {#each app.devices as d (d.id)}
          <Switch
            checked={seleccion[d.id] ?? true}
            label={d.alias || d.model}
            onchange={(v: boolean) => (seleccion = { ...seleccion, [d.id]: v })}
          />
        {/each}
      </div>
      <Switch
        checked={includeSerials}
        label={t("reports.includeSerials.label")}
        hint={t("reports.includeSerials.hint")}
        onchange={(v: boolean) => (includeSerials = v)}
      />
    </Card>

    {#if exportError}
      <div class="rounded-inner bg-crit-soft p-4 text-sm text-crit" role="alert">
        {t(exportError.messageKey, exportError.messageVars)}
      </div>
    {/if}
    {#if ultimaRutaExportada}
      <div class="rounded-inner bg-ok-soft p-4 text-sm text-fg">
        {t("reports.export.savedAt", { path: ultimaRutaExportada })}
      </div>
    {/if}

    <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
      <Card title={t("reports.format.csv")}>
        <p class="m-0 min-h-[40px] text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
          {t("reports.format.csvDesc")}
        </p>
        <Button
          variant="secondary"
          loading={exportando === "csv"}
          disabled={exportando !== null}
          onclick={() => exportar("csv")}
        >
          {t("reports.cta.export")}
        </Button>
      </Card>
      <Card title={t("reports.format.json")}>
        <p class="m-0 min-h-[40px] text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
          {t("reports.format.jsonDesc")}
        </p>
        <Button
          variant="secondary"
          loading={exportando === "json"}
          disabled={exportando !== null}
          onclick={() => exportar("json")}
        >
          {t("reports.cta.export")}
        </Button>
      </Card>
      <Card title={t("reports.format.html")}>
        <p class="m-0 min-h-[40px] text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
          {t("reports.format.htmlDesc")}
        </p>
        <Button
          variant="secondary"
          loading={exportando === "html"}
          disabled={exportando !== null}
          onclick={() => exportar("html")}
        >
          {t("reports.cta.export")}
        </Button>
      </Card>
    </div>

    <Card title={t("reports.diagnostic.title")}>
      <p class="m-0 text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
        {t("reports.diagnostic.desc")}
      </p>
      <Switch
        checked={includeIdentifiers}
        label={t("reports.diagnostic.includeIdentifiers.label")}
        hint={t("reports.diagnostic.includeIdentifiers.hint")}
        onchange={(v: boolean) => {
          includeIdentifiers = v;
          vistaPrevia = null;
        }}
      />
      {#if includeIdentifiers}
        <div class="flex gap-3 rounded-inner bg-warn-soft p-4">
          <span class="shrink-0 text-warn"><Icon name="alert" size={16} /></span>
          <span class="text-xs leading-normal" style="text-wrap: pretty">
            {t("reports.diagnostic.includeIdentifiers.warning")}
          </span>
        </div>
      {/if}

      {#if previaError}
        <div class="rounded-inner bg-crit-soft p-4 text-sm text-crit" role="alert">
          {t(previaError.messageKey, previaError.messageVars)}
        </div>
      {/if}

      {#if !vistaPrevia}
        <Button variant="secondary" loading={cargandoPrevia} onclick={verContenido}>
          {t("reports.diagnostic.cta.preview")}
        </Button>
      {:else}
        <div class="flex flex-col gap-2">
          <span class="text-xs font-semibold">
            {t("reports.diagnostic.preview.total", { size: formatBytes(vistaPrevia.totalBytes) })}
          </span>
          <ul class="m-0 flex list-none flex-col gap-1 p-0">
            {#each vistaPrevia.entries as entrada (entrada.path)}
              <li class="flex justify-between gap-3 rounded-inner bg-glass-3 px-3 py-2 text-xs">
                <span class="font-mono text-fg-dim">{entrada.path}</span>
                <span class="flex items-center gap-3">
                  <span class="text-fg-faint">{t(entrada.descriptionKey)}</span>
                  <span class="tabular-nums">{formatBytes(entrada.sizeBytes)}</span>
                </span>
              </li>
            {/each}
          </ul>
          {#if vistaPrevia.redactedFields.length > 0}
            <span class="text-xs text-fg-dim">
              {t("reports.diagnostic.preview.redacted")}
              {vistaPrevia.redactedFields.map((k) => t(k)).join(", ")}
            </span>
          {/if}
          <div class="flex gap-2">
            <Button variant="ghost" onclick={() => (vistaPrevia = null)}>
              {t("common.cancel")}
            </Button>
            <Button variant="primary" loading={guardandoZip} onclick={guardarZip}>
              {t("reports.diagnostic.cta.save")}
            </Button>
          </div>
        </div>
      {/if}

      {#if ultimaRutaZip}
        <div class="rounded-inner bg-ok-soft p-4 text-sm text-fg">
          {t("reports.export.savedAt", { path: ultimaRutaZip })}
        </div>
      {/if}
    </Card>
  {/if}
</div>
