import { page } from "vitest/browser";
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
});
