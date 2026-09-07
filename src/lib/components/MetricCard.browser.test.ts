import { page, userEvent } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import MetricCard from "./MetricCard.svelte";
import es from "$lib/i18n/es.json";

/** `MetricCard` es responsable de dos de los cinco estados de la constitución §VIII: dato ausente
 *  (nunca 0 ni vacío) y dato obsoleto (la marca de edad, `age`). «Vacío», «no compatible» y
 *  «cargando» los deciden `EmptyState`/`ProgressBar` a nivel de pantalla, no esta tarjeta. */

describe("MetricCard", () => {
  it("con valor, lo muestra tal cual, no como «No disponible»", async () => {
    await render(MetricCard, { props: { label: "Temperatura", value: "42 °C" } });
    await expect.element(page.getByText("42 °C")).toBeInTheDocument();
    expect(page.getByText(es["common.notAvailable"]).query()).toBeNull();
  });

  it("dato ausente: se compone como texto «No disponible», nunca como cifra ni como 0", async () => {
    await render(MetricCard, { props: { label: "Temperatura", value: null } });
    await expect.element(page.getByText(es["common.notAvailable"])).toBeInTheDocument();
    expect(page.getByText("0").query()).toBeNull();
  });

  it("dato obsoleto: la marca de edad se muestra junto a la procedencia", async () => {
    await render(MetricCard, {
      props: { label: "Temperatura", value: "42 °C", provenance: "SMART", age: "hace 12 min" }
    });
    await expect.element(page.getByText("SMART · hace 12 min")).toBeInTheDocument();
  });

  it("sin procedencia ni edad, no deja un separador huérfano", async () => {
    await render(MetricCard, { props: { label: "Temperatura", value: "42 °C" } });
    expect(page.getByText("·").query()).toBeNull();
  });

  it("v3: pinta el icono pedido referenciando su símbolo del sprite", async () => {
    const { container } = await render(MetricCard, {
      props: { label: "Temperatura", value: "42 °C", icon: "temp" }
    });
    expect(container.querySelector('use[href="#i-temp"]')).not.toBeNull();
  });

  it("v3: con serie de valores dibuja la sparkline; sin valor no la dibuja", async () => {
    const serie = [
      { t: 1, v: 40 },
      { t: 2, v: 42 },
      { t: 3, v: 41 }
    ];
    const conValor = await render(MetricCard, {
      props: { label: "Temperatura", value: "42 °C", series: serie }
    });
    expect(conValor.container.querySelector("svg polyline, svg path")).not.toBeNull();

    const ausente = await render(MetricCard, {
      props: { label: "Temperatura", value: null, series: serie }
    });
    expect(ausente.container.querySelector("svg polyline, svg path")).toBeNull();
  });

  it("su sparkline es interactiva: foco de teclado y globo de lectura con la unidad", async () => {
    const serie = [
      { t: Date.parse("2026-09-06T08:00:00Z"), v: 40 },
      { t: Date.parse("2026-09-06T09:00:00Z"), v: 42 },
      { t: Date.parse("2026-09-06T10:00:00Z"), v: 41 }
    ];
    const { container } = await render(MetricCard, {
      props: { label: "Temperatura", value: "42 °C", series: serie, unidad: "°C" }
    });
    // El primer `svg` es el del icono; el de la sparkline es el que se estira (`preserveAspectRatio`).
    const svg = container.querySelector('svg[preserveAspectRatio="none"]')!;
    expect(svg.getAttribute("tabindex")).toBe("0");
    (svg as unknown as HTMLElement).focus();
    await userEvent.keyboard("{Home}");
    expect(container.querySelector('[aria-live="polite"]')?.textContent).toContain("40");
    expect(container.querySelector('[aria-live="polite"]')?.textContent).toContain("°C");
  });
});
