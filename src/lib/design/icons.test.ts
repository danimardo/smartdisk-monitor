import { describe, expect, it } from "vitest";
import {
  ICON_NAMES,
  busIcon,
  eventLevelIcon,
  healthIcon,
  severityIcon,
  testIcon,
  type IconName
} from "./icons";

/** Los mapas semánticos son la única fuente de «qué icono significa qué» (como `health.ts` lo es del
 *  color). Si un componente los duplica, divergen. Estas pruebas fijan el contrato. */

describe("icons — vocabulario", () => {
  it("hay exactamente 15 símbolos y ninguno repetido", () => {
    expect(ICON_NAMES).toHaveLength(15);
    expect(new Set(ICON_NAMES).size).toBe(15);
  });

  it("todo icono de los mapas semánticos existe en el juego", () => {
    const usados: IconName[] = [
      ...Object.values(healthIcon),
      ...Object.values(eventLevelIcon),
      ...Object.values(severityIcon),
      ...Object.values(testIcon)
    ];
    for (const nombre of usados) {
      expect(ICON_NAMES).toContain(nombre);
    }
  });
});

describe("healthIcon", () => {
  it("mapea cada estado de salud a su icono", () => {
    expect(healthIcon).toEqual({ ok: "shield", warn: "alert", crit: "bolt", unknown: "usb" });
  });
});

describe("eventLevelIcon", () => {
  it("mapea nivel de evento a icono", () => {
    expect(eventLevelIcon.error).toBe("bolt");
    expect(eventLevelIcon.warning).toBe("alert");
    expect(eventLevelIcon.info).toBe("shield");
  });
});

describe("busIcon", () => {
  it.each([
    ["USB Mass Storage Device", "usb"],
    ["usb", "usb"],
    ["HDD", "hdd"],
    ["7200 rpm spindle", "hdd"],
    ["Mechanical drive", "hdd"],
    ["NVMe", "nvme"],
    ["SATA SSD", "nvme"],
    ["", "nvme"]
  ] as const)("%s → %s", (deviceType, esperado) => {
    expect(busIcon(deviceType)).toBe(esperado);
  });

  it("un puente USB sobre un disco mecánico se rotula como USB, no como HDD", () => {
    expect(busIcon("USB attached SCSI (7200rpm)")).toBe("usb");
  });
});

describe("testIcon", () => {
  it("usa las mismas claves que el esquema de tipo de prueba", () => {
    expect(testIcon).toEqual({ benchmark: "flask", chkdsk_scan: "shield", smart_short: "bolt" });
  });
});

describe("severityIcon", () => {
  it("mapea severidad de alerta a icono; `info` es `shield`, no el icono de «sin datos»", () => {
    expect(severityIcon).toEqual({ info: "shield", warn: "alert", crit: "bolt" });
  });
});
