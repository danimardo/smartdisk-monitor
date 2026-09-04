---
paths:
  - "src/**/*.test.ts"
  - "src-tauri/**/tests/**"
  - "e2e/**"
  - "tests/**"
---

# Pruebas

Estrategia completa: `docs/testing-strategy.md`. Umbrales de cobertura: constitución §VIII.

## Dónde va cada prueba

| Sufijo o carpeta | Corre en | Comando |
|---|---|---|
| `src/**/*.test.ts` | jsdom, Node | `pnpm test` |
| `src/**/*.svelte.test.ts` | jsdom, Node — módulos `.svelte.ts` con runas, **no** componentes | `pnpm test` |
| `src/**/*.browser.test.ts` | Chromium real | `pnpm test:component` |
| `e2e/ui/**` | Chromium con IPC falso | `pnpm test:e2e` |
| `src-tauri/src/**` con `#[cfg(test)]` | Rust | `cargo test` |

Al navegador van **solo** las comprobaciones que jsdom no puede hacer: contraste sobre el material
compuesto, respaldo a `--sdm-solid`, resolución de variables en tema oscuro, `prefers-reduced-motion`,
visibilidad del foco y recorte a 1024 × 560. Lo demás sale más barato en Node.

`render()` de `vitest-browser-svelte` **es asíncrona**: sin `await` la prueba pasa sin haber
renderizado nada. Lo detecta ESLint, pero conviene saberlo.

Los fixtures de `e2e/ui/fixtures/` se validan contra los esquemas Zod reales. No es ceremonia: en la
primera ejecución rechazaron tres valores inventados que no existían en el contrato.

- **Elige el nivel más barato** que demuestre el requisito. No se reproduce por interfaz la lógica
  que ya cubre un unit test.
- **La prueba va antes que el código** —sin excepción— en: parsers de `smartctl`, motor de alertas
  (activación, histéresis, deduplicación, ciclo), retención, rutas del benchmark, migraciones, y
  cualquier defecto reproducido. Son las áreas donde el fallo es silencioso: no revienta, produce
  un dato equivocado que alguien se cree.
- **Toda regla con umbral** se prueba en el umbral, justo por encima y justo por debajo.
- **Todo formateador** se prueba con dato ausente (`null`, `undefined`, `NaN`) **y** con cero real,
  para demostrar que no se confunden.
- **Localiza por rol o nombre accesible**, no por clase CSS ni por posición. Si un elemento no se
  puede localizar así, es que no es accesible, y eso incumple el principio VII antes que ninguna
  prueba.
- **Prohibido**: `waitForTimeout`, esperas arbitrarias, hora real, aleatoriedad sin semilla,
  dependencia del orden de ejecución, acceso a `%ProgramData%`.
- **Los fixtures se anonimizan al capturarlos**, nunca al usarlos.
- Una prueba inestable es un **defecto**, no una molestia: no se arregla subiendo tiempos ni
  reintentos.

Una prueba debe fallar por una razón comprensible. Sin aserción, o comprobando solo que no lanza
excepción, no vale.
