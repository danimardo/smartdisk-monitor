<script lang="ts">
  /** Pantalla de alertas (US-030/031, `docs/ui-design.md` §7.3): lista de `AlertCard` en columna
   *  fija + detalle con severidad, titular, explicación humana, rejilla de hechos, acciones
   *  (reconocer/silenciar/archivar) y cronología de ocurrencias. La lista llega lista desde `load`
   *  (constitución §XIV); a partir de ahí manda el store, que recibe `alerts:changed` (ADR-015). */
  import {
    AlertCard,
    Button,
    ConfirmDialog,
    DataRow,
    EmptyState,
    Select,
    SegmentedControl,
    StatusPill
  } from "$lib/components";
  import {
    acknowledgeAlert,
    archiveAlert,
    getAlertDetail,
    muteAlert,
    toAppError,
    unmuteAlert
  } from "$lib/api";
  import { formatDateTime } from "$lib/design/format";
  import { severityToHealth } from "$lib/design/health";
  import { t } from "$lib/i18n";
  import { app } from "$lib/stores/app.svelte";
  import type { AlertDetail } from "$lib/api";
  import type { AlertGroup, AlertStatus, AppError } from "$lib/design/types";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  /** Lo que `load` trajo alimenta el store una sola vez; a partir de ahí manda el store. */
  $effect(() => {
    if (app.alertsLoadedAt === null) {
      app.upsertAlerts(data.alerts);
      app.alertsLoadedAt = new Date().toISOString();
    }
  });

  const alerts = $derived(app.alertsLoadedAt ? app.alerts : data.alerts);

  type Filtro = "active" | "resolved" | "archived" | "all";
  const ESTADOS_POR_FILTRO: Record<Filtro, AlertStatus[] | null> = {
    active: ["active", "acknowledged"],
    resolved: ["resolved"],
    archived: ["archived"],
    all: null
  };
  const opcionesFiltro = [
    { id: "active", label: t("alerts.filter.active") },
    { id: "resolved", label: t("alerts.filter.resolved") },
    { id: "archived", label: t("alerts.filter.archived") },
    { id: "all", label: t("alerts.filter.all") }
  ];

  let filtro = $state<Filtro>("active");
  let selectedId = $state<string | null>(null);

  const alertasFiltradas = $derived.by(() => {
    const estados = ESTADOS_POR_FILTRO[filtro];
    const lista = estados ? alerts.filter((a: AlertGroup) => estados.includes(a.status)) : alerts;
    return [...lista].sort((a, b) => b.lastOccurredAt.localeCompare(a.lastOccurredAt));
  });

  /** La selección sigue viva mientras siga en el filtro actual; si no, cae a la primera. */
  $effect(() => {
    if (alertasFiltradas.length === 0) {
      selectedId = null;
    } else if (!alertasFiltradas.some((a) => a.id === selectedId)) {
      selectedId = alertasFiltradas[0].id;
    }
  });

  let detail = $state<AlertDetail | null>(null);
  let detailError = $state<AppError | null>(null);
  let detailLoading = $state(false);
  let actionError = $state<AppError | null>(null);
  let actionPending = $state(false);
  let muteMinutes = $state<"15" | "60" | "480" | "indefinite">("60");
  let archiveDialogOpen = $state(false);

  async function cargarDetalle(id: string) {
    detailLoading = true;
    try {
      detail = await getAlertDetail(id);
      detailError = null;
    } catch (cause) {
      detail = null;
      detailError = toAppError(cause);
    } finally {
      detailLoading = false;
    }
  }

  $effect(() => {
    const id = selectedId;
    if (!id) {
      detail = null;
      detailError = null;
      return;
    }
    let cancelado = false;
    void (async () => {
      detailLoading = true;
      try {
        const d = await getAlertDetail(id);
        if (!cancelado) {
          detail = d;
          detailError = null;
        }
      } catch (cause) {
        if (!cancelado) {
          detail = null;
          detailError = toAppError(cause);
        }
      } finally {
        if (!cancelado) detailLoading = false;
      }
    })();
    return () => {
      cancelado = true;
    };
  });

  async function ejecutar(accion: () => Promise<void>) {
    actionPending = true;
    try {
      await accion();
      actionError = null;
      if (selectedId) await cargarDetalle(selectedId);
    } catch (cause) {
      actionError = toAppError(cause);
    } finally {
      actionPending = false;
    }
  }

  const minutosPorOpcion: Record<string, 15 | 60 | 480 | null> = {
    "15": 15,
    "60": 60,
    "480": 480,
    indefinite: null
  };

  const estaSilenciada = $derived(detail?.mutedUntil != null);
  const puedeReconocer = $derived(detail?.status === "active");
  const puedeArchivar = $derived(detail !== null && detail.status !== "archived");
  const puedeSilenciar = $derived(detail !== null && detail.status !== "archived" && !estaSilenciada);

  const tituloDetalle = $derived(detail ? t(`alert.rule.${detail.ruleKey}.title`) : "");
  const resumenDetalle = $derived(detail ? t(`alert.rule.${detail.ruleKey}.summary`) : "");
  const etiquetaSeveridadDetalle = $derived(detail ? t(`health.${severityToHealth[detail.severity]}`) : "");
</script>

<div class="flex h-full gap-5">
  <div class="flex w-[470px] shrink-0 flex-col gap-3">
    <SegmentedControl options={opcionesFiltro} value={filtro} onchange={(v: Filtro) => (filtro = v)} />
    {#if alertasFiltradas.length === 0}
      <EmptyState kind="empty" title={t("alerts.empty.title")} body={t("alerts.empty.body")} />
    {:else}
      <div class="flex flex-col gap-2 overflow-y-auto">
        {#each alertasFiltradas as alerta (alerta.id)}
          <AlertCard
            alert={alerta}
            selected={alerta.id === selectedId}
            onselect={(id: string) => (selectedId = id)}
          />
        {/each}
      </div>
    {/if}
  </div>

  <div class="flex-1">
    {#if detailLoading && !detail}
      <EmptyState kind="empty" title={t("common.loading")} body="" />
    {:else if detailError}
      <EmptyState
        kind="error"
        title={t("error.screenFailed")}
        body={t(detailError.messageKey, detailError.messageVars)}
        detail={detailError.detail ?? ""}
      />
    {:else if !detail}
      <EmptyState kind="empty" title={t("alerts.detail.empty")} body="" />
    {:else}
      <div class="flex flex-col gap-5 sdm-material rounded-card p-6">
        <div class="flex items-center gap-3">
          <StatusPill state={severityToHealth[detail.severity]} label={etiquetaSeveridadDetalle} />
          <h2 class="m-0 flex-1 text-lg font-semibold">{tituloDetalle}</h2>
        </div>
        <p class="m-0 text-sm text-fg-dim" style="text-wrap: pretty">{resumenDetalle}</p>

        {#if actionError}
          <p class="m-0 text-xs font-medium" style="color: var(--sdm-crit)">
            {t(actionError.messageKey, actionError.messageVars)}
          </p>
        {/if}

        <div class="flex flex-col gap-1">
          {#each detail.facts as fact (fact.labelKey)}
            <DataRow label={t(fact.labelKey)} value={fact.value ?? t("common.notAvailable")} />
          {/each}
        </div>

        <div class="flex flex-wrap items-center gap-3">
          {#if puedeReconocer}
            <Button
              size="sm"
              disabled={actionPending}
              onclick={() => ejecutar(() => acknowledgeAlert(detail!.id))}
            >
              {t("alerts.actions.acknowledge")}
            </Button>
          {/if}
          {#if puedeSilenciar}
            <Select
              label={t("alerts.mute.duration")}
              value={muteMinutes}
              options={[
                { id: "15", label: t("alerts.mute.15") },
                { id: "60", label: t("alerts.mute.60") },
                { id: "480", label: t("alerts.mute.480") },
                { id: "indefinite", label: t("alerts.mute.indefinite") }
              ]}
              onchange={(v: typeof muteMinutes) => (muteMinutes = v)}
            />
            <Button
              size="sm"
              disabled={actionPending}
              onclick={() => ejecutar(() => muteAlert(detail!.id, minutosPorOpcion[muteMinutes]))}
            >
              {t("alerts.actions.mute")}
            </Button>
          {/if}
          {#if estaSilenciada}
            <span class="text-xs text-fg-dim">
              {detail.mutedUntil === "infinite"
                ? t("alerts.mute.untilIndefinite")
                : t("alerts.mute.untilDate", { value: formatDateTime(detail.mutedUntil) })}
            </span>
            <Button
              size="sm"
              disabled={actionPending}
              onclick={() => ejecutar(() => unmuteAlert(detail!.id))}
            >
              {t("alerts.actions.unmute")}
            </Button>
          {/if}
          {#if puedeArchivar}
            <Button size="sm" disabled={actionPending} onclick={() => (archiveDialogOpen = true)}>
              {t("alerts.actions.archive")}
            </Button>
          {/if}
        </div>

        <div class="flex flex-col gap-2">
          <h3 class="m-0 text-sm font-semibold">{t("alerts.timeline.title")}</h3>
          {#each detail.occurrences as ocurrencia (ocurrencia.occurredAt + ocurrencia.cycle)}
            <DataRow
              label={t("alerts.timeline.entry", {
                cycle: ocurrencia.cycle,
                when: formatDateTime(ocurrencia.occurredAt)
              })}
              value={ocurrencia.value !== null ? String(ocurrencia.value) : t("common.notAvailable")}
            />
            {#if ocurrencia.eventId}
              <a
                class="self-start text-xs font-medium text-accent hover:underline"
                href="/events?focus={ocurrencia.eventId}"
              >
                {t("alerts.timeline.viewEvent")}
              </a>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<ConfirmDialog
  open={archiveDialogOpen}
  title={t("alerts.archive.confirmTitle")}
  body={t("alerts.archive.confirmBody")}
  impact={t("alerts.archive.confirmImpact")}
  confirmLabel={t("alerts.actions.archive")}
  destructive
  onconfirm={() => {
    archiveDialogOpen = false;
    if (detail) void ejecutar(() => archiveAlert(detail!.id));
  }}
  oncancel={() => (archiveDialogOpen = false)}
/>
