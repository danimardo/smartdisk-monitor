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
export async function instalarIpcFalso(page: Page, respuestas: Record<string, unknown>) {
  await page.addInitScript((tabla: Record<string, unknown>) => {
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
      // El plugin de eventos habla por el mismo canal. Sin esto, `subscribe()` del layout raíz
      // rechazaría y la aplicación arrancaría en estado de error.
      if (cmd === "plugin:event|listen") {
        const a = args as { event: string; handler: number };
        if (!oyentes.has(a.event)) oyentes.set(a.event, []);
        oyentes.get(a.event)!.push(a.handler);
        return Promise.resolve(a.handler);
      }
      if (cmd === "plugin:event|unlisten" || cmd === "plugin:event|emit") return Promise.resolve(null);
      return Promise.resolve(cmd in tabla ? tabla[cmd] : null);
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
  }, respuestas);
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
