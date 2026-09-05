/** Carga inicial de la pantalla de ajustes (constitución §XIV). */
import { getSettings } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const settings = await getSettings();
  return { settings };
};
