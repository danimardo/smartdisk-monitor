# Implementation Plan: Barra de título propia, integrada con el sistema de diseño

**Branch**: `011-barra-titulo-propia` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/011-barra-titulo-propia/spec.md`

## Summary

Sustituir la decoración de ventana nativa de Windows por una barra de título propia: mismo color de
fondo que el resto de la aplicación, tres controles (minimizar, maximizar/restaurar, cerrar) con el
acento de la app, arrastre por `data-tauri-drag-region`, doble clic para maximizar/restaurar. Se
compensa la altura de la barra en el presupuesto de ventana mínima ya medido (`docs/open-questions.md`
§L) en vez de remedirlo entero. Sin dependencias nuevas: todo con `@tauri-apps/api/window`, ya
presente.

**Hallazgo de la investigación que cambia el perfil de riesgo de este plan**: Tauri 2 bloquea por
defecto las acciones de ventana que esta barra necesita (cerrar, minimizar, maximizar, arrastrar,
consultar si está maximizada) hasta que se conceden explícitamente en
`src-tauri/capabilities/default.json`. Esto **son permisos de Tauri nuevos** — uno de los límites
duros de `AGENTS.md` ("Cambiar... permisos de Tauri... Un permiso nuevo exige su ADR"). No es una
elección de diseño de este plan: es un requisito mecánico del propio framework para poder construir
cualquier barra de título propia. Se documenta como ADR en la fase de implementación, con el mismo
patrón que ya usan `ADR-004`/`ADR-046` para permisos existentes.

## Technical Context

**Language/Version**: TypeScript 5.5 (Svelte 5.0, SvelteKit 2.0) + Rust 1.77.2 (edición 2021),
Tauri 2.

**Primary Dependencies**: ninguna nueva. `@tauri-apps/api` (`^2.0.0`, ya en `package.json`) para
`getCurrentWindow()`; nada nuevo en `Cargo.toml`.

**Storage**: SQLite vía `rusqlite`, sin cambios de esquema — reutiliza las claves `window.*` que ya
existen (`src-tauri/src/platform/ventana.rs`, ADR-040).

**Testing**: `pnpm check` (tipos/accesibilidad), `pnpm test` (lógica, si el wrapper `window.ts`
tiene alguna rama no trivial), `pnpm test:component` (Chromium real, el nuevo `TitleBar.svelte`
como componente presentacional con callbacks simulados — igual que `ExplicacionModal`), `pnpm
test:e2e` si aplica algún recorrido de humo, `cargo test` (Rust, las dos constantes tocadas).

**Target Platform**: Windows 10/11 de escritorio (única plataforma soportada del proyecto).

**Project Type**: aplicación de escritorio, un único proyecto (`src/`, `src-tauri/src/`).

**Performance Goals**: sin objetivo nuevo. Es chrome estático; nada que muestrear ni que optimizar
más allá de lo que ya exige el principio de rendimiento de la interfaz (`docs/ui-design.md`).

**Constraints**: **permisos de Tauri nuevos** (ver "Hallazgo" arriba) — `core:window:allow-close`,
`allow-minimize`, `allow-toggle-maximize`, `allow-start-dragging`, `allow-is-maximized` como mínimo
previsto; la lista exacta se confirma en implementación (Tauri niega en tiempo de ejecución con un
error claro si falta alguno, así que no hay riesgo de un permiso que falte en silencio). Sin
variable de entorno nueva.

**Scale/Scope**: dos ficheros de `src-tauri/` (`tauri.conf.json`, `capabilities/default.json`,
`platform/ventana.rs`), un componente nuevo del catálogo (`TitleBar.svelte`), un wrapper nuevo
(`src/lib/window.ts`), tres iconos nuevos del sprite, cambios estructurales en `+layout.svelte` y
`AppShell.svelte` (contenedor de alto y `h-screen` → `h-full`), dos diccionarios i18n.

## Constitution Check

*GATE: debe pasar antes de la fase 0. Repetido tras el diseño de la fase 1 — ver el final de esta
sección.*

| Principio | Aplica | Cómo se cumple |
|---|---|---|
| III. Pila fija | Sí | Cero dependencias nuevas. |
| IV. Dominio/presentación | Sí | `TitleBar.svelte` presentacional con callbacks (`onminimize`/`onmaximize`/`onclose`), igual que `ExplicacionModal`; las llamadas reales a `@tauri-apps/api/window` viven solo en `src/lib/window.ts` (D5 de `research.md`), no en el componente. |
| VI. Sistema de diseño | Sí | Reutiliza `--sdm-accent`, el token de altura de controles existente y el catálogo de iconos; cero valores visuales literales nuevos. |
| VII. Accesibilidad AA | Sí | Los tres controles llevan nombre accesible vía `t()` (FR-011); `pnpm check` en cero avisos como condición de cierre. |
| VIII. Testeabilidad | Sí | `TitleBar.svelte` probado como componente puro (callbacks simulados, sin `@tauri-apps/api` real); `geometria_visible` ya tiene pruebas y no cambia (D1). |
| IX. Seguridad y privacidad | Sí, con una enmienda | Los permisos nuevos son window-management puro (cerrar/minimizar/maximizar/arrastrar la propia ventana), no exponen datos ni red; aun así son superficie nueva en un binario privilegiado y **exigen su ADR** (ver "Hallazgo"). |
| X. Errores comprensibles | Sí | Las llamadas de `window.ts` son *fire-and-forget* de una acción de UI (igual que `oncancel`/`onclose` ya existentes); un fallo se registra, no rompe la pantalla. |
| XIV. Arquitectura SvelteKit/Tauri | Sí | Ninguna pantalla llama a `@tauri-apps/api` directamente; todo pasa por el wrapper nuevo, mismo principio que `frontera-ipc.md` aplica a los comandos propios. |
| **Límite duro — permisos de Tauri** | Sí, con ADR | `src-tauri/capabilities/default.json` gana como mínimo 5 permisos (`core:window:allow-close/-minimize/-toggle-maximize/-start-dragging/-is-maximized`). Requiere ADR nuevo en `docs/decisions.md` antes de cerrar la tarea (`AGENTS.md`, límites duros). Autorizado por el responsable del producto en la conversación que originó esta spec (aceptó explícitamente "construir la barra a mano... llamando a la API de ventana de Tauri"), pero se deja constancia explícita aquí por ser justo el tipo de cambio que `AGENTS.md` marca como límite duro. |

**Resultado**: PASA, con una enmienda de seguridad pendiente de redactar en implementación (el ADR
de los permisos nuevos), no una violación sin resolver.

*Repetido tras la fase 1 (`data-model.md`, `quickstart.md`): sin cambios — el diseño no introdujo
ningún comando propio, tabla ni dependencia que no estuviera ya previsto aquí. El único contrato que
cambia es el de permisos de Tauri, ya recogido arriba.*

## Project Structure

### Documentation (this feature)

```text
specs/011-barra-titulo-propia/
├── plan.md              # Este fichero
├── research.md          # Fase 0: decisiones D1-D8
├── data-model.md         # Fase 1: sin entidades; constantes de tamaño que cambian de valor
├── quickstart.md         # Fase 1: guía de validación
└── tasks.md              # Fase 2 (/speckit-tasks — no la crea este comando)
```

No hay `contracts/`: esta feature no añade ni cambia ningún comando propio (`invoke`) — el único
contrato que toca es el de permisos de Tauri, ya documentado en la tabla de arriba y en
`research.md` D5.

### Source Code (repository root)

```text
src-tauri/
├── capabilities/default.json   # + 5 permisos core:window:allow-* (con su ADR)
├── tauri.conf.json             # decorations: false; minHeight/height +altura de la barra
└── src/platform/ventana.rs     # MIN_H +altura de la barra (mismo número que arriba)

src/
├── lib/
│   ├── window.ts                     # NUEVO: único punto que importa @tauri-apps/api/window
│   ├── components/
│   │   └── TitleBar.svelte           # NUEVO: catálogo, presentacional
│   ├── design/
│   │   └── icons.ts                  # + minimize/maximize/restore
│   └── i18n/{es,en}.json             # + nombres accesibles de los 3 controles
└── routes/
    └── +layout.svelte                # monta <TitleBar>, nuevo contenedor h-screen

src/lib/components/AppShell.svelte    # h-screen → h-full
```

**Structure Decision**: proyecto único ya establecido; ninguna carpeta nueva. `TitleBar.svelte` es
el único componente nuevo del catálogo (`ui-design.md` §3: control de ventana, sin equivalente
existente que reutilizar).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| 5 permisos de Tauri nuevos (`core:window:allow-*`) | Sin ellos, Tauri 2 bloquea en tiempo de ejecución cualquier llamada de cerrar/minimizar/maximizar/arrastrar/consultar-estado desde el frontend — es la forma en que el propio framework impide que web content controle la ventana sin permiso explícito. | No hay alternativa: es el único mecanismo que Tauri 2 ofrece para esta funcionalidad. La alternativa real (no construir la barra propia) es rechazar la feature entera, ya decidida y confirmada por el responsable del producto. |
