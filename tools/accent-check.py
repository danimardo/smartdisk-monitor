#!/usr/bin/env python3
"""Valida el tratamiento del acento heredado de Windows (open-questions.md I.4).

Windows deja elegir **cualquier** color como acento, no una paleta cerrada, así que la validación
no puede limitarse a unas cuantas muestras: se barre el espacio sRGB completo con un paso fino.

Comprueba las dos formas en que el sistema de diseño usa el acento, que tienen requisitos opuestos:

  1. Como **fondo** (botón primario): necesita contraste contra su propio texto.
     Lo resuelve `accessibleAccent()` en src/lib/design/accent.ts.
  2. Como **texto** (enlaces, selección, serie principal de la gráfica): necesita contraste contra
     el material sobre el que se pinta, que es casi blanco en tema claro y casi negro en oscuro.

    python tools/accent-check.py            barrido con paso 4 (262.144 colores)
    python tools/accent-check.py --fast     paso 16 (4.096 colores)
"""

from __future__ import annotations

import sys

AA = 4.5
AA_LARGE = 3.0  # texto grande y elementos gráficos (WCAG 1.4.11)

RGB = tuple[int, int, int]


# --- Colorimetría WCAG ------------------------------------------------------

def luminance(c: RGB) -> float:
    def lin(v: int) -> float:
        s = v / 255
        return s / 12.92 if s <= 0.03928 else ((s + 0.055) / 1.055) ** 2.4
    r, g, b = (lin(x) for x in c)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: RGB, b: RGB) -> float:
    la, lb = luminance(a), luminance(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def over(fg: RGB, alpha: float, bg: RGB) -> RGB:
    """Color efectivo de una capa translúcida sobre un fondo."""
    return tuple(round(f * alpha + b * (1 - alpha)) for f, b in zip(fg, bg))  # type: ignore[return-value]


def shift(c: RGB, amount: float) -> RGB:
    target = 255 if amount > 0 else 0
    k = abs(amount)
    return tuple(round(v + (target - v) * k) for v in c)  # type: ignore[return-value]


# --- Lo que hace hoy accent.ts ---------------------------------------------

WHITE: RGB = (255, 255, 255)
BLACK: RGB = (17, 17, 20)


def accessible_accent(c: RGB) -> tuple[RGB, RGB, bool]:
    """Traducción fiel de accessibleAccent() de accent.ts."""
    prefers_black = contrast(c, BLACK) > contrast(c, WHITE)
    text = BLACK if prefers_black else WHITE
    if contrast(c, text) >= AA:
        return c, text, False
    direction = 1 if prefers_black else -1
    k = 0.05
    while k <= 0.9:
        cand = shift(c, direction * k)
        if contrast(cand, text) >= AA:
            return cand, text, True
        k += 0.05
    return shift(c, direction * 0.9), text, True


def accent_on_surface(c: RGB, surface: RGB, palette: list[RGB] | None = None) -> RGB:
    """Traducción fiel de accentOnSurface() de accent.ts.

    Prefiere los tonos que Windows ya deriva del acento: son los que el usuario ve en el resto del
    sistema. Solo si ninguno llega a AA se deriva un tono propio.
    """
    if contrast(c, surface) >= AA:
        return c
    surface_is_light = luminance(surface) > 0.5
    lb = luminance(c)
    keep = (lambda p: luminance(p) < lb) if surface_is_light else (lambda p: luminance(p) > lb)
    ordered = sorted(
        (p for p in (palette or []) if keep(p)),
        key=lambda p: abs(luminance(p) - lb),
    )
    for cand in ordered:
        if contrast(cand, surface) >= AA:
            return cand
    direction = -1 if surface_is_light else 1
    k = 0.05
    while k <= 0.95:
        cand = shift(c, direction * k)
        if contrast(cand, surface) >= AA:
            return cand
        k += 0.05
    return shift(c, direction * 0.95)


def readable_on(c: RGB, bg: RGB, target: float = AA) -> tuple[RGB, bool]:
    """Oscurece o aclara el acento hasta que sea legible COMO TEXTO sobre `bg`."""
    if contrast(c, bg) >= target:
        return c, False
    direction = -1 if luminance(bg) > 0.5 else 1  # fondo claro -> oscurecer
    k = 0.05
    while k <= 0.95:
        cand = shift(c, direction * k)
        if contrast(cand, bg) >= target:
            return cand, True
        k += 0.05
    return shift(c, direction * 0.95), True


# --- Materiales efectivos, calculados desde tokens.css ---------------------

# claro: --sdm-bg #e9ebf0 con --sdm-glass rgba(255,255,255,0.72)
LIGHT_SURFACE = over((255, 255, 255), 0.72, (0xE9, 0xEB, 0xF0))
# oscuro: --sdm-bg #101014 con --sdm-glass rgba(42,42,50,0.66)
DARK_SURFACE = over((42, 42, 50), 0.66, (0x10, 0x10, 0x14))


def sweep(step: int):
    for r in range(0, 256, step):
        for g in range(0, 256, step):
            for b in range(0, 256, step):
                yield (r, g, b)


def main() -> int:
    step = 16 if "--fast" in sys.argv else 4
    total = 0
    bg_fail = 0            # el botón primario no llega a AA tras el ajuste
    bg_adjusted = 0        # hubo que retocar el color del usuario
    text_light_fail = 0    # el acento crudo no es legible como texto en tema claro
    text_dark_fail = 0     # idem en tema oscuro
    text_light_fixable = 0
    text_dark_fixable = 0
    worst_shift = 0.0
    worst_color: RGB = (0, 0, 0)

    for c in sweep(step):
        total += 1

        accent, on_accent, adjusted = accessible_accent(c)
        if adjusted:
            bg_adjusted += 1
            d = max(abs(a - b) for a, b in zip(accent, c))
            if d > worst_shift:
                worst_shift, worst_color = d, c
        if contrast(accent, on_accent) < AA:
            bg_fail += 1

        if contrast(c, LIGHT_SURFACE) < AA:
            text_light_fail += 1
            fixed, _ = readable_on(c, LIGHT_SURFACE)
            if contrast(fixed, LIGHT_SURFACE) >= AA:
                text_light_fixable += 1
        if contrast(c, DARK_SURFACE) < AA:
            text_dark_fail += 1
            fixed, _ = readable_on(c, DARK_SURFACE)
            if contrast(fixed, DARK_SURFACE) >= AA:
                text_dark_fixable += 1

    pct = lambda n: f"{n:>7} ({n / total * 100:5.1f} %)"
    print(f"Colores probados: {total} (paso {step})")
    print(f"Superficie clara efectiva: #{'%02x%02x%02x' % LIGHT_SURFACE}")
    print(f"Superficie oscura efectiva: #{'%02x%02x%02x' % DARK_SURFACE}")
    print()
    print("1. Acento como FONDO (botón primario), con accessibleAccent():")
    print(f"   no alcanzan AA .................. {pct(bg_fail)}")
    print(f"   requirieron ajuste del color .... {pct(bg_adjusted)}")
    print(f"   mayor desviación por canal ...... {worst_shift:.0f}/255 "
          f"(en #{'%02x%02x%02x' % worst_color})")
    print()
    print("2. Acento como TEXTO sobre el material, SIN tratar:")
    print(f"   ilegible en tema claro .......... {pct(text_light_fail)}")
    print(f"   ilegible en tema oscuro ......... {pct(text_dark_fail)}")
    print()
    print("3. Acento como TEXTO, tras derivar un tono legible:")
    print(f"   recuperables en tema claro ...... {text_light_fixable} de {text_light_fail}")
    print(f"   recuperables en tema oscuro ..... {text_dark_fixable} de {text_dark_fail}")
    print()

    fg_light_fail = fg_dark_fail = 0
    for c in sweep(step):
        if contrast(accent_on_surface(c, LIGHT_SURFACE), LIGHT_SURFACE) < AA:
            fg_light_fail += 1
        if contrast(accent_on_surface(c, DARK_SURFACE), DARK_SURFACE) < AA:
            fg_dark_fail += 1
    print("4. VERIFICACIÓN de accentOnSurface() (--sdm-accent-fg):")
    print(f"   siguen por debajo de AA en claro  {pct(fg_light_fail)}")
    print(f"   siguen por debajo de AA en oscuro {pct(fg_dark_fail)}")
    print()

    # Casos concretos que conviene mirar a ojo
    print("Casos de referencia:")
    print(f"{'acento':<10} {'fondo btn':<20} {'texto claro':<16} {'texto oscuro'}")
    for name, c in [
        ("azul Win", (0x00, 0x78, 0xD4)),
        ("ámbar", (0xFF, 0xB9, 0x00)),
        ("lima", (0xB1, 0xD9, 0x32)),
        ("amarillo", (0xFF, 0xF0, 0x00)),
        ("verde", (0x00, 0xCC, 0x6A)),
        ("cian claro", (0x4C, 0xC2, 0xFF)),
        ("rojo", (0xE7, 0x48, 0x56)),
        ("negro", (0x00, 0x00, 0x00)),
        ("blanco", (0xFF, 0xFF, 0xFF)),
    ]:
        accent, on_accent, adj = accessible_accent(c)
        cl = contrast(c, LIGHT_SURFACE)
        cd = contrast(c, DARK_SURFACE)
        mark = lambda v: "OK " if v >= AA else "NO "
        print(f"#{'%02x%02x%02x' % c:<9} "
              f"{contrast(accent, on_accent):>4.1f}:1 {'(ajustado)' if adj else '          '}  "
              f"{mark(cl)}{cl:>5.2f}:1      "
              f"{mark(cd)}{cd:>5.2f}:1   {name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
