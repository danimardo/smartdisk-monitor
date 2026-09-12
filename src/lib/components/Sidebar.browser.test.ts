import { page, userEvent } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import Sidebar from "./Sidebar.svelte";
import es from "$lib/i18n/es.json";
import type { IconName } from "$lib/design/icons";

/** El riel de v3 no tiene texto visible: solo es aceptable si la accesibilidad es impecable
 *  (`Sidebar.md` §Accesibilidad). Estas pruebas fijan eso — nombre accesible de cada botón, el
 *  activo con `aria-current`, el punto de aviso que no viaja solo, y el estado global como
 *  `role="status"`. */

function secciones(): { id: string; label: string; icon: IconName; href: string; badge?: number | null }[] {
  return [
    { id: "/", label: "Panel general", icon: "diskStack", href: "/" },
    { id: "/alerts", label: "Alertas", icon: "alert", href: "/alerts", badge: 3 },
    { id: "/events", label: "Eventos", icon: "plug", href: "/events" }
  ];
}

describe("Sidebar (riel v3)", () => {
  it("cada sección es un enlace con nombre accesible, aunque no se vea texto", async () => {
    await render(Sidebar, {
      props: { sections: secciones(), active: "/", globalState: "ok", globalLabel: "Todo en orden" }
    });
    await expect.element(page.getByRole("link", { name: "Panel general" })).toBeInTheDocument();
    await expect.element(page.getByRole("link", { name: "Eventos" })).toBeInTheDocument();
    // texto de sección no visible
    expect(page.getByText("Eventos", { exact: true }).query()).toBeNull();
  });

  it("la sección activa lleva aria-current=page", async () => {
    const { container } = await render(Sidebar, {
      props: { sections: secciones(), active: "/alerts", globalState: "ok", globalLabel: "Todo en orden" }
    });
    const activo = container.querySelector('a[aria-current="page"]')!;
    expect(activo.getAttribute("href")).toBe("/alerts");
  });

  it("el punto de aviso no es el único portador: el recuento entra en el nombre del enlace", async () => {
    await render(Sidebar, {
      props: {
        sections: secciones(),
        active: "/",
        globalState: "warn",
        globalLabel: "1 disco necesita atención"
      }
    });
    await expect.element(page.getByRole("link", { name: "Alertas, 3 sin revisar" })).toBeInTheDocument();
  });

  it("el indicador de estado global es role=status con el texto como nombre", async () => {
    await render(Sidebar, {
      props: {
        sections: secciones(),
        active: "/",
        globalState: "warn",
        globalLabel: "2 discos necesitan atención",
        globalIcon: "alert",
        globalCount: 2
      }
    });
    await expect
      .element(page.getByRole("status", { name: "2 discos necesitan atención" }))
      .toBeInTheDocument();
    await expect.element(page.getByText("2", { exact: true })).toBeInTheDocument();
  });

  it("«Acerca de» solo aparece si se pasa onabout, y llama al callback", async () => {
    const onabout = vi.fn();
    await render(Sidebar, {
      props: {
        sections: secciones(),
        active: "/",
        globalState: "ok",
        globalLabel: "Todo en orden",
        onabout
      }
    });
    await page.getByRole("button", { name: "Acerca de" }).click();
    expect(onabout).toHaveBeenCalledOnce();
  });
});

describe("Sidebar — riel expandible (spec 013)", () => {
  /** `expanded` es una prop controlada (la persistencia vive en `+layout.svelte`, spec 013 US4):
   *  aquí se simula el mismo papel con una variable local y `rerender`, igual que ya hace
   *  `AboutDialog.browser.test.ts` con `open`. */
  async function renderExpandible(extra: Record<string, unknown> = {}) {
    const props = {
      sections: secciones(),
      active: "/",
      globalState: "ok" as const,
      globalLabel: "Todo en orden",
      expanded: false,
      ...extra
    };
    const utils = await render(Sidebar, {
      props: {
        ...props,
        onToggleExpand: () => {
          props.expanded = !props.expanded;
          void utils.rerender({ ...props });
        }
      }
    });
    return utils;
  }

  it("plegado por defecto: sin panel, el botón ofrece expandirlo", async () => {
    await renderExpandible();
    const boton = page.getByRole("button", { name: es["nav.sidebar.expand"] });
    await expect.element(boton).toBeInTheDocument();
    await expect.element(boton).toHaveAttribute("aria-expanded", "false");
    // Sin panel: el nombre de una sección sigue sin verse como texto (ya lo fija la prueba de arriba).
    expect(page.getByRole("dialog").query()).toBeNull();
  });

  it("pulsar el botón expande el panel con el nombre de cada sección visible", async () => {
    await renderExpandible({ active: "/alerts" });
    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();

    const panel = page.getByRole("dialog");
    await expect.element(panel).toBeInTheDocument();
    await expect.element(panel.getByText("Panel general")).toBeInTheDocument();
    await expect.element(panel.getByText("Eventos")).toBeInTheDocument();
    // La sección activa se sigue marcando igual que en el riel plegado.
    const activo = panel.getByRole("link", { name: "Alertas, 3 sin revisar" });
    await expect.element(activo).toHaveAttribute("aria-current", "page");

    const boton = page.getByRole("button", { name: es["nav.sidebar.collapse"] });
    await expect.element(boton).toHaveAttribute("aria-expanded", "true");
    await boton.click();
    await expect.element(page.getByRole("dialog")).not.toBeInTheDocument();
  });

  it("al expandir, el foco entra en el panel", async () => {
    await renderExpandible();
    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    await expect.element(page.getByRole("dialog")).toHaveFocus();
  });

  it("el anillo de foco del panel sigue su propio contorno, no la cápsula genérica", async () => {
    // Bug real (captura del usuario): el foco global redondea a `--sdm-radius-pill` (999px),
    // pensado para un botón cuadrado — sobre el panel entero (alto y estrecho) eso se veía como
    // una cápsula gigante rodeando todo el riel en vez de un contorno pegado al panel.
    const { container } = await renderExpandible();
    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    await expect.element(page.getByRole("dialog")).toHaveFocus();

    const panel = container.querySelector('[role="dialog"]') as HTMLElement;
    const estilo = getComputedStyle(panel);
    expect(estilo.borderTopLeftRadius).toBe("0px");
    expect(estilo.borderBottomLeftRadius).toBe("0px");
    expect(estilo.borderTopRightRadius).toBe("18px");
    expect(estilo.borderBottomRightRadius).toBe("18px");
  });

  it("Escape cierra el panel y devuelve el foco al botón que lo abrió", async () => {
    await renderExpandible();
    const boton = page.getByRole("button", { name: es["nav.sidebar.expand"] });
    await boton.click();
    await expect.element(page.getByRole("dialog")).toHaveFocus();

    await userEvent.keyboard("{Escape}");
    await expect.element(page.getByRole("dialog")).not.toBeInTheDocument();
    await expect.element(page.getByRole("button", { name: es["nav.sidebar.expand"] })).toHaveFocus();
  });

  it("un clic fuera del panel lo cierra", async () => {
    const { container } = await renderExpandible();
    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    await expect.element(page.getByRole("dialog")).toBeInTheDocument();

    // La capa de clic-fuera es el único elemento `fixed inset-0` fuera del propio panel. `role`
    // "presentation" la saca del árbol de accesibilidad, así que no hay una consulta por rol para
    // ella: `userEvent.click` (a diferencia de un `.click()` nativo) sí espera al reflujo de Svelte.
    const capa = container.querySelector("div.fixed.inset-0") as HTMLElement;
    await userEvent.click(capa);
    await expect.element(page.getByRole("dialog")).not.toBeInTheDocument();
  });

  it("un clic dentro del panel no lo cierra por sí solo", async () => {
    const { container } = await renderExpandible();
    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    // El hueco vacío del propio panel (`flex-1`, nunca un enlace, para no disparar una navegación
    // real en el navegador de pruebas): al ser hermano de la capa de clic-fuera, no le llega el clic.
    const hueco = container.querySelector('[role="dialog"] .flex-1') as HTMLElement;
    await userEvent.click(hueco);
    await expect.element(page.getByRole("dialog")).toBeInTheDocument();
  });
});
