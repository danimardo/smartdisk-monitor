#!/usr/bin/env node
/**
 * Impide que el sistema de diseño se erosione (ui-design.md §2, engineering-conventions §3).
 *
 * Dos comprobaciones independientes:
 *   1. Que no haya una segunda copia del sistema de diseño en el repositorio (ADR-029).
 *   2. Que ningún componente lleve un valor visual literal.
 *
 * La norma dice que un componente no puede llevar un color, un radio, una sombra o un tamaño de
 * fuente literal, ni escribir `backdrop-filter` a mano. Sin una comprobación automática esa norma
 * dura exactamente hasta el primer día de prisa, y luego el tema oscuro empieza a fallar por
 * sitios que nadie sabe explicar.
 *
 * Solo se revisa `src/`: `tokens.css` es la fuente de verdad y ahí los literales son legítimos.
 */

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative, resolve, sep } from "node:path";

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

/* ---- 1. Una sola copia del sistema de diseño ------------------------------------------------
 *
 * El paquete del diseñador traía su propia copia de `tokens.css`, del catálogo y de los módulos de
 * `design/`. Convivió con la integrada en `src/` y las dos divergieron sin que nadie se enterara:
 * el arreglo de foco `:focus-visible:focus-visible` (WCAG 2.4.7) llegó solo a la copia viva, y
 * como el consolidado se generaba desde la otra, `historias.md` estuvo publicando durante días
 * unos tokens que reintroducían un fallo de accesibilidad ya resuelto. El motivo completo, con la
 * divergencia medida fichero a fichero, está en ADR-029.
 *
 * Por eso esto no es una advertencia: es un fallo de integración. */
const CANONICOS = [
  { archivo: "tokens.css", unico: "src/design-system/tokens.css" },
  { archivo: "tokens.json", unico: "src/design-system/tokens.json" },
  { archivo: "index.ts", unico: "src/lib/components/index.ts", soloEn: "components" }
];

/** Carpetas donde una coincidencia no significa nada: dependencias, artefactos y salidas. */
const IGNORADAS = new Set([
  "node_modules",
  ".git",
  ".svelte-kit",
  "build",
  "coverage",
  "target",
  "gen",
  "dist"
]);

function* buscar(dir) {
  let entries;
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const e of entries) {
    if (IGNORADAS.has(e.name)) continue;
    const full = join(dir, e.name);
    if (e.isDirectory()) yield* buscar(full);
    else yield full;
  }
}

const todos = [...buscar(ROOT)].map((f) => relative(ROOT, f).split(sep).join("/"));

for (const { archivo, unico, soloEn } of CANONICOS) {
  if (!existsSync(join(ROOT, unico))) {
    console.error(
      `FALTA     ${unico}\n` +
        `          es la copia canónica del sistema de diseño; sin ella no hay fuente de verdad`
    );
    violations++;
    continue;
  }

  const copias = todos.filter((ruta) => {
    if (ruta === unico) return false;
    if (!ruta.endsWith("/" + archivo)) return false;
    // `index.ts` es un nombre corriente: solo cuenta el barrel de un catálogo de componentes.
    if (soloEn && !ruta.includes("/" + soloEn + "/")) return false;
    return true;
  });

  for (const copia of copias) {
    console.error(
      `DUPLICADO ${copia}\n` +
        `          ya existe ${unico}, que es la única copia permitida (ADR-029).\n` +
        `          Dos copias divergen en silencio: borra esta y apunta a la canónica.`
    );
    violations++;
  }
}

/* ---- 2. Cero valores visuales literales ---------------------------------------------------- */

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
console.log("Sistema de diseño: una sola copia y sin valores visuales literales.");
