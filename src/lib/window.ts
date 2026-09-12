/** Único punto del frontend que importa `@tauri-apps/api/window` (spec 011, barra de título
 *  propia). `TitleBar.svelte` es presentacional y recibe estas funciones como *callbacks*, igual
 *  que `ExplicacionModal` recibe `onreprocesar` — ninguna pantalla ni componente toca
 *  `@tauri-apps/api` directamente (mismo principio que `frontera-ipc.md` aplica a los comandos
 *  propios, aunque estas no lo sean: son API ya tipada del SDK oficial de Tauri, sin esquema propio
 *  que mantener).
 *
 *  Los cinco permisos que estas llamadas necesitan viven en `src-tauri/capabilities/default.json`
 *  (ADR-059). Sin ellos, Tauri rechaza la llamada con un error claro en tiempo de ejecución. */

import { getCurrentWindow } from "@tauri-apps/api/window";

const ventana = () => getCurrentWindow();

export const minimizeWindow = (): Promise<void> => ventana().minimize();

export const toggleMaximizeWindow = (): Promise<void> => ventana().toggleMaximize();

export const closeWindow = (): Promise<void> => ventana().close();

export const isWindowMaximized = (): Promise<boolean> => ventana().isMaximized();

/** Envuelve `onResized` para que quien llama no dependa del tipo `PhysicalSize` de Tauri: solo
 *  necesita saber que algo cambió, para volver a preguntar `isWindowMaximized()`. Devuelve la
 *  función de baja, para limpiar el listener igual que ya se hace con `unsubscribe` en
 *  `+layout.svelte`. */
export const onWindowResized = async (cb: () => void): Promise<() => void> => {
  return ventana().onResized(() => cb());
};
