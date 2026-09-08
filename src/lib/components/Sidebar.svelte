<script lang="ts">
  /** Riel de navegación (v3, ADR-034). Sustituye a la barra de 250 px con etiquetas y lista de
   *  discos por un riel de 74 px solo con iconos: devuelve 176 px de ancho al contenido, que es
   *  donde estaba el problema a 1024 px.
   *
   *  Sin logo de marca aparte (a diferencia del boceto `Sidebar.md`): en un riel de solo iconos
   *  duplicaba a «Panel general», que ya va a `/` con el mismo icono (`open-questions.md` J.54).
   *
   *  Un riel sin texto solo es aceptable si la accesibilidad es impecable (`Sidebar.md` §Accesibilidad):
   *  cada botón lleva `title` **y** `aria-label`, el activo `aria-current="page"`, el punto de aviso
   *  no viaja solo (el `aria-label` de Alertas incluye el recuento) y el indicador de estado global
   *  del pie es `role="status"`.
   *
   *  Navega con **enlaces reales** (constitución §XIV): un `onclick` con `goto()` rompería ctrl+clic,
   *  el clic central, el menú contextual y el anuncio como enlace.
   *
   *  La selección se marca con material elevado e icono en acento — **nunca** una barra de color
   *  lateral (`ui-design.md`). */
  import Icon from "./Icon.svelte";
  import { healthToken } from "$lib/design/health";
  import type { IconName } from "$lib/design/icons";
  import type { HealthState } from "$lib/design/types";
  import { t } from "$lib/i18n";

  let {
    /** Cada sección lleva su icono y su destino: la navegación es un enlace, no un callback. */
    sections = [] as {
      id: string;
      label: string;
      icon: IconName;
      href: string;
      badge?: number | null;
    }[],
    active = "",
    /** De `globalStatus()` en `health.ts`: la misma fuente que alimenta la píldora de la `Toolbar`. */
    globalState = "unknown" as HealthState,
    globalLabel = "",
    globalIcon = "shield" as IconName,
    globalCount = null as number | null,
    onabout = undefined as (() => void) | undefined
  } = $props();

  const tone = $derived(healthToken[globalState]);

  const btn = (isActive: boolean) =>
    "relative grid size-11 shrink-0 place-items-center rounded-inner border transition-all duration-base ease-sdm " +
    (isActive
      ? "border-hairline bg-glass-2 text-accent-fg shadow-edge"
      : "border-transparent text-fg-dim hover:bg-glass-3 hover:text-fg");

  /** «Alertas» con avisos sin revisar: el número entra en el nombre accesible, no solo en el punto. */
  const labelDe = (s: { id: string; label: string; badge?: number | null }) =>
    s.id === "/alerts" && s.badge ? t("nav.alertsUnread", { count: s.badge }) : s.label;
</script>

<aside
  class="sdm-material-chrome flex w-rail flex-none flex-col items-center gap-2 border-r border-hairline py-4"
>
  <!-- Sin logo de marca aparte: en un riel de solo iconos duplicaba a «Panel general», que ya
       lleva a `/` con el mismo icono del disco. La identidad de la app vive en la barra de título
       y en «Acerca de». -->
  <nav class="flex flex-col items-center gap-2" aria-label={t("nav.monitoring")}>
    {#each sections as s (s.id)}
      <a
        class={btn(active === s.id)}
        href={s.href}
        aria-current={active === s.id ? "page" : undefined}
        aria-label={labelDe(s)}
        title={s.label}
      >
        <Icon name={s.icon} size={19} />
        {#if s.badge}
          <span class="absolute right-1.5 top-1.5 size-1.5 rounded-pill bg-warn" aria-hidden="true"></span>
        {/if}
      </a>
    {/each}
  </nav>

  <div class="flex-1"></div>

  {#if onabout}
    <button class={btn(false)} aria-label={t("nav.about")} title={t("nav.about")} onclick={onabout}>
      <Icon name="tag" size={19} />
    </button>
  {/if}

  <div
    class="mt-1 grid size-10 shrink-0 place-items-center rounded-inner"
    style="background: {tone.soft}; color: {tone.fg}"
    role="status"
    aria-label={globalLabel}
    title={globalLabel}
  >
    {#if globalCount}
      <Icon name={globalIcon} size={15} />
      <span class="sdm-num text-2xs font-semibold leading-none">{globalCount}</span>
    {:else}
      <Icon name={globalIcon} size={18} />
    {/if}
  </div>
</aside>
