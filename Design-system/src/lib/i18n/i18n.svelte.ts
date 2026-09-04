/** i18n mínimo y tipado. Español e inglés (spec §8, US-003).
 *  El idioma inicial procede del sistema: es-* → es; cualquier otro → en.
 *  La preferencia del usuario se persiste en `settings`, no en localStorage. */

import es from "./es.json";
import en from "./en.json";

export type Locale = "es" | "en";
type Dict = Record<string, string>;

const dicts: Record<Locale, Dict> = { es, en };

/** Locale BCP-47 de respaldo por idioma, usado cuando el del sistema no comparte idioma con la app. */
const FALLBACK_TAG: Record<Locale, string> = { es: "es-ES", en: "en-US" };

let current = $state<Locale>("es");
let systemTag = $state<string>("en-US");

export const i18n = {
  get locale() {
    return current;
  },
  /** Etiqueta BCP-47 para Intl (números, fechas, plurales). **Sigue al idioma de la app, no al del sistema.**
   *  Si el sistema comparte idioma con la app se conserva su variante regional (es-MX, en-GB);
   *  si no, se usa el respaldo. Sin esto, una app en español en un Windows francés formatearía
   *  las cifras en francés (contradicción detectada en la revisión de especificación). */
  get formatLocale(): string {
    return systemTag.toLowerCase().startsWith(current) ? systemTag : FALLBACK_TAG[current];
  },
  /** `systemLocale` es navigator.language o el valor que devuelva el backend. */
  init(preferred: Locale | null, systemLocale = navigator.language) {
    systemTag = systemLocale || "en-US";
    current = preferred ?? (systemTag.toLowerCase().startsWith("es") ? "es" : "en");
    applyDocumentLang();
  },
  set(next: Locale) {
    current = next;
    applyDocumentLang();
    return { key: "settings.appearance.language", value: next };
  }
};

/** El atributo `lang` del documento debe seguir al idioma de la app: de él dependen los lectores
 *  de pantalla, el guionado y la selección de glifos. */
function applyDocumentLang(): void {
  if (typeof document !== "undefined") document.documentElement.lang = current;
}

/** Traduce una clave. Acepta interpolación simple: t("tests.remaining", { seconds: 48 }).
 *  Una clave ausente devuelve la propia clave (visible en desarrollo, nunca cadena vacía). */
export function t(key: string, vars?: Record<string, string | number>): string {
  const raw = dicts[current][key] ?? dicts.en[key] ?? key;
  if (!vars) return raw;
  return raw.replace(/\{(\w+)\}/g, (_match: string, name: string) => String(vars[name] ?? `{${name}}`));
}

/** Plural por cantidad. Busca `<key>.one` / `<key>.other` según `Intl.PluralRules` del idioma activo
 *  e interpola `{count}` automáticamente. Español e inglés solo necesitan `one` y `other`;
 *  la categoría se resuelve con Intl para no codificar la regla a mano.
 *
 *      tp("alerts.occurrences", 3)  →  "3 ocurrencias"
 */
export function tp(key: string, count: number, vars?: Record<string, string | number>): string {
  const rule = new Intl.PluralRules(i18n.formatLocale).select(count);
  const dict = dicts[current];
  const chosen = `${key}.${rule}` in dict ? `${key}.${rule}` : `${key}.other`;
  return t(chosen, { count: count.toLocaleString(i18n.formatLocale), ...vars });
}
