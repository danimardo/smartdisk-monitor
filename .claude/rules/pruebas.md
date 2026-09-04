---
paths:
  - "src/**/*.test.ts"
  - "src-tauri/**/tests/**"
  - "e2e/**"
  - "tests/**"
---

# Pruebas

Estrategia completa: `docs/testing-strategy.md`. Umbrales de cobertura: constitución §VIII.

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
