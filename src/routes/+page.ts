/** Carga inicial del panel general (constitución §XIV).
 *
 *  `load` universal, nunca de servidor: no hay servidor. Entrega el inventario listo para
 *  renderizar, de modo que la pantalla no arranca vacía para llenarse un instante después. Las
 *  actualizaciones posteriores llegan por los eventos que empuja el backend (ADR-015), no
 *  recargando la ruta.
 */
import { getDevices } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const inventory = await getDevices();
  return { inventory };
};
