import { expect, test } from "@playwright/test";
import { apariencia, RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Riel de navegación expandible (spec 013): el panel de nombres se superpone al contenido, nunca
 *  lo empuja (ADR-034, mínimo de ventana 1024×560 ya protegido) — verificado en la ventana mínima,
 *  no solo en un tamaño cómodo. También cubre la persistencia entre sesiones (US4). */

test.describe("riel expandible", () => {
  test("expandir el riel en la ventana mínima no cambia el ancho del contenido ni desborda", async ({
    page
  }) => {
    await page.setViewportSize({ width: 1024, height: 560 });
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    const main = page.getByRole("main");
    await expect(main).toBeVisible();
    const anchoAntes = await main.evaluate((el) => el.getBoundingClientRect().width);

    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    await expect(page.getByRole("dialog")).toBeVisible();

    const anchoDespues = await main.evaluate((el) => el.getBoundingClientRect().width);
    expect(anchoDespues).toBe(anchoAntes);

    const desbordado = await page.evaluate(() => {
      const doc = document.documentElement;
      return doc.scrollWidth > doc.clientWidth + 1;
    });
    expect(desbordado, "expandir el riel no debe producir scroll horizontal nuevo").toBe(false);
  });

  test("el panel expandido muestra el nombre de cada sección y se pliega al elegir una", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    const panel = page.getByRole("dialog");
    await expect(panel.getByRole("link", { name: es["nav.events"] })).toBeVisible();

    await panel.getByRole("link", { name: es["nav.events"] }).click();
    await expect(page).toHaveURL(/\/events/);
    // Se pliega solo al elegir una sección (corrección post-validación, D6 de research.md):
    // dejarlo abierto tapaba la pantalla de destino hasta un segundo gesto.
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByRole("button", { name: es["nav.sidebar.expand"] })).toBeVisible();
  });

  test("elegir una sección no persiste el plegado: es solo una consecuencia de navegar", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    // Expandir sí persiste (value: true) — es el gesto explícito. Lo que no debe pasar es una
    // segunda llamada al elegir una sección.
    await page.getByRole("button", { name: es["nav.sidebar.expand"] }).click();
    const llamadasAntes = (
      await page.evaluate(
        () => (window as unknown as { __llamadas__: { comando: string; args: unknown }[] }).__llamadas__
      )
    ).filter(
      (l) =>
        l.comando === "set_setting" &&
        (l.args as { key?: string }).key === "settings.appearance.sidebar_expanded"
    ).length;

    await page.getByRole("dialog").getByRole("link", { name: es["nav.events"] }).click();
    await expect(page).toHaveURL(/\/events/);

    const llamadasDespues = (
      await page.evaluate(
        () => (window as unknown as { __llamadas__: { comando: string; args: unknown }[] }).__llamadas__
      )
    ).filter(
      (l) =>
        l.comando === "set_setting" &&
        (l.args as { key?: string }).key === "settings.appearance.sidebar_expanded"
    ).length;

    expect(llamadasDespues, "elegir una sección no debe añadir una llamada nueva").toBe(llamadasAntes);
  });

  test("Escape cierra el panel y devuelve el foco al botón", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    const boton = page.getByRole("button", { name: es["nav.sidebar.expand"] });
    await boton.click();
    await expect(page.getByRole("dialog")).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByRole("button", { name: es["nav.sidebar.expand"] })).toBeFocused();
  });

  test("la preferencia se recuerda: arranca expandido si el ajuste ya lo dice, y alternar la persiste", async ({
    page
  }) => {
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_appearance_settings: { ...apariencia, sidebarExpanded: true }
    });
    await page.goto("/");

    // Arranca expandido sin pulsar nada.
    await expect(page.getByRole("dialog")).toBeVisible();

    await page.getByRole("button", { name: es["nav.sidebar.collapse"] }).click();
    const comandos = await page.evaluate(
      () => (window as unknown as { __llamadas__: { comando: string; args: unknown }[] }).__llamadas__
    );
    const llamada = comandos.find(
      (l) =>
        l.comando === "set_setting" &&
        (l.args as { key?: string }).key === "settings.appearance.sidebar_expanded"
    );
    expect(llamada).toBeTruthy();
    expect((llamada?.args as { value?: unknown })?.value).toBe(false);
  });
});
