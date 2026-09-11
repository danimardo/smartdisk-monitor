<script lang="ts">
  /** Pantalla de informes y diagnóstico (US-050/051, `docs/product-specification.md` §9).
   *
   *  Sin boceto aprobado (a diferencia de la de pruebas): compuesta con el catálogo actual según
   *  `docs/ui-design.md` Apéndice C, con autorización explícita del usuario para este enfoque.
   *
   *  El destino de cada exportación lo elige el usuario con el diálogo nativo de guardado
   *  (ADR-031, `chooseSavePath`), nunca una ruta que construya esta pantalla.
   */
  import { onMount } from "svelte";
  import {
    Button,
    Card,
    DateRangePicker,
    EmptyState,
    Icon,
    ProgressBar,
    SegmentedControl,
    Switch
  } from "$lib/components";
  import {
    cancelarInforme,
    chooseSavePath,
    createDiagnosticZip,
    exportReport,
    on,
    previewDiagnosticZip,
    previewInformeIa,
    toAppError
  } from "$lib/api";
  import { formatBytes } from "$lib/design/format";
  import { i18n, t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import { ia } from "$lib/stores/ia.svelte";
  import esDict from "$lib/i18n/es.json";
  import enDict from "$lib/i18n/en.json";
  import type { DiagnosticPreview, PreviewInformeIa } from "$lib/api";
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

  /* ------------------------------------------------------------------ resumen con IA (spec 009) */

  let incluirResumenIa = $state(false);
  let numDiscosIncluidos = $derived(deviceIdsActuales()?.length ?? app.devices.length);

  /** Exportación en espera de la confirmación de la vista previa de IA (principio XVI): los
   *  parámetros ya resueltos (destino elegido, intervalo, discos) para no repetir el diálogo
   *  nativo de guardado al confirmar. */
  type ExportacionPendiente = {
    destino: string;
    from: string;
    to: string;
    deviceIds: string[] | null;
    alertLabels: Record<string, string>;
  };
  let pendienteExportacionIa = $state<ExportacionPendiente | null>(null);
  let vistaPreviaIa = $state<PreviewInformeIa | null>(null);
  let cargandoPreviaIa = $state(false);
  let previaIaError = $state<AppError | null>(null);
  let panelPreviaIa = $state<HTMLDivElement | undefined>();

  /** El foco tiene que entrar al abrir para que `Escape` (atado al velo) llegue a disparar — mismo
   *  motivo que `ConfirmDialog` (`docs/known-issues.md` #2). */
  $effect(() => {
    if (vistaPreviaIa) panelPreviaIa?.focus();
  });

  /** Fase de red de la exportación con IA (spec 009, US3): el modal de vista previa pasa a mostrar
   *  el progreso en cuanto se confirma. `progresoIa` llega por `report:progress` (ADR-015, sin
   *  sondeo); mientras no llegue ninguno, la barra queda indeterminada (la primera llamada puede
   *  tardar en arrancar la conexión TLS). */
  let exportandoConIa = $state(false);
  let progresoIa = $state<{ done: number; total: number; deviceLabel: string } | null>(null);

  onMount(() => {
    let unsubscribe: (() => void) | undefined;
    void (async () => {
      unsubscribe = await on("report:progress", (p) => {
        progresoIa = { done: p.done, total: p.total, deviceLabel: p.deviceLabel };
      });
    })();
    return () => unsubscribe?.();
  });

  /** Inofensivo sin exportación en curso: la próxima resetea la bandera en el backend. */
  async function cancelarExportacionEnCurso() {
    try {
      await cancelarInforme();
    } catch {
      // No hay nada útil que hacer con un fallo al cancelar: la exportación en curso, si la había,
      // sigue su curso normal y resolverá o fallará por su cuenta.
    }
  }

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

  /** Clave de regla de alerta → título legible, para que el informe HTML no pinte la clave cruda
   *  (`smart.error_log`). El backend no tiene diccionarios (ADR-030): se construye aquí, del
   *  mismo diccionario que usa `AlertCard`, y se le pasa a `export_report` para ese render. Se
   *  deriva de las claves `alert.rule.<regla>.title` ya existentes, así que una regla nueva no
   *  necesita ningún cambio en esta pantalla. */
  function etiquetasDeAlerta(): Record<string, string> {
    const dict: Record<string, string> = i18n.locale === "es" ? esDict : enDict;
    const etiquetas: Record<string, string> = {};
    for (const clave of Object.keys(dict)) {
      const m = /^alert\.rule\.(.+)\.title$/.exec(clave);
      if (m) etiquetas[m[1]] = dict[clave];
    }
    return etiquetas;
  }

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
      const deviceIds = deviceIdsActuales();
      const alertLabels = formato === "html" ? etiquetasDeAlerta() : {};

      if (formato === "html" && incluirResumenIa && ia.activa) {
        // Vista previa obligatoria antes de enviar nada a la IA (principio XVI): una confirmación
        // cubre todo el informe, pero primero hay que ver el texto exacto de cada disco.
        cargandoPreviaIa = true;
        previaIaError = null;
        try {
          vistaPreviaIa = await previewInformeIa({ fromUtc: from, toUtc: to, deviceIds, alertLabels });
          pendienteExportacionIa = { destino, from, to, deviceIds, alertLabels };
        } catch (cause) {
          previaIaError = toAppError(cause);
        } finally {
          cargandoPreviaIa = false;
        }
        return;
      }

      const ruta = await exportReport({
        format: formato,
        fromUtc: from,
        toUtc: to,
        deviceIds,
        includeSerials,
        destinationPath: destino,
        alertLabels: formato === "html" ? alertLabels : null
      });
      ultimaRutaExportada = ruta;
    } catch (cause) {
      exportError = toAppError(cause);
    } finally {
      exportando = null;
    }
  }

  function cancelarPreviaIa() {
    vistaPreviaIa = null;
    pendienteExportacionIa = null;
    previaIaError = null;
  }

  async function confirmarExportacionIa() {
    if (!pendienteExportacionIa) return;
    const p = pendienteExportacionIa;
    exportando = "html";
    exportandoConIa = true;
    progresoIa = null;
    exportError = null;
    try {
      const ruta = await exportReport({
        format: "html",
        fromUtc: p.from,
        toUtc: p.to,
        deviceIds: p.deviceIds,
        includeSerials,
        destinationPath: p.destino,
        alertLabels: p.alertLabels,
        includeAiSummary: true,
        previewConfirmada: true
      });
      ultimaRutaExportada = ruta;
    } catch (cause) {
      exportError = toAppError(cause);
    } finally {
      exportando = null;
      exportandoConIa = false;
      progresoIa = null;
      cancelarPreviaIa();
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
    {#if previaIaError}
      <div class="rounded-inner bg-crit-soft p-4 text-sm text-crit" role="alert">
        {t(previaIaError.messageKey, previaIaError.messageVars)}
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
        {#if ia.activa}
          <Switch
            checked={incluirResumenIa}
            label={t("reports.ai.label")}
            hint={t("reports.ai.hint", { count: numDiscosIncluidos })}
            onchange={(v: boolean) => (incluirResumenIa = v)}
          />
        {/if}
        <Button
          variant="secondary"
          loading={exportando === "html" || cargandoPreviaIa}
          disabled={exportando !== null || cargandoPreviaIa}
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

{#if vistaPreviaIa && pendienteExportacionIa}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- El velo solo captura el clic fuera del diálogo; el cierre por teclado lo cubre el
       `onkeydown` de Escape de esta misma capa, alcanzable porque el foco entra al panel al abrir
       (mismo patrón que ConfirmDialog/ExplicacionModal). Ver docs/known-issues.md #2 -->
  <div
    class="fixed inset-0 z-50 grid place-items-center p-8 backdrop-blur-[3px]"
    style="background: var(--sdm-scrim)"
    role="presentation"
    onclick={() => !exportandoConIa && cancelarPreviaIa()}
    onkeydown={(e) => e.key === "Escape" && !exportandoConIa && cancelarPreviaIa()}
  >
    <div
      bind:this={panelPreviaIa}
      class="sdm-material-overlay relative flex max-h-[80vh] w-full max-w-2xl flex-col gap-4 overflow-y-auto rounded-window p-6"
      style="animation: sdm-dialog var(--sdm-duration-overlay) var(--sdm-ease)"
      role="dialog"
      aria-modal="true"
      aria-label={t("reports.ai.preview.title")}
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 class="m-0 text-lg font-semibold tracking-tight">{t("reports.ai.preview.title")}</h2>

      {#if exportandoConIa}
        <div role="status" aria-live="polite">
          <ProgressBar
            indeterminate={progresoIa === null}
            value={progresoIa ? ((progresoIa.done + 1) / progresoIa.total) * 100 : 0}
            caption={progresoIa
              ? t("reports.ai.progress.caption", {
                  done: progresoIa.done + 1,
                  total: progresoIa.total,
                  device: progresoIa.deviceLabel
                })
              : t("reports.ai.progress.starting")}
            trailing=""
          />
        </div>
        <div class="flex justify-end">
          <Button variant="ghost" onclick={cancelarExportacionEnCurso}>{t("common.cancel")}</Button>
        </div>
      {:else}
        <p class="m-0 text-sm text-fg-dim" style="text-wrap: pretty">
          {t("reports.ai.preview.body", { count: vistaPreviaIa.totalLlamadas })}
        </p>

        {#if vistaPreviaIa.redactedFields.length > 0}
          <span class="text-xs text-fg-dim">
            {t("reports.diagnostic.preview.redacted")}
            {vistaPreviaIa.redactedFields.map((k) => t(k)).join(", ")}
          </span>
        {/if}

        <div class="flex flex-col gap-3">
          {#each vistaPreviaIa.discos as disco (disco.deviceId)}
            <details class="rounded-inner bg-glass-3 p-3">
              <summary class="cursor-pointer text-sm font-semibold">{disco.deviceLabel}</summary>
              {#if disco.fragmentos.length > 0}
                <div class="mt-2 flex flex-col gap-1 rounded-inner bg-warn-soft p-3">
                  <p class="m-0 text-xs font-semibold text-warn">{t("ai.review.flaggedTitle")}</p>
                  <ul class="m-0 flex list-disc flex-col gap-0.5 pl-5 text-xs text-fg-dim">
                    {#each disco.fragmentos as f}
                      <li><code class="font-mono">{f.texto}</code> — {t(f.motivoKey)}</li>
                    {/each}
                  </ul>
                </div>
              {/if}
              <pre
                class="m-0 mt-2 max-h-48 overflow-y-auto whitespace-pre-wrap rounded-inner bg-glass-1 p-3 text-xs text-fg-dim">{disco.textoEnviado}</pre>
              {#if disco.recortado}
                <p class="m-0 mt-1 text-2xs text-fg-faint">{t("ai.modal.truncated")}</p>
              {/if}
            </details>
          {/each}
        </div>

        <div class="flex justify-end gap-2">
          <Button variant="ghost" onclick={cancelarPreviaIa}>{t("common.cancel")}</Button>
          <Button variant="primary" onclick={confirmarExportacionIa}>
            {t("reports.ai.preview.confirm")}
          </Button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  @keyframes sdm-dialog {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
</style>
