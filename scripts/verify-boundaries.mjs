#!/usr/bin/env node
/**
 * Verifica los principios XI, XII y XIII de la constitución:
 *
 *   XI.   Ningún dato de una frontera entra sin validar. En la práctica: ninguna aserción de tipo
 *         sobre lo que devuelve `invoke` o `listen`.
 *   XII.  Ni `process.env` ni `$env/*` en el código de la aplicación. No hay servidor: `$env/dynamic`
 *         no existe y `$env/static/public` requiere enmienda.
 *   XIII. Todo silencio de una herramienta (`svelte-ignore`, `eslint-disable`, `@ts-expect-error`)
 *         enlaza a una entrada de `docs/known-issues.md`. Un silencio sin rastro se olvida.
 *   XV.   Ningún `console.*` en el código de la aplicación: todo pasa por `$lib/logger`.
 */

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative, resolve } from "node:path";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

/** Ficheros de configuración que corren en Node al construir: ahí `process.env` es legítimo. */
const BUILD_FILES = new Set([
  "vite.config.ts",
  "svelte.config.js",
  "vitest.config.ts",
  "tailwind.config.cjs"
]);

const RULES = [
  {
    id: "env-prohibido",
    re: /\bprocess\.env\b/g,
    message: "process.env no existe en el navegador y la configuración del producto vive en `settings` (§XII)"
  },
  {
    id: "env-dynamic",
    re: /\$env\/(dynamic|static)\/(private|public)/g,
    message: "$env/* no está disponible: la app es estática y sin servidor. Usa la tabla `settings` (§XII)"
  },
  {
    id: "console-directo",
    re: /\bconsole\.(log|debug|info|warn|error|trace)\b/g,
    message: "console.* directo: usa createLogger() de $lib/logger (§XV)"
  },
  {
    id: "assert-ipc",
    // `invoke<T>(` o `listen<T>(` con parámetro de tipo: es una aserción, no una validación.
    re: /\b(invoke|listen)\s*<(?!unknown>)/g,
    message: "aserción de tipo sobre datos de IPC: valida con su esquema Zod (§XI)"
  }
];

/** Silencios que deben enlazar al registro. */
const SILENCERS = /(svelte-ignore\s+\S+|eslint-disable(?:-next-line)?|@ts-expect-error|@ts-ignore)/g;

function stripComments(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (m) => "\n".repeat((m.match(/\n/g) || []).length))
    .replace(/^\s*\/\/.*$/gm, "");
}

function* walk(dir) {
  let entries;
  try {
    entries = readdirSync(dir);
  } catch {
    return;
  }
  for (const name of entries) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) {
      if (name === "node_modules" || name === "target" || name === ".svelte-kit") continue;
      yield* walk(full);
    } else if (/\.(svelte|ts|js)$/.test(name) && !/\.test\.ts$/.test(name)) {
      yield full;
    }
  }
}

let violations = 0;
const report = (file, line, id, message, text) => {
  console.error(`${relative(ROOT, file)}:${line}  [${id}] ${message}\n    ${text.trim().slice(0, 100)}`);
  violations++;
};

/* --- Reglas de contenido, solo en el código de la aplicación --- */
/** El envoltorio de registro es el único sitio donde `console` es legítimo (§XV): es quien lo
 *  encapsula para que nadie más tenga que usarlo. */
const LOGGER_WRAPPER = join(ROOT, "src", "lib", "logger.ts");

for (const file of walk(join(ROOT, "src"))) {
  const raw = readFileSync(file, "utf8");
  const lines = stripComments(raw).split("\n");
  for (const rule of RULES) {
    if (rule.id === "console-directo" && file === LOGGER_WRAPPER) continue;
    lines.forEach((line, i) => {
      rule.re.lastIndex = 0;
      if (rule.re.test(line)) report(file, i + 1, rule.id, rule.message, line);
    });
  }
}

/* --- process.env fuera de los ficheros de construcción --- */
for (const name of readdirSync(ROOT)) {
  if (!/\.(ts|js|cjs|mjs)$/.test(name) || BUILD_FILES.has(name)) continue;
  const full = join(ROOT, name);
  if (statSync(full).isDirectory()) continue;
  const lines = stripComments(readFileSync(full, "utf8")).split("\n");
  lines.forEach((line, i) => {
    if (/\bprocess\.env\b/.test(line)) {
      report(full, i + 1, "env-prohibido", "process.env solo en ficheros de construcción (§XII)", line);
    }
  });
}

/* --- Silencios sin entrada en el registro --- */
const registryPath = join(ROOT, "docs/known-issues.md");
const registry = existsSync(registryPath) ? readFileSync(registryPath, "utf8") : "";

for (const dir of ["src", "src-tauri/src"]) {
  for (const file of walk(join(ROOT, dir))) {
    const raw = readFileSync(file, "utf8");
    const lines = raw.split("\n");
    lines.forEach((line, i) => {
      SILENCERS.lastIndex = 0;
      const m = SILENCERS.exec(line);
      if (!m) return;
      // Se admite el enlace en la misma línea o en las tres siguientes: los comentarios de
      // justificación suelen ir debajo del silencio.
      const ventana = lines.slice(i, i + 4).join(" ");
      const enlaza = /known-issues\.md\s*#\d+|known-issues\.md/.test(ventana);
      if (!enlaza) {
        report(
          file,
          i + 1,
          "silencio-sin-registro",
          "todo silencio enlaza a docs/known-issues.md (§XIII)",
          line
        );
      } else if (registry) {
        const num = (ventana.match(/known-issues\.md\s*#(\d+)/) || [])[1];
        if (num && !new RegExp(`^\\|\\s*${num}\\s*\\|`, "m").test(registry)) {
          report(file, i + 1, "registro-inexistente", `known-issues.md no tiene la entrada #${num}`, line);
        }
      }
    });
  }
}

if (violations > 0) {
  console.error(`\n${violations} violación(es) de los principios XI, XII o XIII.`);
  process.exit(1);
}
console.log("Fronteras, entorno y silencios: conformes.");
