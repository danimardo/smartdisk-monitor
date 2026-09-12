#!/usr/bin/env node
/**
 * Galería estática de todas las capturas del entregable. Agrupa por pantalla, empareja claro y
 * oscuro, y separa las secciones de los estados. Sin dependencias: solo lee `salida/capturas/`.
 *
 *     node design/entregable-rediseno/generar-indice.mjs
 */

import { readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const DIR = fileURLToPath(new URL("./salida/", import.meta.url));
const CAPTURAS = fileURLToPath(new URL("./salida/capturas/", import.meta.url));

const ficheros = readdirSync(CAPTURAS).filter((n) => n.endsWith(".png"));

/** `panel-general__oscuro__1280x800__completa.png` -> partes. */
function partes(nombre) {
  const base = nombre.replace(/\.png$/, "");
  const trozos = base.split("__");
  return { base, pantalla: trozos[0], tema: trozos[1] ?? "", variante: trozos.slice(2).join(" · ") };
}

const grupos = new Map();
for (const f of ficheros) {
  const p = partes(f);
  const clave = p.pantalla;
  if (!grupos.has(clave)) grupos.set(clave, []);
  grupos.get(clave).push({ ...p, fichero: f });
}

const esEstado = (k) => k.startsWith("estado-");
const claves = [...grupos.keys()].sort((a, b) => {
  if (esEstado(a) !== esEstado(b)) return esEstado(a) ? 1 : -1;
  return a.localeCompare(b);
});

const seccion = (titulo, lista) => `
  <h2>${titulo}</h2>
  ${lista
    .map((k) => {
      const items = grupos
        .get(k)
        .sort((x, y) => x.fichero.localeCompare(y.fichero))
        .map(
          (it) => `
        <figure>
          <a href="capturas/${it.fichero}" target="_blank" rel="noreferrer">
            <img loading="lazy" src="capturas/${it.fichero}" alt="${it.base}">
          </a>
          <figcaption>${it.tema}${it.variante ? ` — ${it.variante}` : ""}</figcaption>
        </figure>`
        )
        .join("");
      const html = grupos.get(k).length
        ? `<p class="html-links">Snapshot HTML:
             <a href="html/${k}__claro.html" target="_blank" rel="noreferrer">claro</a> ·
             <a href="html/${k}__oscuro.html" target="_blank" rel="noreferrer">oscuro</a></p>`
        : "";
      return `<section><h3>${k}</h3>${html}<div class="rejilla">${items}</div></section>`;
    })
    .join("")}
`;

const doc = `<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>SmartDisk Monitor — capturas para rediseño</title>
<style>
  :root { color-scheme: light dark; }
  body { margin: 0; padding: 2rem clamp(1rem, 4vw, 4rem); font: 15px/1.5 system-ui, sans-serif;
         background: #fafafa; color: #1a1a1a; }
  @media (prefers-color-scheme: dark) { body { background: #141414; color: #ededed; } }
  h1 { margin: 0 0 .25rem; }
  h2 { margin: 2.5rem 0 .5rem; padding-bottom: .3rem; border-bottom: 2px solid currentColor; }
  h3 { margin: 1.75rem 0 .5rem; font-family: ui-monospace, monospace; font-size: 1rem; }
  p.lead { margin: 0 0 1rem; opacity: .7; }
  .html-links { margin: .25rem 0 .75rem; font-size: .85rem; opacity: .8; }
  .rejilla { display: grid; grid-template-columns: repeat(auto-fill, minmax(420px, 1fr)); gap: 1rem; }
  figure { margin: 0; border: 1px solid rgba(128,128,128,.35); border-radius: 10px; overflow: hidden;
           background: rgba(128,128,128,.06); }
  img { display: block; width: 100%; height: auto; }
  figcaption { padding: .5rem .75rem; font-size: .82rem; opacity: .75; }
  a { color: inherit; }
</style>
</head>
<body>
  <h1>SmartDisk Monitor — capturas para el rediseño</h1>
  <p class="lead">Generado el ${new Date().toISOString().slice(0, 16).replace("T", " ")}.
     Pincha una imagen para verla a tamaño completo. Lee <code>README.md</code> y
     <code>COMO-ENTREGAR-EL-REDISENO.md</code> antes de empezar.</p>
  ${seccion("Secciones", claves.filter((k) => !esEstado(k)))}
  ${seccion("Estados", claves.filter(esEstado))}
</body>
</html>
`;

writeFileSync(`${DIR}indice.html`, doc, "utf8");
console.log(`indice.html generado con ${ficheros.length} capturas en ${grupos.size} grupos.`);
