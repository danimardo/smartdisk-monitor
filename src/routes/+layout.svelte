<script lang="ts">
  /** Raíz de la aplicación. Se monta una sola vez y es el único sitio donde:
   *   - se importa `tokens.css`;
   *   - se inicializan tema, idioma y acento;
   *   - se suscriben los eventos que empuja el backend;
   *   - se monta el chrome (`AppShell` + `Sidebar` + `Toolbar`).
   *  Ninguna pantalla monta su propio chrome (`AGENTS.md` §4).
   */
  import "../app.css";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { AppShell, Sidebar, Toolbar } from "$lib/components";
  import { theme } from "$lib/design/theme.svelte";
  import { applySystemAccent } from "$lib/design/accent";
  import { i18n, t, tp } from "$lib/i18n";
  import { trayState } from "$lib/design/health";
  import { formatAge } from "$lib/design/format";
  import {
    getAppearanceSettings,
    pauseMonitoring,
    refreshNow,
    resumeMonitoring,
    subscribe,
    toAppError
  } from "$lib/api";
  import { app } from "$lib/stores/app.svelte";
  import type { AppError } from "$lib/design/types";

  let { children } = $props();

  let ready = $state(false);
  let startupError = $state<AppError | null>(null);

  /** Mapa de navegación. Las claves i18n viven en los diccionarios, no aquí. */
  const SECTIONS = [
    { id: "/", key: "nav.dashboard" },
    { id: "/alerts", key: "nav.alerts" },
    { id: "/events", key: "nav.events" },
    { id: "/tests", key: "nav.tests" },
    { id: "/reports", key: "nav.reports" },
    { id: "/settings", key: "nav.settings" }
  ];

  const sections = $derived(
    SECTIONS.map((s) => ({
      id: s.id,
      href: s.id,
      label: t(s.key),
      badge: s.id === "/alerts" ? app.alerts.length || null : null
    }))
  );

  /** El estado global sale de una sola función, nunca se recalcula por pantalla. */
  const globalState = $derived(
    trayState({
      paused: app.paused,
      collectorFailure: app.sources.some((s) => s.status === "error"),
      monitoredStates: app.devices.map((d) => d.state)
    })
  );

  /** Cuántos discos monitorizados no están correctos. Alimenta la etiqueta de estado global. */
  const needAttention = $derived(app.devices.filter((d) => d.state === "warn" || d.state === "crit").length);

  const globalLabel = $derived.by(() => {
    if (app.paused) return t("global.paused");
    // Sin discos no es "necesita atención": es que no hay nada que vigilar todavía.
    if (app.devices.length === 0) return t("global.noDevices");
    if (needAttention === 0) return t("global.allGood");
    return tp("global.needsAttention", needAttention);
  });

  const freshness = $derived.by(() => {
    const age = formatAge(app.loadedAt);
    return age ? t("common.updatedAgo", { value: age }) : "";
  });

  const activeSection = $derived(
    SECTIONS.map((s) => s.id)
      .filter((id) => id !== "/" && page.url.pathname.startsWith(id))
      .at(0) ?? "/"
  );

  const screenTitle = $derived(t(SECTIONS.find((s) => s.id === activeSection)?.key ?? "nav.dashboard"));

  onMount(() => {
    let unsubscribe: (() => void) | undefined;

    // `void` explícito: la promesa se gestiona por completo dentro (try/catch/finally) y no
    // hay nada que esperar fuera. Marcarlo evita que parezca un `await` olvidado.
    void (async () => {
      try {
        // 1. Apariencia antes del primer pintado, para no enseñar el tema equivocado un instante.
        const appearance = await getAppearanceSettings();
        theme.init(appearance.theme);
        i18n.init(appearance.language, appearance.systemLocale);
        if (appearance.useSystemAccent) await applySystemAccent();

        // 2. El inventario lo trae el `load` de cada pantalla (constitución §XIV). Aquí solo se
        //    escucha: nada de sondeo (ADR-015).
        unsubscribe = await subscribe({
          "metrics:updated": (p) => {
            app.upsertDevices(p.devices);
            app.sources = p.sources;
            app.loadedAt = p.emittedAt;
          },
          "alerts:changed": (p) => app.upsertAlerts(p.changed, p.removed),
          "inventory:changed": (p) => {
            app.upsertDevices([...p.added, ...p.updated]);
            app.devices = app.devices.filter((d) => !p.removed.includes(d.id));
          },
          "monitoring:paused": (p) => {
            app.paused = true;
            app.pausedSince = p.since;
          },
          "monitoring:resumed": () => {
            app.paused = false;
            app.pausedSince = null;
          },
          "system:accent-changed": () => void applySystemAccent()
        });
      } catch (cause) {
        // Un fallo de arranque se explica: nunca una ventana en blanco.
        startupError = toAppError(cause);
      } finally {
        ready = true;
      }
    })();

    return () => unsubscribe?.();
  });

  const togglePause = async () => {
    try {
      await (app.paused ? resumeMonitoring() : pauseMonitoring());
    } catch (cause) {
      startupError = toAppError(cause);
    }
  };
</script>

<svelte:head>
  <title>{screenTitle} · SmartDisk Monitor</title>
</svelte:head>

{#if startupError && !ready}
  <div class="flex h-screen items-center justify-center p-6">
    <div class="sdm-material max-w-lg rounded-card border border-hairline p-6 shadow-card">
      <h1 class="mb-2 text-xl font-semibold">{t("startup.failed")}</h1>
      <p class="mb-4 text-sm text-fg-dim">{t(startupError.messageKey, startupError.messageVars)}</p>
      {#if startupError.detail}
        <details class="text-2xs text-fg-faint">
          <summary class="cursor-pointer">{t("common.technicalDetail")}</summary>
          <pre class="mt-2 max-h-48 overflow-auto whitespace-pre-wrap">{startupError.detail}</pre>
        </details>
      {/if}
    </div>
  </div>
{:else}
  <AppShell>
    {#snippet sidebar()}
      <Sidebar
        {sections}
        disks={app.devices}
        active={activeSection}
        activeDiskId={page.params.id ?? ""}
        paused={app.paused}
        footerNote={globalLabel}
        ontogglepause={togglePause}
      />
    {/snippet}

    {#snippet toolbar()}
      <Toolbar
        title={screenTitle}
        {globalState}
        {globalLabel}
        {freshness}
        primaryLabel={t("common.refresh")}
        onprimary={() => refreshNow("all").catch((c) => (startupError = toAppError(c)))}
      />
    {/snippet}

    {#if ready}
      {@render children?.()}
    {:else}
      <div class="flex flex-1 items-center justify-center">
        <span class="text-sm text-fg-dim">{t("common.loading")}</span>
      </div>
    {/if}
  </AppShell>
{/if}
