import type { HealthState, Severity, AlertStatus, UnknownReason } from "./types";

/** Único mapa autorizado de estado → token de color. Ningún componente decide colores por su cuenta. */
export const healthToken: Record<HealthState, { fg: string; soft: string; labelKey: string }> = {
  ok: { fg: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)", labelKey: "health.ok" },
  warn: { fg: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)", labelKey: "health.warn" },
  crit: { fg: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)", labelKey: "health.crit" },
  unknown: { fg: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)", labelKey: "health.unknown" }
};

export const severityToHealth: Record<Severity, HealthState> = {
  info: "unknown",
  warn: "warn",
  crit: "crit"
};

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

/** Estado **presentable** de un disco en las tarjetas, el Hero y el reparto: su `state` de SMART
 *  (frescura), elevado a la peor severidad de sus alertas `active`/`acknowledged`.
 *
 *  Existe porque `B.1` (`docs/open-questions.md`) está a medio conectar: el backend nunca funde las
 *  alertas en `DiskSummary.state` (`enrich_with_smart_data` siempre pasa `None` a `device_state`),
 *  así que se hace aquí —que es donde `B.1` dijo que vivía—, una sola vez, y todo lo que lee
 *  `disk.state` en el panel pasa antes por esta función.
 *
 *  Cuentan tanto las alertas dirigidas al **dispositivo** (`…|device:<id>`) como a un **volumen
 *  suyo** (`…|volume:<id>`): un volumen lleno es un problema del disco que lo contiene, no una
 *  categoría aparte.
 *
 *  **Un `unknown` se queda `unknown`**, sea cual sea el motivo: un disco sin datos SMART se
 *  presenta como «sin datos SMART» (gris), no como advertencia, aunque haya dejado de responder.
 *  Que eso cuente para «N necesitan atención» y el color de la bandeja lo decide aparte
 *  `estadoParaRecuento`; el color de la tarjeta no. */
export function estadoConAlertas(
  disk: {
    id: string;
    state: HealthState;
    volumes?: readonly { id: string }[];
  },
  alerts: readonly { severity: Severity; status: AlertStatus; deduplicationKey: string }[]
): HealthState {
  const suyas = alerts.filter(
    (a) =>
      alertCountsTowardHealth(a.status) &&
      (a.deduplicationKey.includes(`device:${disk.id}`) ||
        (disk.volumes ?? []).some((v) => a.deduplicationKey.includes(`volume:${v.id}`)))
  );
  if (suyas.some((a) => a.severity === "crit")) return "crit";
  if (suyas.some((a) => a.severity === "warn")) return "warn";
  return disk.state;
}

/** Estado de un disco **solo para el recuento global** (píldora de la `Toolbar`, pie del riel,
 *  icono de la bandeja): como `estadoConAlertas`, pero además un `unknown` por `unreadable` o
 *  `collector-error` cuenta como advertencia (`unknownContributesWarning`, §B.5) —una fuente que
 *  debería funcionar y no funciona es una degradación real—, salvo con la monitorización en pausa,
 *  donde el estado de pausa manda. **No** se usa para pintar tarjetas: ahí un `unknown` es gris. */
export function estadoParaRecuento(
  disk: {
    id: string;
    state: HealthState;
    unknownReason?: UnknownReason | null;
    volumes?: readonly { id: string }[];
  },
  alerts: readonly { severity: Severity; status: AlertStatus; deduplicationKey: string }[],
  opts: { paused?: boolean } = {}
): HealthState {
  const s = estadoConAlertas(disk, alerts);
  if (
    s === "unknown" &&
    !opts.paused &&
    disk.unknownReason != null &&
    unknownContributesWarning(disk.unknownReason)
  ) {
    return "warn";
  }
  return s;
}

/** Severidad máxima de una lista. `unknown` no gana nunca a un estado conocido:
 *  se usa solo cuando no hay ningún estado conocido que mostrar. */
export function worstState(states: readonly HealthState[]): HealthState {
  if (states.includes("crit")) return "crit";
  if (states.includes("warn")) return "warn";
  if (states.includes("ok")) return "ok";
  return "unknown";
}

/** Estado global de la aplicación, calculado **una sola vez** y presentado en dos sitios: la píldora
 *  de la `Toolbar` (con texto) y el pie del riel de la `Sidebar` (solo icono). Al ser la misma
 *  función, no pueden contradecirse (`docs/ui-design.md` §7, `09-chrome-y-estados.md`).
 *
 *  `kind` distingue los casos que la interfaz rotula distinto; `state` es el token de color; `count`
 *  es cuántos discos monitorizados necesitan atención. La `Sidebar`/`Toolbar` traducen `kind` a texto
 *  con `t()` — aquí no hay literales de interfaz. */
export type GlobalStatusKind = "loading" | "paused" | "noDevices" | "ok" | "attention";

export function globalStatus(input: {
  /** false mientras el inventario no ha llegado: NO es lo mismo que "no hay discos". */
  loaded: boolean;
  paused: boolean;
  /** Estados de los discos **monitorizados** (los excluidos no cuentan). */
  monitoredStates: readonly HealthState[];
}): { kind: GlobalStatusKind; state: HealthState; count: number } {
  if (!input.loaded) return { kind: "loading", state: "unknown", count: 0 };
  if (input.paused) return { kind: "paused", state: "unknown", count: 0 };
  if (input.monitoredStates.length === 0) return { kind: "noDevices", state: "unknown", count: 0 };
  const count = input.monitoredStates.filter((s) => s === "warn" || s === "crit").length;
  if (count === 0) return { kind: "ok", state: "ok", count: 0 };
  return { kind: "attention", state: worstState(input.monitoredStates), count };
}

/** El disco que protagoniza el `HeroPanel` del panel general (v3, `HeroPanel.md`). **La pantalla
 *  elige, no el componente**, y este es el criterio:
 *
 *   1. el disco con la alerta que cuenta para la salud (`active`/`acknowledged`) de mayor severidad;
 *      empate → la de ocurrencia más reciente;
 *   2. si no hay ninguna, el disco que no está sano —crit, luego warn, luego un `unknown` que
 *      cuenta como degradación (`unreadable`/`collector-error`)—, para que el protagonista nunca
 *      sea un disco sano habiendo uno con problema, aunque venga de un volumen lleno o de un SMART
 *      que dejó de responder y no de una alerta de dispositivo; empate → orden de inventario;
 *   3. si todos van bien, el disco cuyo volumen sea el de sistema (`isSystemVolume`);
 *   4. si no se sabe, el primero del inventario;
 *   5. **un disco sin SMART (`unknown` por `unsupported`) nunca protagoniza**, salvo que sea el único.
 *
 *  `state` debe venir ya con las alertas fundidas (`estadoConAlertas`). Devuelve `null` solo si no
 *  hay ningún disco. */
export function selectHeroDisk<
  D extends {
    id: string;
    state: HealthState;
    unknownReason?: UnknownReason | null;
    volumes?: readonly { isSystemVolume?: boolean }[];
  }
>(
  disks: readonly D[],
  alerts: readonly { severity: Severity; status: AlertStatus; deduplicationKey: string; lastOccurredAt: string }[]
): D | null {
  if (disks.length === 0) return null;

  const sinSmart = (d: D) => d.state === "unknown" && (d.unknownReason ?? "unsupported") === "unsupported";
  const elegibles = disks.some((d) => !sinSmart(d)) ? disks.filter((d) => !sinSmart(d)) : disks;

  const sev: Record<Severity, number> = { crit: 3, warn: 2, info: 1 };
  const puntuados: { disk: D; sev: number; when: string }[] = [];
  for (const d of elegibles) {
    const suyas = alerts.filter(
      (a) => alertCountsTowardHealth(a.status) && a.deduplicationKey.includes(`device:${d.id}`)
    );
    if (!suyas.length) continue;
    let mejorSev = 0;
    let mejorWhen = "";
    for (const a of suyas) {
      if (sev[a.severity] > mejorSev || (sev[a.severity] === mejorSev && a.lastOccurredAt > mejorWhen)) {
        mejorSev = sev[a.severity];
        mejorWhen = a.lastOccurredAt;
      }
    }
    puntuados.push({ disk: d, sev: mejorSev, when: mejorWhen });
  }

  if (puntuados.length) {
    puntuados.sort((a, b) => b.sev - a.sev || b.when.localeCompare(a.when));
    return puntuados[0].disk;
  }

  // Sin alerta de dispositivo, pero el `state` ya trae fundidas las alertas de volumen: si algún
  // disco no está sano —o dejó de responder a SMART—, protagoniza él, no el de sistema.
  const conProblema =
    elegibles.find((d) => d.state === "crit") ??
    elegibles.find((d) => d.state === "warn") ??
    elegibles.find(
      (d) =>
        d.state === "unknown" &&
        d.unknownReason != null &&
        unknownContributesWarning(d.unknownReason)
    );
  if (conProblema) return conProblema;

  return elegibles.find((d) => d.volumes?.some((v) => v.isSystemVolume)) ?? elegibles[0];
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

/** Umbrales efectivos de temperatura para un disco (`docs/alert-rules.md`,
 *  `temp.above_vendor_limit`/`_critical` y `temp.above_configured_warn`/`_crit`): el límite del
 *  fabricante manda si existe; a falta de él, el configurado en `settings.alerts` es el respaldo. */
export function temperatureThresholds(
  vendorLimitC: number | null | undefined,
  vendorCriticalC: number | null | undefined,
  configuredWarnC: number,
  configuredCritC: number
): { warn: number; crit: number } {
  return { warn: vendorLimitC ?? configuredWarnC, crit: vendorCriticalC ?? configuredCritC };
}

/** Estado de salud de una lectura frente a un par de umbrales aviso/crítico. Mismo criterio de
 *  operadores que `alert-rules.md`: el aviso es estrictamente por encima, el crítico llega igual. */
export function classifyAgainstThresholds(
  value: number | null,
  warn: number,
  crit: number
): HealthState {
  if (value === null) return "unknown";
  if (value >= crit) return "crit";
  if (value > warn) return "warn";
  return "ok";
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

/** Color de la **barra** de ocupación de un volumen (no de la alerta). Imita al Explorador de
 *  Windows: rojo cuando queda poco espacio. Umbrales del boceto (`smartdisk-v3.html`,
 *  `bar = u => u >= 91 ? crit : u >= 85 ? warn : ok`): ≥ 91 % ocupado → rojo, ≥ 85 % → ámbar, por
 *  debajo → verde. La regla de alerta `capacity.*` es otra cosa y la decide `capacityState()`. */
export function capacityBarTone(usedPercent: number | null): HealthState {
  if (usedPercent === null) return "unknown";
  if (usedPercent >= 91) return "crit";
  if (usedPercent >= 85) return "warn";
  return "ok";
}
