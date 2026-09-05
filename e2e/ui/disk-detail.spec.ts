import { expect, test } from "@playwright/test";
import { detalleDisco0, RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Detalle de disco (US-012, US-020): cabecera, métricas, gráfica de temperatura y contadores. */

test.describe("detalle de disco", () => {
  test("carga el detalle real y pide la serie de temperatura para 24h", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);

    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();
    // Aparece dos veces a propósito: la tarjeta de métrica y la fila del contador comparten
    // etiqueta (`ETIQUETA_REUTILIZADA` en la pantalla), así que se comprueba que haya al menos una.
    await expect(page.getByText(es["disk.temperature"]).first()).toBeVisible();
    await expect(page.getByText(es["smart.counter.power_cycles"])).toBeVisible();

    const comandos = (await llamadas(page)).map((l) => l.comando);
    expect(comandos).toContain("get_device_detail");
    expect(comandos).toContain("get_metric_series");
  });

  test("cambiar a 7 días vuelve a pedir la serie", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();

    await page.getByRole("radio", { name: es["range.7d"] }).click();
    await expect(page.getByRole("radio", { name: es["range.7d"] })).toHaveAttribute("aria-checked", "true");

    const llamadasSerie = (await llamadas(page)).filter((l) => l.comando === "get_metric_series");
    expect(llamadasSerie.length).toBeGreaterThanOrEqual(2);
  });

  test("el rango personalizado muestra el selector de fechas", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();

    await page.getByRole("radio", { name: es["range.custom"] }).click();
    await expect(page.getByLabel(es["dateRange.from"])).toBeVisible();
    await expect(page.getByLabel(es["dateRange.to"])).toBeVisible();
  });
});
