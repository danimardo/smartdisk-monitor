import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, test, type Page } from "@playwright/test";
import { RESPUESTAS, apariencia, inventario, testRunActivo } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Genera el entregable de rediseño: capturas PNG y snapshots HTML autocontenidos de todas las
 *  secciones y estados, para pasárselos a un diseñador (`design/entregable-rediseno/`).
 *
 *  **No es una prueba de regresión.** No afirma nada sobre el comportamiento: solo produce
 *  artefactos. Por eso está desactivada salvo que se pida explícitamente:
 *
 *      SDM_CAPTURAS=1 pnpm exec playwright test capturas
 *
 *  Reutiliza el IPC falso y los fixtures del plano de interfaz, así que las pantallas salen con
 *  datos realistas y ya validados contra los esquemas Zod reales.
 */

const ACTIVO = process.env.SDM_CAPTURAS === "1";

const SALIDA = fileURLToPath(new URL("../../design/entregable-rediseno/salida/", import.meta.url));
const DIR_PNG = `${SALIDA}capturas/`;
const DIR_HTML = `${SALIDA}html/`;

const FUENTE_LATIN = readFileSync(
  new URL("../../src/design-system/fonts/InstrumentSans-latin.woff2", import.meta.url)
).toString("base64");
const FUENTE_EXT = readFileSync(
  new URL("../../src/design-system/fonts/InstrumentSans-latin-ext.woff2", import.meta.url)
).toString("base64");

type Tema = "claro" | "oscuro";

const TEMAS: Tema[] = ["claro", "oscuro"];

const VIEWPORTS = [
  { slug: "1280x800", width: 1280, height: 800, completa: true, ventana: false },
  { slug: "1024x560-minimo", width: 1024, height: 560, completa: true, ventana: true }
] as const;

const SECCIONES = [
  { slug: "panel-general", ruta: "/" },
  { slug: "alertas", ruta: "/alerts" },
  { slug: "eventos", ruta: "/events" },
  { slug: "pruebas", ruta: "/tests" },
  { slug: "informes", ruta: "/reports" },
  { slug: "ajustes", ruta: "/settings" },
  { slug: "detalle-disco", ruta: "/disks/disk-0" },
  { slug: "configuracion-inicial", ruta: "/onboarding" }
] as const;

/** Tabla de respuestas del IPC para un tema. El tema efectivo lo decide la app a partir de
 *  `get_appearance_settings`, así que basta con cambiar ese valor. */
function tabla(tema: Tema, extra: Record<string, unknown> = {}): Record<string, unknown> {
  const base: Record<string, unknown> =
    tema === "claro"
      ? RESPUESTAS
      : { ...RESPUESTAS, get_appearance_settings: { ...apariencia, theme: "dark" } };
  return { ...base, ...extra };
}

/** Espera a que la pantalla esté pintada de verdad: región principal visible y con contenido, y
 *  la tipografía empotrada resuelta (sin esto, la captura sale con la fuente del sistema). */
async function esperarListo(page: Page): Promise<void> {
  await expect(page.getByRole("main")).toBeVisible();
  await expect(page.getByRole("main")).not.toBeEmpty();
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
}

async function png(page: Page, nombre: string, fullPage: boolean): Promise<void> {
  mkdirSync(DIR_PNG, { recursive: true });
  await page.screenshot({ path: `${DIR_PNG}${nombre}.png`, fullPage, animations: "disabled" });
}

/** Snapshot HTML autocontenido: DOM actual + todo el CSS de la página embebido + las dos `woff2`
 *  en base64. Se abre en cualquier navegador sin servidor y reproduce el render real. No lleva
 *  scripts: es una referencia visual, no la aplicación. */
async function html(page: Page, nombre: string): Promise<void> {
  mkdirSync(DIR_HTML, { recursive: true });

  const capturado = await page.evaluate(() => {
    const css = [...document.styleSheets]
      .map((hoja) => {
        try {
          return [...hoja.cssRules].map((regla) => regla.cssText).join("\n");
        } catch {
          return "";
        }
      })
      .join("\n");
    return {
      css,
      cuerpo: document.body.innerHTML,
      tema: document.documentElement.getAttribute("data-theme") ?? "",
      estiloRaiz: document.documentElement.getAttribute("style") ?? "",
      lang: document.documentElement.getAttribute("lang") ?? "es"
    };
  });

  // Las `@font-face` originales apuntan a rutas con hash que no existen fuera del servidor:
  // se sustituyen las URLs de las `woff2` por su versión en base64.
  const cssConFuentes = capturado.css
    .replace(
      /url\([^)]*InstrumentSans-latin\.[^)]*\.woff2[^)]*\)/g,
      `url(data:font/woff2;base64,${FUENTE_LATIN})`
    )
    .replace(
      /url\([^)]*InstrumentSans-latin-ext\.[^)]*\.woff2[^)]*\)/g,
      `url(data:font/woff2;base64,${FUENTE_EXT})`
    );

  const respaldoFuentes = [
    '@font-face{font-family:"Instrument Sans";font-weight:400 600;font-style:normal;font-display:swap;',
    `src:url(data:font/woff2;base64,${FUENTE_LATIN}) format("woff2")}`,
    '@font-face{font-family:"Instrument Sans";font-weight:400 600;font-style:normal;font-display:swap;',
    `src:url(data:font/woff2;base64,${FUENTE_EXT}) format("woff2")}`
  ].join("");

  const estiloRaiz = capturado.estiloRaiz ? ` style="${capturado.estiloRaiz.replace(/"/g, "&quot;")}"` : "";
  const doc = [
    "<!doctype html>",
    `<html lang="${capturado.lang}" data-theme="${capturado.tema}"${estiloRaiz}>`,
    "<head>",
    '<meta charset="utf-8">',
    `<title>SmartDisk Monitor — ${nombre}</title>`,
    `<style>${respaldoFuentes}\n${cssConFuentes}</style>`,
    "</head>",
    `<body>${capturado.cuerpo}</body>`,
    "</html>"
  ].join("\n");

  writeFileSync(`${DIR_HTML}${nombre}.html`, doc, "utf8");
}

test.describe("entregable de rediseño @capturas", () => {
  test.skip(!ACTIVO, "Genera artefactos, no comprueba nada. Actívala con SDM_CAPTURAS=1.");

  test.describe.configure({ retries: 0 });

  for (const seccion of SECCIONES) {
    for (const tema of TEMAS) {
      test(`sección · ${seccion.slug} · ${tema}`, async ({ page }) => {
        await instalarIpcFalso(page, tabla(tema));

        for (const vp of VIEWPORTS) {
          await page.setViewportSize({ width: vp.width, height: vp.height });
          await page.goto(seccion.ruta);
          await esperarListo(page);

          const base = `${seccion.slug}__${tema}__${vp.slug}`;
          if (vp.completa) await png(page, `${base}__completa`, true);
          if (vp.ventana) await png(page, `${base}__ventana`, false);
        }

        // Snapshot HTML solo a 1280×800: el CSS embebido incluye todas las media queries, así que
        // un único fichero reacomoda al cambiar el ancho del navegador.
        await page.setViewportSize({ width: 1280, height: 800 });
        await page.goto(seccion.ruta);
        await esperarListo(page);
        await html(page, `${seccion.slug}__${tema}`);
      });
    }
  }

  /* ---------------------------------------------------------------------------- estados */

  const inventarioVacio = { ...inventario, devices: [], excluded: [] };

  for (const tema of TEMAS) {
    test(`estado · panel sin discos · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema, { get_devices: inventarioVacio }));
      await page.goto("/");
      await esperarListo(page);
      await png(page, `estado-panel-vacio__${tema}`, true);
      await html(page, `estado-panel-vacio__${tema}`);
    });

    test(`estado · sin alertas · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema, { get_alert_groups: [] }));
      await page.goto("/alerts");
      await esperarListo(page);
      await png(page, `estado-alertas-vacio__${tema}`, true);
      await html(page, `estado-alertas-vacio__${tema}`);
    });

    test(`estado · sin eventos · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(
        page,
        tabla(tema, { get_system_events: { events: [], nextCursor: null, total: 0 } })
      );
      await page.goto("/events");
      await esperarListo(page);
      await png(page, `estado-eventos-vacio__${tema}`, true);
      await html(page, `estado-eventos-vacio__${tema}`);
    });

    test(`estado · error de pantalla · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema, { get_devices: { invalido: true } }));
      await page.goto("/");
      await expect(page.getByRole("main")).toBeVisible();
      await png(page, `estado-error-pantalla__${tema}`, true);
      await html(page, `estado-error-pantalla__${tema}`);
    });

    test(`estado · diálogo Acerca de · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema));
      await page.goto("/");
      await esperarListo(page);
      await page.getByRole("button", { name: es["nav.about"] }).click();
      await expect(page.getByRole("dialog")).toBeVisible();
      // La foto del autor (ADR-052) tiene que haber pintado antes de la captura.
      await expect
        .poll(async () =>
          page
            .getByRole("img", { name: es["about.photoAlt"] })
            .evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0)
        )
        .toBe(true);
      await png(page, `estado-dialogo-acerca-de__${tema}`, false);
      await html(page, `estado-dialogo-acerca-de__${tema}`);
    });

    test(`estado · diálogo destructivo · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema));
      await page.goto("/alerts");
      await esperarListo(page);
      await page.getByRole("button", { name: es["alerts.actions.archive"] }).click();
      await expect(page.getByRole("dialog")).toBeVisible();
      await png(page, `estado-dialogo-destructivo__${tema}`, false);
      await html(page, `estado-dialogo-destructivo__${tema}`);
    });

    test(`estado · diálogo de prueba · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema));
      await page.goto("/tests");
      await esperarListo(page);
      await page.getByRole("button", { name: es["tests.cta.configure"] }).first().click();
      await expect(page.getByRole("dialog")).toBeVisible();
      await png(page, `estado-dialogo-prueba__${tema}`, false);
      await html(page, `estado-dialogo-prueba__${tema}`);
    });

    test(`estado · prueba en curso · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema, { get_test_runs: [testRunActivo] }));
      await page.goto("/tests");
      await esperarListo(page);
      await png(page, `estado-prueba-en-curso__${tema}`, true);
      await html(page, `estado-prueba-en-curso__${tema}`);
    });

    test(`estado · detalle de evento con XML · ${tema}`, async ({ page }) => {
      await instalarIpcFalso(page, tabla(tema));
      await page.goto("/events");
      await esperarListo(page);
      await page
        .getByRole("button", { name: /Microsoft-Windows-Ntfs/ })
        .first()
        .click();
      await png(page, `estado-evento-detalle__${tema}`, true);
      await html(page, `estado-evento-detalle__${tema}`);
    });
  }
});
