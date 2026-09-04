#!/usr/bin/env python3
"""Regenera `historias.md`, el consolidado de todo el material normativo del proyecto.

El consolidado existe para poder entregar el proyecto entero como un solo documento —a una
herramienta de generación como SpecKit, o a una persona—. Los ficheros individuales siguen siendo
la referencia editable: este script se ejecuta después de tocarlos, y nunca al revés.

    python tools/build-historias.py            regenera
    python tools/build-historias.py --check    falla si está desactualizado (para CI)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "historias.md"

# (ruta, título de la sección, lenguaje para el bloque de código o None si ya es Markdown)
SOURCES: list[tuple[str, str, str | None]] = [
    # --- Qué es el producto -------------------------------------------------
    ("README.md", "Presentación del proyecto", None),
    ("docs/product-specification.md", "Especificación del producto", None),
    ("docs/user-stories.md", "Historias de usuario", None),
    ("docs/roadmap.md", "Roadmap y backlog", None),
    # --- Cómo se construye --------------------------------------------------
    ("docs/architecture.md", "Arquitectura", None),
    ("docs/data-model.md", "Modelo de datos", None),
    ("docs/alert-rules.md", "Reglas de alerta", None),
    ("docs/ui-contract.md", "Contrato UI ↔ backend", None),
    ("docs/engineering-conventions.md", "Convenciones de ingeniería", None),
    ("docs/testing-strategy.md", "Estrategia integral de testing", None),
    ("docs/decisions.md", "Registro de decisiones técnicas", None),
    ("docs/open-questions.md", "Cuestiones abiertas y mediciones", None),
    # --- Reglas de interfaz, vinculantes ------------------------------------
    # Se leen de `src/`, que es donde vive la ÚNICA copia del sistema de diseño (ADR-029).
    ("docs/ui-design.md", "Sistema de diseño: reglas de interfaz (VINCULANTES)", None),
    ("design/README.md", "Bocetos del sistema de diseño", None),
    ("src/design-system/README.md", "Sistema de diseño: principios", None),
    # --- Contratos literales que el código debe respetar --------------------
    ("src/design-system/tokens.css", "tokens.css — fuente única de verdad visual", "css"),
    ("src/design-system/tokens.json", "tokens.json — los mismos tokens, para herramientas", "json"),
    ("tailwind.config.cjs", "tailwind.config.cjs — mapeo de tokens", "js"),
    ("src/lib/components/index.ts", "components/index.ts — el catálogo cerrado", "ts"),
    ("src/lib/design/types.ts", "design/types.ts — vocabulario de la UI", "ts"),
    ("src/lib/design/health.ts", "design/health.ts — estado → color, umbrales", "ts"),
    ("src/lib/design/format.ts", "design/format.ts — formato de presentación", "ts"),
    ("src/lib/design/theme.svelte.ts", "design/theme.svelte.ts — tema", "ts"),
    ("src/lib/design/accent.ts", "design/accent.ts — acento de Windows", "ts"),
    ("src/lib/i18n/index.ts", "i18n/index.ts — idioma, formato y plurales", "ts"),
    ("src/lib/i18n/es.json", "i18n/es.json", "json"),
    ("src/lib/i18n/en.json", "i18n/en.json", "json"),
    # --- Recursos redistribuidos y licencias --------------------------------
    ("src/design-system/fonts/README.md", "Tipografía empotrada", None),
    ("third-party/smartmontools/README.md", "smartctl redistribuido", None),
    ("LICENSE", "Licencia del código propio", "text"),
    ("THIRD_PARTY_NOTICES.md", "Avisos de terceros", None),
]

HEADER = """# SmartDisk Monitor — documentación consolidada

Reúne **todo el material normativo** del proyecto en un solo documento: especificación funcional,
arquitectura, modelo de datos, reglas de alerta, contrato entre interfaz y backend, decisiones
técnicas, reglas de interfaz y los ficheros que actúan como contrato literal (tokens, tipos,
formateadores e i18n).

Se genera con `python tools/build-historias.py` a partir de los ficheros del repositorio, que siguen
siendo la referencia editable. **No lo edites a mano**: los cambios se perderían en la siguiente
regeneración.

## Cómo leerlo

| Si buscas… | Ve a |
|---|---|
| Qué hace el producto y qué queda fuera | Especificación del producto |
| Qué hay que construir, con criterios de aceptación | Historias de usuario |
| Cómo se estructura por dentro | Arquitectura, Modelo de datos |
| Cuándo salta una alerta y cuándo se resuelve | Reglas de alerta |
| Qué comandos y eventos existen entre UI y backend | Contrato UI ↔ backend |
| Por qué se decidió algo | Registro de decisiones |
| Qué se midió y qué sigue sin decidirse | Cuestiones abiertas y mediciones |
| Cómo se escribe una pantalla | Sistema de diseño: reglas de interfaz (VINCULANTES) |
| Dónde vive cada pieza del sistema de diseño | Sistema de diseño, §0 «Dónde vive cada cosa» |
| Qué hay que probar, a qué nivel y cuándo | Estrategia integral de testing |

## Precedencia

Si dos documentos se contradicen, mandan en este orden:

1. **Reglas de alerta** sobre el resumen de alertas de la especificación.
2. **Contrato UI ↔ backend** sobre cualquier descripción informal de comandos.
3. **Sistema de diseño** (`docs/ui-design.md`) sobre cualquier criterio visual escrito en otro sitio.
4. **Cuestiones abiertas** sobre todo lo demás para lo que registre una decisión: recoge las
   correcciones posteriores, varias de ellas nacidas de medir sobre un Windows real.

Las entradas marcadas `PROPUESTO` en Cuestiones abiertas son valores por defecto adoptados para no
bloquear el trabajo, no decisiones cerradas. Las marcadas `ABIERTO` bloquean la historia que las
cita. Ninguna implementación debería tener que asumir nada que no esté ahí; si aparece algo, se
añade en vez de resolverlo en el código.
"""

LINK = re.compile(r"\]\((?!https?://|/|#)([^)]+)\)")


def rebase_links(text: str, rel: str) -> str:
    """Reescribe los enlaces relativos para que sigan resolviendo desde la raíz del repositorio.

    Un `[reglas](alert-rules.md)` escrito dentro de `docs/` apunta a `docs/alert-rules.md` una vez
    consolidado en la raíz. Solo se reescribe si el destino existe de verdad desde la carpeta del
    fichero: así una referencia que ya era relativa a la raíz se queda como está.
    """
    folder = Path(rel).parent
    if folder in (Path(""), Path(".")):
        return text

    def fix(m: re.Match[str]) -> str:
        target = m.group(1)
        if (ROOT / folder / target).exists():
            return "](" + (folder / target).as_posix() + ")"
        return m.group(0)

    return LINK.sub(fix, text)


def demote_headings(text: str) -> str:
    """Baja un nivel todos los encabezados y quita el título propio del fichero.

    Cada fichero incluido trae su propio `# Título`, que chocaría con el `# N. Sección` del
    consolidado y dejaría dos H1 seguidos. Bajando un nivel, todo el contenido del fichero cuelga
    de su sección y el índice del documento se puede navegar de verdad.

    Los encabezados dentro de un bloque de código se dejan en paz: en un `.md` con ejemplos de
    shell, una línea que empieza por `#` es un comentario, no un título.
    """
    out: list[str] = []
    in_fence = False
    seen_title = False
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_fence = not in_fence
            out.append(line)
            continue
        if not in_fence and re.match(r"^#{1,5} ", line):
            if not seen_title and line.startswith("# "):
                seen_title = True   # el título del fichero lo sustituye el de la sección
                continue
            out.append("#" + line)
            continue
        out.append(line)
    return "\n".join(out).strip()


def anchor(title: str) -> str:
    """Ancla estilo GitHub para el índice."""
    a = title.lower()
    a = re.sub(r"[^\w\s-]", "", a, flags=re.UNICODE)
    return re.sub(r"\s+", "-", a.strip())


def build() -> str:
    parts = [HEADER, "\n## Índice\n"]

    for i, (rel, title, _) in enumerate(SOURCES, 1):
        parts.append(f"{i}. [{title}](#{i}-{anchor(title)}) · `{rel}`")

    parts.append("")

    for i, (rel, title, lang) in enumerate(SOURCES, 1):
        path = ROOT / rel
        if not path.exists():
            raise SystemExit(f"falta el fichero de origen: {rel}")
        body = path.read_text(encoding="utf-8").strip()
        parts.append(f"\n---\n\n# {i}. {title}\n\nFichero de origen: `{rel}`\n")
        if lang is None:
            parts.append(demote_headings(rebase_links(body, rel)) + "\n")
        else:
            parts.append(f"```{lang}\n{body}\n```\n")

    return "\n".join(parts)


def main() -> int:
    content = build()
    if "--check" in sys.argv:
        current = OUTPUT.read_text(encoding="utf-8") if OUTPUT.exists() else ""
        if current != content:
            print("historias.md está desactualizado: ejecuta python tools/build-historias.py")
            return 1
        print("historias.md está al día")
        return 0
    OUTPUT.write_text(content, encoding="utf-8", newline="\n")
    lines = content.count("\n") + 1
    print(f"historias.md regenerado desde {len(SOURCES)} ficheros ({lines} líneas)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
