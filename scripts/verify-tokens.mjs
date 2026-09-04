#!/usr/bin/env node
/**
 * Impide que el sistema de diseño se erosione (AGENTS.md §2, engineering-conventions §3).
 *
 * La norma dice que un componente no puede llevar un color, un radio, una sombra o un tamaño de
 * fuente literal, ni escribir `backdrop-filter` a mano. Sin una comprobación automática esa norma
 * dura exactamente hasta el primer día de prisa, y luego el tema oscuro empieza a fallar por
 * sitios que nadie sabe explicar.
 *
 * Solo se revisa `src/`: `tokens.css` es la fuente de verdad y ahí los literales son legítimos.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative, resolve } from "node:path";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SCAN = ["src/lib/components", "src/routes"];

const RULES = [
  {
    id: "color-literal",
    // #abc, #aabbcc, rgb(...), rgba(...), hsl(...)
    re: /#[0-9a-fA-F]{3,8}\b|\brgba?\(|\bhsla?\(/g,
    message: "color literal: usa una utilidad Tailwind o var(--sdm-*)"
  },
  {
    id: "radius-literal",
    re: /border-radius\s*:\s*\d|rounded-\[/g,
    message: "radio literal: usa rounded-card / rounded-inner / rounded-nav / rounded-pill"
  },
  {
    id: "shadow-literal",
    re: /box-shadow\s*:\s*(?!var\()/g,
    message: "sombra literal: usa shadow-card / shadow-lift / shadow-edge"
  },
  {
    id: "font-size-literal",
    re: /font-size\s*:\s*[\d.]+(px|rem|em)|text-\[/g,
    message: "tamaño de fuente literal: usa text-2xs … text-metric"
  },
  {
    id: "backdrop-filter",
    re: /backdrop-filter\s*:/g,
    message: "backdrop-filter a mano: usa .sdm-material, .sdm-material-chrome o .sdm-material-overlay"
  }
];

/** Ignora lo que está dentro de un comentario: las normas se explican citando lo que prohíben. */
function stripComments(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (m) => "\n".repeat((m.match(/\n/g) || []).length))
    .replace(/^\s*\/\/.*$/gm, "")
    .replace(/<!--[\s\S]*?-->/g, (m) => "\n".repeat((m.match(/\n/g) || []).length));
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
    if (statSync(full).isDirectory()) yield* walk(full);
    else if (/\.(svelte|ts|css)$/.test(name)) yield full;
  }
}

let violations = 0;

for (const base of SCAN) {
  for (const file of walk(join(ROOT, base))) {
    const raw = readFileSync(file, "utf8");
    const source = stripComments(raw);
    const lines = source.split("\n");

    for (const rule of RULES) {
      lines.forEach((line, i) => {
        rule.re.lastIndex = 0;
        const m = rule.re.exec(line);
        if (!m) return;
        console.error(
          `${relative(ROOT, file)}:${i + 1}  [${rule.id}] ${rule.message}\n    ${line.trim().slice(0, 100)}`
        );
        violations++;
      });
    }
  }
}

if (violations > 0) {
  console.error(`\n${violations} violación(es) del sistema de diseño.`);
  process.exit(1);
}
console.log("Sistema de diseño: sin valores visuales literales.");
