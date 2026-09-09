import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import OnboardingArt from "./OnboardingArt.svelte";

/** `OnboardingArt` es decorativa (ADR-039): sale `aria-hidden`, solo con tokens, una escena por
 *  paso del asistente. La prueba cubre que las cinco escenas se dibujan y que ninguna trae un color
 *  literal ni un nombre accesible (lo demás —contraste, tema oscuro— se ve, no se asegura aquí). */

const ESCENAS = ["welcome", "disks", "alerts", "ai", "done"] as const;

describe("OnboardingArt", () => {
  it("las cinco escenas dibujan un SVG decorativo, sin literal de color ni nombre accesible", async () => {
    for (const name of ESCENAS) {
      const { container } = await render(OnboardingArt, { props: { name } });
      const svg = container.querySelector("svg");
      expect(svg, `escena ${name}`).not.toBeNull();
      expect(svg!.getAttribute("aria-hidden")).toBe("true");
      expect(svg!.getAttribute("aria-label")).toBeNull();
      // Cada escena tiene formas de verdad, no un lienzo vacío.
      expect(svg!.querySelectorAll("path, rect, circle, ellipse").length).toBeGreaterThan(3);
      // Todo el color sale de tokens: ningún `fill`/`stroke` con `#`, `rgb(` ni nombre CSS suelto.
      for (const el of svg!.querySelectorAll("[fill], [stroke]")) {
        for (const attr of ["fill", "stroke"]) {
          const v = el.getAttribute(attr);
          if (v && v !== "none") {
            expect(v.startsWith("var(--sdm-") || v === "currentColor", `${name} ${attr}=${v}`).toBe(true);
          }
        }
      }
    }
  });

  it("el alto sale de la proporción 8:5 del ancho pedido", async () => {
    const { container } = await render(OnboardingArt, { props: { name: "ai", width: 240 } });
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("width")).toBe("240");
    expect(svg.getAttribute("height")).toBe("150");
  });
});
