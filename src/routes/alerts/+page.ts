/** Carga inicial de la pantalla de alertas (constitución §XIV).
 *
 *  Se pide el historial completo, sin filtro: el filtro por estado se aplica en la pantalla, no en
 *  el backend, para que cambiar de pestaña no dispare una nueva petición. Las actualizaciones
 *  posteriores llegan por `alerts:changed` (ADR-015), no recargando la ruta.
 */
import { getAlertGroups } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const alerts = await getAlertGroups();
  return { alerts };
};
