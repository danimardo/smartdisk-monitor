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
import type { HealthState } from "./types";

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
  ctx: CtxMetrica = {}
): Veredicto {
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
    case "activity":
      return { estado: "ok", caso: "info", params: { value: formatPercent(valor) } };
    case "powerOnHours":
      return {
        estado: "ok",
        caso: "info",
        params: { value: formatHours(valor), years: (valor / HORAS_POR_ANIO).toFixed(1) }
      };
  }
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
  ctx: CtxMetrica = {}
): AyudaMetrica {
  const v = veredictoMetrica(metrica, valor, ctx);
  const cuerpo = t(`metric.help.${metrica}.body`);
  const veredicto = t(`metric.help.${metrica}.verdict.${v.caso}`, v.params);
  return { titulo: t(`disk.${metrica}`), texto: `${cuerpo}\n\n${veredicto}`, estado: v.estado };
}
