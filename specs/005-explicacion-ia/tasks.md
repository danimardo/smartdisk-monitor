---
description: "Task list — Explicación en lenguaje llano con IA"
---

# Tasks: Explicación en lenguaje llano con IA

**Input**: Documentos de diseño en `specs/005-explicacion-ia/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/comandos-ia.md

**Tests**: se incluyen las pruebas que la **constitución exige** (principio VIII: parsers y áreas
donde el fallo es silencioso → test-first; cobertura de estados en componentes; prueba de rechazo
por cada esquema Zod). No se añaden pruebas más allá de eso.

**Organización**: por historia de usuario. MVP = Setup + Foundational + US1 + US2.

## Formato: `[ID] [P?] [Story] Descripción con ruta`

- **[P]**: paralelizable (ficheros distintos, sin dependencias pendientes)
- **[US1]**…**[US4]**: historia de usuario de `spec.md`

**Aviso de proceso**: las tareas que tocan `src-tauri/` requieren **modo plan** antes de editar
(CLAUDE.md). Las que tocan interfaz cargan `.claude/rules/interfaz.md`; leer `docs/ui-design.md`
entero antes de una pantalla nueva.

---

## Phase 1: Setup

**Purpose**: dependencias y esqueleto de módulos.

- [X] T001 En `src-tauri/Cargo.toml`, promover `reqwest` a dependencia directa con
  `features = ["rustls", "json"]` y añadir `windows` a `[target.'cfg(windows)'.dependencies]` con
  `features = ["Win32_Security_Credentials", "Win32_Foundation"]`; `cargo build` verifica que no
  entran crates inesperados (`cargo tree -e features` antes/después).
- [X] T002 [P] Crear `specs/005-explicacion-ia/parche-constitucion.md`: parche **patch** de la
  tabla «Dependencias de Rust» de `.specify/memory/constitution.md` sustituyendo «por fijar en el
  plan» por `reqwest` (rustls, json) y `windows` (Win32_Security_Credentials); nota de que no
  requiere permiso de capabilities. Para que lo aplique el usuario (fichero con hook).
- [X] T003 [P] Crear los ficheros vacíos con su `pub mod`: `src-tauri/src/domain/ia.rs` (+ línea
  en `src-tauri/src/domain/mod.rs`), `src-tauri/src/platform/credenciales.rs` y
  `src-tauri/src/platform/ia_openrouter.rs` (+ líneas en `src-tauri/src/platform/mod.rs`).

---

## Phase 2: Foundational (bloquea todas las historias)

**Purpose**: credencial, transporte HTTP, DTO, ajustes, análisis de respuesta/error. Sin esto
ninguna historia puede empezar.

### Credencial

- [X] T004 Escribir la prueba de ida y vuelta de `platform/credenciales.rs` en
  `src-tauri/src/platform/credenciales.rs` (`#[cfg(test)]`): guardar → leer = `Some` → borrar →
  leer = `None`, con un `TargetName` de prueba con sufijo aleatorio y limpieza garantizada.
  **La prueba usa `CRED_PERSIST_SESSION`** (efímero, no exige elevación ni ensucia el almacén de
  la máquina de CI). (Constitución VIII: frontera con el SO. Ver hallazgo F1 de `analyze`.)
- [X] T005 Implementar `platform/credenciales.rs`: `guardar(clave: &str, persist: CredPersist)`,
  `leer() -> Option<String>`, `borrar()` sobre `CredWriteW`/`CredReadW`/`CredFree`/`CredDeleteW`,
  `TargetName` `"SmartDisk Monitor/OpenRouter"`, blob UTF-16LE; `ERROR_NOT_FOUND` no es error. El
  `persist` es un parámetro: producción usa `CRED_PERSIST_LOCAL_MACHINE` (el proceso va elevado,
  ADR-004), las pruebas `CRED_PERSIST_SESSION`. Hacer pasar T004.

### Transporte HTTP

- [X] T006 Implementar `platform/ia_openrouter.rs`: constantes (`ENDPOINT`, `TIMEOUT` 60 s,
  `CONNECT_TIMEOUT` 10 s, `MAX_DETALLE_CHARS` 8000), `fn cliente(clave: &str) -> reqwest::Client`
  (se construye aquí, nunca en `main`), `async fn chat_completions(clave, modelo, system, user)`
  y `async fn listar_modelos()` devolviendo el JSON crudo tipado con structs `serde` explícitos
  (principio XI). Cabeceras `Authorization`, `Content-Type`, `HTTP-Referer`/`X-Title` con el
  nombre de la app.

### DTO y contrato

- [X] T007 [P] Definir en `src-tauri/src/commands/mod.rs` (o un submódulo) los DTO con
  `#[derive(Serialize, Deserialize, TS)]` y `#[ts(export_to = "../../src/lib/api/generated/")]`:
  `EstadoIaWire`, `ModeloIaWire`, `ExplicacionIaWire` (incluye `detalle_recortado: bool` — F2),
  `RevisionAnonimizacionWire`, `SpanWire`, `AiSettingsWire`, `OrigenExplicacion`, y el `enum`
  `ResultadoExplicacion` (`#[serde(tag = "estado")]`).
- [X] T008 Ejecutar `cargo test` para regenerar `src/lib/api/generated/*` y versionar los ficheros
  nuevos; comprobar que la puerta de tipos del contrato no marca diferencia.
- [X] T009 [P] Añadir esquemas Zod de todos los DTO nuevos en `src/lib/api/schemas.ts` y su
  **prueba de rechazo** en `src/lib/api/schemas.test.ts` (constitución XI). Reexportar tipos con
  su comando real en `src/lib/api/types.ts`.

### Ajustes

- [X] T010 [P] En `src-tauri/src/domain/ajustes.rs` añadir validación pura de `settings.ai.model`
  (no vacío, ≤120, sin espacios/saltos), `settings.ai.enabled` y `settings.ai.preview_acknowledged`
  (booleanos), con sus pruebas de límite.
- [X] T011 En `src-tauri/src/commands/mod.rs`: añadir `AiSettingsWire` a `SettingsWire`, poblar
  `get_settings_impl` (defaults: `enabled=false`, `model="openrouter/free"`,
  `preview_acknowledged=false`), y las ramas de `set_setting_impl` para las tres claves.
- [X] T012 En `src-tauri/src/commands/mod.rs`: `reset_settings` — añadir `ai` al ámbito
  correspondiente y hacer que resetear `ai` ejecute también el borrado de credencial
  (equivalente a `borrar_clave_ia`).

### Dominio: análisis de respuesta y error (test-first)

- [X] T013 [P] Escribir las pruebas de `domain/ia.rs` en `src-tauri/src/domain/ia.rs` para
  `analizar_respuesta` (OK con `choices[0].message.content` + `model`; sin `choices` →
  `ia.empty_response`; JSON malformado) y `analizar_error` (401/403 → `ia.unauthorized`;
  429/402 → `ia.rate_limited`; 5xx → `ia.provider`; timeout/red). **Deben fallar.**
- [X] T014 Implementar en `src-tauri/src/domain/ia.rs` `analizar_respuesta(cuerpo) -> Result<ExplicacionIaWire, AppError>`
  y `analizar_error(estado_http, cuerpo, error_reqwest) -> AppError` con la tabla de
  `contracts/comandos-ia.md`; `detail` con el texto crudo, nunca prompt/respuesta. Hacer pasar T013.

### Comando de estado

- [X] T015 En `src-tauri/src/commands/mod.rs`: `estado_ia() -> Result<EstadoIaWire, AppError>`
  (síncrono, sin red): presencia de credencial + `settings.ai.*` + `claveValida` cacheada en
  `AppState`. Registrarlo en `src-tauri/src/lib.rs` (`generate_handler!`). Añadir el campo
  `ia_clave_valida: Mutex<Option<bool>>` a `AppState`.

### i18n base de errores

- [X] T016 [P] Añadir a `src/lib/i18n/es.json` y `src/lib/i18n/en.json` las 8 claves
  `error.ia.*` (noKey, invalidKeyFormat, unauthorized, rateLimited, timeout, network,
  emptyResponse, provider); `pnpm verify:i18n` en verde.

**Checkpoint**: credencial, transporte, DTO, ajustes y análisis listos. Las historias pueden
empezar.

---

## Phase 3: User Story 1 — Activar la ayuda con IA y gestionar la clave (Priority: P1) — MVP

**Goal**: la persona activa/prueba/borra la clave desde el asistente y desde Ajustes; la clave
vive en el Administrador de credenciales; con la función apagada no hay red.

**Independent Test**: introducir una clave en Ajustes → estado «activada», credencial creada,
clave ausente de SQLite y ficheros; clave inválida → mensaje y no se guarda; borrar → «desactivada»
y sin acciones de explicación; con la función apagada, cero red (Escenarios 1 y 2 de quickstart).

### Backend

- [X] T017 [US1] En `src-tauri/src/commands/mod.rs`: `guardar_clave_ia(clave: String) -> Result<EstadoIaWire, AppError>`
  (async): validar forma → llamada de validación (`listar_modelos` con `Authorization`) →
  401/403 no guarda (`ia.unauthorized`), red/timeout no guarda, 2xx/429/402 válida →
  `credenciales::guardar` + `settings.ai.enabled=true` + `claveValida` en `AppState`.
- [X] T018 [US1] En `src-tauri/src/commands/mod.rs`: `borrar_clave_ia() -> Result<EstadoIaWire, AppError>`
  (síncrono): `credenciales::borrar` + `settings.ai.enabled=false` +
  `settings.ai.preview_acknowledged=false`; conserva `settings.ai.model`.
- [X] T019 [US1] En `src-tauri/src/commands/mod.rs`: `probar_clave_ia() -> Result<EstadoIaWire, AppError>`
  (async): valida la clave guardada sin cambiarla; sin credencial → `ia.no_key`.
- [X] T020 [US1] Registrar los tres comandos en `src-tauri/src/lib.rs` y añadir los eventos
  `tracing` `info` «función activada/desactivada» (sin contenido).

### Frontend

- [X] T021 [P] [US1] En `src/lib/api/client.ts` + `src/lib/api/index.ts`: envoltorios
  `estadoIa()`, `guardarClaveIa()`, `borrarClaveIa()`, `probarClaveIa()` con validación Zod de la
  respuesta.
- [X] T022 [P] [US1] Crear `src/lib/stores/ia.svelte.ts`: clase con runes `{ estado, enCurso:Set }`,
  método `refrescar()` que llama `estadoIa()`. Sin store clásico, sin `localStorage`.
- [X] T023 [US1] Añadir a `src/routes/settings/+page.ts` la carga de `estadoIa()` en el `load`
  (junto a `getSettings()`).
- [X] T024 [US1] Crear la sección «Ayuda con IA» en `src/routes/settings/+page.svelte`: estado
  (activada/desactivada), `TextField` para la clave, botones «Activar»/«Probar»/«Borrar»,
  mensajes de `AppError`. Usa `ConfirmDialog` para «Borrar». Tokens y utilidades; sin literales.
- [X] T025 [US1] Añadir el **5.º paso** «Ayuda con IA» a `src/routes/onboarding/+page.svelte`
  (`TOTAL = 5`, `ProgressBar` y textos de paso): explicación + ejemplo de lo que se envía +
  `TextField` + «Activar»/«Omitir». Reutiliza `guardarClaveIa()`.
- [X] T026 [P] [US1] Claves i18n de la sección de Ajustes y del paso del asistente en
  `src/lib/i18n/es.json` y `en.json` (~15 claves).
- [X] T027 [US1] (diferida a US2, cuando se monte la infra de pruebas de componente del modal) Prueba de componente en
  `src/lib/components/…` / ruta de test correspondiente: sección de Ajustes en estados
  desactivada / activando / activada / probando / error (constitución VIII, cobertura de estados).

**Checkpoint**: US1 funcional e independientemente testable. La función se activa y se apaga; sin
clave no hay red.

---

## Phase 4: User Story 2 — Entender una alerta sin ser técnico (Priority: P1) — MVP

**Goal**: en el detalle de una alerta, «Explícamelo» → vista previa la primera vez → progreso →
modal con markdown, modelo y advertencia; anonimización y revisión de texto libre; los fallos no
rompen la pantalla.

**Independent Test**: con clave activa, abrir una alerta, pulsar la acción, confirmar la vista
previa, recibir el modal en markdown; simular `ia.network` y comprobar que `/alerts` sigue
(Escenarios 3, 4, 5, 7, 9 de quickstart).

### Dominio (test-first)

- [X] T028 [P] [US2] Escribir pruebas de `domain/ia.rs` en `src-tauri/src/domain/ia.rs` para:
  composición del prompt (system+user, idioma, contexto del disco correcto y **sin**
  serie/alias/labels); `aplicar_anonimizador` sobre todos los fragmentos (0 fugas sobre fixtures
  con serie + ruta de usuario); `barrer_texto_residual` (rutas `X:\`, UNC, tokens tipo serie) con
  fixtures de descripciones de eventos de Windows. **Deben fallar.**
- [X] T029 [US2] Implementar en `src-tauri/src/domain/ia.rs`: `componer_consulta(origen, contexto_disco, detalle_tecnico, idioma)`,
  `aplicar_anonimizador` (reutiliza `reporting::anonimizar::Anonimizador::para_esta_maquina` +
  serie del disco + labels de volumen), `barrer_texto_residual(texto) -> Vec<SpanWire>`,
  `recortar(MAX_DETALLE_CHARS) -> (String, bool)` devolviendo si hubo recorte (F2, se propaga a
  `ExplicacionIaWire.detalle_recortado`). Hacer pasar T028.

### Comando

- [X] T030 [US2] En `src-tauri/src/commands/mod.rs`: `explicar_detalle_tecnico(origen: OrigenExplicacion) -> Result<ResultadoExplicacion, AppError>`
  (async), **rama `tipo:"alerta"`**: sin credencial → `ia.no_key`; revalidar `origen` contra
  alertas/inventario (`ipc.schema_mismatch`); recomponer detalle desde `get_alert_detail` +
  `get_alert_smart_raw_json`; anonimizar; `barrer_texto_residual` + lógica de `revision`;
  vista previa si `!preview_acknowledged && !previewConfirmada`; si `previewConfirmada` →
  `settings.ai.preview_acknowledged=true`; llamar `ia_openrouter::chat_completions`;
  `analizar_respuesta`. Evento `tracing` `debug` con `modelo_solicitado`, `modelo_usado`
  (no identificable, F8), `resultado`, `ms` — nunca prompt ni respuesta. Registrar en
  `src-tauri/src/lib.rs`.

### Markdown (catálogo)

- [X] T031 [P] [US2] Escribir `src/lib/design/markdown.test.ts`: cada bloque del subconjunto,
  anidamiento simple, y que `<img onerror>`, `<script>`, `[x](javascript:...)` salen como texto
  inerte.
- [X] T032 [US2] Implementar `src/lib/design/markdown.ts`: analizador de subconjunto
  (encabezados `#`–`###`, párrafos, listas `-`/`1.`, código en bloque y en línea, cita, negrita,
  cursiva, enlace) → árbol de tokens tipado. Hacer pasar T031.
- [X] T033 [US2] Crear `src/lib/components/Markdown.svelte`: render del árbol con **solo marcado
  Svelte**, sin `{@html}`; enlaces como texto + URL entre paréntesis (no navegables). Añadir a
  `src/lib/components/index.ts`.
- [X] T034 [US2] Prueba de componente de `Markdown.svelte`: render correcto y payloads hostiles
  inertes.
- [X] T035 [US2] Anotar en `docs/ui-design.md` §3 (catálogo) la incorporación de `Markdown.svelte`
  con su criterio, o registrar la excepción según el proceso de `AGENTS.md` §3.

### Modal y pantalla

- [X] T036 [P] [US2] Crear `src/lib/components/ExplicacionModal.svelte`: `role="dialog" aria-modal`,
  foco atrapado, `Escape`, estados **progreso** (`aria-live`, con botón **Cancelar** que descarta
  el resultado en curso — FR-019, hallazgo F3), **error** (`AppError` con detalle copiable y
  «Reintentar» si `retryable`), **ok** (`Markdown` + modelo usado + nota «orientación por IA, no
  un dato de salud»; si el detalle se recortó, aviso «se envió un extracto» — FR-021, hallazgo F2).
  Basado en `.sdm-material-overlay` como `ConfirmDialog`. Añadir a `index.ts`.
- [X] T037 [US2] Crear el diálogo de **vista previa / revisión** (puede ser un modo de
  `ExplicacionModal` o `ConfirmDialog` extendido): muestra `textoCompleto`, resalta `spans`, y da
  «Enviar igual» / «Quitar fragmento» / «Cancelar» cuando hay spans; «Confirmar» / «Cancelar»
  cuando es solo vista previa (`spans` vacío).
- [X] T038 [US2] En `src/routes/alerts/+page.svelte`: botón «Explícamelo en lenguaje claro» en el
  detalle (visible solo si `iaStore.estado?.activa`), que orquesta
  `explicarDetalleTecnico({tipo:"alerta", …})` → vista previa/revisión → progreso → modal;
  usa `iaStore.enCurso` para bloquear el doble envío (FR-020).
- [X] T039 [P] [US2] Envoltorio `explicarDetalleTecnico()` en `src/lib/api/client.ts` con
  validación Zod del `ResultadoExplicacion` (union por `estado`).
- [X] T040 [P] [US2] Claves i18n del botón, el modal, la vista previa y el diálogo de revisión en
  `src/lib/i18n/es.json` y `en.json` (~12 claves).

### e2e

- [X] T041 [US2] Crear `e2e/ui/ia.spec.ts`: con doble del comando (o clave de prueba), activar →
  explicar una alerta → confirmar vista previa → ver modal; simular `ia.network` → `/alerts` sigue
  viva; respuesta hostil → texto inerte y **assert de SC-007** (ningún color de estado ni alerta
  cambia); tras «reiniciar» (recarga) la vista previa **no** reaparece (**assert de SC-009**).
  (Hallazgo F4.)
- [X] T041a [US2] Prueba en `cargo test`: `explicar_detalle_tecnico` y `listar_modelos_ia`, con
  `credenciales::leer() == None`, devuelven `ia.no_key` **antes** de cualquier construcción de
  cliente HTTP (SC-001 / FR-005, hallazgo F4). Como esos comandos aún no existen en Foundational,
  la prueba se escribe con ellos, en US2/US3. En Foundational basta la garantía estructural: el
  `reqwest::Client` solo se construye dentro de `platform::ia_openrouter`, nunca en el arranque
  (`lib.rs`) ni en `estado_ia` — verificable con `grep`.

**Checkpoint**: US1 + US2 funcionan. MVP completo: se puede activar la ayuda y explicar una alerta.

---

## Phase 5: User Story 3 — Elegir el modelo de IA (Priority: P2)

**Goal**: selector con «automático» por defecto + lista del proveedor; aviso y confirmación al
elegir un modelo de pago; funciona sin lista disponible.

**Independent Test**: abrir el selector en Ajustes, ver «automático» + lista; elegir uno de pago →
aviso; sin red → seguir con «automático» (Escenario 6 de quickstart).

- [X] T042 [P] [US3] Prueba de `domain/ia.rs` para `modelo_es_de_pago(pricing) -> bool`
  (`prompt != "0" || completion != "0"`), con casos gratis / pago / campos ausentes. **Debe fallar.**
- [X] T043 [US3] Implementar `modelo_es_de_pago` en `src-tauri/src/domain/ia.rs` y el mapeo
  `respuesta /models → Vec<ModeloIaWire>` (antepone `openrouter/free`, ordena
  automático→gratis→pago). Hacer pasar T042.
- [X] T044 [US3] En `src-tauri/src/commands/mod.rs`: `listar_modelos_ia() -> Result<Vec<ModeloIaWire>, AppError>`
  (async, `GET /models` sin `Authorization`). Registrar en `src-tauri/src/lib.rs`.
- [X] T045 [P] [US3] Envoltorio `listarModelosIa()` en `src/lib/api/client.ts` con validación Zod.
- [X] T046 [US3] En `src/routes/settings/+page.svelte`: `Select` de modelo en la sección «Ayuda
  con IA» (cargado al abrirlo); al elegir un `esDePago` → `ConfirmDialog` con impacto «puede
  generar cargos en tu cuenta de OpenRouter»; solo tras confirmar `setSetting("settings.ai.model", …)`.
  Si la lista falla, dejar «automático» y avisar.
- [X] T047 [US3] Añadir el mismo `Select` de modelo al 5.º paso del asistente en
  `src/routes/onboarding/+page.svelte` (visible tras activar la clave).
- [X] T048 [P] [US3] Claves i18n del selector y del aviso de coste en `src/lib/i18n/es.json` y
  `en.json` (~6 claves).
- [X] T049 [US3] Prueba de componente del selector: «automático» por defecto, lista, aviso de
  coste, lista no disponible.

**Checkpoint**: US1 + US2 + US3.

---

## Phase 6: User Story 4 — Explicar el detalle SMART de un disco (Priority: P2)

**Goal**: la misma acción «Explícamelo» en la tarjeta de contadores SMART de `/disks/[id]`.

**Independent Test**: abrir el detalle de un disco con la función activada → «Explícamelo» en la
tarjeta de contadores → modal con la misma mecánica; con la función apagada, sin botón
(Escenario 8 de quickstart).

- [X] T050 [P] [US4] Prueba de `domain/ia.rs` para la rama `tipo:"smart"` de `componer_consulta`
  (usa la lista de `SmartCounter` visibles, contexto del disco, sin serie/alias). **Debe fallar.**
- [X] T051 [US4] Extender `componer_consulta` y `explicar_detalle_tecnico` en
  `src-tauri/src/domain/ia.rs` y `src-tauri/src/commands/mod.rs` con la rama `tipo:"smart"`
  (recompone desde `get_device_detail` / contadores). Hacer pasar T050.
- [X] T052 [US4] En `src/routes/disks/[id]/+page.svelte`: botón «Explícamelo en lenguaje claro» en
  la `Card` de contadores (visible solo si `iaStore.estado?.activa`), reutilizando
  `ExplicacionModal` y la orquestación de US2.
- [X] T053 [P] [US4] Claves i18n específicas de la vista SMART si hacen falta en
  `src/lib/i18n/es.json` y `en.json`.
- [X] T054 [US4] Extender `e2e/ui/ia.spec.ts` con el caso del detalle SMART.

**Checkpoint**: las cuatro historias funcionan.

---

## Phase 7: Polish y cierre (cross-cutting)

**Purpose**: documentación normativa, constitución, verificación completa.

- [X] T055 [P] Actualizar `docs/ui-contract.md`: los 6 comandos, sus DTO y los `error.ia.*`.
- [X] T056 [P] Actualizar `docs/architecture.md`: `domain/ia.rs`, `platform/ia_openrouter.rs`,
  `platform/credenciales.rs`; regla «la red solo la origina Rust, bajo gesto explícito».
- [X] T057 [P] Actualizar `docs/data-model.md`: grupo `ai` de `settings`, credencial fuera de
  SQLite, entidades efímeras.
- [X] T058 [P] Actualizar `docs/product-specification.md`: la capacidad opcional en el alcance.
- [X] T059 [P] Actualizar `docs/engineering-conventions.md`: `reqwest` (rustls, json) y `windows`
  (Win32_Security_Credentials) como dependencias, con su justificación (ADR-046).
- [X] T060 [P] Actualizar `docs/open-questions.md` con las decisiones adoptadas: tope 60 s,
  `CRED_PERSIST_LOCAL_MACHINE`, `reset` de `ai` = borrar credencial, subconjunto de markdown, sin
  streaming en v1, `MAX_DETALLE_CHARS = 8000` (marcar `PROPUESTO`).
- [X] T061 Actualizar **ADR-046** en `docs/decisions.md`: fijar `reqwest`+`rustls` y
  `windows`/`Win32_Security_Credentials` como elección; anotar «sin permiso de capabilities porque
  la llamada es desde Rust».
- [X] T062 Pedir al usuario que aplique `specs/005-explicacion-ia/parche-constitucion.md` a
  `.specify/memory/constitution.md` (versión 1.8.1, patch); verificar el resultado.
- [X] T063 `pnpm docs:build` para regenerar `historias.md`; confirmar `pnpm docs:check`.
- [X] T064 [P] Registrar en `docs/known-issues.md` cualquier `svelte-ignore` / `eslint-disable`
  introducido (p. ej. en el modal), con enlace desde el código.
- [X] T065 `pnpm test:a11y` sobre `ExplicacionModal`, la sección de Ajustes y el paso del
  asistente; corregir hasta cero incidencias.
- [X] T066 Ejecutar la batería completa: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `pnpm check`, `pnpm test`, `pnpm test:component`, `pnpm test:e2e`, `pnpm test:a11y`,
  `pnpm verify`, `pnpm lint`. Todo en verde.
- [ ] T067 Recorrer `specs/005-explicacion-ia/quickstart.md` (9 escenarios) en `pnpm app:dev` con
  una clave real; anotar resultados.
- [ ] T068 Revisión humana de pantalla (`AGENTS.md` §8) para el modal, la sección de Ajustes y el
  5.º paso: tema claro/oscuro, acento propio y del sistema, 1024×560 y 1280×720, escalado
  125/150/200 %, estados, teclado y foco, `es` y `en`.
- [X] T069 Skill `cierre-tarea`: matriz de documentación y las nueve puertas de verificación antes
  de proponer el commit.

---

## Dependencias y orden

### Fases

- **Setup (P1)**: sin dependencias.
- **Foundational (P2)**: depende de Setup. **Bloquea** todas las historias.
- **US1 (P3)** y **US2 (P4)**: dependen de Foundational. US2 no depende de US1 en código, pero para
  la demo conviene US1 primero (activar la clave). Comparten `iaStore` y `ExplicacionModal` no —
  US1 no usa el modal.
- **US3 (P5)**: depende de Foundational; toca la sección de Ajustes (US1) y el paso del asistente
  (US1) → mejor después de US1.
- **US4 (P6)**: depende de Foundational; reutiliza `ExplicacionModal` y la orquestación de US2 →
  después de US2.
- **Polish (P7)**: después de las historias que se vayan a entregar.

### Dentro de cada historia

- Las pruebas test-first (constitución VIII) van **antes** de su implementación: T004→T005,
  T013→T014, T028→T029, T031→T032, T042→T043, T050→T051.
- Modelos/dominio antes que comandos; comandos antes que interfaz.

### Oportunidades de paralelismo

- Setup: T002 y T003 en paralelo (T001 primero).
- Foundational: T007, T009, T010, T013, T016 en paralelo tras T003; T004→T005 y T006 en paralelo
  con ellas.
- US1: T021, T022, T026 en paralelo; T024/T025 tras T021+T022.
- US2: T031→T032→T033 (cadena markdown) en paralelo con T028→T029 (dominio) y T036 (modal);
  T038 al final, cuando modal + orquestación + comando están.
- Polish: T055–T060 y T064 en paralelo.

---

## Parallel Example: Foundational

```text
# Tras T003, lanzar en paralelo:
T007  Definir DTO ts-rs en src-tauri/src/commands/mod.rs
T009  Esquemas Zod + prueba de rechazo en src/lib/api/schemas.ts
T010  Validación de settings.ai.* en src-tauri/src/domain/ajustes.rs
T013  Pruebas de analizar_respuesta/analizar_error en src-tauri/src/domain/ia.rs
T016  Claves error.ia.* en src/lib/i18n/es.json y en.json
# En paralelo con T004→T005 (credencial) y T006 (transporte).
```

---

## Implementation Strategy

### MVP (US1 + US2)

1. Phase 1: Setup.
2. Phase 2: Foundational (crítico).
3. Phase 3: US1 → validar el Escenario 2 del quickstart.
4. Phase 4: US2 → validar los Escenarios 3, 4, 5, 7, 9.
5. **Parar y validar**: la ayuda se activa y explica una alerta, con anonimización y sin romperse.

### Entrega incremental

1. Setup + Foundational → base lista.
2. + US1 → activar/gestionar clave (demo).
3. + US2 → explicar alerta (**MVP**).
4. + US3 → elegir modelo.
5. + US4 → explicar detalle SMART.
6. Phase 7 → documentación normativa, constitución (patch), verificación completa y cierre.

---

## Notas

- Toda tarea de `src-tauri/` exige **modo plan** antes de editar (CLAUDE.md).
- Toda tarea de interfaz: leer `docs/ui-design.md` y pasar su §8 antes de dar por terminada.
- Cero valores visuales literales, cero literales de interfaz, cero `console.*`/`println!`.
- El prompt y la respuesta del modelo **nunca** al log ni a un documento.
- Commit al cerrar cada fase o grupo lógico; no antes de pedirlo al usuario (límite duro).
