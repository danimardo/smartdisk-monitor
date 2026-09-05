/** Carga inicial de la pantalla de informes (constitución §XIV).
 *
 *  Puede ser la primera pantalla que se visite: pide inventario y alimenta `app.devices` igual
 *  que `/` (mismo guardián `app.loadedAt`), en vez de asumir que el panel general ya lo hizo.
 */
import { getDevices } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const inventory = await getDevices();
  return { inventory };
};
