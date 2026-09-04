/** Formateo de presentación. Regla de oro: un valor ausente se muestra como "No disponible",
 *  nunca como 0, "-" ni cadena vacía (spec §12 y modelo de datos §1).
 *
 *  Locale: **todas** las funciones formatean con `i18n.formatLocale`, que sigue al idioma elegido
 *  en la aplicación, no al de Windows. Pasar un locale explícito es una excepción reservada a las
 *  exportaciones (un informe puede pedirse en un idioma distinto al de la interfaz).
 *
 *  Unidades: se usa base 1024 con las etiquetas KB/MB/GB, que es la convención del Explorador de
 *  Windows y por tanto la que el usuario podrá contrastar. Es deliberado: no "corregir" a KiB/MiB
 *  ni a base 1000. El dato persistido son siempre bytes (modelo de datos §5).
 */

import { i18n, t } from "$lib/i18n";

export const NOT_AVAILABLE = () => t("common.notAvailable"); // es: "No disponible" / en: "Not available"

const isMissing = (v: unknown): v is null | undefined =>
  v === null || v === undefined || Number.isNaN(v);

/** Base binaria con etiquetas decimales, igual que el Explorador de Windows. */
const KIB = 1024;

/** Bytes → unidad legible, base 1024, locale de la aplicación. El dato original nunca se altera. */
export function formatBytes(bytes: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(bytes)) return NOT_AVAILABLE();
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let value = bytes;
  let i = 0;
  while (value >= KIB && i < units.length - 1) {
    value /= KIB;
    i++;
  }
  const decimals = value < 10 && i > 2 ? 2 : value < 100 && i > 1 ? 1 : 0;
  return `${value.toLocaleString(locale, { minimumFractionDigits: decimals, maximumFractionDigits: decimals })} ${units[i]}`;
}

export function formatTemperature(celsius: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(celsius)) return NOT_AVAILABLE();
  return `${celsius.toLocaleString(locale, { maximumFractionDigits: 0 })} °C`;
}

export function formatPercent(value: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(value)) return NOT_AVAILABLE();
  return `${value.toLocaleString(locale, { maximumFractionDigits: 0 })} %`;
}

export function formatHours(hours: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(hours)) return NOT_AVAILABLE();
  return `${hours.toLocaleString(locale)} h`;
}

/** Caudal en **bytes por segundo**, que es la unidad que persiste el backend
 *  (`read_bytes_per_second` / `write_bytes_per_second`). La escala es la misma base 1024 que
 *  `formatBytes`, de modo que "180 MB/s" son 180 × 1024² B/s.
 *  Nunca pases MB/s ya convertidos: la conversión vive aquí y en un solo sitio. */
export function formatThroughput(bytesPerSecond: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(bytesPerSecond)) return NOT_AVAILABLE();
  return `${formatBytes(bytesPerSecond, locale)}/s`;
}

/** Latencia en milisegundos. Por debajo de 10 ms se muestra un decimal: la diferencia entre
 *  0,2 ms (NVMe) y 4 ms (HDD) es justo la que interesa leer. */
export function formatLatency(milliseconds: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(milliseconds)) return NOT_AVAILABLE();
  const decimals = milliseconds < 10 ? 1 : 0;
  return `${milliseconds.toLocaleString(locale, { minimumFractionDigits: decimals, maximumFractionDigits: decimals })} ms`;
}

/** UTC persistido → hora local del sistema. */
export function formatDateTime(isoUtc: string | null | undefined, locale = i18n.formatLocale): string {
  if (!isoUtc) return NOT_AVAILABLE();
  return new Date(isoUtc).toLocaleString(locale, { dateStyle: "short", timeStyle: "medium" });
}

export function formatTime(isoUtc: string | null | undefined, locale = i18n.formatLocale): string {
  if (!isoUtc) return NOT_AVAILABLE();
  return new Date(isoUtc).toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
}

/** Antigüedad de una lectura ("hace 2 min"). Alimenta la marca de dato obsoleto exigida por
 *  `AGENTS.md` §5. Devuelve null si no hay fecha: el llamante decide si omitir la marca. */
export function formatAge(isoUtc: string | null | undefined, now = Date.now(), locale = i18n.formatLocale): string | null {
  if (!isoUtc) return null;
  const seconds = Math.round((now - new Date(isoUtc).getTime()) / 1000);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto", style: "narrow" });
  if (seconds < 60) return rtf.format(-seconds, "second");
  if (seconds < 3600) return rtf.format(-Math.round(seconds / 60), "minute");
  if (seconds < 86400) return rtf.format(-Math.round(seconds / 3600), "hour");
  return rtf.format(-Math.round(seconds / 86400), "day");
}

/** Número de serie enmascarado para capturas y exportaciones anonimizadas. */
export function maskSerial(serial: string | null | undefined): string {
  if (!serial) return NOT_AVAILABLE();
  if (serial.length <= 6) return "••••";
  return `${serial.slice(0, 4)}••••${serial.slice(-2)}`;
}

/** Porcentaje ocupado de un volumen; null si falta cualquiera de los dos datos. */
export function usedPercent(capacityBytes: number | null, freeBytes: number | null): number | null {
  if (isMissing(capacityBytes) || isMissing(freeBytes) || capacityBytes === 0) return null;
  return Math.min(100, Math.max(0, ((capacityBytes - freeBytes) / capacityBytes) * 100));
}
