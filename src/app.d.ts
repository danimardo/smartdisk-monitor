declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    /** Datos comunes que cualquier ruta puede exponer desde su `load` y que consume el chrome:
     *  el título y el subtítulo de la barra de herramientas (`+layout.svelte`). Si una ruta no los
     *  pone, la barra usa la etiqueta de la sección. */
    interface PageData {
      title?: string;
      subtitle?: string;
    }
    // interface Platform {}
  }
}

export {};
