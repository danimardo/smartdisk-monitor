/** Carga inicial de la pantalla de ajustes (constitución §XIV). */
import { estadoIa, getAppearanceSettings, getSettings } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const [settings, appearance, iaEstado] = await Promise.all([
    getSettings(),
    getAppearanceSettings(),
    estadoIa()
  ]);
  return { settings, appearance, iaEstado };
};
