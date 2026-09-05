import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import CapacityBar from "./CapacityBar.svelte";
import es from "$lib/i18n/es.json";

/** `CapacityBar` es responsable del estado de dato ausente de la constitución §VIII: sin
 *  capacidad o espacio libre conocidos, no dibuja ningún relleno y lo dice como texto, nunca como
 *  una barra vacía indistinguible de «disco lleno al 0 %». */

describe("CapacityBar", () => {
  it("con datos, muestra el espacio libre y dibuja el relleno", async () => {
    const { container } = await render(CapacityBar, {
      props: { label: "C:", capacityBytes: 100_000_000_000, freeBytes: 50_000_000_000 }
    });
    await expect
      .element(page.getByRole("progressbar", { name: "C:" }))
      .toHaveAttribute("aria-valuenow", "50");
    expect(container.querySelector('[style*="width"]')).not.toBeNull();
  });

  it("dato ausente: no hay relleno y el texto dice «No disponible», no 0 %", async () => {
    const { container } = await render(CapacityBar, {
      props: { label: "C:", capacityBytes: null, freeBytes: null }
    });
    await expect.element(page.getByText(es["common.notAvailable"])).toBeInTheDocument();
    expect(container.querySelector('[style*="width"]')).toBeNull();
    const barra = page.getByRole("progressbar", { name: "C:" });
    await expect.element(barra).not.toHaveAttribute("aria-valuenow");
  });
});
