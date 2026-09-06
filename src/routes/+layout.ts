/** La aplicación vive dentro de Tauri: no hay servidor que renderice ni datos que cargar en `load`.
 *  Todo el estado viene de comandos Tauri desde el cliente (ADR-014). */
import { redirect } from "@sveltejs/kit";
import { getAppearanceSettings, getDevices, getSettings, setSetting } from "$lib/api";
import type { LayoutLoad } from "./$types";

export const ssr = false;
export const prerender = true;

/** Guardián del asistente inicial (US-002, FR-032/043). Corre **en cliente** (igual que el `load`
 *  de `/alerts` y `/events`): `ssr = false` hace que estos `load` no se ejecuten al prerenderizar,
 *  solo en el navegador, donde `invoke` existe.
 *
 *  Regla (FR-043): `completedAt` nulo **y** sin ninguna señal de configuración previa → se redirige
 *  al asistente. Si ya hay configuración (tema, idioma, perfil de alerta, algún alias o alguna
 *  exclusión), se graba `completedAt` y se sigue sin mostrarlo — es una instalación que viene de
 *  antes de que el asistente existiera. El frontend no puede detectar *cualquier* clave suelta
 *  guardada sin una señal del backend; comprueba lo observable, que cubre los casos realistas de
 *  actualización (`docs/open-questions.md`). */
export const load: LayoutLoad = async ({ url }) => {
  if (url.pathname === "/onboarding") return {};

  // El guardián solo redirige cuando está **seguro** de que falta el asistente. Cualquier fallo al
  // consultar el backend cae a «seguir normal»: nunca hay que atrapar al usuario en un bucle de
  // redirección — el `+layout.svelte` ya explica un fallo de arranque real.
  let settings, appearance, inventario;
  try {
    [settings, appearance, inventario] = await Promise.all([
      getSettings(),
      getAppearanceSettings(),
      getDevices()
    ]);
  } catch {
    return {};
  }

  if (settings.onboarding.completedAt) return {};

  const yaConfigurado =
    appearance.theme !== "system" ||
    appearance.language !== null ||
    settings.alerts.profile !== "balanced" ||
    inventario.excluded.length > 0 ||
    inventario.devices.some((d) => d.alias != null);

  if (yaConfigurado) {
    await setSetting("settings.onboarding.completed_at", new Date().toISOString()).catch(() => {});
    return {};
  }

  redirect(307, "/onboarding");
};
