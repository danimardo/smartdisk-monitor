import { expect, test } from "@playwright/test";
import { RESPUESTAS, appInfoDePrueba } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Diálogo «Acerca de» del riel (US-061): nombre y versión dinámicos, foto y bio del autor
 *  (ADR-052), cierre con Escape y con la cruz. */

test.describe("acerca de", () => {
  test("abre el diálogo con el nombre y la versión reales y presenta al autor", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.about"] }).click();

    const dialogo = page.getByRole("dialog", {
      name: `${appInfoDePrueba.name} ${appInfoDePrueba.version}`
    });
    await expect(dialogo).toBeVisible();
    await expect(dialogo.getByText(appInfoDePrueba.author)).toBeVisible();
    await expect(dialogo.getByText(es["about.authorName"])).toBeVisible();
    await expect(dialogo.getByRole("img", { name: es["about.photoAlt"] })).toBeVisible();
    await expect(dialogo.getByText(es["about.bio.p1"].slice(0, 40), { exact: false })).toBeVisible();
  });

  test("Escape cierra el diálogo", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.about"] }).click();
    const dialogo = page.getByRole("dialog");
    await expect(dialogo).toBeVisible();

    // El foco entra al panel al abrir (`docs/known-issues.md` #2): basta con pulsar Escape sin
    // ningún clic previo para que burbujee hasta el velo.
    await page.keyboard.press("Escape");
    await expect(dialogo).not.toBeVisible();
  });

  test("la cruz de la esquina cierra el diálogo", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.about"] }).click();
    const dialogo = page.getByRole("dialog");
    await expect(dialogo).toBeVisible();

    // La cruz y el botón del pie comparten el nombre accesible «Cerrar»; la cruz es la primera.
    await dialogo.getByRole("button", { name: es["common.close"] }).first().click();
    await expect(dialogo).not.toBeVisible();
  });
});
