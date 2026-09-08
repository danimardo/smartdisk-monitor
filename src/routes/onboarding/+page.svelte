<script lang="ts">
  /** Asistente inicial (US-002, `cambios/08-onboarding.md`). Cuatro pasos, uno por pantalla, sin
   *  chrome (el `+layout.svelte` omite el `AppShell` en esta ruta). «Omitir y usar los valores de
   *  fábrica» está siempre en la cabecera: omitir es una opción legítima (FR-036).
   *
   *  Nada de lo que se elige aquí es irreversible y se dice: todo se cambia luego en Ajustes. */
  import { goto, invalidateAll } from "$app/navigation";
  import {
    AiModelSelect,
    Button,
    CapacityBar,
    Card,
    EmptyState,
    Icon,
    OnboardingArt,
    ProgressBar,
    StatusPill,
    Switch,
    TextField
  } from "$lib/components";
  import {
    getSettings,
    guardarClaveIa,
    refreshNow,
    setDeviceAlias,
    setDeviceMonitoring,
    setSetting,
    toAppError
  } from "$lib/api";
  import { ia } from "$lib/stores/ia.svelte";
  import { busIcon } from "$lib/design/icons";
  import { PERFILES, PERFIL_RECOMENDADO, perfilLabelKey, type PerfilAlerta } from "$lib/design/perfiles";
  import { t } from "$lib/i18n";
  import type { AppError, DiskSummary } from "$lib/design/types";
  import type { SettingsShape } from "$lib/api/schemas";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const TOTAL = 5;
  let paso = $state(1);
  let error = $state<AppError | null>(null);

  const sinSmart = (d: DiskSummary) =>
    d.state === "unknown" && (d.unknownReason ?? "unsupported") === "unsupported";

  /* -------------------------------------------------------------------------------- paso 2: discos */

  const discos = $derived(data.devices);
  // Compatibles marcados de inicio (FR-034); el USB sin SMART se puede incluir pero arranca sin marcar.
  let incluido = $state<Record<string, boolean>>({});
  let alias = $state<Record<string, string>>({});
  $effect(() => {
    for (const d of discos) {
      if (!(d.id in incluido)) incluido[d.id] = !sinSmart(d);
      if (!(d.id in alias)) alias[d.id] = d.alias ?? "";
    }
  });
  const seleccionados = $derived(discos.filter((d) => incluido[d.id]).length);

  async function aplicarPaso2() {
    error = null;
    try {
      for (const d of discos) {
        const nombre = alias[d.id]?.trim() ?? "";
        if (nombre !== (d.alias ?? "")) await setDeviceAlias(d.id, nombre || null);
        await setDeviceMonitoring(d.id, !!incluido[d.id]);
      }
      paso = 3;
    } catch (cause) {
      error = toAppError(cause);
    }
  }

  /* ------------------------------------------------------------------------------- paso 3: alertas */

  let settings = $state<SettingsShape | null>(null);
  $effect(() => {
    if (settings === null && data.settings) settings = data.settings;
  });
  const perfilElegido = $derived<PerfilAlerta | "custom">(settings?.alerts.profile ?? PERFIL_RECOMENDADO);
  const perfilRadio = PERFILES.map((p) => ({ id: p.id, label: t(p.label), hint: t(p.hint) }));

  async function elegirPerfil(id: string) {
    error = null;
    try {
      await setSetting("alerts.profile", id);
      settings = await getSettings();
    } catch (cause) {
      error = toAppError(cause);
    }
  }

  async function cambiarSwitch(clave: string, valor: boolean) {
    error = null;
    try {
      await setSetting(clave, valor);
      settings = await getSettings();
    } catch (cause) {
      error = toAppError(cause);
    }
  }

  const umbrales = $derived(
    settings
      ? [
          { k: "settings.alerts.tempWarn", v: `${settings.alerts.tempConfiguredWarnC} °C` },
          { k: "settings.alerts.tempCrit", v: `${settings.alerts.tempConfiguredCritC} °C` },
          { k: "settings.alerts.wearWarn", v: `${settings.alerts.wearWarnPercent} %` },
          { k: "settings.alerts.wearCrit", v: `${settings.alerts.wearCritPercent} %` },
          { k: "settings.alerts.capacityWarnPercent", v: `${settings.alerts.capacityWarnPercent} %` },
          { k: "settings.alerts.capacityCritPercent", v: `${settings.alerts.capacityCritPercent} %` },
          { k: "settings.alerts.mediaErrorsWarn", v: String(settings.alerts.mediaErrorsWarnPer24h) },
          { k: "settings.alerts.mediaErrorsCrit", v: String(settings.alerts.mediaErrorsCritPer24h) }
        ]
      : []
  );

  /* ---------------------------------------------------------------------- paso 4: ayuda con IA (opcional) */

  let claveIa = $state("");
  let activandoIa = $state(false);
  let errorIa = $state<AppError | null>(null);

  async function activarIa() {
    if (claveIa.trim().length === 0) {
      paso = 5;
      return;
    }
    activandoIa = true;
    errorIa = null;
    try {
      ia.set(await guardarClaveIa(claveIa.trim()));
      claveIa = "";
      // No se avanza: se muestra el selector de modelo; el pie pasa a «Continuar».
    } catch (cause) {
      errorIa = toAppError(cause);
    } finally {
      activandoIa = false;
    }
  }

  async function cambiarModeloIaAsistente(id: string) {
    errorIa = null;
    try {
      await setSetting("settings.ai.model", id);
      await ia.refrescar();
    } catch (cause) {
      errorIa = toAppError(cause);
    }
  }

  /* --------------------------------------------------------------------------------- paso 5: listo */

  let primeraLecturaLanzada = $state(false);
  $effect(() => {
    if (paso === 5 && !primeraLecturaLanzada) {
      primeraLecturaLanzada = true;
      refreshNow("all").catch(() => {});
    }
  });

  /* ---------------------------------------------------------------------------- navegación y salida */

  function marcarCompletado() {
    return setSetting("settings.onboarding.completed_at", new Date().toISOString());
  }

  async function omitir() {
    error = null;
    try {
      await setSetting("alerts.profile", PERFIL_RECOMENDADO);
      await marcarCompletado();
      await goto("/");
    } catch (cause) {
      error = toAppError(cause);
    }
  }

  async function terminar() {
    error = null;
    try {
      await marcarCompletado();
      await goto("/");
    } catch (cause) {
      error = toAppError(cause);
    }
  }

  async function reintentarDeteccion() {
    error = null;
    try {
      await refreshNow("all");
    } catch {
      // el error de fuente ya lo cuenta el bloque; `invalidateAll` recarga el `load`
    }
    await invalidateAll();
  }
</script>

<div class="flex min-h-screen flex-col bg-bg">
  <!-- Cabecera propia de 56 px (FR-033): logo, indicador de paso, salida siempre visible. -->
  <header class="flex h-14 flex-none items-center gap-3 border-b border-hairline px-6">
    <span class="sdm-display text-sm font-semibold">SmartDisk Monitor</span>
    <div class="flex flex-1 items-center justify-center gap-2" aria-label={t("onboarding.progress")}>
      <span class="flex items-center gap-1 max-[860px]:hidden">
        {#each Array.from({ length: TOTAL }) as _, i}
          <span
            class="inline-block size-1.5 rounded-pill"
            style="background: {i < paso ? 'var(--sdm-accent)' : 'var(--sdm-glass-3)'}"
          ></span>
        {/each}
      </span>
      <span class="text-2xs text-fg-dim">{t("onboarding.stepIndicator", { n: paso, total: TOTAL })}</span>
    </div>
    <Button variant="ghost" size="sm" onclick={omitir}>{t("onboarding.skip")}</Button>
  </header>

  <main class="mx-auto flex w-full max-w-[1000px] flex-1 flex-col gap-5 p-6">
    {#if error}
      <div class="rounded-inner bg-crit-soft p-4 text-sm text-crit" role="alert">
        {t(error.messageKey, error.messageVars)}
      </div>
    {/if}

    {#if paso === 1}
      <div class="mx-auto flex max-w-[620px] flex-col items-center gap-4 py-6 text-center">
        <OnboardingArt name="welcome" />
        <h1 class="sdm-display m-0 text-2xl">{t("onboarding.welcome.title")}</h1>
        <p class="m-0 text-sm text-fg-dim" style="text-wrap: pretty">{t("onboarding.welcome.body")}</p>
        <p
          class="m-0 flex items-center gap-2 rounded-inner bg-ok-soft px-4 py-3 text-sm"
          style="text-wrap: pretty"
        >
          <span class="shrink-0 text-ok"><Icon name="shield" size={16} /></span>
          {t("onboarding.welcome.guarantee")}
        </p>
      </div>
      <div class="grid gap-3 sm:grid-cols-3">
        {#each [["shield", "onboarding.welcome.read"], ["alert", "onboarding.welcome.warn"], ["flask", "onboarding.welcome.test"]] as [icono, clave]}
          <Card>
            <span class="grid size-9 place-items-center rounded-nav bg-accent-soft text-accent-fg">
              <Icon name={icono as "shield"} size={18} />
            </span>
            <span class="text-xs text-fg-dim" style="text-wrap: pretty">{t(clave)}</span>
          </Card>
        {/each}
      </div>
    {:else if paso === 2}
      <div class="flex items-center gap-6">
        <div class="flex flex-col gap-1">
          <h1 class="sdm-display m-0 text-2xl">
            {t("onboarding.disks.title", { count: discos.length })}
          </h1>
          <p class="m-0 text-xs text-fg-dim" style="text-wrap: pretty">{t("onboarding.disks.body")}</p>
        </div>
        <div class="ml-auto shrink-0 max-[720px]:hidden"><OnboardingArt name="disks" width={150} /></div>
      </div>

      {#if data.error}
        <EmptyState
          kind="error"
          title={t("onboarding.disks.error")}
          body={t(data.error.messageKey, data.error.messageVars)}
          detail={data.error.detail ?? ""}
          actionLabel={t("common.retry")}
          onaction={reintentarDeteccion}
        />
      {:else if discos.length === 0}
        <EmptyState
          kind="empty"
          title={t("onboarding.disks.empty")}
          body=""
          actionLabel={t("onboarding.disks.rescan")}
          onaction={reintentarDeteccion}
        />
      {:else}
        <div class="flex flex-col gap-3">
          {#each discos as d (d.id)}
            <div
              class="flex flex-wrap items-center gap-4 rounded-inner border p-4 transition-all duration-base ease-sdm
                     {incluido[d.id] ? 'border-accent' : 'border-hairline'}"
            >
              <input
                type="checkbox"
                class="size-6 shrink-0 accent-[var(--sdm-accent)]"
                checked={incluido[d.id]}
                aria-label={t("onboarding.disks.include", { name: d.alias ?? d.model })}
                onchange={(e) => (incluido[d.id] = e.currentTarget.checked)}
              />
              <span class="grid size-10 shrink-0 place-items-center rounded-nav bg-glass-3 text-fg-dim">
                <Icon name={busIcon(d.deviceType)} size={20} />
              </span>
              <div class="flex min-w-0 flex-1 flex-col gap-1">
                <span class="sdm-selectable truncate text-sm font-semibold">{d.model}</span>
                <span class="sdm-selectable text-2xs text-fg-dim">{d.deviceType}</span>
              </div>
              <div class="w-48">
                <!-- Sin etiqueta visible: con la etiqueta encima, la fila dejaba de centrarse y la
                     píldora de estado quedaba descolocada. El texto va como marcador y como nombre
                     accesible; el intro de la sección ya explica para qué es el campo. -->
                <TextField
                  ariaLabel={t("onboarding.disks.aliasLabel")}
                  placeholder={t("onboarding.disks.aliasLabel")}
                  value={alias[d.id] ?? ""}
                  oninput={(v: string) => (alias[d.id] = v)}
                />
              </div>
              <div class="w-52">
                {#if d.volumes[0]}
                  <CapacityBar
                    label={d.volumes[0].driveLetters.join(", ") || d.volumes[0].label}
                    capacityBytes={d.volumes[0].capacityBytes}
                    freeBytes={d.volumes[0].freeBytes}
                  />
                {/if}
              </div>
              <StatusPill
                state={sinSmart(d) ? "unknown" : "ok"}
                label={sinSmart(d) ? t("disk.noSmartData") : t("health.ok")}
                icon="auto"
              />
            </div>
          {/each}
        </div>

        {#if discos.some(sinSmart)}
          <div class="rounded-inner bg-glass-3 p-4 text-xs leading-relaxed" style="text-wrap: pretty">
            <strong>{t("common.unsupported")}.</strong>
            {t("onboarding.disks.usbNote")}
          </div>
        {/if}
      {/if}
    {:else if paso === 3}
      <div class="flex items-center gap-6">
        <h1 class="sdm-display m-0 text-2xl">{t("onboarding.alerts.title")}</h1>
        <div class="ml-auto shrink-0 max-[720px]:hidden"><OnboardingArt name="alerts" width={150} /></div>
      </div>

      <div class="flex flex-col gap-2" role="radiogroup" aria-label={t("onboarding.alerts.title")}>
        {#each perfilRadio as p (p.id)}
          <button
            type="button"
            role="radio"
            aria-checked={perfilElegido === p.id}
            class="flex flex-col gap-1 rounded-inner border p-4 text-left transition-all duration-base ease-sdm
                   {perfilElegido === p.id
              ? 'border-accent bg-accent-soft'
              : 'border-hairline hover:bg-glass'}"
            onclick={() => elegirPerfil(p.id)}
          >
            <span class="text-sm font-semibold">{p.label}</span>
            <span class="text-xs text-fg-dim">{p.hint}</span>
          </button>
        {/each}
      </div>

      {#if umbrales.length}
        <details class="rounded-inner bg-glass-3 p-4 text-xs">
          <summary class="cursor-pointer font-medium">{t("onboarding.alerts.showThresholds")}</summary>
          <dl class="mt-3 grid grid-cols-2 gap-x-6 gap-y-1.5">
            {#each umbrales as u}
              <div class="flex justify-between gap-2">
                <dt class="text-fg-dim">{t(u.k)}</dt>
                <dd class="sdm-num m-0 font-medium">{u.v}</dd>
              </div>
            {/each}
          </dl>
        </details>
      {/if}

      <div class="flex flex-col gap-2">
        <Switch
          checked={settings?.notifications.enabled ?? true}
          label={t("onboarding.alerts.notifyWindows")}
          hint={t("onboarding.alerts.notifyWindowsHint")}
          onchange={(v: boolean) => cambiarSwitch("notifications.enabled", v)}
        />
        <Switch
          checked={settings?.lifecycle.startWithSystem ?? false}
          label={t("onboarding.alerts.startWithSystem")}
          hint={t("onboarding.alerts.startWithSystemHint")}
          onchange={(v: boolean) => cambiarSwitch("lifecycle.start_with_system", v)}
        />
      </div>
    {:else if paso === 4}
      <div class="flex items-center gap-6">
        <h1 class="sdm-display m-0 text-2xl">{t("onboarding.ai.title")}</h1>
      </div>
      <p class="m-0 max-w-[560px] text-sm leading-relaxed text-fg-dim" style="text-wrap: pretty">
        {t("onboarding.ai.body")}
      </p>
      <div class="flex flex-col gap-3 rounded-inner bg-glass-3 p-4">
        <p class="m-0 text-xs font-semibold text-fg-dim">{t("onboarding.ai.exampleTitle")}</p>
        <p class="m-0 text-xs text-fg-dim" style="text-wrap: pretty">{t("onboarding.ai.example")}</p>
      </div>
      {#if ia.estado?.activa}
        <p class="m-0 text-sm font-medium text-ok">{t("settings.ai.status.on")}</p>
        <AiModelSelect modelo={ia.estado.modelo} onchange={(id) => void cambiarModeloIaAsistente(id)} />
      {:else}
        <TextField
          value={claveIa}
          label={t("settings.ai.key.label")}
          hint={t("settings.ai.key.hint")}
          placeholder={t("settings.ai.key.placeholder")}
          error={errorIa ? t(errorIa.messageKey) : ""}
          disabled={activandoIa}
          oninput={(v: string) => {
            claveIa = v;
          }}
        />
      {/if}
      {#if errorIa && errorIa.detail}
        <details>
          <summary class="cursor-pointer text-xs font-semibold text-fg-dim">
            {t("common.technicalDetail")}
          </summary>
          <pre
            class="mt-2 overflow-x-auto whitespace-pre-wrap rounded-inner bg-glass-3 p-3 text-xs text-fg-dim">{errorIa.detail}</pre>
        </details>
      {/if}
      <p class="m-0 text-2xs text-fg-faint" style="text-wrap: pretty">{t("onboarding.ai.skipHint")}</p>
    {:else}
      <div class="mx-auto flex max-w-[560px] flex-col items-center gap-4 py-6 text-center">
        <OnboardingArt name="done" />
        <h1 class="sdm-display m-0 text-2xl">{t("onboarding.done.title")}</h1>
        <p class="m-0 text-sm text-fg-dim">
          {t("onboarding.done.watching", {
            count: seleccionados,
            profile: t(perfilLabelKey(perfilElegido))
          })}
        </p>
        <p class="m-0 text-sm text-fg-dim">
          {(settings?.notifications.enabled ?? true)
            ? t("onboarding.done.notifyOn")
            : t("onboarding.done.notifyOff")}
        </p>
        <div class="w-full max-w-[360px]">
          <ProgressBar indeterminate caption={t("onboarding.done.firstScan")} trailing="" />
        </div>
        <p class="m-0 text-2xs text-fg-faint">{t("onboarding.done.footnote")}</p>
      </div>
    {/if}
  </main>

  <!-- Pie de navegación pegado abajo (FR/ficha §6): «Continuar» nunca queda fuera de la vista. -->
  <footer
    class="sdm-material-chrome sticky bottom-0 flex flex-none items-center gap-3 border-t border-hairline px-6 py-3"
  >
    {#if paso > 1}
      <Button variant="ghost" onclick={() => (paso -= 1)}>{t("common.back")}</Button>
    {/if}
    <div class="flex flex-1 items-center justify-center text-2xs text-fg-dim">
      {#if paso === 2 && discos.length}
        {t("onboarding.selectedCount", { selected: seleccionados, total: discos.length })}
      {/if}
    </div>
    {#if paso === 1}
      <Button variant="primary" onclick={() => (paso = 2)}>{t("onboarding.welcome.cta")}</Button>
    {:else if paso === 2}
      <Button variant="primary" onclick={aplicarPaso2}>{t("onboarding.disks.cta")}</Button>
    {:else if paso === 3}
      <Button variant="primary" onclick={() => (paso = 4)}>{t("common.continue")}</Button>
    {:else if paso === 4}
      {#if ia.estado?.activa}
        <Button variant="primary" onclick={() => (paso = 5)}>{t("common.continue")}</Button>
      {:else}
        <Button variant="primary" loading={activandoIa} onclick={activarIa}>
          {claveIa.trim().length > 0 ? t("onboarding.ai.cta.activate") : t("onboarding.ai.cta.skip")}
        </Button>
      {/if}
    {:else}
      <Button variant="primary" onclick={terminar}>{t("onboarding.done.cta")}</Button>
    {/if}
  </footer>
</div>
