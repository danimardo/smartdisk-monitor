import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import ChartTip from "./ChartTip.svelte";

/** `ChartTip` es el globo de lectura compartido por todas las gráficas: posiciona en píxeles lo que
 *  le pasa el llamante y **voltea en los bordes** para no recortarse. jsdom no ve el `transform`;
 *  se comprueba en Chromium. */

describe("ChartTip", () => {
  it("muestra el texto y se ancla en la posición dada", async () => {
    const { container } = await render(ChartTip, {
      props: { x: 120, y: 80, anchoContenedor: 400, texto: "44 °C · 7 sep 15:30" }
    });
    const globo = container.querySelector("div")!;
    expect(globo.textContent?.trim()).toBe("44 °C · 7 sep 15:30");
    expect(globo.getAttribute("aria-hidden")).toBe("true");
    expect(globo.style.left).toBe("120px");
    expect(globo.style.top).toBe("80px");
  });

  /** El eje X de `transform: translate(<x>, <y>)`. */
  const ejeX = (el: Element) => (el as HTMLElement).style.transform.match(/translate\(([^,]+),/)?.[1] ?? "";
  const ejeY = (el: Element) => (el as HTMLElement).style.transform.match(/,\s*(.+)\)$/)?.[1] ?? "";

  it("cerca del borde derecho se voltea a la izquierda del punto", async () => {
    const { container: cerca } = await render(ChartTip, {
      props: { x: 380, y: 80, anchoContenedor: 400, texto: "valor largo de ejemplo" }
    });
    const { container: lejos } = await render(ChartTip, {
      props: { x: 20, y: 80, anchoContenedor: 400, texto: "valor largo de ejemplo" }
    });
    // Pegado a la derecha, el desfase X se lleva el globo hacia la izquierda del punto (`-100%`).
    expect(ejeX(cerca.querySelector("div")!)).toContain("-100%");
    expect(ejeX(lejos.querySelector("div")!)).not.toContain("-100%");
  });

  it("cerca del borde superior se coloca debajo del punto, no encima", async () => {
    const { container: arriba } = await render(ChartTip, {
      props: { x: 100, y: 8, anchoContenedor: 400, texto: "44 °C" }
    });
    const { container: enMedio } = await render(ChartTip, {
      props: { x: 100, y: 120, anchoContenedor: 400, texto: "44 °C" }
    });
    // Con `y` pequeño no hay sitio arriba: el desfase Y coloca el globo debajo (sin `-100%`).
    expect(ejeY(arriba.querySelector("div")!)).not.toContain("-100%");
    expect(ejeY(enMedio.querySelector("div")!)).toContain("-100%");
  });
});
