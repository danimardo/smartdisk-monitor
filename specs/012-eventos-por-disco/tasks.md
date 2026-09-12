---
description: "Task list template for feature implementation"
---

# Tasks: Eventos de Windows en el detalle de disco

**Input**: Design documents from `/specs/012-eventos-por-disco/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [quickstart.md](./quickstart.md)

**Tests**: la constitución (principio VIII) no exige TDD para pantallas — las pruebas acompañan al
código, no tienen que precederlo. Se incluyen igualmente como tareas propias, una por caso de
`quickstart.md`, para que cada historia quede verificada antes de pasar a la siguiente.

**Organización**: una fase por historia de usuario (`spec.md`), en orden de prioridad. Cada fase es
un incremento entregable y comprobable por separado.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: se puede hacer en paralelo (ficheros distintos, sin dependencia de una tarea sin acabar)
- **[Story]**: US1, US2 o US3, según `spec.md`
- Cada descripción lleva la ruta exacta del fichero que toca

## Path Conventions

Proyecto único ya establecido (`src/`, `e2e/`, `docs/`) — sin carpetas nuevas.

---

## Phase 1: Setup (infraestructura compartida)

**Propósito**: los textos que van a necesitar las tres historias.

- [X] T001 [P] Añadir las claves i18n de la sección nueva (título de la `Card` y mensaje de estado
      vacío, p. ej. `disk.events.title` / `disk.events.empty`) a `src/lib/i18n/es.json`
- [X] T002 [P] Añadir las mismas claves, en inglés, a `src/lib/i18n/en.json` — exactamente las
      mismas claves que T001, ningún nombre distinto

**Checkpoint**: los diccionarios tienen las claves nuevas y `pnpm check`/`verify:i18n` no se quejan
de una clave que falte en un idioma.

---

## Phase 2: Foundational (prerrequisitos bloqueantes)

*No aplica.* No hay ninguna infraestructura compartida entre las tres historias más allá de las
claves i18n ya cubiertas en el Setup: no hay entidad nueva (`data-model.md`), ni comando nuevo, ni
dependencia nueva (`plan.md`, Constitution Check). Cada historia añade directamente sobre el mismo
bloque de pantalla que la anterior.

---

## Phase 3: User Story 1 - Ver de un vistazo los últimos eventos de este disco (Priority: P1) 🎯 MVP

**Goal**: la persona ve, sin salir del detalle del disco, los eventos de Windows más recientes
asociados a ese disco.

**Independent Test**: abrir el detalle de un disco con eventos asociados y comprobar que aparecen,
del más reciente al más antiguo, con nivel, mensaje, proveedor, ID y hora.

### Implementation for User Story 1

- [X] T003 [US1] En `src/routes/disks/[id]/+page.svelte`, añadir el estado local de la sección
      (`eventos: SystemEvent[]`, `error: AppError | null`, `loading: boolean`) y un `$effect` que
      dependa de `disk.id`, llame a `getSystemEvents({ deviceId: disk.id, limit: 5 })` y guarde el
      resultado — mismo patrón y misma función de limpieza que ya usan `serieTemp`/`serieActividad`
      en ese fichero (líneas ~163-202)
- [X] T004 [US1] En el mismo fichero, añadir la sección "Eventos de este disco": una `Card` situada
      debajo del `grid` de gráficas + contadores, con un `{#each eventos as evento (evento.id)}` de
      componentes `EventRow` (level, message, provider, eventId, occurredAt, mappingConfidence; sin
      `href` todavía, se añade en la User Story 2)
- [X] T005 [US1] En la misma `Card`, cuando `eventos.length === 0` y no hay `error`, mostrar
      `EmptyState kind="empty"` con las claves de T001/T002
- [X] T006 [US1] En la misma `Card`, cuando `error` no es `null`, mostrar `EmptyState kind="error"`
      con `t(error.messageKey, error.messageVars)` y `detail={error.detail ?? ""}` — mismo patrón
      que el snippet `cargaOFallo` ya existente en ese fichero — sin que afecte a la cabecera, las
      métricas, las gráficas ni los contadores

### Tests for User Story 1

- [X] T007 [P] [US1] En `e2e/ui/disk-detail.spec.ts`, prueba: con los fixtures existentes
      (`detalleDisco0`, `eventos`/`paginaEventos` de `e2e/ui/fixtures/respuestas.ts`), el detalle de
      disco muestra los eventos asociados con su mensaje, y el que tenga `mappingConfidence`
      distinta de `exact` lleva la etiqueta `es["events.inferredMapping"]`
- [X] T008 [P] [US1] En el mismo fichero, prueba: con `get_system_events` devolviendo una página sin
      eventos, la sección muestra su estado vacío (no desaparece)
- [X] T009 [P] [US1] En el mismo fichero, prueba: con `get_system_events` rechazando
      (`{ __rechazar__: { code, messageKey, ... } }`), la sección muestra su estado de error y la
      cabecera, las métricas y las gráficas de la pantalla se siguen viendo

**Checkpoint**: la User Story 1 es funcional y comprobable por sí sola — la sección lista eventos,
o dice claramente que no hay, o dice claramente que algo falló.

---

## Phase 4: User Story 2 - Profundizar en un evento concreto (Priority: P2)

**Goal**: desde el resumen, la persona llega al detalle completo de un evento (mensaje íntegro, XML,
«Explícamelo con IA») en la pantalla de Eventos ya existente.

**Independent Test**: pulsar un evento de la sección y comprobar que `/events` se abre con ese
suceso ya resaltado y su panel de detalle abierto.

### Implementation for User Story 2

- [X] T010 [US2] En `src/routes/disks/[id]/+page.svelte`, añadir `href="/events?focus={evento.id}"`
      a cada `EventRow` de la sección (deja de pasar `onselect`; con `href`, `EventRow` se renderiza
      como `<a>`, no como `<button>` — ver `EventRow.svelte`)

### Tests for User Story 2

- [X] T011 [P] [US2] En `e2e/ui/disk-detail.spec.ts`, prueba: pulsar sobre uno de los eventos de la
      sección navega a `/events`, con ese suceso resaltado (`data.focusId`) y su panel de detalle
      (`es["events.detail.title"]`) visible — mismo patrón que ya usa `events.spec.ts` para
      "seleccionar un evento carga su detalle"

**Checkpoint**: las User Stories 1 y 2 funcionan juntas — ver el resumen y profundizar en un evento.

---

## Phase 5: User Story 3 - Ver todos los eventos de este disco (Priority: P2)

**Goal**: desde el resumen, la persona pasa al historial completo de este disco sin reaplicar el
filtro a mano.

**Independent Test**: pulsar "Ver todos" en la sección y comprobar que `/events` se abre con el
filtro de dispositivo ya aplicado a ese disco.

### Implementation for User Story 3

- [X] T012 [US3] En `src/routes/disks/[id]/+page.svelte`, añadir el snippet `action` de la `Card` de
      la sección con un enlace `<a href="/events?deviceId={disk.id}">{t("common.viewAll")}</a>` —
      mismo patrón que ya usa el widget "Últimos eventos" del panel general
      (`src/routes/+page.svelte`, líneas ~245-248)

### Tests for User Story 3

- [X] T013 [P] [US3] En `e2e/ui/disk-detail.spec.ts`, prueba: pulsar "Ver todos" navega a `/events`
      con el filtro de dispositivo preaplicado a ese disco (comprobar `data.deviceId` vía la URL o
      el estado inicial de la `FilterBar`/lista resultante)

**Checkpoint**: las tres historias funcionan juntas — resumen, profundizar y ver todo.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: cerrar la tarea según `AGENTS.md` (skill `cierre-tarea`) y la definición de terminado
de `ui-design.md` §8.

- [X] T014 [P] Documentar el parámetro `?deviceId=` de `/events` en `docs/ui-contract.md` §3.5,
      junto al párrafo ya existente sobre `?focus=` (decisión D6 de `research.md`)
- [X] T015 Ejecutar `pnpm check` (cero errores y cero avisos) y `pnpm test:e2e -- disk-detail` en
      verde; corregir cualquier fallo antes de continuar
- [X] T016 Repasar la pantalla modificada contra la definición de terminado de `ui-design.md` §8:
      tema claro y oscuro, ventana 1024 × 560, foco visible en los nuevos enlaces, textos en
      español e inglés
- [X] T017 [P] Revisar la matriz de documentación de la skill `cierre-tarea` (¿toca
      `docs/user-stories.md`? ¿`docs/open-questions.md`?) y ejecutar `pnpm docs:build` si se editó
      algún documento normativo además de `docs/ui-contract.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias, empieza de inmediato
- **Foundational (Phase 2)**: no aplica — no bloquea nada
- **User Story 1 (Phase 3)**: depende de Setup (necesita las claves i18n)
- **User Story 2 (Phase 4)**: depende de que exista la lista de eventos de US1 (T004) — añade el
  `href` a filas que ya están renderizadas
- **User Story 3 (Phase 5)**: depende de que exista la `Card` de US1 (T004) — añade el snippet
  `action`; no depende de US2
- **Polish (Phase 6)**: depende de que las tres historias estén completas

### Dentro de cada historia

- US1: T003 (estado + petición) antes que T004 (render); T004 antes que T005/T006 (sus ramas
  condicionales); T007-T009 después de T003-T006
- US2: T010 después de T004; T011 después de T010
- US3: T012 después de T004; T013 después de T012

### Parallel Opportunities

- T001 y T002 en paralelo (ficheros distintos)
- T007, T008 y T009 en paralelo entre sí (casos de prueba independientes, mismo fichero pero
  bloques de `test()` distintos)
- T014 puede hacerse en paralelo con cualquier historia (fichero de documentación, sin relación de
  código)
- US2 (Phase 4) y US3 (Phase 5) no dependen entre sí: pueden implementarse en cualquier orden, o en
  paralelo si hay más de una persona, una vez completada US1

---

## Implementation Strategy

### MVP primero (User Story 1 sola)

1. Phase 1 (Setup): T001-T002
2. Phase 2 (Foundational): no aplica
3. Phase 3 (US1): T003-T009
4. **Parar y validar**: comprobar en la app real (`quickstart.md`) que la sección lista los eventos
   del disco, o su estado vacío, o su estado de error
5. Esto ya es el valor completo pedido por el usuario en su forma mínima: ver los eventos sin salir
   de la pantalla

### Entrega incremental

1. Setup → Phase 3 (US1) → validar → esto ya es demostrable
2. Phase 4 (US2): cada evento pasa a ser un enlace real al detalle completo
3. Phase 5 (US3): añadir el enlace "Ver todos"
4. Phase 6 (Polish): documentación y puertas de calidad

---

## Notes

- Ninguna tarea toca `src-tauri/`, `.specify/memory/`, `third-party/` ni `tauri.conf.json`: no hace
  falta modo plan para la implementación.
- Ninguna tarea añade una dependencia, un comando Tauri o un permiso nuevo.
- Commits en imperativo y en español, referenciando esta historia (`012-eventos-por-disco`), uno por
  tarea o grupo lógico de tareas — a confirmar con el usuario antes de cualquier commit real
  (`AGENTS.md`, límites duros).
