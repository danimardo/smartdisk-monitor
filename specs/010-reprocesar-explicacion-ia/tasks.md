---
description: "Task list — Reprocesar la explicación con IA con otro modelo gratuito (010)"
---

# Tasks: Reprocesar la explicación con IA con otro modelo gratuito

**Input**: `specs/010-reprocesar-explicacion-ia/` (plan.md, spec.md, research.md, data-model.md,
contracts/comandos-ia.md, quickstart.md)

**Incremento sobre las specs 005-explicacion-ia / 006-explicacion-ia-contexto-crudo** (ya
fusionadas). No hay proyecto que inicializar; se tocan los mismos módulos.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ir en paralelo (fichero distinto, sin dependencias pendientes)
- **[Story]**: US1 / US2 / US3 / US4 (mapea a las historias de `spec.md`)
- Rutas exactas en cada tarea. Comandos de Rust desde `src-tauri/`.

**Tests**: la constitución §VIII exige cobertura y trata como zona de riesgo silencioso cualquier
sitio donde "el fallo no revienta, produce un dato equivocado que alguien se cree" —
`.claude/rules/pruebas.md`. Sustituir el modelo por el elegido es exactamente ese riesgo (si el
override no llega, la app sigue usando el modelo de siempre sin que nadie lo note), así que sus
tests van **test-first** en Foundational. El resto de tests (componente, store) acompañan a su
historia.

---

## Phase 1: Setup

**Purpose**: dejar la base verde y las claves i18n preparadas antes de tocar lógica.

- [X] T001 Confirmar baseline verde antes de empezar: `cargo test` (desde `src-tauri/`) y, desde la
      raíz, `pnpm check && pnpm lint && pnpm verify`. Anotar cualquier fallo previo no relacionado
      con esta feature. **Hecho**: `pnpm check`/`pnpm verify` en verde. Dos fallos preexistentes sin
      relación, no tocados: `cargo test` — `reporting::export::tests::una_muestra_entera_sin_valor_real_se_exporta_por_su_valor_entero`
      (ya conocido, spec 009); `pnpm lint` — Prettier sobre dos ficheros de `.claude/tmp/` (scratch
      sin versionar, no es código fuente).
- [X] T002 [P] Añadir a `src/lib/i18n/es.json` y `src/lib/i18n/en.json` las claves nuevas (texto
      borrador, se afina en cada historia): `ai.modal.reprocess.invite`,
      `ai.modal.reprocess.modelLabel`, `ai.modal.reprocess.action`,
      `ai.modal.reprocess.historyEntry` (con marcador `{model}`), `ai.modal.reprocess.setDefault`,
      `ai.modal.reprocess.setDefaultDone`, `ai.modal.reprocess.setDefaultError`,
      `settings.ai.model.paidDisabledDemo`. Evita que `pnpm verify:i18n` falle a mitad de las
      historias siguientes. **Hecho**: texto final escrito directamente (no quedó borrador que
      afinar); `pnpm verify:i18n` en verde (605 claves).

**Checkpoint**: `cargo test` y `pnpm verify` en verde.

---

## Phase 2: Foundational (prerequisito bloqueante)

**Purpose**: las piezas de bajo nivel que más de una historia necesita: el campo nuevo en el
contrato IPC (backend), el soporte de opción deshabilitada en el selector (frontend), y que
`AiModelSelect` pueda excluir el modo automático (lo necesitan US1 y US4 por separado, cada una con
su propio criterio). **Bloquea US1 y US4**; US2 y US3 dependen a su vez de US1.

**⚠️ Ninguna historia puede empezar hasta cerrar esta fase.**

### Tests primero (test-first, constitución §VIII)

- [X] T100 [P] En `src-tauri/src/domain/ia.rs` (`#[cfg(test)]`), test que FALLA:
      `OrigenExplicacion` deserializa correctamente tanto con `modelo_solicitado` presente
      (`Some("vendor/x:free".into())`) como ausente (`None`), sin romper la deserialización de un
      payload de la 006 que no conoce el campo. **Hecho**: `origen_explicacion_deserializa_con_y_sin_modelo_solicitado`.
- [X] T101 [P] En `src-tauri/src/commands/mod.rs` (`#[cfg(test)]`, junto a los tests existentes de
      `reunir_datos_explicacion`), test que FALLA: con `origen.modelo_solicitado = Some("vendor/x:free")`,
      `reunir_datos_explicacion` devuelve `datos.modelo == "vendor/x:free"` **sin leer**
      `settings.ai.model`; con `None`, se comporta exactamente como hoy (lee el ajuste). El campo se
      lee una sola vez, antes del `match origen.tipo` (ver `commands/mod.rs`, `reunir_datos_explicacion`),
      así que basta un test con **una** de las tres ramas existentes para cubrir las tres — no hace
      falta repetirlo por rama. Usar la fixture `origen_alerta` ya existente (con
      `crear_grupo_con_regla`), que es la que menos montaje necesita. **Hecho**: dos tests,
      `reunir_datos_usa_el_modelo_solicitado_sin_leer_el_ajuste` y
      `reunir_datos_sin_modelo_solicitado_sigue_leyendo_el_ajuste` (regresión).
- [X] T102 [P] En `src/lib/api/schemas.test.ts`, prueba de rechazo (`.claude/rules/frontera-ipc.md`):
      el esquema Zod de `origenExplicacion` (línea ~402) sigue la convención de `deviceId`/
      `alertGroupId`/`eventId` (`z.string().nullable()`, campo obligatorio): acepta
      `modeloSolicitado: "vendor/x:free"` y `modeloSolicitado: null`; rechaza `modeloSolicitado: 123`
      y rechaza el objeto si la clave falta por completo. Actualizar también los tres fixtures ya
      existentes en ese fichero (líneas ~437-474: "acepta el tipo evento", "rechaza eventId ausente",
      "rechaza un tipo que no existe") añadiéndoles `modeloSolicitado: null`, para que sigan
      pasando/fallando por el motivo que dicen y no por esta clave nueva. **Hecho**.
- [X] T103 [P] En `src/lib/components/Select.browser.test.ts`, test que FALLA: una opción con
      `disabled: true` se renderiza como `<option disabled>` y no se puede seleccionar por teclado
      ni por `selectOptions`; una opción sin `disabled` (o `disabled: false`) se comporta como hoy.
      **Hecho**.
- [X] T104 [P] En `AiModelSelect.browser.test.ts`, nuevo test: con `incluirAutomatico={false}` las
      opciones no incluyen `openrouter/free`; con el prop omitido (u `true`), se sigue incluyendo
      (comportamiento actual de Ajustes, sin regresión). **Hecho**.

### Implementación

- [X] T110 [P] En `src-tauri/src/domain/ia.rs`, añadir `pub modelo_solicitado: Option<String>` a
      `OrigenExplicacion` (mismo `#[serde(rename_all = "camelCase")]` del struct, así que en TS es
      `modeloSolicitado`). Regenerar el tipo espejo (`cargo test` ejecuta `ts-rs`) y confirmar que
      `src/lib/api/generated/OrigenExplicacion.ts` incluye el campo opcional. Hace pasar T100.
      **Hecho** — descubierto al regenerar: `ts-rs` representa `Option<String>` como `string | null`
      (campo **obligatorio**, no opcional), igual que `deviceId`/`alertGroupId`/`eventId` — no como
      `string | undefined`. Corregido en `data-model.md`/`contracts/comandos-ia.md` y en T102/T112.
      También hubo que añadir `modelo_solicitado: None` a las fixtures `origen_alerta`/`origen_evento`
      de `commands/mod.rs` (struct literal, todos los campos son obligatorios en Rust).
- [X] T111 En `src-tauri/src/commands/mod.rs`, en `reunir_datos_explicacion`: sustituir la lectura
      fija `let modelo = leer_ajuste_string(conn, "settings.ai.model", ia::MODELO_AUTOMATICO);` por
      `origen.modelo_solicitado.clone().unwrap_or_else(|| leer_ajuste_string(conn, "settings.ai.model", ia::MODELO_AUTOMATICO))`.
      Hace pasar T101. No se persiste nada nuevo: el ajuste de `settings` no se toca aquí. **Hecho**.
- [X] T112 [P] En `src/lib/api/schemas.ts`, añadir `modeloSolicitado: z.string().nullable()` al
      esquema `origenExplicacion` (mismo patrón que `deviceId`). Hace pasar T102. **Hecho**.
- [X] T113 [P] En `src/lib/components/Select.svelte`, ampliar el tipo de `options` a
      `{ id: string; label: string; disabled?: boolean }[]` y pasar `disabled={opt.disabled}` a cada
      `<option>`. Hace pasar T103. No cambia el comportamiento de ningún uso existente (todas las
      llamadas actuales omiten el campo). **Hecho**.
- [X] T114 En `docs/ui-contract.md` §3.10, documentar `modeloSolicitado` en el tipo
      `OrigenExplicacion` del contrato (copiar la nota de `contracts/comandos-ia.md`). **Hecho**
      (tipo actualizado a `string | null` + nota explicando el uso).
- [X] T115 [P] En `src/lib/components/AiModelSelect.svelte`, añadir prop `incluirAutomatico =
      true`; en `opciones`, omitir la entrada "automático" cuando sea `false`. Sin lógica de
      deshabilitado por clave todavía — eso sigue en US4 (T510). Hace pasar T104. **Hecho**.

**Checkpoint**: `cargo test` y `pnpm test` en verde; un `OrigenExplicacion` con `modeloSolicitado`
cambia el modelo de la llamada sin tocar `settings`; `Select` acepta opciones deshabilitadas;
`AiModelSelect` puede excluir el modo automático. **Verificado.**

---

## Phase 3: User Story 1 - Probar otro modelo gratuito sobre la misma pregunta (Priority: P1) 🎯 MVP

**Goal**: desde una respuesta o un error del modal de ayuda con IA, elegir otro modelo gratuito y
reprocesar la misma petición sin repetir el gesto original.

**Independent Test**: pedir una explicación, elegir un modelo gratuito distinto del que respondió
en el selector nuevo, pulsar reprocesar, y ver una respuesta nueva sin cerrar el modal — tanto tras
un éxito como tras un error.

### Tests para User Story 1

- [X] T200 [P] [US1] En un nuevo `src/lib/stores/explicacion.svelte.test.ts` (jsdom, mock de
      `$lib/api` con `vi.mock`, patrón de `AiModelSelect.browser.test.ts` pero para lógica, no
      componente): `reprocesar("vendor/y:free")` tras una respuesta en fase `resultado` llama a
      `explicarDetalleTecnico` con el mismo `origen` (mismos `tipo`/`alertGroupId`/`deviceId`/
      `eventId`/`idioma`) y `modeloSolicitado: "vendor/y:free"`, y dobla por
      `previewConfirmada: true` (mismo camino que `reintentar`). Cubre también FR-004: si antes del
      reproceso la sesión ya resolvió una revisión (simular una `quitarFragmentos()` previa, que deja
      `revision: "quitar_fragmentos"`), el payload del reproceso conserva ese mismo `revision` — no
      vuelve a `"ninguna"`. **Hecho**.
- [X] T201 [P] [US1] En el mismo fichero: `reprocesar()` disponible también en fase `error` —
      llamar tras un rechazo de `explicarDetalleTecnico` debe reintentar con el modelo nuevo, no con
      el que falló. **Hecho**.
- [X] T202 [P] [US1] En `src/lib/components/ExplicacionModal.browser.test.ts`: mockeando
      `listarModelosIa` a nivel de módulo (mismo patrón que `AiModelSelect.browser.test.ts`, ya que
      el selector interno se resuelve vía `AiModelSelect`), en fase `resultado` se ven el mensaje
      `ai.modal.reprocess.invite`, un `combobox` y un botón `ai.modal.reprocess.action`; al elegir un
      modelo y pulsar reprocesar se dispara el callback `onreprocesar` con ese id. Repetir la
      comprobación en fase `error`. **Hecho** (incluida en la fase `error` con `retryable: false`,
      para probar a propósito que el reproceso se ofrece aunque "Reintentar" no aparezca).
- [X] T203 [P] [US1] En `src/lib/stores/explicacion.svelte.test.ts` (FR-013): cerrar el modal
      (`cerrar()`) mientras un reproceso está en curso (fase `progreso`) pone `open = false`; cuando
      la promesa pendiente de `explicarDetalleTecnico` resuelve después de cerrado, no reabre el
      modal ni pisa el estado de una explicación distinta lanzada mientras tanto con `lanzar()`
      (guarda contra una respuesta tardía de un reproceso ya cancelado). **Hecho** — este test reveló
      una condición de carrera real y preexistente en `#pedir` (no introducida por esta feature, pero
      mucho más alcanzable ahora que las llamadas duran más por poder reprocesar): `this.#clave` es
      una propiedad computada sobre `#origen` mutable, así que si se lanza una explicación distinta
      mientras la anterior sigue en vuelo, el `finally` de la llamada vieja libera la clave de la
      **nueva**, y una respuesta tardía pisaría el estado de la sesión nueva. Corregido en T210
      capturando la clave al principio de `#pedir` (ver nota ahí) — necesario para que este test
      pase, no es una tarea aparte.

### Implementación

- [X] T210 [US1] En `src/lib/stores/explicacion.svelte.ts`: añadir campo privado
      `#modeloOverride: string | undefined`, incluir `modeloSolicitado: this.#modeloOverride ?? null`
      en el payload de `explicarDetalleTecnico` dentro de `#pedir` (el tipo generado exige la clave
      presente, `string | null`, igual que `deviceId`), y el método público
      `reprocesar(modelo: string): Promise<void>` que fija `#modeloOverride = modelo` y llama a
      `this.#pedir(true)` (mismo camino que `reintentar`, reutilizando el `#revision` ya resuelto de
      la sesión). El tipo `Origen` (`Omit<OrigenExplicacion, "revision" | "previewConfirmada">`)
      amplía el `Omit` para incluir también `"modeloSolicitado"`: es un campo que solo gestiona este
      store, nunca quien llama a `lanzar()`. Hace pasar T200/T201. **Hecho.** Además (necesario para
      T203/FR-013): `#pedir` captura `const clave = this.#clave` al principio y la usa en
      `marcarEnCurso`/`liberar` y para descartar en silencio (`if (this.#clave !== clave) return`)
      una respuesta que llegue cuando `#origen` ya ha cambiado a una explicación distinta.
- [X] T211 [US1] En `src/lib/components/ExplicacionModal.svelte`: en las fases `resultado` y
      `error`, añadir el mensaje `t("ai.modal.reprocess.invite")`, un
      `<AiModelSelect incluirAutomatico={false} modelo={modeloElegido} onchange={(id) => (modeloElegido = id)} />`
      (mismo componente que Ajustes, con la prop `incluirAutomatico` ya añadida en T115 — self-fetch
      de su propio catálogo, sin prop `modelos` en este componente) y un
      `<Button variant="secondary">{t("ai.modal.reprocess.action")}</Button>` que llama a
      `onreprocesar?.(modeloElegido)`. Nuevo prop: `onreprocesar: ((modelo: string) => void) |
      undefined`. Hace pasar T202. **Hecho** — `modeloElegido` empieza vacío a propósito (estado
      local del componente) y el botón se deshabilita mientras no haya elección explícita: evita
      tener que sincronizar un valor inicial con `modeloUsado` en cada respuesta nueva.
- [X] T212 En los tres sitios que montan `ExplicacionModal` (`src/routes/alerts/+page.svelte`,
      `src/routes/disks/[id]/+page.svelte` y `src/routes/events/+page.svelte` — este último no
      estaba en el plan original, se detectó al implementar), conectar el nuevo prop `onreprocesar`
      a `explicacion.reprocesar`. **Hecho**.
- [X] T213 [US1] Afinar el texto `es`/`en` de `ai.modal.reprocess.invite`,
      `ai.modal.reprocess.modelLabel` y `ai.modal.reprocess.action` en
      `src/lib/i18n/{es,en}.json`. **Hecho** en T002 (texto final, no quedó borrador).

**Checkpoint**: se puede reprocesar con otro modelo gratuito desde un resultado o desde un error,
sin cerrar el modal. Historia entregable de forma independiente (MVP). **Verificado**: `pnpm check`
en 0 errores, `pnpm test` y `pnpm test:component` en verde.

---

## Phase 4: User Story 2 - Fijar el modelo preferido como modelo por defecto (Priority: P2)

**Goal**: sobre una respuesta obtenida al reprocesar, poder fijar su modelo como el modelo por
defecto de toda la aplicación.

**Independent Test**: reprocesar con un modelo, marcarlo como predeterminado, y comprobar que
Ajustes → asistencia con IA y un informe con resumen de IA usan ese modelo.

### Tests para User Story 2

- [X] T300 [P] [US2] En `src/lib/stores/explicacion.svelte.test.ts`: `fijarPorDefecto(modelo)` llama
      a `setSetting("settings.ai.model", modelo)` y después a `ia.refrescar()`; si `setSetting`
      rechaza, la fase y el `markdown`/`error` que ya se mostraban no cambian (se limita a exponer
      el fallo, no descarta la respuesta — edge case de la spec). **Hecho** — 3 tests: el toggle de
      `esReprocesada`, el camino feliz y el fallo al guardar.
- [X] T301 [P] [US2] En `src/lib/components/ExplicacionModal.browser.test.ts`: el botón
      `ai.modal.reprocess.setDefault` **no** aparece sobre la respuesta inicial (`esReprocesada`
      ausente/`false`); aparece sobre una respuesta marcada como obtenida por reproceso
      (`esReprocesada: true`) y, al pulsarlo, llama a `onfijarpordefecto` con el modelo de esa
      respuesta. **Hecho**.

### Implementación

- [X] T310 [US2] En `src/lib/stores/explicacion.svelte.ts`: añadir `esReprocesada = $state(false)`
      (a `false` en `lanzar()`, a `true` dentro de `reprocesar()` antes de `#pedir`), y el método
      `fijarPorDefecto(modelo: string): Promise<void>` que llama a `setSetting` + `ia.refrescar()`,
      capturando el error con el mismo patrón `toAppError` que ya usa `#pedir`. Hace pasar T300.
      **Hecho** — el error se guarda en un campo propio, `errorFijarPorDefecto`, separado de `error`
      (que es del flujo de pedir la explicación): así un fallo al guardar el ajuste no toca la fase
      ni la respuesta que ya se estaba mostrando.
- [X] T311 [US2] En `src/lib/components/ExplicacionModal.svelte`: props nuevos `esReprocesada:
      boolean` y `onfijarpordefecto: ((modelo: string) => void) | undefined`; en fase `resultado`,
      si `esReprocesada`, mostrar el botón `t("ai.modal.reprocess.setDefault")` junto al pie de
      modelo usado, y tras confirmarlo un texto breve `t("ai.modal.reprocess.setDefaultDone")` (o
      `setDefaultError` si falló). Hace pasar T301. **Hecho, con un recorte deliberado**: se
      implementó el aviso de error (`errorFijarPorDefecto` como prop, mostrando
      `setDefaultError`), pero se **descartó** `setDefaultDone` (la confirmación positiva) — el
      callback `onfijarpordefecto` de `explicacion.fijarPorDefecto` no relanza la excepción (la
      captura internamente en `errorFijarPorDefecto`), así que confirmar el éxito exigiría o bien
      que el método relanzara (cambiando su contrato ya probado en T300) o bien un mecanismo de
      notificación cruzado (`Toast`, sin uso ni cableado hoy en la aplicación) — coste
      desproporcionado para un mensaje de cortesía cuando el ajuste ya cambiado en Ajustes es la
      prueba. La clave `ai.modal.reprocess.setDefaultDone` queda en los diccionarios sin usar por
      ahora; no se retira por si se retoma.
      **Corrección post-validación manual (2026-09-12)**: el botón se pintó primero con
      `variant="ghost"`; en la app real se confundía con texto normal (sin borde ni fondo, al lado
      de párrafos) y el responsable del producto no lo reconoció como botón — detectado con una
      captura de pantalla real. Cambiado a `variant="secondary"`, la misma que "Reprocesar", para
      que se lea claramente como acción pulsable.
- [X] T312 [US2] Conectar en `src/routes/alerts/+page.svelte` y `src/routes/disks/[id]/+page.svelte`
      el nuevo prop `onfijarpordefecto` a `explicacion.fijarPorDefecto`. **Hecho**, y también en
      `src/routes/events/+page.svelte` (mismo hallazgo de T212).
- [X] T313 [US2] Afinar el texto `es`/`en` de `ai.modal.reprocess.setDefault`,
      `ai.modal.reprocess.setDefaultDone` y `ai.modal.reprocess.setDefaultError`. **Hecho** en T002.

**Checkpoint**: fijar un modelo desde el modal cambia `settings.ai.model`; Ajustes y el informe HTML
lo reflejan sin tocar su propio código (ya leen ese ajuste). **Verificado**: `pnpm check` en 0
errores; `pnpm test` (7/7 en `explicacion.svelte.test.ts`) y `pnpm test:component` (12/12 en
`ExplicacionModal.browser.test.ts`) en verde.

---

## Phase 5: User Story 3 - Consultar respuestas anteriores sin perder espacio (Priority: P3)

**Goal**: cada respuesta que deja de estar en primer plano queda accesible en una fila plegada,
identificada por su modelo, sin ocupar espacio.

**Independent Test**: reprocesar dos o tres veces y comprobar que las respuestas anteriores quedan
como filas plegables desplegables, y que desaparecen enteras al cerrar el modal y abrir otra
explicación.

### Tests para User Story 3

- [X] T400 [P] [US3] En `src/lib/stores/explicacion.svelte.test.ts`: cada llamada a `reprocesar()`
      empuja al array `historial` un snapshot `{ modelo, markdown }` o `{ modelo, error }` de lo que
      estaba en primer plano justo antes; `lanzar()` vacía `historial` y resetea `esReprocesada`.
      Sin límite: reprocesar 5 veces dentro de la misma prueba deja 4 entradas en `historial` (nunca
      se descarta la más antigua). **Hecho** — 3 tests, incluido el caso de archivar un error (sin
      `modeloUsado` propio: se identifica por el modelo que se le había pedido).
- [X] T401 [P] [US3] En `src/lib/components/ExplicacionModal.browser.test.ts`: con la prop
      `historial` poblada con dos entradas, se ven dos `<details>` colapsados (`open === false`) con
      el texto `ai.modal.reprocess.historyEntry` interpolando cada modelo; desplegar uno no cierra
      el otro ni afecta a la respuesta en primer plano. **Hecho**, más un test para una entrada con
      error.

### Implementación

- [X] T410 [US3] En `src/lib/stores/explicacion.svelte.ts`: `historial = $state<Intento[]>([])`
      (tipo `Intento = { modelo: string; markdown?: string; error?: AppError }`, ver
      `data-model.md`); en `reprocesar()`, antes de fijar `#modeloOverride`, empujar el snapshot de
      lo que hay en primer plano (`markdown`+`modeloUsado` si fase `resultado`, `error`+modelo
      previo si fase `error`); vaciar en `lanzar()`. Hace pasar T400. **Hecho**.
- [X] T411 [US3] En `src/lib/components/ExplicacionModal.svelte`: prop nuevo
      `historial: { modelo: string; markdown?: string; error?: AppError }[]`; tras el bloque de
      resultado/error, una lista de `<details>` (mismo patrón ya usado en la fase `error` para el
      detalle técnico) con `<summary>{t("ai.modal.reprocess.historyEntry", { model: intento.modelo })}</summary>`
      y dentro el `<Markdown>` o el texto de error de ese intento. Hace pasar T401. **Hecho** — el
      bloque se factorizó en un `{#snippet historialLista()}` (Svelte 5) para no duplicar el marcado
      entre las fases `resultado` y `error`, en vez de repetirlo dos veces.
- [X] T412 [US3] Afinar el texto `es`/`en` de `ai.modal.reprocess.historyEntry`. **Hecho** en T002.

**Checkpoint**: el historial de una sesión de explicación es consultable y desaparece al cambiar de
caso. Las historias 1-3 juntas cubren el flujo completo de comparar modelos. **Verificado**:
`pnpm check` en 0 errores; `pnpm test` (10/10) y `pnpm test:component` (14/14) en verde.

---

## Phase 6: User Story 4 - El selector no ofrece modelos que fallarán con la clave activa (Priority: P4)

**Goal**: mientras la clave activa sea la de demostración compartida, los modelos de pago se ven
pero no se pueden elegir, en los dos selectores de la aplicación; con una clave propia, vuelven a
poder elegirse.

**Independent Test**: con la clave de demostración activa, abrir el selector de Ajustes y el de
este modal y comprobar que los modelos de pago aparecen deshabilitados en los dos; configurar una
clave propia y comprobar que pasan a poder elegirse en los dos.

### Tests para User Story 4

- [X] T500 [P] [US4] En `AiModelSelect.browser.test.ts`, nuevo test: con `ia.set({ ...,
      usandoClaveCompartida: true })` antes de renderizar y un modelo `esDePago: true` en el
      catálogo, la opción correspondiente se renderiza `disabled`; con
      `usandoClaveCompartida: false`, la misma opción es seleccionable (comportamiento actual,
      confirma que no hay regresión sobre el diálogo de aviso de FR-015a de la 005). (La exclusión
      del modo automático por `incluirAutomatico` ya se probó en T104, Foundational.) **Hecho** —
      2 tests (con y sin clave de demostración), más la comprobación del aviso
      `paidDisabledDemo`.

### Implementación

- [X] T510 [US4] En `src/lib/components/AiModelSelect.svelte`: importar el store `ia`
      (`$lib/stores/ia.svelte`) y, en `opciones`, marcar `disabled: m.esDePago &&
      ia.usandoClaveCompartida` en cada modelo de pago (la prop `incluirAutomatico` ya existe desde
      T115). Cuando una opción deshabilitada por este motivo esté visible, mostrar debajo el aviso
      `t("settings.ai.model.paidDisabledDemo")` (igual que ya hace `listUnavailable`). Hace pasar
      T500. Como `ExplicacionModal` ya usa `<AiModelSelect>` desde T211, este cambio se refleja en
      los dos selectores sin tocar el modal. **Hecho**.
- [X] T512 [US4] Afinar el texto `es`/`en` de `settings.ai.model.paidDisabledDemo`. **Hecho** en
      T002.
- [X] T513 [US4] Redactar **ADR-058** en `docs/decisions.md` (enmienda a ADR-054, mismo patrón que
      ADR-045 sobre ADR-044): referenciar ADR-054 sin reescribirlo, explicar que ahora los modelos
      de pago se deshabilitan —no se ocultan— en el selector mientras la clave activa sea la de
      demostración, y por qué (evitar el error de OpenRouter que ADR-054 aceptaba como suficiente).
      Ejecutar `pnpm docs:build` después. **Hecho**.

**Checkpoint**: los dos selectores de modelo de la aplicación respetan el mismo criterio de
disponibilidad según la clave activa; ADR-054 queda enmendado y documentado. **Verificado**:
`pnpm check` en 0 errores; `pnpm test`/`pnpm test:component` en verde; `pnpm docs:build` regenerado.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: cerrar la feature con la definición de terminado del proyecto (skill `cierre-tarea`).

- [X] T900 Ejecutar `pnpm check`, `pnpm lint`, `pnpm verify`, `pnpm test`, `pnpm test:component`,
      `cargo clippy --all-targets -- -D warnings` y `cargo fmt --check` (desde `src-tauri/`); todo
      en verde. **Hecho** — todo en verde salvo dos fallos preexistentes y sin relación, ya
      anotados en T001 (el de `export.rs` y el de Prettier sobre `.claude/tmp/`); `cargo fmt`
      necesitó una pasada sobre `commands/mod.rs` (líneas largas de los tests de T101).
- [ ] T901 Recorrer `quickstart.md` a mano (`pnpm app:dev`) siguiendo las 4 historias en orden.
      **Pendiente — requiere a la persona**: la app va elevada (UAC) y es una ventana de escritorio
      nativa; este agente no tiene forma de abrirla ni interactuar con ella en esta sesión.
- [ ] T902 [P] Comprobar tema claro y oscuro y la ventana mínima (1024 × 560) para el modal con el
      selector, el botón de reproceso y al menos dos filas de historial desplegadas a la vez (§8 de
      `ui-design.md`, definición de terminado). **Pendiente — requiere a la persona**, mismo motivo
      que T901.
- [ ] T903 Actualizar `docs/open-questions.md` o `docs/decisions.md` si la validación manual revela
      algo no anticipado (criterio de la skill `cierre-tarea`); si no hay nada que anotar, dejarlo
      dicho explícitamente en el cierre de la tarea. **Pendiente de T901/T902.** Lo que sí surgió
      durante la implementación (no durante la validación manual) ya quedó anotado en el propio
      `tasks.md`, tarea por tarea: la representación `string | null` de `ts-rs` (T110), la pantalla
      de Eventos como tercer sitio que monta el modal (T212), la condición de carrera de FR-013
      (T203/T210) y el recorte de `setDefaultDone` (T311).
- [X] T904 `pnpm docs:build` si se tocó algún fichero de `docs/` aparte del ADR de T513 (por si
      quedó pendiente). **Hecho** — también se tocó `docs/ui-contract.md` (T114); `pnpm docs:check`
      confirma `historias.md` al día.

---

## Dependencies & Execution Order

### Fases

- **Setup (Fase 1)**: sin dependencias.
- **Foundational (Fase 2)**: depende de Setup. **Bloquea todas las historias.**
- **US1 (Fase 3, P1)**: depende de Foundational. Es el MVP.
- **US2 (Fase 4, P2)**: depende de US1 (necesita poder reprocesar para tener algo que fijar por
      defecto — su propia "Prueba independiente" en `spec.md` ya lo asume así).
- **US3 (Fase 5, P3)**: depende de US1 (necesita reprocesar para que exista historial). Independiente
      de US2.
- **US4 (Fase 6, P4)**: independiente de US1-US3 en la práctica — T510 solo añade el cálculo de
      `disabled` por clave activa a `AiModelSelect`, componente que ya usan tanto Ajustes como el
      modal (este último desde T211/Foundational). No hay una "mitad del modal" que reescribir.
- **Polish (Fase 7)**: depende de las historias que se vayan a entregar.

### Paralelismo

- T100-T104 (tests de Foundational) en paralelo entre sí; T110-T115 (implementación) en paralelo
  entre sí una vez fallan sus tests (T111 depende de T110 por tocar el mismo `origen` ya tipado).
- Dentro de cada historia, las tareas de test marcadas `[P]` van en paralelo; la implementación que
  depende de un test concreto va después de ese test, no en paralelo con él.
- US2 y US3 pueden implementarse en paralelo entre sí una vez cerrada US1 (tocan zonas distintas del
  mismo fichero — vigilar conflictos de merge en `explicacion.svelte.ts`/`ExplicacionModal.svelte`
  si de verdad se reparten entre dos personas).

---

## Implementation Strategy

### MVP primero (User Story 1)

1. Fase 1 (Setup) → Fase 2 (Foundational, bloqueante) → Fase 3 (US1).
2. **Parar y validar**: recorrer el escenario 1 de `quickstart.md` a mano.
3. US1 solo ya es útil: se puede probar otro modelo, aunque todavía no se pueda fijar por defecto ni
   ver el historial ni haya deshabilitado de modelos de pago.

### Entrega incremental

1. Setup + Foundational → base lista.
2. US1 → MVP demostrable.
3. US2 → fijar por defecto.
4. US3 → historial plegable.
5. US4 → coherencia del selector con la clave activa + enmienda de ADR.
6. Polish.

Cada historia deja la aplicación en un estado completo y sin regresiones sobre la anterior.
