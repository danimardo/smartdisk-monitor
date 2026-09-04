import { afterEach, describe, expect, it } from "vitest";
import { congelarTiempo, descongelarTiempo } from "../../../tests/helpers/tiempo";
import { formatAge, formatDateTime } from "./format";

/** Determinismo temporal (`docs/testing-strategy.md` §21).
 *
 *  En este producto la hora no es decoración: la retención, el enfriamiento de las notificaciones y
 *  la correlación con el Visor de eventos de Windows dependen de ella. Estas pruebas existen para
 *  que un cambio de día o un cambio de hora no se descubra en producción.
 */

afterEach(descongelarTiempo);

describe("con el reloj congelado", () => {
  it("formatAge no necesita que le pasen la hora para ser reproducible", () => {
    congelarTiempo("2026-09-04T12:00:00Z");
    // Sin segundo argumento: usa `Date.now()`, que ahora está fijo. Si esto empieza a fallar de
    // forma intermitente es que alguien ha metido una fuente de tiempo no congelable.
    expect(formatAge("2026-09-04T11:00:00Z")).toBe(formatAge("2026-09-04T11:00:00Z"));
    expect(formatAge("2026-09-04T11:00:00Z")).not.toBeNull();
  });

  it("cruzar la medianoche cambia la unidad, no solo el número", () => {
    const antes = formatAge("2026-09-03T23:30:00Z", Date.parse("2026-09-03T23:59:00Z"));
    const despues = formatAge("2026-09-03T23:30:00Z", Date.parse("2026-09-04T00:30:00Z"));
    expect(antes).not.toBe(despues);
  });
});

describe("cambio de hora en Europe/Madrid", () => {
  /** La madrugada del 25 de octubre de 2026 los relojes atrasan de 03:00 a 02:00 (CEST → CET).
   *  Esa noche dura 25 horas y hay una hora local que ocurre **dos veces**. */
  const ANTES_UTC = "2026-10-25T00:30:00Z"; // 02:30 CEST
  const DESPUES_UTC = "2026-10-25T01:30:00Z"; // 02:30 CET, la misma hora local otra vez

  it("dos instantes distintos caen en la misma hora local", () => {
    // No es un fallo: es la razón por la que una marca de tiempo local sin desplazamiento es
    // ambigua, y por la que los logs de Rust llevan el desplazamiento explícito (ADR-024).
    expect(formatDateTime(ANTES_UTC)).toBe(formatDateTime(DESPUES_UTC));
  });

  it("la antigüedad se calcula en tiempo absoluto y el cambio de hora no la falsea", () => {
    // Entre los dos instantes pasa exactamente una hora real, aunque el reloj de pared marque lo
    // mismo. Calcular la antigüedad sobre la hora local daría cero y la lectura parecería fresca.
    const transcurrido = Date.parse(DESPUES_UTC) - Date.parse(ANTES_UTC);
    expect(transcurrido).toBe(3_600_000);

    const edad = formatAge(ANTES_UTC, Date.parse(DESPUES_UTC));
    expect(edad).not.toBeNull();
    expect(edad).not.toBe(formatAge(DESPUES_UTC, Date.parse(DESPUES_UTC)));
  });

  it("la zona horaria de la suite está fija, no heredada del equipo", () => {
    // Si esto falla, la configuración de Vitest ha perdido `TZ` y el resto de este fichero deja de
    // significar nada: pasaría en un equipo español y fallaría en CI.
    expect(Intl.DateTimeFormat().resolvedOptions().timeZone).toBe("Europe/Madrid");
  });
});
