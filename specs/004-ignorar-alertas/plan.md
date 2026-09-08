# Implementation Plan: Ignorar una alerta de forma permanente

**Branch**: `004-ignorar-alertas` | **Date**: 2026-09-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/004-ignorar-alertas/spec.md`

## Summary

Se añade un estado terminal `ignored` al ciclo de vida de un grupo de alerta. Un grupo `ignored`
sigue acumulando ocurrencias en su cronología y sube su severidad registrada al peor valor visto,
pero nunca notifica, nunca cuenta para el color del disco y nunca se reactiva solo. Se sale de él
únicamente con la acción «Dejar de ignorar», que lo devuelve a `resolved` y deja que el motor lo
promocione a `active` en el siguiente ciclo si la condición sigue cumpliéndose. Siete reglas
SMART/NVMe de fallo físico o predicción de fallo quedan vetadas: para ellas la acción se presenta
deshabilitada con su motivo, y el backend rechaza cualquier intento con un `AppError`.

El enfoque técnico reutiliza la maquinaria existente: `AlertStatus` gana una variante,
`alerts::agrupacion::procesar` gana dos ramas de `match` para el caso `ignored`, `alerts::ciclo`
gana `ignorar` / `dejar_de_ignorar` como el resto de acciones de usuario, y la pantalla de alertas
gana una pestaña y dos botones. La migración `0003` reconstruye `alert_groups` para ampliar el
`CHECK` de `status` y añadir `ignored_at_utc`, con el truco de `PRAGMA legacy_alter_table` para no
disparar el `ON DELETE CASCADE` de `alert_occurrences`.

## Technical Context

**Language/Version**: Rust 1.77.2 (edición 2021) · TypeScript 5 · Svelte 5 (runes)

**Primary Dependencies**: Tauri 2 · rusqlite 0.40.2 (`bundled`) · ts-rs 10.1 · Zod · sin
dependencias nuevas

**Storage**: SQLite en `%ProgramData%\SmartDisk Monitor\` (WAL, `foreign_keys=ON`). Migración
numerada `0003`.

**Testing**: `cargo test` (dominio + repos + comandos + migraciones) · `pnpm test` (lógica Node,
`health.ts`) · `pnpm test:component` (Chromium, botones y estados) · `pnpm test:e2e`
(`e2e/ui/alerts.spec.ts`)

**Target Platform**: Windows x64, aplicación de escritorio elevada (`requireAdministrator`)

**Project Type**: desktop-app (frontend SvelteKit estático + backend Rust/Tauri)

**Performance Goals**: sin impacto medible — un estado más en un `WHERE status IN (...)` y un filtro
más en la pantalla. La pantalla de alertas no se virtualiza (SC-006 aplica a eventos, no a
alertas).

**Constraints**: cero valores visuales literales · cero literales de interfaz (claves en `es` y
`en`) · toda acción que escribe pasa por la capa de comandos · toda acción que escribe datos se
confirma con impacto explícito · correcto en tema claro y oscuro y en 1024 × 560

**Scale/Scope**: decenas de grupos ignorados como mucho en una instalación real; una pantalla
tocada (`/alerts`), un componente del catálogo sin tocar (`SegmentedControl` admite el 5.º
segmento), ~10 claves i18n nuevas.

## Constitution Check

*GATE: pasa antes de Phase 0. Re-evaluado tras Phase 1.*

| Principio | Efecto de esta feature | Veredicto |
|---|---|---|
| I. Veracidad del dato | `ignored` nunca inventa un valor; las ocurrencias se siguen registrando con su valor real; la severidad registrada refleja el peor valor visto. El veto de 7 reglas impide ocultar «el disco se está muriendo». | ✅ refuerza |
| II. Orden de prioridades | Ninguna contradicción con documento normativo; `docs/alert-rules.md` se actualiza (FR-015). | ✅ |
| III. Pila fija | Sin dependencias nuevas, sin red, sin componentes de terceros. | ✅ |
| IV. Dominio/presentación | `regla_es_ignorable(rule_key) -> bool` es decisión pura; los comandos `ignore_alert` / `unignore_alert` son finos y delegan en `alerts::ciclo`; ningún componente decide la regla de negocio (llega `ruleIgnorable` en el contrato). | ✅ |
| V. Persistencia íntegra y trazable | Migración numerada compilada; copia consistente previa la hace el runner; la retención nunca borra `alert_groups` ni `alert_occurrences` (sin cambios). Se añade `ignored_at_utc` (UTC). El `CHECK` de `status` se mantiene, ampliado. | ✅ |
| VI. Sistema de diseño | Pestaña y botones con utilidades/tokens; diálogo de confirmación con impacto; claves en los dos diccionarios; sin condicionales de tema. | ✅ (verificar en Phase 1 de implementación) |
| VII. Accesibilidad AA | Botones nuevos con foco visible; acción vetada como `Button` deshabilitado con `disabledReason`; `SegmentedControl` ya es `radiogroup`. | ✅ |
| VIII. Testeabilidad | Decisión pura con fixtures; ramas de `procesar` con pruebas unitarias; prueba de migración que verifica que las ocurrencias sobreviven. | ✅ |
| IX. Seguridad y privacidad | Sin telemetría, sin permisos nuevos, sin procesos externos. El log de `ignore` / `unignore` no incluye datos sensibles. | ✅ |
| X. Errores comprensibles | `alert.rule_not_ignorable` → `AppError` con clave i18n y frase humana. | ✅ |
| XIV. SvelteKit idiomático | `load` sigue trayendo el historial completo sin filtro; el filtro por pestaña es de pantalla; las mutaciones van por comando y refrescan por `alerts:changed`. | ✅ |

**Límites duros de `AGENTS.md`**: cambia el contrato de comandos (permitido, es una feature con
spec; se documenta en `docs/ui-contract.md`). No toca permisos de Tauri, ni `constitution.md`, ni
`third-party/`, ni añade dependencias. **ADR requerido (ADR-044, tarea T001)**: registrar el
conjunto de reglas no ignorables como decisión de producto duradera, con su motivo.

**Resultado del gate**: PASA. Sin violaciones que justificar; `Complexity Tracking` se omite.

## Project Structure

### Documentation (this feature)

```text
specs/004-ignorar-alertas/
├── plan.md              # Este fichero
├── spec.md              # Especificación (con Clarifications)
├── research.md          # Phase 0 — decisiones técnicas
├── data-model.md        # Phase 1 — alert_groups + estado ignored
├── quickstart.md        # Phase 1 — guía de validación end-to-end
├── contracts/
│   └── comandos-alertas.md   # Phase 1 — ignore_alert / unignore_alert + ruleIgnorable
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 — lo genera /speckit-tasks
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   └── 0003_alerta_ignored.sql        # NUEVO — reconstruye alert_groups (CHECK + ignored_at_utc)
├── src/
│   ├── persistence/
│   │   ├── migrations.rs              # + entrada Migration { version: 3, ... }
│   │   └── repo_alertas.rs            # status_to_str/from_str, set_status (Ignored), dejar_de_ignorar
│   ├── domain/
│   │   └── tipos.rs                   # AlertStatus += Ignored; AlertGroup += ignored_at_utc
│   ├── alerts/
│   │   ├── reglas.rs                  # NUEVO (o en ciclo.rs) — REGLAS_NO_IGNORABLES + regla_es_ignorable
│   │   ├── agrupacion.rs              # ramas ignored en procesar(); Transicion::OcurrenciaIgnorada
│   │   ├── ciclo.rs                   # ignorar() / dejar_de_ignorar()
│   │   └── notificaciones.rs          # debe_enviar: OcurrenciaIgnorada => false
│   ├── commands/
│   │   └── mod.rs                     # ignore_alert, unignore_alert; AlertDetail += ruleIgnorable
│   └── lib.rs                         # registra los dos comandos nuevos
│
src/
├── lib/
│   ├── api/
│   │   ├── generated/AlertStatus.ts   # regenerado por `cargo test` (ts-rs)
│   │   ├── schemas.ts                 # alertStatus += "ignored"; alertDetail += ruleIgnorable
│   │   └── client.ts                  # ignoreAlert, unignoreAlert
│   ├── design/
│   │   └── health.ts                  # comentario de COUNTS_TOWARD_HEALTH (ignored no cuenta)
│   ├── components/
│   │   └── AlertCard.svelte           # (solo si hace falta tratar la píldora de estado ignorado)
│   └── i18n/
│       ├── es.json                    # ~10 claves nuevas
│       └── en.json                    # espejo
└── routes/alerts/
    └── +page.svelte                   # pestaña "Ignoradas", botones Ignorar / Dejar de ignorar, diálogo

docs/
├── alert-rules.md      # §1 tabla de estados + "qué cuenta para el color" + conjunto no ignorable (FR-015)
├── ui-contract.md      # §3.4 comandos ignore_alert / unignore_alert + ruleIgnorable
├── data-model.md       # §2 alert_groups: lista de estados + ignored_at_utc
├── ui-design.md        # §7.3 pestaña Ignoradas + acción
└── decisions.md        # ADR-044 — conjunto de reglas no ignorables

e2e/ui/alerts.spec.ts   # flujo ignorar, pestaña Ignoradas, acción vetada deshabilitada
```

**Structure Decision**: no se crea ninguna estructura nueva. La feature encaja en los módulos
existentes de alertas (`src-tauri/src/alerts/`), su repositorio (`persistence/repo_alertas.rs`) y
la única pantalla afectada (`src/routes/alerts/+page.svelte`). El único fichero realmente nuevo es
la migración `0003` y, opcionalmente, `src-tauri/src/alerts/reglas.rs` para alojar la lista de
reglas no ignorables junto a su función de decisión (alternativa: constante privada en `ciclo.rs`,
junto a `politica_notificacion` que ya vive en `alerts/`).

## Complexity Tracking

No aplica: el Constitution Check pasa sin violaciones.
