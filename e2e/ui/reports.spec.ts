import { expect, test } from "@playwright/test";
import { RESPUESTAS, vistaPreviaDiagnostico } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
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
