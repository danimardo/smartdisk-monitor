import { expect, test } from "@playwright/test";
import { RESPUESTAS, testRunActivo, testRunBenchmarkTerminado } from "./fixtures/respuestas";
import { emitirEvento, instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Pantalla de pruebas y diagnóstico (US-040/041/042, `docs/product-specification.md` §6):
 *  confirmación previa con comando literal, prueba en curso y cancelación. */

test.describe("pruebas y diagnóstico", () => {
  test("chkdsk muestra el comando literal antes de confirmar y no lo ejecuta si se cancela", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/tests");

    const tarjetaChkdsk = page.locator("section", { hasText: es["tests.cards.chkdsk.title"] });
    await tarjetaChkdsk.getByRole("button", { name: es["tests.cta.configure"] }).click();

    const dialogo = page.getByRole("dialog");
    await expect(dialogo).toBeVisible();
    await expect(dialogo.getByText("chkdsk C: /scan")).toBeVisible();

    await dialogo.getByRole("button", { name: es["common.cancel"] }).click();
    await expect(dialogo).not.toBeVisible();
    expect((await llamadas(page)).map((l) => l.comando)).not.toContain("run_chkdsk_scan");
  });

  test("el benchmark avisa de los GB a escribir y llama a start_benchmark solo con el volumen (ADR-053)", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/tests");

    const tarjetaBenchmark = page.locator("section", { hasText: es["tests.cards.benchmark.title"] });
    await tarjetaBenchmark.getByRole("button", { name: es["tests.cta.configure"] }).click();

    const dialogo = page.getByRole("dialog");
    await expect(dialogo.getByText("GB", { exact: false })).toBeVisible();
    await dialogo.getByRole("button", { name: es["tests.confirm.benchmark.confirmLabel"] }).click();
    await expect(dialogo).not.toBeVisible();

    const invocacion = (await llamadas(page)).find((l) => l.comando === "start_benchmark");
    expect(invocacion?.args).toEqual({ volumeId: "vol-c" });
  });

  test("un benchmark terminado pinta la tabla de Rendimiento con sus 8 filas", async ({ page }) => {
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_test_runs: [testRunBenchmarkTerminado]
    });
    await page.goto("/tests");

    const historial = page.locator("section", { hasText: es["tests.history.title"] });
    await expect(
      historial.getByText(es["tests.benchmark.measuredWith"].replace("{tool} {version}", "DiskSpd 2.3.0"))
    ).toBeVisible();
    await expect(historial.getByRole("columnheader", { name: es["tests.benchmark.col.iops"] })).toBeVisible();
    await expect(
      historial.getByRole("rowheader", { name: es["tests.benchmark.profile.rnd4k_q1"] }).first()
    ).toBeVisible();
  });

  test("SC-005: si start_benchmark falla porque falta la herramienta, chkdsk y autotest siguen disponibles", async ({
    page
  }) => {
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      start_benchmark: {
        __rechazar__: { code: "test.tool_missing", messageKey: "error.testToolMissing", retryable: false }
      }
    });
    await page.goto("/tests");

    const tarjetaBenchmark = page.locator("section", { hasText: es["tests.cards.benchmark.title"] });
    await tarjetaBenchmark.getByRole("button", { name: es["tests.cta.configure"] }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: es["tests.confirm.benchmark.confirmLabel"] })
      .click();

    await expect(page.getByText(es["error.testToolMissing"])).toBeVisible();
    // Las otras dos pruebas siguen ofreciéndose.
    await expect(page.getByRole("heading", { name: es["tests.cards.chkdsk.title"] })).toBeVisible();
    await expect(page.getByRole("heading", { name: es["tests.cards.autotest.title"] })).toBeVisible();
  });

  test("un evento test:progress en curso muestra el progreso y permite cancelar", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/tests");

    // La aplicación ya está arrancada y su suscripción a `test:progress` registrada (`onMount` de
    // esta pantalla): esperar a que pinte algo suyo antes de simular el evento, igual que
    // `rendimiento.spec.ts` con `metrics:updated`.
    await expect(page.getByRole("heading", { name: es["tests.cards.benchmark.title"] })).toBeVisible();

    await emitirEvento(page, "test:progress", {
      emittedAt: "2026-09-04T10:00:01Z",
      testRun: testRunActivo
    });

    // v3: «Prueba en curso» es la píldora de estado del bloque, ya no un encabezado.
    await expect(page.getByText(es["tests.active.title"], { exact: true })).toBeVisible();
    await expect(page.getByText("40 %", { exact: true })).toBeVisible();

    await page.getByRole("button", { name: es["tests.active.cancel"] }).click();
    const invocacion = (await llamadas(page)).find((l) => l.comando === "cancel_test");
    expect(invocacion?.args).toMatchObject({ testRunId: testRunActivo.id });
  });

  test("v3: sin prueba en curso el bloque no se muestra y las tres tarjetas llevan su icono", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/tests");

    await expect(page.getByRole("heading", { name: es["tests.cards.benchmark.title"] })).toBeVisible();
    // Sin `test:progress`, no hay bloque de prueba en curso: su píldora no aparece.
    await expect(page.getByText(es["tests.active.title"], { exact: true })).toHaveCount(0);

    // Cada tarjeta de prueba tiene su cuadrado de icono del sprite en la cabecera.
    const iconos = await page.getByRole("main").locator('svg use[href^="#i-"]').count();
    expect(iconos).toBeGreaterThanOrEqual(3);
  });
});
