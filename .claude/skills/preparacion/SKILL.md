---
name: preparacion
description: Checklist de lectura de las fuentes de verdad antes de tocar código, y criterio para decidir si el cambio necesita el flujo SpecKit completo. Úsala al empezar cualquier tarea de implementación, al retomar trabajo tras una pausa, o cuando el usuario pida "implementa", "arregla", "añade" o "cambia" algo sin haber indicado antes qué documentos aplican.
---

# Preparación antes de implementar

El objetivo es no descubrir a mitad de la implementación que existía una decisión escrita que la
contradice. Cuesta cinco minutos y ahorra rehacer el trabajo.

## 1. Clasifica el cambio

**Pequeño** — corrección localizada, texto, ajuste visual, refactorización sin efecto observable.
No necesita spec.

**Relevante** — funcionalidad nueva, cambio de comportamiento, reglas de negocio, permisos,
contratos, modelo de datos, arquitectura o integraciones. Necesita el flujo completo.

Si al abordar uno pequeño aparece impacto en comportamiento observable, contratos, modelo de datos,
seguridad, permisos, registro o arquitectura, **deja de ser pequeño**: para y reclasifica.

## 2. Lee, en este orden

Solo lo que la tarea toque. Leerlo todo es tan malo como no leer nada.

| Siempre | `.specify/memory/constitution.md` — los 15 principios |
| Si toca interfaz | `docs/ui-design.md` — su §0 dice dónde está cada pieza |
| Si toca alertas | `docs/alert-rules.md` |
| Si toca comandos o eventos | `docs/ui-contract.md` |
| Si toca datos | `docs/data-model.md` |
| Si toca pruebas | `docs/testing-strategy.md` |
| Siempre | `docs/open-questions.md` — busca el área de la tarea |

`docs/open-questions.md` es el que más se olvida y el que más ahorra: recoge decisiones adoptadas y
mediciones que corrigen la especificación original. Cuatro suposiciones resultaron falsas al
medirlas.

## 3. Comprueba si ya está decidido

Antes de elegir cómo hacer algo, busca si ya hay un ADR: `grep -n "ADR-" docs/decisions.md`. Hay 24.

## 4. Si algo no está claro

**Pregunta antes de implementar.** Si aparece una decisión que no está escrita en ninguna parte:

1. Añádela a `docs/open-questions.md` con su valor propuesto y su razón.
2. Márcala como `PROPUESTO`.
3. Solo entonces impleméntala.

Resolverla en silencio dentro del código es la infracción más grave del proceso: nadie sabrá
después que aquello fue una decisión.

## 5. Para un cambio relevante

Usa las skills de SpecKit en su orden, no reconstruyas el procedimiento:

`speckit-specify` → `speckit-clarify` → `speckit-plan` → `speckit-tasks` → `speckit-analyze` →
`speckit-implement`

`speckit-checklist` genera comprobaciones a medida y `speckit-converge` compara el código contra la
spec cuando ya hay implementación parcial.
