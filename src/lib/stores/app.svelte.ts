/** Estado de aplicación en runes de Svelte 5. Sin stores clásicos (AGENTS.md §1).
 *
 *  El estado se alimenta de los eventos que empuja el backend; las pantallas solo leen. Al montar
 *  se pide el estado completo con `get_*` y a partir de ahí se escucha (ADR-015).
 */

import type { AlertGroup, DiskSummary } from "$lib/design/types";
import type { SourceHealth } from "$lib/api";
import type { Punto } from "$lib/design/series";

/** Ventana que cubre la onda de actividad de fondo de la `DiskCard` (ADR-051). Corta a propósito:
 *  la gráfica es «¿ha estado ocupado este disco ahora mismo?», no un histórico. Con la cadencia de
 *  métricas rápidas (30 s de fábrica) son ~10 puntos. */
export const VENTANA_ACTIVIDAD_TARJETA_MS = 5 * 60 * 1000;

/** Dos muestras a menos de esto se consideran la misma lectura: un ciclo de SMART completo reemite
 *  `metrics:updated` sin que la ventana de actividad haya avanzado, y no debe añadir un punto. */
const DEDUP_ACTIVIDAD_MS = 15 * 1000;

/** Ordena por tiempo, descarta puntos demasiado juntos y recorta a la ventana relativa al punto
 *  más reciente. Compartida por la siembra (`seedActivitySeries`) y el refresco en vivo
 *  (`pushActivitySamples`). */
function normalizarSerieActividad(puntos: Punto[]): Punto[] {
  const ordenados = [...puntos].sort((a, b) => a.t - b.t);
  const espaciados: Punto[] = [];
  for (const p of ordenados) {
    const ultimo = espaciados.at(-1);
    if (!ultimo || p.t - ultimo.t >= DEDUP_ACTIVIDAD_MS) espaciados.push(p);
  }
  const fin = espaciados.at(-1)?.t;
  if (fin === undefined) return espaciados;
  return espaciados.filter((p) => p.t >= fin - VENTANA_ACTIVIDAD_TARJETA_MS);
}

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

  /** Caché de la serie de temperatura de 24 h por disco: la usa **solo el `HeroPanel`**, que la
   *  pide perezosamente para el disco protagonista y la guarda aquí. Clave: `device.id`. */
  temperatureSeries = $state<Record<string, Punto[]>>({});

  /** Onda de actividad de fondo de cada `DiskCard` (ADR-051). Se **siembra** una vez por tarjeta
   *  visible con `get_metric_series("activity_percent")` y a partir de ahí se **refresca en vivo**
   *  apilando el valor de cada evento `metrics:updated` (sin sondeo, ADR-015). Clave: `device.id`. */
  activitySeries = $state<Record<string, Punto[]>>({});

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

  /** Punto de arranque de la onda de una tarjeta: fusiona la serie persistida con lo que ya haya
   *  llegado en vivo (el layout monta antes que la página, así que puede haber puntos previos). */
  seedActivitySeries(deviceId: string, puntos: Punto[]) {
    const fusion = normalizarSerieActividad([...(this.activitySeries[deviceId] ?? []), ...puntos]);
    this.activitySeries = { ...this.activitySeries, [deviceId]: fusion };
  }

  /** Añade un punto por disco desde un evento `metrics:updated`. `v` es la media de la ventana
   *  deslizante; es `null` solo cuando `estado === "no_disponible"` (fuente degradada o recién
   *  arrancada) → hueco en la onda. Con `parcial` sí se pinta: la onda de fondo de la tarjeta es
   *  contexto, no lectura (el cuerpo ya marca `~` la cifra parcial), y una máquina con la ventana de
   *  actividad siempre `parcial` no debe quedarse sin onda. */
  pushActivitySamples(devices: DiskSummary[], emittedAt: string) {
    const t = Date.parse(emittedAt);
    if (!Number.isFinite(t)) return;
    const siguiente = { ...this.activitySeries };
    for (const d of devices) {
      const previa = siguiente[d.id] ?? [];
      const ultimo = previa.at(-1);
      if (ultimo && t - ultimo.t < DEDUP_ACTIVIDAD_MS) continue;
      siguiente[d.id] = normalizarSerieActividad([...previa, { t, v: d.activity.mediaPercent }]);
    }
    this.activitySeries = siguiente;
  }

  /** Descarta las ondas de discos que ya no están presentes (tras `inventory:changed`). */
  prunearSeriesActividad() {
    const vivos = new Set(this.devices.map((d) => d.id));
    this.activitySeries = Object.fromEntries(
      Object.entries(this.activitySeries).filter(([id]) => vivos.has(id))
    );
  }
}

export const app = new AppState();
