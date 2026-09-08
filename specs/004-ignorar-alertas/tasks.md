---
description: "Task list — Ignorar una alerta de forma permanente"
---

# Tasks: Ignorar una alerta de forma permanente

**Feature**: `004-ignorar-alertas` · **Input**: `specs/004-ignorar-alertas/` (plan.md, spec.md,
research.md, data-model.md, contracts/comandos-alertas.md, quickstart.md)

**Tests**: SÍ se generan. La constitución §VIII fija cobertura mínima y cada módulo tocado
(`agrupacion`, `ciclo`, `repo_alertas`, `migrations`, `commands`, `health.ts`) ya tiene su suite;
la feature no es una excepción justificada.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ir en paralelo (fichero distinto, sin dependencias con tareas incompletas)
- **[Story]**: `[US1]` / `[US2]` / `[US3]` para las fases de historia; sin etiqueta en Setup,
  Foundational y Polish

## Path Conventions

Proyecto Tauri + SvelteKit: backend en `src-tauri/src/`, frontend en `src/`, migraciones en
`src-tauri/migrations/`, documentación normativa en `docs/`, pruebas de interfaz en `e2e/ui/`.

---

## Phase 1: Setup

**Purpose**: registrar la decisión de producto que gobierna la parte más delicada (el veto).

- [X] T001 Redactar **ADR-044** en `docs/decisions.md` (skill `adr`): conjunto de reglas no
  ignorables (`smart.health.failed`, `nvme.critical_warning`, `smart.wear_high`,
  `smart.spare_below_threshold`, `smart.media_errors`, `smart.error_log`,
  `events.disk_predictive`) y su motivo (daño físico / predicción de fallo: ignorarlas
  convertiría el monitor en algo que oculta lo que debe vigilar). Enlazar a `spec.md` y a la
  aclaración de 2026-09-08 sobre `events.filesystem_error` / `events.disk_error`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: el estado `ignored` debe existir en el tipo, en el esquema y en el repositorio antes
de que ninguna historia pueda tocar comportamiento. **Ninguna historia empieza hasta cerrar esta
fase.**

- [X] T002 [P] Añadir `AlertStatus::Ignored` a `src-tauri/src/domain/tipos.rs` (el
  `rename_all = "snake_case"` ya produce `"ignored"`) y el campo `ignored_at_utc: Option<String>`
  a `AlertGroup`, tras `archived_at_utc`.
- [X] T003 Crear la migración `src-tauri/migrations/0003_alerta_ignored.sql` según research.md R1
  (**enfoque final**, tras descartar `legacy_alter_table` en la implementación): (1) mover
  `alert_occurrences` a `alert_occurrences_tmp` **sin** clave ajena; (2) reconstruir `alert_groups`
  con el `CHECK` de `status` ampliado a `'ignored'` y la columna `ignored_at_utc TEXT`, recrear sus
  dos índices; (3) recrear `alert_occurrences` con la clave ajena de vuelta y devolver las filas
  (id explícito), recrear `idx_alert_occurrences_group`. Registrar
  `Migration { version: 3, name: "alerta_ignored", sql: include_str!(...) }` al final de
  `MIGRATIONS` en `src-tauri/src/persistence/migrations.rs`.
- [X] T004 [P] Prueba en `src-tauri/src/persistence/migrations.rs` (`mod tests`): partiendo de una
  base con un grupo y varias `alert_occurrences`, aplicar hasta la 3 y verificar que (a) el grupo
  y **todas** sus ocurrencias siguen ahí, (b) `INSERT INTO alert_groups (... status) VALUES (...,
  'ignored')` se acepta, (c) `PRAGMA foreign_key_check` no devuelve filas.
- [X] T005 Ampliar `src-tauri/src/persistence/repo_alertas.rs`: `status_to_str` / `status_from_str`
  con `"ignored"`; `row_to_group` lee `ignored_at_utc`; `create_group` inserta `ignored_at_utc`;
  brazo `AlertStatus::Ignored` en `set_status` (`UPDATE ... status=?, ignored_at_utc=?`); el brazo
  `AlertStatus::Active` añade `ignored_at_utc = NULL`; nueva fn
  `dejar_de_ignorar(conn, id) -> rusqlite::Result<()>` (`status='resolved'`,
  `resolved_at_utc=ahora`, `ignored_at_utc=NULL`).
- [X] T006 [P] Crear `src-tauri/src/alerts/reglas.rs` (y `pub mod reglas;` en
  `src-tauri/src/alerts/mod.rs`): `pub const REGLAS_NO_IGNORABLES: &[&str]` con las siete claves y
  `pub fn regla_es_ignorable(rule_key: &str) -> bool`. Prueba que recorre la tabla de
  `docs/alert-rules.md` (lista embebida en el test) y comprueba que exactamente esas siete no son
  ignorables y una muestra del resto sí.
- [X] T007 Añadir la variante `OcurrenciaIgnorada` a `enum Transicion` en
  `src-tauri/src/alerts/agrupacion.rs` y el brazo `Transicion::OcurrenciaIgnorada => false` en
  `debe_enviar` de `src-tauri/src/alerts/notificaciones.rs` (el compilador lo exige). Sin lógica de
  `procesar` todavía: solo que el árbol compile y `cargo test` pase.

**Checkpoint**: el proyecto compila con el estado nuevo; las historias pueden empezar.

---

## Phase 3: User Story 1 — Dejar de ver para siempre una alerta aceptada (P1) 🎯 MVP

**Goal**: el usuario ignora una alerta de una regla ignorable; desaparece de *Activas*, deja de
teñir el disco y de notificar, pero sigue registrando ocurrencias sin reactivarse.

**Independent Test**: con una alerta `active` de regla ignorable, `ignore_alert` (o el botón) →
la alerta sale de *Activas* y del color, y aparece en la pestaña *Ignoradas* con estado `ignored`;
forzar otra ocurrencia y comprobar en su cronología que se registró sin notificación, sin cambio
de color y sin avanzar `cycle`. (La pestaña *Ignoradas* se crea aquí, en US1, para que el texto
del diálogo de confirmación sea veraz; la acción «Dejar de ignorar» llega en US2.)

### Implementation — backend

- [X] T008 [US1] En `src-tauri/src/alerts/agrupacion.rs::procesar`, añadir **antes** de la rama
  catch-all de reactivación la rama `(Some(g), Some(severidad)) if g.status == AlertStatus::Ignored`:
  `record_occurrence(conn, &g.id, g.cycle, …)` (sin tocar `cycle`), `if severidad_sube(severidad,
  g.severity) { set_severity(…) }`, devolver `Transicion::OcurrenciaIgnorada`. Cambiar la rama de
  reactivación a `(Some(g), Some(severidad)) if matches!(g.status, Resolved | Archived)`; el resto
  cae en `_ => SinCambio`.
- [X] T009 [P] [US1] Pruebas en `src-tauri/src/alerts/agrupacion.rs` (`mod tests`): (a) un grupo
  `ignored` que vuelve a cumplirse registra ocurrencia, incrementa `occurrence_count`, **no**
  cambia `cycle` ni `status`, devuelve `OcurrenciaIgnorada`; (b) un grupo `ignored` cuya severidad
  sube de `Warning` a `Critical` actualiza `severity` pero sigue `ignored`; (c) un grupo `ignored`
  **no** entra en `reopen_as_new_cycle`.
- [X] T010 [US1] Añadir `ignorar(conn, id, ahora_utc) -> Result<(), IgnorarError>` a
  `src-tauri/src/alerts/ciclo.rs`: `get_group` → si no existe `IgnorarError::NoExiste`; si
  `!reglas::regla_es_ignorable(&g.rule_key)` → `IgnorarError::ReglaNoIgnorable`; si no,
  `repo_alertas::set_status(conn, id, AlertStatus::Ignored, ahora_utc)`. Definir `enum IgnorarError`.
- [X] T011 [P] [US1] Pruebas en `src-tauri/src/alerts/ciclo.rs`: `ignorar` deja `status='ignored'`
  y escribe `ignored_at_utc` **partiendo de `active`, de `acknowledged`, de `resolved` y de
  `archived`** (FR-008: se puede ignorar desde cualquier estado); un grupo `ignored` **no** aparece
  en `repo_alertas::list_groups_counting_toward_health`; `ignorar` de un grupo de regla vetada
  devuelve `IgnorarError::ReglaNoIgnorable` y no cambia el estado; ignorar conserva
  `occurrence_count`, `first_occurrence_at_utc` y todas las `alert_occurrences` previas (FR-007).
- [X] T012 [US1] Comando `ignore_alert(app, state, alert_group_id)` en
  `src-tauri/src/commands/mod.rs` (patrón de `archive_alert`): `existe_grupo` →
  `alerts::ciclo::ignorar` mapeando `IgnorarError::ReglaNoIgnorable` a
  `AppError::new("alert.rule_not_ignorable", "error.alertRuleNotIgnorable")` (no reintentable) →
  `emitir_alerts_changed` → `platform::bandeja::actualizar`. Registrar `commands::ignore_alert` en
  `src-tauri/src/lib.rs`.
- [X] T013 [P] [US1] Pruebas de comando en `src-tauri/src/commands/mod.rs` (`mod tests`):
  `ignore_alert` de un grupo de regla ignorable lo deja `ignored` y fuera de
  `get_alert_groups_impl(status=[Active])`; `ignore_alert` de un grupo `smart.wear_high` devuelve
  `AppError` con código `alert.rule_not_ignorable`.
- [X] T014 [P] [US1] Prueba en `src-tauri/src/alerts/notificaciones.rs`: `debe_enviar` con
  `Transicion::OcurrenciaIgnorada` devuelve `false` en toda combinación de silencio/cooldown.

### Implementation — frontend

- [X] T015 [P] [US1] `src/lib/api/schemas.ts`: `alertStatus = z.enum([... , "ignored"])`.
  Regenerar `src/lib/api/generated/AlertStatus.ts` ejecutando `cargo test` (ts-rs) y confirmar el
  diff.
- [X] T016 [P] [US1] `src/lib/api/client.ts`: `export const ignoreAlert = (alertGroupId: string)
  => callVoid("ignore_alert", { alertGroupId });`.
- [X] T017 [US1] `src/routes/alerts/+page.svelte`: (a) **pestaña «Ignoradas»** — `type Filtro`
  gana `"ignored"`, `ESTADOS_POR_FILTRO.ignored = ["ignored"]`, y `{ id: "ignored", label:
  t("alerts.filter.ignored") }` como 5.º elemento de `opcionesFiltro` (así el `confirmImpact` del
  diálogo, que menciona esa pestaña, es veraz ya en el MVP); (b) importar `ignoreAlert`, estado
  `ignoreDialogOpen`, botón **«Ignorar»** en la barra de acciones del detalle cuando
  `detail.status !== "ignored" && detail.status !== "archived"`, que abre un `ConfirmDialog`
  (`title`/`body`/`impact` nuevos) cuya confirmación llama
  `ejecutar(() => ignoreAlert(detail.id))`; añadir `&& detail.status !== "ignored"` a
  `puedeSilenciar`. (El estado deshabilitado por regla vetada es US3; la acción «Dejar de ignorar»
  es US2.)
- [X] T018 [P] [US1] `src/lib/i18n/es.json` y `src/lib/i18n/en.json`: `alerts.filter.ignored`
  («Ignoradas» / «Ignored»), `alerts.actions.ignore`, `alerts.actions.ignore.hint`,
  `alerts.ignore.confirmTitle`, `alerts.ignore.confirmBody` («Se sigue registrando en segundo
  plano, pero deja de avisarte y de contar para el color del disco.»), `alerts.ignore.confirmImpact`
  («Puedes revertirlo desde la pestaña Ignoradas.»), `alerts.status.ignored`, `alert.status.ignored`,
  `error.alertRuleNotIgnorable`. Verificar con `pnpm verify:i18n`.
- [X] T019 [P] [US1] `src/lib/design/health.ts`: actualizar el comentario de
  `COUNTS_TOWARD_HEALTH` para incluir `ignored` entre los que no cuentan (el array no cambia:
  `ignored` no está en él). Añadir caso en `src/lib/design/health.test.ts`: una alerta `ignored`
  no aporta a `deviceState()`.
- [X] T020 [P] [US1] `src/lib/components/AlertCard.svelte`: comprobar que la línea de estado
  (`t(\`alert.status.${alert.status}\`)`) se compone bien para `ignored` en ambos temas; ajustar
  solo si la píldora/color desentona.
- [X] T021 [US1] Escenario en `e2e/ui/alerts.spec.ts` (Playwright + `mockIPC`, es donde este
  proyecto prueba el comportamiento de una ruta — `docs/testing-strategy.md` §10; no hay test de
  componente para `+page.svelte`): seleccionar una alerta `active` de regla ignorable, pulsar
  «Ignorar», comprobar que el `ConfirmDialog` muestra su impacto, confirmar, y que la alerta sale
  de *Activas* y aparece en la pestaña *Ignoradas*. Ampliar `e2e/ui/fixtures/respuestas.ts` si hace
  falta. (El comportamiento genérico de `Button` deshabilitado con `disabledReason` ya lo cubre
  `src/lib/components/Button.browser.test.ts`.)

**Checkpoint**: MVP — se puede ignorar una alerta ignorable y el sistema se comporta como debe.

---

## Phase 4: User Story 2 — Revisar y revertir lo ignorado (P1)

**Goal**: acción **«Dejar de ignorar»** que devuelve un grupo `ignored` a la vida normal. (La
pestaña *Ignoradas* ya la creó US1; aquí se le añade la capacidad de revertir.)

**Independent Test**: ignorar una alerta, abrir la pestaña *Ignoradas* y verla listada con su
disco/severidad/última ocurrencia; pulsar «Dejar de ignorar» → si la condición ya no se cumple
queda en *Resueltas*, si se cumple aparece en *Activas* en el ciclo siguiente.

### Implementation — backend

- [X] T022 [US2] Añadir `dejar_de_ignorar(conn, id) -> rusqlite::Result<()>` a
  `src-tauri/src/alerts/ciclo.rs` delegando en `repo_alertas::dejar_de_ignorar` (T005).
- [X] T023 [P] [US2] Pruebas en `src-tauri/src/alerts/ciclo.rs`: `dejar_de_ignorar` deja
  `status='resolved'`, `ignored_at_utc = NULL`, `resolved_at_utc` puesto; un grupo que se **archivó
  y luego se ignoró** acaba en `resolved` tras `dejar_de_ignorar`, **nunca de vuelta en
  `archived`** (edge case del spec); y un grupo así resuelto, al pasar por `agrupacion::procesar`
  con la condición cumpliéndose, se reactiva a `active` con `cycle + 1` (prueba de integración
  corta en `agrupacion.rs` o `ciclo.rs`).
- [X] T024 [US2] Comando `unignore_alert(app, state, alert_group_id)` en
  `src-tauri/src/commands/mod.rs` (patrón de `unmute_alert`): `existe_grupo` →
  `alerts::ciclo::dejar_de_ignorar` → `emitir_alerts_changed` → `platform::bandeja::actualizar`.
  Registrar `commands::unignore_alert` en `src-tauri/src/lib.rs`.
- [X] T025 [P] [US2] Prueba de comando en `src-tauri/src/commands/mod.rs`: `unignore_alert` de un
  grupo `ignored` lo deja en `resolved` y lo saca de `get_alert_groups_impl(status=[Ignored])`.

### Implementation — frontend

- [X] T026 [P] [US2] `src/lib/api/client.ts`: `export const unignoreAlert = (alertGroupId:
  string) => callVoid("unignore_alert", { alertGroupId });`.
- [X] T027 [US2] `src/routes/alerts/+page.svelte`: botón **«Dejar de ignorar»** en la barra de
  acciones cuando `detail.status === "ignored"`, que llama `ejecutar(() => unignoreAlert(detail.id))`.
  Ocultar «Archivar» / «Reconocer» / «Silenciar» para `status === "ignored"` (`puedeArchivar`,
  `puedeSilenciar`). (El 5.º segmento «Ignoradas» ya lo añadió T017.)
- [X] T028 [P] [US2] `src/lib/i18n/es.json` y `en.json`: `alerts.actions.unignore`,
  `alerts.actions.unignore.hint`.
- [X] T029 [US2] `e2e/ui/alerts.spec.ts`: escenario — ignorar una alerta desde el detalle,
  cambiar a la pestaña *Ignoradas*, verla listada; pulsar «Dejar de ignorar» y comprobar que sale
  de *Ignoradas*. Usar los fixtures de `e2e/ui/fixtures/respuestas.ts` (ampliar si hace falta un
  grupo `ignored`).
- [X] T030 [US2] Escenario en `e2e/ui/alerts.spec.ts`: con el fixture con un grupo `ignored` y
  otro no, la pestaña *Ignoradas* muestra solo el `ignored`; con un fixture sin ninguno, se ve el
  estado vacío (`alerts.empty.title` / `.body`). (Comparte fichero con T021/T029/T037 → no `[P]`
  entre ellos.)

**Checkpoint**: US1 + US2 — ignorar es visible y reversible.

---

## Phase 5: User Story 3 — No se puede ignorar un disco que se está muriendo (P1)

**Goal**: para las siete reglas vetadas, la acción «Ignorar» aparece deshabilitada con su motivo y
el backend rechaza cualquier intento.

**Independent Test**: abrir el detalle de una alerta `smart.wear_high` → el botón «Ignorar» está
deshabilitado y al enfocar/pasar el ratón muestra el motivo; una llamada directa a `ignore_alert`
con ese grupo devuelve `AppError alert.rule_not_ignorable` (ya cubierto por T013) y el grupo no
cambia.

### Implementation

- [X] T031 [US3] `src/lib` contrato: en `src-tauri/src/commands/mod.rs`, `AlertDetail` gana
  `rule_ignorable: bool` (serializa a `ruleIgnorable`); `get_alert_detail_impl` lo calcula con
  `alerts::reglas::regla_es_ignorable(&grupo.rule_key)`. Actualizar el struct espejo / doc-comment
  correspondiente.
- [X] T032 [P] [US3] Prueba en `src-tauri/src/commands/mod.rs`: `get_alert_detail_impl` devuelve
  `rule_ignorable = false` para un grupo `smart.wear_high` y `true` para `temp.above_configured_warn`.
- [X] T033 [P] [US3] `src/lib/api/schemas.ts`: `alertDetail = alertGroup.extend({ ruleIgnorable:
  z.boolean(), … })`.
- [X] T034 [US3] `src/routes/alerts/+page.svelte`: cuando el botón «Ignorar» se muestra pero
  `!detail.ruleIgnorable`, renderizarlo `disabled` con
  `disabledReason={t("alerts.ignore.notIgnorable")}` (no abre el diálogo).
- [X] T035 [P] [US3] `src/lib/i18n/es.json` y `en.json`: `alerts.ignore.notIgnorable` («Esta
  alerta señala un posible fallo del disco y no se puede ignorar.» / equivalente en inglés).
- [X] T036 [US3] Escenario en `e2e/ui/alerts.spec.ts`: seleccionar una alerta de regla vetada
  (`ruleIgnorable: false` en el fixture) y comprobar que «Ignorar» está deshabilitada y que su
  `title` es el motivo (`alerts.ignore.notIgnorable`); con una de regla ignorable, el botón está
  activo. (El comportamiento genérico `disabled` + `disabledReason` de `Button` ya está probado en
  `src/lib/components/Button.browser.test.ts`; esto valida el cableado en `+page.svelte`.)
- [X] T037 [P] [US3] Verificar que `e2e/ui/fixtures/respuestas.ts` incluye una alerta de regla
  vetada con `ruleIgnorable: false` y otra ignorable con `ruleIgnorable: true`, para T036 y para
  el escenario de US1/US2. (Si T029 ya lo cubre, esta tarea se cierra confirmándolo.)

**Checkpoint**: las tres historias funcionan de forma independiente.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T038 [P] `docs/alert-rules.md` §1 (FR-015): añadir `ignored` a estados y a la tabla de
  transiciones (ignorar desde cualquier estado; `ignored` + condición ⇒ ocurrencia sin cambio de
  estado ni de ciclo; `ignored` ⇒ `dejar de ignorar` ⇒ `resolved`; restricción de que solo
  `resolved`/`archived` reabren); actualizar «Qué cuenta para el color» (`ignored` no cuenta);
  añadir el conjunto de reglas no ignorables (nueva subsección o columna).
- [X] T039 [P] `docs/ui-contract.md` §3.4: `ignore_alert` / `unignore_alert`,
  `AlertDetail.ruleIgnorable`, error `alert.rule_not_ignorable`; nota de que `get_alert_groups`
  acepta `status: ["ignored"]`.
- [X] T040 [P] `docs/data-model.md` §2 (`alert_groups`): lista de estados con `ignored`, columna
  `ignored_at_utc`, y la nota de que `cycle` no avanza mientras está `ignored`.
- [X] T041 [P] `docs/ui-design.md` §7.3: pestaña *Ignoradas* en el filtro de `/alerts` y las
  acciones «Ignorar» / «Dejar de ignorar» del panel de detalle (incluida la variante deshabilitada
  con motivo para reglas vetadas).
- [X] T042 Ejecutar `pnpm docs:build` y `pnpm docs:check` (regenerar `historias.md`).
- [X] T043 Pasar las nueve puertas de `cierre-tarea`: `pnpm check` · `pnpm lint` · `pnpm verify` ·
  `pnpm test` · `pnpm build` · `pnpm docs:check`; y desde `src-tauri/`: `cargo fmt --check` ·
  `cargo clippy --all-targets -- -D warnings` · `cargo test`.
- [ ] T044 **PENDIENTE (manual)** Ejecutar `quickstart.md` con `pnpm app:dev`: escenarios 1–5,
  tema claro/oscuro, ventana 1024 × 560 (los 5 segmentos caben), escalado 125/150/200 %. No
  automatizable: `pnpm app:dev` pide UAC y necesita un Windows real. Cubierto por e2e/componente
  todo salvo la comprobación visual en la app real.
- [X] T045 Revisar si algún `svelte-ignore` / `eslint-disable` nuevo — si lo hay, entrada en
  `docs/known-issues.md` enlazada por número (probablemente ninguno).
- [X] T046 [P] FR-016 — retención: prueba en `src-tauri/src/domain/retencion.rs` (`mod tests`) de
  que `compact_samples` (y cualquier otra ruta de compactación) **no** borra ni modifica
  `alert_groups` ni `alert_occurrences` de un grupo en estado `ignored`. `domain::retencion` hoy
  solo toca `metric_samples`, así que la prueba fija esa garantía por si cambia; si al abordarlo se
  ve que no hay ninguna ruta de borrado alcanzable, dejar la prueba mínima + una línea en
  `docs/data-model.md` §4 confirmando que `ignored` queda fuera de la retención.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (T001)**: independiente; puede ir en paralelo con Foundational.
- **Foundational (T002–T007)**: bloquea todas las historias. Dentro: T002 y T006 en paralelo;
  T003 antes de T004; T005 después de T002; T007 después de T002.
- **US1 (T008–T021)**: tras Foundational. MVP.
- **US2 (T022–T030)**: tras Foundational. Reutiliza `repo_alertas::dejar_de_ignorar` (T005). US1
  y US2 comparten `+page.svelte`: si se hacen en paralelo, T017 y T027 no se marcan [P] entre sí.
- **US3 (T031–T037)**: tras Foundational. Independiente de US2; comparte `+page.svelte`,
  `schemas.ts`, i18n y `e2e/ui/alerts.spec.ts` con US1/US2.
- **Polish (T038–T046)**: tras cerrar las historias que se vayan a entregar.

### Conflictos de fichero (no paralelizar entre sí)

- `src/routes/alerts/+page.svelte`: T017 (segmento + botón Ignorar), T027 (botón Dejar de ignorar),
  T034 (deshabilitado por regla vetada)
- `src/lib/api/schemas.ts`: T015 (`alertStatus`), T033 (`alertDetail`)
- `src/lib/i18n/es.json` + `en.json`: T018, T028, T035
- `e2e/ui/alerts.spec.ts` + `e2e/ui/fixtures/respuestas.ts`: T021, T029, T030, T036, T037
- `src-tauri/src/commands/mod.rs`: T012, T024, T031 (+ pruebas T013, T025, T032 en el mismo `mod tests`)
- `src-tauri/src/alerts/ciclo.rs`: T010, T022 (+ T011, T023)
- `src-tauri/src/alerts/agrupacion.rs`: T008 (+ T009, y la parte de integración de T023)
- `src-tauri/src/lib.rs`: T012, T024

### User Story Dependencies

- US1, US2, US3 son **todas P1** y todas independientes una vez cerrada la fase Foundational.
  Orden recomendado por valor incremental: US1 → US2 → US3.

---

## Parallel Example: Foundational

```text
# En paralelo (ficheros distintos):
T002  domain/tipos.rs  — AlertStatus::Ignored + ignored_at_utc
T006  alerts/reglas.rs — REGLAS_NO_IGNORABLES + regla_es_ignorable

# Luego, en serie:
T003  migrations/0003 + migrations.rs
T004  prueba de migración   (tras T003)
T005  repo_alertas.rs       (tras T002)
T007  Transicion + debe_enviar (tras T002)
```

## Parallel Example: User Story 1 (tras backend)

```text
T015  schemas.ts (+ regenerar AlertStatus.ts)
T016  client.ts — ignoreAlert
T018  i18n es/en
T019  health.ts + health.test.ts
T020  AlertCard.svelte
# T017 (+page.svelte: segmento Ignoradas + botón) tras T016/T018.
# T021 (e2e alerts.spec.ts) tras T017.
```

---

## Implementation Strategy

### MVP (solo US1)

1. Phase 1 (T001) + Phase 2 (T002–T007).
2. Phase 3 (T008–T021).
3. **Parar y validar**: ignorar una alerta ignorable, comprobar color/notificación/cronología y
   que aparece en la pestaña *Ignoradas* (sin poder revertir todavía — eso es US2).

### Entrega incremental

1. Foundational listo.
2. + US1 → validar → demo (MVP: se puede ignorar).
3. + US2 → validar → demo («Dejar de ignorar» revierte).
4. + US3 → validar → demo (veto visible y aplicado).
5. Polish (docs + puertas + quickstart) antes de proponer commit.

---

## Notes

- `[P]` = ficheros distintos, sin dependencias.
- Verificar que las pruebas nuevas fallan antes de implementar la lógica.
- `cargo test` regenera `generated/AlertStatus.ts`: confirmar ese diff junto al resto.
- No proponer commit sin autorización (`AGENTS.md`); al terminar, mensaje en español e imperativo.
- La migración `0003`, una vez publicada, **no se edita** (`persistence/migrations.rs` cabecera).
