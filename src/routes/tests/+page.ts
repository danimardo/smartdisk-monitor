/** Carga inicial de la pantalla de pruebas (constitución §XIV).
 *
 *  Pide inventario **y** historial: esta pantalla puede ser la primera que se visite (enlace
 *  directo, recarga), así que no puede asumir que el panel general ya rellenó `app.devices`. El
 *  inventario alimenta el store igual que hace `/` (mismo guardián `app.loadedAt`); el historial
 *  y sus actualizaciones posteriores llegan por `test:progress` (ADR-015), no recargando la ruta.
 */
import { getDevices, getTestRuns } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const [inventory, testRuns] = await Promise.all([getDevices(), getTestRuns()]);
  return { inventory, testRuns };
};
