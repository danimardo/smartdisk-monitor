import { expect, test } from "@playwright/test";
import { detalleDisco0, eventos, RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Detalle de disco (US-012, US-020): cabecera, métricas, gráfica de temperatura y contadores. */

test.describe("detalle de disco", () => {
  test("carga el detalle real y pinta las gráficas de temperatura y actividad para 24h", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);

    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();
    // Aparece dos veces a propósito: la tarjeta de métrica y la fila del contador comparten
    // etiqueta (`ETIQUETA_REUTILIZADA` en la pantalla), así que se comprueba que haya al menos una.
    await expect(page.getByText(es["disk.temperature"]).first()).toBeVisible();
    await expect(page.getByText(es["smart.counter.power_cycles"])).toBeVisible();

    // Cada gráfica en su propio panel con encabezado (unidad incluida) y su `role="img"` distinto.
    await expect(page.getByRole("heading", { name: `${es["disk.temperature"]} · °C` })).toBeVisible();
    await expect(page.getByRole("heading", { name: `${es["disk.activity"]} · %` })).toBeVisible();
    await expect(page.getByRole("img", { name: new RegExp(`^${es["disk.temperature"]}`) })).toBeVisible();
    await expect(page.getByRole("img", { name: new RegExp(`^${es["disk.activity"]}`) })).toBeVisible();

    const series = (await llamadas(page))
      .filter((l) => l.comando === "get_metric_series")
      .map((l) => (l.args as { metricKey?: string }).metricKey);
    expect(series).toContain("temperature_celsius");
    expect(series).toContain("activity_percent");
  });

  test("cambiar a 7 días vuelve a pedir las dos series con el intervalo nuevo", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();
    await expect(page.getByRole("img", { name: new RegExp(`^${es["disk.activity"]}`) })).toBeVisible();

    const seriesAntes = (await llamadas(page)).filter((l) => l.comando === "get_metric_series").length;

    await page.getByRole("radio", { name: es["range.7d"] }).click();
    await expect(page.getByRole("radio", { name: es["range.7d"] })).toHaveAttribute("aria-checked", "true");

    // El selector es único y gobierna ambas gráficas: un cambio de rango dispara al menos dos
    // peticiones nuevas (temperatura y actividad).
    await expect
      .poll(async () => (await llamadas(page)).filter((l) => l.comando === "get_metric_series").length)
      .toBeGreaterThanOrEqual(seriesAntes + 2);
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

  test("el detalle de disco ofrece el rango de 1 hora y al elegirlo vuelve a pedir las series", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto(`/disks/${detalleDisco0.id}`);
    await expect(page.getByRole("img", { name: new RegExp(`^${es["disk.activity"]}`) })).toBeVisible();

    const antes = (await llamadas(page)).filter((l) => l.comando === "get_metric_series").length;
    await page.getByRole("radio", { name: es["range.1h"] }).click();
    await expect(page.getByRole("radio", { name: es["range.1h"] })).toHaveAttribute("aria-checked", "true");
    await expect
      .poll(async () => (await llamadas(page)).filter((l) => l.comando === "get_metric_series").length)
      .toBeGreaterThanOrEqual(antes + 2);
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

  test.describe("eventos de este disco (spec 012)", () => {
    test("muestra los eventos recientes del disco y etiqueta la asociación inferida", async ({ page }) => {
      await instalarIpcFalso(page, RESPUESTAS);
      await page.goto(`/disks/${detalleDisco0.id}`);

      const seccion = page.locator("section", { hasText: es["disk.events.title"] });
      await expect(seccion.getByText(eventos[0].message)).toBeVisible();
      await expect(seccion.getByText(eventos[1].message)).toBeVisible();
      await expect(seccion.getByText(es["events.inferredMapping"])).toBeVisible();
    });

    test("un disco sin eventos asociados muestra el estado vacío de la sección, sin ocultarla", async ({
      page
    }) => {
      await instalarIpcFalso(page, {
        ...RESPUESTAS,
        get_system_events: { events: [], nextCursor: null, total: 0 }
      });
      await page.goto(`/disks/${detalleDisco0.id}`);

      const seccion = page.locator("section", { hasText: es["disk.events.title"] });
      await expect(seccion.getByText(es["disk.events.empty"])).toBeVisible();
    });

    test("un fallo al consultar los eventos degrada solo esa sección, el resto del detalle sigue", async ({
      page
    }) => {
      await instalarIpcFalso(page, {
        ...RESPUESTAS,
        get_system_events: {
          __rechazar__: { code: "events.query_failed", messageKey: "error.unexpected", retryable: true }
        }
      });
      await page.goto(`/disks/${detalleDisco0.id}`);

      await expect(page.getByRole("heading", { name: detalleDisco0.model })).toBeVisible();
      await expect(page.getByText(es["disk.temperature"]).first()).toBeVisible();
      const seccion = page.locator("section", { hasText: es["disk.events.title"] });
      await expect(seccion.getByText(es["error.unexpected"])).toBeVisible();
    });

    test("pulsar un evento de la sección lleva a su detalle en la pantalla de Eventos", async ({ page }) => {
      await instalarIpcFalso(page, RESPUESTAS);
      await page.goto(`/disks/${detalleDisco0.id}`);

      // El primer suceso del fixture (`eventos[0]`, id "1") es un enlace, no un botón.
      const fila = page.getByRole("link", { name: /Volumen C: es correcto/ });
      await expect(fila).toHaveAttribute("href", "/events?focus=1");
      await fila.click();
      await expect(page).toHaveURL(/\/events\?focus=1/);
      await expect(page.getByRole("heading", { name: es["events.detail.title"] })).toBeVisible();
    });

    test('"Ver todos" lleva a Eventos con el filtro de este disco ya aplicado', async ({ page }) => {
      await instalarIpcFalso(page, RESPUESTAS);
      await page.goto(`/disks/${detalleDisco0.id}`);

      const seccion = page.locator("section", { hasText: es["disk.events.title"] });
      const enlace = seccion.getByRole("link", { name: es["common.viewAll"] });
      await expect(enlace).toHaveAttribute("href", `/events?deviceId=${detalleDisco0.id}`);
      await enlace.click();
      await expect(page).toHaveURL(`/events?deviceId=${detalleDisco0.id}`);
    });
  });
});
