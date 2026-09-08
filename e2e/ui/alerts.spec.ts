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
      "collector.stalled",
      "events.disk_error",
      "events.filesystem_error",
      "events.controller_reset",
      "events.paging_error",
      "events.delayed_write",
      "events.disk_predictive",
      "events.storage_space_degraded",
      "events.filesystem_repaired",
      "events.filesystem_repair_storm",
      "events.io_retry",
      "device.removed_unexpected",
      "inventory.duplicate_id"
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

  test("una alerta de evento enlaza a su suceso en la pantalla de eventos", async ({ page }) => {
    const alerta = {
      ...alertaActiva,
      id: "a-ev",
      ruleKey: "events.disk_error",
      deduplicationKey: "events.disk_error|device:disk-0|disk:7"
    };
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_alert_groups: [alerta],
      get_alert_detail: {
        ...alerta,
        facts: [{ labelKey: "alert.fact.ruleKey", value: "events.disk_error" }],
        occurrences: [
          { occurredAt: new Date().toISOString(), cycle: 1, value: null, eventId: "2", context: null }
        ],
        relatedEvents: []
      }
    });
    await page.goto("/alerts");
    await page.getByRole("button", { name: new RegExp(es["alert.rule.events.disk_error.title"]) }).click();

    const enlace = page.getByRole("link", { name: es["alerts.timeline.viewEvent"] });
    await expect(enlace).toHaveAttribute("href", "/events?focus=2");
    await enlace.click();

    await expect(page).toHaveURL(/\/events\?focus=2/);
    // El evento 2 del fixture (disk 157) queda resaltado / abierto.
    await expect(
      page.getByText("El disco 1 se ha extraído de forma imprevista del sistema.").first()
    ).toBeVisible();
  });

  test("cambiar al filtro de resueltas vacía la lista (la única alerta está activa)", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/alerts");

    await page.getByRole("radio", { name: es["alerts.filter.resolved"] }).click();
    await expect(page.getByText(es["alerts.empty.title"])).toBeVisible();

    await page.getByRole("radio", { name: es["alerts.filter.active"] }).click();
    await expect(page.getByText(es["alerts.empty.title"])).not.toBeVisible();
  });

  test("el detalle muestra el disco afectado y permite ver el JSON técnico de smartctl", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/alerts");

    const titulo = es[`alert.rule.${alertaActiva.ruleKey}.title` as keyof typeof es];
    await page.getByRole("button", { name: new RegExp(titulo) }).click();
    await expect(page.getByRole("heading", { name: titulo })).toBeVisible();

    // El disco afectado se ve en el propio detalle, no solo en la tarjeta de la lista.
    await expect(page.getByText(alertaActiva.target).nth(1)).toBeVisible();

    const boton = page.getByRole("button", { name: es["alerts.viewTechnicalDetail"] });
    await expect(boton).toBeVisible();
    await boton.click();
    await expect(page.getByText(/model_name/)).toBeVisible();

    expect((await llamadas(page)).map((l) => l.comando)).toContain("get_alert_smart_raw_json");
  });

  test("una regla que no es de smartctl no ofrece ver detalle técnico de SMART", async ({ page }) => {
    const alerta = {
      ...alertaActiva,
      id: "a-cap",
      ruleKey: "capacity.low",
      deduplicationKey: "capacity.low|volume:v1"
    };
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_alert_groups: [alerta],
      get_alert_detail: {
        ...alerta,
        facts: [{ labelKey: "alert.fact.ruleKey", value: "capacity.low" }],
        occurrences: [],
        relatedEvents: []
      }
    });
    await page.goto("/alerts");
    await page.getByRole("button", { name: new RegExp(es["alert.rule.capacity.low.title"]) }).click();

    await expect(page.getByRole("button", { name: es["alerts.viewTechnicalDetail"] })).toHaveCount(0);
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
