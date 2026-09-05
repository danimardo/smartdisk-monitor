<script lang="ts">
  /** Pantalla de ajustes (US-070/071/072/073, épica H de `docs/user-stories.md`).
   *
   *  Sin boceto aprobado, igual que informes (T092): compuesta con el catálogo actual según
   *  `docs/ui-design.md` Apéndice C, mismo criterio ya autorizado por el usuario.
   *
   *  Apariencia (tema/idioma/acento) sigue teniendo su propio `get_appearance_settings()`, pero se
   *  persiste con el mismo `set_setting` genérico que el resto (`docs/open-questions.md` J.32):
   *  esta pantalla es el primer consumidor real de `theme.set()`/`i18n.set()`, que ya devolvían
   *  `{key, value}` a la espera de quien los llamara y los guardara.
   */
  import {
    Button,
    Card,
    ConfirmDialog,
    RadioGroup,
    SegmentedControl,
    Switch,
    TextField
  } from "$lib/components";
  import { deleteAllData, openLogFolder, resetSettings, setLogLevel, setSetting, toAppError } from "$lib/api";
  import { theme } from "$lib/design/theme.svelte";
  import { applySystemAccent, clearSystemAccent } from "$lib/design/accent";
  import { formatBytes } from "$lib/design/format";
  import { i18n, t, type Locale } from "$lib/i18n";
  import type { SettingsShape } from "$lib/api/schemas";
  import type { AppError } from "$lib/design/types";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  /** Mismo guardián que `/tests` y `/`: solo toma el valor inicial de `load` una vez; a partir de
   *  ahí manda el estado local, que las acciones de esta pantalla actualizan de forma optimista.
   *  Arranca en `null` (nunca leyendo `data` en el propio inicializador de `$state`) y la
   *  plantilla espera a que el efecto lo rellene, igual que el resto de pantallas espera a
   *  `app.loadedAt`. */
  let settings = $state<SettingsShape | null>(null);
  $effect(() => {
    if (settings === null) settings = data.settings;
  });

  let saveError = $state<AppError | null>(null);

  /* -------------------------------------------------------------------------------- apariencia */

  let themePref = $state(theme.preference);
  let languagePref = $state<Locale>(i18n.locale);
  let useSystemAccent = $state(true);

  const opcionesTema = [
    { id: "light", label: t("settings.appearance.theme.light") },
    { id: "dark", label: t("settings.appearance.theme.dark") },
    { id: "system", label: t("settings.appearance.theme.system") }
  ];
  const opcionesIdioma = [
    { id: "es", label: t("settings.appearance.language.es") },
    { id: "en", label: t("settings.appearance.language.en") }
  ];

  async function cambiarTema(next: "light" | "dark" | "system") {
    themePref = next;
    const { key, value } = theme.set(next);
    try {
      await setSetting(key, value);
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  async function cambiarIdioma(next: Locale) {
    languagePref = next;
    const { key, value } = i18n.set(next);
    try {
      await setSetting(key, value);
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  async function cambiarAcentoSistema(activo: boolean) {
    useSystemAccent = activo;
    try {
      await setSetting("settings.appearance.use_system_accent", activo);
      if (activo) await applySystemAccent();
      else clearSystemAccent();
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  async function cambiarSonido(activo: boolean) {
    if (!settings) return;
    try {
      await setSetting("notifications.sound_enabled", activo);
      settings = { ...settings, notifications: { soundEnabled: activo } };
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  /* ------------------------------------------------------------------------------- frecuencias */

  type ClaveFrecuencia =
    | "schedule.metrics_fast_seconds"
    | "schedule.smart_full_seconds"
    | "schedule.events_seconds"
    | "schedule.discovery_seconds";

  const FRECUENCIAS: {
    clave: ClaveFrecuencia;
    campo: keyof SettingsShape["schedule"];
    labelKey: string;
    min: number;
    max: number;
  }[] = [
    {
      clave: "schedule.metrics_fast_seconds",
      campo: "metricsFastSeconds",
      labelKey: "settings.schedule.metricsFast",
      min: 10,
      max: 300
    },
    {
      clave: "schedule.smart_full_seconds",
      campo: "smartFullSeconds",
      labelKey: "settings.schedule.smartFull",
      min: 60,
      max: 3600
    },
    {
      clave: "schedule.events_seconds",
      campo: "eventsSeconds",
      labelKey: "settings.schedule.events",
      min: 15,
      max: 300
    },
    {
      clave: "schedule.discovery_seconds",
      campo: "discoverySeconds",
      labelKey: "settings.schedule.discovery",
      min: 30,
      max: 600
    }
  ];

  /** `setSetting` no devuelve el valor guardado (`ui-contract.md` §3.1: `invoke<void>`): estas
   *  tres funciones actualizan el estado local de forma optimista tras guardar, y si el backend
   *  rechaza el valor, `saveError` lo dice — el campo vuelve a su valor real en el próximo
   *  `get_settings` (recarga manual). Una función por grupo, no una genérica con indexado
   *  dinámico: TypeScript no puede esparcir un tipo indexado con seguridad, y el indexado dinámico
   *  sobre `settings` perdería la comprobación de qué campo pertenece a qué grupo. */
  async function cambiarSchedule(campo: keyof SettingsShape["schedule"], clave: string, valor: number) {
    if (!settings) return;
    saveError = null;
    try {
      await setSetting(clave, valor);
      settings = { ...settings, schedule: { ...settings.schedule, [campo]: valor } };
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  async function cambiarAlerta(campo: keyof SettingsShape["alerts"], clave: string, valor: number) {
    if (!settings) return;
    saveError = null;
    try {
      await setSetting(clave, valor);
      settings = { ...settings, alerts: { ...settings.alerts, [campo]: valor } };
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  async function cambiarRetencion(campo: keyof SettingsShape["retention"], clave: string, valor: number) {
    if (!settings) return;
    saveError = null;
    try {
      await setSetting(clave, valor);
      settings = { ...settings, retention: { ...settings.retention, [campo]: valor } };
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  /* --------------------------------------------------------------------------------- registro */

  async function cambiarModoDetallado(activo: boolean) {
    if (!settings) return;
    saveError = null;
    try {
      await setLogLevel(activo);
      settings = { ...settings, logging: { verbose: activo } };
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  let abriendoCarpeta = $state(false);
  async function abrirCarpetaRegistro() {
    abriendoCarpeta = true;
    try {
      await openLogFolder();
    } catch (cause) {
      saveError = toAppError(cause);
    } finally {
      abriendoCarpeta = false;
    }
  }

  /* ---------------------------------------------------------------------------- ciclo de vida */

  const opcionesCierre = [
    {
      id: "minimize",
      label: t("settings.lifecycle.closeAction.minimize"),
      hint: t("settings.lifecycle.closeAction.minimizeHint")
    },
    {
      id: "exit",
      label: t("settings.lifecycle.closeAction.exit"),
      hint: t("settings.lifecycle.closeAction.exitHint")
    }
  ];

  async function cambiarAccionCierre(next: string) {
    if (!settings) return;
    saveError = null;
    try {
      await setSetting("lifecycle.close_action", next);
      settings = {
        ...settings,
        lifecycle: { ...settings.lifecycle, closeAction: next as "minimize" | "exit" }
      };
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  /* ----------------------------------------------------------------------------------- reinicio */

  async function restaurarAmbito(scope: "schedule" | "alerts" | "retention") {
    saveError = null;
    try {
      settings = await resetSettings(scope);
    } catch (cause) {
      saveError = toAppError(cause);
    }
  }

  /* ------------------------------------------------------------------------------ borrar todo */

  let dialogoBorrarAbierto = $state(false);
  let fraseEscrita = $state("");
  let borrando = $state(false);
  let borradoOk = $state(false);
  const FRASE_ESPERADA = "SmartDisk Monitor";

  async function confirmarBorrado() {
    borrando = true;
    try {
      await deleteAllData(fraseEscrita);
      borradoOk = true;
      dialogoBorrarAbierto = false;
    } catch (cause) {
      saveError = toAppError(cause);
    } finally {
      borrando = false;
      fraseEscrita = "";
    }
  }
</script>

<div class="flex flex-col gap-5 p-5">
  <h1 class="m-0 text-2xl font-semibold tracking-tight">{t("nav.settings")}</h1>

  {#if saveError}
    <div class="rounded-inner bg-crit-soft p-4 text-sm text-crit" role="alert">
      {t(saveError.messageKey, saveError.messageVars)}
    </div>
  {/if}

  {#if settings}
    <Card title={t("settings.appearance.title")}>
      <SegmentedControl
        options={opcionesTema}
        value={themePref}
        onchange={(v: "light" | "dark" | "system") => cambiarTema(v)}
      />
      <SegmentedControl
        options={opcionesIdioma}
        value={languagePref}
        onchange={(v: Locale) => cambiarIdioma(v)}
      />
      <Switch
        checked={useSystemAccent}
        label={t("settings.appearance.useSystemAccent.label")}
        hint={t("settings.appearance.useSystemAccent.hint")}
        onchange={cambiarAcentoSistema}
      />
      <Switch
        checked={settings.notifications.soundEnabled}
        label={t("settings.notifications.sound.label")}
        hint={t("settings.notifications.sound.hint")}
        onchange={cambiarSonido}
      />
    </Card>

    <Card title={t("settings.schedule.title")}>
      {#snippet action()}
        <Button variant="ghost" size="sm" onclick={() => restaurarAmbito("schedule")}>
          {t("settings.cta.restoreDefaults")}
        </Button>
      {/snippet}
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        {#each FRECUENCIAS as f (f.clave)}
          <TextField
            type="number"
            label={t(f.labelKey)}
            hint={t("settings.schedule.rangeHint", { min: f.min, max: f.max })}
            value={String(settings.schedule[f.campo])}
            min={f.min}
            max={f.max}
            suffix={t("settings.unit.seconds")}
            oninput={(v: string) => {
              const n = Number(v);
              if (Number.isFinite(n)) void cambiarSchedule(f.campo, f.clave, n);
            }}
          />
        {/each}
      </div>
    </Card>

    <Card title={t("settings.alerts.title")}>
      {#snippet action()}
        <Button variant="ghost" size="sm" onclick={() => restaurarAmbito("alerts")}>
          {t("settings.cta.restoreDefaults")}
        </Button>
      {/snippet}
      <p class="m-0 text-xs text-fg-dim" style="text-wrap: pretty">{t("settings.alerts.tempPrecedence")}</p>
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <TextField
          type="number"
          label={t("settings.alerts.tempWarn")}
          value={String(settings.alerts.tempConfiguredWarnC)}
          min={40}
          max={95}
          suffix="°C"
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n))
              void cambiarAlerta("tempConfiguredWarnC", "alerts.temp_configured_warn_c", n);
          }}
        />
        <TextField
          type="number"
          label={t("settings.alerts.tempCrit")}
          value={String(settings.alerts.tempConfiguredCritC)}
          min={40}
          max={100}
          suffix="°C"
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n))
              void cambiarAlerta("tempConfiguredCritC", "alerts.temp_configured_crit_c", n);
          }}
        />
        <TextField
          type="number"
          label={t("settings.alerts.capacityWarnPercent")}
          value={String(settings.alerts.capacityWarnPercent)}
          min={1}
          max={50}
          suffix="%"
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n))
              void cambiarAlerta("capacityWarnPercent", "alerts.capacity_warn_percent", n);
          }}
        />
        <TextField
          type="number"
          label={t("settings.alerts.capacityCritPercent")}
          value={String(settings.alerts.capacityCritPercent)}
          min={1}
          max={50}
          suffix="%"
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n))
              void cambiarAlerta("capacityCritPercent", "alerts.capacity_crit_percent", n);
          }}
        />
      </div>
    </Card>

    <Card title={t("settings.retention.title")}>
      {#snippet action()}
        <Button variant="ghost" size="sm" onclick={() => restaurarAmbito("retention")}>
          {t("settings.cta.restoreDefaults")}
        </Button>
      {/snippet}
      <p class="m-0 text-xs text-fg-dim" style="text-wrap: pretty">{t("settings.retention.neverPurged")}</p>
      <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <TextField
          type="number"
          label={t("settings.retention.raw")}
          value={String(settings.retention.rawDays)}
          min={1}
          max={30}
          suffix={t("settings.unit.days")}
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n)) void cambiarRetencion("rawDays", "retention.raw_days", n);
          }}
        />
        <TextField
          type="number"
          label={t("settings.retention.fiveMinutes")}
          value={String(settings.retention.fiveMinutesDays)}
          min={7}
          max={365}
          suffix={t("settings.unit.days")}
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n))
              void cambiarRetencion("fiveMinutesDays", "retention.five_minutes_days", n);
          }}
        />
        <TextField
          type="number"
          label={t("settings.retention.hourly")}
          value={String(settings.retention.hourlyDays)}
          min={90}
          max={1825}
          suffix={t("settings.unit.days")}
          oninput={(v: string) => {
            const n = Number(v);
            if (Number.isFinite(n)) void cambiarRetencion("hourlyDays", "retention.hourly_days", n);
          }}
        />
      </div>
      <span class="text-xs text-fg-dim">
        {t("settings.retention.freeSpaceGuard", {
          warn: formatBytes(settings.retention.freeSpaceWarnBytes),
          halt: formatBytes(settings.retention.freeSpaceHaltBytes)
        })}
      </span>
    </Card>

    <Card title={t("settings.logging.title")}>
      <Switch
        checked={settings.logging.verbose}
        label={t("settings.logging.verbose.label")}
        hint={t("settings.logging.verbose.hint")}
        onchange={cambiarModoDetallado}
      />
      <Button variant="secondary" loading={abriendoCarpeta} onclick={abrirCarpetaRegistro}>
        {t("settings.logging.cta.openFolder")}
      </Button>
    </Card>

    <Card title={t("settings.lifecycle.title")}>
      <RadioGroup
        label={t("settings.lifecycle.closeAction.label")}
        options={opcionesCierre}
        value={settings.lifecycle.closeAction}
        onchange={cambiarAccionCierre}
      />
    </Card>

    <Card title={t("settings.dangerZone.title")}>
      <p class="m-0 text-xs leading-relaxed text-fg-dim" style="text-wrap: pretty">
        {t("settings.dangerZone.desc")}
      </p>
      {#if borradoOk}
        <div class="rounded-inner bg-ok-soft p-4 text-sm text-fg">
          {t("settings.dangerZone.done")}
        </div>
      {:else}
        <TextField
          label={t("settings.dangerZone.confirmPhraseLabel", { phrase: FRASE_ESPERADA })}
          value={fraseEscrita}
          oninput={(v: string) => (fraseEscrita = v)}
          disabled={borrando}
        />
        <Button
          variant="danger"
          disabled={fraseEscrita !== FRASE_ESPERADA}
          onclick={() => (dialogoBorrarAbierto = true)}
        >
          {t("settings.dangerZone.cta")}
        </Button>
      {/if}
    </Card>
  {/if}
</div>

<ConfirmDialog
  open={dialogoBorrarAbierto}
  title={t("settings.dangerZone.confirmTitle")}
  body={t("settings.dangerZone.confirmBody")}
  impact={t("settings.dangerZone.confirmImpact")}
  confirmLabel={t("settings.dangerZone.confirmLabel")}
  destructive
  onconfirm={confirmarBorrado}
  oncancel={() => (dialogoBorrarAbierto = false)}
/>
