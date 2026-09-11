#!/usr/bin/env node
/**
 * Renombra el instalador NSIS que acaba de producir `tauri build` a un nombre fijo.
 *
 * Por defecto, Tauri lo llama `<productName>_<version>_<arch>-setup.exe` — con espacios (viene de
 * "SmartDisk Monitor") y el número de versión dentro del nombre. Eso obliga a actualizar cualquier
 * enlace de descarga fijo (p. ej. `releases/latest/download/...`) en cada versión. Aquí se deja
 * siempre con el mismo nombre, sin espacios ni versión, para que un enlace de "última versión" no
 * tenga que cambiar nunca.
 *
 * Se ejecuta después de `tauri build` (ver `app:build` en package.json), nunca antes: necesita que
 * el `.exe` ya exista.
 */

import { existsSync, readdirSync, renameSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const NSIS_DIR = join(ROOT, "src-tauri", "target", "release", "bundle", "nsis");
const NOMBRE_FIJO = "smartdisk-monitor-setup.exe";

if (!existsSync(NSIS_DIR)) {
  console.error(`No existe ${NSIS_DIR}\n` + '¿Se ha ejecutado "tauri build" antes de este paso?');
  process.exit(1);
}

const candidatos = readdirSync(NSIS_DIR).filter((f) => f.toLowerCase().endsWith(".exe") && f !== NOMBRE_FIJO);

if (candidatos.length === 0) {
  console.error(`Ningún instalador nuevo en ${NSIS_DIR} (solo ${NOMBRE_FIJO}, si ya existía).`);
  process.exit(1);
}
if (candidatos.length > 1) {
  console.error(
    `Más de un .exe en ${NSIS_DIR}, no se sabe cuál renombrar:\n` +
      candidatos.map((f) => `  - ${f}`).join("\n")
  );
  process.exit(1);
}

const origen = join(NSIS_DIR, candidatos[0]);
const destino = join(NSIS_DIR, NOMBRE_FIJO);

// `renameSync` no sobrescribe en Windows si el destino ya existe (de una compilación anterior).
if (existsSync(destino)) rmSync(destino);
renameSync(origen, destino);

console.log(`${candidatos[0]} -> ${NOMBRE_FIJO}`);
