import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invokeMock(...a) }));

const { createLogger, setLogLevel, getLogLevel } = await import("./logger");

/** Fija las reglas del principio XV que no se ven leyendo el código: qué se filtra, qué llega al
 *  fichero y qué pasa si el envío falla. */

describe("filtro por nivel", () => {
  let salida: unknown[][];

  beforeEach(() => {
    salida = [];
    vi.spyOn(console, "log").mockImplementation((...a) => void salida.push(a));
    vi.spyOn(console, "warn").mockImplementation((...a) => void salida.push(a));
    vi.spyOn(console, "error").mockImplementation((...a) => void salida.push(a));
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  afterEach(() => vi.restoreAllMocks());

  it("el nivel por defecto es info", () => {
    expect(getLogLevel()).toBe("info");
  });

  it("con nivel info, `debug` no se emite", () => {
    setLogLevel("info");
    const log = createLogger("prueba");
    log.debug("no debería verse");
    expect(salida).toHaveLength(0);
  });

  it("con nivel debug, `debug` sí se emite", () => {
    setLogLevel("debug");
    createLogger("prueba").debug("visible");
    expect(salida).toHaveLength(1);
  });

  it("`silent` calla incluso los errores", () => {
    setLogLevel("silent");
    const log = createLogger("prueba");
    log.error("ni esto");
    expect(salida).toHaveLength(0);
  });

  it("el ámbito aparece en cada entrada, para poder filtrar un log largo", () => {
    setLogLevel("info");
    createLogger("collector").info("hola");
    expect(salida[0][0]).toBe("[collector]");
  });
});

describe("envío al backend", () => {
  beforeEach(() => {
    vi.spyOn(console, "log").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
    vi.spyOn(console, "error").mockImplementation(() => {});
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    setLogLevel("trace");
  });

  afterEach(() => vi.restoreAllMocks());

  it("`debug` e `info` NO cruzan el IPC: son ruido de desarrollo", () => {
    const log = createLogger("prueba");
    log.trace("a");
    log.debug("b");
    log.info("c");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("`warn` y `error` sí llegan al fichero, para que estén en el ZIP de diagnóstico", () => {
    const log = createLogger("prueba");
    log.warn("cuidado");
    log.error("roto");
    expect(invokeMock).toHaveBeenCalledTimes(2);
    expect(invokeMock.mock.calls[0][0]).toBe("log_from_ui");
    expect(invokeMock.mock.calls[0][1]).toMatchObject({ level: "warn", scope: "prueba", message: "cuidado" });
  });

  it("el contexto viaja como objeto, no incrustado en la frase", () => {
    createLogger("api").error("respuesta fuera de contrato", { command: "get_devices" });
    expect(invokeMock.mock.calls[0][1]).toMatchObject({ context: { command: "get_devices" } });
  });

  it("sin contexto se envía null, no undefined", () => {
    createLogger("api").warn("algo");
    expect(invokeMock.mock.calls[0][1]).toMatchObject({ context: null });
  });

  it("si el envío falla, no se propaga: un log no puede tumbar la interfaz", () => {
    invokeMock.mockImplementation(() => Promise.reject(new Error("IPC caído")));
    const log = createLogger("prueba");
    expect(() => log.error("con el IPC roto")).not.toThrow();
  });
});
