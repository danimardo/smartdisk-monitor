---
description: "Task list — Informe HTML por disco (contenido útil + resumen con IA)"
---

# Tasks: Informe HTML por disco (contenido útil + resumen con IA)

**Input**: `specs/009-informe-mejorado/` — plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: incluidos. Este repositorio prueba cada módulo (constitución §VIII) y la anonimización
es crítica de privacidad (principio XVI); las tareas de prueba van **antes** de su implementación
en cada historia.

**Organización**: por historia de usuario, para poder entregar e incrementar de forma independiente.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: se puede hacer en paralelo (ficheros distintos, sin dependencias pendientes)
- **[Story]**: `[US1]`/`[US2]`/`[US3]` para las fases de historia

## Convenciones de ruta

- Backend Rust: `src-tauri/src/`
- Frontend Svelte/TS: `src/`
- Pruebas de interfaz: `e2e/ui/`
- Documentación normativa: `docs/`, decisiones en `docs/decisions.md`, spec en `specs/009-informe-mejorado/`

---

## Phase 1: Setup (infraestructura compartida)

**Propósito**: preparar lo mecánico y transversal antes de tocar la generación.

- [X] T001 [P] Añadir el esqueleto de **ADR-057** («El informe imprimible gana un resumen por disco con IA») en `docs/decisions.md` con estado «propuesta», recogiendo la decisión y las alternativas de `specs/009-informe-mejorado/research.md`
- [X] T002 [P] Separar la versión de esquema del HTML de la de CSV/JSON en `src-tauri/src/reporting/export.rs`: nueva constante `SCHEMA_VERSION_HTML = 2`; `SCHEMA_VERSION` (CSV/JSON) **sin tocar**
- [ ] T003 [P] Reservar las claves i18n de la feature en `src/lib/i18n/es.json` y `src/lib/i18n/en.json` (encabezados de sección del informe, marca «resumen IA», errores `report.*`, textos de progreso y del modal de vista previa) con valores provisionales; los afina cada tarea posterior

**Checkpoint**: base mecánica lista.

---

## Phase 2: Foundational (prerrequisitos que bloquean las historias)

**Propósito**: helpers de recogida de datos que la Historia 1 necesita y la Historia 2 reutiliza.

**⚠️ Ninguna historia puede empezar hasta cerrar esta fase.**

- [X] T004 [P] Prueba unit de `reporting/resumen_metricas.rs`: serie vacía → todo `None`; serie con huecos; agregados de 5 min → `resolucion` correcta — en `src-tauri/src/reporting/resumen_metricas.rs` (`#[cfg(test)]`)
- [X] T005 [P] Implementar `reporting/resumen_metricas.rs`: `ResumenMetrico { minimo, media, maximo, pico, muestras, resolucion }` a partir de una serie/agregados en `[desde, hasta]` (reusa `repo_metricas`/`repo_agregados` y la cascada de resolución de `open-questions.md` E.1) — `src-tauri/src/reporting/resumen_metricas.rs`
- [X] T006 [P] Prueba unit de `reporting/minigrafica.rs`: sin datos → SVG de «sin muestras» (o `None`); una serie con un hueco → dos `<polyline>`; el SVG **no** contiene `http` ni `<script` — `src-tauri/src/reporting/minigrafica.rs` (`#[cfg(test)]`)
- [X] T007 [P] Implementar `reporting/minigrafica.rs`: SVG embebido de una serie — `domain::series::completar_serie` rellena huecos con `null`, se emite un `<polyline>` por tramo continuo y se **rompe** en cada `null`; eje Y con 2-3 marcas; tamaño fijo (p. ej. 480×120); trazo gris oscuro fijo, legible en gris; pie con la resolución — `src-tauri/src/reporting/minigrafica.rs`
- [X] T008 Prueba unit de `frescura(...)` en `src-tauri/src/reporting/informe.rs` (`#[cfg(test)]`): en el umbral y a ambos lados (`Actual` / `Obsoleta{hace}` / `Nunca`)
- [X] T009 Implementar el helper `frescura(ultima_muestra, inicio_intervalo, cadencia_esperada) -> Frescura` en `src-tauri/src/reporting/informe.rs` (`Obsoleta` si la última muestra es anterior al intervalo o más vieja que 3× la cadencia; `Nunca` si no hay muestra)
- [X] T010 Prueba unit de `contadores_con_delta(...)` en `src-tauri/src/reporting/informe.rs` (`#[cfg(test)]`): delta numérico con línea base; `SinReferencia` sin muestra previa; excluye `smart_query_ok` y `vendor_temp_limit_celsius`
- [X] T011 Implementar `contadores_con_delta(conn, device_id, desde, hasta) -> Vec<ContadorConDelta>` en `src-tauri/src/reporting/informe.rs` (mismo filtro que el panel «Contadores» del detalle de disco; delta = último valor en el intervalo − último en o antes de `desde`)
- [X] T012 Prueba unit de `eventos_de_dispositivo_en_rango(...)` en `src-tauri/src/persistence/repo_varios.rs` (`#[cfg(test)]`): respeta `[desde, hasta]`, tope 50, devuelve el nº de omitidos, incluye el mensaje
- [X] T013 Implementar `eventos_de_dispositivo_en_rango(conn, device_id, desde, hasta, limite) -> (Vec<SystemEvent>, u32)` en `src-tauri/src/persistence/repo_varios.rs`
- [X] T014 [P] Añadir a `src-tauri/src/reporting/anonimizar.rs` el método `con_etiqueta_volumen(&str)` + constante `CAMPO_ETIQUETA_VOLUMEN` (clave i18n `diagnostic.redacted.volumeLabel`), con su prueba unit (marcador `<VOLUMEN-1>`, consistente, ordenado por longitud, cadena vacía no genera sustitución)

**Checkpoint**: helpers de datos y anonimización listos; las historias pueden empezar.

---

## Phase 3: User Story 1 — Informe legible por disco, sin IA (Priority: P1) 🎯 MVP

**Goal**: por cada disco, una sección con identidad, salud «a fecha de hoy» (con nota de
antigüedad), contadores SMART con delta, alertas con frase legible, eventos de Windows, y dos
mini-gráficas SVG. Autónomo, se abre sin conexión. CSV/JSON no cambian.

**Independent Test**: exportar el HTML de un `conn` con datos de varios discos → cada sección
completa; la salida no contiene ninguna URL externa; una alerta de evento sin objeto de disco no
aparece.

### Pruebas (van antes)

- [X] T015 [P] [US1] Prueba unit `reporting::informe` en `src-tauri/src/reporting/informe.rs` (`#[cfg(test)]`): una sección de disco contiene salud actual, ≥1 contador con delta, la alerta pintada con el texto de `alertLabels` (no `smart.error_log`), eventos, dos `<svg>`; y `assert!` de que la salida no contiene `http://`, `https://`, `<script src`, `<link ` ni `<img src="http`
- [X] T016 [P] [US1] Prueba unit: una alerta con `target_device_id == None` **no** aparece en ninguna sección; una con objeto de disco aparece en la suya — `src-tauri/src/reporting/informe.rs`
- [X] T017 [P] [US1] Prueba unit: disco con última lectura SMART anterior al intervalo → valores con «última lectura: hace X»; disco que nunca tuvo SMART → «No disponible» — `src-tauri/src/reporting/informe.rs`

### Implementación

- [X] T018 [US1] `src-tauri/src/reporting/informe.rs`: tipos `SeccionDiscoInforme`, `IdentidadDisco`, `SaludActual`, `VolumenInforme`, `ContadorConDelta`, `AlertaLegible`, `EventosSeccion`, `Frescura` (según `data-model.md` §2)
- [X] T019 [US1] `src-tauri/src/reporting/informe.rs`: reescribir `seccion_dispositivo` para montar una `SeccionDiscoInforme` (identidad, `SaludActual` con `frescura`, volúmenes, `contadores_con_delta`, alertas filtradas por `target_device_id` + `alerta_en_rango` con texto de `alertLabels`, `eventos_de_dispositivo_en_rango`, `minigrafica` de temperatura y de actividad) — depende de T005/T007/T009/T011/T013
- [X] T020 [US1] `src-tauri/src/reporting/informe.rs`: reescribir `generar_html` — cabecera con `SCHEMA_VERSION_HTML` y un resumen de flota de una línea; CSS embebido ampliado (tablas, SVG, hoja de impresión, tema claro); ensamblar las secciones; firma acepta `alert_labels: &HashMap<String, String>`
- [X] T021 [US1] `src-tauri/src/commands/mod.rs` `export_report_impl` rama `"html"`: reunir por disco (inventario, `volumes_for_device`/`get_volume`, contadores, alertas de `list_groups`, eventos, series de temperatura y actividad) y llamar a `generar_html` con el mapa `alert_labels`
- [X] T022 [US1] Contrato: ampliar la firma de `export_report` en `src/lib/api/client.ts` (`alertLabels?`, `includeAiSummary?`, `previewConfirmada?`) y el esquema en `src/lib/api/schemas.ts`; el backend valida los nombres nuevos aunque aún ignore `includeAiSummary`
- [X] T023 [US1] `src/routes/reports/+page.svelte`: construir `alertLabels` desde los diccionarios i18n (claves de regla de `docs/alert-rules.md` §2) y pasarlo en `exportReport({ format: "html", … })`; afinar las claves i18n de T003 para los encabezados de sección
- [X] T024 [US1] `e2e/ui/reports.spec.ts`: al exportar HTML se invoca `export_report` con `format: "html"` y `alertLabels` no vacío; `includeAiSummary` ausente/false

**Checkpoint**: la Historia 1 funciona y se prueba sola. **MVP entregable.**

---

## Phase 4: User Story 2 — Resumen en lenguaje llano por disco, con IA (Priority: P2)

**Goal**: si la ayuda con IA está configurada, la pantalla de Informes ofrece «incluir resumen con
IA»; al activarla y exportar HTML se muestra la vista previa del texto anonimizado por disco; una
confirmación → una llamada por disco → párrafo de orientación por disco en el informe; degrada sin
tirar la exportación.

**Independent Test**: IA configurada + un disco con alertas → `preview_informe_ia` devuelve su
texto anonimizado; confirmar → HTML con el párrafo IA; un disco con fallo simulado → nota.

### Pruebas (van antes)

- [X] T025 [P] [US2] Prueba unit `domain/ia.rs` (`#[cfg(test)]`): `componer_consulta` con `Detalle::Informe` → `system`+`user` contienen alertas, contenidos de suceso, contadores y resúmenes numéricos de **un** disco; con dos discos, el de uno no contiene datos del otro
- [X] T026 [P] [US2] Prueba unit `reporting/informe_ia.rs` (`#[cfg(test)]`): la construcción del payload de un disco cuyo nº de serie, etiqueta de volumen y (`COMPUTERNAME`) nombre de equipo aparecen en el texto → salen como `<SERIE-1>`/`<VOLUMEN-1>`/`<EQUIPO>`; `redactedFields` los lista; **no** se invoca `platform::ia_openrouter`
- [X] T027 [P] [US2] Prueba unit `reporting/informe_ia.rs` (`#[cfg(test)]`): degradación — un doble de `chat_completions` que devuelve `Err` para el disco `k` → `resumen_ia[k] == NoDisponible`, el resto `Generado`; el HTML se ensambla y escribe

### Implementación

- [X] T028 [US2] `src-tauri/src/domain/ia.rs`: `DetalleInforme<'a>` + variante `Detalle::Informe` + rama en `componer_consulta` (o `componer_consulta_informe`); `system` adaptado («resume el estado de **este** disco en el periodo; orientación, no diagnóstico»)
- [X] T029 [US2] `src-tauri/src/reporting/informe_ia.rs` NUEVO: por disco reúne alertas + `extraer_contenido_suceso` de sus sucesos + contadores + `ResumenMetrico` de temperatura y actividad; `componer_consulta`; anonimiza (`Anonimizador::para_esta_maquina(series).con_etiqueta_volumen(...)` por cada volumen + `redactar_identificadores`); `recortar`; `barrer_texto_residual` (omitido con `send_without_review`). Expone `preview_datos(conn, rango, device_ids, alert_labels) -> Vec<PreviewDiscoWire>` (sin red) y `generar_resumen_disco(clave, modelo, payload) -> ResumenIaSeccion` (con red)
- [X] T030 [US2] `src-tauri/src/commands/mod.rs`: comando `preview_informe_ia(from_utc, to_utc, device_ids, alert_labels) -> PreviewInformeIaWire` (sin red); registrarlo en `src-tauri/src/lib.rs`
- [X] T031 [US2] `src-tauri/src/commands/mod.rs` `export_report_impl`: cuando `format == "html"` && `include_ai_summary` → función `async`; error `report.preview_required` si falta `preview_confirmada`; bucle por disco llamando a `informe_ia::generar_resumen_disco`; `tracing::debug!(modelo, resultado, ms)` por llamada, **sin** número de serie, equipo ni ruta (§XV); ensambla el HTML con los resúmenes/notas y lo escribe. `export_report` (comando `#[tauri::command]`) pasa a `async`
- [X] T032 [US2] `src-tauri/src/reporting/informe.rs`: pintar `ResumenIaSeccion` en la sección del disco — `Generado{markdown, modelo}` como **texto HTML-escapado** en `<div style="white-space: pre-wrap">` precedido de «Resumen generado por IA con {modelo} — orientación, no un diagnóstico»; `NoDisponible{motivo}` como nota (texto de `alertLabels`/i18n)
- [X] T033 [US2] `src/lib/api/client.ts` + `src/lib/api/schemas.ts`: `previewInformeIa(...)` + esquema Zod `PreviewInformeIaWire` (+ `FragmentoDudosoWire` ya existe); `exportReport` acepta `includeAiSummary`/`previewConfirmada`
- [X] T034 [US2] `src/routes/reports/+page.svelte`: `Switch` «incluir resumen con IA», visible solo si `estado_ia().activa`, con aviso del número de llamadas; al exportar HTML con la casilla marcada → `previewInformeIa` → modal de vista previa (patrón `ExplicacionModal`/`ConfirmDialog`) con el texto por disco y los `fragmentos` → al confirmar, `exportReport({ includeAiSummary: true, previewConfirmada: true })`
- [X] T035 [US2] `src/lib/i18n/{es,en}.json`: casilla, aviso, título y cuerpo del modal, `report.preview_required`, `diagnostic.redacted.volumeLabel`. La marca de IA en el informe y la nota de degradación no necesitan clave: viven en el HTML del backend, que no tiene diccionarios (ADR-030) y es solo-español por diseño de todo `informe.rs`, igual que el resto de sus textos
- [X] T036 [US2] `e2e/ui/fixtures/respuestas.ts`: `estado_ia` activa, respuesta de `preview_informe_ia` (`vistaPreviaInformeIa`, 2 discos, uno con fragmentos dudosos)
- [X] T037 [US2] `e2e/ui/reports.spec.ts`: IA configurada → casilla visible; marcar + exportar HTML → se invoca `preview_informe_ia` **antes** de `export_report`; el modal muestra el texto por disco; IA **no** configurada → casilla ausente
- [X] T038 [US2] `e2e/ui/reports.spec.ts`: confirmar la vista previa → `export_report` se invoca con `includeAiSummary: true, previewConfirmada: true` y resuelve con una ruta. La degradación por disco (una llamada falla, el resto sigue) es enteramente responsabilidad del backend (el bucle por disco vive dentro de `export_report`, nunca como llamadas IPC independientes que el doble de IPC pueda interceptar una a una) y ya está probada a nivel Rust: `informe_ia::tests` (fallo de red/proveedor → `NoDisponible`) e `informe::tests::un_resumen_ia_no_disponible_usa_la_etiqueta_si_la_hay_o_una_nota_generica`

**Checkpoint**: Historias 1 y 2 funcionan de forma independiente.

---

## Phase 5: User Story 3 — Progreso y cancelación (Priority: P3)

**Goal**: la exportación con IA muestra el progreso (disco n de N) y se puede cancelar sin dejar un
fichero a medias; la exportación sin IA sigue siendo inmediata.

**Independent Test**: export con IA + 3 discos → 3 eventos `report:progress`; cancelar tras el 2º →
`export.cancelled`, sin fichero, sin llamada del disco 3.

### Pruebas (van antes)

- [X] T039 [P] [US3] Prueba unit `src-tauri/src/commands/mod.rs` (`#[cfg(test)]`): con la bandera `informe_cancelado` a `true` antes del disco `k` → no se escribe fichero, se devuelve `export.cancelled`, no hay llamada para `k` ni siguientes. Implementado sobre `resumenes_ia_por_disco` (helper extraído para inyectar la llamada de red y probar sin tocarla ni añadir un ejecutor async de más — mismo criterio que `resultado_a_seccion`), con un `Waker` mínimo hecho a mano (sin dependencia nueva): `cancelar_antes_de_empezar_no_llama_a_la_red_de_ningun_disco` + `sin_cancelar_emite_progreso_en_orden_y_cada_disco_recibe_su_propio_texto`
- [X] T040 [US3] `src-tauri/src/persistence/db.rs`: campo `AppState.informe_cancelado: std::sync::Arc<std::sync::atomic::AtomicBool>` inicializado en `AppState::open`; **no se persiste**
- [X] T041 [US3] `src-tauri/src/commands/mod.rs`: comando `cancelar_informe()` (pone la bandera a `true`); registrado en `src-tauri/src/lib.rs`; `exportar_informe_con_ia` la resetea a `false` al empezar la fase de IA y la comprueba antes de cada disco (dentro de `resumenes_ia_por_disco`)
- [X] T042 [US3] `src-tauri/src/commands/mod.rs`: emitir el evento `report:progress` `{ emittedAt, done, total, deviceLabel }` antes de la llamada de cada disco (mismo patrón que `test:progress`, vía `emitir_report_progress`)
- [X] T043 [US3] `src/lib/api/client.ts`: `cancelarInforme()`; esquema `reportProgress`/`report:progress` añadido a `eventSchemas` (`schemas.ts`, `event-types.ts`, `events.ts`) — suscripción directa en `reports/+page.svelte` vía `on()` (no hizo falta un store nuevo: es la única pantalla que lo necesita, mismo criterio que `tests/+page.svelte` con `test:progress`)
- [X] T044 [US3] `src/routes/reports/+page.svelte`: el modal de vista previa cambia de fase al confirmar — «Resumiendo disco {done} de {total}: {deviceLabel}» con `role="status" aria-live="polite"`, `ProgressBar` indeterminada hasta el primer evento, botón «Cancelar» que llama a `cancelar_informe`; el velo/Escape no cierran el modal mientras corre; los botones de exportación ya quedaban deshabilitados por el `exportando`/`cargandoPreviaIa` existentes
- [X] T045 [US3] `e2e/ui/reports.spec.ts`: dos pruebas — progreso visible tras 2 `report:progress` (con `export_report` retardado) y clic en «Cancelar» invoca `cancelar_informe`; y, por separado, un `export_report` que rechaza con `export.cancelled` (el desenlace real de una cancelación) muestra la nota y cierra el modal. El doble de IPC no reproduce «cancelar a mitad de N llamadas reales»: eso ya lo prueba `commands::tests_informes` en Rust (T039) sobre el bucle real

**Checkpoint**: las tres historias funcionan de forma independiente.

---

## Phase 6: Polish & Cross-Cutting

- [X] T046 [P] Finalizar **ADR-057** en `docs/decisions.md` (flujo completo, contratos nuevos, la decisión de `alertLabels` frente a ADR-030, el resumen IA como texto sin render); marcar «aceptada»; fecha
- [X] T047 [P] `docs/ui-contract.md`: §3.7 `export_report` con `alertLabels`/`includeAiSummary`/`previewConfirmada`; añadir `preview_informe_ia` y `cancelar_informe`; §3.10 nota del segundo uso de la IA; §4 evento `report:progress`
- [X] T048 [P] `docs/product-specification.md` §9: el informe HTML por disco (contenido) y el resumen IA opcional con vista previa y degradación
- [X] T049 [P] `docs/open-questions.md`: sección Y nueva con los valores adoptados (mapa `alertLabels`; tope 50 eventos; `schemaVersion` HTML = 2; resumen IA como texto escapado sin render de markdown; una llamada por disco incluido; motivo de degradación vía `alertLabels`; cancelación antes de cada disco); J.30 marcada como reemplazada en lo que toca al HTML
- [X] T050 [P] `docs/ui-design.md` §7 punto «Informes»: modal de vista previa del envío (mismo patrón que `ConfirmDialog`/`ExplicacionModal`, no un componente nuevo) y su cambio de fase a bloque de progreso. Sin `svelte-ignore` nuevo (se reutiliza el ya cubierto por `docs/known-issues.md` #2)
- [X] T051 `pnpm docs:build` y `pnpm docs:check`
- [X] T052 Puertas completas: desde `src-tauri/` `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; en la raíz `pnpm check`, `pnpm lint`, `pnpm verify`, `pnpm test`, `pnpm test:component`, `pnpm test:e2e`, `pnpm test:a11y`, `pnpm build`, `pnpm docs:check` — todas en verde salvo el fallo preexistente ya documentado de `export.rs` (no relacionado con esta spec)
- [ ] T053 Validación manual (app real) de `specs/009-informe-mejorado/quickstart.md`: escenario 1 (informe abierto sin red), escenario 3 (IA con 4 discos y un fallo forzado), escenario 4 (IA apagada = cero red). **Pendiente del usuario**: exige la app real con GUI nativa (fuera del alcance de las herramientas de este agente) y, para el escenario 3, una clave de IA real u obtenida con la clave de demostración. Anotar el resultado en la memoria del proyecto
- [ ] T054 Skill `cierre-tarea`: matriz de documentación y las nueve puertas antes de proponer commit (documentación ya aplicada en T046-T050; pendiente de formalizar tras T053)

---

## Dependencies & Execution Order

### Fases

- **Setup (F1)**: sin dependencias.
- **Foundational (F2)**: tras F1. **Bloquea** todas las historias.
- **US1 (F3)**: tras F2. MVP. Sin dependencias de otras historias.
- **US2 (F4)**: tras F2. Usa la sección de disco de US1 (T032 pinta dentro de la sección que crea T019) → en la práctica **tras US1**, aunque el trabajo de dominio (T025/T028/T029) puede empezar en cuanto F2 cierre.
- **US3 (F5)**: tras US2 (el progreso y la cancelación son de la fase de IA de US2).
- **Polish (F6)**: tras las historias que se vayan a entregar.

### Dentro de cada historia

- Las pruebas (marcadas «van antes») se escriben y **fallan** antes de su implementación.
- Tipos/helpers antes que su uso; comando antes que la interfaz; interfaz antes que el e2e.

### Oportunidades de paralelismo

- **F1**: T001, T002, T003 en paralelo.
- **F2**: T004+T005, T006+T007, T014 en paralelo entre sí (ficheros distintos); T008-T013 encadenadas de dos en dos (prueba→impl) pero los tres pares son independientes entre sí.
- **US1**: T015, T016, T017 (pruebas) en paralelo; luego T018→T019→T020→T021 en serie (mismo fichero `informe.rs` y dependencias), T022+T023 en paralelo con T021.
- **US2**: T025, T026, T027 en paralelo; T028 y T029 tocan ficheros distintos; T033/T034/T035 (frontend) en paralelo con T028-T032 (backend).
- **US3**: T040→T041→T042 en serie (mismo `commands/mod.rs`/`db.rs`); T043+T044 en paralelo.
- **F6**: T046-T050 en paralelo; T051→T052→T053→T054 en serie.

---

## Implementation Strategy

### MVP (solo US1)

1. F1 Setup → 2. F2 Foundational → 3. F3 US1 → 4. **Validar US1 sola** (informe HTML útil, offline) → entregable.

El informe deja de «no decir nada» sin depender de la IA ni de red.

### Entrega incremental

1. Setup + Foundational → base.
2. US1 → informe legible por disco (**MVP**, se puede commitear/entregar).
3. US2 → resumen con IA opt-in (bloqueante, con spinner genérico).
4. US3 → progreso por disco + cancelación.

Cada historia añade valor sin romper la anterior. US1 puede ir en su propio commit; US2+US3 pueden
ir juntas o separadas.

---

## Notes

- `[P]` = ficheros distintos, sin dependencias pendientes.
- La constitución exige prueba antes de código en anonimización, degradación y cancelación: las
  tareas ya lo reflejan.
- Sin dependencias nuevas, sin permisos de Tauri nuevos, sin cambios de esquema SQLite.
- `historias.md` es generado: se editan los `docs/*` y se corre `pnpm docs:build` (T051).
- Commit por tarea o grupo lógico; no confirmar sin `cierre-tarea` (T054) y autorización.
