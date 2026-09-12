# Fase 1 — Modelo de datos: barra de título propia

Esta feature no introduce ninguna entidad de dominio ni tabla nueva. Es la única spec de las
recientes sin un modelo de datos propio: es chrome de ventana, no una funcionalidad con estado que
persistir más allá de lo que ya existe.

## Sin cambios: geometría de ventana persistida

Las claves de `settings` que ya guardan la geometría de la ventana (`window.width`, `window.height`,
`window.x`, `window.y`, `window.maximized`; `src-tauri/src/platform/ventana.rs`, ADR-040) no
cambian de forma ni de significado. Siguen midiendo el tamaño y la posición de la ventana en su
conjunto (`inner_size`/`outer_position` de Tauri) — con decoración nativa o sin ella, son las
mismas magnitudes: el alto guardado ya incluirá la barra propia igual que hoy incluye el contenido
bajo la barra nativa, sin necesidad de un campo nuevo que distinga una cosa de otra.

## Constantes que sí cambian de valor (no de forma)

| Constante | Fichero | Antes | Después |
|---|---|---|---|
| `MIN_H` | `src-tauri/src/platform/ventana.rs` | 560 | 560 + altura de la barra propia |
| `minHeight` | `src-tauri/tauri.conf.json` | 560 | 560 + altura de la barra propia |
| `height` (tamaño por defecto) | `src-tauri/tauri.conf.json` | 988 | 988 + altura de la barra propia |

`MIN_W`/`minWidth` (1024) no cambian: la barra no añade anchura, solo altura. Ver `research.md` D3
para la justificación de por qué se suma en vez de remedir todo el presupuesto de §L.

## Sin entidad: controles de la barra

Los tres controles (minimizar, maximizar/restaurar, cerrar) no tienen estado propio que persista:
`maximizar/restaurar` se limita a reflejar `ventana.is_maximized()` en el momento, ya expuesto por
`@tauri-apps/api/window` sin necesidad de guardarlo aparte del `window.maximized` que ya existe.
