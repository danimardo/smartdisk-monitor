import { describe, expect, it } from "vitest";
import { parseInline, parseMarkdown } from "./markdown";

/** La respuesta de un LLM es contenido no confiable (principio XVI). Estas pruebas verifican que
 *  el analizador reconoce el subconjunto esperado y —lo importante— que cualquier intento de
 *  colar marcado activo sale como texto inerte. `Markdown.svelte` nunca usa `{@html}`. */

describe("parseMarkdown — bloques", () => {
  it("un encabezado ## produce un bloque de nivel 2", () => {
    const [b] = parseMarkdown("## Qué significa");
    expect(b).toMatchObject({ tipo: "encabezado", nivel: 2 });
  });

  it("los # más allá de 3 se topan en nivel 3", () => {
    const [b] = parseMarkdown("##### hondo");
    expect(b).toMatchObject({ tipo: "encabezado", nivel: 3 });
  });

  it("líneas '- ' consecutivas son una sola lista no ordenada", () => {
    const [b] = parseMarkdown("- uno\n- dos\n- tres");
    expect(b).toMatchObject({ tipo: "lista", ordenada: false });
    if (b.tipo === "lista") expect(b.items).toHaveLength(3);
  });

  it("líneas '1. ' consecutivas son una lista ordenada", () => {
    const [b] = parseMarkdown("1. primero\n2. segundo");
    expect(b).toMatchObject({ tipo: "lista", ordenada: true });
  });

  it("un bloque ``` conserva el contenido literal, sin interpretarlo", () => {
    const [b] = parseMarkdown("```\nsmartctl -a -j\n**no es negrita aquí**\n```");
    expect(b).toEqual({ tipo: "codigo", valor: "smartctl -a -j\n**no es negrita aquí**" });
  });

  it("líneas '> ' consecutivas son una cita", () => {
    const [b] = parseMarkdown("> ojo con esto\n> y con esto");
    expect(b.tipo).toBe("cita");
  });

  it("dos párrafos separados por una línea en blanco son dos bloques", () => {
    const bloques = parseMarkdown("Primer párrafo.\n\nSegundo párrafo.");
    expect(bloques).toHaveLength(2);
    expect(bloques.every((b) => b.tipo === "parrafo")).toBe(true);
  });

  it("normaliza saltos de línea CRLF", () => {
    const bloques = parseMarkdown("uno\r\n\r\ndos");
    expect(bloques).toHaveLength(2);
  });
});

describe("parseInline", () => {
  it("**texto** es negrita", () => {
    expect(parseInline("hola **mundo**")).toEqual([
      { tipo: "texto", valor: "hola " },
      { tipo: "negrita", hijos: [{ tipo: "texto", valor: "mundo" }] }
    ]);
  });

  it("*texto* es cursiva y no se confunde con **", () => {
    const r = parseInline("un *matiz* y un **peso**");
    expect(r.some((n) => n.tipo === "cursiva")).toBe(true);
    expect(r.some((n) => n.tipo === "negrita")).toBe(true);
  });

  it("`texto` es código en línea literal", () => {
    expect(parseInline("mira `Reallocated_Sector_Ct`")).toContainEqual({
      tipo: "codigo",
      valor: "Reallocated_Sector_Ct"
    });
  });

  it("un enlace guarda texto y url por separado (la url no se usa como href)", () => {
    expect(parseInline("ver [la guía](https://example.com/x)")).toContainEqual({
      tipo: "enlace",
      texto: "la guía",
      url: "https://example.com/x"
    });
  });

  it("un ** sin cierre se queda como texto, no rompe nada", () => {
    expect(parseInline("esto **no cierra")).toEqual([{ tipo: "texto", valor: "esto **no cierra" }]);
  });
});

describe("contenido hostil sale inerte", () => {
  it("una etiqueta <img onerror> es texto, no un nodo de imagen", () => {
    const [b] = parseMarkdown('<img src=x onerror="alert(1)">');
    expect(b.tipo).toBe("parrafo");
    if (b.tipo === "parrafo") {
      expect(b.hijos).toEqual([{ tipo: "texto", valor: '<img src=x onerror="alert(1)">' }]);
    }
  });

  it("un <script> es texto plano", () => {
    const [b] = parseMarkdown("<script>fetch('/rob')</script>");
    expect(b.tipo).toBe("parrafo");
    if (b.tipo === "parrafo") {
      expect(b.hijos.every((n) => n.tipo === "texto")).toBe(true);
    }
  });

  it("un enlace con javascript: se guarda como dato, nunca se ejecuta ni navega", () => {
    // El componente jamás lo pone en un href; aquí solo se comprueba que no se pierde ni se trata
    // distinto de cualquier otra url.
    const r = parseInline("[pulsa](javascript:stealCookies)");
    expect(r).toContainEqual({ tipo: "enlace", texto: "pulsa", url: "javascript:stealCookies" });
  });

  it("las entidades HTML no se decodifican", () => {
    const [b] = parseMarkdown("&lt;b&gt;negrita&lt;/b&gt;");
    if (b.tipo === "parrafo") {
      expect(b.hijos).toEqual([{ tipo: "texto", valor: "&lt;b&gt;negrita&lt;/b&gt;" }]);
    }
  });
});
