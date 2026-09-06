---
description: "Lista de tareas — Rediseño visual «SmartDisk Monitor v3»"
---

# Tasks: Rediseño visual «SmartDisk Monitor v3»

**Input**: `specs/002-rediseno-v3/` — `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Rama**: `002-rediseno-v3` · **Entrega**: 9 PR ordenados por dependencia (ver `plan.md` §«Fases de entrega»).

## Formato: `[ID] [P?] [Story] Descripción con ruta de fichero`

- **[P]**: paralelizable (ficheros distintos, sin dependencias con tareas incompletas).
- **[Story]**: US1…US10 (mapea a las historias de `spec.md`). Setup / Foundational / Polish no llevan etiqueta.
- Todo cambio de token va **solo** en `tokens.css` / `tailwind.config.cjs` / `tokens.json`. Cero literales en componentes (`pnpm verify:tokens`).
- Todo texto visible (incluidos `aria-label`, `title`) va en `es.json` **y** `en.json` en el mismo commit (`pnpm verify:i18n`).
- Cada PR deja `pnpm verify && pnpm check && pnpm test && pnpm test:component && pnpm lint` en verde.

## Convenciones de ruta

- Frontend: `src/` en la raíz. Componentes en `src/lib/components/`, lógica de presentación en `src/lib/design/`, rutas en `src/routes/`, diccionarios en `src/lib/i18n/`.
- Backend: `src-tauri/src/` (solo Fase 7 / PR 5). Comandos en `src-tauri/src/commands/`, dominio en `src-tauri/src/domain/`, motor de alertas en `src-tauri/src/alerts/`.
- Documentación normativa en `docs/`.

---

## Phase 1: Setup (infraestructura compartida)

**Propósito**: dejar la base lista y capturar el «antes» para comparar.

- [x] T001 Confirmar rama `002-rediseno-v3` y que `pnpm install && pnpm verify && pnpm check && pnpm test` está en verde antes de tocar nada; anotar el resultado en el PR 1. — *baseline verde: verify ✓, check 0/0, test 179/179, test:component 43.*
- [x] T002 [P] Leer entera `docs/ui-design.md` (§0–§8) y `design/propuesta-redisenov2/RESUMEN.md` + `RESPUESTAS-A-LA-REVISION.md`; no empezar ninguna pantalla sin haberlo hecho.
- [ ] T003 [P] Capturar el estado visual actual de las 8 pantallas... — **diferida**: requiere la app elevada (UAC). Ver `regresion-visual.md`; se apoya en `test:component` + e2e + revisión manual.
- [x] T004 [P] Abrir `design/propuesta-redisenov2/mockups/smartdisk-v3.html` y `mockups/icons-hoja-de-contacto.html` como referencia durante toda la feature.

---

## Phase 2: Foundational (prerrequisitos que bloquean historias)

**Propósito**: piezas transversales que no son de una sola historia.

**⚠️ La historia US1 (Fase 3) es a su vez prerrequisito duro de todas las demás: sin los tokens v3 ningún componente ni pantalla puede recomponerse.**

- [x] T005 [P] ADR-034 «Sistema de diseño v3: paleta propia Ciruela» en `docs/decisions.md` (aceptada), enmienda ADR-013, con «Qué NO cambia». *(Cuerpo completo hecho ya en este paso, T012 redundante.)*
- [x] T006 [P] ADR-035 «La herencia del acento de Windows pasa a opción apagada de fábrica» (aceptada), matiza ADR-017 (conserva `accessibleAccent()`/`accentOnSurface()`).
- [ ] T007 Añadir en `.specify/memory/constitution.md` §VI una nota al pie del punto «El acento… se hereda de Windows» remitiendo a ADR-035. **Requiere modo plan y autorización explícita del usuario.** — *pendiente: PENDIENTE DE AUTORIZACIÓN.*
- [x] T008 [P] `specs/002-rediseno-v3/regresion-visual.md` creado con la checklist §8 por PR.

**Checkpoint**: base documental lista. La implementación de tokens empieza en la Fase 3.

---

## Phase 3: User Story 1 — Sistema de diseño v3 aplicado (Priority: P1) 🎯 MVP · PR 1

**Goal**: la interfaz entera adopta la paleta Ciruela, la tipografía de display y el contraste corregido, sin recomponer ninguna pantalla todavía.

**Independent Test**: recorrer las 8 pantallas en claro y oscuro; el acento es morado, el rojo crítico bermellón, el texto sobre el acento en oscuro es tinta ≥ 4,5:1; `pnpm verify:tokens` en verde.

### Implementación

- [x] T009 [US1] `src/design-system/tokens.css`: paleta Ciruela (light + dark), `--sdm-on-accent` tinta en oscuro, 8 tokens nuevos.
- [x] T010 [US1] Utilidad `.sdm-display` en `tokens.css`.
- [x] T011 [US1] `tailwind.config.cjs` (`font-display`, `text-display`, `text-hero`, `w-rail`, `h-hero`) + `tokens.json` (v3.0.0, colores + escalas + layout + icon).
- [x] T012 [US1] ADR-034 y ADR-035 completos (hecho en T005/T006).
- [x] T013 [US1] `docs/ui-design.md` §intro, §0, §2, §2.bis, §3 → v3. §4/§6/§8 conservados. *(§7 composiciones se actualizan por PR de pantalla.)*
- [x] T014 [US1] Ratios medidos con la colorimetría de `accent-check.py` y registrados en `docs/open-questions.md` §S. Todos ≥ 4,5:1; los 3 más justos verificados (text-faint 4,80 · ok píldora 4,68 · crit píldora oscuro 4,57).
- [x] T015 [US1] `Button.svelte` `primary` → `text-fg-onAccent` (ya estaba; verificado).
- [x] T016 [P] [US1] Barrido hecho. `bg-warn text-white` (ConfirmDialog, reports, tests) es pre-existente y NO es «sobre el acento» → deuda anotada en `regresion-visual.md` para PR 7. `Sidebar` logo → PR 3. `Switch` punto blanco → excepción.
- [x] T017 [US1] `tokens.browser.test.ts` ampliado: +3 pruebas (tokens nuevos por tema, `--sdm-on-accent` tinta en oscuro, `.sdm-display`). 46 pruebas de componente en verde.
- [x] T018 [US1] `pnpm docs:build` + `pnpm docs:check` al día (31 ficheros).
- [x] T019 [US1] Checklist §8 del cambio global en `regresion-visual.md`. Puerta: verify ✓, check 0/0, lint ✓, test 179, test:component 46, e2e:smoke 5/5. **Falta la revisión visual manual del usuario.**

**Checkpoint**: la app se ve en v3. Nada más cambia todavía. **Prerrequisito de todas las fases siguientes.**

---

## Phase 4: User Story 2 — Iconografía (Priority: P1) · PR 2

**Goal**: los 15 iconos disponibles en el catálogo y cada estado/magnitud/bus con su símbolo.

**Independent Test**: `Icon` renderiza los 15 símbolos heredando `currentColor` en los 2 temas; sin `label` queda `aria-hidden`, con `label` se anuncia como imagen; `busIcon("USB…") === "usb"`.

### Implementación

- [x] T020 [US2] Sprite de 15 `<symbol>` en `AppShell.svelte`, sin el bloque `<metadata><c2pa:manifest>`. Montaje único.
- [x] T021 [P] [US2] `src/lib/design/icons.ts`: `IconName` (15) + `ICON_NAMES` + `healthIcon`, `eventLevelIcon`, `busIcon`, `testIcon`.
- [x] T022 [US2] `src/lib/components/Icon.svelte`: `{ name, size=16, label? }`; con `label` → `role="img"`+`aria-label`; sin → `aria-hidden` + `focusable="false"`.
- [x] T023 [US2] `Icon` exportado en `index.ts`.
- [x] T024 [US2] `StatusPill.svelte`: prop `icon?: IconName | "auto"`, `aria-hidden`, excluyente con `withDot` (gana `icon` por construcción). *(El aviso de consola en dev se descartó: `console.*` directo viola §XV; la exclusión se cubre con comentario + prueba.)*
- [x] T025 [P] [US2] `Icon.browser.test.ts`: 5 pruebas (aria-hidden sin label, role img con label, `href="#i-{name}"`, sin color propio, tamaño).
- [x] T026 [P] [US2] `src/lib/design/icons.test.ts`: 8 pruebas (15 símbolos, mapas, `busIcon` incl. USB sobre HDD).
- [x] T027 [P] [US2] `StatusPill.browser.test.ts` nuevo: 5 pruebas (label siempre, `icon="auto"`, icon explícito manda, excluyente con `withDot`, punto con `withDot` solo).
- [x] T028 [US2] La regla «`role="img"` sin nombre es peor que no ponerlo» la aplica `Icon.svelte` en su estructura; la cubren `Icon.browser.test.ts` + la suite `axe` de `e2e/ui/a11y.spec.ts`.

**Checkpoint**: iconos montados. `StatusPill` ya los usa.

---

## Phase 5: User Story 3 — Chrome coherente y sin contradicciones (Priority: P1) · PR 3

**Goal**: riel de 74 px, título de la barra de herramientas por ruta, estado global único. Corrige 2 de los 3 bugs preexistentes.

**Independent Test**: a 1024×560 la barra lateral ocupa 74 px; el título cambia con la ruta; la píldora de estado y el icono del pie del riel dicen lo mismo; teclado recorre las secciones con su nombre anunciado.

### Implementación

- [x] T029 [US3] `src/lib/design/health.ts`: `globalStatus({loaded, paused, monitoredStates}) → {kind, state, count}` (kind: loading/paused/noDevices/ok/attention). Distingue «aún no cargado» de «cero discos» — la causa real del bug. 7 pruebas en `health.test.ts`.
- [x] T030 [US3] `Sidebar.svelte` reescrito como riel de `w-rail` (74 px). Props `{ sections:{id,label,icon,href,badge?}[], active, globalState, globalLabel, globalIcon, globalCount?, onabout? }`. Logo `diskStack`, 6 secciones + «Acerca de», botones de `size-11` (44 px) con `title`+`aria-label`, punto de aviso `bg-warn` sobre Alertas (recuento también en el `aria-label`), pie `role="status"` con `Icon` + contador. Selección: material elevado + `text-accent-fg`, sin barra lateral.
- [x] T031 [US3] `Toolbar.svelte`: props `{ title, subtitle?, globalState, globalLabel, freshness?, stale?, primaryLabel?, primaryLoading?, onprimary? }`. Alto `h-14` (56 px). Título `.sdm-display text-xl`. `StatusPill … icon="auto"`. Sin `onabout`, sin ranura `controls`, sin botón «?».
- [x] T032 [US3] `+layout.svelte`: `globalStatus()` una vez → misma fuente a `Sidebar` y `Toolbar`. Título de `page.data.title` (con `App.PageData` en `app.d.ts`), fallback a la etiqueta de sección. `disks/[id]/+page.ts` devuelve `title: disk.alias ?? disk.model`. **Inventario cargado en el `onMount` del layout** (no solo en `/`) para que el estado global sea correcto en cualquier ruta de entrada. **Corrige los 2 bugs de chrome.**
- [x] T033 [US3] Pausa movida a `settings/+page.svelte` (card «Registro de actividad») como `Switch` «Recopilación de datos activa» (`checked={!app.paused}`). El estado sigue viniendo por eventos, no se persiste.
- [x] T034 [US3] Claves i18n: `app.name`, `global.loading`, `nav.alertsUnread`, `settings.collection.label`/`.hint` (es+en); `global.paused` acortado a «En pausa»/«Paused».
- [x] T035 [P] [US3] `Sidebar.browser.test.ts` reescrito (5 pruebas: nombre accesible sin texto, `aria-current`, recuento en el nombre, `role="status"`, `onabout`). `Toolbar.browser.test.ts` nuevo (5: título variable, píldora con icono, sin «?», frescura `stale`, botón primario).
- [x] T036 [P] [US3] Cubierto por `e2e/ui/smoke.spec.ts` (navega las 6 secciones) + `escalado.spec.ts`.
- [x] T037 [P] [US3] `e2e/ui/escalado.spec.ts` pasa: el riel a 1024×560 no recorta (7 pantallas × 5 escalados).
- [x] T038 [US3] Checklist §8 del chrome en `regresion-visual.md`. Iconos de sección = los del `navDefs` del mockup (Informes `shield`, Ajustes `wear`, etc.). Menor: el icono de «Acerca de» en el riel (`tag` provisional) no está en el mockup — a confirmar con el diseñador.

**Checkpoint**: chrome nuevo, 2 bugs corregidos, 176 px devueltos al contenido.

---

## Phase 6: User Story 4 — La gráfica de temperatura vuelve a comunicar (Priority: P1) · PR 4

**Goal**: `Sparkline` nuevo + `TimeSeriesChart` con trazo, eje Y, relleno y banda de hueco. Corrige el 3.er bug preexistente.

**Independent Test**: detalle de un disco con serie con hueco → trazo visible, eje Y con 4 marcas, relleno bajo la curva, banda gris sobre el hueco con leyenda; nunca interpola; el grosor no cambia al redimensionar.

### Implementación

- [x] T039 [US4] `src/lib/design/series.ts` NUEVO — `tramos`, `huecos`, `cadencia`, `rangoConAire`, `submuestrear` (puras, sin píxeles). La regla «un hueco es un hueco» **una sola vez**. `Sparkline.svelte` NUEVO: props `{ points, min?, max?, color?, height?, fill?, strokeWidth?, expectedIntervalMs?, label? }`; `<polyline>` por tramo, `vector-effect="non-scaling-stroke"`, `preserveAspectRatio="none"`, `id` de degradado con `crypto.randomUUID()`. Serie `[]`/toda `null` → no renderiza `<svg>`.
- [x] T040 [US4] `Sparkline` exportado en `index.ts`.
- [x] T041 [US4] `TimeSeriesChart.svelte` reescrito sobre `series.ts` (misma lógica que `Sparkline`): eje Y `GUTTER=34` con 4 marcas `sdm-num`, relleno degradado `stop-opacity .32→0`, banda de hueco al 14 % con leyenda `chart.gapRange` («sin datos HH:MM – HH:MM») por hueco, umbral discontinuo + leyenda, grosor 2,6, `color` prop que sigue el estado. Cursor teclado/ratón conservado. *(Nota: comparte la lógica vía función, no anida `<Sparkline>` — los sistemas de coordenadas son incompatibles; el objetivo del diseñador —una sola implementación de «hueco es hueco»— se cumple igual.)*
- [x] T042 [US4] `!hayMuestras` → marco + eje Y + `<text>` central `chart.noSamples`, sin línea a cero.
- [x] T043 [US4] i18n: `chart.gapRange`, `chart.noSamples` (es+en); `chart.gaps.one/other` eliminadas (ya no se usan). Reutiliza `chart.tempWarnLabel`/`chart.tempCritLabel`/`chart.resolution.*`.
- [x] T044 [P] [US4] `series.test.ts` (18 pruebas — tramos, huecos, cadencia, rango, submuestreo). `Sparkline.browser.test.ts` (6 — dos tramos, non-scaling-stroke, vacía sin svg, id único, a11y).
- [x] T045 [P] [US4] `TimeSeriesChart.browser.test.ts` +4: eje Y de 4 marcas, trazo se dibuja (regresión), leyenda de hueco, umbral.
- [x] T046 [P] [US4] Cubierto por `e2e/ui` (detalle de disco) — 76/76.
- [x] T047 [US4] Checklist §8 de la gráfica en `regresion-visual.md`.

**Checkpoint**: el elemento más grande del detalle vuelve a comunicar.

---

## Phase 7: User Story 10 — Perfiles de alerta configurables (Priority: P2) · PR 5 · ⚠️ MODO PLAN

**Goal**: umbrales configurables en `settings.alerts`, `settings.alerts.profile`, lógica «perfil → custom», consumo en el motor de alertas. Añade `VolumeSummary.isSystemVolume`.

**Independent Test**: en Ajustes elegir «Prudente» escribe los umbrales de `data-model.md` §3.3 y `profile = "cautious"`; editar uno a mano → `custom` + «Personalizado (a partir de Prudente)»; el motor genera el grupo correcto al superar un umbral nuevo; guardar crítico < aviso → `AppError` y revierte.

> **PR 5 hecho.** Modo plan aprobado (`~/.claude/plans/velvet-skipping-mango.md`). Decisión del usuario: **también cablear las reglas de capacidad** en el motor (`evaluar_capacidad` + persistir `volume_free_bytes`). `driver_retry_*` se guarda pero no se consume (Historia 4). ADR-036 escrito. Puerta Rust: `cargo test` 459 ✓ · `clippy -D warnings` ✓ · cobertura `domain`+`alerts` **93,3 %**.

### Preparación

- [x] T048 Modo plan hecho; Q1/Q2 registradas en `docs/open-questions.md` §T antes de programar.

### Backend — esquema y validación

- [x] T049 [US10] `src-tauri/src/domain/ajustes.rs`: constantes de rango + fábrica de `wear`, `media_errors`, `driver_retry`; `TEMP_*_DEFAULT_C` → 60/70. Enum `PerfilAlerta` + `UmbralesPerfil` + `PERFIL_DEFECTO`. `VolumeSummary` (en `commands/mod.rs`, que es donde vive) gana `is_system_volume: bool`.
- [x] T050 [US10] `ajustes.rs`: `validar_desgaste`, `validar_media_errors`, `validar_reintentos_controlador` (patrón de `validar_temperaturas`). Pruebas de rechazo (rango, `crit ≤ warn`) — 7 tests nuevos.
- [x] T051 [US10] `set_setting_impl`: brazos para las 6 claves de umbral + `alerts.profile` + `settings.onboarding.completed_at`. `claves_por_ambito("alerts")` ampliado. `SettingsWire` + `OnboardingSettingsWire` + `get_settings_impl`. Prueba: clave desconocida sigue rechazándose.
- [x] T052 [US10] `alerts.profile` con un identificador de perfil escribe los 12 umbrales de `PerfilAlerta::umbrales()` en la misma conexión. `marcar_perfil_personalizado()` tras editar cualquier `alerts.*` a mano. 4 tests (perfil escribe 12, editar → custom, perfil desconocido, umbral fuera de rango).
- [x] T053 [US10] `platform/sistema.rs` NUEVO: `letra_unidad_sistema()` vía `GetSystemWindowsDirectoryW` (FFI kernel32). `build_volume_summaries` calcula `is_system_volume` comparando la letra. **Sin migración.** ts-rs regenera `VolumeSummary.ts` en `cargo test`.

### Backend — motor de alertas (test-first, 5 pruebas por regla · §VIII)

- [x] T054 [US10] `motor.rs`: `evaluar_wear_high(serie, warn_pct, crit_pct)`. `alerts/mod.rs` gana `ConfigUmbrales` (plano, resuelto por `commands::refresh_smart` de `settings`). `aplicar_simple`/`aplicar_par_volumen` con `impl Fn`. Pruebas en el umbral, ±ε, sin dato, configurables.
- [x] T055 [US10] `motor.rs`: `evaluar/resuelve_temperatura_configurada_{warn,crit}(serie, umbral)`. Histéresis conserva el margen (−3 / −5 °C). Prueba de integración en `alerts/mod.rs`: 62 °C salta con 60, no con 70. Tabla de `alert-rules.md` §2 actualizada.
- [x] T056 [US10] **Reglas de capacidad implementadas** (decisión del usuario): `domain/capacidad.rs` NUEVO (`estado_capacidad`, espejo del TS, 6 tests). `motor::evaluar_capacidad_{low,critical}` + `resuelve_*`. `alerts::evaluar_capacidad(conn, volume_id, cap, ahora, umbrales)`. `refresh_inventory` persiste `volume_free_bytes` por volumen monitorizado y llama a `evaluar_capacidad`; devuelve las transiciones a `post_procesar_ciclo`. 3 tests de integración (activación, sin datos, dedup+resolución).
- [x] T057 [US10] `motor::evaluar_media_errors(serie, warn, crit)` = umbral sobre la magnitud del incremento por ciclo (Q1). `error_log` sin cambios. 4 tests.
- [~] T058 [US10] `events.controller_reset` / `events.io_retry` — **fuera de alcance**: esas reglas no existen (necesitan el colector de eventos, Historia 4). `driver_retry_*` se guarda; `open-questions.md` §T.4 lo registra.
- [~] T059 [US10] `temp.above_vendor_limit` ya manda sobre el configurado en el motor actual (`domain::salud` colapsa al peor de los grupos independientes); no cambia con esto.
- [x] T060 [US10] `cargo llvm-cov` sobre `domain/` + `alerts/`: **93,3 %** (≥ 90 %).

### Frontend

- [x] T061 [US10] `schemas.ts`: `settings.alerts` 7→14 campos + `profile` (`z.enum`); `settings.onboarding.completedAt`; `volumeSummary.isSystemVolume`. `schemas.test.ts` +4 pruebas de rechazo (perfil fuera del enum, falta un umbral, marca de asistente inválida).
- [x] T062 [US10] `VolumeSummary.ts` regenerado por `cargo test`. `src/lib/design/types.ts` y las fixtures (`DiskCard.browser.test.ts`, `e2e/ui/fixtures/respuestas.ts`) actualizadas.
- [x] T063 [US10] `settings/+page.svelte`: `RadioGroup` de perfil + 6 `TextField` nuevos + «Personalizado (a partir de {base})» (base del último perfil concreto en memoria). `cambiarPerfil` recarga `getSettings()` tras elegir; `cambiarAlerta` marca `profile: "custom"` de forma optimista.
- [x] T064 [US10] Claves i18n del PR 5 (17 nuevas, es+en).
- [x] T065 [US10] `cambiarAlerta` ya enruta el `AppError` a `saveError` (junto al bloque de la pantalla). *(Un error junto a cada control concreto queda como afinado de PR 7.)*

### Documentación

- [x] T066 [US10] `docs/alert-rules.md` §2, `docs/ui-contract.md` §3, `docs/data-model.md`, `docs/open-questions.md` §T, **ADR-036** en `docs/decisions.md`.
- [x] T067 [US10] `pnpm docs:build` + `pnpm docs:check` al día.

**Checkpoint**: perfiles funcionales desde Ajustes; el motor los respeta; `isSystemVolume` disponible para el héroe.

---

## Phase 8: User Story 5 — El panel responde «¿tengo un problema?» (Priority: P2) · PR 6 (parte 1)

**Goal**: `HeroPanel` + rejilla recompuesta + «Reparto de estados» + sparkline en `DiskCard`, con carga perezosa de series.

**Independent Test**: panel con 4 discos y uno en advertencia → el héroe muestra ese disco con curva de fondo, cifra a 76 px y 2 acciones; todo en orden → héroe con el disco de sistema, «Todo en orden»; USB sin SMART en la rejilla en gris, nunca en el héroe; el panel pinta al instante y las sparklines llegan después.

### Implementación

- [x] T068 [US5] `src/lib/design/health.ts`: `selectHeroDisk(disks, alerts)` según `data-model.md` §4.2 (la regla 2 usa `isSystemVolume` leyéndolo de `disk.volumes`, sin parámetro extra). Prueba unitaria de los 4 casos + empate por `lastOccurredAt` (`health.test.ts`, +8).
- [x] T069 [US5] `src/lib/components/HeroPanel.svelte` según `contracts/componentes-nuevos.md`: `Card` alto `h-hero`, `Sparkline fill` de fondo absoluta, **velo de legibilidad** (`linear-gradient(90deg, var(--sdm-glass) 0%, var(--sdm-glass) 42%, transparent 68%)`, `pointer-events:none`), columna izquierda (píldora con icono, cifra `.sdm-display` a `text-hero`, explicación, acciones), columna derecha de 290 px con 4 hechos sobre `bg-glass-3`. Callbacks opcionales.
- [x] T070 [US5] Exportar `HeroPanel` en `src/lib/components/index.ts`.
- [x] T071 [US5] `src/lib/components/DiskCard.svelte`: recompuesta — cabecera de 52 px que hereda `--sdm-{state}-soft`, `Sparkline` de temperatura de fondo (prop nueva opcional `temperatureSeries`; sin ella, cabecera plana), cuadrado de `Icon` de bus, `StatusPill`, alias `.sdm-display`, 3 magnitudes `[Icon + etiqueta]` sobre cifra `.sdm-display`, `CapacityBar` sin cambios. Dato ausente → «—» a `text-xs` en `text-fg-dim`, texto completo en `title` (`t("common.notAvailable")`).
- [x] T072 [US5] `src/lib/stores/app.svelte.ts`: `temperatureSeries` = caché en memoria de series de temperatura por disco (evita repetir `getMetricSeries`).
- [x] T073 [US5] `src/routes/+page.svelte` + `+page.ts`: panel recompuesto — `HeroPanel` (disco de `selectHeroDisk()`) + rejilla `repeat(auto-fill, minmax(272px, 1fr))` de `DiskCard` + fila inferior `1fr 300px` con «Sucesos del sistema» (`EventRow`) y «Reparto de estados». El `load` del layout trae solo el inventario; `+page.ts` → `() => ({})`.
- [x] T074 [US5] Carga perezosa: un `$effect` pide la serie de 24 h del disco del héroe y —si `conSparklines` (≤ 12 discos)— la del resto, tras el primer render. Un fallo por disco degrada solo esa sparkline. **La `VirtualList` de la rejilla se retira** (el encuadre v3 exige una sola región de scroll); SC-006 se cubre con la variante compacta de `DiskCard` (`open-questions.md` §U).
- [x] T075 [US5] Bloque «Reparto de estados» como composición de pantalla (`Card` + `Icon` + barra de proporción + cifras), **no** componente. `HealthDonut` se mantiene en el catálogo y sale del panel.
- [x] T076 [US5] Claves i18n del PR 6 en `es.json`/`en.json`: `dashboard.hero.*`, `dashboard.spread.title`, `dashboard.events.{title,empty}`, `dashboard.noDevicesCta`.
- [x] T077 [P] [US5] Prueba de componente `src/lib/components/HeroPanel.browser.test.ts` (7): cargando / todo en orden / sin serie / protagonista sin SMART / con alerta / callbacks opcionales.
- [x] T078 [P] [US5] `src/lib/components/DiskCard.browser.test.ts` actualizada: dato ausente como «—» con `title="No disponible"`; fixtures de volumen con `isSystemVolume`.
- [x] T079 [P] [US5] e2e del panel: `e2e/ui/dashboard.spec.ts` (héroe con SMART, USB sin SMART en la rejilla, fila inferior, estado vacío). 20 discos sin tarea > 50 ms: `e2e/ui/rendimiento.spec.ts` (sin cambios, sigue verde), medición anotada en `docs/open-questions.md` §U.
- [x] T080 [US5] Checklist §8 del panel general: `regresion-visual.md` PR 6.

**Checkpoint**: la pantalla principal responde a la pregunta del usuario sin leer.

---

## Phase 9: User Story 6 — El detalle de disco da contexto a cada cifra (Priority: P2) · PR 6 (parte 2)

**Goal**: 4 `MetricCard` con icono/sparkline/procedencia, cabecera de identidad, intervalo junto a la gráfica, `DataRow` con delta.

**Independent Test**: detalle de un disco compatible → 4 `MetricCard` con icono + sparkline + procedencia; `SegmentedControl` de intervalo junto a la gráfica (no en la barra de herramientas); valor `null` como «No disponible» a `text-lg`; ninguna cifra con peso 800.

### Implementación

- [x] T081 [US6] `src/lib/components/MetricCard.svelte`: prop `icon: IconName` (por defecto `"pulse"`) + prop `series?` opcional (`Sparkline` de 22 px). `font-black` (peso 800) → `.sdm-display` (600). `value === null` → «No disponible» a `text-lg` en `fg-dim`, sin sparkline. Cuadrado de icono con `-soft` del `state` o `accent-soft`.
- [x] T082 [US6] `src/routes/disks/[id]/+page.svelte`: recompuesto — cabecera de identidad (`Card` de una fila: cuadrado de `Icon` de 56 px con color del estado, alias `.sdm-display text-2xl`, `StatusPill` con icono, línea de identidad `model · busType · maskSerial · firmware · frescura`, botón «Probar disco» → `/tests`) + 4 `MetricCard` en fila + rejilla `1.6fr 1fr` con `TimeSeriesChart` y contadores.
- [x] T083 [US6] El `SegmentedControl` + `DateRangePicker` de intervalo viven **junto a la gráfica**, dentro de `<main>`; ya no en la barra de herramientas (que perdió la ranura `controls` en PR 3). *(Corrige la mención contradictoria de T082 en la versión previa de este documento: el intervalo va con la gráfica, no en la cabecera de identidad.)*
- [x] T084 [US6] `src/lib/components/DataRow.svelte`: ya tenía columna de delta con color solo cuando significa algo (`deltaState` → `text-warn`; si no, `text-dim`). Sin cambios; la pantalla ahora la alimenta con `counter.delta`/`counter.deltaIsMeaningful`.
- [ ] ~~T085 [US6] Bloque de vida estimada~~ — **aplazado**: el contrato `DeviceDetail` no expone hoy una proyección de vida ni su ventana; añadir el cálculo en el cliente sería inventar un dato. Se retoma cuando el backend lo provea (nota en `regresion-visual.md` PR 6).
- [x] T086 [P] [US6] `src/lib/components/MetricCard.browser.test.ts` actualizada (+2): icono referenciado del sprite, sparkline con serie / sin ella cuando `value=null`.
- [x] T087 [P] [US6] e2e del detalle (`disk-detail.spec.ts`, +1): 4 métricas con etiqueta; intervalo dentro de `<main>` y **no** en la barra; iconos del sprite en el cuerpo.
- [x] T088 [US6] Checklist §8 del detalle de disco: `regresion-visual.md` PR 6.

**Checkpoint**: PR 6 completo (panel + detalle).

---

## Phase 10: User Story 7 — Pantallas secundarias en el nuevo lenguaje (Priority: P2) · PR 7

**Goal**: `ProgressBar` `emphasis`, prueba en curso con cifra a 58 px, severidad/nivel con icono, Ajustes con zona destructiva separada.

**Independent Test**: prueba en curso → bloque arriba con `ProgressBar emphasis="display"` (12 px) y cifra a 58 px; evento `error` como cuadrado de 26 px con icono y `aria-label`, fila sigue a 42 px; `AlertCard` `crit` con `#i-bolt` y `×N` en `.sdm-num`; Ajustes → «Borrado de datos» separada con borde crítico y fondo sin teñir.

### Implementación

- [x] T089 [US7] `src/lib/components/ProgressBar.svelte`: prop `emphasis?: "inline" | "display"` («inline» por defecto). `display`: alto 12 px (`h-3`), relleno `linear-gradient(90deg, var(--sdm-accent), var(--sdm-accent-hi))`, filo interior (`shadow-edge`). La cifra grande la pone la pantalla.
- [x] T090 [US7] `src/lib/components/EventRow.svelte`: el nivel pasa de píldora de texto a cuadrado de 26 px (`rounded-nav`, fondo `-soft`, `Icon` `eventLevelIcon` en el color del token, `label` con el nombre del nivel → `role="img"`). La píldora de texto se traslada al panel de detalle de `/events`. Etiqueta «asociación inferida» a `text-2xs` sobre `bg-unknown-soft` sin `font-semibold`.
- [x] T091 [US7] `src/lib/components/AlertCard.svelte`: la píldora de severidad gana icono vía `severityIcon` (nuevo mapa en `icons.ts`: `info→shield`, `warn→alert`, `crit→bolt` — `severityToHealth` no sirve porque `info→unknown→usb`); `×N` ya iba en `.sdm-num`.
- [x] T092 [US7] `src/routes/tests/+page.svelte`: bloque de prueba en curso **arriba** y a ancho completo — cuadrado de `Icon` (`testIcon`), píldora «Prueba en curso», tipo `.sdm-display text-2xl`, cifra de progreso `text-metric` (→ `text-display` en `min-[1100px]`), botón Cancelar (deshabilitado con motivo en `cancelling`); `ProgressBar emphasis="display"`; métricas en cuadros `bg-glass-3` con icono + `.sdm-display text-metric`; aviso de impacto con `Icon` en `text-warn`. Tarjetas de prueba con ranura `leading` (cuadrado de `Icon`). Historial con columna de icono de estado de 28 px. Sin prueba en curso, el bloque no se muestra.
- [x] T093 [US7] `src/lib/components/Card.svelte`: ranura `leading` + prop `border` (`hairline`|`crit`). `src/routes/settings/+page.svelte`: «Borrar todos los datos» en `Card border="crit"` dentro de un `div.mt-3` (≈ 32 px de separación sobre el `gap-5`). El resto de secciones ya iba una por `Card`; los controles ya van sobre `bg-glass-3`.
- [x] T094 [P] [US7] `alerts` / `events` / `reports`: heredan tokens; `events` gana la píldora de nivel en el detalle; `reports` cambia el badge `bg-warn text-white` por `Icon` en `text-warn`. `verify:tokens` + `a11y` (axe, claro y oscuro) verdes en las 7 pantallas.
- [x] ~~T095~~ **sin claves nuevas**: `tests.running.title` y `settings.data.dangerZone` proponían duplicar `tests.active.title` («Prueba en curso») y `settings.dangerZone.title` («Borrar todos los datos»), que ya existen y encajan. No se añaden claves muertas.
- [x] T096 [P] [US7] `ProgressBar.browser.test.ts` (+2, emphasis inline/display), `EventRow.browser.test.ts` (nuevo, 4), `AlertCard.browser.test.ts` (+1, icono + `.sdm-num`), `icons.test.ts` (+1, `severityIcon`).
- [x] T097 [P] [US7] `e2e/ui/tests.spec.ts` (+1: sin prueba en curso, bloque ausente + iconos de tarjeta). a11y de Alertas y Eventos ya cubierta por `a11y.spec.ts` (las 7 pantallas, 2 temas).
- [x] T098 [US7] Checklist §8 de Pruebas / Alertas / Eventos / Informes / Ajustes: `regresion-visual.md` PR 7.

**Checkpoint**: todas las pantallas existentes en v3.

---

## Phase 11: User Story 9 — El usuario puede volver al acento de Windows (Priority: P3) · PR 8

**Goal**: interruptor en Ajustes → Apariencia, apagado de fábrica.

**Independent Test**: instalación nueva → `useSystemAccent === false`, acento Ciruela; activar con acento de Windows claro de prueba → botón primario y texto de acento cumplen AA en los 2 temas; desactivar → vuelve el morado; persiste al reiniciar.

### Implementación

- [x] T099 [US9] Backend: valor de fábrica de `settings.appearance.use_system_accent` `true` → `false` (`get_appearance_settings_impl`, `src-tauri/src/commands/mod.rs`). +prueba: conexión sin claves → `false`; `el_acento_del_sistema_es_un_booleano_simple` cubre ahora los dos sentidos.
- [x] T100 [US9] `src/routes/settings/+page.ts` añade `getAppearanceSettings()` al `load`. `src/routes/settings/+page.svelte`: `useSystemAccent` toma el valor real del `load` (en el `$effect` que rellena `settings`). El `Switch` y `cambiarAcentoSistema` (persistir + `applySystemAccent()`/`clearSystemAccent()`) ya estaban cableados.
- [x] T101 [US9] `settings.appearance.useSystemAccent.label` / `.hint` reescritas en `es.json`/`en.json` con el texto de `RESUMEN.md` / `contracts/i18n-claves.md` (PR 8).
- [x] T102 [US9] Verificado: `applySystemAccent()`/`paint()` sobrescribe los 3 roles de acento + 2 derivados (5 propiedades CSS); `clearSystemAccent()` las restaura; `refreshAccentForTheme()` sigue enganchado en `theme.svelte.ts`. La redacción «tres tokens» del ADR-035 se precisó. **Sin cambio de código.**
- [x] T103 [P] [US9] `src/lib/design/accent.test.ts` (+2): acento claro `#ffb900` — `accessibleAccent()` (fondo) y `accentOnSurface()` sobre `LIGHT` y `DARK` (texto) dan AA.
- [x] T104 [P] [US9] `e2e/ui/settings.spec.ts` (+1): apagado de fábrica → activar persiste `value: true` + sobrescribe `--sdm-accent` con un par AA ≥ 4,5 → desactivar persiste `value: false` + quita la sobreescritura. Fixture `apariencia.useSystemAccent` → `false`.
- [x] T105 [US9] Checklist §8 de Ajustes (con el acento del sistema y con el de respaldo, 2 temas): `regresion-visual.md` PR 8.

**Checkpoint**: identidad propia por defecto, herencia disponible.

---

## Phase 12: User Story 8 — Asistente inicial funcional (Priority: P1) · PR 9

**Goal**: `/onboarding` con 4 pasos, guardián de arranque, «Omitir» en cada paso, relanzable.

**Independent Test**: con `completedAt` nulo, arrancar → redirige al asistente; recorrer los 4 pasos; excluir/renombrar un disco; «Omitir» en el paso 3 → aplica Equilibrado + graba `completedAt` + navega al panel; reiniciar → no reaparece; relanzar desde Ajustes → abre con valores actuales sin borrar nada.

### Implementación

- [x] T106 [US8] `src/routes/+layout.ts` gana un `load` (guardián): ruta ≠ `/onboarding` y `completedAt` nulo → si hay configuración previa observable (tema ≠ `system`, idioma forzado, `alerts.profile` ≠ `balanced`, algún alias, alguna exclusión) graba `completed_at` y sigue; si no, `redirect(307, "/onboarding")`. Cualquier fallo del backend → seguir normal, sin redirigir (`open-questions.md` §V). Corre en cliente (como `alerts/+page.ts`); `pnpm build` confirma que no se ejecuta al prerenderizar.
- [x] T107 [US8] `src/routes/+layout.svelte`: `esOnboarding` → renderiza solo `{@render children()}` (sin `AppShell`); el `onMount` (tema, idioma, inventario, suscripción) sigue corriendo.
- [x] T108 [US8] `src/routes/onboarding/+page.svelte` + `+page.ts`: 4 pasos, cabecera de 56 px (logo, indicador de 4 puntos que colapsa a texto por debajo de 860 px, «Omitir» siempre), `max-w-[1000px]`, pie `sticky bottom-0` con `.sdm-material-chrome`. El `+page.ts` usa `Promise.allSettled` y **no lanza**.
- [x] T109 [US8] Paso 1 · Bienvenida: cuerpo ≤ 620 px, garantía «SmartDisk solo lee…» sobre `bg-ok-soft` con `Icon` en `text-ok` (texto en color normal — a11y), 3 `Card` con `Icon` `shield`/`alert`/`flask`, primario «Buscar mis discos».
- [x] T110 [US8] Paso 2 · Discos: tarjeta por disco (casilla 24 px como portador principal, cuadrado de `Icon` de bus 40 px, modelo/metadatos, `TextField` de alias, `CapacityBar` del volumen principal, `StatusPill` de compatibilidad SMART); compatibles marcados de inicio; bloque `bg-glass-3` del USB con «no es una avería» en `<strong>`, sin rojo; pie con «Atrás», `onboarding.selectedCount`, primario «Continuar con las alertas». Al avanzar aplica `set_device_alias` / `set_device_monitoring`.
- [x] T111 [US8] Paso 3 · Alertas: 3 tarjetas de perfil (`role="radiogroup"`) desde `src/lib/design/perfiles.ts` **nuevo** (compartido con Ajustes) + `<details>` de solo lectura con la tabla real de `settings.alerts` (reacciona al cambio de perfil vía `getSettings()`); 2 `Switch` → `notifications.enabled` (encendido) y `lifecycle.start_with_system` (apagado, con `hint`).
- [x] T112 [US8] Paso 4 · Listo: cuadrado de `Icon name="check"`, resumen (nº de discos + perfil, notificaciones on/off), `ProgressBar` indeterminada + `refreshNow("all")` al entrar, primario «Ir al panel» → graba `completed_at` + `goto("/")`, nota `onboarding.done.footnote`.
- [x] T113 [US8] «Omitir» (cabecera, los 4 pasos) → `set_setting("alerts.profile", "balanced")` → `set_setting("settings.onboarding.completed_at", ahora)` → `goto("/")`.
- [x] T114 [US8] Estados: cargando (`+page.ts` bloquea, es breve); vacío (`EmptyState kind="empty"` + «Volver a buscar» + el primario del pie sirve de «continuar igualmente»); no compatible (el USB en el cuerpo); error de fuente (`+page.ts` devuelve `error`; el paso 2 muestra `EmptyState kind="error"` con `<details>` + «Reintentar»; «Omitir» de la cabecera siempre disponible).
- [x] T115 [US8] `src/routes/settings/+page.svelte`: `Card` «Repetir la configuración inicial» → `set_setting("settings.onboarding.completed_at", null)` + `goto("/onboarding")`. Además, `notifications.enabled` y `lifecycle.start_with_system` ganan su `Switch` en Ajustes (Apariencia / Ciclo de vida).
- [x] T116 [US8] 42 claves nuevas en `es.json`/`en.json` (`onboarding.*`, `settings.onboarding.repeat`, `common.back`, `common.retry`). Textos «(ver RESUMEN.md)» copiados literales.
- [x] T117 [US8] `docs/ui-contract.md` §3.1 (`notifications.enabled`, `lifecycle.startWithSystem`, nota de `completedAt` + guardián), `docs/data-model.md`, `docs/product-specification.md`, `docs/user-stories.md` (US-002 **cubierta**), `docs/decisions.md` (**ADR-037/038**), `docs/open-questions.md` §V. `pnpm docs:build` al día.
- [x] T118 [P] [US8] Estados cubiertos por e2e (`onboarding.spec.ts`) y por `a11y.spec.ts` (`/onboarding` en los 2 temas); no se hace prueba de componente del `+page.svelte` de ruta (necesita simular `$app/navigation`; el nivel e2e es más barato y demuestra lo mismo — `.claude/rules/pruebas.md`).
- [x] T119 [P] [US8] `e2e/ui/onboarding.spec.ts` (6): redirección + recorrido de los 4 pasos; «Omitir»; instalación con configuración previa → no redirige y graba; «Repetir…» desde Ajustes; paso 2 vacío; paso 2 con error de fuente. `ipc-falso.ts` refleja `set_setting("settings.onboarding.completed_at")` en `get_settings` para que el guardián no rebote.
- [x] T120 [US8] Checklist §8 del asistente: `regresion-visual.md` PR 9.

**Checkpoint**: US-002 del producto cubierta por primera vez.

---

## Phase 13: Polish & Cross-Cutting Concerns

**Propósito**: cierre transversal.

- [ ] T121 [P] Ejecutar `quickstart.md` entero sobre la app compilada — **pendiente**: requiere `pnpm app:build` y la app elevada (UAC), fuera del alcance de esta automatización. Lista de comprobación manual en `regresion-visual.md`.
- [x] T122 [P] `pnpm test:e2e` (95) y `pnpm test:a11y` (16) completos, sin regresiones. `escalado.spec.ts` (40, +5 de `/onboarding`).
- [x] T123 SC-002: `escalado.spec.ts` — sin desbordamiento horizontal en las 8 pantallas × 5 escalados, `/onboarding` incluido. SC-003: `a11y.spec.ts` (axe, 2 temas) + `accent.test.ts`. SC-006: `rendimiento.spec.ts` (20 discos, cero tareas ≥ 50 ms). SC-010: catálogo 31 componentes exportados (28 + `Icon`/`Sparkline`/`HeroPanel`), **ninguno eliminado** (`HealthDonut` sigue).
- [x] T124 [P] `verify:tokens` solo escanea `src/lib/components` + `src/routes` (`scripts/verify-tokens.mjs:22`): los bocetos HTML de `design/` no pueden hacerlo fallar. `design/propuesta-redisenov2/` se commiteó (lo citan la spec y los comentarios del código). **Archivar la v1 (`design/propuesta-rediseno/`) queda a decisión del usuario** — sin trackear por ahora.
- [x] T125 [P] `pnpm docs:build` + `docs:check` al día; `historias.md` regenerado (8832 líneas) refleja US-002, ADR-034…038 y las claves nuevas de `settings`.
- [x] T126 Sin `svelte-ignore` / `eslint-disable` / `@ts-` nuevos en el commit `45e6767`; `pnpm verify` (fronteras) verde. Nada que enlazar en `known-issues.md`.
- [x] T127 Memoria del proyecto: `rediseno-v3-estado.md` (estado, lo pendiente, los dos ficheros que no commitear). Lo demás (tokens, sprite, `selectHeroDisk`) ya vive en `ui-design.md` §0 — no se duplica en memoria.
- [ ] T128 Skill `cierre-tarea`: la documentación de todo cambio observable está hecha (ADR, `ui-contract`, `data-model`, `alert-rules`, `open-questions`, `user-stories`, `product-specification`). Queda la revisión manual del usuario (T121) antes del cierre formal de la feature.

---

## Dependencies & Execution Order

### Orden de fases (= orden de PR)

```
Setup (F1) → Foundational (F2) → US1 (F3) ─┬─ US2 (F4) ─┬─ US3 (F5)
                                           │            └─ US4 (F6) ─── US5+US6 (F8+F9)
                                           └─ US10 (F7) ─────────────── US8 (F12)
US9 (F11) depende solo de US1.
Polish (F13) depende de todo lo que se quiera entregar.
```

- **US1 (F3)** bloquea todo lo demás (tokens).
- **US2 (F4)** bloquea US3, US4, US5, US6, US7, US8 (todos usan `Icon`).
- **US4 (F6)** bloquea US5 y US6 (usan `Sparkline`).
- **US10 (F7)** bloquea US8 (el paso 3 reutiliza la lógica de perfiles). US10 da de alta `settings.onboarding.completedAt` y `isSystemVolume`, que US5 y US8 necesitan.
- **US3 (F5)** debe ir antes de US6 (la barra de herramientas pierde la ranura `controls` y el detalle mueve el intervalo).
- **US9 (F11)** es independiente de US2–US8; solo necesita US1.

### Dentro de cada historia

- Test-first **obligatorio** en toda la Fase 7 (motor de alertas y validación de esquema) — constitución §VIII.
- En componentes: modelo de props → componente → prueba de estados → integración en pantalla.
- Los tokens (T009–T011) antes que cualquier componente.
- Las claves i18n en el mismo commit que el código que las usa.

### Paralelizables

- F1: T002, T003, T004.
- F2: T005, T006, T008.
- Dentro de cada historia, las tareas `[P]` son ficheros distintos: pruebas de componente, e2e y pruebas unitarias suelen poder ir en paralelo con la implementación una vez definidas las props.
- Con equipo: tras US1+US2, un dev puede llevar US10 (backend) mientras otro lleva US3→US4 (frontend chrome+gráfica).

---

## Parallel Example: User Story 5 (panel)

```
# Tras T068–T075 (implementación):
T077  Prueba de componente de HeroPanel      (HeroPanel.browser.test.ts)
T078  Prueba de componente de DiskCard        (DiskCard.browser.test.ts)
T079  e2e del panel con 4 y 20 discos         (e2e/ui/)
# los tres tocan ficheros distintos y pueden ir a la vez
```

---

## Implementation Strategy

### MVP

**US1 + US2 + US3 + US4** (los cuatro P1 de presentación) ya entregan valor real: identidad propia, iconos, chrome que no se contradice y la gráfica arreglada. Se puede parar ahí, validar y demostrar antes de seguir con el panel y el asistente.

### Entrega incremental

Un PR por fase (F3→F12). Cada PR:

1. Deja `pnpm verify && pnpm check && pnpm test && pnpm test:component && pnpm lint` en verde (+ `cargo test` + `cargo clippy -D warnings` en F7).
2. Adjunta la checklist §8 rellenada de las pantallas que toca.
3. No rompe las fases anteriores.

**El PR 5 (F7) se aborda en modo plan** por tocar `src-tauri/` y documentación normativa, y lleva su propio ADR.

### Notas

- Nada de commits/push/PR sin pedirlo al usuario (AGENTS.md §«Límites duros»).
- Ninguna ambigüedad se resuelve en el código en silencio: si aparece una decisión no escrita, va a `docs/open-questions.md` con su valor propuesto **antes** de programarla.
- `HealthDonut` **no se elimina**. Ningún componente del catálogo se elimina.
