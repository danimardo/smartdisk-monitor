import { defineConfig, devices } from "@playwright/test";

/** Plano de interfaz: la aplicación real dentro de un navegador, con el IPC de Tauri simulado
 *  (`docs/testing-strategy.md` §10). Cubre navegación, estados, formularios, errores, responsive y
 *  accesibilidad — todo lo que depende de la interfaz y no del binario.
 *
 *  El plano de aplicación real (`tauri-driver`) es otra cosa y vive aparte: Playwright no puede
 *  conducir una ventana de Tauri.
 *
 *  Se sirve el **build**, no `vite dev`: es donde aparecen los problemas que el servidor de
 *  desarrollo esconde (§10).
 */
export default defineConfig({
  testDir: "./e2e/ui",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [["github"], ["html", { open: "never" }]] : [["list"]],
  // Sin `waitForTimeout` ni esperas arbitrarias en ninguna prueba (§13): la sincronización es
  // por estado observable. Este plazo es solo el techo antes de declarar un fallo.
  timeout: 30_000,
  expect: { timeout: 5_000 },
  use: {
    baseURL: "http://localhost:4173",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "off"
  },
  projects: [
    {
      name: "chromium",
      // Chromium y solo Chromium: el WebView2 de la aplicación es Chromium. Probar en Firefox o
      // WebKit mediría un motor que ningún usuario va a ejecutar.
      use: { ...devices["Desktop Chrome"], viewport: { width: 1280, height: 800 } }
    }
  ],
  webServer: {
    command: "pnpm build && pnpm preview --port 4173 --strictPort",
    url: "http://localhost:4173",
    reuseExistingServer: !process.env.CI,
    timeout: 180_000
  }
});
