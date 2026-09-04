---
paths:
  - "docs/**/*.md"
  - "README.md"
  - "THIRD_PARTY_NOTICES.md"
  - "design/**/*.md"
---

# Documentación

## `historias.md` se genera, no se edita

Es el consolidado de 31 ficheros. Si lo editas directamente, **tu cambio se pierde** en la
siguiente regeneración. El flujo correcto:

1. Editar el fichero de `docs/` que corresponda.
2. Ejecutar `pnpm docs:build`.
3. Confirmar los dos cambios juntos.

`pnpm docs:check` falla si el consolidado va por detrás, y CI lo comprueba.
Para añadir un documento nuevo al consolidado hay que registrarlo en `tools/build-historias.py`.

## Al escribir

- **En español**, con la ortografía completa: tildes, eñes y signos de apertura.
- **Di por qué, no solo qué.** Una decisión sin su razón se revierte en cuanto alguien la encuentre
  incómoda.
- **Los números vienen de medir**, no de estimar. Si es una estimación, se dice que lo es.
- **Una decisión provisional se marca como tal.** `docs/open-questions.md` distingue `DECIDIDO`,
  `PROPUESTO` y `ABIERTO`, y esa distinción importa: lo `PROPUESTO` es barato de cambiar ahora y
  caro después.
- **No se borra historia.** Una decisión superada se marca como reemplazada, con su fecha, y se
  conserva.

## Qué documento toca

Matriz completa en la skill `cierre-tarea`. En resumen: reglas de alerta a `docs/alert-rules.md`,
comandos y eventos a `docs/ui-contract.md`, decisiones a `docs/decisions.md`, mediciones y valores
adoptados a `docs/open-questions.md`, criterios visuales a `docs/ui-design.md`.
