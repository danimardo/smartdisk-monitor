import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import Toolbar from "./Toolbar.svelte";

/** La `Toolbar` de v3: título de la ruta, píldora de estado global con icono (única fuente), sin
 *  botón «?». Estas pruebas fijan lo que jsdom no ve bien y lo que corrige defectos de v2. */

describe("Toolbar (v3)", () => {
  it("muestra el título que recibe, no uno fijo", async () => {
    await render(Toolbar, {
      props: { title: "Disco de trabajo", globalState: "ok", globalLabel: "Todo en orden" }
    });
    await expect.element(page.getByText("Disco de trabajo")).toBeInTheDocument();
  });

  it("la píldora de estado global lleva icono y texto", async () => {
    const { container } = await render(Toolbar, {
      props: { title: "Panel general", globalState: "crit", globalLabel: "1 disco necesita atención" }
    });
    await expect.element(page.getByText("1 disco necesita atención")).toBeInTheDocument();
    expect(container.querySelector("use")!.getAttribute("href")).toBe("#i-bolt");
  });

  it("no hay botón «?» (Acerca de vive en el riel)", async () => {
    await render(Toolbar, {
      props: { title: "Panel general", globalState: "ok", globalLabel: "Todo en orden" }
    });
    expect(page.getByRole("button", { name: "?" }).query()).toBeNull();
  });

  it("la frescura obsoleta se pinta en text-warn", async () => {
    const { container } = await render(Toolbar, {
      props: {
        title: "Panel general",
        globalState: "ok",
        globalLabel: "Todo en orden",
        freshness: "hace 14 min",
        stale: true
      }
    });
    const fresh = [...container.querySelectorAll("span")].find((s) => s.textContent === "hace 14 min")!;
    expect(fresh.className).toContain("text-warn");
  });

  it("el botón primario solo aparece con primaryLabel y llama a onprimary", async () => {
    const onprimary = vi.fn();
    await render(Toolbar, {
      props: {
        title: "Panel general",
        globalState: "ok",
        globalLabel: "Todo en orden",
        primaryLabel: "Actualizar",
        onprimary
      }
    });
    await page.getByRole("button", { name: "Actualizar" }).click();
    expect(onprimary).toHaveBeenCalledOnce();
  });
});
