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
  import { AppShell, ConfirmDialog, Sidebar, Toolbar } from "$lib/components";
  import { theme } from "$lib/design/theme.svelte";
  import { applySystemAccent } from "$lib/design/accent";
  import { i18n, t, tp } from "$lib/i18n";
  import { globalStatus, type GlobalStatusKind } from "$lib/design/health";
  import { healthIcon, type IconName } from "$lib/design/icons";
  import { formatAge } from "$lib/design/format";
  import {
    getAppInfo,
    getAppearanceSettings,
    getDevices,
    getLogLevel,
    refreshNow,
    subscribe,
    toAppError
  } from "$lib/api";
  import { setLogLevel as setLoggerLevel } from "$lib/logger";
  import { app } from "$lib/stores/app.svelte";
  import type { AppError } from "$lib/design/types";

  let { children } = $props();

  let ready = $state(false);
  let startupError = $state<AppError | null>(null);

  /** El asistente inicial (US-002) ocupa la ventana entera: sin riel ni barra de herramientas
   *  (FR-033). El `onMount` de abajo sigue corriendo —el asistente necesita tema, idioma e
   *  inventario—, solo se omite el `AppShell`. */
  const esOnboarding = $derived(page.url.pathname === "/onboarding");

  /** Mapa de navegación. Las claves i18n viven en los diccionarios; el icono, en el riel. */
  const SECTIONS: { id: string; key: string; icon: IconName }[] = [
    { id: "/", key: "nav.dashboard", icon: "diskStack" },
    { id: "/alerts", key: "nav.alerts", icon: "alert" },
    { id: "/events", key: "nav.events", icon: "plug" },
    { id: "/tests", key: "nav.tests", icon: "flask" },
    { id: "/reports", key: "nav.reports", icon: "shield" },
    { id: "/settings", key: "nav.settings", icon: "wear" }
  ];

  const sections = $derived(
    SECTIONS.map((s) => ({
      id: s.id,
      href: s.id,
      label: t(s.key),
      icon: s.icon,
      badge: s.id === "/alerts" ? app.alerts.length || null : null
    }))
  );

  /** El estado global sale de **una sola función** (`globalStatus`) y se presenta en dos sitios —la
   *  píldora de la `Toolbar` y el pie del riel—, que así no pueden contradecirse. */
  const status = $derived(
    globalStatus({
      loaded: app.loadedAt !== null,
      paused: app.paused,
      monitoredStates: app.devices.map((d) => d.state)
    })
  );

  const globalLabel = $derived.by(() => {
    switch (status.kind) {
      case "loading":
        return t("global.loading");
      case "paused":
        return t("global.paused");
      case "noDevices":
        return t("global.noDevices");
      case "ok":
        return t("global.allGood");
      case "attention":
        return tp("global.needsAttention", status.count);
    }
  });

  const GLOBAL_ICON: Record<GlobalStatusKind, IconName> = {
    loading: "clock",
    paused: "clock",
    noDevices: "shield",
    ok: "shield",
    attention: "alert"
  };
  const globalIcon = $derived(
    status.kind === "attention" ? healthIcon[status.state] : GLOBAL_ICON[status.kind]
  );

  const freshness = $derived.by(() => {
    const age = formatAge(app.loadedAt);
    return age ? t("common.updatedAgo", { value: age }) : "";
  });

  const activeSection = $derived(
    SECTIONS.map((s) => s.id)
      .filter((id) => id !== "/" && page.url.pathname.startsWith(id))
      .at(0) ?? "/"
  );

  /** El título de la barra de herramientas sale **de la ruta**: cada `+page.ts` puede exponer
   *  `title`/`subtitle` en su `load` (el detalle de disco pone el alias); si no, se usa la etiqueta
   *  de la sección. Corrige el defecto de v2 («Panel general» fijo en todas las pantallas). */
  const routeTitle = $derived(
    typeof page.data?.title === "string" && page.data.title ? page.data.title : null
  );
  const routeSubtitle = $derived(typeof page.data?.subtitle === "string" ? page.data.subtitle : "");
  const screenTitle = $derived(
    routeTitle ?? t(SECTIONS.find((s) => s.id === activeSection)?.key ?? "nav.dashboard")
  );

  /* ---------------------------------------------------------------------------- acerca de (US-061) */

  let aboutOpen = $state(false);
  let appInfo = $state<{ name: string; version: string; author: string } | null>(null);
  let aboutError = $state<AppError | null>(null);

  async function abrirAcercaDe() {
    aboutOpen = true;
    if (appInfo) return;
    try {
      appInfo = await getAppInfo();
      aboutError = null;
    } catch (cause) {
      aboutError = toAppError(cause);
    }
  }

  /** "Información diagnóstica no sensible" (US-061): nombre, versión y autor, nada del equipo del
   *  usuario ni de sus discos. */
  async function copiarInformacion() {
    if (!appInfo) return;
    const texto = `${appInfo.name} ${appInfo.version}\n${appInfo.author}`;
    try {
      await navigator.clipboard?.writeText(texto);
    } catch {
      // Igual que `CodeOutput`: si el portapapeles rechaza la escritura, no se finge éxito.
    }
  }

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

        // El backend ya resolvió la precedencia completa del nivel de registro (constitución §XV:
        // --log-level > settings > info); el frontend solo adopta ese valor efectivo, nunca decide
        // el suyo propio.
        setLoggerLevel(await getLogLevel());

        // 2. El inventario. El `load` de `/` también lo trae, pero el estado global vive en el chrome
        //    y debe ser correcto en cualquier ruta de entrada (un enlace directo a `/disks/x` o a
        //    `/alerts`), no solo cuando se pasa por el panel. Se pide una vez aquí y a partir de ahí
        //    manda el evento `metrics:updated` (ADR-015, nada de sondeo).
        if (app.loadedAt === null) {
          const inv = await getDevices();
          app.devices = inv.devices;
          app.excluded = inv.excluded;
          app.sources = inv.sources;
          app.paused = inv.paused;
          app.pausedSince = inv.pausedSince;
          app.loadedAt = new Date().toISOString();
        }

        // 3. A partir de aquí solo se escucha.
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
{:else if esOnboarding}
  {#if ready}
    {@render children?.()}
  {:else}
    <div class="flex h-screen items-center justify-center">
      <span class="text-sm text-fg-dim">{t("common.loading")}</span>
    </div>
  {/if}
{:else}
  <AppShell transitionKey={page.url.pathname}>
    {#snippet sidebar()}
      <Sidebar
        {sections}
        active={activeSection}
        globalState={status.state}
        {globalLabel}
        {globalIcon}
        globalCount={status.count || null}
        onabout={abrirAcercaDe}
      />
    {/snippet}

    {#snippet toolbar()}
      <Toolbar
        title={screenTitle}
        subtitle={routeSubtitle}
        globalState={status.state}
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

<ConfirmDialog
  open={aboutOpen}
  title={appInfo ? `${appInfo.name} ${appInfo.version}` : t("about.title")}
  body={appInfo
    ? t("about.body", { author: appInfo.author })
    : aboutError
      ? t(aboutError.messageKey, aboutError.messageVars)
      : t("common.loading")}
  confirmLabel={t("about.cta.copy")}
  onconfirm={() => {
    void copiarInformacion();
    aboutOpen = false;
  }}
  oncancel={() => (aboutOpen = false)}
/>
