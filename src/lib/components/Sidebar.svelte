<script lang="ts">
  /** Barra lateral de material (v2): secciones de navegación y lista de discos monitorizados
   *  con su punto de estado y temperatura. Sustituye a las pestañas superiores de v1.
   *  La selección se marca con material elevado + punto de acento, nunca con una barra de color.
   *
   *  Navega con **enlaces reales** (constitución §XIV). Un `onclick` con `goto()` destruiría el
   *  ctrl+clic, el menú contextual y el anuncio como enlace de un lector de pantalla. */
  import StatusDot from "./StatusDot.svelte";
  import Button from "./Button.svelte";
  import type { DiskSummary } from "$lib/design/types";
  import { formatTemperature, NOT_AVAILABLE } from "$lib/design/format";
  import { t } from "$lib/i18n";

  let {
    /** Cada sección lleva su destino: la navegación es un enlace, no un callback. */
    sections = [] as { id: string; label: string; href: string; badge?: number | null }[],
    disks = [] as DiskSummary[],
    /** Construye el destino de cada disco. Devuelve el href; no navega. */
    diskHref = ((id: string) => `/disks/${id}`) as (id: string) => string,
    active = "",
    activeDiskId = "",
    /** "1 disco necesita atención", "Monitorización pausada"… */
    footerNote = "",
    paused = false,
    ontogglepause = undefined
  } = $props();

  const row = (isActive: boolean) =>
    "flex h-8 items-center gap-3 rounded-nav border px-3 text-xs transition-all duration-base ease-sdm " +
    (isActive
      ? "border-hairline bg-glass-2 text-fg font-semibold shadow-edge"
      : "border-transparent text-fg-dim font-medium hover:bg-glass-3 hover:text-fg");
</script>

<!-- Por debajo de 1180 px de ancho CSS disponible (el escalado de Windows no encoge el texto,
     encoge ese espacio: `docs/ui-design.md` §4.0.bis), la barra se reduce a 56 px: solo el
     logotipo, un marcador circular con la inicial de cada sección (sin icono propio todavía — el
     boceto aprobado no define ninguno; usar la inicial evita inventar iconografía sin revisión) y
     el punto de estado de cada disco, que es lo único que la norma exige conservar. El pie de
     pausar/reanudar se oculta: la misma acción sigue disponible desde el menú de la bandeja. -->
<aside
  class="sdm-material-chrome flex w-[250px] max-[1179px]:w-14 flex-none flex-col border-r border-hairline"
>
  <div
    class="flex h-13 flex-none items-center gap-3 px-4 max-[1179px]:justify-center max-[1179px]:px-0"
    style="height: 52px"
  >
    <div
      class="grid size-6 shrink-0 place-items-center rounded-nav bg-[linear-gradient(180deg,var(--sdm-accent-hi),var(--sdm-accent))] shadow-[inset_0_1px_0_rgba(255,255,255,.45)]"
    >
      <span class="block size-2 rounded-pill border-2 border-white/95"></span>
    </div>
    <span class="text-base font-semibold tracking-tight max-[1179px]:hidden">SmartDisk</span>
  </div>

  <p
    class="m-0 px-3 pb-1 pt-2 text-2xs font-semibold uppercase tracking-wider text-fg-faint max-[1179px]:hidden"
  >
    {t("nav.monitoring")}
  </p>
  <nav class="flex flex-col gap-0.5 px-2" aria-label={t("nav.monitoring")}>
    {#each sections as s (s.id)}
      <a
        class={row(active === s.id) + " max-[1179px]:justify-center max-[1179px]:px-0"}
        href={s.href}
        aria-current={active === s.id ? "page" : undefined}
        aria-label={s.label}
        title={s.label}
      >
        <span
          class="hidden size-5 shrink-0 place-items-center rounded-pill bg-glass-3 text-2xs font-semibold max-[1179px]:grid"
          aria-hidden="true"
        >
          {s.label.charAt(0).toUpperCase()}
        </span>
        <span
          class="size-1.5 shrink-0 rounded-pill max-[1179px]:hidden"
          style="background: {active === s.id ? 'var(--sdm-accent)' : 'var(--sdm-text-faint)'}"
        ></span>
        <span class="flex-1 text-left max-[1179px]:hidden">{s.label}</span>
        {#if s.badge}
          <span class="rounded-pill bg-warn-soft px-2 text-2xs font-semibold text-warn max-[1179px]:hidden"
            >{s.badge}</span
          >
        {/if}
      </a>
    {/each}
  </nav>

  {#if disks.length}
    <p
      class="m-0 px-3 pb-1 pt-4 text-2xs font-semibold uppercase tracking-wider text-fg-faint max-[1179px]:hidden"
    >
      {t("nav.monitoredDisks")}
    </p>
    <!-- Scroll propio: con veinte discos, la navegación y el pie no se desplazan con la lista. -->
    <nav class="flex min-h-0 flex-col gap-0.5 overflow-y-auto px-2" aria-label={t("nav.monitoredDisks")}>
      {#each disks as d (d.id)}
        <a
          class={row(activeDiskId === d.id) + " max-[1179px]:justify-center max-[1179px]:px-0"}
          href={diskHref(d.id)}
          aria-current={activeDiskId === d.id ? "page" : undefined}
          aria-label={d.alias ?? d.model}
          title={d.alias ?? d.model}
        >
          <StatusDot state={d.state} label="" size={8} />
          <span class="flex-1 truncate text-left max-[1179px]:hidden">{d.alias ?? d.model}</span>
          <span class="sdm-num text-2xs text-fg-faint max-[1179px]:hidden">
            {d.temperatureC === null ? NOT_AVAILABLE() : formatTemperature(d.temperatureC)}
          </span>
        </a>
      {/each}
    </nav>
  {/if}

  <div class="flex-1"></div>

  <div
    class="m-3 flex flex-col gap-2 rounded-inner border border-hairline bg-glass-2 p-3 shadow-edge max-[1179px]:hidden"
  >
    {#if footerNote}<span class="text-xs text-fg-dim">{footerNote}</span>{/if}
    <Button size="sm" full onclick={ontogglepause}>
      {paused ? t("nav.resume") : t("nav.pause")}
    </Button>
  </div>
</aside>
