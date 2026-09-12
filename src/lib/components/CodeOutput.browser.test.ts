import { page } from "vitest/browser";
import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import CodeOutput from "./CodeOutput.svelte";

/** Salida literal de un proceso auxiliar o del XML de un suceso (`$lib/design/xml`). Se renderiza
 *  siempre como texto — cada fragmento coloreado es una interpolación de Svelte dentro de un
 *  `<span>`, nunca `{@html}` — y el modo por defecto ("text") no cambia de comportamiento respecto
 *  a antes de que existiera `lang="xml"`. */

describe("CodeOutput", () => {
  it("por defecto (JSON de smartctl, salida de chkdsk) muestra el contenido tal cual, sin colorear", async () => {
    const { container } = await render(CodeOutput, { props: { content: '{"a":1}' } });
    const pre = container.querySelector("pre");
    expect(pre?.textContent).toBe('{"a":1}');
    expect(pre?.querySelector("span")).toBeNull();
  });

  it('con lang="xml" reindenta el XML compacto en líneas legibles', async () => {
    const { container } = await render(CodeOutput, {
      props: { content: '<System><Provider Name="disk"/></System>', lang: "xml" }
    });
    const pre = container.querySelector("pre");
    expect(pre?.textContent).toBe(["<System>", '  <Provider Name="disk"/>', "</System>"].join("\n"));
  });

  it('con lang="xml" colorea por fragmento, cada uno como texto plano dentro de su span', async () => {
    const { container } = await render(CodeOutput, {
      props: { content: "<EventID>157</EventID>", lang: "xml" }
    });
    const pre = container.querySelector("pre");
    const spans = pre?.querySelectorAll("span") ?? [];
    expect(spans.length).toBeGreaterThan(0);
    // Reconstruir el texto de todos los `<span>` reproduce el XML exacto: nada se pierde ni se
    // interpreta como marcado del propio suceso.
    expect(
      Array.from(spans)
        .map((s) => s.textContent)
        .join("")
    ).toBe("<EventID>157</EventID>");
    const nombreEtiqueta = Array.from(spans).find((s) => s.textContent === "EventID");
    expect(nombreEtiqueta?.classList.contains("text-accent-fg")).toBe(true);
  });

  it("copiar en modo xml copia el texto ya reindentado, no el original compacto", async () => {
    const escribir = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue(undefined);
    await render(CodeOutput, { props: { content: "<a><b/></a>", lang: "xml" } });

    await page.getByRole("button", { name: "Copiar" }).click();
    expect(escribir).toHaveBeenCalledWith(["<a>", "  <b/>", "</a>"].join("\n"));
    escribir.mockRestore();
  });
});
