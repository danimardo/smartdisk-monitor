/** Carga inicial de la pantalla de eventos (constitución §XIV).
 *
 *  Sin filtro: los filtros se aplican en la pantalla, sobre nuevas peticiones al cambiar
 *  (a diferencia de alertas, aquí no tiene sentido traer todo el historial de una vez —
 *  puede haber miles de eventos, US-021). El primer disco vía `?deviceId=` preaplica el filtro
 *  al entrar desde el detalle de un disco (`docs/open-questions.md` J.10).
 */
import { getSystemEvents } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ url }) => {
  const deviceId = url.searchParams.get("deviceId") ?? undefined;
  const page = await getSystemEvents({ deviceId, limit: 200 });
  return { page, deviceId };
};
