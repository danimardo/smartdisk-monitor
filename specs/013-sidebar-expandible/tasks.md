---
description: "Task list template for feature implementation"
---

# Tasks: Riel de navegación expandible con etiquetas de texto

**Input**: Design documents from `/specs/013-sidebar-expandible/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/ajustes-apariencia.md](./contracts/ajustes-apariencia.md),
[quickstart.md](./quickstart.md)

**Tests**: la constitución (principio VIII) no exige TDD para pantallas — las pruebas acompañan al
código. Se incluyen como tareas propias, una por caso de `quickstart.md`.

**Organización**: una fase por historia de usuario (`spec.md`), en orden de prioridad. US1 entrega
un expandir/plegar en memoria (sin persistir); US4 al final lo convierte en persistente — así cada
fase es un incremento real, nunca a medias.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: se puede hacer en paralelo (ficheros distintos, sin dependencia de una tarea sin acabar)
- **[Story]**: US1-US4, según `spec.md`
- Cada descripción lleva la ruta exacta del fichero que toca

## Path Conventions

Proyecto único ya establecido (`src/`, `src-tauri/`, `e2e/`, `docs/`) — sin carpetas nuevas.

---

## Phase 1: Setup (infraestructura compartida)

- [X] T001 [P] Añadir `nav.sidebar.expand` / `nav.sidebar.collapse` (nombre accesible del botón
      según el estado) a `src/lib/i18n/es.json`
- [X] T002 [P] Añadir las mismas claves, en inglés, a `src/lib/i18n/en.json`

**Checkpoint**: los diccionarios tienen las claves nuevas y `pnpm verify:i18n` no se queja.

---

## Phase 2: Foundational (prerrequisitos bloqueantes)

*No aplica.* Ninguna historia depende de una infraestructura compartida más allá de las claves
i18n del Setup. El campo persistido de US4 solo bloquea US4.

---

## Phase 3: User Story 1 - Ver el nombre de cada sección sin adivinar por el icono (Priority: P1) 🎯 MVP

**Goal**: un botón en el riel alterna un panel con icono + nombre de cada sección.

**Independent Test**: pulsar el botón y comprobar que aparece el panel con las seis secciones y
«Acerca de», cada una con su nombre visible; volver a pulsar lo cierra.

### Implementation for User Story 1

- [X] T003 [US1] En `src/lib/components/Sidebar.svelte`, añadir un botón de alternar (icono tipo
      flecha/chevron) con estado local `expanded = $state(false)` (sin persistir todavía — eso es
      US4), `aria-expanded={expanded}` y `aria-label`/`title` que alternan entre
      `t("nav.sidebar.expand")` y `t("nav.sidebar.collapse")`
- [X] T004 [US1] En el mismo fichero, renderizar el panel expandido cuando `expanded` es `true`:
      un `<nav>` con una fila por sección (`sections`, la misma prop que ya recibe el riel) donde
      cada fila es un `<a href={s.href}>` con `<Icon name={s.icon}>` **y** `{t(s.label)}` como
      texto visible — reutiliza el mismo dato que hoy solo va al `title`/`aria-label`, sin
      duplicar copy; incluye la fila de «Acerca de» y el indicador de estado global con su texto

### Tests for User Story 1

- [X] T005 [P] [US1] En `src/lib/components/Sidebar.browser.test.ts`, pruebas: plegado por
      defecto (sin panel visible); pulsar el botón muestra el panel con el icono y el nombre de
      cada sección; volver a pulsarlo lo cierra; `aria-expanded` refleja el estado en ambos casos

**Checkpoint**: expandir/plegar funciona en memoria, sin persistencia todavía — ya es
demostrable.

---

## Phase 4: User Story 2 - El panel expandido no reduce el espacio de la pantalla (Priority: P1)

**Goal**: expandir el riel no cambia el ancho del contenido en ningún tamaño de ventana.

**Independent Test**: con la ventana en 1024×560, expandir el riel y comprobar que ninguna región
de contenido cambia de ancho.

### Implementation for User Story 2

- [X] T006 [US2] En `Sidebar.svelte`, cuando `expanded` es `true`: el `<aside>` pasa a
      `class="absolute inset-y-0 left-0 z-40 w-[232px] ..."` con `sdm-material-overlay` y
      `shadow-lift` (reutilizados, sin tokens nuevos — D1/D2 de `research.md`); se añade un
      `<div class="w-rail flex-none" aria-hidden="true">` en el flujo normal mientras está
      expandido, para que `AppShell.svelte` no cambie de tamaño. Plegado, el `<aside>` vuelve a
      ser el elemento de flujo normal de 74 px de siempre, sin espaciador — cero cambios en
      `AppShell.svelte`

### Tests for User Story 2

- [X] T007 [P] [US2] En `e2e/ui/escalado.spec.ts` (o un nuevo `e2e/ui/sidebar.spec.ts`, a decidir
      al implementar según quepa mejor), prueba: con la ventana en 1024×560, expandir el riel y
      comprobar que el ancho de la región de contenido (`main`) no cambia respecto a plegado, y
      que no aparece scroll horizontal nuevo

**Checkpoint**: US1 + US2 — el panel se ve y no rompe el mínimo de ventana protegido por
`ADR-034`.

---

## Phase 5: User Story 3 - Cerrar el panel expandido sin usar el ratón (Priority: P2)

**Goal**: abrir y cerrar el panel es accesible por completo desde el teclado.

**Independent Test**: abrir con teclado, comprobar que el foco entra en el panel, `Escape` lo
cierra y el foco vuelve al botón.

### Implementation for User Story 3

- [X] T008 [US3] En `Sidebar.svelte`: `$effect` que mueve el foco al panel al abrir
      (`panel.focus()`, `tabindex="-1"`, mismo patrón que `ConfirmDialog.svelte`); una capa
      `<div class="fixed inset-0 z-30" role="presentation">` **sin fondo visible** (a diferencia
      del velo de un diálogo: este panel no bloquea el resto de la app) que captura el clic fuera
      y el `Escape` mientras está abierto; al cerrarse por cualquier vía, el foco vuelve al botón
      que abrió el panel

### Tests for User Story 3

- [X] T009 [P] [US3] En `Sidebar.browser.test.ts`, pruebas: al abrir, el foco entra en el panel;
      `Escape` cierra el panel y devuelve el foco al botón; un clic en la capa de fuera cierra el
      panel; un clic dentro del panel (una fila) no lo cierra por sí solo

**Checkpoint**: US1-US3 — la funcionalidad es accesible por completo sin ratón.

---

## Phase 6: User Story 4 - La preferencia se recuerda entre sesiones (Priority: P3)

**Goal**: el estado expandido/plegado persiste entre aperturas de la aplicación.

**Independent Test**: expandir, cerrar la aplicación, volver a abrirla y comprobar que sigue
expandido.

### Implementation for User Story 4

- [X] T010 [US4] **Modo plan de Claude Code antes de escribir este cambio** (toca `src-tauri/`).
      En `src-tauri/src/commands/mod.rs`: añadir `pub sidebar_expanded: bool` al struct
      `AppearanceSettings`, una constante `CLAVE_APARIENCIA_SIDEBAR_EXPANDIDO =
      "settings.appearance.sidebar_expanded"`, y su lectura en `get_appearance_settings_impl` con
      `leer_ajuste_bool(conn, CLAVE_APARIENCIA_SIDEBAR_EXPANDIDO, false)` — mismo patrón exacto que
      `use_system_accent`
- [X] T011 [P] [US4] Prueba Rust en `commands/mod.rs` (junto a las de `get_appearance_settings_impl`
      ya existentes): valor por defecto `false` sin ajuste guardado; `true` tras
      `guardar_ajuste(..., "settings.appearance.sidebar_expanded", &true, ...)`. `cargo test`
      regenera `src/lib/api/generated/AppearanceSettings.ts` con el campo nuevo
- [X] T012 [US4] En `src/lib/api/schemas.ts`, añadir `sidebarExpanded: z.boolean()` al esquema
      `appearanceSettings`
- [X] T013 [P] [US4] En `src/lib/api/schemas.test.ts`, prueba de rechazo: un `sidebarExpanded` que
      no sea booleano hace fallar `safeParse`
- [X] T014 [US4] En `src/routes/+layout.svelte`: `let expandedSidebar = $state(false)`,
      inicializado desde `appearance.sidebarExpanded` en el mismo bloque donde ya se inicializan
      tema e idioma; un manejador `alternarSidebar()` que invierte el valor y llama a
      `setSetting("settings.appearance.sidebar_expanded", expandedSidebar)`
- [X] T015 [US4] En `Sidebar.svelte`: sustituir el `expanded = $state(false)` interno de T003 por
      una prop `expanded` (controlada desde `+layout.svelte`) y un callback `onToggleExpand`; el
      resto del componente (T004, T006, T008) no cambia, solo deja de poseer su propio estado
- [X] T016 [P] [US4] Prueba e2e en `e2e/ui/smoke.spec.ts` o un nuevo `e2e/ui/sidebar.spec.ts`: con
      `get_appearance_settings` devolviendo `sidebarExpanded: true`, el riel arranca ya expandido
      sin pulsar nada; alternar el botón llama a `set_setting` con la clave
      `settings.appearance.sidebar_expanded` y el valor booleano correcto

**Checkpoint**: las cuatro historias funcionan juntas — expandir, sin romper el mínimo de
ventana, accesible por teclado, y recordado entre sesiones.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T017 [P] Documentar `sidebarExpanded` en `docs/ui-contract.md` §3.1
      (`contracts/ajustes-apariencia.md` ya tiene el texto exacto)
- [X] T018 [P] Añadir una nota/enmienda en `docs/decisions.md` junto a `ADR-034` explicando por qué
      el panel superpuesto no reabre el problema de 1024 px que ese ADR cerró (D1 de
      `research.md`)
- [X] T019 Ejecutar las nueve puertas completas: `pnpm check`, `pnpm lint`, `pnpm verify`,
      `pnpm test`, `pnpm test:component`, `pnpm test:e2e`, `pnpm test:a11y`, `pnpm build`,
      `pnpm docs:check`; y en `src-tauri/`: `cargo fmt --check`, `cargo clippy --all-targets -- -D
      warnings`, `cargo test`
- [X] T020 Repasar la pantalla contra la definición de terminado de `ui-design.md` §8: tema claro
      y oscuro, ventana 1024×560, escalado 125%/150%, estados de foco, textos en español e inglés
- [X] T021 [P] Revisar la matriz de documentación de `cierre-tarea` (¿corresponde una línea nueva
      en `docs/user-stories.md`, junto a la navegación?) y ejecutar `pnpm docs:build` si se editó
      algún documento normativo

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias
- **Foundational (Phase 2)**: no aplica
- **User Story 1 (Phase 3)**: depende de Setup (necesita las claves i18n)
- **User Story 2 (Phase 4)**: depende de que exista el panel de US1 (T004) — cambia cómo se
  posiciona, no qué contiene
- **User Story 3 (Phase 5)**: depende de que exista el panel de US1 (T004) — añade foco/`Escape`/
  clic fuera; independiente de US2
- **User Story 4 (Phase 6)**: depende de que el botón y el panel ya existan (T003-T004) — los
  convierte de estado local a estado controlado y persistido
- **Polish (Phase 7)**: depende de que las cuatro historias estén completas

### Dentro de cada historia

- US1: T003 (botón + estado) antes que T004 (panel, usa `expanded`); T005 después de ambas
- US2: T006 después de T004; T007 después de T006
- US3: T008 después de T004; T009 después de T008
- US4: T010 (Rust) → T011 (pruebas Rust + regenerar `.ts`) → T012 (esquema Zod) → T013 (prueba de
  rechazo) → T014 (`+layout.svelte`) → T015 (`Sidebar.svelte` pasa a controlado) → T016 (e2e)

### Parallel Opportunities

- T001 y T002 en paralelo (ficheros distintos)
- T007, T009 y T016 no se solapan con la implementación de otras historias una vez completado su
  prerrequisito
- T017 y T018 en paralelo con cualquier historia (documentación, sin relación de código)
- T011 y T013 en paralelo entre sí (Rust vs. TypeScript)

---

## Implementation Strategy

### MVP primero (User Story 1 + 2, P1 ambas)

1. Phase 1 (Setup): T001-T002
2. Phase 2 (Foundational): no aplica
3. Phase 3 (US1): T003-T005 — expandir/plegar en memoria
4. Phase 4 (US2): T006-T007 — que no rompa el mínimo de ventana
5. **Parar y validar**: con `quickstart.md`, comprobar visualmente que el panel se ve y no empuja
   nada, aunque todavía no sobreviva a cerrar la aplicación

### Entrega incremental

1. Setup → US1 → US2 → validar → ya es demostrable y no rompe nada existente
2. US3: accesible por teclado
3. US4: persistente entre sesiones (el único tramo que toca `src-tauri/`)
4. Polish: documentación y puertas de calidad

---

## Notes

- T010 es la única tarea que toca `src-tauri/`: usar modo plan de Claude Code antes de escribirla,
  según las instrucciones del proyecto para esa carpeta.
- Ninguna tarea añade una dependencia, un comando Tauri o un permiso nuevo.
- Commits en imperativo y en español, referenciando esta historia (`013-sidebar-expandible`) — a
  confirmar con el usuario antes de cualquier commit real (`AGENTS.md`, límites duros).
