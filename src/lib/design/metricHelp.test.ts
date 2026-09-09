import { describe, expect, it } from "vitest";
import { ayudaMetrica, veredictoMetrica } from "./metricHelp";
import es from "$lib/i18n/es.json";

/** El veredicto de una métrica **no puede contradecir** al color de la tarjeta ni a una alerta:
 *  usa los mismos umbrales (`classifyAgainstThresholds`). Se prueba en el umbral, justo por encima
 *  y justo por debajo. */

describe("veredictoMetrica — clasificación (pura, sin i18n)", () => {
  it("temperatura: normal por debajo del aviso, alta en el aviso, crítica en el crítico", () => {
    const ctx = { tempWarnC: 60, tempCritC: 70 };
    expect(veredictoMetrica("temperature", 59.9, ctx).estado).toBe("ok");
    expect(veredictoMetrica("temperature", 60.1, ctx).estado).toBe("warn");
    expect(veredictoMetrica("temperature", 70, ctx).estado).toBe("crit");
  });

  it("temperatura: el límite del fabricante manda sobre el configurado", () => {
    const v = veredictoMetrica("temperature", 56, { tempWarnC: 60, vendorLimitC: 55 });
    expect(v.estado).toBe("warn");
  });

  it("desgaste: bien por debajo del 80 %, alto en el 80, crítico en el 90; menciona lo que queda", () => {
    const ctx = { wearWarnPct: 80, wearCritPct: 90 };
    const bajo = veredictoMetrica("wear", 5, ctx);
    expect(bajo.estado).toBe("ok");
    expect(bajo.params.remaining).toBe(95);
    expect(veredictoMetrica("wear", 80.1, ctx).estado).toBe("warn");
    expect(veredictoMetrica("wear", 90, ctx).estado).toBe("crit");
  });

  it("horas encendido es informativa: siempre `ok`", () => {
    expect(veredictoMetrica("powerOnHours", 50000, {}).estado).toBe("ok");
  });

  it("actividad: `valido` → `info`, `parcial` → `parcial`, ambos `ok` (es rendimiento, no salud)", () => {
    const ventana = (estado: "valido" | "parcial") => ({
      estado,
      mediaPercent: 30,
      picoPercent: 80,
      muestras: 30,
      ventanaSegundos: 30
    });
    const val = veredictoMetrica("activity", null, {}, ventana("valido"));
    expect(val.estado).toBe("ok");
    expect(val.caso).toBe("info");
    expect(val.params).toMatchObject({ media: "30 %", pico: "80 %", ventana: 30 });

    const par = veredictoMetrica("activity", null, {}, ventana("parcial"));
    expect(par.estado).toBe("ok");
    expect(par.caso).toBe("parcial");
  });

  it("actividad sin agregado o `no_disponible`: caso `unknown`", () => {
    expect(veredictoMetrica("activity", null, {}).caso).toBe("unknown");
    expect(
      veredictoMetrica("activity", null, {}, {
        estado: "no_disponible",
        mediaPercent: null,
        picoPercent: null,
        muestras: 0,
        ventanaSegundos: 30
      }).caso
    ).toBe("unknown");
  });

  it("sin valor: estado `unknown` y caso `unknown` para cualquier métrica", () => {
    for (const m of ["temperature", "wear", "activity", "powerOnHours"] as const) {
      expect(veredictoMetrica(m, null, {}).caso).toBe("unknown");
      expect(veredictoMetrica(m, null, {}).estado).toBe("unknown");
    }
  });

  it("horas encendido: los años de funcionamiento se derivan del valor", () => {
    const v = veredictoMetrica("powerOnHours", 8766, {});
    expect(v.params.years).toBe("1.0");
  });
});

describe("ayudaMetrica — texto ya traducido", () => {
  it("junta explicación y veredicto, y todas las claves existen en el diccionario", () => {
    const a = ayudaMetrica("wear", 5, { wearWarnPct: 80, wearCritPct: 90 });
    expect(a.titulo).toBe(es["disk.wear"]);
    expect(a.texto).toContain(es["metric.help.wear.body"]);
    expect(a.texto).toContain("95"); // el veredicto interpolado
    expect(a.estado).toBe("ok");
  });

  it("todas las combinaciones métrica × caso tienen su clave en es.json", () => {
    const dict = es as Record<string, string>;
    for (const m of ["temperature", "wear"] as const) {
      expect(dict[`metric.help.${m}.body`]).toBeTruthy();
      for (const c of ["ok", "warn", "crit", "unknown"]) {
        expect(dict[`metric.help.${m}.verdict.${c}`], `${m}.${c}`).toBeTruthy();
      }
    }
    for (const m of ["activity", "powerOnHours"] as const) {
      expect(dict[`metric.help.${m}.body`]).toBeTruthy();
      expect(dict[`metric.help.${m}.verdict.info`]).toBeTruthy();
      expect(dict[`metric.help.${m}.verdict.unknown`]).toBeTruthy();
    }
    // La actividad tiene además el caso `parcial` (ventana aún no completa, spec 007).
    expect(dict["metric.help.activity.verdict.parcial"]).toBeTruthy();
  });
});
