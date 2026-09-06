import { expect, test } from "@playwright/test";
import { RESPUESTAS, testRunActivo } from "./fixtures/respuestas";
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

  test("confirmar el benchmark llama a start_benchmark con los valores predeterminados", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/tests");

    const tarjetaBenchmark = page.locator("section", { hasText: es["tests.cards.benchmark.title"] });
    await tarjetaBenchmark.getByRole("button", { name: es["tests.cta.configure"] }).click();

    const dialogo = page.getByRole("dialog");
    await dialogo.getByRole("button", { name: es["tests.confirm.benchmark.confirmLabel"] }).click();
    await expect(dialogo).not.toBeVisible();

    const invocacion = (await llamadas(page)).find((l) => l.comando === "start_benchmark");
    expect(invocacion?.args).toMatchObject({
      volumeId: "vol-c",
      sizeBytes: 1_073_741_824,
      blockSizeBytes: 1_048_576,
      mode: "sequential",
      passes: 1
    });
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
