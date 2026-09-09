---
description: "Lista de tareas — Actividad de disco representativa mediante ventana continua"
---

# Tasks: Actividad de disco representativa mediante ventana continua

**Input**: `specs/007-actividad-disco-representativa/` (plan.md, spec.md, research.md, data-model.md, contracts/disk-activity.md, quickstart.md)

**Tests**: SÍ se incluyen. La constitución §VIII exige **test-first** para `domain/` (la agregación
es un área de fallo silencioso, FR-021) y **prueba de rechazo** para todo esquema Zod nuevo (§XI).
Los componentes llevan cobertura de estados (válido / parcial / no disponible), no porcentaje.

**Organización**: por historia de usuario. La Fase 2 (fundacional) es grande a propósito: el motor
de ventana y el cambio de contrato son infraestructura compartida por las tres historias.

## Formato: `[ID] [P?] [Story] Descripción con ruta de fichero`

- **[P]**: paralelizable (fichero distinto, sin dependencia de una tarea incompleta).
- **[Story]**: US1 / US2 / US3 en las fases de historia; sin etiqueta en Setup, Fundacional y Pulido.

---

## Phase 1: Setup

**Purpose**: dejar registrados los valores numéricos **antes** de programarlos (constitución, flujo
de desarrollo §1) y crear el hueco del módulo nuevo.

- [ ] T001 Añadir la subsección **D.4** a `docs/open-questions.md` con estado `PROPUESTO`: intervalo de muestreo (1 s en red, 4 s en batería), tamaño de la ventana (= `schedule.metrics_fast_seconds`), umbral de hueco (3 × intervalo), regla de hueco en el histórico cuando la ventana es parcial, y la nota de que la ventana es estado en memoria no persistido. Valores tomados de `research.md` R3.
- [ ] T002 [P] Crear `src-tauri/src/domain/actividad.rs` con solo el `//!` de cabecera (qué resuelve, por qué es puro) y registrar `pub mod actividad;` en `src-tauri/src/domain/mod.rs`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: ninguna historia puede empezar hasta que esta fase esté completa y `cargo test` +
`pnpm check` estén en verde. Aquí vive el motor de ventana, la consulta PDH persistente y la
migración del contrato `activityPercent → activity` con todos sus consumidores adaptados.

### Dominio de la ventana (test-first)

- [ ] T003 [P] Escribir las pruebas unitarias **que fallan** de `VentanaActividad` en el `#[cfg(test)]` de `src-tauri/src/domain/actividad.rs`: media aritmética y pico sobre muestras conocidas; ventana que aún no cubre la cadencia → `Parcial` con media/pico de lo que hay; ventana vacía → `NoDisponible` con medias `None`; una muestra separada > umbral de hueco vacía la ventana antes de agregar; muestras > tamaño de ventana se descartan por el frente; **una secuencia con instantes en los que el colector no llamó a `registrar` (muestra fallida) no introduce ningún `0.0` ni desplaza la media** (FR-005); `Instant` sintético (un cambio de hora no la altera). (FR-021, spec §User Story tests)
- [ ] T004 Implementar `MuestraActividad`, `VentanaActividad` (`nueva`, `registrar`, `agregado`, `purgar`), `AgregadoActividad`, `EstadoActividad` en `src-tauri/src/domain/actividad.rs` hasta que T003 pase. Sin dependencias de PDH, Tauri ni SQLite (constitución §IV). Ver `data-model.md` §1.2–§1.3.

### Contrato y DTO (Rust)

- [ ] T005 Añadir los structs `ActividadDisco` y `EstadoActividad` con `#[derive(Serialize, TS)]` en `src-tauri/src/commands/mod.rs`, más `ActividadDisco::no_disponible(ventana_segundos)` y `From<AgregadoActividad> for ActividadDisco`. Ver `contracts/disk-activity.md` §1.1.
- [ ] T006 Sustituir `pub activity_percent: Option<f64>` por `pub activity: ActividadDisco` en `struct DiskSummary` de `src-tauri/src/commands/mod.rs`; actualizar `device_to_summary` (usa `ActividadDisco::no_disponible(cadencia)`) y **todos** los literales/tests de Rust que construyen `DiskSummary` o fijan `activity_percent`.

### Colector: consulta PDH persistente

- [ ] T007 [P] En `src-tauri/src/collectors/perf_counters.rs`: añadir las constantes `PDH_CSTATUS_*` al módulo `ffi`; implementar `ConsultaActividad` (`reconstruir`, `muestrear`, `discos`, `Drop → PdhCloseQuery`) según `data-model.md` §1.5 y `research.md` R1–R2; **quitar** `activity_percent` de `LecturaRendimiento` y dejar de añadir el contador `% Idle Time` en `leer()`; conservar `derivar_activity_percent` y sus pruebas.
- [ ] T008 [P] Añadir el campo `pub actividad: std::sync::Mutex<std::collections::HashMap<String, crate::domain::actividad::AgregadoActividad>>` a `AppState` en `src-tauri/src/persistence/db.rs` e inicializarlo vacío en `AppState::open`.

### Muestreo y publicación

- [ ] T009 Cablear el muestreo en `iniciar_planificador` de `src-tauri/src/commands/mod.rs`: contador de ticks (muestrear cada tick en red, 1 de cada 4 en batería, nunca en pausa — FR-017/FR-018); estado local del hilo `Option<ConsultaActividad>` + `HashMap<String, VentanaActividad>`; reconstruir la consulta si el conjunto de discos monitorizados cambió (refrescado 1×/ciclo de métricas rápidas y tras alta/baja); `muestrear()` → `registrar` por disco, `Err` → no se registra nada; publicar `agregado()` por disco en `AppState.actividad`.
- [ ] T010 En `refresh_metricas_rendimiento` (`src-tauri/src/commands/mod.rs`), fase 3: persistir `metric_sample("activity_percent", media)` **solo si** `AppState.actividad[id].estado == Valido` (FR-010a); eliminar la persistencia de actividad que venía de `persist_perf_reading`/`LecturaRendimiento`; contabilizar un ciclo con todos los muestreos de actividad fallidos hacia `actualizar_source_health(PerformanceCounter, …)` (research.md R7).
- [ ] T010a [P] Prueba de `refresh_metricas_rendimiento` en el `#[cfg(test)]` de `src-tauri/src/commands/mod.rs`: con un `AppState.actividad` cuyo agregado es `Parcial` (o `NoDisponible`), el ciclo **no** escribe fila en `metric_samples` para `activity_percent`; con agregado `Valido`, escribe **una** fila con la media. (FR-010a, FR-021, SC-008)
- [ ] T011 Adaptar `enrich_with_smart_data` y su llamada desde `emitir_metrics_updated` en `src-tauri/src/commands/mod.rs`: rellenar `resumen.activity` desde `AppState.actividad` (pasar el agregado o el mapa ya bloqueado como parámetro) en vez de `latest_device_sample("activity_percent")`.

### Contrato y consumidores (TypeScript) — hasta que compile

- [ ] T012 Regenerar los DTO con `cargo test` y versionar `src/lib/api/generated/ActividadDisco.ts`, `EstadoActividad.ts`, `DiskSummary.ts`, `DeviceDetail.ts`.
- [ ] T013 [P] Escribir la **prueba de rechazo que falla** del objeto `activity` en `src/lib/api/schemas.test.ts`: un número suelto en `activity`, un `estado` fuera del enum y un objeto sin `ventanaSegundos` deben fallar la validación (constitución §XI).
- [ ] T014 En `src/lib/api/schemas.ts`: añadir `estadoActividad` (`z.enum`) y `actividadDisco` (`z.object`), sustituir `diskSummary.activityPercent` por `activity: actividadDisco`, y corregir el fixture `discoValido` de `schemas.test.ts` (hoy `activityPercent: 12`) hasta que T013 pase.
- [ ] T015 [P] Actualizar la interfaz `DiskSummary` en `src/lib/design/types.ts`: `activityPercent: number | null` → `activity: ActividadDisco;` (con su tipo importado o inline).
- [ ] T016 Adaptar el resto de consumidores para que `pnpm check` pase con cero avisos: `src/routes/+page.svelte` (hecho «Actividad» del HeroPanel → `heroDisk.activity.picoPercent`), `src/routes/disks/[id]/+page.svelte`, `src/lib/components/DiskCard.svelte`, y los fixtures `src/lib/stores/app.svelte.test.ts`, `src/lib/components/DiskCard.browser.test.ts`, `src/lib/components/HeroPanel.browser.test.ts` (cableado mínimo al valor de pico; el refinado de estados va en las fases de historia).

**Checkpoint**: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `pnpm check`, `pnpm test` y `pnpm test:component` en verde. Contrato migrado; el motor de ventana existe y se muestrea.

---

## Phase 3: User Story 1 - Ver que un disco está trabajando de verdad, ahora (Priority: P1) 🎯 MVP

**Goal**: el panel general muestra el **pico de la ventana** por disco; sube de forma coherente con
la carga real en menos de una cadencia y es estable entre repeticiones; el arranque no inventa cero.

**Independent Test**: generar carga sostenida sobre un disco durante 60–120 s y comprobar que la
cifra del panel sube a un valor claramente alto en < 30 s, estable al repetir; tras reiniciar, la
actividad se muestra como parcial/no disponible durante los primeros ~30 s, nunca como `0 %`
(quickstart §2.1 y §2.3, SC-001 y SC-003).

- [ ] T017 [P] [US1] Ampliar `src/lib/components/DiskCard.browser.test.ts` con la cobertura de estados de `activity`: `valido` (muestra el pico con `formatPercent`), `no_disponible` (muestra «—» discreto con el texto completo en `title`), `parcial` (muestra el valor con la marca de «midiendo aún», nunca solo por color).
- [ ] T018 [US1] `src/lib/components/DiskCard.svelte`: la magnitud `activity` usa `disk.activity.picoPercent`; `estado === "no_disponible"` → «—» con `title` vía `t()`; `estado === "parcial"` → valor + marca (icono o sufijo textual, §VI); pasar el estado al tooltip (`mostrarTip`).
- [ ] T019 [P] [US1] Claves i18n nuevas en `src/lib/i18n/es.json` y `src/lib/i18n/en.json` (idénticas claves en ambos): marca/etiqueta de «midiendo aún» para el panel, y revisión de `metric.help.activity.body` para hablar de agregado de ventana en vez de instantánea.
- [ ] T020 [US1] `src/lib/design/metricHelp.ts` + `src/lib/design/metricHelp.test.ts`: `veredictoMetrica("activity", …)` acepta la nueva forma (recibe el pico como valor de referencia), añade el caso `parcial` y conserva `unknown` para `no_disponible`; pruebas de los tres casos.
- [ ] T021 [US1] Validación manual de `quickstart.md` §2.1 (carga real, estabilidad al repetir) y §2.3 (arranque sin inventar; hueco en el histórico); registrar el resultado.

**Checkpoint**: US1 funcional y verificable de forma aislada. **Este es el MVP.**

---

## Phase 4: User Story 2 - Distinguir el pico reciente del promedio (Priority: P2)

**Goal**: el detalle de disco muestra **media y pico** de la ventana como dos valores distintos, con
procedencia y antigüedad, y la ayuda contextual explica que es un agregado de los últimos ~30 s.

**Independent Test**: provocar una ráfaga corta (2–3 s) seguida de calma y comprobar en el detalle
que el pico refleja la ráfaga ~30 s mientras la media desciende; la ayuda contextual menciona
media y pico y el periodo (quickstart §2.2, SC-002).

- [ ] T022 [P] [US2] Prueba de componente del `MetricCard` de actividad en `src/lib/components/MetricCard.browser.test.ts` (o el fichero de prueba equivalente del detalle): valor principal = media, secundario = «pico N %»; estados `parcial` y `no_disponible`.
- [ ] T023 [US2] `src/routes/disks/[id]/+page.svelte`: el `MetricCard` de actividad usa `disk.activity.mediaPercent` como valor y muestra `disk.activity.picoPercent` como dato secundario; `ayudaActividad` se deriva de `disk.activity` (media, pico, `ventanaSegundos`).
- [ ] T024 [P] [US2] Claves i18n en `src/lib/i18n/es.json` y `en.json`: `disk.activityMean` / `disk.activityPeak` (o los parámetros `{media}`/`{pico}`/`{ventana}` del veredicto), y `metric.help.activity.verdict.info` reescrito como «media de los últimos {ventana} s, pico {pico} %».
- [ ] T025 [US2] `src/lib/design/metricHelp.ts` + `metricHelp.test.ts`: el veredicto `info` compone el texto con media, pico y `ventanaSegundos`; prueba del texto resultante.
- [ ] T026 [US2] Validación manual de `quickstart.md` §2.2; registrar el resultado.

**Checkpoint**: US1 y US2 funcionan de forma independiente.

---

## Phase 5: User Story 3 - Ver la forma de la actividad reciente de un vistazo (Priority: P3)

**Goal**: en el detalle de disco, una sparkline de la actividad reciente junto a la cifra.

**Decisión de alcance** (`research.md`, incógnitas restantes): se **reutiliza la mini-serie de 24 h
que ya carga** `src/routes/disks/[id]/+page.svelte` (`getMetricSeries("activity_percent", 24h)`,
línea ~203) — que ahora contiene la media de la ventana por ciclo. **No** se añaden las muestras de
la ventana al contrato (evita engordar `ActividadDisco`). Si se decide que la sparkline debe reflejar
la ventana en vivo y no la serie de 24 h, es un lote posterior.

**Independent Test**: con serie de actividad no vacía, la `MetricCard` del detalle dibuja la
sparkline; con serie vacía o parcial, no dibuja nada engañoso (quickstart §2, spec US3).

- [ ] T027 [P] [US3] Prueba de componente: la `MetricCard` de actividad del detalle dibuja la sparkline con una serie de puntos y **no** la dibuja con serie vacía, en `src/lib/components/MetricCard.browser.test.ts`.
- [ ] T028 [US3] `src/routes/disks/[id]/+page.svelte`: confirmar/ajustar que `series={mini["activity_percent"] ?? []}` sigue alimentando la sparkline del `MetricCard` de actividad tras el cambio de origen del valor persistido; sin nuevas llamadas de datos.
- [ ] T029 [US3] Validación manual: abrir el detalle de un disco con historial de actividad y comprobar la sparkline; sin historial, ausencia limpia.

**Checkpoint**: las tres historias funcionan de forma independiente.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: documentación normativa, ADR, puertas de calidad y definición de terminado.

- [ ] T030 [P] `docs/architecture.md` «Colector Windows»: reescribir la viñeta de actividad para describir la consulta PDH persistente y la ventana deslizante (media y pico); mantener la nota de que no se usa `% Disk Time`.
- [ ] T031 [P] `docs/data-model.md` §3, viñeta `activity_percent`: «media de una ventana deslizante del tamaño de la cadencia de métricas rápidas»; nota de que la ventana es memoria no persistida y de la regla de hueco (FR-010a).
- [ ] T032 [P] `docs/ui-contract.md` §3.2 y la nota de `src/lib/design/types.ts` referida en §2: `activity: ActividadDisco` en `DiskSummary`/`DeviceDetail`; describir `ActividadDisco`/`EstadoActividad`.
- [ ] T033 ADR-050 en `docs/decisions.md` (plantilla de la skill `adr`): consulta PDH persistente para la actividad de disco + sustitución de `activityPercent` por `ActividadDisco`. Contexto (instantánea de 1 s/30 s no representativa), decisión, consecuencias (regeneración `ts-rs`, esquema Zod + prueba de rechazo, sin dependencia, sin permiso de Tauri), y enlace a spec 007.
- [ ] T034 Regenerar el consolidado: `pnpm docs:build`; verificar `pnpm docs:check` en verde.
- [ ] T035 [P] Revisar y adaptar fixtures/pruebas de extremo a extremo que referencien `activityPercent` (`grep -rn activityPercent src tests`), incluidas las de `pnpm test:e2e:smoke`.
- [ ] T035a [P] Validación manual de `quickstart.md` §2.5 (batería: el muestreo pasa a ≈4 s y se restaura al enchufar — FR-017) y §2.6 (CPU en reposo no distinguible de la versión anterior — SC-005); registrar el resultado.
- [ ] T036 Definición de terminado de interfaz (`docs/ui-design.md` §8) sobre `DiskCard` y el detalle de disco: tema claro/oscuro, acento propio y del sistema, 1024×560 y 1280×720, escalado 125/150/200 %, estados válido/parcial/no disponible de la actividad, teclado y foco en el `MetricCard`, textos en los dos idiomas (quickstart §3).
- [ ] T037 Puertas completas: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `pnpm check`, `pnpm test`, `pnpm test:component`, `pnpm test:e2e:smoke`, `pnpm test:a11y`, `pnpm lint`, `pnpm verify`.
- [ ] T038 Cierre con la skill `cierre-tarea`: matriz de documentación y las nueve puertas de verificación; confirmar que todo cambio observable está documentado antes de proponer commit.

---

## Estado (2026-09-09)

**Implementación y puertas automáticas: completas.** `cargo test` (680), `cargo clippy -D warnings`,
`cargo fmt --check`, `pnpm check` (0/0), `pnpm test` (326), `pnpm test:component` (138),
`pnpm test:e2e:smoke` (7), `pnpm test:a11y` (18), `pnpm lint`, `pnpm verify`, `pnpm docs:check` —
todo en verde.

**Pendiente: validación manual con la aplicación real** (necesita Windows elevado, carga de disco
real e inspección visual — no ejecutable desde este entorno):

- **T021** — quickstart §2.1 (el pico del panel sube con carga real, estable al repetir) y §2.3
  (arranque: `parcial`/`no disponible` ~30 s, nunca `0 %`; hueco en el histórico).
- **T026** — quickstart §2.2 (ráfaga corta: el pico del detalle la refleja mientras la media baja).
- **T029** — sparkline de actividad del detalle con y sin historial.
- **T035a** — quickstart §2.5 (batería → muestreo ≈4 s) y §2.6 (CPU en reposo sin cambio medible).
- **T036** — definición de terminado de interfaz (`ui-design.md` §8): temas, acento, tamaños,
  escalado, estados de la actividad, teclado, dos idiomas, sobre `DiskCard` y el detalle de disco.
- **T038** — `cierre-tarea` y propuesta de commit (pendiente de autorización de la persona).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias. T001 antes de cualquier código que use los números.
- **Foundational (Phase 2)**: depende de Phase 1. **Bloquea las tres historias.**
- **US1 (Phase 3)**: depende de Phase 2. Es el MVP.
- **US2 (Phase 4)**: depende de Phase 2. Independiente de US1 (toca el detalle, no el panel).
- **US3 (Phase 5)**: depende de Phase 2 (y de que la serie `activity_percent` ya se persista como media, T010).
- **Polish (Phase 6)**: depende de las historias que se vayan a entregar. T033 (ADR) y T030–T032 (docs normativos) pueden ir en cuanto Phase 2 esté cerrada.

### Within Phase 2 (orden)

- T003 → T004 (test-first del dominio).
- T004 → T005 → T006 (el DTO usa `AgregadoActividad`; `DiskSummary` usa el DTO).
- T007, T008 en paralelo con T005/T006 (ficheros distintos).
- T004 + T007 + T008 → T009 (el muestreo necesita ventana, consulta y estado).
- T009 → T010, T011. T010 → T010a (la prueba sigue a la implementación del recorte de persistencia).
- T006 → T012 (regenerar contrato) → T013 → T014.
- T012 → T015, T016.
- T013 → T014 (la prueba de rechazo antes del esquema).

### Within each User Story

- Prueba de estados (T017 / T022 / T027) antes de la implementación de su pantalla.
- i18n en paralelo con la lógica de presentación (fichero distinto).
- Validación manual al final de la fase.

### Parallel Opportunities

- **Phase 1**: T002 ∥ T001.
- **Phase 2**: {T003}, {T007}, {T008} pueden empezar a la vez; {T013} ∥ {T015}; {T005/T006} en su hilo.
- **Phase 3**: T017 ∥ T019; luego T018, T020.
- **Phase 4**: T022 ∥ T024.
- **Phase 6**: T030 ∥ T031 ∥ T032 ∥ T035.

---

## Parallel Example: Phase 2 arranque

```
# A la vez, ficheros distintos:
T003  Pruebas que fallan de VentanaActividad        (src-tauri/src/domain/actividad.rs)
T007  ConsultaActividad + quitar activity de leer()  (src-tauri/src/collectors/perf_counters.rs)
T008  AppState.actividad                             (src-tauri/src/persistence/db.rs)
```

---

## Implementation Strategy

### MVP (recomendado)

1. Phase 1: Setup (T001–T002).
2. Phase 2: Foundational completa (T003–T016) → checkpoint en verde.
3. Phase 3: US1 (T017–T021).
4. **PARAR y VALIDAR**: quickstart §2.1 y §2.3. Es una entrega útil por sí sola: la actividad del
   panel deja de mentir.
5. Cerrar el mínimo de Polish imprescindible para commit: T033 (ADR-050), T030–T032, T034, T037, T038.

### Entrega incremental

- MVP (US1) → validar → commit.
- US2 (detalle con media y pico) → validar → commit.
- US3 (sparkline, casi gratis al reusar la mini-serie) → validar → commit.

### Notas

- `[P]` = ficheros distintos, sin dependencia.
- Verificar que las pruebas de T003 y T013 **fallan** antes de implementar (constitución §VIII/§XI).
- El hook `proteger-rutas.mjs` pedirá confirmación al editar `docs/decisions.md` y los contratos
  (T033, T005/T006/T012, T032): es esperado, se autoriza en ese momento.
- Commit por tarea o grupo lógico; el commit final lo pide la persona (`AGENTS.md`, límites duros).
