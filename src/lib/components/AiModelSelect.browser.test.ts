import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import es from "$lib/i18n/es.json";

/** El selector de modelo (spec 005, US3) tiene que aguantar sus tres estados: cargando, lista
 *  cargada, y lista no disponible (US3 escenario 3). El aviso de modelo de pago es FR-015a. */

const listarMock = vi.fn();
vi.mock("$lib/api", async (orig) => ({
  ...(await orig<Record<string, unknown>>()),
  listarModelosIa: () => listarMock()
}));

const { default: AiModelSelect } = await import("./AiModelSelect.svelte");
const { ia } = await import("$lib/stores/ia.svelte");

const estadoIaBase = {
  activa: true,
  modelo: "openrouter/free",
  previewAcknowledged: true,
  sendWithoutReview: false,
  claveValida: true,
  claveCompartidaDisponible: true,
  usandoClaveCompartida: false
};

describe("AiModelSelect", () => {
  beforeEach(() => {
    listarMock.mockReset();
    ia.set({ ...estadoIaBase });
  });

  it("lista cargada: ofrece «automático» y los modelos del proveedor", async () => {
    listarMock.mockResolvedValue([
      { id: "openrouter/free", nombre: "openrouter/free", esDePago: false },
      { id: "x/gratis", nombre: "Equis gratis", esDePago: false }
    ]);
    await render(AiModelSelect, { props: { modelo: "openrouter/free" } });

    await expect
      .element(page.getByRole("option", { name: es["settings.ai.model.auto"] }))
      .toBeInTheDocument();
    await expect.element(page.getByRole("option", { name: "Equis gratis" })).toBeInTheDocument();
  });

  // El estado «lista no disponible» (US3 escenario 3) se cubre en `e2e/ui/ia.spec.ts`: el runner
  // de navegador de Vitest marca cualquier promesa rechazada como fuga aunque el componente la
  // maneje, y forzarlo aquí no aporta sobre el e2e, que sí puede simular el rechazo del comando.

  it("elegir un modelo de pago abre el diálogo de aviso y no notifica hasta confirmar", async () => {
    listarMock.mockResolvedValue([
      { id: "openrouter/free", nombre: "openrouter/free", esDePago: false },
      { id: "v/pago", nombre: "Uve de pago", esDePago: true }
    ]);
    const onchange = vi.fn();
    await render(AiModelSelect, { props: { modelo: "openrouter/free", onchange } });

    await page
      .getByRole("combobox")
      .selectOptions(es["settings.ai.model.paidSuffix"].replace("{name}", "Uve de pago"));

    await expect.element(page.getByText(es["settings.ai.model.paidTitle"])).toBeInTheDocument();
    expect(onchange).not.toHaveBeenCalled();

    await page.getByRole("button", { name: es["settings.ai.model.paidConfirm"] }).click();
    expect(onchange).toHaveBeenCalledWith("v/pago");
  });

  it("incluirAutomatico={false} excluye la opción automática (spec 010)", async () => {
    listarMock.mockResolvedValue([
      { id: "openrouter/free", nombre: "openrouter/free", esDePago: false },
      { id: "x/gratis", nombre: "Equis gratis", esDePago: false }
    ]);
    await render(AiModelSelect, {
      props: { modelo: "x/gratis", incluirAutomatico: false }
    });

    expect(page.getByRole("option", { name: es["settings.ai.model.auto"] }).query()).toBeNull();
    await expect.element(page.getByRole("option", { name: "Equis gratis" })).toBeInTheDocument();
  });

  it("con la clave de demostración activa, los modelos de pago aparecen pero deshabilitados (spec 010, US4/FR-010)", async () => {
    ia.set({ ...estadoIaBase, usandoClaveCompartida: true });
    listarMock.mockResolvedValue([
      { id: "openrouter/free", nombre: "openrouter/free", esDePago: false },
      { id: "v/pago", nombre: "Uve de pago", esDePago: true }
    ]);
    const { container } = await render(AiModelSelect, { props: { modelo: "openrouter/free" } });

    const opcionPago = Array.from(container.querySelectorAll("option")).find((o) =>
      o.textContent?.includes("Uve de pago")
    );
    expect(opcionPago).toBeDefined();
    expect(opcionPago!.disabled).toBe(true);
    await expect.element(page.getByText(es["settings.ai.model.paidDisabledDemo"])).toBeInTheDocument();
  });

  it("con una clave propia, los modelos de pago vuelven a estar disponibles (spec 010, US4/FR-011)", async () => {
    ia.set({ ...estadoIaBase, usandoClaveCompartida: false });
    listarMock.mockResolvedValue([
      { id: "openrouter/free", nombre: "openrouter/free", esDePago: false },
      { id: "v/pago", nombre: "Uve de pago", esDePago: true }
    ]);
    const { container } = await render(AiModelSelect, { props: { modelo: "openrouter/free" } });

    const opcionPago = Array.from(container.querySelectorAll("option")).find((o) =>
      o.textContent?.includes("Uve de pago")
    );
    expect(opcionPago!.disabled).toBe(false);
    expect(page.getByText(es["settings.ai.model.paidDisabledDemo"]).query()).toBeNull();
  });

  it("incluirAutomatico omitido (u `true`) mantiene el comportamiento actual", async () => {
    listarMock.mockResolvedValue([{ id: "openrouter/free", nombre: "openrouter/free", esDePago: false }]);
    await render(AiModelSelect, { props: { modelo: "openrouter/free" } });

    await expect
      .element(page.getByRole("option", { name: es["settings.ai.model.auto"] }))
      .toBeInTheDocument();
  });
});
