import { beforeEach, describe, expect, it, vi } from "vitest";

const listenMock = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args)
}));

const { on, subscribe } = await import("./events");

/** La interfaz no sondea: escucha lo que el backend empuja (ADR-015). Estas pruebas fijan que la
 *  suscripción usa los nombres del contrato y que soltar los oyentes al desmontar funciona —un
 *  oyente que sobrevive a su pantalla es una fuga que solo se nota tras horas de uso. */

describe("on", () => {
  beforeEach(() => {
    listenMock.mockReset();
    listenMock.mockResolvedValue(() => {});
  });

  it("se suscribe con el nombre exacto del contrato", async () => {
    await on("metrics:updated", () => {});
    expect(listenMock).toHaveBeenCalledWith("metrics:updated", expect.any(Function));
  });

  it("entrega al manejador la carga útil, no el sobre del evento", async () => {
    let recibido: unknown = null;
    listenMock.mockImplementation((_name: string, cb: (e: { payload: unknown }) => void) => {
      cb({ payload: { emittedAt: "2026-09-04T12:00:00Z", devices: [], sources: [] } });
      return Promise.resolve(() => {});
    });
    await on("metrics:updated", (p) => (recibido = p));
    expect(recibido).toMatchObject({ emittedAt: "2026-09-04T12:00:00Z" });
  });
});

describe("subscribe", () => {
  beforeEach(() => {
    listenMock.mockReset();
  });

  it("suscribe todos los eventos que se le pasan", async () => {
    listenMock.mockResolvedValue(() => {});
    await subscribe({
      "metrics:updated": () => {},
      "alerts:changed": () => {},
      "monitoring:paused": () => {}
    });
    expect(listenMock).toHaveBeenCalledTimes(3);
    const nombres = listenMock.mock.calls.map((c) => c[0]);
    expect(nombres).toEqual(
      expect.arrayContaining(["metrics:updated", "alerts:changed", "monitoring:paused"])
    );
  });

  it("devuelve una sola función que suelta todos los oyentes", async () => {
    const sueltos: string[] = [];
    listenMock.mockImplementation((name: string) => Promise.resolve(() => sueltos.push(name)));

    const unsubscribe = await subscribe({
      "metrics:updated": () => {},
      "alerts:changed": () => {}
    });
    expect(sueltos).toEqual([]);

    unsubscribe();
    expect(sueltos).toHaveLength(2);
    expect(sueltos).toEqual(expect.arrayContaining(["metrics:updated", "alerts:changed"]));
  });

  it("sin manejadores no suscribe nada y sigue devolviendo función", async () => {
    const unsubscribe = await subscribe({});
    expect(listenMock).not.toHaveBeenCalled();
    expect(() => unsubscribe()).not.toThrow();
  });
});
