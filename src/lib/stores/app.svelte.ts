/** Estado de aplicación en runes de Svelte 5. Sin stores clásicos (AGENTS.md §1).
 *
 *  El estado se alimenta de los eventos que empuja el backend; las pantallas solo leen. Al montar
 *  se pide el estado completo con `get_*` y a partir de ahí se escucha (ADR-015).
 */

import type { AlertGroup, DiskSummary } from "$lib/design/types";
import type { SourceHealth } from "$lib/api";
import type { Punto } from "$lib/design/series";

class AppState {
  devices = $state<DiskSummary[]>([]);
  excluded = $state<DiskSummary[]>([]);
  alerts = $state<AlertGroup[]>([]);
  sources = $state<SourceHealth[]>([]);
  paused = $state(false);
  pausedSince = $state<string | null>(null);
  /** null mientras no se ha cargado nada todavía: no es lo mismo que "no hay discos". */
  loadedAt = $state<string | null>(null);
  /** Igual que `loadedAt`, pero para la pantalla de alertas: se carga por separado porque no toda
   *  navegación pasa antes por el panel general. */
  alertsLoadedAt = $state<string | null>(null);

  /** Caché de la serie de temperatura de 24 h por disco (v3): el panel la pide **perezosamente**
   *  por tarjeta visible tras el primer render, y se guarda aquí para no repetir la petición al
   *  re-montar filas durante el scroll. Clave: `device.id`. */
  temperatureSeries = $state<Record<string, Punto[]>>({});

  /** Reemplaza por identificador. Los eventos traen el objeto completo, no un parche. */
  upsertDevices(incoming: DiskSummary[]) {
    const byId = new Map(this.devices.map((d) => [d.id, d]));
    for (const d of incoming) byId.set(d.id, d);
    this.devices = [...byId.values()];
  }

  upsertAlerts(changed: AlertGroup[], removed: string[] = []) {
    const byId = new Map(this.alerts.map((a) => [a.id, a]));
    for (const a of changed) byId.set(a.id, a);
    for (const id of removed) byId.delete(id);
    this.alerts = [...byId.values()];
  }
}

export const app = new AppState();
