/** Vocabulario de los perfiles de alerta (v3, ADR-036). Los umbrales concretos los escribe el
 *  backend (`PerfilAlerta::umbrales()`) al elegir un perfil; aquí solo vive el identificador y sus
 *  claves de texto, compartidas por Ajustes y por el paso 3 del asistente inicial — una sola fuente,
 *  para que las dos pantallas no se desincronicen. */

export type PerfilAlerta = "cautious" | "balanced" | "quiet";

export const PERFIL_RECOMENDADO: PerfilAlerta = "balanced";

export const PERFILES: readonly { id: PerfilAlerta; label: string; hint: string }[] = [
  {
    id: "cautious",
    label: "settings.alerts.profile.cautious",
    hint: "settings.alerts.profile.cautiousHint"
  },
  {
    id: "balanced",
    label: "settings.alerts.profile.balanced",
    hint: "settings.alerts.profile.balancedHint"
  },
  { id: "quiet", label: "settings.alerts.profile.quiet", hint: "settings.alerts.profile.quietHint" }
];

/** Clave de texto del nombre de un perfil, incluido `custom` (que no está en `PERFILES`). */
export function perfilLabelKey(id: string): string {
  return id === "custom"
    ? "settings.alerts.profile.custom"
    : (PERFILES.find((p) => p.id === id)?.label ?? "settings.alerts.profile.balanced");
}
