# 00 · Tokens

Todo valor visual del rediseño está aquí. Fuera de esta tabla no hay ni un literal.

Los ratios están **medidos sobre el material compuesto**, no sobre el fondo sólido: se ha compuesto
`--sdm-glass` sobre la media del degradado del lienzo (`#e9e4ec` en claro, `#161119` en oscuro) y, para
la columna «bloque», `--sdm-glass-3` encima de ese resultado. La columna «píldora» mide el color contra
su propio `-soft` compuesto sobre el material, que es el caso real de `StatusPill`.

Mínimo exigido: **4,5:1**.

---

## 1. Color · tema claro

| Token | Ahora | Propuesto | Ratio material / bloque / píldora | Dónde afecta |
|---|---|---|---|---|
| `--sdm-bg` | `#e9ebf0` | `#efeaf1` | — | lienzo de la ventana |
| `--sdm-bg-2` | `#dfe2ea` | `#e6dfe9` | — | extremo del degradado del lienzo |
| `--sdm-glass-3` | `rgba(120,124,140,.1)` | `rgba(124,114,128,.1)` | — | pistas de barra y bloques internos |
| `--sdm-solid` | `#fdfdfe` | `#fdfcfe` | — | respaldo sin `backdrop-filter` |
| `--sdm-hairline` | `rgba(22,24,32,.09)` | `rgba(30,23,35,.09)` | — | bordes de material |
| `--sdm-scrim` | `rgba(10,10,14,.34)` | `rgba(14,10,16,.36)` | — | velo de diálogo |
| `--sdm-text` | `#191b22` | `#1e1723` | **16,44** / 14,63 / — | texto principal |
| `--sdm-text-dim` | `#5f6371` | `#635a6b` | **6,16** / 5,49 / — | texto secundario |
| `--sdm-text-faint` | `#6a6f7b` | `#6c6274` | **5,43** / 4,84 / — | etiquetas y unidades |
| `--sdm-accent` | `#0067c0` | `#7a3f9d` | **6,53** / 5,81 / 5,46 | acción primaria, selección, serie principal |
| `--sdm-accent-hi` | `#1a7cd4` | `#8b4bb0` | — | extremo claro del degradado del botón |
| `--sdm-accent-soft` | `rgba(0,103,192,.12)` | `rgba(122,63,157,.12)` | — | fondo de píldora y chip de acento |
| `--sdm-ok` | `#2f7256` | *(sin cambio)* | **5,39** / 4,80 / 4,53 | estado correcto |
| `--sdm-warn` | `#7d5619` | *(sin cambio)* | **6,14** / 5,47 / 5,18 | advertencia |
| `--sdm-crit` | `#a83d45` | `#b03434` | **5,83** / 5,19 / 4,86 | crítico |
| `--sdm-crit-soft` | `rgba(168,61,69,.12)` | `rgba(176,52,52,.12)` | — | fondo de píldora crítica |
| `--sdm-unknown` | `#5d616d` | `#635c69` | **6,05** / 5,39 / 5,20 | no compatible / sin datos |
| `--sdm-unknown-soft` | `rgba(93,97,109,.11)` | `rgba(99,92,105,.11)` | — | fondo de píldora neutra |
| `--sdm-shadow` | `…rgba(20,22,30,…)` | `…rgba(24,18,28,…)` | — | sombra de tarjeta (misma geometría) |
| `--sdm-shadow-lift` | `…rgba(20,22,30,…)` | `…rgba(24,18,28,…)` | — | sombra de diálogo y ventana |

Texto blanco sobre el acento sólido: **6,94:1**. `--sdm-on-accent` sigue siendo `#ffffff`.

## 2. Color · tema oscuro

| Token | Ahora | Propuesto | Ratio material / bloque / píldora | Dónde afecta |
|---|---|---|---|---|
| `--sdm-bg` | `#101014` | `#130f16` | — | lienzo |
| `--sdm-bg-2` | `#16161c` | `#19141d` | — | extremo del degradado |
| `--sdm-glass` | `rgba(42,42,50,.66)` | `rgba(48,42,52,.66)` | — | material de tarjeta y chrome |
| `--sdm-glass-2` | `rgba(58,58,68,.42)` | `rgba(64,56,68,.42)` | — | controles secundarios |
| `--sdm-solid` | `#1b1b21` | `#1c1620` | — | respaldo sin `backdrop-filter` |
| `--sdm-scrim` | `rgba(0,0,0,.5)` | `rgba(0,0,0,.52)` | — | velo de diálogo |
| `--sdm-text` | `#f2f2f6` | `#f4f0f6` | **13,86** / 11,61 / — | texto principal |
| `--sdm-text-dim` | `#a2a4b0` | `#a89eb0` | **6,09** / 5,09 / — | texto secundario |
| `--sdm-text-faint` | `#9195a1` | `#a29aa8` | **5,74** / 4,81 / — | etiquetas. Subido: el valor intermedio `#988ea0` daba 4,18 sobre bloque interno |
| `--sdm-on-accent` | `#ffffff` | `#20132a` | 7,80 sobre el acento | **texto sobre el acento**. En blanco daba 2,27:1 — inaceptable |
| `--sdm-accent` | `#3d95ea` | `#c79aec` | **6,89** / 5,77 / 4,86 | acción primaria y selección |
| `--sdm-accent-hi` | `#5aa8f2` | `#d4aef5` | — | extremo claro del degradado |
| `--sdm-accent-soft` | `rgba(61,149,234,.18)` | `rgba(199,154,236,.18)` | — | fondo de píldora de acento |
| `--sdm-ok` | `#6cc79c` | *(sin cambio)* | **7,65** / 6,41 / 5,56 | correcto |
| `--sdm-warn` | `#e0b473` | *(sin cambio)* | **8,14** / 6,82 / 5,79 | advertencia |
| `--sdm-crit` | `#f47a84` | `#ef8080` | **6,00** / 5,02 / 4,59 | crítico |
| `--sdm-crit-soft` | `rgba(244,122,132,.16)` | `rgba(239,128,128,.16)` | — | fondo de píldora crítica |
| `--sdm-unknown` | `#9396a2` | `#a8a0b0` | **6,19** / 5,18 / 4,84 | no compatible. Subido: `#9c94a4` daba 4,28 en píldora |
| `--sdm-unknown-soft` | `rgba(147,150,162,.14)` | `rgba(168,160,176,.14)` | — | fondo de píldora neutra |
| `--sdm-shadow`, `--sdm-shadow-lift` | — | *(sin cambio)* | — | — |

> **Atención al implantar:** `--sdm-on-accent` deja de ser `#ffffff` en oscuro. Cualquier componente que
> escriba `text-white` sobre el acento (en lugar de `text-fg-onAccent`) hay que corregirlo. Afecta al
> menos a `Button` variante `primary`, al logotipo del riel y al icono de la cabecera de `DiskCard`.

## 3. Tipografía

| Token | Ahora | Propuesto | Dónde afecta |
|---|---|---|---|
| `--sdm-font-display` | — (nuevo) | `"Bricolage Grotesque", "Instrument Sans", sans-serif` (claro y oscuro) | **solo** cifras y titulares ≥22 px |
| `--sdm-text-display` | — (nuevo) | `3.625rem` / 58 px | cifra de progreso de prueba |
| `--sdm-text-hero` | — (nuevo) | `4.75rem` / 76 px | temperatura del héroe del panel |
| `--sdm-tracking-display` | — (nuevo) | `-0.04em` | acompaña a los dos anteriores |

Se añade la utilidad `.sdm-display` en `tokens.css` (familia + 600 + `tabular-nums` + tracking + `line-height:1`),
para no repetir las cuatro propiedades. **No se toca** la escala existente ni los pesos: sigue habiendo un
máximo de 600 y no existe la negrita 700.

La fuente hay que **empaquetarla localmente** (`woff2`, pesos 500-700, subconjunto latino): la aplicación
es de escritorio y no debe depender de Google Fonts en tiempo de ejecución. Con `font-display: swap` y el
respaldo a Instrument Sans, si falla la carga la cifra sigue siendo legible.

## 4. Espaciado, forma y material

Sin cambios. El rediseño usa la escala tal cual: `space-5` (18 px) entre tarjetas, `space-6` (20 px) de
margen de pantalla, radios 18 / 13 / 9 / cápsula, desenfoques 28 / 24 / 44.

Dos medidas nuevas, ambas como token porque se repiten:

| Token | Valor (claro y oscuro) | Dónde afecta |
|---|---|---|
| `--sdm-rail-width` | `74px` | ancho del riel de `Sidebar` |
| `--sdm-hero-height` | `246px` | alto del `HeroPanel` del panel general |

## 5. Iconografía

No es un token de color: el juego de iconos hereda `currentColor` y no fija ninguno. Sí fija dos valores,
que van como token para que no se escriban a mano:

| Token | Valor | Dónde afecta |
|---|---|---|
| `--sdm-icon-stroke` | `1.7` | `stroke-width` de todos los iconos de línea |
| `--sdm-icon-size` | `24px` | `viewBox` de referencia; se renderizan a 12-28 px |
