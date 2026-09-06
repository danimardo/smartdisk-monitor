import { expect, test, type Page } from "@playwright/test";
import { RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";

/** T111 (`docs/ui-design.md` §8, §4.0): "la ventana nunca recorta contenido en silencio", medido
 *  a la ventana mínima 1024×560 y al espacio CSS que de verdad queda tras el escalado de Windows.
 *
 *  El escalado de Windows no encoge el texto: encoge el espacio disponible en píxeles CSS
 *  (`docs/ui-design.md` §4.0, medido en `open-questions.md` K.4). Una ventana de 1024×560 físicos
 *  al 125 %/150 %/200 % dispone de 1024/560 divididos por ese factor en píxeles CSS reales — por
 *  eso el viewport de Playwright se fija a esos valores reducidos, no a 1024×560 con el zoom del
 *  navegador (que sí escala el texto, al revés de lo que hace Windows).
 *
 *  Comprobación automática: sin desbordamiento horizontal del documento (un recorte silencioso se
 *  delata como scroll horizontal que nadie pidió). Las capturas quedan en `test-results/escalado/`
 *  para la revisión visual humana que ningún assert sustituye.
 */

const PANTALLAS = [
  "/",
  "/alerts",
  "/disks/disk-0",
  "/events",
  "/tests",
  "/reports",
  "/settings",
  "/onboarding"
];

const TAMANOS: { nombre: string; width: number; height: number }[] = [
  { nombre: "1024x560-minimo-tecnico", width: 1024, height: 560 },
  { nombre: "1280x720-objetivo-diseno", width: 1280, height: 720 },
  { nombre: "819x448-125pct-sobre-1024x560", width: 819, height: 448 },
  { nombre: "683x373-150pct-sobre-1024x560", width: 683, height: 373 },
  { nombre: "512x280-200pct-sobre-1024x560", width: 512, height: 280 }
];

async function sinDesbordamientoHorizontal(page: Page) {
  const desbordado = await page.evaluate(() => {
    const doc = document.documentElement;
    return doc.scrollWidth > doc.clientWidth + 1;
  });
  return desbordado;
}

test.describe("escalado y ventana mínima", () => {
  for (const ruta of PANTALLAS) {
    for (const tamano of TAMANOS) {
      test(`${ruta} a ${tamano.nombre} no recorta en silencio`, async ({ page }) => {
        await page.setViewportSize({ width: tamano.width, height: tamano.height });
        await instalarIpcFalso(page, RESPUESTAS);
        await page.goto(ruta);
        await expect(page.getByRole("main")).toBeVisible();

        const nombreArchivo = `${ruta.replace(/\//g, "_") || "_raiz"}__${tamano.nombre}.png`;
        await page.screenshot({
          path: `test-results/escalado/${nombreArchivo}`,
          fullPage: false
        });

        const desbordado = await sinDesbordamientoHorizontal(page);
        expect(
          desbordado,
          `${ruta} a ${tamano.width}×${tamano.height} desborda horizontalmente: contenido recortado en silencio`
        ).toBe(false);
      });
    }
  }
});
