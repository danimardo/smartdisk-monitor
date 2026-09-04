---
name: adr
description: Cuándo una decisión merece un ADR, la plantilla que usa este proyecto, la numeración y cómo marcar una decisión superada sin borrarla. Úsala al elegir entre alternativas con consecuencias duraderas, al añadir una dependencia, al cambiar una versión de la pila, o cuando el usuario pregunte "por qué se hizo así" y no encuentres la razón escrita.
---

# Registro de decisiones

Los ADR de este proyecto viven en **`docs/decisions.md`**, un fichero único con numeración
correlativa. Hay 24, de ADR-001 a ADR-024.

> Nota: la constitución menciona `docs/decisions/` como carpeta. Aquí es un fichero único. Si
> algún día se parte en carpeta, hay que actualizar `tools/build-historias.py` y las referencias.

## Cuándo procede

Sí, cuando la decisión:

- descarta alternativas razonables y alguien preguntará por qué;
- añade una dependencia o cambia una versión de la pila;
- fija un compromiso que costará revertir;
- resuelve una tensión entre dos principios de la constitución.

No, cuando:

- es un detalle de implementación reversible en una tarde;
- ya está cubierta por un principio de la constitución (ahí se cita, no se duplica);
- es un valor adoptado provisionalmente: eso va a `docs/open-questions.md` como `PROPUESTO`.

**Un ADR no es una nota de trabajo.** Si no descarta nada, probablemente no sea un ADR.

## Plantilla

```markdown
## ADR-0NN — Título en una línea, en imperativo o afirmativo

Estado: aceptada.

### El problema

Qué situación obliga a decidir. Sin esto, el ADR no se entiende dentro de un año.

### La decisión

Qué se hace. Concreto y verificable.

### Alternativas descartadas

- **Nombre de la alternativa.** Por qué se descartó. Su ventaja real, y qué la supera.

### Consecuencias

Qué se acepta a cambio. Un ADR sin coste asumido suele estar incompleto.
```

## Numeración

El siguiente número sale de:

```sh
grep -c "^## ADR-" docs/decisions.md
```

Los ADR **se añaden al final**, nunca se intercalan ni se renumeran: las referencias cruzadas del
proyecto los citan por número.

## Superar una decisión

Nunca se borra ni se reescribe. Se cambia el estado y se enlaza:

```markdown
## ADR-007 — Sin red ni telemetría

Estado: **reemplazada por ADR-031** (2026-11-14).

<texto original intacto>
```

Y el ADR nuevo explica en su apartado del problema qué cambió respecto al anterior.

## Al terminar

1. `pnpm docs:build` para regenerar el consolidado.
2. Si la decisión cambia una regla de trabajo, comprueba si toca actualizar `AGENTS.md` o una
   regla de `.claude/rules/`.
3. Si crees que debería ser un **principio permanente**, no toques la constitución: propón el
   cambio y espera autorización.
