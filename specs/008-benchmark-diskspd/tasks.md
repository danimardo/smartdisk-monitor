---
description: "Task list — Benchmark de disco con DiskSpd"
---

# Tasks: Benchmark de disco con DiskSpd (perfiles estilo CrystalDiskMark)

**Input**: `specs/008-benchmark-diskspd/` — plan.md, spec.md, research.md, data-model.md,
contracts/pruebas-benchmark.md, quickstart.md

**Tests**: incluidos donde la constitución §VIII los exige (parsers, lógica pura «área donde el
error es silencioso»: `diskspd_xml`, construcción de comandos, cálculo de `-d`) y donde ya hay
patrón (esquema Zod → prueba de rechazo, estados de componente). No se generan tests de la
orquestación real (lanzar DiskSpd): se valida en `quickstart.md`, como el resto de E/S real.

**Organización**: por historia de usuario. US1 y US2 son ambas **P1** y comparten el hilo de
orquestación; US2 endurece lo que US1 construye.

## Format: `[ID] [P?] [Story] Descripción con ruta de fichero`

- **[P]**: paralelizable (fichero distinto, sin dependencia de una tarea incompleta)
- **[Story]**: [US1] / [US2] / [US3]; Setup / Foundational / Polish sin etiqueta

---

## Phase 1: Setup — recurso redistribuido y decisión

**Propósito**: dejar `diskspd.exe` disponible, empaquetado y verificable, y su decisión escrita.

- [x] T001 **[MANUAL — DUEÑO]** Descargar DiskSpd de `https://aka.ms/getdiskspd`, extraer
  `amd64/diskspd.exe`, verificar que es un PE de máquina `0x8664` (AMD64), y colocar en el repo:
  `third-party/diskspd/bin/diskspd.exe`, `third-party/diskspd/licenses/LICENSE.txt` (MIT íntegra).
  Aportar al agente el **SHA-256** y la **versión** exacta (p. ej. «DiskSpd 2.2»). El hook
  `proteger-rutas.mjs` impide al agente escribir en `third-party/`.
- [ ] T002 [P] Crear `third-party/diskspd/README.md` (espejo de
  `third-party/smartmontools/README.md`): versión, origen, qué se incluye (`bin/diskspd.exe` amd64,
  licencia), qué se deja fuera (arm64, x86, docs), y la nota de que la MIT **no** impone obligación
  de fuente.
- [x] T003 Añadir a `docs/decisions.md` el **ADR-053 — «DiskSpd como motor del benchmark de disco»**:
  problema (el motor propio da cifras que no comparan con CrystalDiskMark), decisión (redistribuir
  DiskSpd como proceso externo, MIT, solo amd64), alternativas descartadas (motor propio con
  overlapped I/O; extensión modesta; empaquetar CDM), consecuencias (binario nuevo ~200 KB,
  `THIRD_PARTY_NOTICES`, `verify:assets`, se elimina el motor propio y la verificación byte a byte).
- [x] T004 Añadir la entrada de `bin/diskspd.exe` a `scripts/verify-assets.mjs` con su SHA-256
  (de T001) y su `why`.
- [x] T005 Añadir la sección **DiskSpd** a `THIRD_PARTY_NOTICES.md`: MIT, Microsoft, versión, hash,
  «distribution status: bundled», y la nota de que la obligación 3(a) de la GPL **no aplica**.
- [x] T006 En `src-tauri/tauri.conf.json` → `bundle.resources`: añadir
  `"../third-party/diskspd/bin/diskspd.exe": "bin/diskspd.exe"` y
  `"../third-party/diskspd/licenses/LICENSE.txt": "licenses/diskspd/LICENSE.txt"`. **Requiere modo
  plan** (ruta protegida por CLAUDE.md).
- [x] T007 [P] `pnpm verify:assets` en verde con el binario colocado (comprueba T001+T004).

**Checkpoint**: `diskspd.exe` presente, empaquetado, verificado y documentado.

---

## Phase 2: Foundational — quitar el motor viejo y construir las piezas puras

**⚠️ Bloquea todas las historias.**

### Eliminar el motor propio

- [x] T008 Borrar `src-tauri/src/tests/benchmark.rs` (motor, `BufferAlineado`, `orden_de_bloques`,
  `calcular_*` y sus tests).
- [x] T009 Borrar `src-tauri/src/tests/patron.rs` (patrón comprobable y sus tests).
- [x] T010 En `src-tauri/src/tests/mod.rs`: quitar `mod benchmark;` y `mod patron;`; añadir
  `mod diskspd;` y `mod diskspd_xml;`.
- [x] T011 En `src-tauri/src/commands/mod.rs`: quitar del `start_benchmark` actual toda referencia a
  `tests::benchmark::*`, `tests::patron::*`, `ParametrosBenchmark`, `ModoAcceso`, `RazonParada`
  (dejar el comando compilando aunque temporalmente devuelva un error `not_implemented` — se rehace
  en la Fase 3).

### Resolución del binario

- [x] T012 [P] Crear `src-tauri/src/platform/diskspd.rs` con `resolve_diskspd_path() -> PathBuf`,
  calcado de `collectors::smartctl::resolve_smartctl_path()` (dev:
  `../third-party/diskspd/bin/diskspd.exe`; prod: `bin/diskspd.exe` junto al ejecutable), con el
  mismo comentario de «sin verificar contra una compilación empaquetada real». Registrar el módulo
  en `src-tauri/src/platform/mod.rs`.

### Valores adoptados y tipos de dominio (puros)

- [x] T013a Registrar en `docs/open-questions.md` una entrada nueva con los valores propuestos en
  `research.md` D3, **antes de escribir código con ellos** (constitución, flujo §1): `D_OBJETIVO`
  (≈5 s), `D_MIN` (≈2 s), `D_CALENTAMIENTO` (≈2 s, `-W`), `TOPE_DATOS` (≈4 GiB por perfil de
  escritura), `TAMANO_ARCHIVO` (1 GiB), y la lista de los 4 perfiles con su `-b`/`-o`. Incluir la
  nota de D3 (tensión tope-de-datos vs. suelo de medición en discos muy rápidos) y que el resultado
  etiqueta cuándo el tope recorta la medición.
- [x] T013 [P] En `src-tauri/src/tests/diskspd.rs`: `Acceso`, `Sentido`, `Perfil` y la constante
  `PERFILES: [Perfil; 4]` (`seq1m_q8`, `seq1m_q1`, `rnd4k_q32`, `rnd4k_q1`) según `data-model.md`
  §1.1. Constantes `D_OBJETIVO`, `D_MIN`, `D_CALENTAMIENTO`, `TOPE_DATOS_BYTES`,
  `TAMANO_ARCHIVO_BYTES` **con los valores de T013a**.
- [x] T014 **Test-first** en `src-tauri/src/tests/diskspd.rs` (mod tests): `Medicion::args`
  produce la línea de DiskSpd correcta para cada perfil × sentido (bloque, `-o`, `-r4K`/`-s`, `-w0`/
  `-w100`, `-Sh`, `-Z1M`, `-L`, `-Rxml`, `-c`, `-d`, `-W`, ruta) — casos de los 4 perfiles.
- [x] T015 Implementar `Medicion::args` hasta que T014 pase.
- [x] T016 **Test-first**: `Medicion::duracion_efectiva(caudal_ref)` según D3 —
  `clamp(TOPE/caudal, D_MIN, D_OBJETIVO)`; `caudal` que agota el tope antes del suelo →
  `(D_MIN, true)`; lectura (sin tope) → `(D_OBJETIVO, false)`; `caudal` ausente → `(D_OBJETIVO, false)`.
- [x] T017 Implementar `duracion_efectiva` hasta que T016 pase.

### Parseo del XML de DiskSpd (puro, test-first — §VIII)

- [x] T018 [P] **[MANUAL — DUEÑO o agente con la app]** Capturar fixtures de `-Rxml` reales
  ejecutando el `diskspd.exe` redistribuido: una salida por perfil/sentido representativo y una de
  una ejecución abortada / con error. Guardar en `src-tauri/src/tests/fixtures/diskspd/`.
- [x] T019 **Test-first** en `src-tauri/src/tests/diskspd_xml.rs`: parsear cada fixture
  de T018 a `SalidaDiskspd { test_time_s, bytes, ops, avg_latency_ms, tool_version }`; una salida sin
  esos campos o irrecorrible → `Err(SalidaIlegible)`. Nunca ceros.
- [x] T020 Implementar `tests::diskspd_xml` a mano (helpers `extraer_*` estilo
  `collectors/event_log.rs`, struct explícito — **no** `serde_json::Value`) hasta que T019 pase.

### Contrato (Rust → ts-rs → Zod)

- [x] T021 En `src-tauri/src/tests/` (o `domain/tipos.rs` según dónde vivan los DTO de
  pruebas): structs `BenchmarkResultWire`, `BenchmarkRowWire`, y ampliar `ResumenPruebaJson` /
  `TestResult` según `contracts/pruebas-benchmark.md` (quita los campos planos del benchmark, añade
  `benchmark: Option<BenchmarkResultWire>`). Anotar con `#[derive(TS)]` / `#[ts(export)]`.
- [x] T022 `cargo test` regenera `src/lib/api/generated/*.ts`; confirmar el diff y
  versionarlo.
- [x] T023 En `src/lib/api/schemas.ts`: adaptar el esquema Zod de `TestResult` /
  `TestRun` a la forma nueva (`benchmark` con `rows`/`notRun`), inferido con `z.infer`.
- [x] T024 [P] En `src/lib/api/schemas.test.ts`: **prueba de rechazo** — `BenchmarkRow`
  sin `mbPerSecond`, `profile` fuera del enum, `direction` inválido → el parseo falla
  (`ipc.schema_mismatch`).

**Checkpoint**: motor viejo fuera; piezas puras (perfiles, comandos, `-d`, parser) construidas y
probadas; contrato nuevo tipado y validado.

---

## Phase 3: User Story 1 — Medir el rendimiento con cifras reconocibles (P1) 🎯 MVP

**Goal**: lanzar la prueba y obtener una tabla con MB/s, IOPS y latencia por perfil y sentido,
del orden de las de CrystalDiskMark.

**Independent Test**: lanzar sobre un volumen real, dejar terminar, comprobar que la tabla trae 8
filas con cifras plausibles y «Medido con DiskSpd `<versión>`».

- [x] T025 [US1] En `src-tauri/src/tests/diskspd.rs`: `orquestar_matriz(...)` — función que recibe
  la ruta del archivo, un `cancelado: impl FnMut() -> bool`, un `leer_temperatura: impl FnMut() ->
  Option<f64>`, un `limite_termico`, y un `on_progreso: impl FnMut(clave_perfil, sentido, hechas,
  total)`; recorre `PERFILES` (lectura antes que escritura), llama a `platform::proceso_externo::
  ejecutar_con_limite` una vez por medición con `Medicion::args`, parsea el `-Rxml` con
  `tests::diskspd_xml`, y devuelve `Vec<FilaResultado>` + `notRun` + `RazonParada`. La `-d` de cada
  escritura sale de `duracion_efectiva` con el caudal de la lectura del mismo perfil.
- [x] T026 [US1] En `src-tauri/src/commands/mod.rs`: reescribir `start_benchmark` — firma
  `{ volume_id }` (sin `size_bytes`/`block_size_bytes`/`mode`/`passes`); valida volumen, letra,
  `hay_prueba_activa`, `resolver_tamano_bytes` contra `TAMANO_ARCHIVO_BYTES`; comprueba que
  `resolve_diskspd_path()` existe (→ `test.tool_missing`); crea la carpeta y resuelve la ruta del
  archivo (`tests::rutas`, `confirmar_no_sobrescribe`); crea la fila `TestRun` con `command` =
  descripción de la matriz y `parameters` = `{ tool, toolVersion, fileSizeBytes, profiles }`.
- [x] T027 [US1] En el hilo de ejecución de `start_benchmark`: llamar a `orquestar_matriz`, mapear
  `Vec<FilaResultado>` → `BenchmarkResultWire`, borrar el archivo temporal (siempre), y
  `finish_test_run` con el resumen. Emitir `test:progress` con `progressPercent = hechas*100/8` y la
  clave del perfil/sentido en curso.
- [x] T028 [P] [US1] En `src/lib/i18n/es.json` y `en.json`:
  - **Renombrar la prueba** (A1 → B): `tests.benchmark`, `tests.cards.benchmark.title`,
    `tests.confirm.benchmark.title`, `tests.type.benchmark` de «Lectura y escritura» / «Lectura/
    escritura» → **«Rendimiento»** (en: «Performance»). Reescribir `tests.cards.benchmark.desc`
    (ya no «verifica el patrón» — ahora «mide el caudal, los IOPS y la latencia con perfiles tipo
    CrystalDiskMark») y `tests.confirm.benchmark.body` / `.impact` (ya no «1 GiB en bloques de 1 MiB
    secuencial»; ahora la matriz + los GB que se escribirán).
  - **Claves nuevas**: nombres visibles de los 4 perfiles, encabezados de la tabla (Perfil,
    Lectura/Escritura, MB/s, IOPS, Latencia), «Medido con {tool} {version}», «tope de datos
    alcanzado».
- [x] T029 [US1] En `src/routes/tests/+page.svelte`: `startBenchmark({ volumeId })` sin los otros
  parámetros; quitar `BENCHMARK_TAMANO_BYTES`/`BENCHMARK_BLOQUE_BYTES`/`mode`/`passes`.
- [x] T030 [US1] Tabla de resultados del benchmark en `src/routes/tests/+page.svelte` (o un
  `BenchmarkResults.svelte` si no compone limpio con `Card` + `DataRow`): 8 filas
  (perfil × sentido) con MB/s, IOPS, latencia; encabezados de columna reales; pie «Medido con
  DiskSpd {version}»; filas `notRun` mostradas como «no ejecutado», nunca 0.
- [x] T031 [P] [US1] `src/lib/components/*.browser.test.ts` (o el de la tabla): estado
  **completado** (8 filas con cifras) y estado **con `notRun`** (parada anticipada: filas
  parciales + «no ejecutado»). Claro y oscuro, 1024×560.
- [x] T032 [P] [US1] `docs/ui-contract.md` §3.6: reemplazar por `contracts/pruebas-benchmark.md`
  (comando sin parámetros, `TestResult` con `benchmark`, tabla de errores nueva).
- [x] T033 [P] [US1] `docs/data-model.md` §3 (test_runs): documentar la forma nueva de
  `result_summary_json` para un benchmark (`BenchmarkResult`); sigue sin ser tabla nueva.

**Checkpoint**: se puede lanzar el benchmark y ver la tabla. Guardas mínimas (espacio, herramienta
ausente) ya funcionan; térmica y cancelación se endurecen en US2.

---

## Phase 4: User Story 2 — Que la prueba no dañe ni bloquee el equipo (P1)

**Goal**: reserva de espacio, guardia térmica que mata DiskSpd, cancelación que mata DiskSpd,
exclusión con el autotest SMART, borrado del archivo, aviso previo con GB a escribir.

**Independent Test**: forzar cada condición de parada y comprobar razón correcta + resultados
parciales conservados + sin archivo huérfano.

- [x] T034 [US2] En `orquestar_matriz` (`tests/diskspd.rs`): entre mediciones **y** durante cada
  invocación de DiskSpd, sondear `leer_temperatura` cada ~2 s; si `tests::guardia::
  debe_detenerse_por_temperatura` → **matar el proceso DiskSpd en curso**, devolver `RazonParada::
  Thermal` con las filas ya medidas. (El `ejecutar_con_limite` actual mata al agotar el límite;
  añadir una vía de muerte anticipada por bandera, o lanzar DiskSpd con un `Child` propio vigilado.)
- [x] T035 [US2] Misma vía para **cancelación**: `cancelado()` → matar el proceso DiskSpd → E/S
  cesa en ≤ 3 s (SC-003), `RazonParada::Cancelled` con filas parciales. Reutiliza
  `state.test_cancel_flags` de `AppState` (ya existe para el motor viejo).
- [x] T036 [US2] En `start_benchmark`: `resolver_tamano_bytes(TAMANO_ARCHIVO_BYTES, free, capacity)`
  → `test.insufficient_space` si no cabe tras la reserva (2 GiB o 5 %). Confirmar que la exclusión
  `hay_prueba_activa(..., Some(volume_id))` sigue cubriendo autotest SMART del mismo disco físico
  (J.29, sin cambios).
- [x] T037 [US2] Borrado del archivo temporal en **todas** las salidas (completada, cancelada,
  térmica, error de DiskSpd, XML ilegible); si falla, `temp_path`/`orphanPath` conserva la ruta
  (reutiliza el patrón del hilo viejo).
- [x] T038 [US2] Cálculo del **worst-case de bytes a escribir** de la matriz (2 perfiles de
  escritura, `D_OBJETIVO`/`D_MIN` × un caudal estimado + calentamientos) → exponerlo para el
  `ConfirmDialog`. En `src/routes/tests/+page.svelte`: el aviso previo muestra «se escribirán hasta
  ~N GB en el SSD» y cancelar aquí no escribe nada (SC-007).
- [x] T039 [P] [US2] `src/lib/i18n/{es,en}.json`: claves del aviso de datos a escribir, de la parada
  térmica en el resultado, y de los errores nuevos (`test.tool_missing`, `test.tool_output_unreadable`).
- [x] T040 [P] [US2] `docs/ui-contract.md` §1 (tabla de códigos de error): añadir
  `test.tool_missing`, `test.tool_output_unreadable`; nota de que `stoppedReason: "space"` ya no
  aparece en ejecución (solo rechazo previo).
- [x] T041 [P] [US2] `e2e/ui/tests.spec.ts`: ajustar el test del benchmark — `start_benchmark` se
  llama con `{ volumeId }`; el `ConfirmDialog` muestra el aviso de GB a escribir; un evento
  `test:progress` con resultado de benchmark pinta la tabla. **Caso SC-005**: con `start_benchmark`
  que rechaza (`{ __rechazar__: { code: "test.tool_missing" } }`), la pantalla de pruebas sigue
  ofreciendo chkdsk y autotest SMART. Actualizar `e2e/ui/fixtures/respuestas.ts` (`testRunActivo` /
  un `TestRun` de benchmark terminado con `rows`).

**Checkpoint**: las 6 guardias del quickstart §2.3 pasan.

---

## Phase 5: User Story 3 — Consultar y comparar resultados anteriores (P2)

**Goal**: cada ejecución (completada, cancelada, térmica) en el historial con su resumen y abrible.

**Independent Test**: dos ejecuciones sobre volúmenes distintos → ambas en el historial con estado
y cifras.

- [x] T042 [US3] En `src/routes/tests/+page.svelte` (historial): que una fila de benchmark muestre
  fecha, volumen, estado y un **resumen de cifras** (p. ej. el MB/s secuencial de lectura y el IOPS
  aleatorio), y al abrirla se vea la tabla completa (reutiliza el componente de T030).
- [x] T043 [US3] Distinguir en el historial una ejecución **completada** de una **detenida por
  temperatura / cancelada** (icono/etiqueta), mostrando sus filas parciales.
- [x] T044 [P] [US3] `src/lib/components/*.browser.test.ts`: la fila de historial de un benchmark
  parcial se distingue de uno completo.

**Checkpoint**: las tres historias funcionan de forma independiente.

---

## Phase 6: Polish & Cross-Cutting

- [x] T045 [P] Reescribir `docs/product-specification.md` §6 «Prueba de lectura y escritura»:
  motor DiskSpd, 4 perfiles, matriz lectura+escritura, tabla MB/s/IOPS/latencia, acotado
  tiempo+tope, sin verificación de integridad.
- [x] T046 [P] `docs/open-questions.md`: revisar que la entrada de T013a sigue coincidiendo con lo
  implementado (`-d` real, tope, tamaño); anular la parte de J.29 que hablaba del motor propio
  (`RazonParada::Space` no alcanzable en ejecución) marcándola reemplazada.
- [x] T047 [P] `docs/roadmap.md`: la línea «Benchmark de archivo temporal» de v0.4 → nota de que
  v0.1.5 lo sustituye por DiskSpd (ADR-053).
- [x] T048 [P] `docs/known-issues.md`: si el hilo de ejecución necesita algún `svelte-ignore` o
  silencio, su entrada; si no, nada.
- [x] T049 `pnpm docs:build` → `historias.md` al día; `pnpm docs:check` en verde.
- [ ] T050 [P] Regenerar la captura de la pantalla de pruebas (`pnpm docs:screenshots`). Pendiente:
  `docs:screenshots` movió las 14 capturas (ruido de entorno) y se revirtieron; hay que regenerarlas
  a propósito con un fixture de benchmark terminado y revisarlas a ojo.
- [x] T051 Ejecutar **todas** las puertas: `pnpm check` (0/0), `lint`, `verify`, `test`,
  `test:component`, `build`, `docs:check`; y desde `src-tauri/`: `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- [ ] T052 Ejecutar la validación manual de `quickstart.md` §2 (§3 «empaquetado» queda para la
  puerta «por cada versión publicada»). Requiere `pnpm app:dev` y un volumen de pruebas real.
- [x] T053 Pasar la skill `cierre-tarea` (matriz de documentación + nueve puertas) y proponer el
  commit. Subir la versión a **0.1.5** (`package.json` + `Cargo.toml` + `Cargo.lock`).

---

## Phase 7: Presentación estilo CrystalDiskMark (incremento posterior, decidido con el dueño)

Tras la primera implementación, el dueño pidió que la tabla de resultados se viera como la rejilla
de CrystalDiskMark (misma disposición, colores del tema), con tooltips por campo, y que se rellenara
celda a celda durante la ejecución. Recogido en `spec.md` FR-002a / FR-009 y en el contrato.

- [x] T054 Backend: `orquestar_matriz` publica las filas ya medidas en cada avance
  (`on_avance(filas, version, hechas)`); `repo_varios::update_test_run_progress_summary` (nuevo)
  persiste el `BenchmarkResult` parcial en la fila `running`; `start_benchmark` arranca la fila con
  una rejilla vacía. `test:progress` —que ya lleva `result`— la transporta. Tests de `on_avance`.
- [x] T055 `BenchmarkResults.svelte` reescrito como rejilla 4×2 estilo CDM: `<table>` con
  `<th scope>`, cifra grande + barra proporcional al máximo, `SegmentedControl` MB/s ↔ IOPS
  (recordado en `localStorage`), `Tooltip` en encabezados y perfiles, relleno celda a celda con
  `running`, `formatBenchLatency` (µs/ms). Claves i18n de tooltips y notación `SEQ1M Q8T1`.
- [x] T056 Pruebas de componente (rejilla completa, en curso, parada anticipada, toggle, tope) y
  e2e (`tests.spec.ts`) adaptadas. Docs: `spec.md`, `contracts/`, `ui-contract.md` §3.6,
  `ui-design.md` §3, `product-specification.md` §6.

---

## Dependencies & Execution Order

- **Phase 1 (Setup)**: T001 (dueño) desbloquea T004/T005/T007. T002/T003/T006 en paralelo con T001.
- **Phase 2 (Foundational)**: depende de T010 (mod.rs) para compilar. T008–T011 en serie corta.
  **T013a (registrar valores en `open-questions.md`) va antes de T013** (constitución, flujo §1).
  T012, T013 en paralelo tras T013a. T014→T015, T016→T017, T019→T020 son pares test→impl. T018
  (fixtures) bloquea T019. T021→T022→T023→T024 en serie (contrato). **Bloquea US1/US2/US3.**
- **Phase 3 (US1)**: T025 (orquestación) es el núcleo; T026→T027 lo usan. T028/T031/T032/T033 en
  paralelo. T029→T030.
- **Phase 4 (US2)**: T034/T035 modifican `orquestar_matriz` de T025 → **después de US1**. T036–T038
  tocan `start_benchmark`/UI. T039/T040/T041 en paralelo.
- **Phase 5 (US3)**: depende de T030 (componente de tabla). T042→T043; T044 en paralelo.
- **Phase 6 (Polish)**: T045–T048 en paralelo; T049 tras ellos; T051 al final; T052/T053 los últimos.

### Historias independientes

- **US1** es el MVP: entregable en cuanto pasan Phase 1+2+3 (con las guardas mínimas de espacio y
  herramienta ausente). Se puede demostrar sin US2/US3.
- **US2** endurece US1 (no lo reemplaza): la prueba ya corre; US2 añade que se pare bien.
- **US3** es puramente de presentación sobre datos que US1 ya persiste.

## Parallel Example: Foundational

```
# En paralelo tras T013a:
T012  platform/diskspd.rs (ruta del binario)
T013  tests/diskspd.rs (Perfil, PERFILES, constantes con los valores de T013a)
T018  fixtures de -Rxml (dueño / agente con la app)
```

## Implementation Strategy

1. **Setup + Foundational** → base lista, motor viejo fuera, piezas puras probadas.
2. **US1** → lanzar el benchmark y ver la tabla. **PARAR y VALIDAR** (quickstart §2.1, §2.2).
3. **US2** → endurecer guardias. Validar quickstart §2.3.
4. **US3** → historial rico.
5. **Polish** → docs, versión 0.1.5, `cierre-tarea`, commit.

## Notes

- El binario y los fixtures de XML **los aporta el dueño** (T001, T018) — el agente no descarga ni
  coloca binarios de terceros, y el hook bloquea `third-party/`.
- T006 y cualquier edición de `src-tauri/tauri.conf.json` exige **modo plan** (CLAUDE.md).
- Cada `cargo test` regenera `src/lib/api/generated/*.ts`: versionar el diff es parte de T022.
- No se commitea sin pedirlo (T053).
