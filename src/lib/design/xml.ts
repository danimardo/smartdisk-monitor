/** Formateo y resaltado de sintaxis de XML para la vista de detalle de un evento de Windows. El
 *  XML que entrega `get_event_raw_xml` llega compacto (sin indentar), así que se reparte en líneas
 *  legibles antes de mostrarse (`formatXml`) y se colorea por token (`tokenizeXml`).
 *
 *  Analizador propio y deliberadamente simple — sin librería nueva (la pila del proyecto es fija):
 *  no valida el documento, solo lo redistribuye para que se lea. Un XML truncado o mal formado se
 *  degrada a texto plano en el punto del error, nunca lanza.
 *
 *  Cada token se renderiza más tarde con interpolación de texto normal de Svelte (`{tok.texto}`
 *  dentro de un `<span>`), nunca con `{@html}`: sigue cumpliendo que el contenido de un suceso se
 *  presenta como texto, jamás como HTML (`ui-design.md`), aunque se coloree por fragmento. */

export type TipoTokenXml = "punct" | "tagName" | "attrName" | "attrValue" | "text" | "comment";

export interface TokenXml {
  tipo: TipoTokenXml;
  texto: string;
}

/** Encuentra el `>` que cierra una etiqueta empezando en `desde` (que debe apuntar a un `<`),
 *  respetando comillas: un `>` dentro de un valor de atributo no cuenta. `-1` si no hay uno bien
 *  formado (XML truncado). */
function finDeEtiqueta(xml: string, desde: number): number {
  let comilla: string | null = null;
  for (let i = desde; i < xml.length; i++) {
    const c = xml[i];
    if (comilla) {
      if (c === comilla) comilla = null;
    } else if (c === '"' || c === "'") {
      comilla = c;
    } else if (c === ">") {
      return i;
    }
  }
  return -1;
}

const RE_ATRIBUTO = /([A-Za-z_][\w:.-]*)(\s*=\s*)("[^"]*"|'[^']*')/g;
const RE_NOMBRE_ETIQUETA = /^<(\/?)([A-Za-z_][\w:.-]*)/;

/** Reparte el interior de una etiqueta ya delimitada (`<...>` completo) en sus tokens: apertura,
 *  nombre, cada par atributo/valor y el cierre. Una etiqueta que no encaja en el patrón esperado
 *  (declaración `<?xml ...?>`, `<!DOCTYPE ...>`) se degrada a un único token de puntuación: se ve
 *  igualmente, solo que sin colorear por dentro. */
function tokenizarEtiqueta(etiqueta: string): TokenXml[] {
  const nombre = RE_NOMBRE_ETIQUETA.exec(etiqueta);
  if (!nombre) return [{ tipo: "punct", texto: etiqueta }];

  const tokens: TokenXml[] = [
    { tipo: "punct", texto: nombre[1] ? "</" : "<" },
    { tipo: "tagName", texto: nombre[2] }
  ];
  const cierre = etiqueta.endsWith("/>") ? "/>" : ">";
  const cuerpo = etiqueta.slice(nombre[0].length, etiqueta.length - cierre.length);

  RE_ATRIBUTO.lastIndex = 0;
  let ultimo = 0;
  let m: RegExpExecArray | null;
  while ((m = RE_ATRIBUTO.exec(cuerpo))) {
    if (m.index > ultimo) tokens.push({ tipo: "text", texto: cuerpo.slice(ultimo, m.index) });
    tokens.push({ tipo: "attrName", texto: m[1] });
    tokens.push({ tipo: "punct", texto: m[2] });
    tokens.push({ tipo: "attrValue", texto: m[3] });
    ultimo = m.index + m[0].length;
  }
  if (ultimo < cuerpo.length) tokens.push({ tipo: "text", texto: cuerpo.slice(ultimo) });
  tokens.push({ tipo: "punct", texto: cierre });
  return tokens;
}

/** Analiza el XML completo en tokens léxicos, para colorear la salida. */
export function tokenizeXml(xml: string): TokenXml[] {
  const tokens: TokenXml[] = [];
  let i = 0;
  while (i < xml.length) {
    if (xml.startsWith("<!--", i)) {
      const fin = xml.indexOf("-->", i + 4);
      const hasta = fin === -1 ? xml.length : fin + 3;
      tokens.push({ tipo: "comment", texto: xml.slice(i, hasta) });
      i = hasta;
    } else if (xml[i] === "<") {
      const fin = finDeEtiqueta(xml, i);
      if (fin === -1) {
        tokens.push({ tipo: "text", texto: xml.slice(i) });
        break;
      }
      tokens.push(...tokenizarEtiqueta(xml.slice(i, fin + 1)));
      i = fin + 1;
    } else {
      const siguiente = xml.indexOf("<", i);
      const hasta = siguiente === -1 ? xml.length : siguiente;
      tokens.push({ tipo: "text", texto: xml.slice(i, hasta) });
      i = hasta;
    }
  }
  return tokens;
}

type UnidadXml = {
  tipo: "open" | "close" | "selfclose" | "comment" | "text";
  raw: string;
  nombre: string | null;
};

/** Reagrupa los tokens léxicos en unidades estructurales (una etiqueta completa, un comentario o
 *  un fragmento de texto), para poder decidir la indentación sin reanalizar la cadena. */
function segmentarXml(xml: string): UnidadXml[] {
  const tokens = tokenizeXml(xml);
  const unidades: UnidadXml[] = [];
  let i = 0;
  while (i < tokens.length) {
    const t = tokens[i];
    if (t.tipo === "comment" || t.tipo === "text") {
      unidades.push({ tipo: t.tipo, raw: t.texto, nombre: null });
      i++;
      continue;
    }
    if (t.tipo === "punct" && (t.texto === "<" || t.texto === "</")) {
      const esCierre = t.texto === "</";
      let raw = t.texto;
      let nombre: string | null = null;
      let j = i + 1;
      if (tokens[j]?.tipo === "tagName") {
        nombre = tokens[j].texto;
        raw += tokens[j].texto;
        j++;
      }
      while (j < tokens.length && !(tokens[j].tipo === "punct" && (tokens[j].texto === ">" || tokens[j].texto === "/>"))) {
        raw += tokens[j].texto;
        j++;
      }
      const cierre = tokens[j]?.texto ?? ">";
      raw += cierre;
      j++;
      unidades.push({ tipo: esCierre ? "close" : cierre === "/>" ? "selfclose" : "open", raw, nombre });
      i = j;
      continue;
    }
    // Una declaración (`<?xml ...?>`) u otro `punct` suelto que `tokenizarEtiqueta` degradó entero:
    // se conserva como texto para no perderlo, sin intentar indentarlo como si fuera una etiqueta.
    unidades.push({ tipo: "text", raw: t.texto, nombre: null });
    i++;
  }
  return unidades;
}

/** Reindenta un XML compacto en líneas legibles, dos espacios por nivel. Un elemento hoja simple
 *  (`<Data Name="X">3</Data>`, sin hijos) se mantiene en una sola línea, como hace cualquier
 *  formateador de XML habitual; un elemento con hijos abre su etiqueta, indenta el contenido y
 *  cierra en su propia línea. El texto que es solo espacio en blanco (indentación previa) se
 *  descarta: la indentación la decide esta función, no la del XML de origen. */
export function formatXml(xml: string, indent = "  "): string {
  const unidades = segmentarXml(xml).filter((u) => !(u.tipo === "text" && u.raw.trim() === ""));
  const lineas: string[] = [];
  let profundidad = 0;
  let i = 0;
  while (i < unidades.length) {
    const u = unidades[i];
    if (u.tipo === "open") {
      const siguiente = unidades[i + 1];
      const tercera = unidades[i + 2];
      if (siguiente?.tipo === "text" && tercera?.tipo === "close" && tercera.nombre === u.nombre) {
        lineas.push(indent.repeat(profundidad) + u.raw + siguiente.raw + tercera.raw);
        i += 3;
        continue;
      }
      lineas.push(indent.repeat(profundidad) + u.raw);
      profundidad++;
      i++;
      continue;
    }
    if (u.tipo === "close") {
      profundidad = Math.max(0, profundidad - 1);
      lineas.push(indent.repeat(profundidad) + u.raw);
      i++;
      continue;
    }
    lineas.push(indent.repeat(profundidad) + u.raw);
    i++;
  }
  return lineas.join("\n");
}
