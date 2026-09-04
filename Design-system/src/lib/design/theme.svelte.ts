/** Preferencia de tema: claro / oscuro / sistema (spec §8, US-003).
 *  El valor efectivo se escribe en <html data-theme="light|dark">; los tokens hacen el resto.
 *  Persistir la preferencia en `settings` vía comando Tauri, no en localStorage. */

import type { ThemePreference } from "./types";
import { refreshAccentForTheme } from "./accent";

const STORAGE_HINT = "settings.appearance.theme"; // clave tipada en la tabla settings

let preference = $state<ThemePreference>("system");
let systemDark = $state(false);

export const theme = {
  get preference() {
    return preference;
  },
  get resolved(): "light" | "dark" {
    return preference === "system" ? (systemDark ? "dark" : "light") : preference;
  },
  /** Llamar una vez al arrancar la app, con el valor leído de settings. */
  init(initial: ThemePreference) {
    preference = initial;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    systemDark = mq.matches;
    mq.addEventListener("change", (e) => (systemDark = e.matches));
    apply();
  },
  set(next: ThemePreference) {
    preference = next;
    apply();
    return { key: STORAGE_HINT, value: next }; // el llamante lo persiste con invoke("set_setting", …)
  }
};

function apply() {
  document.documentElement.dataset.theme = theme.resolved;
  // `--sdm-accent-fg` depende de la superficie, y la superficie cambia con el tema: si no se
  // recalcula aquí, un acento heredado que era legible en claro puede quedar por debajo de AA al
  // pasar a oscuro (open-questions.md §O).
  refreshAccentForTheme();
}

$effect.root(() => {
  $effect(() => {
    if (typeof document !== "undefined") apply();
  });
});
