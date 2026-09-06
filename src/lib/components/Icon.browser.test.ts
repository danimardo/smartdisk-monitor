import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Icon from "./Icon.svelte";

/** La regla de `ui-design.md` §6: un `role="img"` sin nombre es peor que no ponerlo. `Icon` la aplica
 *  en su estructura — con `label` sale como imagen con nombre; sin `label` queda `aria-hidden` y
 *  fuera del árbol de accesibilidad. Estas pruebas fijan ese comportamiento. */

describe("Icon", () => {
  it("sin label: aria-hidden, no lo ve un lector de pantalla", async () => {
    const { container } = await render(Icon, { props: { name: "temp" } });
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("aria-hidden")).toBe("true");
    expect(svg.hasAttribute("role")).toBe(false);
    expect(svg.hasAttribute("aria-label")).toBe(false);
    expect(svg.getAttribute("focusable")).toBe("false");
  });

  it("con label: role img y nombre accesible", async () => {
    await render(Icon, { props: { name: "alert", label: "Advertencia" } });
    const el = page.getByRole("img", { name: "Advertencia" });
    await expect.element(el).toBeInTheDocument();
  });

  it("referencia el símbolo del sprite por su id `i-{name}`", async () => {
    const { container } = await render(Icon, { props: { name: "diskStack" } });
    const use = container.querySelector("use")!;
    expect(use.getAttribute("href")).toBe("#i-diskStack");
  });

  it("hereda currentColor: el svg no fija ningún color propio", async () => {
    const { container } = await render(Icon, { props: { name: "shield", label: "ok" } });
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("fill")).toBeNull();
    expect(svg.style.color).toBe("");
  });

  it("respeta el tamaño pedido", async () => {
    const { container } = await render(Icon, { props: { name: "clock", size: 24 } });
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("width")).toBe("24");
    expect(svg.getAttribute("height")).toBe("24");
  });
});
