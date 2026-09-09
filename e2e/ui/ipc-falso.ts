import type { Page } from "@playwright/test";

/** Instala un IPC de Tauri falso **antes** de que cargue el script de la aplicación.
 *
 *  No se usa `mockIPC()` de `@tauri-apps/api/mocks` directamente porque esa función vive en el
 *  proceso de Node y aquí hace falta que el doble exista **dentro de la página**, antes del primer
 *  `invoke`. Lo que se hace es lo mismo que hace `mockIPC` —poblar `window.__TAURI_INTERNALS__`—
 *  pero mediante `addInitScript`, que Playwright ejecuta en cada documento nuevo antes que nada.
 *
 *  Las llamadas quedan registradas en `window.__llamadas__` para poder afirmar sobre ellas.
 */
export async function instalarIpcFalso(
  page: Page,
  respuestas: Record<string, unknown>,
  opciones: { retardoMs?: Record<string, number> } = {}
) {
  await page.addInitScript(
    ({ tabla, retardos }: { tabla: Record<string, unknown>; retardos: Record<string, number> }) => {
      const w = window as unknown as Record<string, unknown>;
      const llamadas: { comando: string; args: unknown }[] = [];
      w.__llamadas__ = llamadas;

      const oyentes = new Map<string, number[]>();
      const callbacks = new Map<number, (dato: unknown) => void>();

      function registrarCallback(cb: (dato: unknown) => void, once = false) {
        const id = window.crypto.getRandomValues(new Uint32Array(1))[0];
        callbacks.set(id, (dato) => {
          if (once) callbacks.delete(id);
          return cb?.(dato);
        });
        return id;
      }

      // Devuelve una promesa sin ser `async`: el `invoke` real de Tauri lo es, así que el contrato
      // tiene que coincidir, pero aquí no hay nada que esperar y marcarla `async` solo serviría para
      // tener que silenciar una regla del análisis estático.
      function invoke(cmd: string, args: unknown): Promise<unknown> {
        llamadas.push({ comando: cmd, args });

        // La marca del asistente inicial (US-002) es lo único que el guardián de `+layout.ts` vuelve
        // a leer tras escribirla: se refleja en `get_settings` para que un `goto("/")` después de
        // terminar el asistente no rebote otra vez a `/onboarding`.
        if (cmd === "set_setting") {
          const a = args as { key?: string; value?: unknown };
          const s = tabla.get_settings as { onboarding?: { completedAt: unknown } } | undefined;
          if (a.key === "settings.onboarding.completed_at" && s && typeof s === "object") {
            s.onboarding = { completedAt: a.value ?? null };
          }
        }
        // El plugin de eventos habla por el mismo canal. Sin esto, `subscribe()` del layout raíz
        // rechazaría y la aplicación arrancaría en estado de error.
        if (cmd === "plugin:event|listen") {
          const a = args as { event: string; handler: number };
          if (!oyentes.has(a.event)) oyentes.set(a.event, []);
          oyentes.get(a.event)!.push(a.handler);
          return Promise.resolve(a.handler);
        }
        if (cmd === "plugin:event|unlisten" || cmd === "plugin:event|emit") return Promise.resolve(null);
        // `get_metric_series` se pide con `metricKey` distinto para la misma pantalla (temperatura
        // de fondo del Hero, actividad de la DiskCard). Un fixture `get_metric_series:<metricKey>`
        // gana al genérico; así una serie puede ser una onda de 10 h y otra de 5 min.
        const clave =
          cmd === "get_metric_series" &&
          `get_metric_series:${(args as { metricKey?: string } | null)?.metricKey}` in tabla
            ? `get_metric_series:${(args as { metricKey?: string }).metricKey}`
            : cmd;
        const valor = clave in tabla ? tabla[clave] : null;
        // Un valor `{ __rechazar__: AppError }` hace que el comando **rechace** con ese error, para
        // probar el manejo de fallos (spec 005: la explicación con IA falla y la pantalla sigue).
        // Se envuelve en un `Error` con los campos del `AppError` copiados encima: `toAppError` de
        // `$lib/api` lo reconoce por `code`/`messageKey` igual que reconoce el objeto que devuelve
        // el `invoke` real de Tauri.
        if (valor && typeof valor === "object" && "__rechazar__" in valor) {
          const motivo = (valor as Record<string, unknown>).__rechazar__ as Record<string, unknown>;
          const codigo = typeof motivo.code === "string" ? motivo.code : "ipc";
          return Promise.reject(Object.assign(new Error(codigo), motivo));
        }
        // Retardo opcional por comando: para probar el indicador de navegación, que solo aparece si
        // el `load` de la ruta tarda más de 150 ms (spec 004).
        const ms = retardos[cmd];
        if (ms) return new Promise((r) => setTimeout(() => r(valor), ms));
        return Promise.resolve(valor);
      }

      w.__TAURI_INTERNALS__ = {
        invoke,
        transformCallback: registrarCallback,
        unregisterCallback: (id: number) => callbacks.delete(id),
        runCallback: (id: number, dato: unknown) => callbacks.get(id)?.(dato),
        callbacks
      };
      w.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        unregisterListener: (_e: string, id: number) => callbacks.delete(id)
      };

      // Expuesto para que `emitirEvento()` (lado Playwright) pueda simular un evento del backend
      // sin reimplementar el registro de oyentes.
      w.__emitirEvento__ = (event: string, payload: unknown) => {
        for (const id of oyentes.get(event) ?? []) callbacks.get(id)?.({ event, id, payload });
      };
    },
    { tabla: respuestas, retardos: opciones.retardoMs ?? {} }
  );
}

/** Simula que el backend emite `event` con `payload`, para las mismas suscripciones que instaló
 *  `subscribe()` del layout raíz. Sirve para medir el coste de una actualización en caliente
 *  (SC-007), no solo el de la carga inicial. */
export function emitirEvento(page: Page, event: string, payload: unknown): Promise<void> {
  return page.evaluate(
    ([e, p]) =>
      (window as unknown as { __emitirEvento__: (e: string, p: unknown) => void }).__emitirEvento__(e, p),
    [event, payload] as [string, unknown]
  );
}

/** Comandos que la aplicación ha invocado, en orden. */
export function llamadas(page: Page): Promise<{ comando: string; args: unknown }[]> {
  return page.evaluate(
    () => (window as unknown as { __llamadas__: { comando: string; args: unknown }[] }).__llamadas__
  );
}
