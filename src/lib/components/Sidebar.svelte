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
    onabout = undefined as (() => void) | undefined,
    /** Riel expandible (spec 013): controlado desde `+layout.svelte`, que también persiste la
     *  preferencia entre sesiones — este componente no conoce `$lib/api` ni decide si se guarda. */
    expanded = false,
    onToggleExpand = undefined as (() => void) | undefined,
    /** Pliega el panel al elegir una sección (o «Acerca de»), sin tocar la preferencia guardada:
     *  es una consecuencia de navegar, no un "ya no lo quiero expandido" explícito. */
    onSelect = undefined as (() => void) | undefined
  } = $props();

  const tone = $derived(healthToken[globalState]);

  const btn = (isActive: boolean) =>
    "relative grid size-11 shrink-0 place-items-center rounded-inner border transition-all duration-base ease-sdm " +
    (isActive
      ? "border-hairline bg-glass-2 text-accent-fg shadow-edge"
      : "border-transparent text-fg-dim hover:bg-glass-3 hover:text-fg");

  /** Fila del panel expandido: mismo criterio de selección que `btn()`, pero de ancho completo con
   *  el nombre visible en vez de un cuadrado de icono solo. */
  const fila = (isActive: boolean) =>
    "flex w-full items-center gap-3 rounded-inner border px-3 py-2.5 text-sm transition-all duration-base ease-sdm " +
    (isActive
      ? "border-hairline bg-glass-2 text-accent-fg shadow-edge"
      : "border-transparent text-fg-dim hover:bg-glass-3 hover:text-fg");

  /** «Alertas» con avisos sin revisar: el número entra en el nombre accesible, no solo en el punto. */
  const labelDe = (s: { id: string; label: string; badge?: number | null }) =>
    s.id === "/alerts" && s.badge ? t("nav.alertsUnread", { count: s.badge }) : s.label;

  /** Riel expandible (spec 013): un botón alterna un panel superpuesto (nunca empuja el contenido,
   *  ADR-034) con el nombre de cada sección visible, en vez de depender del `title` al pasar el
   *  ratón. Elegir una sección (o «Acerca de») pliega el panel a la vez que navega — corrección
   *  post-validación: dejarlo abierto tapaba la pantalla de destino hasta un segundo gesto (D6 de
   *  `research.md`, revisado tras usarlo de verdad). */
  let botonRef = $state<HTMLButtonElement>();
  let panelRef = $state<HTMLElement>();

  /** Usada por `Escape` y por el clic fuera: a diferencia de pulsar el propio botón (que conserva
   *  el foco donde ya está), aquí el foco estaba dentro del panel y hay que devolverlo a mano. */
  function plegar() {
    if (!expanded) return;
    onToggleExpand?.();
    botonRef?.focus();
  }

  /** El foco entra al panel al abrir, mismo patrón que `ConfirmDialog`/`ExplicacionModal`/
   *  `AboutDialog` (`docs/known-issues.md` #2): sin esto, `Escape` —atado al propio panel— nunca
   *  llegaría a disparar porque el foco se quedaría en el botón, fuera de su árbol. */
  $effect(() => {
    if (expanded) panelRef?.focus();
  });
</script>

{#if expanded}
  <!-- Solo detecta el clic fuera: el cierre por teclado (Escape) lo cubre el `onkeydown` del propio
       panel, alcanzable porque el foco entra ahí al abrir. -->
  <div class="fixed inset-0 z-30" role="presentation" onclick={plegar}></div>
{/if}

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- El `role="dialog"` dinámico (solo con `expanded`) hace el panel interactivo de verdad cuando
     se muestra, pero el análisis estático del linter sigue viendo el `<aside>` como no interactivo.
     Ver docs/known-issues.md #6. -->
<aside
  bind:this={panelRef}
  class={expanded
    ? "sdm-material-overlay absolute inset-y-0 left-0 z-40 flex w-[232px] flex-none flex-col gap-2 rounded-r-window p-3 shadow-lift"
    : "sdm-material-chrome flex w-rail flex-none flex-col items-center gap-2 border-r border-hairline py-4"}
  tabindex={expanded ? -1 : undefined}
  role={expanded ? "dialog" : undefined}
  aria-label={expanded ? t("nav.monitoring") : undefined}
  onkeydown={expanded ? (e) => e.key === "Escape" && plegar() : undefined}
>
  <!-- Sin logo de marca aparte: en un riel de solo iconos duplicaba a «Panel general», que ya
       lleva a `/` con el mismo icono del disco. La identidad de la app vive en la barra de título
       y en «Acerca de». -->
  <button
    bind:this={botonRef}
    class={expanded ? fila(false) : btn(false)}
    aria-expanded={expanded}
    aria-label={t(expanded ? "nav.sidebar.collapse" : "nav.sidebar.expand")}
    title={t(expanded ? "nav.sidebar.collapse" : "nav.sidebar.expand")}
    onclick={onToggleExpand}
  >
    <Icon name="sidebarToggle" size={19} />
    {#if expanded}<span class="truncate">{t("nav.sidebar.collapse")}</span>{/if}
  </button>

  {#if expanded}
    <nav class="flex w-full flex-col gap-2" aria-label={t("nav.monitoring")}>
      {#each sections as s (s.id)}
        <a
          class={fila(active === s.id)}
          href={s.href}
          aria-current={active === s.id ? "page" : undefined}
          onclick={onSelect}
        >
          <Icon name={s.icon} size={19} />
          <span class="truncate">{labelDe(s)}</span>
        </a>
      {/each}
    </nav>
  {:else}
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
  {/if}

  <div class="flex-1"></div>

  {#if onabout}
    <button
      class={expanded ? fila(false) : btn(false)}
      aria-label={t("nav.about")}
      title={expanded ? undefined : t("nav.about")}
      onclick={() => {
        onabout?.();
        if (expanded) onSelect?.();
      }}
    >
      <Icon name="tag" size={19} />
      {#if expanded}<span class="truncate">{t("nav.about")}</span>{/if}
    </button>
  {/if}

  <div
    class="mt-1 flex h-10 {expanded
      ? 'w-full justify-start px-3'
      : 'min-w-10'} shrink-0 items-center gap-1 rounded-inner px-1.5"
    style="background: {tone.soft}; color: {tone.fg}"
    role="status"
    aria-label={globalLabel}
    title={expanded ? undefined : globalLabel}
  >
    {#if globalCount}
      <span class="sdm-num text-2xs font-semibold leading-none">{globalCount}</span>
      <Icon name={globalIcon} size={15} />
    {:else}
      <Icon name={globalIcon} size={18} />
    {/if}
    {#if expanded}<span class="truncate text-xs font-semibold">{globalLabel}</span>{/if}
  </div>
</aside>

{#if expanded}
  <!-- Hueco fijo de 74 px en el flujo normal: sin él, `AppShell` perdería el ancho que el `<aside>`
       ya no ocupa al pasar a `position: absolute` (US2, spec 013) — el contenido no debe moverse
       ni un píxel al expandir. -->
  <div class="w-rail flex-none" aria-hidden="true"></div>
{/if}
