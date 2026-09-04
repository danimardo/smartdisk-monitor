# Sistema de diseño — SmartDisk Monitor

Versión aprobada: **v2, material translúcido** (evolución de la dirección 1b). Una herramienta de
administración que se lee de un vistazo: capas de cristal sutil que dejan intuir el contenido detrás,
profundidad en lugar de líneas divisorias, color reservado para el significado y cifras grandes donde importa.

El acento lo hereda del color de acento de Windows; la app solo aporta el respaldo.

## Principios

1. **El color es información.** Verde/ámbar/rojo/gris solo significan salud. El violeta de acento significa
   acción o selección, nunca estado.
2. **La ausencia de dato es un dato.** "No disponible" es una respuesta legítima y frecuente; un cero inventado
   es un error de producto.
3. **Calma sobre densidad.** Aire, pocas líneas divisorias, jerarquía por material y tamaño antes que por bordes o grasa tipográfica (peso máximo 600).
4. **Cada número lleva su procedencia.** Fuente y antigüedad acompañan a la métrica.
5. **Nada ocurre sin avisar.** Cualquier operación que escriba o caliente el disco se explica antes.

## Anatomía

- **Tipografía**: Instrument Sans. 600 como peso máximo, 500 para etiquetas, 400 para prosa.
  Escala: 11 · 12 · 12,5 · 13,5 · 14,5 · 20 · 21 · 27 px. Cifras siempre con `.sdm-num`.
- **Espaciado**: escala de 4. `18 px` entre tarjetas, `20 px` de margen de pantalla.
- **Radios concéntricos**: 18 (ventana y tarjeta) → 13 (bloque interno) → 9 (navegación) → cápsula (controles).
  Interior = exterior − padding.
- **Material**: tres capas — `.sdm-material-chrome` (barra lateral y barra de herramientas, desenfoque 28),
  `.sdm-material` (tarjetas, 24), `.sdm-material-overlay` (diálogos, 44). Cada una con hairline y filo
  superior de 1 px. Sin soporte de `backdrop-filter`, caen a `--sdm-solid`.
- **Elevación**: `--sdm-shadow` para tarjetas, `--sdm-shadow-lift` para diálogos y ventana. Nada más.
- **Movimiento**: 220 ms con `cubic-bezier(.32,.72,0,1)`; los botones se hunden un 2 % al pulsar.
- **Capas de color**: `bg`→`bg-2` (lienzo con degradado tenue) → `glass` (material) → `glass-2` (controles)
  → `glass-3` (pistas y bloques internos).

## Archivos

| Archivo | Contenido |
|---|---|
| `tokens.css` | variables `--sdm-*`, temas claro/oscuro, base y `:focus-visible` |
| `tokens.json` | los mismos valores en formato legible por herramientas |
| `../tailwind.config.cjs` | mapeo token → utilidad |
| `../src/lib/design/` | tipos, formateo, mapa de salud, control de tema y acento del sistema |
| `../src/lib/components/` | catálogo Svelte |
| `../AGENTS.md` | reglas de uso obligatorias para agentes |

## Arranque

```ts
// src/main.ts
import "../design-system/tokens.css";
import { theme } from "$lib/design/theme.svelte";
import { invoke } from "@tauri-apps/api/core";

import { applySystemAccent } from "$lib/design/accent";

const pref = await invoke<"light" | "dark" | "system">("get_theme_preference");
theme.init(pref);
await applySystemAccent(); // hereda el color de acento de Windows; si falla, se mantiene el respaldo
```
