/** Analizador de un **subconjunto** de Markdown a un árbol de tokens (spec 005-explicacion-ia).
 *
 *  La respuesta de un LLM es **contenido no confiable** (principio XVI): este módulo nunca produce
 *  HTML ni deja pasar etiquetas. Todo lo que no reconoce se convierte en texto literal, que el
 *  componente `Markdown.svelte` renderiza con interpolación normal de Svelte (que escapa). No hay
 *  ninguna vía a `{@html}`.
 *
 *  Subconjunto soportado:
 *  - Bloques: párrafo, encabezados `#`–`###`, listas `-`/`*`/`1.`, bloque de código ```` ``` ````,
 *    cita `>`.
 *  - En línea: `**negrita**`, `*cursiva*`, `` `código` ``, enlace `[texto](url)` (se muestra solo
 *    el texto; la URL va entre paréntesis como texto plano, nunca como `href`).
 */

export type Inline =
  | { tipo: "texto"; valor: string }
  | { tipo: "negrita"; hijos: Inline[] }
  | { tipo: "cursiva"; hijos: Inline[] }
  | { tipo: "codigo"; valor: string }
  | { tipo: "enlace"; texto: string; url: string };

export type Bloque =
  | { tipo: "parrafo"; hijos: Inline[] }
  | { tipo: "encabezado"; nivel: 1 | 2 | 3; hijos: Inline[] }
  | { tipo: "lista"; ordenada: boolean; items: Inline[][] }
  | { tipo: "codigo"; valor: string }
  | { tipo: "cita"; hijos: Inline[] };

const RE_ENCABEZADO = /^(#{1,6})\s+(.*)$/;
const RE_VINETA = /^[-*]\s+(.*)$/;
const RE_ORDINAL = /^\d+\.\s+(.*)$/;
const RE_CITA = /^>\s?(.*)$/;
const RE_CERCA = /^```/;

export function parseMarkdown(src: string): Bloque[] {
  const lineas = src.replace(/\r\n?/g, "\n").split("\n");
  const bloques: Bloque[] = [];
  let i = 0;

  while (i < lineas.length) {
    const linea = lineas[i];

    if (linea.trim() === "") {
      i++;
      continue;
    }

    // Bloque de código con cerca ```
    if (RE_CERCA.test(linea)) {
      const cuerpo: string[] = [];
      i++;
      while (i < lineas.length && !RE_CERCA.test(lineas[i])) {
        cuerpo.push(lineas[i]);
        i++;
      }
      i++; // salta la cerca de cierre (o el final)
      bloques.push({ tipo: "codigo", valor: cuerpo.join("\n") });
      continue;
    }

    // Encabezado
    const enc = linea.match(RE_ENCABEZADO);
    if (enc) {
      const nivel = Math.min(enc[1].length, 3) as 1 | 2 | 3;
      bloques.push({ tipo: "encabezado", nivel, hijos: parseInline(enc[2]) });
      i++;
      continue;
    }

    // Lista (viñeta u ordinal): líneas consecutivas del mismo tipo
    const esVineta = RE_VINETA.test(linea);
    const esOrdinal = RE_ORDINAL.test(linea);
    if (esVineta || esOrdinal) {
      const re = esVineta ? RE_VINETA : RE_ORDINAL;
      const items: Inline[][] = [];
      while (i < lineas.length) {
        const m = lineas[i].match(re);
        if (!m) break;
        items.push(parseInline(m[1]));
        i++;
      }
      bloques.push({ tipo: "lista", ordenada: esOrdinal, items });
      continue;
    }

    // Cita: líneas consecutivas que empiezan por >
    if (RE_CITA.test(linea)) {
      const texto: string[] = [];
      while (i < lineas.length) {
        const m = lineas[i].match(RE_CITA);
        if (!m) break;
        texto.push(m[1]);
        i++;
      }
      bloques.push({ tipo: "cita", hijos: parseInline(texto.join(" ")) });
      continue;
    }

    // Párrafo: líneas consecutivas no vacías que no abren otro bloque
    const parrafo: string[] = [];
    while (i < lineas.length) {
      const l = lineas[i];
      if (
        l.trim() === "" ||
        RE_CERCA.test(l) ||
        RE_ENCABEZADO.test(l) ||
        RE_VINETA.test(l) ||
        RE_ORDINAL.test(l) ||
        RE_CITA.test(l)
      ) {
        break;
      }
      parrafo.push(l);
      i++;
    }
    bloques.push({ tipo: "parrafo", hijos: parseInline(parrafo.join(" ")) });
  }

  return bloques;
}

/** Analiza el contenido en línea de un fragmento. No anida negrita dentro de negrita ni cursiva
 *  dentro de cursiva; sí permite texto dentro de cualquiera de ellas. */
export function parseInline(src: string): Inline[] {
  const salida: Inline[] = [];
  let buffer = "";
  let i = 0;

  const volcarTexto = () => {
    if (buffer) {
      salida.push({ tipo: "texto", valor: buffer });
      buffer = "";
    }
  };

  while (i < src.length) {
    const resto = src.slice(i);

    // Código en línea: `...`
    if (src[i] === "`") {
      const cierre = src.indexOf("`", i + 1);
      if (cierre !== -1) {
        volcarTexto();
        salida.push({ tipo: "codigo", valor: src.slice(i + 1, cierre) });
        i = cierre + 1;
        continue;
      }
    }

    // Negrita: **...**
    if (resto.startsWith("**")) {
      const cierre = src.indexOf("**", i + 2);
      if (cierre !== -1) {
        volcarTexto();
        salida.push({ tipo: "negrita", hijos: parseInline(src.slice(i + 2, cierre)) });
        i = cierre + 2;
        continue;
      }
    }

    // Cursiva: *...* (un solo asterisco, no seguido de otro)
    if (src[i] === "*" && src[i + 1] !== "*") {
      const cierre = src.indexOf("*", i + 1);
      if (cierre !== -1) {
        volcarTexto();
        salida.push({ tipo: "cursiva", hijos: parseInline(src.slice(i + 1, cierre)) });
        i = cierre + 1;
        continue;
      }
    }

    // Enlace: [texto](url)
    if (src[i] === "[") {
      const finTexto = src.indexOf("]", i + 1);
      if (finTexto !== -1 && src[finTexto + 1] === "(") {
        const finUrl = src.indexOf(")", finTexto + 2);
        if (finUrl !== -1) {
          volcarTexto();
          salida.push({
            tipo: "enlace",
            texto: src.slice(i + 1, finTexto),
            url: src.slice(finTexto + 2, finUrl)
          });
          i = finUrl + 1;
          continue;
        }
      }
    }

    buffer += src[i];
    i++;
  }

  volcarTexto();
  return salida;
}
