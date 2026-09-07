import { expect, test } from "@playwright/test";
import { RESPUESTAS, appInfoDePrueba } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Botón «Acerca de» de la `Toolbar` (US-061): nombre y versión dinámicos, cierre con Escape. */

test.describe("acerca de", () => {
  test("el botón ? abre el diálogo con el nombre y la versión reales", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.about"] }).click();

    const dialogo = page.getByRole("dialog", {
      name: `${appInfoDePrueba.name} ${appInfoDePrueba.version}`
    });
    await expect(dialogo).toBeVisible();
    await expect(dialogo.getByText(appInfoDePrueba.author)).toBeVisible();
  });

  test("Escape cierra el diálogo", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.about"] }).click();
    const dialogo = page.getByRole("dialog");
    await expect(dialogo).toBeVisible();

    // El foco entra al panel al abrir (`ConfirmDialog.svelte`, `docs/known-issues.md` #2): basta
    // con pulsar Escape sin ningún clic previo para que burbujee hasta el velo.
    await page.keyboard.press("Escape");
    await expect(dialogo).not.toBeVisible();
  });

  test("la cruz de la esquina cierra el diálogo", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.about"] }).click();
    const dialogo = page.getByRole("dialog");
    await expect(dialogo).toBeVisible();

    await dialogo.getByRole("button", { name: es["common.close"] }).click();
    await expect(dialogo).not.toBeVisible();
  });
});
