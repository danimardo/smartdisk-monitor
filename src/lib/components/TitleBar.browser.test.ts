import { page, userEvent } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import TitleBar from "./TitleBar.svelte";
import es from "$lib/i18n/es.json";

/** Barra de título propia (spec 011). Presentacional: no importa `@tauri-apps/api` — solo emite
 *  intenciones (`onminimize`/`ontogglemaximize`/`onclose`), igual que `ExplicacionModal`. */

describe("TitleBar — US1: paridad funcional", () => {
  it("restaurada: el control de maximizar muestra el icono y el nombre «maximizar»", async () => {
    await render(TitleBar, { props: { maximized: false } });
    const boton = page.getByRole("button", { name: es["titlebar.maximize"] });
    await expect.element(boton).toBeInTheDocument();
    expect(
      document.querySelector('button[aria-label="' + es["titlebar.maximize"] + '"] use[href="#i-maximize"]')
    ).not.toBeNull();
  });

  it("maximizada: el mismo control pasa a mostrar el icono y el nombre «restaurar»", async () => {
    await render(TitleBar, { props: { maximized: true } });
    const boton = page.getByRole("button", { name: es["titlebar.restore"] });
    await expect.element(boton).toBeInTheDocument();
    expect(page.getByRole("button", { name: es["titlebar.maximize"] }).query()).toBeNull();
    expect(
      document.querySelector('button[aria-label="' + es["titlebar.restore"] + '"] use[href="#i-restore"]')
    ).not.toBeNull();
  });

  it("minimizar: tiene su nombre accesible y llama a onminimize al pulsarlo", async () => {
    const onminimize = vi.fn();
    await render(TitleBar, { props: { maximized: false, onminimize } });
    await page.getByRole("button", { name: es["titlebar.minimize"] }).click();
    expect(onminimize).toHaveBeenCalledTimes(1);
  });

  it("maximizar/restaurar: llama a ontogglemaximize al pulsarlo", async () => {
    const ontogglemaximize = vi.fn();
    await render(TitleBar, { props: { maximized: false, ontogglemaximize } });
    await page.getByRole("button", { name: es["titlebar.maximize"] }).click();
    expect(ontogglemaximize).toHaveBeenCalledTimes(1);
  });

  it("cerrar: usa el nombre accesible ya existente («common.close») y llama a onclose", async () => {
    const onclose = vi.fn();
    await render(TitleBar, { props: { maximized: false, onclose } });
    await page.getByRole("button", { name: es["common.close"] }).click();
    expect(onclose).toHaveBeenCalledTimes(1);
  });

  it("el control minimizar es operable por teclado: tabulación + Enter lo activan de verdad", async () => {
    // `userEvent` (Chromium real) simula tecleo genuino, a diferencia de despachar un
    // `KeyboardEvent` a mano: solo así se comprueba la activación nativa del <button>, no un
    // simulacro que pasaría igual con un <div onclick> (principio VII, WCAG 2.1.1).
    const onminimize = vi.fn();
    await render(TitleBar, { props: { maximized: false, onminimize } });
    await userEvent.tab();
    expect(document.activeElement?.getAttribute("aria-label")).toBe(es["titlebar.minimize"]);
    await userEvent.keyboard("{Enter}");
    expect(onminimize).toHaveBeenCalledTimes(1);
  });

  it("la región de arrastre no cubre los tres botones", async () => {
    const { container } = await render(TitleBar, { props: { maximized: false } });
    for (const boton of container.querySelectorAll("button")) {
      expect(boton.closest("[data-tauri-drag-region]")).toBeNull();
    }
    expect(container.querySelector("[data-tauri-drag-region]")).not.toBeNull();
  });
});

/** Lee el valor resuelto de una variable CSS tal como el navegador la pinta de verdad — más fiable
 *  que comparar cadenas de `var(--sdm-*)`, que no dicen nada sobre si de verdad se resuelve. */
function colorResuelto(nombreVariable: string): string {
  const sonda = document.createElement("div");
  sonda.style.color = `var(${nombreVariable})`;
  document.body.appendChild(sonda);
  const valor = getComputedStyle(sonda).color;
  sonda.remove();
  return valor;
}

describe("TitleBar — US2: integración visual", () => {
  it("el fondo de la barra es --sdm-solid (el mismo plano opaco que el resto de la interfaz), no un valor distinto", async () => {
    // Corrección post-validación: el degradado bg/bg-2 se leía más oscuro que el resto del lienzo
    // a la altura de una barra tan fina; --sdm-solid es indistinguible a ojo (es el mismo respaldo
    // opaco que usan .sdm-material-* cuando el navegador no soporta backdrop-filter).
    const { container } = await render(TitleBar, { props: { maximized: false } });
    const barra = container.firstElementChild as HTMLElement;
    expect(barra.className).toContain("bg-solid");
    expect(getComputedStyle(barra).backgroundColor).toBe(colorResuelto("--sdm-solid"));
  });

  it("muestra el icono y el nombre de la aplicación arriba a la izquierda, dentro de la región de arrastre", async () => {
    const { container } = await render(TitleBar, { props: { maximized: false } });
    const region = container.querySelector("[data-tauri-drag-region]")!;
    const img = region.querySelector("img");
    expect(img).not.toBeNull();
    // Decorativo: el nombre de al lado ya lo anuncia a quien usa lector de pantalla.
    expect(img?.getAttribute("alt")).toBe("");
    expect(region.textContent).toContain(es["app.name"]);
  });

  it("los tres iconos usan el acento de la aplicación en reposo, no un color por defecto", async () => {
    const { container } = await render(TitleBar, { props: { maximized: false } });
    const esperado = colorResuelto("--sdm-accent");
    for (const boton of container.querySelectorAll("button")) {
      expect(getComputedStyle(boton).color).toBe(esperado);
    }
  });

  it("con el tema oscuro activo, el acento de los iconos es el del tema oscuro, no el claro", async () => {
    const claro = colorResuelto("--sdm-accent");
    document.documentElement.setAttribute("data-theme", "dark");
    try {
      const oscuro = colorResuelto("--sdm-accent");
      expect(oscuro).not.toBe(claro); // si esto fallara, el resto de la prueba no probaría nada

      const { container } = await render(TitleBar, { props: { maximized: false } });
      const boton = container.querySelector("button")!;
      expect(getComputedStyle(boton).color).toBe(oscuro);
    } finally {
      document.documentElement.removeAttribute("data-theme");
    }
  });
});

describe("TitleBar — US3: doble clic para maximizar/restaurar", () => {
  it("doble clic sobre la región de arrastre llama a ontogglemaximize", async () => {
    const ontogglemaximize = vi.fn();
    const { container } = await render(TitleBar, { props: { maximized: false, ontogglemaximize } });
    const region = container.querySelector("[data-tauri-drag-region]")!;
    await userEvent.dblClick(region);
    expect(ontogglemaximize).toHaveBeenCalledTimes(1);
  });

  it("doble clic sobre minimizar o cerrar no llama a ontogglemaximize (no burbujea a la región de arrastre)", async () => {
    const ontogglemaximize = vi.fn();
    await render(TitleBar, { props: { maximized: false, ontogglemaximize } });
    await userEvent.dblClick(page.getByRole("button", { name: es["titlebar.minimize"] }).element());
    await userEvent.dblClick(page.getByRole("button", { name: es["common.close"] }).element());
    expect(ontogglemaximize).not.toHaveBeenCalled();
  });

  it("doble clic sobre el propio botón maximizar solo dispara su `onclick` (2 clics), no también la región de arrastre", async () => {
    // Si el manejador de la región capturara este doble clic, veríamos 3 llamadas (2 del botón +
    // 1 de la región) en vez de las 2 legítimas del propio botón.
    const ontogglemaximize = vi.fn();
    await render(TitleBar, { props: { maximized: false, ontogglemaximize } });
    await userEvent.dblClick(page.getByRole("button", { name: es["titlebar.maximize"] }).element());
    expect(ontogglemaximize).toHaveBeenCalledTimes(2);
  });
});
