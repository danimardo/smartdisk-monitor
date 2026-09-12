import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Select from "./Select.svelte";

/** Lo que solo se ve en un navegador real: que `size="sm"` deja el control a la misma altura que un
 *  `Button size="sm"` sin perder el nombre accesible (`docs/ui-design.md` §3, Apéndice B). La lógica
 *  de `onchange` no se repite aquí: es de Node. */

const opciones = [
  { id: "15", label: "15 minutos" },
  { id: "60", label: "1 hora" }
];

describe("Select", () => {
  it("por defecto muestra el rótulo visible y el control alto", async () => {
    const { container } = await render(Select, {
      props: { label: "Duración del silencio", value: "60", options: opciones }
    });
    // El rótulo se ve y además es el nombre accesible.
    await expect.element(page.getByText("Duración del silencio")).toBeInTheDocument();
    const select = container.querySelector("select")!;
    expect(select.getAttribute("aria-label")).toBe("Duración del silencio");
  });

  it('`size="sm"`: oculta el rótulo visible pero conserva el nombre accesible', async () => {
    const { container } = await render(Select, {
      props: { size: "sm", label: "Duración del silencio", value: "60", options: opciones }
    });
    // El rótulo ya no se pinta…
    expect(container.textContent).not.toContain("Duración del silencio");
    // …pero el lector de pantalla sigue teniendo el nombre.
    const select = page.getByRole("combobox", { name: "Duración del silencio" });
    await expect.element(select).toBeInTheDocument();
  });

  it('`size="sm"` deja el control más bajo que el tamaño por defecto', async () => {
    const lg = await render(Select, { props: { label: "x", value: "60", options: opciones } });
    const altoLg = lg.container.querySelector("select")!.getBoundingClientRect().height;
    await lg.unmount();

    const sm = await render(Select, {
      props: { size: "sm", label: "x", value: "60", options: opciones }
    });
    const altoSm = sm.container.querySelector("select")!.getBoundingClientRect().height;
    expect(altoSm).toBeLessThan(altoLg);
  });

  it("una opción con `disabled: true` se renderiza como `<option disabled>` (spec 010)", async () => {
    const { container } = await render(Select, {
      props: {
        label: "Modelo",
        value: "15",
        options: [
          { id: "15", label: "15 minutos" },
          { id: "60", label: "1 hora", disabled: true }
        ]
      }
    });
    const [primera, segunda] = Array.from(container.querySelectorAll("option"));
    expect(primera.disabled).toBe(false);
    expect(segunda.disabled).toBe(true);
  });

  it("una opción sin `disabled` se comporta como hoy (no deshabilitada)", async () => {
    const { container } = await render(Select, { props: { label: "x", value: "15", options: opciones } });
    for (const opt of container.querySelectorAll("option")) {
      expect(opt.disabled).toBe(false);
    }
  });
});
