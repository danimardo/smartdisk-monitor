import { expect, test, type Page } from "@playwright/test";
import { apariencia, acento, inventario } from "./fixtures/respuestas";
import { generarInventarioDeCarga, generarPaginaDeEventosDeCarga } from "./fixtures/carga";
import { emitirEvento, instalarIpcFalso } from "./ipc-falso";

/** Medición de R3 (`specs/001-monitor-discos-windows/research.md`, SC-007/SC-009): con 20 discos y
 *  5.000 eventos, la interfaz nunca deja de responder más de 50 ms seguidos. La API de "long
 *  tasks" del navegador solo informa de tareas de **50 ms o más** (es su propio umbral de
 *  disparo): cualquier entrada registrada durante la interacción medida ya es, por definición, un
 *  incumplimiento.
 *
 *  **No se mide la navegación ni la hidratación inicial**, y no por comodidad: medido con este
 *  mismo arnés, la primera navegación produce 70-120 ms de tarea larga **incluso con 0 o 2
 *  discos** — es coste fijo de evaluar el paquete de la aplicación en un Chromium recién
 *  arrancado, no algo que dependa de cuántos discos haya. Incluirlo en la medición habría hecho
 *  fallar la prueba por una razón ajena a SC-007, que habla de "seguir respondiendo *durante*
 *  recopilaciones" — un dato que llega en caliente a una aplicación ya arrancada, no el arranque
 *  en sí. Por eso cada prueba deja pasar la carga inicial y **luego** reinicia el observador,
 *  midiendo solo la actualización o el desplazamiento que de verdad importa.
 */

const VEINTE_DISCOS = generarInventarioDeCarga(20);
const CINCO_MIL_EVENTOS = generarPaginaDeEventosDeCarga(5000);

async function instalarObservadorDeTareasLargas(page: Page) {
  await page.addInitScript(() => {
    const w = window as unknown as { __longTasks__: number[] };
    w.__longTasks__ = [];
    new PerformanceObserver((lista) => {
      for (const entrada of lista.getEntries()) w.__longTasks__.push(entrada.duration);
    }).observe({ entryTypes: ["longtask"] });
  });
}

function reiniciarTareasLargas(page: Page) {
  return page.evaluate(() => {
    (window as unknown as { __longTasks__: number[] }).__longTasks__ = [];
  });
}

function leerTareasLargas(page: Page) {
  return page.evaluate(() => (window as unknown as { __longTasks__: number[] }).__longTasks__);
}

test.describe("rendimiento @rendimiento", () => {
  test("desplazar 5.000 eventos no produce ninguna tarea de 50 ms o más", async ({ page }) => {
    await instalarObservadorDeTareasLargas(page);
    await instalarIpcFalso(page, {
      get_appearance_settings: apariencia,
      get_system_accent_color: acento,
      get_devices: { devices: [], excluded: [], sources: [], paused: false, pausedSince: null },
      get_system_events: CINCO_MIL_EVENTOS,
      get_log_level: "info"
    });
    await page.goto("/events");
    await expect(page.getByRole("main")).toBeVisible();
    await expect(page.getByText(CINCO_MIL_EVENTOS.events[0].message)).toBeVisible();

    await reiniciarTareasLargas(page);

    // Diez tramos de desplazamiento, simulando que se recorre la lista entera con rueda de ratón.
    // La sincronización entre tramos es un fotograma real (`requestAnimationFrame`), no una espera
    // arbitraria: es lo que exige `.claude/rules/pruebas.md` frente a `waitForTimeout`.
    const lista = page.getByRole("list");
    await lista.hover();
    for (let i = 0; i < 10; i++) {
      await page.mouse.wheel(0, 400);
      await page.evaluate(() => new Promise((r) => requestAnimationFrame(r)));
    }

    const tareas = await leerTareasLargas(page);
    expect(tareas, `tareas de 50 ms o más durante el desplazamiento: ${JSON.stringify(tareas)}`).toEqual([]);
  });

  test("recibir 20 discos en caliente (metrics:updated) no produce ninguna tarea de 50 ms o más", async ({
    page
  }) => {
    await instalarObservadorDeTareasLargas(page);
    // Arranca con el inventario pequeño de siempre: dejar pasar el coste fijo de la primera
    // navegación (ver cabecera) antes de medir nada.
    await instalarIpcFalso(page, {
      get_appearance_settings: apariencia,
      get_system_accent_color: acento,
      get_devices: inventario,
      get_log_level: "info"
    });
    await page.goto("/");
    await expect(page.getByRole("main")).toBeVisible();
    await expect(page.getByText(inventario.devices[0].model).first()).toBeVisible();

    await reiniciarTareasLargas(page);

    // La aplicación ya está arrancada y respondiendo: ahora llega una recopilación real con 20
    // discos, exactamente como la emitiría el backend al cerrar un ciclo (ADR-015).
    await emitirEvento(page, "metrics:updated", {
      emittedAt: "2026-09-04T10:05:00Z",
      devices: VEINTE_DISCOS.devices,
      sources: [],
      historyWriteHalted: false
    });
    await expect(page.getByText(VEINTE_DISCOS.devices[19].model).first()).toBeVisible();

    const tareas = await leerTareasLargas(page);
    expect(
      tareas,
      `tareas de 50 ms o más al recibir 20 discos en caliente: ${JSON.stringify(tareas)}`
    ).toEqual([]);
  });
});
