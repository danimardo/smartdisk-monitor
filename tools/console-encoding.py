#!/usr/bin/env python3
"""Detección de la codificación de la salida de una herramienta de consola de Windows
(open-questions.md I.3).

El problema, medido: **las herramientas del propio Windows no coinciden entre sí.** En el mismo
equipo, con el mismo tipo de tubería y sin consola adjunta:

    chkdsk, chkntfs    ->  CP1252 (ANSI)   bytes E1 E9 ED F1 F3 FA = áéíñóú
    fsutil, vssadmin   ->  CP850  (OEM)    bytes A0 A1 A2 A3 A4    = áíóúñ

Y la vía "obvia" no sirve: fijar `chcp` antes de invocar **no cambia nada** cuando la salida está
redirigida. Se comprobó con chcp 850 y 65001, y los tres volcados salieron byte a byte idénticos.

Por eso no se puede codificar una constante: hay que detectar. Esta es la heurística de referencia,
validada contra las cuatro herramientas de arriba.

    python tools/console-encoding.py <fichero.bin> [...]
"""

from __future__ import annotations

import sys
from pathlib import Path

# Letras que un texto real en español o inglés produce de verdad.
ESPERADAS = set("áéíóúüñÁÉÍÓÚÜÑ¿¡ºª°·—€“”‘’")
# Lo que sale cuando se acierta la familia pero no la página: griego, dibujo de cajas, matemáticas.
SOSPECHOSAS = set("ßÚÝ±¾·░▒▓│┤╡╢╖╕╣║╗╝┐└┴┬├─┼╞╟╚╔╩╦╠═╬¤ðÐÊËÈıÍÎÏ┘┌█▄▌▐▀αΓπΣσµτΦΘΩδ∞φε∩")

CANDIDATAS = ("utf-8-sig", "utf-8", "cp1252", "cp850", "cp437")


def puntuar(texto: str) -> int:
    """Cuantas más letras plausibles y menos símbolos raros, mejor."""
    return sum(1 for c in texto if c in ESPERADAS) - 3 * sum(1 for c in texto if c in SOSPECHOSAS)


def detectar(datos: bytes) -> tuple[str, str, dict[str, int]]:
    """Devuelve (codificación elegida, texto decodificado, puntuaciones).

    Orden de decisión:
      1. Un BOM manda: es una declaración explícita, no una conjetura.
      2. Si todo es ASCII, cualquier codificación vale: se elige utf-8 y no se adivina nada.
      3. Si decodifica como UTF-8 estricto, es UTF-8. Cubre los sistemas con el modo
         "Beta: usar Unicode UTF-8", donde la ANSI del sistema pasa a ser 65001.
      4. Si no, se puntúan las páginas de un byte y gana la que produzca texto plausible.
      5. Empate o puntuación nula: se usa la ANSI del sistema, que es lo que emite chkdsk.
    """
    if datos.startswith(b"\xef\xbb\xbf"):
        return "utf-8-sig", datos.decode("utf-8-sig", errors="replace"), {}
    if datos[:2] in (b"\xff\xfe", b"\xfe\xff"):
        enc = "utf-16"
        return enc, datos.decode(enc, errors="replace"), {}

    if not any(b > 127 for b in datos):
        return "ascii", datos.decode("ascii"), {}

    try:
        return "utf-8", datos.decode("utf-8"), {}
    except UnicodeDecodeError:
        pass

    puntuaciones: dict[str, int] = {}
    mejor, mejor_texto, mejor_punt = None, "", -10**9
    for enc in ("cp1252", "cp850", "cp437"):
        try:
            texto = datos.decode(enc)
        except UnicodeDecodeError:
            continue
        p = puntuar(texto)
        puntuaciones[enc] = p
        if p > mejor_punt:
            mejor, mejor_texto, mejor_punt = enc, texto, p

    if mejor is None or mejor_punt <= 0:
        # Sin señal clara: la ANSI del sistema. Nunca se falla ni se pierde la salida.
        return "cp1252", datos.decode("cp1252", errors="replace"), puntuaciones
    return mejor, mejor_texto, puntuaciones


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    for ruta in argv[1:]:
        datos = Path(ruta).read_bytes()
        enc, texto, punt = detectar(datos)
        muestra = next(
            (l.strip() for l in texto.splitlines() if any(c in l for c in ESPERADAS)), ""
        )
        detalle = "  ".join(f"{k}={v}" for k, v in punt.items()) or "(decidido sin puntuar)"
        print(f"{Path(ruta).name:<14} -> {enc:<10} [{detalle}]")
        if muestra:
            print(f"{'':<17} {muestra[:88]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
