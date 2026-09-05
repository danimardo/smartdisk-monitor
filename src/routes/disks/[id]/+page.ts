/** Carga inicial del detalle de disco (constitución §XIV).
 *
 *  `prerender = false` porque los identificadores no existen hasta que el backend enumera el
 *  hardware. Si el disco no existe, el error sube a `+error.svelte` con la forma del principio X:
 *  no se devuelve un objeto vacío que parezca un disco válido.
 */
import { getDeviceDetail, getSettings } from "$lib/api";
import type { PageLoad } from "./$types";

export const prerender = false;

export const load: PageLoad = async ({ params }) => {
  const [disk, settings] = await Promise.all([getDeviceDetail(params.id), getSettings()]);
  return { disk, settings };
};
