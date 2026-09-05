import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import ProgressBar from "./ProgressBar.svelte";

/** `ProgressBar` es el componente que las pantallas usan para el estado «cargando» de una
 *  operación en curso (constitución §VIII): benchmark, chkdsk, autotest. Determinado vs.
 *  indeterminado es justo la distinción que separa «progreso real» de «cargando sin saber cuánto
 *  falta», así que es lo que se prueba aquí. */

describe("ProgressBar", () => {
  it("determinado: expone el porcentaje real como valor accesible", async () => {
    await render(ProgressBar, { props: { value: 42, caption: "Analizando", trailing: "42 %" } });
    const barra = page.getByRole("progressbar");
    await expect.element(barra).toHaveAttribute("aria-valuenow", "42");
    await expect.element(page.getByText("Analizando")).toBeInTheDocument();
    await expect.element(page.getByText("42 %")).toBeInTheDocument();
  });

  it("indeterminado: no anuncia un valor concreto que no existe", async () => {
    await render(ProgressBar, {
      props: { indeterminate: true, caption: "Preparando", trailing: "" }
    });
    const barra = page.getByRole("progressbar");
    await expect.element(barra).not.toHaveAttribute("aria-valuenow");
  });

  it("un valor fuera de rango se recorta a 0-100, nunca se desborda visualmente", async () => {
    const { container } = await render(ProgressBar, { props: { value: 150 } });
    const relleno = container.querySelector<HTMLElement>('[style*="width"]');
    expect(relleno?.style.width).toBe("100%");
  });
});
