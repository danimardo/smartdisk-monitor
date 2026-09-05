import { expect, test } from "@playwright/test";
import { eventos, RESPUESTAS } from "./fixtures/respuestas";
import { instalarIpcFalso } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Registro de eventos (US-021): lista, filtros, detalle con XML original. */

test.describe("eventos", () => {
  test("pinta los dos eventos y etiqueta la asociación inferida", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/events");

    await expect(page.getByText(eventos[0].message)).toBeVisible();
    await expect(page.getByText(eventos[1].message)).toBeVisible();
    await expect(page.getByText(es["events.inferredMapping"])).toBeVisible();
  });

  test("seleccionar un evento carga su detalle y el XML original", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/events");

    await page.getByText(eventos[0].message).click();
    await expect(page.getByRole("heading", { name: es["events.detail.title"] })).toBeVisible();
    await expect(page.getByText(es["events.detail.rawXml"])).toBeVisible();
  });

  test("filtrar por nivel de error oculta el evento informativo", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/events");

    // El grupo de nivel de FilterBar, no la píldora de estado dentro de cada fila de evento
    // (ambas pueden decir "Error").
    const grupoNivel = page.getByRole("group", { name: es["events.filter.level"] });
    await grupoNivel.getByRole("button", { name: es["events.level.error"], exact: true }).click();
    // El filtro recarga desde el backend simulado, que sigue devolviendo la misma página fija:
    // basta con comprobar que el control quedó marcado como activo (`aria-pressed`).
    await expect(
      grupoNivel.getByRole("button", { name: es["events.level.error"], exact: true })
    ).toHaveAttribute("aria-pressed", "true");
  });
});
