import { expect, test } from "@playwright/test";
import {
  RESPUESTAS,
  catalogoModelos,
  detalleAlertaActiva,
  estadoIaActiva,
  estadoIaDesactivada,
  explicacionOk
} from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Ayuda con IA (spec 005-explicacion-ia). US1: activar/gestionar la clave en Ajustes. US2:
 *  explicar una alerta en lenguaje claro, con el fallo degradando solo el modal. */

const conIa = (extra: Record<string, unknown> = {}) => ({
  ...RESPUESTAS,
  estado_ia: estadoIaActiva,
  ...extra
});

test.describe("ayuda con IA — Ajustes (US1)", () => {
  test("sin clave: la sección muestra «Desactivada» y un campo para la clave", async ({ page }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaDesactivada });
    await page.goto("/settings");

    await expect(page.getByText(es["settings.ai.title"])).toBeVisible();
    await expect(page.getByText(es["settings.ai.status.off"])).toBeVisible();
    await expect(page.getByLabel(es["settings.ai.key.label"])).toBeVisible();
  });

  test("introducir una clave llama a guardar_clave_ia y pasa a «Activada»", async ({ page }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaDesactivada });
    await page.goto("/settings");

    await page.getByLabel(es["settings.ai.key.label"]).fill("sk-or-v1-clavedeprueba");
    await page.getByRole("button", { name: es["settings.ai.cta.activate"] }).click();

    const llamada = (await llamadas(page)).find((l) => l.comando === "guardar_clave_ia");
    expect(llamada?.args).toMatchObject({ clave: "sk-or-v1-clavedeprueba" });
    await expect(page.getByText(es["settings.ai.status.on"])).toBeVisible();
  });

  test("si la lista de modelos no carga, se puede seguir con «automático» (US3 escenario 3)", async ({
    page
  }) => {
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      estado_ia: estadoIaActiva,
      listar_modelos_ia: {
        __rechazar__: { code: "ia.network", messageKey: "error.ia.network", retryable: true }
      }
    });
    await page.goto("/settings");

    await expect(page.getByText(es["settings.ai.model.listUnavailable"])).toBeVisible();
    await expect(page.getByRole("option", { name: es["settings.ai.model.auto"] })).toBeAttached();
  });

  test("elegir un modelo de pago pide confirmación antes de guardarlo (US3)", async ({ page }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaActiva });
    await page.goto("/settings");

    const dePago = catalogoModelos.find((m) => m.esDePago)!;
    await page
      .getByLabel(es["settings.ai.model.change"])
      .selectOption({ label: es["settings.ai.model.paidSuffix"].replace("{name}", dePago.nombre) });

    await expect(page.getByText(es["settings.ai.model.paidTitle"])).toBeVisible();
    // Aún no se ha guardado nada.
    expect((await llamadas(page)).some((l) => l.comando === "set_setting")).toBe(false);

    await page.getByRole("button", { name: es["settings.ai.model.paidConfirm"] }).click();
    const guardado = (await llamadas(page)).find((l) => l.comando === "set_setting");
    expect(guardado?.args).toMatchObject({ key: "settings.ai.model", value: dePago.id });
  });
});

test.describe("ayuda con IA — explicar una alerta (US2)", () => {
  test("con la función desactivada, el detalle de la alerta no muestra el botón", async ({ page }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaDesactivada });
    await page.goto("/alerts");

    const titulo = es[`alert.rule.${detalleAlertaActiva.ruleKey}.title` as keyof typeof es];
    await page.getByRole("button", { name: new RegExp(titulo) }).click();
    await expect(page.getByRole("button", { name: es["alerts.explainCta"] })).toHaveCount(0);
  });

  test("«Explícamelo» abre el modal con la explicación en markdown, el modelo y la advertencia", async ({
    page
  }) => {
    await instalarIpcFalso(page, conIa());
    await page.goto("/alerts");

    const titulo = es[`alert.rule.${detalleAlertaActiva.ruleKey}.title` as keyof typeof es];
    await page.getByRole("button", { name: new RegExp(titulo) }).click();
    await page.getByRole("button", { name: es["alerts.explainCta"] }).click();

    const dialogo = page.getByRole("dialog");
    await expect(dialogo.getByRole("heading", { name: "Qué hacer" })).toBeVisible();
    await expect(dialogo.getByText(explicacionOk.modeloUsado, { exact: false })).toBeVisible();
    await expect(dialogo.getByText(es["ai.modal.disclaimer"])).toBeVisible();

    // No se ha renderizado ningún enlace navegable del contenido del modelo.
    await expect(dialogo.locator("a")).toHaveCount(0);
  });

  test("si la explicación falla, el modal muestra el error y la pantalla de alertas sigue viva", async ({
    page
  }) => {
    await instalarIpcFalso(
      page,
      conIa({
        explicar_detalle_tecnico: {
          __rechazar__: {
            code: "ia.network",
            messageKey: "error.ia.network",
            detail: "reqwest: dns error",
            retryable: true
          }
        }
      })
    );
    await page.goto("/alerts");

    const titulo = es[`alert.rule.${detalleAlertaActiva.ruleKey}.title` as keyof typeof es];
    await page.getByRole("button", { name: new RegExp(titulo) }).click();
    await page.getByRole("button", { name: es["alerts.explainCta"] }).click();

    await expect(page.getByRole("dialog").getByText(es["error.ia.network"])).toBeVisible();
    await page.getByRole("dialog").press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);

    // La lista y el detalle siguen ahí.
    await expect(page.getByRole("heading", { name: titulo })).toBeVisible();
  });

  test("el detalle SMART de un disco también tiene «Explícamelo» (US4)", async ({ page }) => {
    await instalarIpcFalso(page, conIa());
    await page.goto("/disks/disk-0");

    await page.getByRole("button", { name: es["alerts.explainCta"] }).click();
    await expect(page.getByRole("dialog").getByText(es["ai.modal.disclaimer"])).toBeVisible();

    const llamada = (await llamadas(page)).find((l) => l.comando === "explicar_detalle_tecnico");
    expect(llamada?.args).toMatchObject({ origen: { tipo: "smart", deviceId: "disk-0" } });
  });

  test("con la función desactivada, el detalle SMART no muestra el botón", async ({ page }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaDesactivada });
    await page.goto("/disks/disk-0");
    await expect(page.getByRole("button", { name: es["alerts.explainCta"] })).toHaveCount(0);
  });

  test("la primera vez pide confirmar la vista previa del texto que se enviará", async ({ page }) => {
    await instalarIpcFalso(
      page,
      conIa({
        estado_ia: { ...estadoIaActiva, previewAcknowledged: false },
        explicar_detalle_tecnico: {
          estado: "revision",
          textoCompleto: "Regla de alerta activada: smart.wear_high\nValor actual: 92",
          fragmentos: []
        }
      })
    );
    await page.goto("/alerts");

    const titulo = es[`alert.rule.${detalleAlertaActiva.ruleKey}.title` as keyof typeof es];
    await page.getByRole("button", { name: new RegExp(titulo) }).click();
    await page.getByRole("button", { name: es["alerts.explainCta"] }).click();

    await expect(page.getByRole("dialog").getByText("smart.wear_high", { exact: false })).toBeVisible();
    await expect(page.getByRole("button", { name: es["ai.preview.confirm"] })).toBeVisible();
  });
});
