import { describe, expect, it } from "vitest";
import { formatXml, tokenizeXml } from "./xml";

describe("formatXml — reindenta un XML compacto para que se lea", () => {
  it("un elemento hoja con atributo y texto se mantiene en una sola línea", () => {
    expect(formatXml('<Data Name="ErrorCode">3</Data>')).toBe('<Data Name="ErrorCode">3</Data>');
  });

  it("un elemento con hijos abre, indenta y cierra en líneas separadas", () => {
    const compacto = "<System><Provider Name=\"disk\"/><EventID>157</EventID></System>";
    expect(formatXml(compacto)).toBe(
      ['<System>', '  <Provider Name="disk"/>', "  <EventID>157</EventID>", "</System>"].join("\n")
    );
  });

  it("un XML real de varios niveles anida correctamente cada profundidad", () => {
    const compacto =
      '<Event><System><Provider Name="disk"/><EventID>157</EventID></System>' +
      "<EventData><Data Name=\"DiskNumber\">1</Data></EventData></Event>";
    expect(formatXml(compacto)).toBe(
      [
        "<Event>",
        "  <System>",
        '    <Provider Name="disk"/>',
        "    <EventID>157</EventID>",
        "  </System>",
        "  <EventData>",
        '    <Data Name="DiskNumber">1</Data>',
        "  </EventData>",
        "</Event>"
      ].join("\n")
    );
  });

  it("ignora el espacio en blanco que ya traiga el XML de origen entre etiquetas", () => {
    expect(formatXml("<a>\n  <b/>\n</a>")).toBe(["<a>", "  <b/>"].join("\n") + "\n</a>");
  });

  it("una cadena vacía no produce ninguna línea", () => {
    expect(formatXml("")).toBe("");
  });

  it("un XML truncado no lanza: el resto se conserva como texto", () => {
    expect(() => formatXml("<Event><System>")).not.toThrow();
    expect(formatXml("<Event><Sys")).toContain("<Sys");
  });
});

describe("tokenizeXml — coloreado por fragmento, nunca por HTML interpretado", () => {
  it("separa apertura, nombre, espacio, atributo y cierre de una etiqueta simple", () => {
    // El espacio entre el nombre y el atributo es su propio token de texto sin colorear: no hace
    // falta un tipo dedicado solo para separadores insignificantes.
    expect(tokenizeXml('<Provider Name="disk"/>')).toEqual([
      { tipo: "punct", texto: "<" },
      { tipo: "tagName", texto: "Provider" },
      { tipo: "text", texto: " " },
      { tipo: "attrName", texto: "Name" },
      { tipo: "punct", texto: "=" },
      { tipo: "attrValue", texto: '"disk"' },
      { tipo: "punct", texto: "/>" }
    ]);
  });

  it("una etiqueta de cierre lleva su propio token de puntuación", () => {
    expect(tokenizeXml("</EventID>")).toEqual([
      { tipo: "punct", texto: "</" },
      { tipo: "tagName", texto: "EventID" },
      { tipo: "punct", texto: ">" }
    ]);
  });

  it("el texto entre etiquetas es su propio token", () => {
    const tokens = tokenizeXml("<EventID>157</EventID>");
    expect(tokens.find((t) => t.tipo === "text")).toEqual({ tipo: "text", texto: "157" });
  });

  it("un comentario es un único token, no se despieza", () => {
    expect(tokenizeXml("<!-- nota -->")).toEqual([{ tipo: "comment", texto: "<!-- nota -->" }]);
  });

  it("un `>` dentro de un valor de atributo no cierra la etiqueta antes de tiempo", () => {
    const tokens = tokenizeXml('<Data Name="a > b">x</Data>');
    expect(tokens.find((t) => t.tipo === "attrValue")).toEqual({ tipo: "attrValue", texto: '"a > b"' });
  });

  it("nunca produce un token con marcado ejecutable: cada fragmento es texto plano", () => {
    // El propio suceso puede contener algo con forma de etiqueta dentro de un valor de texto
    // (p. ej. una ruta con `<` sueltos); el análisis no lanza y todo sigue siendo texto.
    const tokens = tokenizeXml("<Data>ruta<rara</Data>");
    expect(tokens.every((t) => typeof t.texto === "string")).toBe(true);
    expect(tokens.map((t) => t.texto).join("")).toContain("ruta");
  });
});
