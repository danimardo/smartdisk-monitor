/** Preparación común de las pruebas de navegador.
 *
 *  Importar `vitest-browser-svelte` tiene efectos: registra `render` en el objeto `page` de Vitest
 *  y encola un `cleanup()` en cada `beforeEach`. Se hace aquí y no en cada fichero para que a nadie
 *  se le olvide y las pruebas se contaminen entre sí.
 */
import "vitest-browser-svelte";

/** Los estilos globales, en este orden: `tokens.css` primero y luego las capas de Tailwind. Sin
 *  esto, `getComputedStyle` devolvería valores vacíos y las comprobaciones de contraste y material
 *  —que son el motivo de usar un navegador— no medirían nada. */
import "../src/app.css";
