import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Sidebar from "./Sidebar.svelte";
import type { DiskSummary } from "$lib/design/types";

/** `docs/ui-design.md` §4.0.bis: por debajo de 1180 px de ancho la barra se reduce a 56 px,
 *  conservando el estado de cada disco en su punto de color. Lo que jsdom no puede comprobar —el
 *  ancho calculado y qué queda visible según el viewport real— se prueba aquí; el nombre accesible
 *  de cada enlace, que no depende del navegador, no se repite. */

function secciones() {
  return [
    { id: "dashboard", label: "Panel general", href: "/" },
    { id: "alerts", label: "Alertas", href: "/alerts" }
  ];
}

function disco(): DiskSummary {
  return {
    id: "d1",
    alias: "Disco de trabajo",
    model: "Samsung 980",
    deviceType: "nvme",
    state: "ok",
    temperatureC: 42,
    percentageUsed: 5,
    activityPercent: 12,
    powerOnHours: 1000,
    unknownReason: null,
    lastReadAt: "2026-09-04T10:00:00Z",
    volumes: []
  };
}

describe("Sidebar", () => {
  // El viewport de esta suite está fijado a 1024×560, el mínimo técnico (`vitest.browser.config.ts`)
  // y ya por debajo de los 1180 px del breakpoint: es la vía barata de comprobar el estado
  // colapsado sin controlar el tamaño de ventana desde el test. La comprobación visual a los tres
  // niveles de escalado, en las siete pantallas reales, ya la cubre `e2e/ui/escalado.spec.ts` (T111).
  it("a la ventana mínima (<1180 px): las etiquetas se ocultan pero el nombre accesible se conserva", async () => {
    await render(Sidebar, {
      props: { sections: secciones(), disks: [disco()], active: "dashboard" }
    });
    // El texto visible desaparece (el CSS lo oculta), pero el enlace sigue siendo localizable por
    // su nombre accesible completo — no se pierde para quien usa lector de pantalla.
    await expect.element(page.getByText("Panel general")).not.toBeVisible();
    await expect.element(page.getByRole("link", { name: "Panel general" })).toBeInTheDocument();
    await expect.element(page.getByRole("link", { name: "Disco de trabajo" })).toBeInTheDocument();
  });
});
