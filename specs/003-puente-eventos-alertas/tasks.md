---

description: "Task list — Puente del registro de eventos de Windows al motor de alertas"
---

# Tasks: Puente del registro de eventos de Windows al motor de alertas

**Input**: `specs/003-puente-eventos-alertas/` (spec.md, plan.md, research.md, data-model.md,
contracts/eventos-alertas.md, quickstart.md)

**Tests**: SÍ son obligatorios. La spec los exige (FR-019: las 5 pruebas de `alert-rules.md` §5 por
regla; FR-020: correlación de ráfaga) y la constitución §VIII impone **test-first** para el motor de
alertas (activación, histéresis, deduplicación, ciclo de recaída). Las tareas 🔴 se escriben y
**fallan** antes de su implementación.

**Organización**: por historia de usuario de spec.md, en orden de prioridad. Cada historia es un
incremento demostrable por sí solo.

**Estado (2026-09-07): TODAS las tareas completas.** Motor de eventos en
`src-tauri/src/alerts/{reglas_eventos,eventos,correlacion_rafaga}.rs`, conectado por
`commands::refresh_events` y el barrido en `post_procesar_ciclo`. Enlace de interfaz alerta→suceso
(`/events?focus=`). ~56 pruebas nuevas de Rust + e2e. Todas las puertas en verde
(`cargo test` 556, `clippy`, `fmt`, `pnpm check/lint/verify/test/test:component/test:e2e/test:a11y/build/docs:check`).
Desviaciones respecto al plan, todas menores y documentadas en la tarea correspondiente: T008
(estampado tras la inserción en vez de tocar las 3 funciones), T011 (módulo `eventos.rs` propio),
T013 (el DTO ya existía), T037 (`inventory.duplicate_id` deduplica por par solo si el mensaje da
los dos números; si no, grupo único). Reglas de objetivo de volumen crean alertas **sin objeto**
mientras `system_events.volume_id` no se pueble (trabajo futuro del colector, anotado en J.16).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ir en paralelo (fichero distinto, sin dependencia de tareas incompletas)
- **[Story]**: US1–US5, o sin etiqueta en Setup / Foundational / Polish
- Rutas relativas a la raíz del repo

## Convenciones de ruta (de plan.md)

- Motor: `src-tauri/src/alerts/` — `motor.rs`, `mod.rs`, `agrupacion.rs`, `notificaciones.rs`, y los
  nuevos `reglas_eventos.rs` y `correlacion_rafaga.rs`
- Comandos: `src-tauri/src/commands/mod.rs` (`refresh_events`, `reconciliar_inventario`,
  `post_procesar_ciclo`)
- Persistencia: `src-tauri/src/persistence/` — `repo_alertas.rs`, `repo_varios.rs`
- Interfaz: `src/lib/i18n/{es,en}.json`, `src/lib/api/`, `src/routes/alerts/+page.svelte`,
  `src/routes/events/+page.svelte`, `e2e/ui/`

---

## Phase 1: Setup

**Purpose**: esqueletos de los módulos nuevos y el registro de decisiones previo a programar.

- [X] T001 Crear `src-tauri/src/alerts/reglas_eventos.rs` con los tipos de data-model.md §2
  (`Objetivo`, `ContextoDedup`, `Evaluacion`, `ClasificacionEvento`) y registrar `pub mod
  reglas_eventos;` en `src-tauri/src/alerts/mod.rs`. Solo tipos y `todo!()`; compila.
- [X] T002 Crear `src-tauri/src/alerts/correlacion_rafaga.rs` con `Rafaga`, `ResultadoCorrelacion` y
  la firma de `correlacionar_rafaga(...)`; registrar `pub mod correlacion_rafaga;` en
  `src-tauri/src/alerts/mod.rs`. Solo tipos y firma; compila.
- [X] T003 [P] Añadir a `docs/open-questions.md` §J las entradas D2–D6 de `research.md` marcadas
  `PROPUESTO` (disparadores de `device.removed_unexpected`; ubicación de `reglas_eventos`; ventana de
  60 s a caballo de ciclos; `triggeringEventId`; «solo hacia delante» sin marcador). Flujo de
  desarrollo §1: antes de programar. — J.47–J.51 añadidas; `pnpm docs:build` ejecutado.

---

## Phase 2: Foundational (bloquea TODAS las historias)

**Purpose**: la maquinaria compartida que hace que cualquier regla de evento llegue a producir un
`alert_group`, se notifique y aparezca en la interfaz.

**⚠️ CRITICAL**: ninguna historia puede empezar hasta cerrar esta fase.

- [X] T004 [P] Constructores de `EventoParaRegla` para las pruebas (`evento(conn, provider, id,
  cuando)` en `alerts/eventos.rs`, que persiste el `system_events` real por la FK de
  `triggering_event_id`). Los fixtures XML crudos ya existen en `collectors/event_log.rs`
  (`EVENTO_NTFS_98`, `EVENTO_DISK_158`); se añaden más en las historias que prueben el camino
  completo `refresh_events → correlación → evaluar` (US1). Storage Spaces con XML sintético, pendiente
  para su prueba.
- [X] T005 Definir `EventoParaRegla` (vista ligera, data-model.md §2) en
  `src-tauri/src/alerts/mod.rs` y su construcción desde `SystemEvent` + inventario (campo
  `es_extraible` a partir de `bus_type`/tipo de medio del disco correlacionado).
- [X] T006 🔴 Prueba de completitud **y fidelidad** de la tabla `reglas_eventos` en
  `src-tauri/src/alerts/reglas_eventos.rs`: (a) cada fila «genera alerta» de `docs/alert-rules.md`
  §3.2 tiene entrada y ningún id de §3.3 (`Microsoft-Windows-Ntfs` 98, `NvmeDisk` 501, `Volsnap`,
  `volmgr` 161, `Microsoft-Windows-Disk`) la tiene; (b) por cada `rule_key`, la severidad base, el
  objetivo, el contexto de deduplicación y la ventana de resolución de `ClasificacionEvento`
  coinciden con la fila correspondiente de `alert-rules.md` §2 — la tabla de valores esperados se
  escribe a mano en la prueba, no se deriva del código bajo prueba. Cubre FR-004.
- [X] T007 Rellenar la tabla `ClasificacionEvento` completa en
  `src-tauri/src/alerts/reglas_eventos.rs` (transcripción de `docs/alert-rules.md` §3.2: los 12
  `rule_key`, con proveedor(es), id(s), objetivo, contexto de dedup y `Evaluacion`). Hace pasar T006.
- [X] T008 Añadir `triggering_event_id: Option<i64>` a `agrupacion::EvaluacionAlerta` y escribirlo en
  las ocurrencias. **Implementado** como estampado tras la inserción: `agrupacion::procesar` llama a
  `repo_alertas::set_triggering_event_ultima_ocurrencia` cuando la transición registró una ocurrencia
  y el evaluador pasó el evento — cero cambios en las 3 funciones de inserción ni en sus pruebas
  (constitución §II.5). 6 literales de `EvaluacionAlerta` ganan `triggering_event_id: None`. Prueba
  nueva en `agrupacion.rs`.
- [X] T009 🔴 Pruebas del barrido de resolución temporal en `src-tauri/src/alerts/mod.rs`: un grupo
  `active` cuyo `last_occurrence_at_utc` queda más allá de la ventana de la regla (24 h / 7 días) →
  `resolved`; un evento nuevo después → `active`, `cycle + 1`, contador conservado.
- [X] T010 Implementar `resolver_grupos_de_eventos_vencidos(conn, ahora)` en
  `src-tauri/src/alerts/mod.rs`: recorre los `alert_groups` de reglas de eventos en
  `active`/`acknowledged`, calcula `resuelto = ahora - last_occurrence_at_utc > ventana` y lo aplica
  con `agrupacion::procesar`. Ventana por regla desde `reglas_eventos`.
- [X] T011 Implementar `evaluar_eventos` en `src-tauri/src/alerts/eventos.rs` (módulo nuevo, no
  `mod.rs`, para no mezclarlo con SMART/capacidad). Clasifica con `reglas_eventos`, resuelve
  target/context (data-model.md §3), llama a `agrupacion::procesar` con `triggering_event_id`.
  Atribución `unknown` → sin objeto + `context = provider:event_id` (FR-004a; `context_json` con la
  nota queda pendiente, no afecta al comportamiento observable). `PorFrecuencia` y `Especial` se
  reconocen pero no producen grupo (US3/US5); ráfaga sin integrar aún (US1). 8 pruebas.
- [X] T012 Conectar el motor al ciclo en `src-tauri/src/commands/mod.rs`: `refresh_events` acumula
  los eventos que `insert_event_if_new` marca nuevos (`Ok(true)`) y, tras persistirlos, llama a
  `evaluar_eventos` + `resolver_grupos_de_eventos_vencidos`; devuelve las transiciones.
  `refresh_now("all")` y `ejecutar_ciclo` (rama `EventosWindows`) las suman a `ResultadoCicloPost`;
  `post_procesar_ciclo` ya las notifica y emite `alerts:changed`. Un fallo del colector no aborta el
  ciclo (FR-015).
- [X] T012a 🔴 Prueba de «solo hacia delante» en `src-tauri/src/alerts/mod.rs` (`mod tests`):
  insertar varios eventos directamente en `system_events` con fechas pasadas (simula el histórico ya
  ingerido antes de desplegar el puente), ejecutar un ciclo de `evaluar_eventos` con la lista de
  «eventos nuevos» **vacía** más el barrido de resolución, y afirmar que **no** se crea ningún
  `alert_group`. Cubre FR-018 / SC-007 (research.md D6). Depende de: T011.
- [X] T013 [P] `triggeringEventId` en el DTO de ocurrencia. **Ya existía** desde la spec 001
  (Historia 4): `AlertOccurrenceWire.event_id` (`docs/ui-contract.md:278`, `schemas.ts:186`
  `eventId: z.string().nullable()`), solo que ningún dato lo poblaba. Con T008 ya llega poblado. Sin
  cambios de contrato.
- [X] T014 [P] Ampliar `notificaciones::politica_notificacion` en
  `src-tauri/src/alerts/notificaciones.rs` con los 12 `rule_key` nuevos (cooldowns de
  `docs/alert-rules.md` §2: `events.disk_error`/`filesystem_error`/`controller_reset`/`delayed_write`
  → 1 h; `paging_error`/`io_retry` → 6 h; `filesystem_repaired`/`disk_predictive` → 24 h;
  `filesystem_repair_storm` → 6 h; `storage_space_degraded` → 1 h; `device.removed_unexpected` →
  `Siempre` («ninguno»); `inventory.duplicate_id` → `SoloAlCambiarDeNivel` («una sola vez por
  par»)). Ampliar la prueba `las_reglas_en_alcance_tienen_titulo_y_resumen_propios`.

**Checkpoint**: el andamiaje está listo. Una regla implementada ya llega de punta a punta.

---

## Phase 3: User Story 1 — Saber que un disco se ha desconectado sin avisar (P1) 🎯 MVP

**Goal**: una desconexión imprevista de un disco produce **una** alerta `device.removed_unexpected`,
con la ráfaga de eventos derivados como ocurrencias suyas, y se resuelve al reconectarlo.

**Independent Test**: inyectar una ráfaga fixture (`disk` 157 + derivados) del mismo disco en 60 s →
un solo grupo; los derivados en su cronología; reaparición del disco → `resolved`.

### Tests (🔴 antes de implementar)

- [X] T015 🔴 [P] [US1] Pruebas de `correlacion_rafaga::correlacionar_rafaga` en
  `src-tauri/src/alerts/correlacion_rafaga.rs`: ráfaga con `disk` 157 → los demás se pliegan como
  ocurrencias; ráfaga sin `disk` 157 → cada evento sigue su regla (FR-009a); `disk` 157 que llega
  **después** de un derivado ya agrupado → el grupo derivado se marca para reasignación.
- [X] T016 🔴 [US1] Las 5 pruebas de `alert-rules.md` §5 para `device.removed_unexpected` en
  `src-tauri/src/alerts/mod.rs` (`mod tests`): (1) activación por `disk` 157 correlacionado y por
  baja de inventario de disco no-USB; (2) **no** activación con baja USB sin `disk` 157 y con
  reaparición inmediata; (3) histéresis: reaparecer y volver a desaparecer no crea dos grupos; (4)
  deduplicación; (5) ciclo: resolver al reaparecer, recaer al volver a desaparecer → `cycle + 1`.
  USB → severidad advertencia; fijo → crítica.

### Implementación

- [X] T017 [US1] Implementar `correlacion_rafaga::correlacionar_rafaga` (función pura, data-model.md
  §2). Hace pasar T015.
- [X] T018 [US1] Consulta de ventana de 60 s en `src-tauri/src/persistence/repo_varios.rs`: eventos
  del mismo `device_id` (y sin disco, por proximidad) en `[occurred_at - 60s, occurred_at]`, sobre
  los índices `idx_system_events_time`/`idx_system_events_device`. Integrarla en `evaluar_eventos`
  para que la correlación funcione aunque la ráfaga cruce dos ciclos (research.md D4), incluida la
  reasignación de grupos derivados cuando el `disk` 157 llega tarde.
- [X] T019 [US1] Implementar `device.removed_unexpected` en `src-tauri/src/alerts/mod.rs`
  (`evaluar_eventos`, rama `disk` 157) y en `src-tauri/src/commands/mod.rs`: en
  `reconciliar_inventario`, por cada id de `ids_dados_de_baja` de un disco **no USB**, emitir la
  transición de `device.removed_unexpected`; borrar el comentario obsoleto de T026 sobre «expulsión
  segura pendiente de la Historia 4». Resolución: el `fingerprint` reaparece en el inventario.
- [X] T020 [P] [US1] Añadir `alert.rule.device.removed_unexpected.{title,summary}` a
  `src/lib/i18n/es.json` y `en.json` (redacción según `alert-rules.md` §4; borrador en
  `contracts/eventos-alertas.md` §1).
- [X] T021 [US1] Ampliar `e2e/ui/alerts.spec.ts`: con una alerta `device.removed_unexpected` activa,
  `/alerts` muestra su título traducido, nunca la `rule_key`.

**Checkpoint**: US1 funcional y demostrable — desconectar un disco produce una alerta, no cuatro.

---

## Phase 4: User Story 2 — Enterarme de un error grave que registró Windows (P1)

**Goal**: los eventos críticos de almacenamiento (`Ntfs` 55/131, `disk` 7, `NvmeDisk` 500,
`StorageSpaces-Driver` 202/203/209 y 300–311, `Ntfs` 50 en volumen no extraíble) producen alertas
críticas con enlace al evento.

**Independent Test**: inyectar un fixture de cada familia → grupo con la severidad de la tabla,
título traducido, y el detalle enlaza al evento en `/events`.

### Tests (🔴)

- [X] T022 🔴 [US2] Las 5 pruebas de `alert-rules.md` §5 para `events.disk_error`,
  `events.filesystem_error`, `events.storage_space_degraded` y `events.delayed_write` en
  `src-tauri/src/alerts/mod.rs` (una batería por regla). Incluir: `Microsoft-Windows-Ntfs` 98 **no**
  activa nada (FR-005); `events.delayed_write` crítico si el volumen no es extraíble, advertencia si
  lo es; `events.filesystem_error` cubre `Ntfs` 55 y 131 con un `rule_key`.

### Implementación

- [X] T023 [US2] Implementar la evaluación `Inmediata` y `CondicionalNoExtraible` en
  `src-tauri/src/alerts/motor.rs` y conectarla en `evaluar_eventos` para las 4 reglas. Hace pasar
  T022.
- [X] T024 [P] [US2] Añadir a `src/lib/i18n/{es,en}.json` las claves `title`/`summary` de
  `events.disk_error`, `events.filesystem_error`, `events.storage_space_degraded`,
  `events.delayed_write`.
- [X] T025 [US2] En `src/routes/alerts/+page.svelte`, para una fila de cronología con
  `triggeringEventId != null`, mostrar un enlace «Ver el suceso» → `<a href="/events?focus=<id>">`
  (navegación por enlace, §XIV).
- [X] T026 [US2] En `src/routes/events/+page.svelte`, manejar `?focus=<id>`: resaltar / desplazar
  hasta ese evento; id inexistente = comportarse como sin parámetro, sin error. Documentar el
  parámetro en `docs/ui-contract.md`.
- [X] T027 [P] [US2] `e2e/ui/`: (a) `alerts.spec.ts` — títulos traducidos de las 4 reglas y el
  enlace «Ver el suceso» navega a `/events?focus=`; (b) `events.spec.ts` — `?focus=` resalta el
  evento; (c) `a11y.spec.ts` sigue en verde en `/alerts` y `/events`.
- [X] T027a [US2] Pasar la definición de terminado de `docs/ui-design.md` §8 en las pantallas
  tocadas: `/alerts` con el enlace «Ver el suceso» y `/events` con `?focus=` — tema claro y oscuro,
  ventana 1024×560 y 1280×720, escalado 125/150/200 %, estados (cronología vacía, evento no
  encontrado por `focus` inexistente), teclado y foco visible sobre el enlace nuevo, textos en los
  dos idiomas. Registrar el resultado; parcial admisible si algo exige la app real (mismo patrón que
  T045/T076 de la spec 001). Depende de: T025, T026.

**Checkpoint**: US1 y US2 funcionan de forma independiente. Un `Ntfs` 131 aparece como alerta con
traza al evento.

---

## Phase 5: User Story 3 — No recibir alertas por el ruido normal de Windows (P2)

**Goal**: `events.paging_error` (≥ 10/h, solo discos no extraíbles), `events.io_retry` (≥ 5/h) y la
escalada de `events.controller_reset` (≥ 3/h) solo alertan al cruzar su umbral de frecuencia.

**Independent Test**: inyectar N-1 eventos del mismo tipo/disco en 1 h → sin alerta; el N-ésimo →
un grupo.

### Tests (🔴)

- [X] T028 🔴 [US3] Pruebas de la evaluación `PorFrecuencia` en `src-tauri/src/alerts/motor.rs` en el
  borde (9 vs 10 `paging_error`, 4 vs 5 `io_retry`, 2 vs 3 para la escalada de `controller_reset`),
  exclusión de medios extraíbles en `paging_error`, y las 5 pruebas de §5 por regla en
  `src-tauri/src/alerts/mod.rs`.

### Implementación

- [X] T029 [US3] Consulta de conteo por ventana de 1 h en
  `src-tauri/src/persistence/repo_varios.rs` (eventos por `rule`/proveedor+id, `device_id`,
  `occurred_at_utc` en `[t-1h, t]`) e implementar `PorFrecuencia` en `motor.rs` + `evaluar_eventos`.
  El conteo va sobre `occurred_at_utc` (hora del evento), no la de ingesta (data-model.md §5).
- [X] T030 [P] [US3] Claves i18n de `events.paging_error`, `events.io_retry`,
  `events.controller_reset` en `src/lib/i18n/{es,en}.json`.
- [X] T031 [P] [US3] `e2e/ui/alerts.spec.ts`: títulos traducidos de las 3 reglas (el comportamiento
  de frecuencia lo cubren las pruebas unitarias).

**Checkpoint**: sobre el registro de referencia sano, cero alertas por eventos aislados.

---

## Phase 6: User Story 4 — Advertencias leves: reparaciones y predicción de fallo (P2)

**Goal**: `events.filesystem_repaired` (`Ntfs` 130), `events.filesystem_repair_storm` (`Ntfs` 132) y
`events.disk_predictive` (`disk` 52) producen grupos con la severidad y la ventana de resolución
larga (7 días) de la tabla.

**Independent Test**: inyectar un fixture de cada uno → grupo con severidad correcta; sin repetición
7 días → `resolved`.

### Tests (🔴)

- [X] T032 🔴 [US4] Las 5 pruebas de §5 para `events.filesystem_repaired`,
  `events.filesystem_repair_storm` y `events.disk_predictive` en `src-tauri/src/alerts/mod.rs`, con
  énfasis en la histéresis temporal de 7 días (reutiliza el barrido de T010).

### Implementación

- [X] T033 [US4] Conectar las 3 reglas (evaluación `Inmediata`) en `evaluar_eventos`; verificar que
  el barrido de T010 usa la ventana de 7 días para ellas. Hace pasar T032.
- [X] T034 [P] [US4] Claves i18n de las 3 reglas en `src/lib/i18n/{es,en}.json`.
- [X] T035 [P] [US4] `e2e/ui/alerts.spec.ts`: títulos traducidos de las 3 reglas.

**Checkpoint**: las cuatro primeras historias funcionan de forma independiente.

---

## Phase 7: User Story 5 — Detectar identidad de disco duplicada (P3)

**Goal**: el evento `disk` 158 produce `inventory.duplicate_id` una sola vez por pareja de discos.

**Independent Test**: inyectar un `disk` 158 → un grupo cuyo contexto es la pareja; un segundo evento
igual no crea grupo nuevo; sin repetición 7 días → `resolved`.

### Tests (🔴)

- [X] T036 🔴 [US5] Las 5 pruebas de §5 para `inventory.duplicate_id` en
  `src-tauri/src/alerts/mod.rs`: activación por `disk` 158; contexto de dedup = pareja de discos
  (identidades ordenadas, estable); segundo evento → ocurrencia, no grupo; resolución a 7 días.

### Implementación

- [X] T037 [US5] Implementar `inventory.duplicate_id` en `evaluar_eventos`: extraer las dos
  identidades del evento `disk` 158 (data-model.md §3, contexto «par de discos»), sin objeto de
  disco. `politica_notificacion` ya lo trata como `SoloAlCambiarDeNivel` (T014).
- [X] T038 [P] [US5] Claves i18n de `inventory.duplicate_id` en `src/lib/i18n/{es,en}.json`.
- [X] T039 [P] [US5] `e2e/ui/alerts.spec.ts`: título traducido de `inventory.duplicate_id`.

**Checkpoint**: las 12 reglas del alcance están implementadas y probadas.

---

## Phase 8: Polish & Cross-Cutting

- [X] T040 [P] Actualizar `docs/alert-rules.md`: marcar como implementadas las filas de `events.*`,
  `device.removed_unexpected` e `inventory.duplicate_id`; precisar en §3.5 la mecánica real de la
  ventana de 60 s (consulta a `system_events` hacia atrás; reasignación de grupos derivados).
- [X] T041 [P] Actualizar `docs/open-questions.md`: pasar D2–D6 (§J) de `PROPUESTO` a `DECIDIDO`;
  cerrar en J.16 la parte de `events.*`/`device.removed_unexpected`/`inventory.duplicate_id` (queda
  solo `temp.above_vendor_critical`).
- [X] T042 [P] Actualizar `docs/data-model.md` §2/§3 (nota de que `alert_occurrences.triggering_event_id`
  ya se escribe) y `docs/ui-contract.md` (`triggeringEventId`, `/events?focus`). Ejecutar
  `pnpm docs:build`.
- [X] T043 Decidir y documentar `J.17` (`smart.media_errors`/`smart.error_log` sin resolución
  temporal): o el barrido de T010 se generaliza para cubrirlas, o se deja explícitamente fuera del
  alcance de esta feature. Registrar la decisión en `docs/open-questions.md`.
- [X] T044 Ejecutar `specs/003-puente-eventos-alertas/quickstart.md` completo: las nueve puertas del
  frontend + `cargo fmt --check` / `clippy --all-targets -D warnings` / `cargo test`; cobertura
  `src-tauri/src/alerts/` y `src-tauri/src/domain/` ≥ 90 % (`cargo llvm-cov`).
- [X] T044a 🔴 Prueba de «cero falsos positivos sobre el corpus de referencia» en
  `src-tauri/src/alerts/mod.rs` (`mod tests`): fixture con un subconjunto representativo del registro
  de referencia de `docs/alert-rules.md` §3 — `disk` 51 ×15 sobre un disco fijo y ×15 sobre uno
  extraíble, `Microsoft-Windows-Ntfs` 98 ×5, `NvmeDisk` 501 ×3, `Volsnap` 25/33/36, `volmgr` 161,
  `disk` 11 ×2 (bajo el umbral de escalada) —; ejecutar `evaluar_eventos` sobre todos y afirmar
  **cero** grupos creados (ni críticos ni de advertencia). Es la comprobación **automatizada** de
  SC-001; la prueba de humo manual (T045) la complementa, no la sustituye. Depende de: T007, T011,
  T023, T029.
- [X] T045 [P] Prueba de humo manual sobre un Windows real (quickstart.md §5, opcional según
  disponibilidad): equipo sano → cero críticos falsos; desconectar/reconectar un USB de descarte →
  una alerta `device.removed_unexpected` que se resuelve.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias.
- **Foundational (Phase 2)**: depende de Setup. **BLOQUEA todas las historias.**
- **US1–US5 (Phases 3–7)**: dependen de Foundational. Entre sí son independientes y podrían
  paralelizarse; en secuencia van por prioridad (US1 → US2 → US3 → US4 → US5).
- **Polish (Phase 8)**: depende de las historias que se quieran cerrar.

### Dependencias concretas entre tareas

- T007 depende de T001, T006. T008 toca pruebas existentes de `repo_alertas`. T010 depende de T009.
  T011 depende de T005, T007, T008. T012 depende de T011, T010. T012a depende de T011.
- T018/T019 (US1) dependen de T011, T017. T019 borra el comentario de `reconciliar_inventario`.
- T023 (US2) depende de T007, T011. T025 depende de T013. T026 es independiente de T025 pero ambas
  cierran el enlace de US2. T027a depende de T025, T026.
- T044a (Phase 8) depende de T007, T011, T023, T029 (necesita las reglas inmediatas y las de
  frecuencia implementadas para que «cero grupos» sea una afirmación real).
- T029 (US3) y T018 (US1) tocan `repo_varios.rs` (consultas de ventana): coordinar si se hacen en
  paralelo — funciones distintas, mismo fichero.
- Las tareas de i18n (T020, T024, T030, T034, T038) tocan `es.json`/`en.json`: en secuencia sin
  problema; en paralelo, conflicto de fichero.
- T033 (US4) depende de que T010 lea la ventana de 7 días de `reglas_eventos` (T007).

### Paralelizables

- T003, T004 en Phase 1/2.
- T013, T014 en Phase 2 (ficheros distintos de T004–T012).
- Dentro de cada historia: la tarea 🔴 de pruebas y la de i18n y la de e2e ([P]) frente a la
  implementación de motor.
- Historias distintas por personas distintas tras cerrar Phase 2 (con la salvedad de `repo_varios.rs`
  e i18n).

---

## Parallel Example: Phase 2

```text
# En paralelo (ficheros distintos):
T004  Helpers de fixtures de eventos            (reglas_eventos.rs #[cfg(test)])
T013  triggeringEventId en el DTO de ocurrencia (src/lib/api/ o struct ts-rs)
T014  politica_notificacion + su prueba          (notificaciones.rs)
```

## Parallel Example: User Story 1

```text
# Tras T011/T012:
T015  🔴 pruebas de correlacionar_rafaga   (correlacion_rafaga.rs)
T016  🔴 5 pruebas de device.removed_unexpected (alerts/mod.rs)
T020  [P] i18n device.removed_unexpected    (es.json + en.json)
# luego, en serie: T017 → T018 → T019 → T021
```

---

## Implementation Strategy

### MVP (solo US1)

1. Phase 1: Setup (T001–T003).
2. Phase 2: Foundational (T004–T014 + T012a). **Crítico — bloquea todo.**
3. Phase 3: US1 (T015–T021).
4. **PARAR Y VALIDAR**: desconectar un disco produce una alerta `device.removed_unexpected`, no
   cuatro; reconectarlo la resuelve.

### Entrega incremental

1. Setup + Foundational → maquinaria lista.
2. + US1 → validar → demo (MVP: la desconexión imprevista).
3. + US2 → validar → demo (errores graves de Windows con traza al evento).
4. + US3 → validar → demo (silencio ante el ruido de fondo).
5. + US4 → validar → demo (advertencias de mantenimiento).
6. + US5 → validar → demo (identidad duplicada).
7. Phase 8: pulido, documentación normativa, cobertura, humo real.

---

## Notes

- 🔴 = test-first (constitución §VIII, `alert-rules.md` §5). Escribir la prueba, verla fallar, luego
  implementar.
- Toda regla con umbral se prueba en el umbral, justo por encima y justo por debajo
  (`.claude/rules/pruebas.md`).
- Fixtures anonimizados **al capturarlos**, nunca al usarlos. Sin números de serie ni nombres de
  equipo reales.
- Nada se commitea sin pedirlo. Al cerrar cada historia: sus textos en los dos idiomas, sus pruebas,
  y la definición de terminado de `docs/ui-design.md` §8 si se tocó interfaz (US1, US2).
- `docs/open-questions.md` antes de programar cualquier decisión no escrita (T003 la adelanta).
