import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import Sidebar from "./Sidebar.svelte";
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
