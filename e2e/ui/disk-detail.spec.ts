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

  test("v3: las cuatro métricas llevan icono y el control de intervalo vive en el cuerpo, no en la barra", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();

    // Las cuatro etiquetas de MetricCard (algunas se repiten en los contadores: basta la primera).
    for (const clave of ["disk.temperature", "disk.activity", "disk.wear", "disk.powerOnHours"] as const) {
      await expect(page.getByText(es[clave]).first()).toBeVisible();
    }

    // El SegmentedControl de intervalo está dentro de <main>, no en la <Toolbar> (que perdió su ranura).
    const toolbar = page.getByRole("banner");
    await expect(toolbar.getByRole("radio", { name: es["range.24h"] })).toHaveCount(0);
    await expect(page.getByRole("main").getByRole("radio", { name: es["range.24h"] })).toBeVisible();

    // Iconos del sprite presentes en el cuerpo (cabecera de identidad + tarjetas de métrica).
    const iconos = await page.getByRole("main").locator('svg use[href^="#i-"]').count();
    expect(iconos).toBeGreaterThanOrEqual(4);
  });

  test("el rango personalizado muestra el selector de fechas", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();

    await page.getByRole("radio", { name: es["range.custom"] }).click();
    await expect(page.getByLabel(es["dateRange.from"])).toBeVisible();
    await expect(page.getByLabel(es["dateRange.to"])).toBeVisible();
  });

  test("un disco SATA ilegible bloqueado por Defender ofrece reintentar la excepción (J.56)", async ({
    page
  }) => {
    const discoBloqueado = { ...detalleDisco0, deviceType: "sata_ssd", unknownReason: "unreadable" };
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_device_detail: discoBloqueado,
      check_smartctl_defender_exception: false,
      add_smartctl_defender_exception: { added: true, detail: null }
    });
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();

    const boton = page.getByRole("button", { name: es["disk.retryFolderProtection"] });
    await expect(boton).toBeVisible();
    await boton.click();
    await expect(page.getByText(es["disk.retryFolderProtectionSuccess"])).toBeVisible();

    const comandos = (await llamadas(page)).map((l) => l.comando);
    expect(comandos).toContain("check_smartctl_defender_exception");
    expect(comandos).toContain("add_smartctl_defender_exception");
  });

  test("un disco NVMe ilegible nunca ofrece la excepción de Defender: no la necesita", async ({ page }) => {
    const discoNvme = { ...detalleDisco0, deviceType: "nvme", unknownReason: "unreadable" };
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_device_detail: discoNvme,
      check_smartctl_defender_exception: false
    });
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();

    await expect(page.getByRole("button", { name: es["disk.retryFolderProtection"] })).toHaveCount(0);
    const comandos = (await llamadas(page)).map((l) => l.comando);
    expect(comandos).not.toContain("check_smartctl_defender_exception");
  });
});
