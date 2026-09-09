import { page, userEvent } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import AboutDialog from "./AboutDialog.svelte";
import es from "$lib/i18n/es.json";

/** «Acerca de» (US-061), enriquecido con la foto y la bio del autor (ADR-052). Es
 *  `role="dialog" aria-modal` con cierre por teclado, cruz y botón; cubre sus estados
 *  (constitución §VIII): cargando, error y cargado. */

const appInfo = { name: "SmartDisk Monitor", version: "0.1.3", author: "Daniel Diez Mardomingo" };

describe("AboutDialog", () => {
  it("cerrado no pinta nada", async () => {
    await render(AboutDialog, { props: { open: false, appInfo } });
    expect(document.querySelector("[role=dialog]")).toBeNull();
  });

  it("cargado: el diálogo se llama «nombre versión» y presenta al autor con foto y bio", async () => {
    await render(AboutDialog, { props: { open: true, appInfo } });

    const dialogo = page.getByRole("dialog", { name: `${appInfo.name} ${appInfo.version}` });
    await expect.element(dialogo).toBeInTheDocument();
    await expect.element(page.getByText(es["about.authorName"])).toBeInTheDocument();
    await expect
      .element(page.getByText(es["about.bio.p1"].slice(0, 30), { exact: false }))
      .toBeInTheDocument();
    // La foto empaquetada (ADR-052) carga de verdad: `naturalWidth > 0`, no solo el `<img>` vacío.
    const foto = page.getByRole("img", { name: es["about.photoAlt"] });
    await expect.element(foto).toBeInTheDocument();
    await expect.poll(() => (foto.element() as HTMLImageElement).naturalWidth).toBeGreaterThan(0);
    // Los créditos llevan el autor real (lo comprueba también `e2e/ui/about.spec.ts`).
    await expect.element(page.getByText(appInfo.author, { exact: false })).toBeInTheDocument();
  });

  it("sin appInfo todavía: muestra «cargando», nunca datos inventados", async () => {
    await render(AboutDialog, { props: { open: true, appInfo: null } });
    await expect.element(page.getByText(es["common.loading"])).toBeInTheDocument();
    // La bio es estática: se pinta igual.
    await expect.element(page.getByText(es["about.authorName"])).toBeInTheDocument();
  });

  it("con error de carga: muestra la frase humana del error", async () => {
    await render(AboutDialog, {
      props: {
        open: true,
        appInfo: null,
        error: { code: "app.info", messageKey: "error.unexpected", detail: "boom", retryable: false }
      }
    });
    await expect.element(page.getByText(es["error.unexpected"])).toBeInTheDocument();
  });

  it("Escape cierra (dispara onclose): el foco entra al panel al abrir", async () => {
    const onclose = vi.fn();
    await render(AboutDialog, { props: { open: true, appInfo, onclose } });
    // Sin ningún clic previo: el `$effect` mete el foco en el panel, así que el keydown burbujea
    // hasta el velo (docs/known-issues.md #2).
    await userEvent.keyboard("{Escape}");
    expect(onclose).toHaveBeenCalled();
  });

  it("tiene un objetivo «Cerrar» y el botón «Copiar información»", async () => {
    await render(AboutDialog, { props: { open: true, appInfo } });
    await expect.element(page.getByRole("button", { name: es["common.close"] }).first()).toBeInTheDocument();
    await expect.element(page.getByRole("button", { name: es["about.cta.copy"] })).toBeInTheDocument();
  });

  it("«Copiar información» está deshabilitado mientras no hay datos", async () => {
    const { rerender } = await render(AboutDialog, { props: { open: true, appInfo: null } });
    await expect.element(page.getByRole("button", { name: es["about.cta.copy"] })).toBeDisabled();
    await rerender({ open: true, appInfo });
    await expect.element(page.getByRole("button", { name: es["about.cta.copy"] })).toBeEnabled();
  });
});
