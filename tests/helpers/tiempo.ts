import { vi } from "vitest";

/** Control del tiempo en las pruebas (`docs/testing-strategy.md` §21).
 *
 *  No se escribe un módulo de reloj propio: Vitest ya sabe congelar el reloj del sistema, y
 *  duplicarlo obligaría a que el código de producción expusiera un punto de inyección que solo
 *  usarían las pruebas. Lo que sí hace falta es que congelarlo sea **una sola línea**, porque un
 *  ayudante incómodo se acaba sorteando con `Date.now()` a pelo.
 *
 *  La zona horaria se fija en `vitest.config.ts`, no aquí: tiene que estar puesta antes de que
 *  cualquier módulo evalúe una fecha, y un `beforeEach` llega tarde.
 */

/** Congela el reloj del sistema en un instante ISO. Devuelve la marca en milisegundos, que es lo
 *  que esperan las funciones que aceptan `now` explícito. */
export function congelarTiempo(iso: string): number {
  const marca = Date.parse(iso);
  if (Number.isNaN(marca)) throw new Error(`instante no válido: ${iso}`);
  vi.useFakeTimers();
  vi.setSystemTime(marca);
  return marca;
}

/** Devuelve el control al reloj real. Va en un `afterEach`: un reloj congelado que sobrevive a su
 *  prueba envenena las siguientes, y el fallo aparece lejos de su causa. */
export function descongelarTiempo(): void {
  vi.useRealTimers();
}
