import { expect, test } from "@playwright/test";
import { RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Pantalla de ajustes (épica H de `docs/user-stories.md`): frecuencias con límites, restaurar
 *  valores de fábrica, y borrado de todos los datos con frase de confirmación. */

test.describe("ajustes", () => {
  test("cambiar una frecuencia llama a set_setting con la clave y el valor correctos", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/settings");

    const campo = page.getByRole("spinbutton", { name: es["settings.schedule.metricsFast"] });
    await campo.fill("45");
    await campo.blur();

    const llamada = (await llamadas(page)).find((l) => l.comando === "set_setting");
    expect(llamada?.args).toMatchObject({ key: "schedule.metrics_fast_seconds", value: 45 });
  });

  test("restaurar valores de fábrica de un ámbito llama a reset_settings con ese ámbito", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/settings");

    const tarjetaFrecuencias = page.locator("section", { hasText: es["settings.schedule.title"] });
    await tarjetaFrecuencias.getByRole("button", { name: es["settings.cta.restoreDefaults"] }).click();

    const llamada = (await llamadas(page)).find((l) => l.comando === "reset_settings");
    expect(llamada?.args).toMatchObject({ scope: "schedule" });
  });

  test("borrar todos los datos exige escribir la frase exacta antes de confirmar", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/settings");

    const boton = page.getByRole("button", { name: es["settings.dangerZone.cta"] });
    await expect(boton).toBeDisabled();

    await page.getByRole("textbox", { name: /SmartDisk Monitor/ }).fill("SmartDisk Monitor");
    await expect(boton).toBeEnabled();
    await boton.click();

    const dialogo = page.getByRole("dialog", { name: es["settings.dangerZone.confirmTitle"] });
    await expect(dialogo).toBeVisible();
    await dialogo.getByRole("button", { name: es["settings.dangerZone.confirmLabel"] }).click();

    const llamada = (await llamadas(page)).find((l) => l.comando === "delete_all_data");
    expect(llamada?.args).toMatchObject({ confirmationPhrase: "SmartDisk Monitor" });
  });
});
