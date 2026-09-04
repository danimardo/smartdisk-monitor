/** Única API de registro del frontend (constitución §XV).
 *
 *  **Ningún fichero de la aplicación llama a `console.*` directamente.** Verificado en CI. Pasar
 *  todo por aquí permite cambiar el destino, el filtro o el formato sin tocar cien ficheros, y es
 *  lo que hace posible que un fallo de interfaz acabe en el ZIP de diagnóstico.
 *
 *  El WebView no puede escribir ficheros, así que:
 *    - `debug` e `info` van solo a la consola: son ruido de desarrollo, y enviarlos por IPC
 *      costaría más que el valor que aportan;
 *    - `warn` y `error` van además a Rust, que los escribe en el fichero único.
 *
 *  No se usa una biblioteca externa: lo que aquí hace falta son cuarenta líneas, y el envío
 *  selectivo por IPC habría que escribirlo igual (ADR-024).
 */

import { invoke } from "@tauri-apps/api/core";

export const LEVELS = ["trace", "debug", "info", "warn", "error", "silent"] as const;
export type LogLevel = (typeof LEVELS)[number];

const ORDER: Record<LogLevel, number> = { trace: 0, debug: 1, info: 2, warn: 3, error: 4, silent: 5 };

/** `info` por defecto (constitución §XV). El backend lo ajusta al arrancar con el valor efectivo,
 *  que puede venir de `--log-level` o de `settings`. */
let current: LogLevel = "info";

/** Lo fija el arranque con el nivel que resuelva el backend. No se lee de variables de entorno:
 *  el principio XII lo prohíbe y aquí no hay servidor donde vivirían. */
export function setLogLevel(level: LogLevel): void {
  current = level;
}

export function getLogLevel(): LogLevel {
  return current;
}

/** Contexto de una entrada: pares clave=valor, nunca frases con datos incrustados.
 *
 *  **Nunca se pasan aquí** números de serie, nombre de equipo o de usuario, rutas con perfil ni
 *  etiquetas de volumen (§XV). Para identificar un disco se usa su identificador interno, que no
 *  significa nada fuera de esta instalación. El log viaja dentro del ZIP de diagnóstico. */
export type LogContext = Record<string, string | number | boolean | null>;

function enabled(level: Exclude<LogLevel, "silent">): boolean {
  return ORDER[level] >= ORDER[current];
}

/** Envía al backend para que quede en el fichero. Si falla, se ignora a propósito: un registro que
 *  tumba la interfaz es peor que no tener registro. */
function forward(level: "warn" | "error", scope: string, message: string, context?: LogContext): void {
  void invoke("log_from_ui", { level, scope, message, context: context ?? null }).catch(() => undefined);
}

function emit(
  level: Exclude<LogLevel, "silent">,
  scope: string,
  message: string,
  context?: LogContext
): void {
  if (!enabled(level)) return;

  const prefix = `[${scope}]`;
  // Este es el único sitio del proyecto donde `console` es legítimo: es precisamente el módulo que
  // lo encapsula para que nadie más tenga que usarlo (constitución §XV). El verificador de
  // fronteras lo exceptúa por ruta, no por comentario, para que la excepción sea una y verificable.
  const sink = level === "error" ? console.error : level === "warn" ? console.warn : console.log;
  if (context) sink(prefix, message, context);
  else sink(prefix, message);

  if (level === "warn" || level === "error") forward(level, scope, message, context);
}

export interface Logger {
  trace(message: string, context?: LogContext): void;
  debug(message: string, context?: LogContext): void;
  info(message: string, context?: LogContext): void;
  warn(message: string, context?: LogContext): void;
  error(message: string, context?: LogContext): void;
}

/**
 * Crea el registrador de un módulo. El `scope` identifica el origen y aparece en cada entrada:
 * sin él, un log de veinte mil líneas no se puede filtrar.
 *
 *     const log = createLogger("api");
 *     log.warn("respuesta fuera de contrato", { command: "get_devices" });
 */
export function createLogger(scope: string): Logger {
  return {
    trace: (m, c) => emit("trace", scope, m, c),
    debug: (m, c) => emit("debug", scope, m, c),
    info: (m, c) => emit("info", scope, m, c),
    warn: (m, c) => emit("warn", scope, m, c),
    error: (m, c) => emit("error", scope, m, c)
  };
}

/** Registrador general, para código que no pertenece a un módulo concreto. */
export const logger = createLogger("app");
