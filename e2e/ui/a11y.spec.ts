import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";

/** Accesibilidad automática sobre cada pantalla, **en ambos temas**
 *  (`docs/testing-strategy.md` §16, constitución §VII).
 *
 *  Una pasada de `axe` no es una auditoría: detecta una parte de los incumplimientos, no todos.
 *  Antes de la 1.0 hay una pasada manual con lector de pantalla, que sigue siendo tarea aparte.
 *
 *  Lo que `axe` **no** sabe hacer y aquí importa —contraste sobre material compuesto, foco no
 *  tapado por el chrome translúcido— se comprueba en las pruebas de componente en navegador, donde
 *  hay acceso a `getComputedStyle` del elemento concreto.
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

async function fijarTema(page: Page, tema: "light" | "dark") {
  await page.evaluate((t) => document.documentElement.setAttribute("data-theme", t), tema);
}

test.describe("accesibilidad @a11y", () => {
  for (const ruta of PANTALLAS) {
    for (const tema of ["light", "dark"] as const) {
      test(`${ruta} no tiene incumplimientos de axe en tema ${tema}`, async ({ page }) => {
        await instalarIpcFalso(page, RESPUESTAS);
        await page.goto(ruta);
        await expect(page.getByRole("main")).toBeVisible();
        await fijarTema(page, tema);

        const resultado = await new AxeBuilder({ page })
          .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
          .analyze();

        // El mensaje lista regla e impacto: un identificador de axe a secas no dice qué arreglar.
        const resumen = resultado.violations.map(
          (v) => `${v.id} (${v.impact}): ${v.help} — ${v.nodes.length} nodo(s)`
        );
        expect(resumen, `incumplimientos en ${ruta} con tema ${tema}`).toEqual([]);
      });
    }
  }
});
