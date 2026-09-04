import { svelte } from "@sveltejs/vite-plugin-svelte";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";
import { resolve } from "node:path";

/** Configuración de las pruebas que **necesitan un navegador de verdad**.
 *
 *  Es un fichero aparte y no un proyecto dentro de `vitest.config.ts` a propósito
 *  (`docs/testing-strategy.md` §24): `pnpm test` tiene que seguir siendo la suite rápida que se
 *  teclea mientras se programa, y Stryker apunta solo a la de Node — mutar código lanza la suite
 *  cientos de veces y abrir un navegador en cada una la haría inviable.
 *
 *  **Qué justifica el navegador**, frente a jsdom (§7):
 *  - contraste efectivo con `getComputedStyle` sobre el material compuesto;
 *  - que el material cae a `--sdm-solid` sin `backdrop-filter`;
 *  - que el tema oscuro resuelve todas las variables — una definida solo dentro de un bloque de
 *    media queda sin valor, y ese fallo es invisible en jsdom;
 *  - que las animaciones desaparecen con `prefers-reduced-motion`;
 *  - que el foco es visible y nadie anula `:focus-visible`;
 *  - que el texto no se recorta a 1024 × 560, el mínimo técnico.
 *
 *  Lo que ya cubre una prueba de Node **no se repite aquí**: `capacityState()` se prueba una vez
 *  en Node, no otra vez pintando una barra.
 */
export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    alias: {
      $lib: resolve("./src/lib"),
      "$design-system": resolve("./src/design-system")
    },
    conditions: ["browser"]
  },
  test: {
    // `*.browser.test.ts`, y no `*.svelte.test.ts` como proponía la primera versión de la
    // estrategia: ese sufijo ya está tomado por las pruebas de los módulos `.svelte.ts` con runas
    // (`theme.svelte.test.ts`), que corren en Node. El discriminante real es el entorno.
    include: ["src/**/*.browser.test.ts"],
    setupFiles: ["./tests/setup-browser.ts"],
    css: true,
    browser: {
      enabled: true,
      provider: playwright(),
      headless: true,
      instances: [{ browser: "chromium" }],
      // El viewport es el mínimo técnico que exige `Design-system/AGENTS.md` §8: si un componente
      // se recorta aquí, se recorta en la ventana más pequeña que la aplicación permite abrir.
      viewport: { width: 1024, height: 560 },
      screenshotFailures: false
    }
  }
});
