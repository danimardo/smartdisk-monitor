import { expect, test } from "@playwright/test";
import { alertaActiva, RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Pantalla de alertas (US-030/031, `docs/ui-design.md` §7.3): lista + detalle con acciones. */

test.describe("alertas", () => {
  test("pinta la alerta activa en la lista y su detalle al seleccionarla", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/alerts");

    const titulo = es[`alert.rule.${alertaActiva.ruleKey}.title` as keyof typeof es];
    const tarjeta = page.getByRole("button", { name: new RegExp(titulo) });
    await expect(tarjeta).toBeVisible();

    await tarjeta.click();
    await expect(page.getByRole("heading", { name: titulo })).toBeVisible();
    await expect(page.getByText(es["alert.fact.ruleKey"])).toBeVisible();
  });

  test("cambiar al filtro de resueltas vacía la lista (la única alerta está activa)", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/alerts");

    await page.getByRole("radio", { name: es["alerts.filter.resolved"] }).click();
    await expect(page.getByText(es["alerts.empty.title"])).toBeVisible();

    await page.getByRole("radio", { name: es["alerts.filter.active"] }).click();
    await expect(page.getByText(es["alerts.empty.title"])).not.toBeVisible();
  });

  test("archivar pide confirmación y no llama al comando si se cancela", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/alerts");

    await page.getByRole("button", { name: es["alerts.actions.archive"] }).click();
    const dialogo = page.getByRole("dialog", { name: es["alerts.archive.confirmTitle"] });
    await expect(dialogo).toBeVisible();

    // `ConfirmDialog.svelte` renderiza "Cancelar" como literal, no vía `t()` — pendiente, ver
    // el informe de esta tarea; no es defecto introducido aquí, es preexistente al componente.
    await dialogo.getByRole("button", { name: "Cancelar" }).click();
    await expect(dialogo).not.toBeVisible();

    expect((await llamadas(page)).map((l) => l.comando)).not.toContain("archive_alert");
  });
});
