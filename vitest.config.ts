import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";
import { resolve } from "node:path";

/** Configuración de test aparte de la de Vite: `vitest/config` extiende el tipo con `test`,
 *  que la config de SvelteKit no admite.
 *
 *  Los umbrales de cobertura implementan el principio VIII de la constitución. No son un objetivo
 *  aspiracional: bloquean la integración. Están puestos por capa a propósito — un porcentaje
 *  uniforme incentivaría escribir pruebas triviales de marcado mientras la lógica que decide si un
 *  disco está sano queda igual de cubierta que un botón.
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
    environment: "jsdom",
    include: ["src/**/*.{test,spec}.{ts,js}"],
    coverage: {
      provider: "v8",
      reporter: ["text-summary", "html", "lcov"],
      reportsDirectory: "coverage",
      // Solo la lógica: los componentes y las rutas se cubren por estados, no por porcentaje.
      include: [
        "src/lib/design/**/*.ts",
        "src/lib/api/**/*.ts",
        "src/lib/i18n/**/*.ts",
        "src/lib/stores/**/*.ts",
        "src/lib/logger.ts"
      ],
      exclude: ["**/*.test.ts", "**/*.spec.ts", "**/*.d.ts"],
      thresholds: {
        // Constitución §VIII: 70 % en la lógica del frontend.
        lines: 70,
        functions: 70,
        branches: 70,
        statements: 70
      }
    }
  }
});
