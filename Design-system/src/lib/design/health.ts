import type { HealthState, Severity, AlertStatus, UnknownReason } from "./types";

/** Único mapa autorizado de estado → token de color. Ningún componente decide colores por su cuenta. */
export const healthToken: Record<HealthState, { fg: string; soft: string; labelKey: string }> = {
  ok: { fg: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)", labelKey: "health.ok" },
  warn: { fg: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)", labelKey: "health.warn" },
  crit: { fg: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)", labelKey: "health.crit" },
  unknown: { fg: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)", labelKey: "health.unknown" }
};

export const severityToHealth: Record<Severity, HealthState> = { info: "unknown", warn: "warn", crit: "crit" };

/** Estados de alerta que siguen pesando sobre el color de salud.
 *  Decisión de producto: **reconocer no cambia el color**. Reconocer saca la alerta de la lista de
 *  pendientes y le pone un distintivo, pero la condición sigue siendo real y el color no debe mentir
 *  sobre el estado del hardware. Solo `resolved` y `archived` dejan de contar.
 *  El silencio es ortogonal al estado: silencia la notificación, nunca el color. */
const COUNTS_TOWARD_HEALTH: readonly AlertStatus[] = ["active", "acknowledged"];

export function alertCountsTowardHealth(status: AlertStatus): boolean {
  return COUNTS_TOWARD_HEALTH.includes(status);
}

/** Salud presentable de un disco: la peor severidad de sus alertas no resueltas.
 *  Un disco sin alertas y sin datos SMART legibles es `unknown`, nunca `ok`: no sabemos que esté bien. */
export function deviceState(
  alerts: readonly { severity: Severity; status: AlertStatus }[],
  hasFreshData: boolean
): HealthState {
  const live = alerts.filter((a) => alertCountsTowardHealth(a.status));
  if (live.some((a) => a.severity === "crit")) return "crit";
  if (live.some((a) => a.severity === "warn")) return "warn";
  return hasFreshData ? "ok" : "unknown";
}

/** Severidad máxima de una lista. `unknown` no gana nunca a un estado conocido:
 *  se usa solo cuando no hay ningún estado conocido que mostrar. */
export function worstState(states: readonly HealthState[]): HealthState {
  if (states.includes("crit")) return "crit";
  if (states.includes("warn")) return "warn";
  if (states.includes("ok")) return "ok";
  return "unknown";
}

/** Un `unknown` que se debe a una fuente que **debería** funcionar es una degradación real y se
 *  presenta como advertencia; un `unknown` declarado por el propio dispositivo (un USB que no expone
 *  SMART) es normalidad y no ensucia el estado global. Regla derivada de spec §5
 *  ("SMART ilegible persistentemente: advertencia; 'no compatible' no genera alerta"). */
export function unknownContributesWarning(reason: UnknownReason): boolean {
  return reason === "unreadable" || reason === "collector-error";
}

/** Color del icono de la bandeja del sistema (spec §3).
 *  Prioridad: un crítico vigente manda sobre la pausa — la condición sigue siendo cierta aunque
 *  hayamos dejado de mirar; la pausa se comunica con el texto del menú, no apagando la señal. */
export function trayState(input: {
  paused: boolean;
  collectorFailure: boolean;
  monitoredStates: readonly HealthState[];
}): HealthState {
  if (input.monitoredStates.includes("crit")) return "crit";
  if (input.paused || input.collectorFailure || input.monitoredStates.length === 0) return "unknown";
  if (input.monitoredStates.includes("warn")) return "warn";
  return worstState(input.monitoredStates);
}

/** Por debajo de esta capacidad, el suelo absoluto de espacio libre no se aplica: en un volumen
 *  pequeño, 20 GB libres pueden ser un tercio del disco y marcarlo en rojo sería ruido puro.
 *  Configurable en `settings` (`alerts.capacity.absoluteFloorMinCapacityBytes`). */
export const CAPACITY_ABSOLUTE_FLOOR_MIN_BYTES = 256 * 1024 ** 3;

/** Nivel de capacidad (spec §5). Reglas:
 *  - siempre por porcentaje: <10 % advertencia, <5 % crítico;
 *  - además, en volúmenes de ≥256 GB, por valor absoluto: <20 GB advertencia, <10 GB crítico.
 *  Gana el más severo de los dos criterios. */
export function capacityState(
  freeBytes: number | null,
  capacityBytes: number | null,
  absoluteFloorMinBytes = CAPACITY_ABSOLUTE_FLOOR_MIN_BYTES
): HealthState {
  if (freeBytes === null || capacityBytes === null || capacityBytes <= 0) return "unknown";
  const pct = (freeBytes / capacityBytes) * 100;
  const GB = 1024 ** 3;
  const applyAbsolute = capacityBytes >= absoluteFloorMinBytes;

  if (pct < 5 || (applyAbsolute && freeBytes < 10 * GB)) return "crit";
  if (pct < 10 || (applyAbsolute && freeBytes < 20 * GB)) return "warn";
  return "ok";
}
