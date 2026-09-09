import { expect, test, type ConsoleMessage, type Page } from "@playwright/test";
import { RESPUESTAS, validar } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Smoke mínimo del plano de interfaz (`docs/testing-strategy.md` §10).
 *
 *  No comprueba reglas de negocio: comprueba que la aplicación **existe**. Si esto falla, ninguna
 *  otra prueba de este plano significa nada.
 */

/** Avisos tolerados. La lista tiene que ser mínima y vivir aquí, junto a la prueba que la aplica,
 *  nunca dispersa por los ficheros (§10). Cada entrada explica por qué se tolera. */
const AVISOS_TOLERADOS: { patron: RegExp; motivo: string }[] = [
  {
    patron: /Vite|\[vite\]|HMR/i,
    motivo: "Ruido del servidor de vista previa, no de la aplicación."
  }
];

function esFalloReal(mensaje: ConsoleMessage): boolean {
  if (mensaje.type() !== "error") return false;
  return !AVISOS_TOLERADOS.some((a) => a.patron.test(mensaje.text()));
}

/** Engancha la vigilancia de consola y devuelve el acumulador. Se llama antes de navegar: un error
 *  durante la carga inicial es justo el que más importa y el que más fácil se pierde. */
function vigilarConsola(page: Page) {
  const fallos: string[] = [];
  page.on("console", (m) => {
    if (esFalloReal(m)) fallos.push(`console.error: ${m.text()}`);
  });
  page.on("pageerror", (e) => fallos.push(`excepción no controlada: ${e.message}`));
  return fallos;
}

test.beforeAll(() => {
  // Si los fixtures dejan de cumplir el contrato, se falla aquí y no dentro de una pantalla.
  validar();
});

test.describe("smoke @smoke", () => {
  test("arranca, pinta el chrome y no deja la ventana en blanco", async ({ page }) => {
    const fallos = vigilarConsola(page);
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    // Barra lateral, barra de herramientas y región de contenido, por rol y por nombre accesible,
    // nunca por clase (§12). El nombre se toma del diccionario y no de un literal: si alguien
    // renombra la sección, la prueba se entera en vez de fallar por una cadena desincronizada.
    await expect(page.getByRole("navigation", { name: es["nav.monitoring"] })).toBeVisible();
    await expect(page.getByRole("main")).toBeVisible();
    await expect(page.getByRole("main")).not.toBeEmpty();

    // Hay dos landmarks de navegación —secciones y discos— y ambos deben estar etiquetados: dos
    // `<nav>` sin nombre son indistinguibles para un lector de pantalla.
    const navegaciones = page.getByRole("navigation");
    expect(await navegaciones.count()).toBeGreaterThanOrEqual(1);
    for (const nav of await navegaciones.all()) {
      await expect(nav).toHaveAttribute("aria-label", /.+/);
    }

    expect(fallos, "errores de consola durante el arranque").toEqual([]);
  });

  test("navega por las seis secciones y cada una pinta algo", async ({ page }) => {
    const fallos = vigilarConsola(page);
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");

    const secciones = ["/alerts", "/events", "/tests", "/reports", "/settings", "/"];
    for (const destino of secciones) {
      // Enlaces reales, no botones que empujen la ruta (constitución §XIV).
      const enlace = page.getByRole("link").filter({ has: page.locator(`[href="${destino}"]`) });
      const objetivo = (await enlace.count()) ? enlace.first() : page.locator(`a[href="${destino}"]`).first();
      await objetivo.click();
      await expect(page).toHaveURL(new RegExp(`${destino === "/" ? "/$" : destino}`));
      await expect(page.getByRole("main")).not.toBeEmpty();
    }

    expect(fallos, "errores de consola durante la navegación").toEqual([]);
  });

  test("una navegación lenta muestra la barra de progreso, no una interfaz congelada (spec 004)", async ({
    page
  }) => {
    // El `load` de `/events` tarda: antes esto congelaba la aplicación sin ninguna señal.
    await instalarIpcFalso(page, RESPUESTAS, { retardoMs: { get_system_events: 600 } });
    await page.goto("/");

    // Por nombre accesible: la `CapacityBar` de cada disco también es un `progressbar`; la barra de
    // navegación es la que se anuncia como "Cargando…".
    const barra = page.getByRole("progressbar", { name: es["common.loading"] });
    await page.locator('a[href="/events"]').first().click();
    await expect(barra).toBeVisible();
    // Y desaparece al terminar.
    await expect(page).toHaveURL(/\/events/);
    await expect(barra).toHaveCount(0);
  });

  test("«Refrescar» no congela: el botón muestra carga y aparece la barra superior", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS, { retardoMs: { refresh_now: 600 } });
    await page.goto("/");

    const boton = page.getByRole("button", { name: es["common.refresh"] });
    const barra = page.getByRole("progressbar", { name: es["common.loading"] });
    await boton.click();

    // Mientras trabaja: botón deshabilitado + barra visible, y la navegación sigue respondiendo.
    await expect(boton).toBeDisabled();
    await expect(barra).toBeVisible();
    await page.locator('a[href="/alerts"]').first().click();
    await expect(page).toHaveURL(/\/alerts/);

    // Al terminar: vuelve a la normalidad.
    await expect(boton).toBeEnabled();
    await expect(barra).toHaveCount(0);
  });

  test("aplica el tema y resuelve el material", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");
    await expect(page.getByRole("main")).toBeVisible();

    const tema = await page.evaluate(() => document.documentElement.getAttribute("data-theme"));
    expect(tema, "el layout raíz no ha fijado data-theme").toMatch(/^(light|dark)$/);

    const fondo = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue("--sdm-bg").trim()
    );
    expect(fondo, "los tokens no se han cargado").not.toBe("");
  });

  test("la tipografía empotrada carga y no cae al respaldo del sistema", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");
    await expect(page.getByRole("main")).toBeVisible();

    const cargada = await page.evaluate(async () => {
      await document.fonts.ready;
      return [...document.fonts].some((f) => f.family.includes("Instrument") && f.status === "loaded");
    });
    expect(
      cargada,
      "Instrument Sans no ha cargado: la aplicación está pintando con la fuente del sistema"
    ).toBe(true);
  });

  test("consulta el backend al arrancar, en vez de pintar datos inventados", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/");
    await expect(page.getByRole("main")).toBeVisible();

    const comandos = (await llamadas(page)).map((l) => l.comando);
    expect(comandos).toContain("get_devices");
    expect(comandos).toContain("get_appearance_settings");
  });
});
