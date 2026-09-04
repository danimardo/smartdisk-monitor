import { beforeEach, describe, expect, it, vi } from "vitest";

const refreshAccentMock = vi.fn();
vi.mock("./accent", () => ({ refreshAccentForTheme: () => refreshAccentMock() }));

const { theme } = await import("./theme.svelte");

/** Fija dos cosas: que "sistema" sigue de verdad al sistema, y que un cambio de tema recalcula el
 *  acento. Lo segundo no es cosmético — el tono legible del acento depende de la superficie, y la
 *  superficie cambia con el tema (`open-questions.md` §O.6). Sin ese recálculo, un acento que
 *  cumplía AA en claro puede quedar por debajo al pasar a oscuro. */

/** jsdom no implementa matchMedia. */
function mockMatchMedia(prefiereOscuro: boolean) {
  const listeners: ((e: { matches: boolean }) => void)[] = [];
  vi.stubGlobal("matchMedia", (query: string) => ({
    matches: query.includes("dark") ? prefiereOscuro : false,
    media: query,
    addEventListener: (_: string, cb: (e: { matches: boolean }) => void) => listeners.push(cb),
    removeEventListener: () => {},
    dispatchEvent: () => true
  }));
  return { emitirCambio: (matches: boolean) => listeners.forEach((l) => l({ matches })) };
}

describe("preferencia de tema", () => {
  beforeEach(() => {
    refreshAccentMock.mockReset();
    document.documentElement.removeAttribute("data-theme");
  });

  it("'light' se aplica sin mirar al sistema", () => {
    mockMatchMedia(true);
    theme.init("light");
    expect(theme.resolved).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("'dark' se aplica sin mirar al sistema", () => {
    mockMatchMedia(false);
    theme.init("dark");
    expect(theme.resolved).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("'system' sigue al sistema cuando prefiere oscuro", () => {
    mockMatchMedia(true);
    theme.init("system");
    expect(theme.resolved).toBe("dark");
  });

  it("'system' sigue al sistema cuando prefiere claro", () => {
    mockMatchMedia(false);
    theme.init("system");
    expect(theme.resolved).toBe("light");
  });

  it("conserva la preferencia elegida, no solo la resuelta", () => {
    mockMatchMedia(true);
    theme.init("system");
    // El usuario eligió "sistema": la interfaz debe poder mostrar eso, no "oscuro".
    expect(theme.preference).toBe("system");
    expect(theme.resolved).toBe("dark");
  });
});

describe("cambio de tema", () => {
  beforeEach(() => {
    refreshAccentMock.mockReset();
    mockMatchMedia(false);
    theme.init("light");
    refreshAccentMock.mockReset();
  });

  it("aplica el nuevo tema al documento", () => {
    theme.set("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("recalcula el acento, porque la superficie ha cambiado", () => {
    theme.set("dark");
    expect(refreshAccentMock).toHaveBeenCalled();
  });

  it("devuelve la clave a persistir, sin persistirla por su cuenta", () => {
    // La persistencia es del llamante: este módulo no conoce la capa de acceso al backend.
    const r = theme.set("dark");
    expect(r).toMatchObject({ key: "settings.appearance.theme", value: "dark" });
  });
});
