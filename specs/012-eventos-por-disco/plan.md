# Implementation Plan: Eventos de Windows en el detalle de disco

**Branch**: `012-eventos-por-disco` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/012-eventos-por-disco/spec.md`

## Summary

Añadir, debajo del bloque de gráficas y contadores de `/disks/[id]`, una `Card` con los 5 eventos
de Windows más recientes de ese disco (`getSystemEvents({ deviceId, limit: 5 })`), cada uno como
`EventRow` con `href="/events?focus={id}"`, y un enlace «Ver todos» a `/events?deviceId={id}`. Es
el mismo patrón que ya usa el widget «Últimos eventos» del panel general (`src/routes/+page.svelte`),
aplicado a un solo disco. Estado vacío y de error diseñados, cada uno aislado del resto de la
pantalla (mismo patrón que ya usan hoy las dos gráficas históricas de esta pantalla). Sin cambios de
backend, de comando ni de esquema: `get_system_events` ya acepta `deviceId`, y la ruta `/events` ya
sabe leer `?deviceId=` y `?focus=` — hoy nada enlaza a ese primero desde el detalle de un disco.

## Technical Context

**Language/Version**: TypeScript 5.9.3 (Svelte 5.57.0 con runas, SvelteKit 2.70.3, `adapter-static`,
SSR off). Sin cambios en Rust/Tauri.

**Primary Dependencies**: ninguna nueva. Reutiliza `getSystemEvents` (`src/lib/api/client.ts`, ya
valida con `S.systemEventPage`) y los componentes ya existentes `Card` y `EventRow`
(`src/lib/components`).

**Storage**: N/A — no hay esquema nuevo ni tabla nueva; se lee un dato que el backend ya expone.

**Testing**: `pnpm check` (tipos y accesibilidad); `pnpm test:e2e` ampliando
`e2e/ui/disk-detail.spec.ts` con el patrón ya usado por `e2e/ui/events.spec.ts` (IPC falso,
localización por rol/nombre accesible). No hace falta `pnpm test:component`: no se crea ningún
componente nuevo del catálogo, solo se componen dos ya probados (`Card`, `EventRow` ya tienen sus
pruebas propias). No hay lógica de dominio nueva que requiera `cargo test` ni `pnpm test`.

**Target Platform**: Windows de escritorio (sin cambio, única plataforma soportada).

**Project Type**: aplicación de escritorio, proyecto único ya establecido.

**Performance Goals**: ninguno nuevo. Reutiliza un comando ya paginado y ya usado en otras dos
pantallas (panel general, Eventos); `limit: 5` es la petición más pequeña de las tres.

**Constraints**: ninguna. No hay permiso de Tauri nuevo, no hay dependencia nueva, no hay variable
de entorno.

**Scale/Scope**: un fichero de pantalla (`src/routes/disks/[id]/+page.svelte`), un puñado de claves
nuevas en los dos diccionarios i18n, una pantalla de prueba de extremo a extremo ampliada, y un
párrafo nuevo en `docs/ui-contract.md` documentando el parámetro `?deviceId=` de `/events` (ya
implementado en el `load` de esa ruta desde `open-questions.md` J.10, pero nunca documentado ni
usado desde ningún sitio real hasta ahora).

## Constitution Check

*GATE: debe pasar antes de la fase 0. Repetido tras el diseño de la fase 1 — ver el final de esta
sección.*

| Principio | Aplica | Cómo se cumple |
|---|---|---|
| I. Veracidad del dato | Sí | Un evento con `mappingConfidence` no exacta se marca con la misma etiqueta que ya pinta `EventRow` (FR-004); sin eventos, la sección dice explícitamente que no hay, nunca se oculta ni se rellena con un cero o un guion (FR-007). |
| III. Pila fija | Sí | Cero dependencias nuevas, cero permisos de Tauri nuevos. |
| IV. Dominio/presentación | Sí | La pantalla sigue llamando solo a `$lib/api` (`getSystemEvents`, ya existente); no decide ninguna regla de negocio nueva — la asociación evento→disco y su confianza las decide el backend, la pantalla solo las muestra. |
| VI. Sistema de diseño | Sí | Reutiliza `Card` y `EventRow` del catálogo, cero valores visuales literales nuevos, textos nuevos por `t()` en ambos diccionarios. |
| VII. Accesibilidad AA | Sí | El enlace «Ver todos» es un `<a href>` real (no `goto()` en `onclick`); cada evento sigue siendo accesible por rol/nombre igual que en el panel general; estado vacío y de error con su texto, no solo color. |
| VIII. Testeabilidad | Sí | Cobertura de estados (recientes, vacío, error) vía Playwright con IPC falso, mismo patrón que `events.spec.ts` y que las dos gráficas ya probadas de esta misma pantalla; `src/routes/` no tiene umbral numérico de cobertura (constitución §VIII). |
| X. Errores comprensibles | Sí | Un fallo de `getSystemEvents` para esta sección se muestra con la forma `AppError` y degrada solo la sección, igual que ya hacen hoy `serieTemp`/`serieActividad` en la misma pantalla; el resto de la pantalla sigue funcionando. |
| XI. Validación de fronteras | Sí | Reutiliza el esquema Zod ya existente (`S.systemEventPage`) sin tocarlo; no cruza ningún dato nuevo sin validar. |
| XIII. Validación de tipos primero | Sí | `pnpm check` en cero errores y cero avisos, como en cualquier cambio de interfaz. |
| XIV. Arquitectura SvelteKit/Tauri | Sí | Navegación por `<a href>` (el enlace «Ver todos» y cada `EventRow`), nunca `goto()` en un `onclick`; la petición de eventos vive en un `$effect` que depende de `disk.id`, mismo patrón ya usado en esta pantalla para las dos series históricas, con su función de limpieza. |

**Resultado**: PASA, sin excepciones y sin entrada de `Complexity Tracking`: no hay ninguna
violación que justificar.

*Repetido tras la fase 1 (`data-model.md`, `quickstart.md`): sin cambios — el diseño no introduce
ninguna entidad, comando ni dependencia que no estuviera ya prevista aquí.*

## Project Structure

### Documentation (this feature)

```text
specs/012-eventos-por-disco/
├── plan.md              # Este fichero
├── research.md          # Fase 0: decisiones D1-D6
├── data-model.md        # Fase 1: sin entidades nuevas, solo estado de pantalla
├── quickstart.md         # Fase 1: guía de validación manual
└── tasks.md              # Fase 2 (/speckit-tasks — no la crea este comando)
```

No hay `contracts/`: esta funcionalidad no añade ni cambia ningún comando Tauri (`invoke`). El único
contrato que se ve afectado es el parámetro de URL `?deviceId=` de la pantalla de Eventos, que ya
existe en el código (`src/routes/events/+page.ts`) pero nunca se documentó en `docs/ui-contract.md`
ni se usó desde ninguna pantalla real; se documenta ahí directamente, junto al `?focus=` ya descrito
en la §3.5 de ese fichero, en vez de en un `contracts/` de esta carpeta.

### Source Code (repository root)

```text
src/
├── routes/
│   └── disks/[id]/
│       └── +page.svelte        # + sección "Eventos de este disco" (Card, EventRow, EmptyState)
└── lib/
    └── i18n/
        ├── es.json              # + claves de la sección nueva
        └── en.json              # + mismas claves, en inglés

e2e/ui/
└── disk-detail.spec.ts         # + casos: eventos recientes, enlace "ver todos", vacío, error

docs/
└── ui-contract.md              # + párrafo documentando `?deviceId=` en /events (§3.5)
```

**Structure Decision**: proyecto único ya establecido; ningún fichero ni carpeta nueva en
`src/lib/components/` — la sección se compone enteramente con `Card` y `EventRow`, que ya cubren lo
que pide `ui-design.md` §3 para no justificar un componente nuevo.

## Complexity Tracking

*Sin entradas: el Constitution Check no encontró ninguna violación que justificar.*
