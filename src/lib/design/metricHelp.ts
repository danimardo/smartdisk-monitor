/** Ayuda contextual de las métricas de disco (temperatura, desgaste, actividad, horas): qué es cada
 *  una y un **veredicto sobre el valor actual**. La usan `DiskCard` (panel) y `MetricCard` (detalle)
 *  dentro de un `Tooltip`.
 *
 *  `veredictoMetrica` es puro (sin i18n) para poder probar la clasificación; `ayudaMetrica` monta el
 *  texto ya traducido. El criterio de «alto»/«crítico» reutiliza `classifyAgainstThresholds` y los
 *  umbrales de `settings.alerts`, de modo que el globo nunca contradice al color de la tarjeta ni a
 *  una alerta. */

import { classifyAgainstThresholds, temperatureThresholds } from "./health";
import { formatHours, formatPercent, formatTemperature } from "./format";
import { t } from "$lib/i18n";
import type { ActividadDisco, HealthState } from "./types";

export type MetricaAyudable = "temperature" | "wear" | "activity" | "powerOnHours";

export interface Veredicto {
  estado: HealthState;
  /** Sufijo de la clave `metric.help.<metrica>.verdict.<caso>`. */
  caso: string;
  params: Record<string, string | number>;
}

/** Umbrales opcionales. A falta del valor configurado se usa el de fábrica — es una ayuda, no una
 *  alerta, y los de fábrica son los que la inmensa mayoría de instalaciones tienen. */
export interface CtxMetrica {
  tempWarnC?: number;
  tempCritC?: number;
  vendorLimitC?: number | null;
  wearWarnPct?: number;
  wearCritPct?: number;
}

const TEMP_WARN_DEFECTO = 60;
const TEMP_CRIT_DEFECTO = 70;
const WEAR_WARN_DEFECTO = 80;
const WEAR_CRIT_DEFECTO = 90;
const HORAS_POR_ANIO = 8766;

export function veredictoMetrica(
  metrica: MetricaAyudable,
  valor: number | null,
  ctx: CtxMetrica = {},
  /** Solo para `metrica === "activity"`: el agregado de la ventana deslizante (spec 007). Manda
   *  sobre `valor`, que se ignora para esta métrica. */
  actividad?: ActividadDisco | null
): Veredicto {
  if (metrica === "activity") return veredictoActividad(actividad);
  if (valor === null) return { estado: "unknown", caso: "unknown", params: {} };

  switch (metrica) {
    case "temperature": {
      const { warn, crit } = temperatureThresholds(
        ctx.vendorLimitC,
        null,
        ctx.tempWarnC ?? TEMP_WARN_DEFECTO,
        ctx.tempCritC ?? TEMP_CRIT_DEFECTO
      );
      const estado = classifyAgainstThresholds(valor, warn, crit);
      return { estado, caso: estado, params: { value: formatTemperature(valor) } };
    }
    case "wear": {
      const warn = ctx.wearWarnPct ?? WEAR_WARN_DEFECTO;
      const crit = ctx.wearCritPct ?? WEAR_CRIT_DEFECTO;
      const estado = classifyAgainstThresholds(valor, warn, crit);
      return {
        estado,
        caso: estado,
        params: {
          value: formatPercent(valor),
          remaining: Math.max(0, Math.round(100 - valor)),
          warn
        }
      };
    }
    case "powerOnHours":
      return {
        estado: "ok",
        caso: "info",
        params: { value: formatHours(valor), years: (valor / HORAS_POR_ANIO).toFixed(1) }
      };
  }
}

/** Veredicto de la actividad a partir del agregado de la ventana (media, pico, estado). Sin
 *  umbrales: es un indicador de rendimiento, no de salud. `no_disponible` (o ausente) → `unknown`;
 *  `parcial` → caso propio («midiendo aún»); `valido` → `info`. */
function veredictoActividad(a: ActividadDisco | null | undefined): Veredicto {
  if (!a || a.estado === "no_disponible") {
    return { estado: "unknown", caso: "unknown", params: {} };
  }
  const params = {
    media: formatPercent(a.mediaPercent),
    pico: formatPercent(a.picoPercent),
    ventana: a.ventanaSegundos
  };
  return { estado: "ok", caso: a.estado === "parcial" ? "parcial" : "info", params };
}

/** El pico de la ventana, para la cifra única de actividad del panel / Hero (spec 007, Q1 → A),
 *  con marca «~» cuando la ventana todavía es parcial. `null` cuando no hay dato
 *  (`no_disponible`): quien lo muestra pinta «—». */
export function picoActividadPanel(a: ActividadDisco | null | undefined): string | null {
  if (!a || a.estado === "no_disponible") return null;
  const pico = formatPercent(a.picoPercent);
  return a.estado === "parcial" ? t("disk.activityApprox", { value: pico }) : pico;
}

/** La actividad para el detalle de disco (spec 007, FR-013): media como cifra principal (con marca
 *  «~» si la ventana es parcial) y pico como cifra secundaria. `valor: null` ⇒ «No disponible». */
export function actividadDetalle(a: ActividadDisco | null | undefined): {
  valor: string | null;
  secundario: string | null;
} {
  if (!a || a.estado === "no_disponible") return { valor: null, secundario: null };
  const media = formatPercent(a.mediaPercent);
  return {
    valor: a.estado === "parcial" ? t("disk.activityApprox", { value: media }) : media,
    secundario: t("disk.activityPeak", { value: formatPercent(a.picoPercent) })
  };
}

export interface AyudaMetrica {
  titulo: string;
  /** Explicación fija + `\n\n` + veredicto del valor actual. Se renderiza con `white-space: pre-line`. */
  texto: string;
  estado: HealthState;
}

export function ayudaMetrica(
  metrica: MetricaAyudable,
  valor: number | null,
  ctx: CtxMetrica = {},
  actividad?: ActividadDisco | null
): AyudaMetrica {
  const v = veredictoMetrica(metrica, valor, ctx, actividad);
  const cuerpo = t(`metric.help.${metrica}.body`);
  const veredicto = t(`metric.help.${metrica}.verdict.${v.caso}`, v.params);
  return { titulo: t(`disk.${metrica}`), texto: `${cuerpo}\n\n${veredicto}`, estado: v.estado };
}
