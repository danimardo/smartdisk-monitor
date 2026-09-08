import { page, userEvent } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import { createRawSnippet } from "svelte";
import Tooltip from "./Tooltip.svelte";

/** El `Tooltip` es la ayuda al pasar el ratón / al enfocar (patrón WAI-ARIA). Lo que jsdom no ve
 *  —el foco, la aparición y `Escape`— se comprueba en Chromium. */

const disparo = createRawSnippet(() => ({ render: () => `<span>Temp.</span>` }));
const globo = (c: Element) => c.querySelector('[role="tooltip"]');

describe("Tooltip", () => {
  it("aparece al pasar el ratón, con role=tooltip, y desaparece al salir", async () => {
    const { container } = await render(Tooltip, {
      props: { titulo: "Temperatura", text: "Explicación del dato", children: disparo }
    });
    expect(globo(container)).toBeNull();

    await userEvent.hover(page.getByText("Temp."));
    expect(globo(container)?.textContent).toContain("Explicación del dato");

    await userEvent.unhover(page.getByText("Temp."));
    await expect.element(page.getByRole("tooltip")).not.toBeInTheDocument();
  });

  it("focusable: aparece al enfocar, describe al disparador, y se cierra con Escape", async () => {
    const { container } = await render(Tooltip, {
      props: { text: "Ayuda por teclado", children: disparo }
    });
    const boton = container.querySelector("button")!;

    boton.focus();
    await expect.element(page.getByRole("tooltip")).toBeInTheDocument();
    expect(boton.getAttribute("aria-describedby")).toBe(globo(container)!.id);

    await userEvent.keyboard("{Escape}");
    await expect.element(page.getByRole("tooltip")).not.toBeInTheDocument();
  });

  it("focusable={false}: el disparador es un <span> sin botón (válido dentro de un <a>)", async () => {
    const { container } = await render(Tooltip, {
      props: { focusable: false, text: "Solo ratón", children: disparo }
    });
    expect(container.querySelector("button")).toBeNull();

    await userEvent.hover(page.getByText("Temp."));
    expect(globo(container)?.textContent).toContain("Solo ratón");
  });
});
