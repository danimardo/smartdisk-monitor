/** Suscripción a los eventos que empuja el backend.
 *
 *  **La interfaz no hace sondeo** (ADR-015): no hay `setInterval` pidiendo datos. El planificador
 *  del backend ya sabe cuándo hay algo nuevo, así que lo emite. Preguntar cada pocos segundos
 *  duplicaría el reloj, gastaría batería y garantizaría que lo mostrado va siempre un poco por
 *  detrás de la realidad.
 *
 *  **Cada carga útil se valida contra su esquema antes de tocar el estado** (constitución §XI).
 *  `listen<T>()` es una aserción de tipo sobre un dato que cruza una frontera de proceso: sin
 *  validar, un cambio de contrato en el backend llegaría a la interfaz como `undefined` disfrazado
 *  de dato bueno. Un evento con forma inesperada se descarta y se informa; no se aplica a medias.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { createLogger } from "$lib/logger";
import { eventSchemas, type EventName, type EventPayload } from "./schemas";

const log = createLogger("ipc");

export type { EventName, EventPayload } from "./schemas";
export type {
  MetricsUpdated,
  AlertsChanged,
  InventoryChanged,
  TestProgress,
  ReportProgress,
  SourceDegraded,
  MonitoringPaused
} from "./event-types";

/** Se avisa por consola y se descarta: un evento malformado no puede corromper el estado, y
 *  tampoco debe tumbar la aplicación. El siguiente ciclo traerá datos nuevos. */
function onInvalid(event: string, issues: { path: PropertyKey[]; message: string }[]): void {
  const detalle = issues
    .slice(0, 3)
    .map((i) => `${i.path.join(".") || "(raíz)"}: ${i.message}`)
    .join("; ");
  log.warn("evento descartado por no cumplir su esquema", { event, detalle });
}

/** Escucha un evento tipado y validado. Devuelve la función para dejar de escuchar. */
export function on<K extends EventName>(
  event: K,
  handler: (payload: EventPayload<K>) => void
): Promise<UnlistenFn> {
  const schema = eventSchemas[event];
  return listen<unknown>(event, (e) => {
    const result = schema.safeParse(e.payload);
    if (!result.success) {
      onInvalid(event, result.error.issues);
      return;
    }
    handler(result.data as EventPayload<K>);
  });
}

/**
 * Suscribe varios eventos a la vez y devuelve una sola función de limpieza.
 * Pensado para el `onMount` del layout: suscribir en un sitio y soltar todo junto al desmontar.
 */
export async function subscribe(handlers: {
  [K in EventName]?: (payload: EventPayload<K>) => void;
}): Promise<UnlistenFn> {
  const entries = Object.entries(handlers) as [EventName, (p: never) => void][];
  const unlisteners = await Promise.all(entries.map(([event, handler]) => on(event, handler as never)));
  return () => unlisteners.forEach((u) => u());
}
