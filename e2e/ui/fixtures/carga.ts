import type { z } from "zod";
import * as S from "../../../src/lib/api/schemas";

/** Fixtures de escala (T074, SC-007/SC-009): 20 discos y 5.000 eventos, para medir que la
 *  interfaz no deja de responder más de 50 ms seguidos con el volumen máximo previsto. Vive
 *  aparte de `respuestas.ts` para no cargar cinco mil objetos en cada prueba que no los necesita.
 */

const AHORA = "2026-09-04T10:00:00Z";

export function generarInventarioDeCarga(n: number): z.infer<typeof S.deviceListResponse> {
  const devices = Array.from({ length: n }, (_, i) => ({
    id: `disk-${i}`,
    alias: null,
    model: `Disco de prueba ${i}`,
    deviceType: "nvme",
    state: "ok" as const,
    temperatureC: 35 + (i % 20),
    percentageUsed: i % 100,
    activity: {
      estado: "valido" as const,
      mediaPercent: i % 100,
      picoPercent: (i % 100) + 1,
      muestras: 30,
      ventanaSegundos: 30
    },
    powerOnHours: 1000 + i,
    lastReadAt: AHORA,
    volumes: []
  }));
  return { devices, excluded: [], sources: [], paused: false, pausedSince: null };
}

export function generarPaginaDeEventosDeCarga(n: number): z.infer<typeof S.systemEventPage> {
  const niveles = ["error", "warning", "info"] as const;
  const events = Array.from({ length: n }, (_, i) => ({
    id: `${i}`,
    occurredAt: AHORA,
    provider: i % 2 === 0 ? "disk" : "Microsoft-Windows-Ntfs",
    eventId: 50 + (i % 10),
    level: niveles[i % niveles.length],
    message: `Evento de prueba número ${i} para medir el desplazamiento`,
    deviceId: null,
    volumeId: null,
    mappingConfidence: "unknown" as const,
    hasRawXml: false
  }));
  return { events, nextCursor: null, total: n };
}
