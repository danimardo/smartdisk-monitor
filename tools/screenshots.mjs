#!/usr/bin/env node
/**
 * Regenera las capturas del README (`docs/screenshots/`).
 *
 * `e2e/ui/capturas.spec.ts` ya sabe pintar cada pantalla en tema claro y oscuro con el IPC
 * simulado y datos de ejemplo; este script solo la invoca para las secciones que salen en el
 * README y copia el resultado con el nombre que espera el `<picture>` de `README.md`.
 *
 * Hacerlo con un script y no a mano es lo que impide que las capturas se queden calladamente
 * obsoletas cada vez que cambia una pantalla (le pasó al panel general tras ADR-051).
 *
 *   pnpm docs:screenshots
 */

import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const ORIGEN = join(ROOT, "design/entregable-rediseno/salida/capturas");
const DESTINO = join(ROOT, "docs/screenshots");

/** Los slugs de `SECCIONES` de `capturas.spec.ts` que el README enseña (el asistente inicial no). */
const SECCIONES = ["panel-general", "detalle-disco", "alertas", "eventos", "pruebas", "informes", "ajustes"];
const TEMAS = ["claro", "oscuro"];

console.log("Generando capturas con Playwright (IPC simulado)…");
// Se invoca el CLI de Playwright con `node` directamente: `spawnSync` de un `.cmd` (pnpm/npx)
// falla en Node ≥ 20 en Windows sin `shell: true`, y `shell: true` con argumentos es un aviso
// de obsolescencia. El JS del CLI no tiene ese problema.
const playwrightCli = createRequire(import.meta.url).resolve("@playwright/test/cli");
execFileSync(process.execPath, [playwrightCli, "test", "e2e/ui/capturas.spec.ts", "-g", "sección ·"], {
  cwd: ROOT,
  stdio: "inherit",
  env: { ...process.env, SDM_CAPTURAS: "1" }
});

mkdirSync(DESTINO, { recursive: true });
let copiadas = 0;
for (const slug of SECCIONES) {
  for (const tema of TEMAS) {
    const origen = join(ORIGEN, `${slug}__${tema}__1280x800__completa.png`);
    if (!existsSync(origen)) {
      console.error(`FALTA  ${origen} — ¿cambió el nombre en capturas.spec.ts?`);
      process.exitCode = 1;
      continue;
    }
    copyFileSync(origen, join(DESTINO, `${slug}-${tema}.png`));
    copiadas++;
  }
}
console.log(`\n${copiadas} capturas copiadas a docs/screenshots/. Revísalas a ojo antes de confirmar.`);
