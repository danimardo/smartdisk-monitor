#!/usr/bin/env node
/**
 * Comprueba los diccionarios (AGENTS.md §1, engineering-conventions §3).
 *
 *   1. Paridad: `es.json` y `en.json` tienen exactamente las mismas claves. Una clave que solo
 *      existe en uno de los dos produce texto en el idioma equivocado, y `t()` devuelve la clave
 *      cruda, que es peor todavía porque parece un error de programación.
 *   2. Interpolaciones: `{nombre}` debe aparecer en ambos idiomas. Una traducción que se come un
 *      marcador pierde el dato, no solo la forma.
 *   3. Plurales: si existe `clave.one`, debe existir `clave.other`.
 *   4. Claves usadas que no existen: `t("x.y")` con `x.y` ausente de los diccionarios.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative, resolve } from "node:path";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const I18N = join(ROOT, "src/lib/i18n");

const es = JSON.parse(readFileSync(join(I18N, "es.json"), "utf8"));
const en = JSON.parse(readFileSync(join(I18N, "en.json"), "utf8"));

let problems = 0;
const fail = (msg) => {
  console.error(msg);
  problems++;
};

// 1. Paridad de claves
const esKeys = new Set(Object.keys(es));
const enKeys = new Set(Object.keys(en));
for (const k of esKeys) if (!enKeys.has(k)) fail(`solo en es.json: ${k}`);
for (const k of enKeys) if (!esKeys.has(k)) fail(`solo en en.json: ${k}`);

// 2. Interpolaciones coherentes
const vars = (s) => new Set([...String(s).matchAll(/\{(\w+)\}/g)].map((m) => m[1]));
for (const k of esKeys) {
  if (!enKeys.has(k)) continue;
  const a = vars(es[k]);
  const b = vars(en[k]);
  for (const v of a) if (!b.has(v)) fail(`"${k}": {${v}} está en es pero no en en`);
  for (const v of b) if (!a.has(v)) fail(`"${k}": {${v}} está en en pero no en es`);
}

// 3. Plurales completos
for (const k of esKeys) {
  if (k.endsWith(".one") && !esKeys.has(`${k.slice(0, -4)}.other`)) {
    fail(`"${k}" no tiene su forma .other`);
  }
}

// 4. Claves usadas en el código que no existen
function* walk(dir) {
  let entries;
  try {
    entries = readdirSync(dir);
  } catch {
    return;
  }
  for (const name of entries) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) yield* walk(full);
    else if (/\.(svelte|ts)$/.test(name)) yield full;
  }
}

const USE = /\bt\(\s*["'`]([\w.]+)["'`]/g;
const USE_PLURAL = /\btp\(\s*["'`]([\w.]+)["'`]/g;

for (const file of walk(join(ROOT, "src"))) {
  if (file.includes(`${join("src", "lib", "i18n")}`)) continue;
  const src = readFileSync(file, "utf8");

  for (const m of src.matchAll(USE)) {
    if (!esKeys.has(m[1])) fail(`${relative(ROOT, file)}: t("${m[1]}") no existe en los diccionarios`);
  }
  for (const m of src.matchAll(USE_PLURAL)) {
    if (!esKeys.has(`${m[1]}.other`)) {
      fail(`${relative(ROOT, file)}: tp("${m[1]}") necesita "${m[1]}.one" y "${m[1]}.other"`);
    }
  }
}

if (problems > 0) {
  console.error(`\n${problems} problema(s) de i18n.`);
  process.exit(1);
}
console.log(`i18n: ${esKeys.size} claves, es y en sincronizados.`);
