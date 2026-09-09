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

  it("indeterminado: un segmento que recorre la pista, no un relleno fijo con porcentaje", async () => {
    const { container } = await render(ProgressBar, { props: { indeterminate: true } });
    const relleno = container.querySelector<HTMLElement>('[role="progressbar"] > div')!;
    // La clase que anima el recorrido, y ningún `width` en línea (eso sería un porcentaje fijo).
    expect(relleno.className).toContain("sdm-indeterminate");
    expect(relleno.getAttribute("style") ?? "").not.toContain("width");
  });

  it("un valor fuera de rango se recorta a 0-100, nunca se desborda visualmente", async () => {
    const { container } = await render(ProgressBar, { props: { value: 150 } });
    const relleno = container.querySelector<HTMLElement>('[style*="width"]');
    expect(relleno?.style.width).toBe("100%");
  });

  it("emphasis por defecto («inline»): relleno plano `bg-accent`, sin degradado", async () => {
    const { container } = await render(ProgressBar, { props: { value: 40 } });
    const relleno = container.querySelector<HTMLElement>('[style*="width"]')!;
    expect(relleno.className).toContain("bg-accent");
    expect(relleno.getAttribute("style") ?? "").not.toContain("linear-gradient");
  });

  it("emphasis «display»: barra de 12 px con degradado del acento y filo interior", async () => {
    const { container } = await render(ProgressBar, {
      props: { value: 40, emphasis: "display" }
    });
    const carril = container.querySelector('[role="progressbar"]')!;
    expect(carril.className).toContain("h-3");
    const relleno = container.querySelector<HTMLElement>('[style*="width"]')!;
    expect(relleno.className).toContain("shadow-edge");
    expect(relleno.getAttribute("style") ?? "").toContain("linear-gradient");
  });
});
