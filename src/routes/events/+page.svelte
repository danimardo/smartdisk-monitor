<script lang="ts">
  /** Registro de eventos de Windows (US-021): lista virtualizada con filtros de nivel y
   *  proveedor, detalle con el XML original. El texto del mensaje llega en el idioma de Windows
   *  y se renderiza **como texto, jamás como HTML** (`docs/ui-contract.md` §3.5). */
  import { CodeOutput, EmptyState, EventRow, FilterBar, StatusPill, VirtualList } from "$lib/components";
  import { getEventRawXml, getSystemEvents, toAppError } from "$lib/api";
  import { formatDateTime } from "$lib/design/format";
  import { eventLevelIcon } from "$lib/design/icons";
  import { t } from "$lib/i18n";
  import type { AppError, HealthState } from "$lib/design/types";
  import type { SystemEvent } from "$lib/api";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  /** Mismo mapa que `EventRow`: el detalle repite la píldora de texto que la fila ya no lleva. */
  const NIVEL_A_SALUD: Record<"error" | "warning" | "info", HealthState> = {
    error: "crit",
    warning: "warn",
    info: "unknown"
  };

  /** Los mismos ocho proveedores que vigila el colector (`event_log::PROVEEDORES_VIGILADOS`):
   *  no son texto de interfaz, son identificadores literales del sistema, igual que `target` en
   *  `AlertGroup` — no pasan por `t()`. */
  const PROVEEDORES = [
    "disk",
    "Ntfs",
    "Microsoft-Windows-Ntfs",
    "volmgr",
    "Microsoft-Windows-NvmeDisk",
    "stornvme",
    "storahci",
    "Microsoft-Windows-StorageSpaces-Driver"
  ];

  let eventos = $state<SystemEvent[]>([]);
  let nextCursor = $state<string | null>(null);
  let cargandoMas = $state(false);
  let error = $state<AppError | null>(null);

  /** Lo que `load` trajo alimenta el estado una sola vez, al montar (mismo patrón que
   *  `+page.svelte` de alertas y del panel general). */
  $effect(() => {
    eventos = data.page.events;
    nextCursor = data.page.nextCursor;
  });

  let filtroSeleccionado = $state<Record<string, string[]>>({ level: [], provider: [] });

  const gruposFiltro = [
    {
      id: "level",
      label: t("events.filter.level"),
      options: [
        { id: "error", label: t("events.level.error") },
        { id: "warning", label: t("events.level.warning") },
        { id: "info", label: t("events.level.info") }
      ]
    },
    {
      id: "provider",
      label: t("events.filter.provider"),
      options: PROVEEDORES.map((p) => ({ id: p, label: p }))
    }
  ];

  async function recargar() {
    error = null;
    try {
      const pagina = await getSystemEvents({
        deviceId: data.deviceId,
        levels: filtroSeleccionado.level.length
          ? (filtroSeleccionado.level as SystemEvent["level"][])
          : undefined,
        providers: filtroSeleccionado.provider.length ? filtroSeleccionado.provider : undefined,
        limit: 200
      });
      eventos = pagina.events;
      nextCursor = pagina.nextCursor;
    } catch (cause) {
      error = toAppError(cause);
    }
  }

  function alCambiarFiltro(groupId: string, valores: string[]) {
    filtroSeleccionado = { ...filtroSeleccionado, [groupId]: valores };
    void recargar();
  }

  async function cargarMas() {
    if (!nextCursor) return;
    cargandoMas = true;
    try {
      const pagina = await getSystemEvents({
        deviceId: data.deviceId,
        levels: filtroSeleccionado.level.length
          ? (filtroSeleccionado.level as SystemEvent["level"][])
          : undefined,
        providers: filtroSeleccionado.provider.length ? filtroSeleccionado.provider : undefined,
        cursor: nextCursor,
        limit: 200
      });
      eventos = [...eventos, ...pagina.events];
      nextCursor = pagina.nextCursor;
    } catch (cause) {
      error = toAppError(cause);
    } finally {
      cargandoMas = false;
    }
  }

  let seleccionado = $state<SystemEvent | null>(null);
  let xml = $state<string | null>(null);
  let xmlError = $state<AppError | null>(null);

  async function seleccionar(evento: SystemEvent) {
    seleccionado = evento;
    xml = null;
    xmlError = null;
    if (!evento.hasRawXml) return;
    try {
      xml = await getEventRawXml(evento.id);
    } catch (cause) {
      xmlError = toAppError(cause);
    }
  }
</script>

<div class="flex h-full gap-5">
  <div class="flex flex-1 flex-col gap-3">
    <FilterBar groups={gruposFiltro} selected={filtroSeleccionado} onchange={alCambiarFiltro} />

    {#if error}
      <EmptyState
        kind="error"
        title={t("error.screenFailed")}
        body={t(error.messageKey, error.messageVars)}
        detail={error.detail ?? ""}
      />
    {:else if eventos.length === 0}
      <EmptyState kind="empty" title={t("events.empty.title")} body={t("events.empty.body")} />
    {:else}
      <VirtualList items={eventos} itemHeight={48} height={560} label={t("nav.events")}>
        {#snippet row(evento: SystemEvent)}
          <EventRow
            level={evento.level}
            message={evento.message}
            provider={evento.provider}
            eventId={evento.eventId}
            occurredAt={evento.occurredAt}
            mappingConfidence={evento.mappingConfidence}
            onselect={() => seleccionar(evento)}
          />
        {/snippet}
      </VirtualList>
      {#if nextCursor}
        <button
          class="self-center text-xs font-semibold text-fg-dim hover:text-fg"
          disabled={cargandoMas}
          onclick={cargarMas}
        >
          {t("events.loadMore")}
        </button>
      {/if}
    {/if}
  </div>

  <div class="w-[420px] shrink-0">
    {#if !seleccionado}
      <EmptyState kind="empty" title={t("events.detail.empty")} body="" />
    {:else}
      <div class="flex flex-col gap-3 sdm-material rounded-card p-6">
        <div class="flex items-center gap-2">
          <h2 class="m-0 text-sm font-semibold">{t("events.detail.title")}</h2>
          <div class="flex-1"></div>
          <StatusPill
            state={NIVEL_A_SALUD[seleccionado.level]}
            label={t(`events.level.${seleccionado.level}`)}
            icon={eventLevelIcon[seleccionado.level]}
          />
        </div>
        <p class="m-0 text-xs text-fg-dim">
          {seleccionado.provider} · {seleccionado.eventId} · {formatDateTime(seleccionado.occurredAt)}
        </p>
        <p class="m-0 text-sm" style="text-wrap: pretty">{seleccionado.message}</p>
        {#if xmlError}
          <p class="m-0 text-xs" style="color: var(--sdm-crit)">
            {t(xmlError.messageKey, xmlError.messageVars)}
          </p>
        {:else if xml}
          <span class="text-2xs font-semibold text-fg-dim">{t("events.detail.rawXml")}</span>
          <CodeOutput content={xml} />
        {/if}
      </div>
    {/if}
  </div>
</div>
