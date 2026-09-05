import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import AlertCard from "./AlertCard.svelte";
import type { AlertGroup } from "$lib/design/types";

/** `AlertCard` es responsable del estado vacío de la constitución §VIII: sin grupo de alerta, no
 *  renderiza nada — la lista la resuelve `EmptyState` a nivel de pantalla, esta tarjeta no debe
 *  dejar un hueco a medio pintar. */

function grupoDePrueba(overrides: Partial<AlertGroup> = {}): AlertGroup {
  return {
    id: "g1",
    ruleKey: "temp.above_configured_warn",
    deduplicationKey: "dedup",
    severity: "warn",
    status: "active",
    count: 3,
    firstOccurredAt: "2026-09-04T00:00:00Z",
    lastOccurredAt: "2026-09-04T10:00:00Z",
    target: "Mi disco",
    mutedUntil: null,
    ...overrides
  };
}

describe("AlertCard", () => {
  it("vacío: sin alerta, no renderiza nada", async () => {
    const { container } = await render(AlertCard, { props: { alert: null } });
    expect(container.textContent?.trim()).toBe("");
  });

  it("con alerta, resuelve título y resumen a partir de la clave de regla, nunca la muestra cruda", async () => {
    await render(AlertCard, { props: { alert: grupoDePrueba() } });
    await expect.element(page.getByText("Temperatura por encima de lo esperado")).toBeInTheDocument();
    expect(page.getByText("temp.above_configured_warn").query()).toBeNull();
  });

  it("al pulsarla, invoca onselect con el id del grupo", async () => {
    const onselect = vi.fn();
    await render(AlertCard, { props: { alert: grupoDePrueba(), onselect } });
    await page.getByRole("button").click();
    expect(onselect).toHaveBeenCalledWith("g1");
  });
});
