import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import EmptyState from "./EmptyState.svelte";
import es from "$lib/i18n/es.json";

/** `EmptyState` es el componente que las pantallas usan para presentar sus tres estados de fuente
 *  (constitución §VIII): vacío, no compatible y error de fuente. «Cargando» no es responsabilidad
 *  suya (lo cubre `ProgressBar`), y «dato obsoleto» tampoco (lo cubre `MetricCard`/`DiskCard` con
 *  su marca de edad) — este componente solo prueba lo que él mismo decide: el rótulo por `kind`,
 *  el detalle técnico colapsado y la acción opcional.
 */

describe("EmptyState", () => {
  it("vacío: usa el rótulo «Sin datos», nunca lo confunde con un error", async () => {
    await render(EmptyState, { props: { kind: "empty", title: "Sin discos todavía" } });
    await expect.element(page.getByText(es["common.noData"])).toBeInTheDocument();
    await expect.element(page.getByText("Sin discos todavía")).toBeInTheDocument();
  });

  it("no compatible: usa su propio rótulo, distinto del de error", async () => {
    await render(EmptyState, { props: { kind: "unsupported", title: "RAID sin SMART" } });
    await expect.element(page.getByText(es["common.unsupported"])).toBeInTheDocument();
  });

  it("error de fuente: usa su propio rótulo, distinto del de no compatible", async () => {
    await render(EmptyState, { props: { kind: "error", title: "No se pudo leer smartctl" } });
    await expect.element(page.getByText(es["common.sourceError"])).toBeInTheDocument();
  });

  it("el detalle técnico va colapsado tras un <summary>, no expuesto por defecto", async () => {
    await render(EmptyState, {
      props: { kind: "error", title: "Fallo", detail: "smartctl: exit code 1" }
    });
    await expect.element(page.getByText(es["common.technicalDetail"])).toBeInTheDocument();
    const detalle = document.querySelector("details");
    expect(detalle?.open).toBe(false);
  });

  it("sin acción no aparece ningún botón", async () => {
    await render(EmptyState, { props: { kind: "empty", title: "Vacío" } });
    expect(document.querySelector("button")).toBeNull();
  });

  it("con acción, el botón dispara el callback al pulsarlo", async () => {
    const onaction = vi.fn();
    await render(EmptyState, {
      props: { kind: "empty", title: "Vacío", actionLabel: "Reintentar", onaction }
    });
    await page.getByRole("button", { name: "Reintentar" }).click();
    expect(onaction).toHaveBeenCalledOnce();
  });
});
