import { beforeEach, describe, expect, it } from "vitest";
import { i18n } from "$lib/i18n";
import {
  deviceLabel,
  formatAge,
  formatBytes,
  formatHours,
  formatLatency,
  formatPercent,
  formatSpanShort,
  formatTemperature,
  formatThroughput,
  maskSerial,
  usedPercent
} from "./format";

/** El principio I de la constitución dice que nunca se inventa un valor. Estas pruebas lo fijan
 *  para cada formateador: un dato ausente sale como "No disponible", nunca como cero. */

describe("un dato ausente nunca se convierte en cero", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it.each([
    ["formatBytes", formatBytes],
    ["formatTemperature", formatTemperature],
    ["formatPercent", formatPercent],
    ["formatHours", formatHours],
    ["formatThroughput", formatThroughput],
    ["formatLatency", formatLatency],
    ["formatSpanShort", formatSpanShort]
  ])("%s devuelve 'No disponible' con null, undefined y NaN", (_name, fn) => {
    expect(fn(null)).toBe("No disponible");
    expect(fn(undefined)).toBe("No disponible");
    expect(fn(Number.NaN)).toBe("No disponible");
  });

  it("un cero real sí se muestra: 0 no es lo mismo que ausente", () => {
    expect(formatPercent(0)).toBe("0 %");
    expect(formatTemperature(0)).toBe("0 °C");
  });
});

describe("formatBytes — base 1024 con etiquetas de Windows", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("usa base binaria, como el Explorador de Windows", () => {
    // 1 KB son 1024 B, no 1000: es la convención de Windows, con la que el usuario podrá contrastar.
    expect(formatBytes(1024)).toBe("1 KB");
    expect(formatBytes(1024 ** 2)).toBe("1,0 MB");
    expect(formatBytes(1024 ** 3)).toBe("1,00 GB");
  });

  it("no salta de unidad antes de tiempo", () => {
    expect(formatBytes(1023)).toBe("1023 B");
  });

  it("da más decimales cuanto mayor es la unidad", () => {
    // En GB y por encima, un entero escondería medio terabyte de diferencia.
    expect(formatBytes(1536 * 1024 ** 2)).toBe("1,50 GB");
    expect(formatBytes(512 * 1024)).toBe("512 KB");
  });

  it("sigue el idioma de la aplicación, no el del sistema", () => {
    i18n.init("es", "en-US");
    const es = formatBytes(1536 * 1024 ** 2);
    i18n.init("en", "en-US");
    const en = formatBytes(1536 * 1024 ** 2);
    // El separador decimal cambia con el idioma elegido.
    expect(es).not.toBe(en);
  });
});

describe("formatThroughput — recibe bytes por segundo, no MB/s", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("convierte desde bytes por segundo", () => {
    expect(formatThroughput(180 * 1024 ** 2)).toBe("180 MB/s");
  });
});

describe("formatLatency — un decimal por debajo de 10 ms", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("distingue la latencia de un NVMe de la de un disco mecánico", () => {
    expect(formatLatency(0.2)).toBe("0,2 ms");
    expect(formatLatency(4)).toBe("4,0 ms");
    expect(formatLatency(120)).toBe("120 ms");
  });
});

describe("formatSpanShort — ventana del gráfico del panel", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("elige la unidad por el tamaño del intervalo", () => {
    expect(formatSpanShort(45_000)).toBe("45 s");
    expect(formatSpanShort(6 * 60_000)).toBe("6 min");
    expect(formatSpanShort(3 * 3_600_000)).toBe("3 h");
    expect(formatSpanShort(24 * 3_600_000)).toBe("24 h");
  });

  it("un intervalo negativo es un dato imposible, no cero", () => {
    expect(formatSpanShort(-1)).toBe("No disponible");
  });
});

describe("usedPercent", () => {
  it("calcula el porcentaje ocupado", () => {
    expect(usedPercent(100, 25)).toBe(75);
  });

  it("devuelve null si falta cualquiera de los dos datos", () => {
    expect(usedPercent(null, 25)).toBeNull();
    expect(usedPercent(100, null)).toBeNull();
  });

  it("no divide por cero", () => {
    expect(usedPercent(0, 0)).toBeNull();
  });

  it("acota el resultado entre 0 y 100", () => {
    expect(usedPercent(100, 200)).toBe(0);
    expect(usedPercent(100, -50)).toBe(100);
  });
});

describe("maskSerial — anonimización de exportaciones", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("conserva los extremos para poder distinguir dos discos", () => {
    expect(maskSerial("S4EVNF0M123456")).toBe("S4EV••••56");
  });

  it("oculta por completo un número de serie corto", () => {
    expect(maskSerial("ABC123")).toBe("••••");
  });

  it("un serial ausente no revela nada", () => {
    expect(maskSerial(null)).toBe("No disponible");
  });
});

describe("deviceLabel — etiqueta corta de disco para listas que mezclan varios", () => {
  const discos = [
    { id: "disk-0", alias: "Sistema", model: "Samsung SSD 990 PRO 2TB" },
    { id: "disk-1", alias: null, model: "WD Blue 1TB" }
  ];

  it("prefiere el alias cuando la persona le puso uno", () => {
    expect(deviceLabel("disk-0", discos)).toBe("Sistema");
  });

  it("cae al modelo si no hay alias", () => {
    expect(deviceLabel("disk-1", discos)).toBe("WD Blue 1TB");
  });

  it("sin deviceId no inventa una etiqueta", () => {
    expect(deviceLabel(null, discos)).toBeUndefined();
    expect(deviceLabel(undefined, discos)).toBeUndefined();
  });

  it("un disco que ya no está en el inventario tampoco inventa una etiqueta", () => {
    expect(deviceLabel("disk-9-desconectado", discos)).toBeUndefined();
  });
});

describe("formatAge — marca de dato obsoleto", () => {
  beforeEach(() => i18n.init("es", "es-ES"));

  it("devuelve null sin fecha, para que el llamante omita la marca", () => {
    expect(formatAge(null)).toBeNull();
  });

  it("expresa la antigüedad en la unidad adecuada", () => {
    const now = Date.parse("2026-09-04T12:00:00Z");
    expect(formatAge("2026-09-04T11:59:30Z", now)).toBeTruthy();
    expect(formatAge("2026-09-04T11:30:00Z", now)).toBeTruthy();
    expect(formatAge("2026-09-01T12:00:00Z", now)).toBeTruthy();
  });
});
