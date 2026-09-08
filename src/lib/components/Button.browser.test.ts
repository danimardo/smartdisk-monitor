import { createRawSnippet } from "svelte";
import { page, userEvent } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import Button from "./Button.svelte";

/** Prueba de referencia de un componente del catálogo. Lo que se comprueba aquí es lo que **solo**
 *  se puede comprobar en un navegador real; la lógica pura ya tiene su prueba en Node y no se
 *  repite (`docs/testing-strategy.md` §7).
 *
 *  Los localizadores van por rol y nombre accesible, nunca por clase ni por estructura del DOM
 *  (§12): si un elemento no se puede localizar por su rol, es que no es accesible, y eso incumple
 *  el principio VII de la constitución antes que ninguna prueba.
 */

/** El componente recibe su contenido como *snippet*, no como slot. */
function contenido(texto: string) {
  return createRawSnippet(() => ({ render: () => `<span>${texto}</span>` }));
}

describe("Button", () => {
  it("expone rol de botón y su nombre accesible", async () => {
    await render(Button, { props: { children: contenido("Analizar disco") } });
    await expect.element(page.getByRole("button", { name: "Analizar disco" })).toBeInTheDocument();
  });

  it("deshabilitado, lo anuncia y explica por qué", async () => {
    await render(Button, {
      props: {
        disabled: true,
        disabledReason: "Requiere privilegios de administrador",
        children: contenido("Reparar")
      }
    });
    const boton = page.getByRole("button", { name: "Reparar" });
    await expect.element(boton).toBeDisabled();
    // `disabled` a secas deja al usuario sin saber qué le falta. El motivo es parte del contrato.
    await expect.element(boton).toHaveAttribute("title", "Requiere privilegios de administrador");
  });

  it("activo, expone `hint` como ayuda `title`; deshabilitado, manda el motivo", async () => {
    const { rerender } = await render(Button, {
      props: { hint: "Deja de contar para el color del disco", children: contenido("Archivar") }
    });
    const boton = page.getByRole("button", { name: "Archivar" });
    await expect.element(boton).toHaveAttribute("title", "Deja de contar para el color del disco");

    await rerender({
      disabled: true,
      disabledReason: "Ya está archivada",
      hint: "Deja de contar para el color del disco",
      children: contenido("Archivar")
    });
    await expect.element(boton).toHaveAttribute("title", "Ya está archivada");
  });

  it("mientras carga, sigue anunciándose como deshabilitado", async () => {
    await render(Button, { props: { loading: true, children: contenido("Exportando") } });
    await expect.element(page.getByRole("button", { name: "Exportando" })).toBeDisabled();
  });

  it("al llegar por teclado muestra un indicador de foco distinto del estado normal", async () => {
    await render(Button, { props: { children: contenido("Aceptar") } });
    const elemento = document.querySelector("button")!;
    const reposo = getComputedStyle(elemento).boxShadow;

    // Con teclado, no con `.focus()`: Chromium solo hace casar `:focus-visible` cuando la última
    // interacción fue de teclado, así que enfocar por script daría un falso negativo.
    await userEvent.tab();
    expect(document.activeElement).toBe(elemento);

    // WCAG 2.4.11. El sistema hace `outline: none` y lo sustituye por `--sdm-focus-ring`
    // (`tokens.css`), que es legítimo **siempre que el sustituto exista**. Comparar contra el
    // estado de reposo es lo que distingue «sustituido» de «suprimido»: comprobar solo que hay
    // alguna sombra pasaría en verde con el anillo borrado, porque la variante `secondary` ya
    // lleva `shadow-edge` de serie.
    const enfocado = getComputedStyle(elemento).boxShadow;
    expect(enfocado, "el foco de teclado no cambia el aspecto del botón").not.toBe(reposo);
    expect(enfocado).not.toBe("none");
  });

  it("no se recorta a 1024 × 560, el mínimo técnico", async () => {
    await render(Button, {
      props: { size: "lg", children: contenido("Crear paquete de diagnóstico completo") }
    });
    const elemento = document.querySelector("button")!;
    // `whitespace-nowrap` hace que un texto largo desborde en vez de partirse: es justo el caso
    // que la ventana mínima destapa (`docs/ui-design.md` §8).
    expect(elemento.scrollWidth).toBeLessThanOrEqual(elemento.clientWidth + 1);
  });
});
