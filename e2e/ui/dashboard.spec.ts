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

  test("una alerta activa tiñe el panel: ni «Todo en orden», y el héroe ofrece «Ver la alerta» (B.1)", async ({
    page
  }) => {
    // El inventario trae disk-0 en `state: "ok"` y una alerta `warn` activa a su nombre. El backend
    // no funde una cosa con la otra (`enrich_with_smart_data` pasa `None`); el panel sí debe hacerlo.
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await expect(page.getByText(inventario.devices[0].model).first()).toBeVisible();
    await expect(page.getByText(es["global.allGood"])).toHaveCount(0);
    await expect(page.getByText(es["global.needsAttention.one"]).first()).toBeVisible();
    await expect(page.getByRole("button", { name: es["dashboard.hero.viewAlert"] }).first()).toBeVisible();
  });

  test("la fila inferior trae «Sucesos del sistema» y «Reparto de estados»", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await expect(page.getByText(es["dashboard.events.title"])).toBeVisible();
    await expect(page.getByText(es["dashboard.spread.title"])).toBeVisible();
  });

  test("pulsar un suceso del panel lleva a ese suceso en la pantalla de eventos", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    // El primer suceso del fixture (`eventos[0]`, id "1") es un enlace, no un botón.
    const fila = page.getByRole("link", { name: /Volumen C: es correcto/ });
    await expect(fila).toHaveAttribute("href", "/events?focus=1");
    await fila.click();
    await expect(page).toHaveURL(/\/events\?focus=1/);
  });

  test("un disco que dejó de responder a SMART (unreadable) cuenta como advertencia, no se calla (§B.5)", async ({
    page
  }) => {
    const disco = inventario.devices[0];
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_alert_groups: [],
      get_devices: {
        ...inventario,
        devices: [
          {
            ...disco,
            state: "unknown",
            unknownReason: "unreadable",
            // El backend deja la última lectura vieja; la tarjeta no debe enseñarla como actual.
            temperatureC: 44,
            lastReadAt: "2026-09-04T08:00:00Z"
          }
        ]
      }
    });
    await page.goto("/");

    await expect(page.getByText(disco.model).first()).toBeVisible();
    // El chrome lo cuenta para «necesitan atención» (§B.5)...
    await expect(page.getByText(es["global.allGood"])).toHaveCount(0);
    await expect(page.getByText(es["global.needsAttention.one"]).first()).toBeVisible();
    // ...pero la tarjeta y el Hero lo presentan como «Sin datos SMART» (gris), no como advertencia,
    // y no enseñan la temperatura vieja.
    await expect(page.getByText(es["disk.noSmartData"]).first()).toBeVisible();
    await expect(page.getByText("44 °C")).toHaveCount(0);
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
