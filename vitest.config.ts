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
    // Zona horaria fija (§21). Sin esto la suite pasa en un equipo español y falla en CI, que va
    // en UTC — y al revés. Se elige Europe/Madrid, no UTC, precisamente porque tiene cambio de
    // hora: es donde aparecen los fallos de retención, de cooldown y de correlación con el Visor
    // de eventos de Windows, que muestra hora local.
    env: { TZ: "Europe/Madrid" },
    include: ["src/**/*.{test,spec}.{ts,js}"],
    // Las pruebas de navegador tienen su propia configuración (`vitest.browser.config.ts`).
    // Sin excluirlas aquí, `pnpm test` las recogería y fallarían por falta de `page`.
    exclude: ["**/node_modules/**", "**/build/**", "src/**/*.browser.test.ts"],
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
