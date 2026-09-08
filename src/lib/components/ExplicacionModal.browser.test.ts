import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import ExplicacionModal from "./ExplicacionModal.svelte";
import es from "$lib/i18n/es.json";

/** El modal de la ayuda con IA tiene que cubrir sus estados (constitución §VIII): progreso, error,
 *  resultado y revisión/vista previa. Es `role="dialog" aria-modal` con cierre por teclado. */

describe("ExplicacionModal", () => {
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
