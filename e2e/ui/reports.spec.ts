import { expect, test } from "@playwright/test";
import {
  RESPUESTAS,
  estadoIaActiva,
  vistaPreviaDiagnostico,
  vistaPreviaInformeIa
} from "./fixtures/respuestas";
import { emitirEvento, instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Pantalla de informes y diagnóstico (US-050/051, `docs/product-specification.md` §9):
 *  exportación con destino elegido por diálogo nativo, y vista previa antes de guardar el ZIP. */

test.describe("informes y diagnóstico", () => {
  test("exportar CSV pide el destino al diálogo nativo y llama a export_report", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/reports");

    const tarjetaCsv = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.csv"], exact: true })
    });
    await tarjetaCsv.getByRole("button", { name: es["reports.cta.export"] }).click();

    await expect(page.getByText(es["reports.export.savedAt"].split("{")[0])).toBeVisible();

    const llamada = (await llamadas(page)).find((l) => l.comando === "export_report");
    expect(llamada?.args).toMatchObject({
      format: "csv",
      destinationPath: "C:\\destino\\de\\prueba\\elegido.tmp"
    });
    // CSV no lleva mapa de textos de alerta: es el volcado completo, sin cambios (spec 009).
    expect((llamada?.args as { alertLabels?: unknown }).alertLabels).toBeFalsy();
  });

  test("exportar HTML pasa el mapa de textos de alerta legibles (spec 009)", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/reports");

    const tarjetaHtml = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.html"], exact: true })
    });
    await tarjetaHtml.getByRole("button", { name: es["reports.cta.export"] }).click();
    await expect(page.getByText(es["reports.export.savedAt"].split("{")[0])).toBeVisible();

    const llamada = (await llamadas(page)).find((l) => l.comando === "export_report");
    expect(llamada?.args).toMatchObject({ format: "html" });
    const etiquetas = (llamada?.args as { alertLabels?: Record<string, string> }).alertLabels;
    expect(etiquetas).toBeTruthy();
    // Una regla real ya traducida en es.json aparece con su título legible, no la clave cruda.
    expect(etiquetas?.["smart.wear_high"]).toBe(es["alert.rule.smart.wear_high.title"]);
  });

  test("la casilla «incluir resumen con IA» solo aparece si la ayuda con IA está activada (spec 009)", async ({
    page
  }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaActiva });
    await page.goto("/reports");
    await expect(page.getByRole("switch", { name: es["reports.ai.label"] })).toBeVisible();
  });

  test("sin la ayuda con IA activada, la casilla no aparece", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS); // estado_ia desactivada por defecto
    await page.goto("/reports");
    await expect(page.getByRole("switch", { name: es["reports.ai.label"] })).toHaveCount(0);
  });

  test("con la ayuda con IA activada pero la casilla apagada, exportar HTML no toca la IA (SC-004)", async ({
    page
  }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaActiva });
    await page.goto("/reports");

    // La casilla aparece (IA activa) pero se deja sin marcar: exportación directa, inmediata.
    await expect(page.getByRole("switch", { name: es["reports.ai.label"] })).toBeVisible();
    const tarjetaHtml = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.html"], exact: true })
    });
    await tarjetaHtml.getByRole("button", { name: es["reports.cta.export"] }).click();
    await expect(page.getByText(es["reports.export.savedAt"].split("{")[0])).toBeVisible();

    const registradas = await llamadas(page);
    expect(registradas.find((l) => l.comando === "preview_informe_ia")).toBeFalsy();
    const llamada = registradas.find((l) => l.comando === "export_report");
    expect((llamada?.args as { includeAiSummary?: unknown } | undefined)?.includeAiSummary).toBeFalsy();
  });

  test("marcar el resumen con IA pide la vista previa antes de exportar (spec 009)", async ({ page }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaActiva });
    await page.goto("/reports");

    await page.getByRole("switch", { name: es["reports.ai.label"] }).click();
    const tarjetaHtml = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.html"], exact: true })
    });
    await tarjetaHtml.getByRole("button", { name: es["reports.cta.export"] }).click();

    await expect(page.getByRole("heading", { name: es["reports.ai.preview.title"] })).toBeVisible();
    await expect(page.getByText(vistaPreviaInformeIa.discos[0].deviceLabel)).toBeVisible();
    await expect(page.getByText(vistaPreviaInformeIa.discos[1].deviceLabel)).toBeVisible();

    const registradas = await llamadas(page);
    expect(registradas.find((l) => l.comando === "preview_informe_ia")).toBeTruthy();
    expect(registradas.find((l) => l.comando === "export_report")).toBeFalsy();
  });

  test("confirmar la vista previa exporta con includeAiSummary y previewConfirmada (spec 009)", async ({
    page
  }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, estado_ia: estadoIaActiva });
    await page.goto("/reports");

    await page.getByRole("switch", { name: es["reports.ai.label"] }).click();
    const tarjetaHtml = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.html"], exact: true })
    });
    await tarjetaHtml.getByRole("button", { name: es["reports.cta.export"] }).click();
    await expect(page.getByRole("heading", { name: es["reports.ai.preview.title"] })).toBeVisible();

    await page.getByRole("button", { name: es["reports.ai.preview.confirm"] }).click();
    await expect(page.getByText(es["reports.export.savedAt"].split("{")[0])).toBeVisible();

    const llamada = (await llamadas(page)).find((l) => l.comando === "export_report");
    expect(llamada?.args).toMatchObject({
      format: "html",
      includeAiSummary: true,
      previewConfirmada: true
    });
  });

  test("mientras corre la exportación con IA se ve el progreso y el botón cancelar llama a cancelar_informe (spec 009, US3)", async ({
    page
  }) => {
    // `retardoMs` mantiene `export_report` en vuelo el tiempo justo para poder observar el
    // progreso antes de que resuelva (mismo patrón que `smoke.spec.ts`, «una navegación lenta»).
    await instalarIpcFalso(
      page,
      { ...RESPUESTAS, estado_ia: estadoIaActiva },
      { retardoMs: { export_report: 800 } }
    );
    await page.goto("/reports");

    await page.getByRole("switch", { name: es["reports.ai.label"] }).click();
    const tarjetaHtml = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.html"], exact: true })
    });
    await tarjetaHtml.getByRole("button", { name: es["reports.cta.export"] }).click();
    await expect(page.getByRole("heading", { name: es["reports.ai.preview.title"] })).toBeVisible();

    await page.getByRole("button", { name: es["reports.ai.preview.confirm"] }).click();

    // El progreso llega por `report:progress`, no por la respuesta del comando (ADR-015, sin
    // sondeo) — mismo patrón que `tests.spec.ts`.
    const leyendaProgreso = (done: number, total: number, disco: string) =>
      es["reports.ai.progress.caption"]
        .replace("{done}", String(done))
        .replace("{total}", String(total))
        .replace("{device}", disco);

    await emitirEvento(page, "report:progress", {
      emittedAt: "2026-09-04T10:00:01Z",
      done: 0,
      total: 3,
      deviceLabel: "Disco 1"
    });
    await expect(page.getByText(leyendaProgreso(1, 3, "Disco 1"))).toBeVisible();

    await emitirEvento(page, "report:progress", {
      emittedAt: "2026-09-04T10:00:02Z",
      done: 1,
      total: 3,
      deviceLabel: "Disco 2"
    });
    await expect(page.getByText(leyendaProgreso(2, 3, "Disco 2"))).toBeVisible();

    await page.getByRole("button", { name: es["common.cancel"] }).click();
    const llamadaCancelar = (await llamadas(page)).find((l) => l.comando === "cancelar_informe");
    expect(llamadaCancelar).toBeTruthy();
  });

  test("si la exportación con IA se cancela, la pantalla muestra la nota de cancelación (spec 009, US3)", async ({
    page
  }) => {
    // El doble de IPC no puede simular «cancelar a mitad»: representa el desenlace que produce el
    // backend real cuando `cancelar_informe` corta antes de un disco — `export_report` rechaza con
    // `export.cancelled` y no se escribe fichero.
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      estado_ia: estadoIaActiva,
      export_report: { __rechazar__: { code: "export.cancelled", messageKey: "error.exportCancelled" } }
    });
    await page.goto("/reports");

    await page.getByRole("switch", { name: es["reports.ai.label"] }).click();
    const tarjetaHtml = page.locator("section").filter({
      has: page.getByRole("heading", { name: es["reports.format.html"], exact: true })
    });
    await tarjetaHtml.getByRole("button", { name: es["reports.cta.export"] }).click();
    await expect(page.getByRole("heading", { name: es["reports.ai.preview.title"] })).toBeVisible();

    await page.getByRole("button", { name: es["reports.ai.preview.confirm"] }).click();

    await expect(page.getByText(es["error.exportCancelled"])).toBeVisible();
    await expect(page.getByRole("heading", { name: es["reports.ai.preview.title"] })).toHaveCount(0);
  });

  test("ver contenido del diagnóstico muestra las entradas y los campos sustituidos", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/reports");

    await page.getByRole("button", { name: es["reports.diagnostic.cta.preview"] }).click();

    await expect(page.getByText(vistaPreviaDiagnostico.entries[0].path)).toBeVisible();
    await expect(
      page.getByText(
        `${es["reports.diagnostic.preview.redacted"]} ${es["diagnostic.redacted.serialNumber"]}`,
        {
          exact: false
        }
      )
    ).toBeVisible();
  });

  test("guardar el paquete de diagnóstico llama a create_diagnostic_zip con el destino elegido", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/reports");

    await page.getByRole("button", { name: es["reports.diagnostic.cta.preview"] }).click();
    await expect(page.getByText(vistaPreviaDiagnostico.entries[0].path)).toBeVisible();
    await page.getByRole("button", { name: es["reports.diagnostic.cta.save"] }).click();

    const llamada = (await llamadas(page)).find((l) => l.comando === "create_diagnostic_zip");
    expect(llamada?.args).toMatchObject({
      includeIdentifiers: false,
      destinationPath: "C:\\destino\\de\\prueba\\elegido.tmp"
    });
  });
});
