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
  import { AboutDialog, AppShell, IconSprite, Sidebar, TitleBar, Toolbar } from "$lib/components";
  import {
    closeWindow,
    isWindowMaximized,
    minimizeWindow,
    onWindowResized,
    toggleMaximizeWindow
  } from "$lib/window";
  import { theme } from "$lib/design/theme.svelte";
  import { applySystemAccent } from "$lib/design/accent";
  import { i18n, t, tp } from "$lib/i18n";
  import { estadoParaRecuento, globalStatus, type GlobalStatusKind } from "$lib/design/health";
  import { healthIcon, type IconName } from "$lib/design/icons";
  import { formatAge } from "$lib/design/format";
  import {
    getAlertGroups,
    getAppInfo,
    getAppearanceSettings,
    getDevices,
    getLogLevel,
    refreshNow,
    subscribe,
    toAppError
  } from "$lib/api";
  import { createLogger, setLogLevel as setLoggerLevel } from "$lib/logger";
  import { app } from "$lib/stores/app.svelte";
  import { ia } from "$lib/stores/ia.svelte";
  import type { AppError } from "$lib/design/types";

  let { children } = $props();

  const log = createLogger("app");

  let ready = $state(false);
  let startupError = $state<AppError | null>(null);

  /** Estado de la ventana para el control de maximizar/restaurar de la barra propia (spec 011,
   *  FR-005): se reconsulta cada vez que la ventana cambia de tamaño (doble clic, `Win+↑`, arrastre
   *  al borde), no solo tras pulsar el propio botón. */
  let maximizada = $state(false);

  /** Refresco manual de datos («Refrescar» de la barra). `refresh_now` corre en un hilo bloqueante
   *  del backend, así que la ventana no se congela; aquí solo se refleja que hay trabajo en curso
   *  (spinner en el botón + barra superior). */
  let refrescando = $state(false);
  async function refrescar() {
    if (refrescando) return;
    refrescando = true;
    try {
      await refreshNow("all");
    } catch (cause) {
      startupError = toAppError(cause);
    } finally {
      refrescando = false;
    }
  }

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
      monitoredStates: app.devices.map((d) => estadoParaRecuento(d, app.alerts, { paused: app.paused }))
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
    if (!app.loadedAt) return "";
    // Por debajo de 10 s, «hace un momento» en vez del «hace ahora» que sale de `Intl` con
    // `numeric: "auto"`, que en la barra queda raro.
    if (Date.now() - new Date(app.loadedAt).getTime() < 10_000) {
      return t("common.updatedAgo", { value: t("common.moment") });
    }
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
  /** El subtítulo sale de la ruta; y en el panel general, a falta de uno, el recuento de discos
   *  monitorizados (boceto). El recuento es dinámico y vive en el store, no en el `load`. */
  const screenSubtitle = $derived(
    routeSubtitle ||
      (activeSection === "/" && app.loadedAt && app.devices.length > 0
        ? tp("dashboard.deviceCount", app.devices.length)
        : "")
  );
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
    let dejarDeEscucharRedimension: (() => void) | undefined;

    // Estado de la barra de título: aparte del arranque principal (try/catch propio) para que un
    // fallo aquí no dispare `startupError` — perder el reflejo de "maximizada" no debe tumbar el
    // resto de la aplicación.
    void (async () => {
      try {
        maximizada = await isWindowMaximized();
        dejarDeEscucharRedimension = await onWindowResized(() => {
          void isWindowMaximized().then((m) => (maximizada = m));
        });
      } catch (cause) {
        log.warn("no se pudo inicializar el estado de la barra de título", {
          code: toAppError(cause).code
        });
      }
    })();

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

        // Estado de la ayuda con IA (spec 005): igual que el inventario, tiene que ser correcto en
        // cualquier ruta de entrada, no solo si se pasa antes por Ajustes. No toca la red.
        if (ia.estado === null) await ia.refrescar().catch(() => {});

        // 3. A partir de aquí solo se escucha.
        unsubscribe = await subscribe({
          "metrics:updated": (p) => {
            app.upsertDevices(p.devices);
            // Cada evento aporta un punto a la onda de actividad de cada tarjeta (ADR-051): así se
            // refresca en vivo sin sondeo (el backend ya empuja, ADR-015).
            app.pushActivitySamples(p.devices, p.emittedAt);
            app.sources = p.sources;
            app.loadedAt = p.emittedAt;
          },
          "alerts:changed": (p) => app.upsertAlerts(p.changed, p.removed),
          "inventory:changed": (p) => {
            app.upsertDevices([...p.added, ...p.updated]);
            app.devices = app.devices.filter((d) => !p.removed.includes(d.id));
            app.prunearSeriesActividad();
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

        // 4. Las alertas, por el mismo motivo que el inventario: el color del disco y el estado
        //    global (píldora + pie del riel) deben ser correctos aunque se entre directo a
        //    `/disks/x` o al panel, no solo al pasar por `/alerts`. Sin filtro, igual que
        //    `alerts/+page.ts`. Va **después** de suscribirse y en su propio `try`: no son
        //    imprescindibles para arrancar y un fallo aquí no debe tumbar la suscripción ni la
        //    pantalla — si no llegan ahora, llegan por `alerts:changed` (ADR-015).
        if (app.alertsLoadedAt === null) {
          try {
            app.upsertAlerts(await getAlertGroups());
            app.alertsLoadedAt = new Date().toISOString();
          } catch (cause) {
            log.warn("no se pudieron cargar las alertas al arrancar", {
              code: toAppError(cause).code
            });
          }
        }
      } catch (cause) {
        // Un fallo de arranque se explica: nunca una ventana en blanco.
        startupError = toAppError(cause);
      } finally {
        ready = true;
      }
    })();

    return () => {
      unsubscribe?.();
      dejarDeEscucharRedimension?.();
    };
  });
</script>

<svelte:head>
  <title>{screenTitle} · SmartDisk Monitor</title>
</svelte:head>

<!-- Sprite de iconos: una sola vez, fuera de toda rama, para que `<Icon>` resuelva también en
     `/onboarding` (que se pinta sin `AppShell`). -->
<IconSprite />

<!-- Barra de título propia (spec 011): fuera de las tres ramas de abajo, mismo motivo que
     `IconSprite` — el asistente inicial y la pantalla de error de arranque también necesitan poder
     mover/cerrar la ventana. El contenedor de aquí es el único `h-screen` real; todo lo de dentro
     pasa a `h-full` (ocupa lo que deja la barra, no el alto entero de la ventana). -->
<div class="flex h-screen flex-col">
  <TitleBar
    maximized={maximizada}
    onminimize={() => void minimizeWindow()}
    ontogglemaximize={() => void toggleMaximizeWindow()}
    onclose={() => void closeWindow()}
  />
  <div class="min-h-0 flex-1 overflow-hidden">
    {#if startupError && !ready}
      <div class="flex h-full items-center justify-center p-6">
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
        <div class="flex h-full items-center justify-center">
          <span class="text-sm text-fg-dim">{t("common.loading")}</span>
        </div>
      {/if}
    {:else}
      <AppShell transitionKey={page.url.pathname} busy={refrescando}>
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
            subtitle={screenSubtitle}
            globalState={status.state}
            {globalLabel}
            {freshness}
            primaryLabel={t("common.refresh")}
            primaryLoading={refrescando}
            onprimary={refrescar}
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
  </div>
</div>

<AboutDialog
  open={aboutOpen}
  {appInfo}
  error={aboutError}
  oncopy={() => {
    void copiarInformacion();
    aboutOpen = false;
  }}
  onclose={() => (aboutOpen = false)}
/>
