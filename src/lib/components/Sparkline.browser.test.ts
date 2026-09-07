import { userEvent } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Sparkline from "./Sparkline.svelte";
import type { Punto } from "$lib/design/series";

/** `Sparkline` por defecto es contexto, no lectura: sin ejes ni etiqueta. Con `interactivo` gana el
 *  cursor de lectura (ratón + teclado) y el globo. Lo que jsdom no ve bien —el trazo por tramos,
 *  curvo, el foco y el cursor— se comprueba aquí. */

describe("Sparkline", () => {
  it("serie de dos tramos: dos <path> de trazo, nunca una línea que cruza el hueco", async () => {
    const points: Punto[] = [
      { t: 0, v: 10 },
      { t: 1, v: 12 },
      { t: 2, v: null },
      { t: 3, v: 20 },
      { t: 4, v: 22 }
    ];
    const { container } = await render(Sparkline, { props: { points } });
    expect(container.querySelectorAll('path[fill="none"]')).toHaveLength(2);
  });

  it("el trazo es una curva, no una polilínea recta (`C` en el path)", async () => {
    const points: Punto[] = [
      { t: 0, v: 10 },
      { t: 1, v: 14 },
      { t: 2, v: 9 },
      { t: 3, v: 16 }
    ];
    const { container } = await render(Sparkline, { props: { points } });
    expect(container.querySelector('path[fill="none"]')?.getAttribute("d")).toContain(" C ");
  });

  it("todo trazo lleva vector-effect=non-scaling-stroke (no desaparece al estirar)", async () => {
    const { container } = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: 1 },
          { t: 1, v: 2 }
        ]
      }
    });
    for (const pl of container.querySelectorAll('path[fill="none"]')) {
      expect(pl.getAttribute("vector-effect")).toBe("non-scaling-stroke");
    }
  });

  it("serie vacía: no renderiza SVG, no dibuja una línea a cero", async () => {
    const { container } = await render(Sparkline, { props: { points: [] } });
    expect(container.querySelector("svg")).toBeNull();
  });

  it("serie toda null: igual que vacía", async () => {
    const { container } = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: null },
          { t: 1, v: null }
        ] as Punto[]
      }
    });
    expect(container.querySelector("svg")).toBeNull();
  });

  it("con fill, el degradado tiene un id único por instancia", async () => {
    const a = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: 1 },
          { t: 1, v: 2 }
        ] as Punto[],
        fill: true
      }
    });
    const b = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: 1 },
          { t: 1, v: 2 }
        ] as Punto[],
        fill: true
      }
    });
    const idA = a.container.querySelector("linearGradient")!.id;
    const idB = b.container.querySelector("linearGradient")!.id;
    expect(idA).not.toBe(idB);
  });

  it("sin label queda aria-hidden; con label sale como imagen con nombre", async () => {
    const sinLabel = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: 1 },
          { t: 1, v: 2 }
        ] as Punto[]
      }
    });
    expect(sinLabel.container.querySelector("svg")!.getAttribute("aria-hidden")).toBe("true");

    const conLabel = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: 1 },
          { t: 1, v: 2 }
        ] as Punto[],
        label: "Temperatura 24 h"
      }
    });
    const svg = conLabel.container.querySelector("svg")!;
    expect(svg.getAttribute("role")).toBe("img");
    expect(svg.getAttribute("aria-label")).toBe("Temperatura 24 h");
  });

  it("sin `interactivo`: no es foco de teclado ni reacciona (fondo decorativo intacto)", async () => {
    const { container } = await render(Sparkline, {
      props: {
        points: [
          { t: 0, v: 1 },
          { t: 1, v: 2 }
        ] as Punto[]
      }
    });
    const svg = container.querySelector("svg")!;
    expect(svg.hasAttribute("tabindex")).toBe(false);
    expect(container.querySelector('[aria-live="polite"]')).toBeNull();
  });

  it("con `interactivo`: es foco de teclado con nombre accesible, y las flechas mueven el cursor", async () => {
    const points: Punto[] = Array.from({ length: 12 }, (_, i) => ({
      t: Date.parse("2026-09-06T08:00:00Z") + i * 3_600_000,
      v: 40 + i
    }));
    const { container } = await render(Sparkline, {
      props: { points, interactivo: true, unidad: "°C" }
    });
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("role")).toBe("img");
    expect(svg.getAttribute("tabindex")).toBe("0");
    expect(svg.getAttribute("aria-label")).toBeTruthy();

    const viva = container.querySelector('[aria-live="polite"]')!;
    expect(viva.textContent).toBe(""); // nada señalado todavía

    (svg as unknown as HTMLElement).focus();
    await userEvent.keyboard("{Home}");
    // El primer punto: 40 °C. El globo y la región viva lo anuncian.
    expect(viva.textContent).toContain("40");
    expect(viva.textContent).toContain("°C");
    expect(container.querySelector('[aria-hidden="true"]')?.textContent).toContain("40");

    await userEvent.keyboard("{ArrowRight}{ArrowRight}");
    expect(viva.textContent).toContain("42");
  });
});
