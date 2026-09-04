import * as S from "../../../src/lib/api/schemas";

/** Respuestas del backend para el plano de interfaz.
 *
 *  Se validan contra los **mismos esquemas Zod** que usa la aplicación en tiempo de ejecución
 *  (`validar()`, más abajo). Sin eso, un fixture podría quedarse desfasado del contrato y las
 *  pruebas seguirían en verde probando una forma de datos que ya no existe — que es exactamente
 *  la clase de fallo silencioso que el principio XI trata de impedir.
 */

const AHORA = "2026-09-04T10:00:00Z";

export const apariencia = {
  theme: "light",
  language: "es",
  systemLocale: "es-ES",
  useSystemAccent: true
};

export const acento = { hex: "#0067c0", palette: ["#0067c0"] };

/** Un NVMe sano y un USB sin SMART. El segundo importa: comprueba que «no compatible» se pinta en
 *  gris y no en rojo, que es la confusión que la especificación §12 prohíbe expresamente. */
export const inventario = {
  devices: [
    {
      id: "disk-0",
      alias: null,
      model: "Samsung SSD 990 PRO 2TB",
      deviceType: "nvme",
      state: "ok",
      temperatureC: 41,
      percentageUsed: 3,
      activityPercent: 12,
      powerOnHours: 1840,
      lastReadAt: AHORA,
      volumes: [
        {
          id: "vol-c",
          label: "Sistema",
          driveLetters: ["C:"],
          capacityBytes: 2_000_398_934_016,
          freeBytes: 1_204_000_000_000,
          mappingConfidence: "exact"
        }
      ]
    },
    {
      id: "disk-1",
      alias: "Copia externa",
      model: "WD Elements 25A3",
      deviceType: "usb",
      state: "unknown",
      temperatureC: null,
      percentageUsed: null,
      activityPercent: null,
      powerOnHours: null,
      unknownReason: "unsupported",
      lastReadAt: AHORA,
      volumes: []
    }
  ],
  excluded: [],
  sources: [
    { source: "smartctl", status: "ok", lastSuccessAt: AHORA, lastAttemptAt: AHORA },
    { source: "windows-storage", status: "ok", lastSuccessAt: AHORA, lastAttemptAt: AHORA }
  ],
  paused: false,
  pausedSince: null
};

/** Comando → respuesta. Lo que no esté aquí devuelve `null`, y la pantalla debe aguantarlo. */
export const RESPUESTAS: Record<string, unknown> = {
  get_appearance_settings: apariencia,
  get_system_accent_color: acento,
  get_devices: inventario,
  get_alert_groups: { groups: [], total: 0 },
  get_log_level: { level: "info" }
};

/** Falla en cuanto un fixture deja de cumplir el contrato, no cuando una pantalla se rompe. */
export function validar(): void {
  S.appearanceSettings.parse(apariencia);
  S.windowsAccent.parse(acento);
  S.deviceListResponse.parse(inventario);
}
