import { describe, expect, it } from "vitest";
import { accentOnSurface, accessibleAccent } from "./accent";

/** Fija las conclusiones de la medición de `open-questions.md` §O, hecha sobre los 262.144 colores
 *  del espacio sRGB. Si alguna de estas pruebas falla, se ha roto el contraste para una parte del
 *  espacio de acentos que Windows permite elegir. */

type RGB = [number, number, number];

const toRgb = (hex: string): RGB => {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
};

const luminance = ([r, g, b]: RGB) => {
  const lin = [r, g, b].map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2];
};

const contrast = (a: RGB, b: RGB) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};

/** Superficies efectivas: el material translúcido compuesto sobre el lienzo de cada tema. */
const LIGHT: RGB = [0xf9, 0xf9, 0xfb];
const DARK: RGB = [0x21, 0x21, 0x28];

const AA = 4.5;

describe("accessibleAccent — el acento como fondo del botón primario", () => {
  it("el azul de fábrica de Windows se deja intacto", () => {
    const r = accessibleAccent("#0078d4");
    expect(r.adjusted).toBe(false);
    expect(r.accent).toBe("#0078d4");
  });

  it("elige texto negro sobre un acento claro", () => {
    expect(accessibleAccent("#ffb900").onAccent).toBe("#111114");
    expect(accessibleAccent("#fff000").onAccent).toBe("#111114");
  });

  it("elige texto blanco sobre un acento oscuro", () => {
    expect(accessibleAccent("#0067c0").onAccent).toBe("#ffffff");
  });

  it("ningún acento del espacio sRGB queda por debajo de AA", () => {
    // Barrido grueso: la medición completa (paso 4) vive en tools/accent-check.py.
    for (let r = 0; r < 256; r += 51) {
      for (let g = 0; g < 256; g += 51) {
        for (let b = 0; b < 256; b += 51) {
          const hex = "#" + [r, g, b].map((c) => c.toString(16).padStart(2, "0")).join("");
          const out = accessibleAccent(hex);
          expect(contrast(toRgb(out.accent), toRgb(out.onAccent)), `falla en ${hex}`).toBeGreaterThanOrEqual(
            AA
          );
        }
      }
    }
  });
});

describe("accentOnSurface — el acento como texto sobre el material", () => {
  it("corrige el azul de fábrica, que NO es legible como texto en tema claro", () => {
    // 4,31:1 sin corregir: por debajo de AA. Este es el caso que justifica que existan dos tokens.
    expect(contrast(toRgb("#0078d4"), LIGHT)).toBeLessThan(AA);
    const fixed = accentOnSurface("#0078d4", LIGHT);
    expect(contrast(toRgb(fixed), LIGHT)).toBeGreaterThanOrEqual(AA);
  });

  it("prefiere un tono de la paleta de Windows antes que inventarse un color", () => {
    const palette = ["#99EBFF", "#4CC2FF", "#0091F8", "#0078D4", "#0067C0", "#003E92", "#001A68"];
    // El tono 4 (#0067C0) da 5,40:1 sobre el material claro y es el más cercano al base.
    expect(accentOnSurface("#0078d4", LIGHT, palette)).toBe("#0067C0");
  });

  it("en tema oscuro elige un tono claro de la paleta", () => {
    const palette = ["#99EBFF", "#4CC2FF", "#0091F8", "#0078D4", "#0067C0", "#003E92", "#001A68"];
    const fg = accentOnSurface("#0078d4", DARK, palette);
    expect(contrast(toRgb(fg), DARK)).toBeGreaterThanOrEqual(AA);
    expect(luminance(toRgb(fg))).toBeGreaterThan(luminance(toRgb("#0078d4")));
  });

  it("deja intacto un acento que ya es legible", () => {
    expect(accentOnSurface("#0067c0", LIGHT)).toBe("#0067c0");
  });

  it("sin paleta, deriva un tono que cumple AA en ambos temas", () => {
    for (const hex of ["#ffb900", "#00cc6a", "#e74856", "#4cc2ff", "#ffffff", "#000000"]) {
      expect(contrast(toRgb(accentOnSurface(hex, LIGHT)), LIGHT), `claro, ${hex}`).toBeGreaterThanOrEqual(AA);
      expect(contrast(toRgb(accentOnSurface(hex, DARK)), DARK), `oscuro, ${hex}`).toBeGreaterThanOrEqual(AA);
    }
  });
});
