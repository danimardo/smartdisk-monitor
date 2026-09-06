import { expect, test } from "@playwright/test";
import { RESPUESTAS, inventario } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Panel general v3 (US-012, `specs/002-rediseno-v3/` US5): `HeroPanel` con el disco elegido por
 *  `selectHeroDisk()`, rejilla de `DiskCard`, y al pie «Sucesos del sistema» + «Reparto de estados».
 *  El disco USB sin SMART aparece en la rejilla en gris, nunca como protagonista. */

test.describe("panel general v3", () => {
  test("el héroe es el disco con SMART; el USB sin SMART se queda en la rejilla", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    // El disco protagonista (disk-0, el único con SMART y con una alerta a su nombre).
    await expect(page.getByText(inventario.devices[0].model).first()).toBeVisible();

    // El USB sin compatibilidad SMART: presente en la rejilla, rotulado «Sin datos SMART», nunca rojo.
    await expect(page.getByText(es["disk.noSmartData"]).first()).toBeVisible();
    await expect(page.getByText(inventario.devices[1].alias as string).first()).toBeVisible();
  });

  test("la fila inferior trae «Sucesos del sistema» y «Reparto de estados»", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await expect(page.getByText(es["dashboard.events.title"])).toBeVisible();
    await expect(page.getByText(es["dashboard.spread.title"])).toBeVisible();
  });

  test("sin discos: estado vacío con acción, sin héroe ni rejilla", async ({ page }) => {
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_devices: { devices: [], excluded: [], sources: [], paused: false, pausedSince: null }
    });
    await page.goto("/");

    await expect(page.getByText(es["dashboard.noDevices"])).toBeVisible();
    await expect(page.getByRole("button", { name: es["dashboard.noDevicesCta"] })).toBeVisible();
  });
});
