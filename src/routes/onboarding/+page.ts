/** Carga del asistente inicial (US-002). Trae el inventario (paso 2) y los ajustes (perfil inicial
 *  y valores del `<details>` del paso 3).
 *
 *  El `load` **no lanza**: un fallo de detección se devuelve como `error` para que el paso 2 ofrezca
 *  «Reintentar» y «Omitir» dentro del propio asistente, no un `+error.svelte` que dejaría al usuario
 *  encallado (FR-038). «Omitir» funciona siempre, incluso si todo falla. */
import { getDevices, getSettings, toAppError } from "$lib/api";
import type { AppError, DiskSummary } from "$lib/design/types";
import type { SettingsShape } from "$lib/api/schemas";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  const [ajustes, inventario] = await Promise.allSettled([getSettings(), getDevices()]);

  const settings: SettingsShape | null = ajustes.status === "fulfilled" ? ajustes.value : null;

  if (inventario.status === "fulfilled") {
    const inv = inventario.value;
    return {
      settings,
      devices: [...inv.devices, ...inv.excluded] as DiskSummary[],
      error: null as AppError | null
    };
  }
  return {
    settings,
    devices: [] as DiskSummary[],
    error: toAppError(inventario.reason)
  };
};
