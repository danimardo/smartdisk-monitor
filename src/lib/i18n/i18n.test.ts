import { beforeEach, describe, expect, it } from "vitest";
import { i18n, t, tp } from "./index";

/** El principio VI exige que todo texto salga de los diccionarios y que los dos idiomas estén
 *  sincronizados. Estas pruebas fijan además la decisión de `open-questions.md` A.6: el formato de
 *  números sigue al idioma de la aplicación, no al de Windows. */

describe("selección de idioma", () => {
  it("un sistema en español arranca en español", () => {
    i18n.init(null, "es-ES");
    expect(i18n.locale).toBe("es");
  });

  it("cualquier otro idioma arranca en inglés", () => {
    i18n.init(null, "fr-FR");
    expect(i18n.locale).toBe("en");
    i18n.init(null, "de-DE");
    expect(i18n.locale).toBe("en");
  });

  it("la preferencia del usuario manda sobre el sistema", () => {
    i18n.init("en", "es-ES");
    expect(i18n.locale).toBe("en");
  });
});

describe("formatLocale — sigue al idioma de la app, no al del sistema", () => {
  it("conserva la variante regional cuando comparten idioma", () => {
    i18n.init("es", "es-MX");
    expect(i18n.formatLocale).toBe("es-MX");
  });

  it("cae al respaldo cuando el sistema habla otro idioma", () => {
    // Este es el caso que motivó la corrección: app en español, Windows en francés.
    i18n.init("es", "fr-FR");
    expect(i18n.formatLocale).toBe("es-ES");
  });

  it("app en inglés sobre Windows en español usa en-US", () => {
    i18n.init("en", "es-ES");
    expect(i18n.formatLocale).toBe("en-US");
  });
});

describe("t — traducción e interpolación", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("traduce una clave existente", () => {
    expect(t("common.notAvailable")).toBe("No disponible");
  });

  it("interpola variables", () => {
    expect(t("common.updatedAgo", { value: "hace 2 min" })).toContain("hace 2 min");
  });

  it("una clave ausente devuelve la propia clave, nunca cadena vacía", () => {
    // Visible en desarrollo: un hueco silencioso sería peor que un texto feo.
    expect(t("clave.que.no.existe")).toBe("clave.que.no.existe");
  });

  it("una variable no proporcionada deja el marcador visible", () => {
    expect(t("common.updatedAgo")).toContain("{value}");
  });

  it("cambiar de idioma cambia el texto", () => {
    i18n.set("en");
    expect(t("common.notAvailable")).toBe("Not available");
  });
});

describe("tp — plurales con Intl.PluralRules", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("usa la forma singular con uno", () => {
    expect(tp("global.needsAttention", 1)).toBe("1 disco necesita atención");
  });

  it("usa la forma plural con varios", () => {
    expect(tp("global.needsAttention", 3)).toBe("3 discos necesitan atención");
  });

  it("el cero usa la forma plural en español", () => {
    expect(tp("global.needsAttention", 0)).toContain("discos");
  });

  it("funciona igual en inglés", () => {
    i18n.set("en");
    expect(tp("global.needsAttention", 1)).toBe("1 disk needs attention");
    expect(tp("global.needsAttention", 5)).toBe("5 disks need attention");
  });
});
