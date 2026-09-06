import { describe, expect, it } from "vitest";

/** Paridad entre temas.
 *
 *  Esta prueba existe porque el fallo que persigue **es invisible en jsdom**: una variable definida
 *  solo en el bloque de un tema queda sin valor en el otro, y el componente no se rompe — pinta con
 *  el valor heredado, o con nada. No hay error en consola, no falla ningún tipo, y el defecto llega
 *  al usuario como un color que «se ve raro» en oscuro (`docs/testing-strategy.md` §7).
 *
 *  Se leen las reglas reales de la hoja de estilos, no una lista escrita a mano: una lista habría
 *  que mantenerla, y nadie mantiene una lista.
 */

/** Nombres de las propiedades `--sdm-*` declaradas en las reglas cuyo selector case con `patron`. */
function propiedadesDeclaradas(patron: RegExp): Set<string> {
  const nombres = new Set<string>();
  for (const hoja of Array.from(document.styleSheets)) {
    let reglas: CSSRuleList;
    try {
      reglas = hoja.cssRules;
    } catch {
      // Hoja de otro origen. No debería haberla aquí, pero leerla lanzaría y no es el fallo que
      // esta prueba busca.
      continue;
    }
    for (const regla of Array.from(reglas)) {
      if (!(regla instanceof CSSStyleRule) || !patron.test(regla.selectorText)) continue;
      for (const propiedad of Array.from(regla.style)) {
        if (propiedad.startsWith("--sdm-")) nombres.add(propiedad);
      }
    }
  }
  return nombres;
}

function valorResuelto(propiedad: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(propiedad).trim();
}

describe("tokens del sistema de diseño", () => {
  it("declara los mismos tokens en claro y en oscuro", () => {
    const claro = propiedadesDeclaradas(/^:root,\s*\[data-theme="light"\]$|\[data-theme="light"\]/);
    const oscuro = propiedadesDeclaradas(/\[data-theme="dark"\]/);

    // Si esto falla es que la hoja no se ha cargado, no que falten tokens: sin este ancla la
    // prueba pasaría en verde comparando dos conjuntos vacíos.
    expect(claro.size, "no se han cargado los tokens").toBeGreaterThan(20);

    const soloEnClaro = [...claro].filter((p) => !oscuro.has(p)).sort();
    expect(soloEnClaro, "tokens sin equivalente en el tema oscuro").toEqual([]);
  });

  it("resuelve todos los tokens en ambos temas, sin valores vacíos", () => {
    const tokens = [...propiedadesDeclaradas(/\[data-theme="dark"\]/)].sort();
    expect(tokens.length).toBeGreaterThan(20);

    for (const tema of ["light", "dark"] as const) {
      document.documentElement.setAttribute("data-theme", tema);
      const vacios = tokens.filter((t) => valorResuelto(t) === "");
      expect(vacios, `tokens sin valor con data-theme="${tema}"`).toEqual([]);
    }
    document.documentElement.removeAttribute("data-theme");
  });

  it("cambia de verdad de valor entre temas, no solo de atributo", () => {
    document.documentElement.setAttribute("data-theme", "light");
    const fondoClaro = valorResuelto("--sdm-bg");
    document.documentElement.setAttribute("data-theme", "dark");
    const fondoOscuro = valorResuelto("--sdm-bg");
    document.documentElement.removeAttribute("data-theme");

    expect(fondoClaro).not.toBe("");
    expect(fondoOscuro).not.toBe(fondoClaro);
  });
});

describe("tokens v3 — display, riel e iconos", () => {
  it("declara los tokens de composición nuevos, resueltos en ambos temas", () => {
    const nuevos = [
      "--sdm-font-display",
      "--sdm-text-display",
      "--sdm-text-hero",
      "--sdm-tracking-display",
      "--sdm-rail-width",
      "--sdm-hero-height",
      "--sdm-icon-stroke",
      "--sdm-icon-size"
    ];
    for (const tema of ["light", "dark"] as const) {
      document.documentElement.setAttribute("data-theme", tema);
      for (const t of nuevos) {
        expect(valorResuelto(t), `${t} sin valor con data-theme="${tema}"`).not.toBe("");
      }
    }
    document.documentElement.removeAttribute("data-theme");
  });

  it("--sdm-on-accent es tinta en oscuro, no blanco (paleta Ciruela)", () => {
    document.documentElement.setAttribute("data-theme", "light");
    const claro = valorResuelto("--sdm-on-accent").toLowerCase();
    document.documentElement.setAttribute("data-theme", "dark");
    const oscuro = valorResuelto("--sdm-on-accent").toLowerCase();
    document.documentElement.removeAttribute("data-theme");

    // En claro sigue siendo blanco sobre el acento sólido; en oscuro el acento es claro y el texto
    // blanco daba 2,27:1 — pasa a tinta.
    expect(oscuro).not.toBe(claro);
    expect(["#fff", "#ffffff", "rgb(255, 255, 255)", "white"]).not.toContain(oscuro);
  });

  it(".sdm-display usa la familia de display y el peso 600, sin inventar 700", () => {
    const el = document.createElement("span");
    el.className = "sdm-display";
    document.body.appendChild(el);
    const estilo = getComputedStyle(el);
    expect(estilo.fontWeight).toBe("600");
    expect(estilo.fontVariantNumeric).toContain("tabular-nums");
    el.remove();
  });
});
