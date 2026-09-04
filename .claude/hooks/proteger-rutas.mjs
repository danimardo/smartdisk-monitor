#!/usr/bin/env node
/**
 * Cerrojo de escritura sobre las rutas protegidas del proyecto.
 *
 * Los ficheros de instrucciones son contexto, no configuración aplicada: el agente los lee y
 * trata de seguirlos, pero no hay garantía. Lo que no puede quedar a criterio del modelo se
 * bloquea aquí, que se ejecuta pase lo que pase (constitución, capa de enforcement).
 *
 * Se invoca como hook `PreToolUse` con matcher `Write|Edit`. Recibe el JSON de la llamada por
 * la entrada estándar y responde por la salida estándar con la decisión.
 *
 * En Node y no en shell a propósito: `jq` no está garantizado en Windows y este proyecto ya
 * depende de Node.
 */

import { readFileSync } from "node:fs";

/** Cada regla explica **qué hacer en su lugar**: un bloqueo sin salida solo produce desconcierto. */
const PROTEGIDAS = [
  {
    prueba: (r) => r === "historias.md",
    motivo:
      "`historias.md` es un consolidado GENERADO a partir de 31 ficheros: editarlo directamente " +
      "pierde el cambio en la siguiente regeneración.\n" +
      "En su lugar: edita el fichero de `docs/` que corresponda y ejecuta `pnpm docs:build`."
  },
  {
    prueba: (r) => r.startsWith("third-party/"),
    motivo:
      "`third-party/` contiene binarios redistribuidos cuyos hashes están registrados en " +
      "`THIRD_PARTY_NOTICES.md`. Cambiar un byte hace fallar `pnpm verify:assets` con un " +
      "diagnóstico poco evidente.\n" +
      "Si de verdad hay que actualizar un recurso, sigue el procedimiento de " +
      "`third-party/smartmontools/README.md` y pide autorización antes."
  },
  {
    prueba: (r) => r === ".specify/memory/constitution.md",
    motivo:
      "La constitución solo se modifica con autorización explícita (su propia sección de " +
      "gobernanza lo exige).\n" +
      "En su lugar: presenta la propuesta de enmienda con su motivo, el cambio exacto y su " +
      "impacto, y espera respuesta. Una corrección de errata sí es una revisión `patch` y no " +
      "necesita aprobación: dilo al hacerla."
  }
];

function denegar(motivo) {
  process.stdout.write(
    JSON.stringify({
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "deny",
        permissionDecisionReason: motivo
      }
    })
  );
  process.exit(0);
}

let entrada = "";
try {
  entrada = readFileSync(0, "utf8");
} catch {
  process.exit(0); // Sin entrada legible no se bloquea nada: el hook nunca estorba.
}

let datos;
try {
  datos = JSON.parse(entrada);
} catch {
  process.exit(0);
}

const ruta = datos?.tool_input?.file_path;
if (typeof ruta !== "string" || ruta.length === 0) process.exit(0);

const raiz = (datos.cwd ?? process.cwd()).replace(/\\/g, "/").replace(/\/+$/, "");
const absoluta = ruta.replace(/\\/g, "/");

// Ruta relativa a la raíz del proyecto, en minúsculas: Windows no distingue mayúsculas.
const relativa = (
  absoluta.toLowerCase().startsWith(raiz.toLowerCase() + "/") ? absoluta.slice(raiz.length + 1) : absoluta
).replace(/^\.\//, "");

for (const regla of PROTEGIDAS) {
  if (regla.prueba(relativa)) denegar(regla.motivo);
}

process.exit(0);
