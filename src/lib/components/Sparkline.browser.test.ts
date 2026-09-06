import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Sparkline from "./Sparkline.svelte";
import type { Punto } from "$lib/design/series";

/** `Sparkline` es contexto, no lectura: sin ejes ni etiqueta. Lo que jsdom no ve bien —el trazo por
 *  tramos y que no se degenera al estirarse— se comprueba aquí. */

describe("Sparkline", () => {
  it("serie de dos tramos: dos <polyline>, nunca una línea que cruza el hueco", async () => {
    const points: Punto[] = [
      { t: 0, v: 10 },
      { t: 1, v: 12 },
      { t: 2, v: null },
      { t: 3, v: 20 },
      { t: 4, v: 22 }
    ];
    const { container } = await render(Sparkline, { props: { points } });
    expect(container.querySelectorAll("polyline")).toHaveLength(2);
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
    for (const pl of container.querySelectorAll("polyline")) {
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
});
