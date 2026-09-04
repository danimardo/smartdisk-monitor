/** La aplicación vive dentro de Tauri: no hay servidor que renderice ni datos que cargar en `load`.
 *  Todo el estado viene de comandos Tauri desde el cliente (ADR-014). */
export const ssr = false;
export const prerender = true;
