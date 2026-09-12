import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EstadoIaWire, OrigenExplicacion, ResultadoExplicacion } from "$lib/api";

/** Spec 010: reprocesar la explicación con IA con otro modelo gratuito. Se mockea
 *  `explicarDetalleTecnico`, `setSetting` y `estadoIa` (el resto de `$lib/api` se usa tal cual,
 *  como `toAppError`, que es pura). `ia` es la instancia real del store — sus claves «en curso» se
 *  liberan solas en el `finally` de `#pedir`, así que no hace falta resetearla entre pruebas
 *  mientras cada explicación use un `alertGroupId` distinto. */

const explicarMock = vi.fn<(origen: OrigenExplicacion) => Promise<ResultadoExplicacion>>();
const setSettingMock = vi.fn<(key: string, value: unknown) => Promise<void>>();
const estadoIaMock = vi.fn<() => Promise<EstadoIaWire>>();
vi.mock("$lib/api", async (orig) => ({
  ...(await orig<Record<string, unknown>>()),
  explicarDetalleTecnico: (origen: OrigenExplicacion) => explicarMock(origen),
  setSetting: (key: string, value: unknown) => setSettingMock(key, value),
  estadoIa: () => estadoIaMock()
}));

const { explicacion } = await import("./explicacion.svelte");
const { ia } = await import("./ia.svelte");

const estadoIaBase: EstadoIaWire = {
  activa: true,
  modelo: "openrouter/free",
  previewAcknowledged: true,
  sendWithoutReview: false,
  claveValida: true,
  claveCompartidaDisponible: false,
  usandoClaveCompartida: false
};

const origen = (alertGroupId: string): Parameters<typeof explicacion.lanzar>[0] => ({
  tipo: "alerta",
  deviceId: null,
  alertGroupId,
  eventId: null,
  idioma: "es"
});

const ok = (over: Partial<Extract<ResultadoExplicacion, { estado: "ok" }>> = {}) =>
  ({
    estado: "ok",
    markdown: "## Hola",
    modeloUsado: "vendor/auto:free",
    detalleRecortado: false,
    sinVolcado: false,
    sinSuceso: false,
    ...over
  }) satisfies ResultadoExplicacion;

// Bloquea hasta que se llame a `resolve`/`reject`, para controlar cuándo "llega" la respuesta.
function diferida<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("explicacion — reprocesar con otro modelo (spec 010)", () => {
  beforeEach(() => {
    explicarMock.mockReset();
    setSettingMock.mockReset().mockResolvedValue(undefined);
    estadoIaMock.mockReset().mockResolvedValue(estadoIaBase);
  });

  it("reprocesar() envía el mismo origen y el modelo elegido, con previewConfirmada=true (US1)", async () => {
    explicarMock.mockResolvedValueOnce(ok());
    await explicacion.lanzar(origen("g1"));
    expect(explicacion.fase).toBe("resultado");

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/y:free" }));
    await explicacion.reprocesar("vendor/y:free");

    expect(explicarMock).toHaveBeenCalledTimes(2);
    const payload = explicarMock.mock.calls[1][0];
    expect(payload).toMatchObject({
      tipo: "alerta",
      alertGroupId: "g1",
      deviceId: null,
      eventId: null,
      idioma: "es",
      previewConfirmada: true,
      modeloSolicitado: "vendor/y:free"
    });
    expect(explicacion.fase).toBe("resultado");
    expect(explicacion.modeloUsado).toBe("vendor/y:free");
  });

  it("reprocesar() conserva la revisión ya resuelta de la sesión (FR-004)", async () => {
    // Primera llamada: la anonimización marca fragmentos dudosos.
    explicarMock.mockResolvedValueOnce({
      estado: "revision",
      textoCompleto: "texto con <fragmento>",
      fragmentos: [{ texto: "<fragmento>", motivoKey: "ia.review.reason.unknown" }]
    });
    await explicacion.lanzar(origen("g2"));
    expect(explicacion.fase).toBe("revision");

    // La persona decide quitar los fragmentos: la sesión resuelve `revision: "quitar_fragmentos"`.
    explicarMock.mockResolvedValueOnce(ok());
    await explicacion.quitarFragmentos();
    expect(explicacion.fase).toBe("resultado");

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/z:free" }));
    await explicacion.reprocesar("vendor/z:free");

    const payloadDelReproceso = explicarMock.mock.calls[2][0];
    expect(payloadDelReproceso.revision).toBe("quitar_fragmentos");
    expect(payloadDelReproceso.modeloSolicitado).toBe("vendor/z:free");
  });

  it("reprocesar() está disponible también desde la fase error, con el modelo nuevo (US1)", async () => {
    explicarMock.mockRejectedValueOnce({
      code: "ia.network",
      messageKey: "error.ia.network",
      retryable: true
    });
    await explicacion.lanzar(origen("g3"));
    expect(explicacion.fase).toBe("error");

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/w:free" }));
    await explicacion.reprocesar("vendor/w:free");

    expect(explicacion.fase).toBe("resultado");
    const payload = explicarMock.mock.calls[1][0];
    expect(payload.modeloSolicitado).toBe("vendor/w:free");
  });

  it("cerrar durante un reproceso no reabre el modal ni pisa una explicación distinta lanzada después (FR-013)", async () => {
    explicarMock.mockResolvedValueOnce(ok());
    await explicacion.lanzar(origen("g4"));
    expect(explicacion.fase).toBe("resultado");

    const pendiente = diferida<ResultadoExplicacion>();
    explicarMock.mockReturnValueOnce(pendiente.promise);
    const reproceso = explicacion.reprocesar("vendor/tarde:free");
    expect(explicacion.fase).toBe("progreso");

    explicacion.cerrar();
    expect(explicacion.open).toBe(false);

    // Mientras el reproceso de g4 sigue pendiente, se lanza una explicación distinta (g5).
    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/g5:free" }));
    await explicacion.lanzar(origen("g5"));
    expect(explicacion.open).toBe(true);
    expect(explicacion.fase).toBe("resultado");
    expect(explicacion.modeloUsado).toBe("vendor/g5:free");

    // Llega tarde la respuesta del reproceso cancelado de g4: no debe reabrir ni pisar g5.
    pendiente.resolve(ok({ modeloUsado: "vendor/tarde:free" }));
    await reproceso;

    expect(explicacion.open).toBe(true);
    expect(explicacion.modeloUsado).toBe("vendor/g5:free");
    expect(explicacion.fase).toBe("resultado");
  });
});

describe("explicacion — fijar el modelo por defecto (spec 010, US2)", () => {
  beforeEach(() => {
    explicarMock.mockReset();
    setSettingMock.mockReset().mockResolvedValue(undefined);
    estadoIaMock.mockReset().mockResolvedValue(estadoIaBase);
  });

  it("marca esReprocesada solo tras reprocesar, nunca en la respuesta inicial (FR-008)", async () => {
    explicarMock.mockResolvedValueOnce(ok());
    await explicacion.lanzar(origen("g10"));
    expect(explicacion.esReprocesada).toBe(false);

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/y:free" }));
    await explicacion.reprocesar("vendor/y:free");
    expect(explicacion.esReprocesada).toBe(true);
  });

  it("fijarPorDefecto guarda el ajuste y refresca el estado de IA (FR-009)", async () => {
    explicarMock.mockResolvedValueOnce(ok());
    await explicacion.lanzar(origen("g11"));
    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/y:free" }));
    await explicacion.reprocesar("vendor/y:free");

    await explicacion.fijarPorDefecto("vendor/y:free");

    expect(setSettingMock).toHaveBeenCalledWith("settings.ai.model", "vendor/y:free");
    expect(estadoIaMock).toHaveBeenCalled();
    expect(ia.estado?.modelo).toBe("openrouter/free"); // el mock de estadoIa devuelve la base
    expect(explicacion.errorFijarPorDefecto).toBeNull();
  });

  it("si falla al guardar, expone el error sin tocar la respuesta que se estaba viendo (edge case)", async () => {
    explicarMock.mockResolvedValueOnce(ok());
    await explicacion.lanzar(origen("g12"));
    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/y:free", markdown: "## Y" }));
    await explicacion.reprocesar("vendor/y:free");

    setSettingMock.mockRejectedValueOnce(new Error("disco lleno"));
    await explicacion.fijarPorDefecto("vendor/y:free");

    expect(explicacion.errorFijarPorDefecto).not.toBeNull();
    expect(explicacion.fase).toBe("resultado");
    expect(explicacion.markdown).toBe("## Y");
  });
});

describe("explicacion — historial de respuestas (spec 010, US3)", () => {
  beforeEach(() => {
    explicarMock.mockReset();
  });

  it("cada reprocesar() archiva la respuesta que estaba en primer plano, sin límite (FR-007)", async () => {
    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/a:free", markdown: "## A" }));
    await explicacion.lanzar(origen("g20"));
    expect(explicacion.historial).toEqual([]);

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/b:free", markdown: "## B" }));
    await explicacion.reprocesar("vendor/b:free");
    expect(explicacion.historial).toEqual([{ modelo: "vendor/a:free", markdown: "## A" }]);

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/c:free", markdown: "## C" }));
    await explicacion.reprocesar("vendor/c:free");
    expect(explicacion.historial).toEqual([
      { modelo: "vendor/a:free", markdown: "## A" },
      { modelo: "vendor/b:free", markdown: "## B" }
    ]);
    expect(explicacion.markdown).toBe("## C");
  });

  it("archiva también un error antes de reprocesar", async () => {
    explicarMock.mockRejectedValueOnce({
      code: "ia.network",
      messageKey: "error.ia.network",
      retryable: true
    });
    await explicacion.lanzar(origen("g21"));

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/b:free" }));
    await explicacion.reprocesar("vendor/b:free");

    expect(explicacion.historial).toHaveLength(1);
    expect(explicacion.historial[0].modelo).toBe("");
    expect(explicacion.historial[0].error?.code).toBe("ia.network");
  });

  it("lanzar() vacía el historial y reinicia esReprocesada (FR-012)", async () => {
    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/a:free" }));
    await explicacion.lanzar(origen("g22"));
    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/b:free" }));
    await explicacion.reprocesar("vendor/b:free");
    expect(explicacion.historial).toHaveLength(1);

    explicarMock.mockResolvedValueOnce(ok({ modeloUsado: "vendor/auto:free" }));
    await explicacion.lanzar(origen("g23"));
    expect(explicacion.historial).toEqual([]);
    expect(explicacion.esReprocesada).toBe(false);
  });
});
