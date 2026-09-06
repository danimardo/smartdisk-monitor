/** Carga inicial de la pantalla de ajustes (constitución §XIV). */
import { getAppearanceSettings, getSettings } from "$lib/api";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const [settings, appearance] = await Promise.all([getSettings(), getAppearanceSettings()]);
  return { settings, appearance };
};
