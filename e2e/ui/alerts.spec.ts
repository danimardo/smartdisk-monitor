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

  test("las reglas nuevas del motor muestran su título traducido, no la clave", async ({ page }) => {
    const alerta = (id: string, ruleKey: string) => ({
      ...alertaActiva,
      id,
      ruleKey,
      deduplicationKey: `${ruleKey}|device:disk-0`
    });
    const reglas = [
      "capacity.low",
      "smart.unreadable",
      "temp.above_vendor_limit",
      "collector.stalled"
    ] as const;
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_alert_groups: reglas.map((r, i) => alerta(`a-${i}`, r))
    });
    await page.goto("/alerts");

    for (const r of reglas) {
      await expect(page.getByText(es[`alert.rule.${r}.title`]).first()).toBeVisible();
    }
    await expect(page.getByText(/^alert\.rule\./)).toHaveCount(0);
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

    await dialogo.getByRole("button", { name: es["common.cancel"] }).click();
    await expect(dialogo).not.toBeVisible();

    expect((await llamadas(page)).map((l) => l.comando)).not.toContain("archive_alert");
  });
});
