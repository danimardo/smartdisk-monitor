import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import EventRow from "./EventRow.svelte";
import es from "$lib/i18n/es.json";

/** `EventRow` vive en una `VirtualList` de miles de filas: la altura no cambia (42 px) y el nivel
 *  se lee por color **e** icono, nunca por color solo (`03-eventos.md`, constitución §VII). */

describe("EventRow", () => {
  it("el nivel es un cuadrado con icono y nombre accesible, no una píldora de texto en la fila", async () => {
    const { container } = await render(EventRow, {
      props: { level: "error", message: "Fallo de E/S en el disco 2", provider: "disk", eventId: 7 }
    });
    // El icono del nivel `error` es `bolt` y lleva el nombre del nivel como etiqueta accesible.
    expect(container.querySelector('use[href="#i-bolt"]')).not.toBeNull();
    await expect.element(page.getByRole("img", { name: es["events.level.error"] })).toBeInTheDocument();
  });

  it("una asociación inferida se etiqueta de forma explícita, sin competir con el dato", async () => {
    await render(EventRow, {
      props: { level: "info", message: "m", provider: "Ntfs", eventId: 1, mappingConfidence: "inferred" }
    });
    await expect.element(page.getByText(es["events.inferredMapping"])).toBeInTheDocument();
  });

  it("una asociación exacta no muestra la etiqueta", async () => {
    await render(EventRow, {
      props: { level: "info", message: "m", provider: "Ntfs", eventId: 1, mappingConfidence: "exact" }
    });
    expect(page.getByText(es["events.inferredMapping"]).query()).toBeNull();
  });

  it("al pulsarla invoca onselect", async () => {
    const onselect = vi.fn();
    await render(EventRow, { props: { level: "warning", message: "m", onselect } });
    await page.getByRole("button").click();
    expect(onselect).toHaveBeenCalledOnce();
  });

  it("con `href` es un enlace de bloque (panel general), no un botón", async () => {
    const { container } = await render(EventRow, {
      props: { level: "error", message: "m", href: "/events?focus=42" }
    });
    const enlace = container.querySelector("a");
    expect(enlace?.getAttribute("href")).toBe("/events?focus=42");
    // Enlace de bloque: no se subraya ni muestra el cursor de mano, pero sí tiñe el fondo con el
    // violeta de acento al pasar el ratón (feedback de que se puede pulsar, como una lista nativa
    // de Windows) — no un aclarado. El tinte lo pinta una `::after` colgada de `sdm-hover-bloque`.
    expect(enlace?.classList.contains("sdm-block-link")).toBe(true);
    expect(enlace?.classList.contains("sdm-hover-bloque")).toBe(true);
    expect(container.querySelector("button")).toBeNull();
  });

  it("como botón (pantalla de eventos) también tiñe el fondo de acento al pasar el ratón", async () => {
    const { container } = await render(EventRow, {
      props: { level: "warning", message: "m", onselect: () => {} }
    });
    expect(container.querySelector("button")?.classList.contains("sdm-hover-bloque")).toBe(true);
  });
});
