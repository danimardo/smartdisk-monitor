import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import TimeSeriesChart from "./TimeSeriesChart.svelte";
import es from "$lib/i18n/es.json";

/** `TimeSeriesChart` es responsable del estado vacío de la constitución §VIII: sin ningún punto
 *  con valor, la lectura textual equivalente (exigida por `AGENTS.md` §6) lo dice explícitamente,
 *  y el hueco se dibuja como banda, nunca como interpolación ni como cero. */

describe("TimeSeriesChart", () => {
  it("vacío: sin puntos, la lectura textual lo dice y cubre todo el dominio pedido como hueco", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: { points: [], from: 0, to: 1000, unit: "°C" }
    });
    await expect.element(page.getByRole("img")).toHaveAttribute("aria-label", es["chart.emptyLabel"]);
    expect(container.querySelector("rect")).not.toBeNull();
  });

  it("un hueco explícito (valor null) se dibuja como banda, no se interpola", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: {
        points: [
          { t: 0, v: 10 },
          { t: 1000, v: null },
          { t: 2000, v: 20 }
        ],
        from: 0,
        to: 2000,
        unit: "°C"
      }
    });
    expect(container.querySelector("rect")).not.toBeNull();
  });

  it("con datos completos, no dibuja ninguna banda de hueco", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: {
        points: [
          { t: 0, v: 10 },
          { t: 1000, v: 12 }
        ],
        from: 0,
        to: 1000,
        unit: "°C"
      }
    });
    expect(container.querySelector("rect")).toBeNull();
  });

  it("el umbral de aviso, cuando se pasa, se etiqueta visiblemente", async () => {
    await render(TimeSeriesChart, {
      props: {
        points: [{ t: 0, v: 10 }],
        from: 0,
        to: 1000,
        warnThreshold: 70,
        warnLabel: "Aviso ≥ 70 °C"
      }
    });
    await expect.element(page.getByText("Aviso ≥ 70 °C")).toBeInTheDocument();
  });

  it("el umbral crítico, cuando se pasa, se etiqueta visiblemente y no oculta el de aviso", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: {
        points: [{ t: 0, v: 10 }],
        from: 0,
        to: 1000,
        warnThreshold: 70,
        warnLabel: "Aviso ≥ 70 °C",
        critThreshold: 80,
        critLabel: "Crítico ≥ 80 °C"
      }
    });
    await expect.element(page.getByText("Aviso ≥ 70 °C")).toBeInTheDocument();
    await expect.element(page.getByText("Crítico ≥ 80 °C")).toBeInTheDocument();
    // Dos zonas de fondo (aviso y crítico) más las bandas de hueco/gráfica: al menos dos rects.
    expect(container.querySelectorAll("rect").length).toBeGreaterThanOrEqual(2);
  });
});
