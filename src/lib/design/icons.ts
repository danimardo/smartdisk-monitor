/** Juego de iconos de línea (v3, ADR-034) y sus mapas semánticos.
 *
 *  Los SVG viven en un sprite montado una sola vez en `AppShell`; aquí solo está el vocabulario
 *  (`IconName`) y la única fuente de verdad de «qué icono significa qué». Ningún componente decide
 *  un icono por su cuenta, igual que ninguno decide un color (`health.ts`).
 *
 *  Regla de accesibilidad (la aplica `Icon.svelte`): si el icono es el único portador de un
 *  significado, `label` es obligatorio; si acompaña a un texto que ya lo dice, va sin `label` y
 *  queda `aria-hidden`. */

import type { HealthState } from "./types";

/** Los 15 símbolos del sprite. El `id` en el sprite es `i-{name}`. */
export type IconName =
  | "temp"
  | "wear"
  | "pulse"
  | "clock"
  | "nvme"
  | "hdd"
  | "usb"
  | "shield"
  | "alert"
  | "bolt"
  | "flask"
  | "plug"
  | "diskStack"
  | "check"
  | "tag";

/** Todos los `IconName`, para pruebas y para validar el sprite. */
export const ICON_NAMES: readonly IconName[] = [
  "temp",
  "wear",
  "pulse",
  "clock",
  "nvme",
  "hdd",
  "usb",
  "shield",
  "alert",
  "bolt",
  "flask",
  "plug",
  "diskStack",
  "check",
  "tag"
];

/** Estado de salud → icono. `unknown` usa `usb` porque el 90 % de los casos reales son puentes USB
 *  que no exponen SMART; cuando el motivo sea otro (RAID, disco virtual, recopilador en pausa) la
 *  pantalla pasa el icono explícito. */
export const healthIcon: Record<HealthState, IconName> = {
  ok: "shield",
  warn: "alert",
  crit: "bolt",
  unknown: "usb"
};

/** Nivel de un evento de Windows → icono. Mismas claves que `EventRow`. */
export const eventLevelIcon: Record<"error" | "warning" | "info", IconName> = {
  error: "bolt",
  warning: "alert",
  info: "shield"
};

/** Severidad de un grupo de alerta → icono (`AlertCard`, `HeroPanel`). `info` no es `unknown`:
 *  aquí sí lleva `shield`, no el icono USB de «sin datos». */
export const severityIcon: Record<"info" | "warn" | "crit", IconName> = {
  info: "shield",
  warn: "alert",
  crit: "bolt"
};

/** Tipo de dispositivo (texto libre del inventario) → icono de bus. NVMe y SATA SSD comparten
 *  `nvme`; solo se distingue el mecánico y el externo. */
export function busIcon(deviceType: string): IconName {
  if (/usb/i.test(deviceType)) return "usb";
  if (/hdd|spindle|rpm|mechanical/i.test(deviceType)) return "hdd";
  return "nvme";
}

/** Tipo de prueba (`test_runs.type`) → icono. Mismas claves que el esquema Zod `type`. */
export const testIcon: Record<"benchmark" | "chkdsk_scan" | "smart_short", IconName> = {
  benchmark: "flask",
  chkdsk_scan: "shield",
  smart_short: "bolt"
};
