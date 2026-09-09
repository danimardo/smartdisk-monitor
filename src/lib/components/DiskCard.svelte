<script lang="ts">
  /** Tarjeta de disco del panel general (US-012), recompuesta en v3 (`DiskCard.md`): cabecera de
   *  52 px que **hereda el color del estado** con la sparkline de temperatura de fondo, cifras de
   *  display, y el dato ausente como «—» discreto —no «No disponible» a 23 px, que pesaba más que
   *  el dato presente—.
   *
   *  Navega con **un enlace real** (constitución §XIV): conserva ctrl+clic, clic central, menú
   *  contextual, foco y el anuncio como enlace. Sin `href` se renderiza como bloque no interactivo. */
  import Card from "./Card.svelte";
  import CapacityBar from "./CapacityBar.svelte";
  import Icon from "./Icon.svelte";
  import Sparkline from "./Sparkline.svelte";
  import StatusPill from "./StatusPill.svelte";
  import { healthToken } from "$lib/design/health";
  import { busIcon } from "$lib/design/icons";
  import {
    ayudaMetrica,
    picoActividadPanel,
    type AyudaMetrica,
    type MetricaAyudable
  } from "$lib/design/metricHelp";
  import { formatTemperature, formatPercent, formatBytes } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import type { DiskSummary, HealthState } from "$lib/design/types";
  import type { Punto } from "$lib/design/series";

  let {
    disk = null as DiskSummary | null,
    href = undefined as string | undefined,
    /** Serie de temperatura de 24 h para el fondo de la cabecera. Sin ella, cabecera plana. */
    temperatureSeries = [] as Punto[],
    /** Umbrales de `settings.alerts` para el veredicto del tooltip de cada métrica. A falta de
     *  ellos, `metricHelp` usa los de fábrica (una ayuda, no una alerta). */
    umbrales = undefined as
      { tempWarnC: number; tempCritC: number; wearWarnPct: number; wearCritPct: number } | undefined
  } = $props();

  const estado = $derived<HealthState>(disk?.state ?? "unknown");
  const tone = $derived(healthToken[estado]);
  /** No hay lectura SMART reciente: el bus no la expone, dejó de responder, o aún no ha llegado la
   *  primera. El backend lo marca poniendo `unknownReason` (lo deja en `null` en cuanto la lectura
   *  es fresca). Se usa para vaciar magnitudes y quitar la curva; **no** para la píldora. */
  const sinSmartFresco = $derived(disk?.unknownReason != null);
  /** La píldora sigue al `state` (ya fundido por `estadoConAlertas`): un disco que dejó de
   *  responder es `warn` → «Advertencia», y así concuerda con «Reparto de estados». «Sin datos
   *  SMART» queda solo para el que de verdad no lo soporta, que sigue en `unknown`. */
  const label = $derived(estado === "unknown" ? t("disk.noSmartData") : t(`health.${estado}`));
  const overTempLimit = $derived(
    !!disk?.temperatureC && !!disk?.vendorTempLimitC && disk.temperatureC >= disk.vendorTempLimitC
  );
  const volume = $derived(disk?.volumes?.[0] ?? null);
  const hayCurva = $derived(!sinSmartFresco && temperatureSeries.some((p) => p.v !== null));

  /** Un dato ausente se compone como «—» a text-xs en gris, con el texto completo en el `title`
   *  (`DiskCard.md`): así el disco sin SMART deja de pesar más que el que tiene datos. */
  const magnitudes = $derived([
    {
      metrica: "temperature" as MetricaAyudable,
      icon: "temp" as const,
      label: t("disk.temperatureShort"),
      value: disk?.temperatureC ?? null,
      text: disk ? formatTemperature(disk.temperatureC) : "",
      ignoraSmartFresco: false,
      color: overTempLimit ? "var(--sdm-warn)" : "var(--sdm-text)"
    },
    {
      metrica: "wear" as MetricaAyudable,
      icon: "wear" as const,
      label: t("disk.wearShort"),
      value: disk?.percentageUsed ?? null,
      text: disk ? formatPercent(disk.percentageUsed) : "",
      ignoraSmartFresco: false,
      color: "var(--sdm-text)"
    },
    {
      metrica: "activity" as MetricaAyudable,
      icon: "pulse" as const,
      label: t("disk.activityShort"),
      // El panel muestra el **pico** de la ventana (spec 007, Q1 → A): la señal más directa de
      // «¿ha estado ocupado este disco hace poco?».
      value: disk?.activity?.picoPercent ?? null,
      text: picoActividadPanel(disk?.activity) ?? "",
      // La actividad tiene su propio estado (`no_disponible`/`parcial`/`valido`): no se apaga
      // porque falte una lectura SMART fresca, a diferencia de temperatura y desgaste.
      ignoraSmartFresco: true,
      color: "var(--sdm-text)"
    }
  ]);

  const ctxAyuda = $derived({ ...umbrales, vendorLimitC: disk?.vendorTempLimitC });

  /** Tooltip de ayuda de la métrica, **solo con el ratón** (la tarjeta es un `<a>` entero: no puede
   *  contener un elemento tabulable; la versión con teclado está en el detalle). Uno local y ligero
   *  en vez del componente `Tooltip`: con 20 discos serían 60 instancias y el panel debe pintarse
   *  rápido (`open-questions.md` J.59, SC-006). El texto se calcula al pasar el ratón, no antes. */
  type TipDisco = AyudaMetrica & {
    x: number;
    y: number;
    debajo: boolean;
    ali: "centro" | "izq" | "der";
  };
  let tip = $state<TipDisco | null>(null);
  function mostrarTip(m: (typeof magnitudes)[number], e: MouseEvent) {
    const cel = e.currentTarget as HTMLElement;
    const r = cel.getBoundingClientRect();
    const cx = r.left + r.width / 2;
    const ali = cx < 170 ? "izq" : cx > window.innerWidth - 170 ? "der" : "centro";
    const esActividad = m.metrica === "activity";
    tip = {
      ...ayudaMetrica(
        m.metrica,
        sinSmartFresco && !esActividad ? null : m.value,
        ctxAyuda,
        esActividad ? disk?.activity : undefined
      ),
      x: cel.offsetLeft + (ali === "izq" ? 0 : ali === "der" ? cel.offsetWidth : cel.offsetWidth / 2),
      y: cel.offsetTop,
      debajo: r.top < 180,
      ali
    };
  }
  const desplazamientoX = { centro: "-50%", izq: "0", der: "-100%" };
</script>

{#if disk}
  <svelte:element
    this={href ? "a" : "div"}
    href={href || undefined}
    class="sdm-block-link flex flex-col rounded-card text-left {href ? 'sdm-hover-bloque' : ''} {tip
      ? 'relative z-30'
      : ''}"
    aria-label={href ? t("disk.open", { name: disk.alias ?? disk.model }) : undefined}
  >
    <Card padding="none" class="flex-1">
      <div
        class="relative flex h-[52px] items-center gap-3 overflow-hidden px-4"
        style="background: {tone.soft}"
      >
        {#if hayCurva}
          <div class="pointer-events-none absolute inset-0 opacity-[0.85]">
            <Sparkline points={temperatureSeries} color={tone.fg} height={52} strokeWidth={1.6} />
          </div>
        {/if}
        <span
          class="relative grid size-[30px] shrink-0 place-items-center rounded-nav text-fg-onAccent"
          style="background: {tone.fg}"
        >
          <Icon name={busIcon(disk.deviceType)} size={16} />
        </span>
        <div class="relative flex-1"></div>
        <span class="relative sdm-material rounded-pill">
          <StatusPill state={estado} {label} icon="auto" />
        </span>
      </div>

      <!-- `flex-1` para llenar la tarjeta (que a su vez llena la celda de la rejilla): así las
           cuatro tarjetas de una fila quedan a la misma altura y la barra de capacidad, con
           `mt-auto`, se alinea abajo en todas aunque las magnitudes ocupen distinto. -->
      <div class="flex flex-1 flex-col gap-3 px-4 pb-4 pt-3">
        <div class="flex min-w-0 flex-col gap-0.5">
          <span class="sdm-selectable sdm-display truncate text-base">{disk.alias ?? disk.model}</span>
          <span class="sdm-selectable truncate text-2xs text-fg-dim">{disk.model} · {disk.deviceType}</span>
        </div>

        <!-- El tooltip de ayuda de cada métrica es **solo con el ratón** (`mouseenter`/`leave`); la
             vía con teclado y su `aria` viven en el detalle de disco (la tarjeta es un `<a>` entero
             y no puede contener un objetivo tabulable). -->
        <div class="relative grid grid-cols-3 gap-2">
          {#each magnitudes as m}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <!-- Ver docs/known-issues.md #4: el tooltip de ayuda es un extra de ratón; la misma
                 información, accesible por teclado, está en el detalle de disco. -->
            <div
              class="flex min-w-0 flex-col gap-0.5"
              onmouseenter={(e) => mostrarTip(m, e)}
              onmouseleave={() => (tip = null)}
            >
              <span class="flex items-center gap-1 text-2xs font-medium text-fg-faint">
                <Icon name={m.icon} size={12} />
                <span class="truncate">{m.label}</span>
              </span>
              <!-- Ranura de alto fijo: «—» y «48 °C» ocupan lo mismo, para que la fila no se
                   descuadre. Sin SMART fresco las tres van a «—» aunque quede una lectura vieja
                   o un contador en vivo (boceto §4): un dato caduco presentado como actual engaña. -->
              <span class="flex h-[26px] items-end">
                {#if m.value === null || (sinSmartFresco && !m.ignoraSmartFresco)}
                  <span class="text-xs text-fg-dim">—</span>
                {:else}
                  <span class="sdm-num sdm-display text-xl" style="color: {m.color}">{m.text}</span>
                {/if}
              </span>
            </div>
          {/each}
          {#if tip}
            <div
              role="tooltip"
              class="sdm-material-overlay pointer-events-none absolute z-50 w-max max-w-xs whitespace-pre-line rounded-inner p-3 text-2xs leading-normal"
              style="left: {tip.x}px; top: {tip.y}px; transform: translate({desplazamientoX[
                tip.ali
              ]}, {tip.debajo
                ? 'calc(46px)'
                : 'calc(-100% - 6px)'}); background: var(--sdm-glass-strong); border-left: 3px solid {healthToken[
                tip.estado
              ].fg}"
            >
              <span class="mb-1 block font-semibold text-fg">{tip.titulo}</span>
              <span class="block text-fg-dim">{tip.texto}</span>
            </div>
          {/if}
        </div>

        <div class="mt-auto">
          {#if volume}
            <CapacityBar
              label={`${volume.driveLetters.join(", ") || volume.label} · ${formatBytes(volume.capacityBytes)}`}
              capacityBytes={volume.capacityBytes}
              freeBytes={volume.freeBytes}
            />
          {:else}
            <span class="flex h-[38px] items-center text-2xs text-fg-faint">{t("disk.noVolumes")}</span>
          {/if}
        </div>
      </div>
    </Card>
  </svelte:element>
{/if}
