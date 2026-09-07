import { page, userEvent } from "vitest/browser";
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
        min: 0,
        max: 100,
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

  it("v3: dibuja un eje Y con cuatro marcas numéricas fuera del área de trazo", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: {
        points: [
          { t: 0, v: 40 },
          { t: 1000, v: 44 }
        ],
        from: 0,
        to: 1000
      }
    });
    const textos = [...container.querySelectorAll("text")].filter(
      (el) => el.getAttribute("text-anchor") === "end"
    );
    expect(textos).toHaveLength(4);
  });

  it("v3: el trazo se dibuja (regresión — la gráfica salía vacía en v2)", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: {
        points: [
          { t: 0, v: 41 },
          { t: 100, v: 42 },
          { t: 200, v: 41 },
          { t: 300, v: null },
          { t: 700, v: 43 },
          { t: 800, v: 44 },
          { t: 900, v: 43 }
        ],
        from: 0,
        to: 900,
        expectedIntervalMs: 100
      }
    });
    // dos tramos continuos ⇒ dos trazos (cada uno un <path> de línea), no un rectángulo hueco
    const lineas = container.querySelectorAll('path[fill="none"]');
    expect(lineas.length).toBe(2);
    // y son curvas, no polilíneas rectas
    expect(lineas[0].getAttribute("d")).toContain(" C ");
  });

  it("v3: el hueco lleva su leyenda con el rango de horas", async () => {
    await render(TimeSeriesChart, {
      props: {
        points: [
          { t: Date.parse("2026-09-06T08:00:00Z"), v: 41 },
          { t: Date.parse("2026-09-06T14:00:00Z"), v: 43 }
        ],
        from: Date.parse("2026-09-06T08:00:00Z"),
        to: Date.parse("2026-09-06T14:00:00Z"),
        expectedIntervalMs: 60_000
      }
    });
    await expect.element(page.getByText(/sin datos/)).toBeInTheDocument();
  });

  it("al recorrer la serie con el teclado, el valor sale en un globo junto al punto, no en el pie", async () => {
    const { container } = await render(TimeSeriesChart, {
      props: {
        points: [
          { t: Date.parse("2026-09-06T08:00:00Z"), v: 41 },
          { t: Date.parse("2026-09-06T09:00:00Z"), v: 45 },
          { t: Date.parse("2026-09-06T10:00:00Z"), v: 43 }
        ],
        from: Date.parse("2026-09-06T08:00:00Z"),
        to: Date.parse("2026-09-06T10:00:00Z"),
        expectedIntervalMs: 3_600_000,
        unit: "°C"
      }
    });
    const svg = container.querySelector("svg")!;
    (svg as unknown as HTMLElement).focus();
    await userEvent.keyboard("{Home}");

    // El globo (aria-hidden) y la región viva llevan el valor.
    const globo = container.querySelector('[aria-hidden="true"]');
    expect(globo?.textContent).toContain("41");
    expect(container.querySelector('[aria-live="polite"]')?.textContent).toContain("41");

    // El pie ya no repite el valor: sigue mostrando su contexto (aquí no hay huecos ⇒ vacío salvo
    // los extremos de fecha), nunca «41 °C».
    const pie = container.querySelector("figcaption")!;
    expect(pie.textContent).not.toContain("41 °C");
  });
});
