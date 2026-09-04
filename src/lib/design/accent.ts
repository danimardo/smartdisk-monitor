/** El acento de la app hereda el color de acento de Windows (decisión de diseño v2).
 *  El backend Rust lo lee del registro y lo expone como comando Tauri; aquí solo se traduce a
 *  tokens. Si el usuario lo tiene desactivado o falla la lectura, se conservan los respaldos de
 *  `tokens.css`, que ya están verificados contra AA en cada tema.
 *
 *  **El acento tiene dos usos con requisitos opuestos, y por eso hay dos tokens.**
 *
 *    `--sdm-accent`     va de FONDO (botón primario). Se mide contra `--sdm-on-accent`.
 *    `--sdm-accent-fg`  va de TEXTO sobre el material (enlaces, selección, serie de la gráfica).
 *                       Se mide contra la superficie, casi blanca en claro y casi negra en oscuro.
 *
 *  Un solo color no puede servir para las dos cosas. Barriendo el espacio sRGB completo
 *  (262.144 colores, `tools/accent-check.py`): el 65 % de los acentos posibles son ilegibles como
 *  texto sobre el material claro y el 50 % sobre el oscuro — **incluido el azul #0078d4 que Windows
 *  trae de fábrica**, que da 4,31:1 en tema claro, por debajo de AA. Aplicar el acento del sistema
 *  a los dos tokens a la vez, como se hacía antes, rompía el contraste en la configuración más
 *  común que existe.
 *
 *  Por eso `--sdm-accent-fg` se deriva por tema y se recalcula cuando el tema cambia.
 */

import { getSystemAccentColor } from "$lib/api";
import type { WindowsAccentShape as WindowsAccent } from "$lib/api/schemas";

/* El acento llega validado por su esquema Zod (constitución §XI): el `hex` es siempre `#RRGGBB`
   —el backend hace la conversión desde el ABGR del registro— y la paleta, si viene, son los tonos
   que Windows ya usa en su propia interfaz. Aquí solo se traduce a tokens. */

/** Contraste mínimo para texto normal (WCAG AA). */
const AA = 4.5;

type RGB = [number, number, number];

function toRgb(hex: string): RGB {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function toHex([r, g, b]: RGB): string {
  return (
    "#" +
    [r, g, b]
      .map((c) =>
        Math.round(Math.min(255, Math.max(0, c)))
          .toString(16)
          .padStart(2, "0")
      )
      .join("")
  );
}

/** Luminancia relativa WCAG 2.x. */
function luminance([r, g, b]: RGB): number {
  const lin = [r, g, b].map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2];
}

function contrast(a: RGB, b: RGB): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const WHITE: RGB = [255, 255, 255];
const BLACK: RGB = [17, 17, 20];

/** Mezcla lineal hacia blanco (`amount` > 0) o hacia negro (`amount` < 0).
 *
 *  **Redondea a enteros**, y no es un detalle: el color que se devuelve al navegador es de 8 bits
 *  por canal, así que medir el contraste sobre los valores en coma flotante y emitir después los
 *  redondeados puede entregar un color por debajo de AA. Ocurría con `#00cc6a`, que salía a
 *  4,497:1 tras redondear. Se redondea aquí para que lo medido y lo emitido sean el mismo color. */
function shift(rgb: RGB, amount: number): RGB {
  const target = amount > 0 ? 255 : 0;
  const k = Math.abs(amount);
  return rgb.map((c) => Math.round(c + (target - c) * k)) as RGB;
}

function lighten(hex: string, amount = 0.14): string {
  return toHex(shift(toRgb(hex), amount));
}

function rgba(hex: string, alpha: number): string {
  const [r, g, b] = toRgb(hex);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

const isHex = (v: unknown): v is string => typeof v === "string" && /^#[0-9a-fA-F]{6}$/.test(v);

/**
 * Acento utilizable **como fondo**, con el color de texto que le corresponde.
 * Busca el ajuste más pequeño que alcance AA, para no desvirtuar el color elegido por el usuario.
 * Verificado sobre el espacio sRGB completo: ningún color queda por debajo de AA, y el retoque
 * máximo es de 26/255 en un canal, imperceptible.
 */
export function accessibleAccent(hex: string): { accent: string; onAccent: string; adjusted: boolean } {
  const original = toRgb(hex);
  const prefersBlackText = contrast(original, BLACK) > contrast(original, WHITE);
  const text = prefersBlackText ? BLACK : WHITE;

  if (contrast(original, text) >= AA) {
    return { accent: hex, onAccent: toHex(text), adjusted: false };
  }

  const direction = prefersBlackText ? 1 : -1;
  for (let k = 0.05; k <= 0.9; k += 0.05) {
    const candidate = shift(original, direction * k);
    if (contrast(candidate, text) >= AA) {
      return { accent: toHex(candidate), onAccent: toHex(text), adjusted: true };
    }
  }
  return { accent: toHex(shift(original, direction * 0.9)), onAccent: toHex(text), adjusted: true };
}

/**
 * Acento utilizable **como texto** sobre una superficie dada.
 * Si Windows ofrece su paleta de tonos, se busca ahí primero: son los tonos que el usuario ya ve en
 * el resto del sistema, así que la aplicación se integra en lugar de inventarse un color propio.
 * Si ninguno llega a AA, se deriva oscureciendo o aclarando el acento base.
 */
export function accentOnSurface(hex: string, surface: RGB, palette?: string[]): string {
  const base = toRgb(hex);
  if (contrast(base, surface) >= AA) return hex;

  const surfaceIsLight = luminance(surface) > 0.5;

  const candidates = (palette ?? []).filter(isHex);
  // En tema claro interesan los tonos oscuros de la paleta, y al revés. Se ordenan por cercanía
  // al acento base para elegir el mínimo cambio que cumpla.
  const ordered = candidates
    .map((h) => ({ hex: h, rgb: toRgb(h) }))
    .filter((c) => (surfaceIsLight ? luminance(c.rgb) < luminance(base) : luminance(c.rgb) > luminance(base)))
    .sort(
      (a, b) => Math.abs(luminance(a.rgb) - luminance(base)) - Math.abs(luminance(b.rgb) - luminance(base))
    );

  for (const c of ordered) {
    if (contrast(c.rgb, surface) >= AA) return c.hex;
  }

  const direction = surfaceIsLight ? -1 : 1;
  for (let k = 0.05; k <= 0.95; k += 0.05) {
    const candidate = shift(base, direction * k);
    if (contrast(candidate, surface) >= AA) return toHex(candidate);
  }
  return toHex(shift(base, direction * 0.95));
}

/**
 * Color efectivo de la superficie sobre la que se lee el texto: el material translúcido compuesto
 * sobre el lienzo. Se calcula desde los tokens vivos del tema activo, no desde constantes, para que
 * un cambio en `tokens.css` no deje esta comprobación mintiendo en silencio.
 */
function effectiveSurface(): RGB {
  const cs = getComputedStyle(document.documentElement);
  const bg = parseColor(cs.getPropertyValue("--sdm-bg").trim()) ?? [255, 255, 255];
  const glass = parseColor(cs.getPropertyValue("--sdm-glass").trim());
  if (!glass) return bg;
  const alpha = parseAlpha(cs.getPropertyValue("--sdm-glass").trim());
  return glass.map((c, i) => Math.round(c * alpha + bg[i] * (1 - alpha))) as RGB;
}

function parseColor(value: string): RGB | null {
  // `isHex` es una guarda de tipo: sin el `else`, TypeScript estrecha `value` a `never` después.
  if (/^#[0-9a-fA-F]{6}$/.test(value)) return toRgb(value);
  const m = value.match(/rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)/i);
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

function parseAlpha(value: string): number {
  const m = value.match(/rgba\(\s*[\d.]+[\s,]+[\d.]+[\s,]+[\d.]+[\s,/]+([\d.]+)\s*\)/i);
  return m ? Number(m[1]) : 1;
}

/** Último acento aplicado, para poder recalcular `--sdm-accent-fg` al cambiar de tema. */
let current: WindowsAccent | null = null;

/** Llamar al arrancar y cuando el backend emita `system:accent-changed`. */
export async function applySystemAccent(): Promise<void> {
  let accent: WindowsAccent;
  try {
    accent = await getSystemAccentColor();
  } catch {
    return; // sin acento del sistema: se mantienen los respaldos de tokens.css
  }
  if (!accent || !isHex(accent.hex)) return;

  current = accent;
  paint();
}

/** Reaplica el acento al tema actual. Debe llamarse tras cada cambio de tema: `--sdm-accent-fg`
 *  depende de la superficie, y la superficie cambia con el tema. */
export function refreshAccentForTheme(): void {
  if (current) paint();
}

function paint(): void {
  if (!current) return;
  const { accent: safe, onAccent } = accessibleAccent(current.hex);
  const fg = accentOnSurface(current.hex, effectiveSurface(), current.palette);

  const root = document.documentElement.style;
  root.setProperty("--sdm-accent", safe);
  root.setProperty("--sdm-accent-hi", lighten(safe));
  root.setProperty("--sdm-accent-soft", rgba(safe, 0.14));
  root.setProperty("--sdm-on-accent", onAccent);
  root.setProperty("--sdm-accent-fg", fg);
}

/** Quita la sobreescritura y vuelve a los respaldos del sistema de diseño. */
export function clearSystemAccent(): void {
  current = null;
  for (const p of [
    "--sdm-accent",
    "--sdm-accent-hi",
    "--sdm-accent-soft",
    "--sdm-on-accent",
    "--sdm-accent-fg"
  ]) {
    document.documentElement.style.removeProperty(p);
  }
}
