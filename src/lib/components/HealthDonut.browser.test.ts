import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import HealthDonut from "./HealthDonut.svelte";

/** `HealthDonut` es responsable del estado vacío de la constitución §VIII: sin ningún disco
 *  monitorizado (los cuatro contadores a cero), no debe dibujar ningún segmento de color —
 *  dibujar un anillo entero de un color inventado mentiría sobre datos que no existen. */

describe("HealthDonut", () => {
  it("vacío: con todos los contadores a cero, no dibuja ningún segmento", async () => {
    const { container } = await render(HealthDonut, {
      props: { counts: { ok: 0, warn: 0, crit: 0, unknown: 0 } }
    });
    // El aro de fondo es el único <circle>; ningún segmento de color se añade con conteo cero.
    expect(container.querySelectorAll("circle").length).toBe(1);
  });

  it("con datos, dibuja un segmento por cada estado presente", async () => {
    const { container } = await render(HealthDonut, {
      props: { counts: { ok: 3, warn: 1, crit: 0, unknown: 0 } }
    });
    // El aro de fondo más un segmento por cada estado con conteo > 0 (ok y warn).
    expect(container.querySelectorAll("circle").length).toBe(3);
  });

  it("siempre lleva nombre accesible, el anillo solo no basta", async () => {
    await render(HealthDonut, { props: { counts: { ok: 1, warn: 0, crit: 0, unknown: 0 } } });
    await expect.element(page.getByRole("img")).toBeInTheDocument();
  });
});
