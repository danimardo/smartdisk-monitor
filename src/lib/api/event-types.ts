/** Tipos de las cargas útiles de eventos, **inferidos de los esquemas** (constitución §XI).
 *
 *  No se declaran a mano: dos declaraciones de la misma forma acaban divergiendo, y la que manda
 *  es la que valida en ejecución. Este fichero existe solo para dar nombres legibles a los tipos
 *  que `z.infer<>` produce.
 */

import type { z } from "zod";
import type {
  alertsChanged,
  inventoryChanged,
  metricsUpdated,
  monitoringPaused,
  sourceDegraded,
  testProgress
} from "./schemas";

export type MetricsUpdated = z.infer<typeof metricsUpdated>;
export type AlertsChanged = z.infer<typeof alertsChanged>;
export type InventoryChanged = z.infer<typeof inventoryChanged>;
export type TestProgress = z.infer<typeof testProgress>;
export type SourceDegraded = z.infer<typeof sourceDegraded>;
export type MonitoringPaused = z.infer<typeof monitoringPaused>;
