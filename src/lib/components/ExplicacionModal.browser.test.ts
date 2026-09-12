import { page } from "vitest/browser";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import es from "$lib/i18n/es.json";

/** El modal de la ayuda con IA tiene que cubrir sus estados (constitución §VIII): progreso, error,
 *  resultado y revisión/vista previa. Es `role="dialog" aria-modal` con cierre por teclado.
 *
 *  Spec 010: el selector de reproceso se resuelve internamente vía `AiModelSelect`, que carga su
 *  propio catálogo — se mockea `listarModelosIa` a nivel de módulo (mismo patrón que
 *  `AiModelSelect.browser.test.ts`). */

const listarMock = vi.fn();
vi.mock("$lib/api", async (orig) => ({
  ...(await orig<Record<string, unknown>>()),
  listarModelosIa: () => listarMock()
}));

const { default: ExplicacionModal } = await import("./ExplicacionModal.svelte");

describe("ExplicacionModal", () => {
  beforeEach(() => {
    listarMock.mockReset();
    listarMock.mockResolvedValue([{ id: "x/gratis", nombre: "Equis gratis", esDePago: false }]);
  });

  it("cerrado no pinta nada", async () => {
    await render(ExplicacionModal, { props: { open: false } });
    expect(document.querySelector("[role=dialog]")).toBeNull();
  });

  it("progreso: muestra el indicador y un botón de cancelar, sin botón de cerrar", async () => {
    await render(ExplicacionModal, { props: { open: true, fase: "progreso" } });
    await expect.element(page.getByText(es["ai.modal.progress"])).toBeInTheDocument();
    await expect.element(page.getByRole("button", { name: es["common.cancel"] })).toBeInTheDocument();
    expect(page.getByRole("button", { name: es["common.close"] }).query()).toBeNull();
  });

  it("error reintentable: muestra la frase humana, el detalle colapsado y «Reintentar»", async () => {
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "error",
        error: {
          code: "ia.network",
          messageKey: "error.ia.network",
          detail: "reqwest: dns error",
          retryable: true
        }
      }
    });
    await expect.element(page.getByText(es["error.ia.network"])).toBeInTheDocument();
    await expect.element(page.getByRole("button", { name: es["ai.modal.retry"] })).toBeInTheDocument();
    expect(document.querySelector("details")?.open).toBe(false);
  });

  it("resultado: renderiza el markdown, el modelo usado y la advertencia de IA", async () => {
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "resultado",
        markdown: "## Resumen\n\nTu disco está bien.",
        modeloUsado: "vendor/model:free"
      }
    });
    await expect.element(page.getByRole("heading", { name: "Resumen" })).toBeInTheDocument();
    await expect.element(page.getByText(es["ai.modal.disclaimer"])).toBeInTheDocument();
    await expect.element(page.getByText("vendor/model:free", { exact: false })).toBeInTheDocument();
  });

  it("resultado: ofrece reprocesar con otro modelo (spec 010, US1)", async () => {
    const onreprocesar = vi.fn();
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "resultado",
        markdown: "## Resumen",
        modeloUsado: "vendor/model:free",
        onreprocesar
      }
    });
    await expect.element(page.getByText(es["ai.modal.reprocess.invite"])).toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: es["ai.modal.reprocess.action"] }))
      .toBeInTheDocument();

    await page.getByRole("combobox").selectOptions("Equis gratis");
    await page.getByRole("button", { name: es["ai.modal.reprocess.action"] }).click();
    expect(onreprocesar).toHaveBeenCalledWith("x/gratis");
  });

  it("«fijar por defecto» no aparece sobre la respuesta inicial del modo automático (spec 010, US2/FR-008)", async () => {
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "resultado",
        markdown: "## Resumen",
        modeloUsado: "openrouter/free",
        esReprocesada: false
      }
    });
    expect(page.getByRole("button", { name: es["ai.modal.reprocess.setDefault"] }).query()).toBeNull();
  });

  it("«fijar por defecto» aparece sobre una respuesta reprocesada y llama a onfijarpordefecto con su modelo", async () => {
    const onfijarpordefecto = vi.fn();
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "resultado",
        markdown: "## Resumen",
        modeloUsado: "vendor/y:free",
        esReprocesada: true,
        onfijarpordefecto
      }
    });
    await page.getByRole("button", { name: es["ai.modal.reprocess.setDefault"] }).click();
    expect(onfijarpordefecto).toHaveBeenCalledWith("vendor/y:free");
  });

  it("error: también ofrece reprocesar con otro modelo (spec 010, US1)", async () => {
    const onreprocesar = vi.fn();
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "error",
        error: { code: "ia.network", messageKey: "error.ia.network", retryable: false },
        onreprocesar
      }
    });
    await expect.element(page.getByText(es["ai.modal.reprocess.invite"])).toBeInTheDocument();

    await page.getByRole("combobox").selectOptions("Equis gratis");
    await page.getByRole("button", { name: es["ai.modal.reprocess.action"] }).click();
    expect(onreprocesar).toHaveBeenCalledWith("x/gratis");
  });

  it("historial: cada entrada se ve plegada, con su modelo, y se puede desplegar sin cerrar las demás (spec 010, US3)", async () => {
    const { container } = await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "resultado",
        markdown: "## Actual",
        modeloUsado: "vendor/c:free",
        historial: [
          { modelo: "vendor/a:free", markdown: "Respuesta de A" },
          { modelo: "vendor/b:free", markdown: "Respuesta de B" }
        ]
      }
    });

    const resumenes = Array.from(container.querySelectorAll("details > summary"));
    expect(resumenes.map((s) => s.textContent)).toEqual([
      es["ai.modal.reprocess.historyEntry"].replace("{model}", "vendor/a:free"),
      es["ai.modal.reprocess.historyEntry"].replace("{model}", "vendor/b:free")
    ]);

    const detallesArray = Array.from(container.querySelectorAll("details"));
    expect(detallesArray.map((d) => d.open)).toEqual([false, false]);

    detallesArray[0].open = true;
    expect(page.getByText("Respuesta de A").query()).not.toBeNull();
    expect(detallesArray[1].open).toBe(false);
  });

  it("historial: una entrada con error muestra el mensaje humano al desplegarla", async () => {
    const { container } = await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "resultado",
        markdown: "## Actual",
        modeloUsado: "vendor/b:free",
        historial: [
          {
            modelo: "",
            error: { code: "ia.network", messageKey: "error.ia.network", retryable: true }
          }
        ]
      }
    });
    const detalle = container.querySelector("details")!;
    detalle.open = true;
    await expect.element(page.getByText(es["error.ia.network"])).toBeInTheDocument();
  });

  it("vista previa: muestra el texto exacto y un botón de confirmar", async () => {
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "vistaPrevia",
        textoRevision: "Regla: smart.temp_high\nDisco: modelo <SERIE-1>"
      }
    });
    await expect.element(page.getByText("Regla: smart.temp_high", { exact: false })).toBeInTheDocument();
    await expect.element(page.getByRole("button", { name: es["ai.preview.confirm"] })).toBeInTheDocument();
  });

  it("revisión con fragmentos: los lista y ofrece enviar tal cual o quitar", async () => {
    await render(ExplicacionModal, {
      props: {
        open: true,
        fase: "revision",
        textoRevision: "…",
        fragmentos: [{ texto: "D:\\Usuarios\\Ana", motivoKey: "ai.review.path" }]
      }
    });
    await expect.element(page.getByText("D:\\Usuarios\\Ana")).toBeInTheDocument();
    await expect.element(page.getByText(es["ai.review.path"], { exact: false })).toBeInTheDocument();
    await expect.element(page.getByRole("button", { name: es["ai.review.sendAnyway"] })).toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: es["ai.review.stripFragments"] }))
      .toBeInTheDocument();
  });

  it("es un diálogo modal accesible", async () => {
    await render(ExplicacionModal, { props: { open: true, fase: "resultado", markdown: "x" } });
    const dialogo = document.querySelector("[role=dialog]");
    expect(dialogo?.getAttribute("aria-modal")).toBe("true");
  });

  it("Escape en el velo cierra (llama a oncancel en progreso)", async () => {
    const oncancel = vi.fn();
    await render(ExplicacionModal, { props: { open: true, fase: "progreso", oncancel } });
    const velo = document.querySelector("[role=presentation]") as HTMLElement;
    velo.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(oncancel).toHaveBeenCalled();
  });
});
