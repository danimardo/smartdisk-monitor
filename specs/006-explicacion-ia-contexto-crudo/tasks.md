---
description: "Task list — Contexto crudo para la explicación con IA (006)"
---

# Tasks: Contexto crudo para la explicación con IA

**Input**: `specs/006-explicacion-ia-contexto-crudo/` (plan.md, spec.md, research.md, data-model.md,
contracts/comandos-ia.md, quickstart.md)

**Incremento sobre la feature 005** (ya fusionada). No hay proyecto que inicializar; se tocan los
mismos módulos de la 005.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ir en paralelo (fichero distinto, sin dependencias pendientes)
- **[Story]**: US1 / US2 / US3 / US4 (mapea a las historias de `spec.md`)
- Rutas exactas en cada tarea. Comandos de Rust desde `src-tauri/`.

**Tests**: la constitución §VIII exige **test-first** para el barrido de anonimización (es un
parser de privacidad). Esas tareas de test van marcadas y **deben fallar antes** de implementar.
El resto de tests (componente, e2e) van en su historia o en Pulido.

---

## Phase 0: Puerta de gobernanza (BLOQUEANTE — previa a todo)

**Purpose**: la constitución (§Gobernanza) exige que una enmienda se aplique **antes** de escribir
el código que la necesita. La feature 006 contradice dos viñetas del principio XVI vigente; sin la
enmienda aplicada, cualquier tarea de las fases 2–6 produce código que viola la constitución.

**⚠️ Ninguna tarea `T0xx`–`T6xx` puede empezar hasta que T000 esté hecha.**

- [X] T000 **[Usuario]** Aplicar la enmienda al principio XVI (versión 1.9.0): ejecutar el script de `specs/006-explicacion-ia-contexto-crudo/parche-constitucion.md`, luego `pnpm docs:build`, y comprobar (`grep -n "1.9.0\|enviar sin revisar" .specify/memory/constitution.md`, `pnpm verify`). Fichero protegido por hook — lo hace la persona. **Hasta aquí no se toca código de `src/` ni `src-tauri/`.**

**Checkpoint**: `constitution.md` en 1.9.0, `pnpm verify` verde.

---

## Phase 1: Setup

**Purpose**: dejar la base verde y preparada.

- [X] T001 Confirmar baseline verde antes de empezar: `cd src-tauri && cargo test` y, desde la raíz, `pnpm check && pnpm lint && pnpm verify`. Anotar cualquier fallo previo no relacionado. (Requiere T000 hecha.)
- [X] T002 [P] Añadir al fichero fuente de i18n las claves nuevas (vacías o con texto borrador) en `src/lib/i18n/es.json` y `src/lib/i18n/en.json`: `settings.ai.sendWithoutReview.{label,hint,confirmTitle,confirmBody,confirmImpact,confirmCta}`, `ia.explain.withoutDump`, `ia.explain.withoutEvent`. (El texto final se afina en cada historia; esto evita que `verify:i18n` falle a mitad.)

**Checkpoint**: `cargo test` y `pnpm verify` pasan.

---

## Phase 2: Foundational (prerequisito bloqueante)

**Purpose**: la anonimización en capas y la composición de la consulta ampliada. **Bloquea US1,
US2 y US3** (las tres envían volcado/suceso anonimizado). US4 no depende de esta fase salvo por el
punto de corte en `explicar_detalle_tecnico` (T516).

**⚠️ Ninguna historia de envío de contenido puede empezar hasta cerrar esta fase.**

### Tests primero (barrido de anonimización — test-first, constitución §VIII)

- [X] T100 [P] En `src-tauri/src/domain/ia.rs` (módulo `#[cfg(test)]`), tests que FALLAN para la función nueva `redactar_identificadores(&str) -> String`: sustituye `S-1-5-21-…` → `<SID>`; `\Device\HarddiskVolume24` y `\Device\Harddisk1\DR19` → `<DISPOSITIVO>`; `0x` + 12–16 hex → `<WWN>`. Y casos que **no** debe tocar: `0xFF` (corto), una palabra normal, un marcador ya puesto (`<SERIE-1>`, `<EQUIPO>`).
- [X] T101 [P] En `src-tauri/src/domain/ia.rs`, tests que FALLAN para la extracción de identificadores del JSON de `smartctl`: dado un JSON con `serial_number` y `wwn: {naa,oui,id}`, `identificadores_de_volcado(json) -> {serie: Option<String>, wwn: Option<String>}` devuelve la serie y la cadena decimal exacta de `wwn.id`. JSON sin `wwn` → `wwn: None`, sin romper.
- [X] T102 [P] En `src-tauri/src/domain/ia.rs`, tests que FALLAN para `componer_consulta` ampliada: con `volcado = Some(...)` el `user` **en bruto** (sin anonimizar) contiene el resumen **antes** que el volcado; con `suceso = Some(...)` el suceso va **al final**; tras `recortar` con un `max` que obliga a cortar se pierde el suceso, no el resumen. `componer_consulta` **no** anonimiza (eso es una pasada aparte en el comando — ver research §D1).
- [X] T103 [P] En `src-tauri/src/domain/ia.rs` o `src-tauri/src/commands/`, test que FALLA para la pasada de anonimización completa: dado un `user_crudo` con serie/WWN (del volcado), SID y `\Device\…`, el resultado de `anon.aplicar` + `redactar_identificadores` no contiene ninguno; y `model_name`/`firmware_version` con dígitos sobreviven sin marcador (FR-005).

### Implementación

- [X] T110 En `src-tauri/src/domain/ia.rs`, implementar `redactar_identificadores` (byte-a-byte, mismo estilo que `barrer_texto_residual`, **sin crate `regex`**). Hacer pasar T100.
- [X] T111 En `src-tauri/src/domain/ia.rs`, implementar `identificadores_de_volcado`: tipo `serde` mínimo local (`struct VolcadoIds { serial_number: Option<String>, wwn: Option<Wwn> }`, `struct Wwn { id: Option<i64>, .. }`), parseo tolerante. Hacer pasar T101. No exportar a `SmartctlResult` ni a métricas.
- [X] T112 En `src-tauri/src/domain/ia.rs`, ampliar `Detalle`/`ContextoDisco` y la firma de `componer_consulta` para aceptar `volcado: Option<&str>` y `suceso: Option<&str>`; **ensamblar el `user` en bruto** en orden `resumen → volcado → suceso`, **sin anonimizar** (la anonimización es una pasada aparte en el comando, T211–T212). Hacer pasar T102. Actualizar las llamadas existentes (`None, None` hasta US1/US2).
- [X] T113 En `src-tauri/src/platform/ia_openrouter.rs`, subir `MAX_DETALLE_CHARS` de `8_000` a `40_000`; actualizar el comentario (referencia a open-questions y a la spec 006).
- [X] T114 En `src-tauri/src/domain/ia.rs`, añadir a `ExplicacionIaWire` los campos `sin_volcado: bool` y `sin_suceso: bool` (FR-014/FR-015; **no** reutilizar `detalle_recortado` — ver `data-model.md` §«Explicación devuelta»). Regenerar el DTO con `ts-rs` (`cargo test` genera los `.ts`); reflejar en `src/lib/api/schemas.ts` y en `src/lib/api/generated/`. Ajustar `analizar_respuesta` y `explicacion.svelte.ts` (nuevos campos `sinVolcado`/`sinSuceso` en la rama `ok`).

**Checkpoint**: `cargo test ia::` verde; `componer_consulta` compone con volcado y suceso; nada de red todavía.

---

## Phase 3: User Story 1 — Alerta SMART con volcado crudo (P1) 🎯 MVP

**Goal**: al explicar una alerta `smart.*`/`temp.*`/`nvme.*`, el modelo recibe el volcado
`smartctl -a -j` del disco, anonimizado, además del resumen.

**Independent Test**: abrir una alerta SMART con la ayuda con IA activada, pulsar «Explícamelo»,
confirmar la vista previa, y ver en el texto que sale el volcado con `serial_number` y `wwn`
sustituidos; la explicación cita cifras concretas del disco.

### Tests

- [X] T200 [P] [US1] En `src-tauri/src/commands/mod.rs` (`#[cfg(test)]`), test de `reunir_datos_explicacion` para `tipo: Alerta` de regla `smart.*`: el `user_crudo` resultante incluye el bloque de volcado **en bruto** (sin anonimizar), y devuelve la serie y el WWN extraídos del volcado. La comprobación de que serie/WWN quedan sustituidos en el texto que sale es de T103/T212 (tras la pasada de anonimización).
- [X] T201 [P] [US1] Test de la reserva FR-014: si el dispositivo no tiene `smartctl_path` o `query_device_json` falla, `reunir_datos_explicacion` devuelve datos con `sin_volcado = true` y **no** es `Err`.

### Implementación

- [X] T210 [US1] En `src-tauri/src/commands/mod.rs`, en `reunir_datos_explicacion` rama `TipoOrigen::Alerta`: si `rule_key` es `smart.*`/`temp.*`/`nvme.*`, obtener el volcado con el mismo camino que `get_alert_smart_raw_json_impl` (`smartctl_path` del `disp` → `collectors::smartctl::query_device_json`) y pasarlo **en bruto** a `ia::componer_consulta(..., Some(&volcado_bruto), None)`. El fallo → `sin_volcado = true` (no `Err`), `componer_consulta(..., None, None)`.
- [X] T211 [US1] En `src-tauri/src/commands/mod.rs`, en `reunir_datos_explicacion`: `ia::identificadores_de_volcado(&volcado_bruto)` → devolver la serie y el WWN extraídos para que `explicar_detalle_tecnico` los añada al `Anonimizador` (junto a las series de inventario que ya recoge). Marca/modelo/firmware **no** se añaden.
- [X] T212 [US1] En `src-tauri/src/commands/mod.rs`, `explicar_detalle_tecnico`: **una sola pasada de anonimización sobre el `user_crudo` ya ensamblado** (que ya incluye el volcado en bruto): `anon.aplicar(&user_crudo)` (con serie+WWN del volcado añadidas) → `ia::redactar_identificadores(&texto)` → `ia::recortar(&texto, MAX_DETALLE_CHARS)` → `barrer_texto_residual`. Propagar `sin_volcado`/`sin_suceso` al resultado `Ok`. (Ver research §D1 para el orden exacto; `componer_consulta` no anonimiza.)
- [X] T213 [US1] En `src/routes/alerts/+page.svelte` (modal de explicación) y/o `src/lib/stores/explicacion.svelte.ts`: si el resultado trae `sinVolcado`, mostrar `t("ia.explain.withoutDump")` bajo la explicación. Sin cambios de layout del modal.
- [X] T214 [US1] Afinar el texto `es`/`en` de `ia.explain.withoutDump` en `src/lib/i18n/{es,en}.json`.

**Checkpoint**: explicar una alerta SMART envía el volcado anonimizado; con el disco ausente, avisa y sigue.

---

## Phase 4: User Story 2 — Alerta de suceso de Windows con el contenido del evento (P1)

**Goal**: al explicar una alerta nacida de un suceso de Windows, el modelo recibe el `message` del
suceso y sus `EventData`, anonimizados, sin el bloque `<System>`.

**Independent Test**: abrir una alerta de suceso, pulsar la acción, y ver que el texto incluye el
mensaje y los `Data` del suceso, sin `<Computer>`/`<Security>`, con `\Device\…` → `<DISPOSITIVO>`.

### Tests

- [X] T300 [P] [US2] En `src-tauri/src/collectors/event_log.rs` o `src-tauri/src/domain/ia.rs` (`#[cfg(test)]`), test que FALLA para `extraer_contenido_suceso(message: Option<&str>, raw_xml: &str) -> Option<String>`: con las fixtures `EVENTO_NTFS_98` y `EVENTO_DISK_158` de `event_log.rs`, devuelve el `message` (si hay) y los pares `Name=valor` de `<EventData>`; **no** incluye `Provider`, `Computer`, `Security`, `Execution`, `EventRecordID`.
- [X] T301 [P] [US2] Test de `reunir_datos_explicacion` para `tipo: Alerta` de regla de suceso: toma el `triggering_event_id` de la ocurrencia más reciente y el `user_crudo` incluye el contenido del suceso **en bruto**, al final del texto. Un test aparte (nivel `explicar_detalle_tecnico` / T103) comprueba que SID y `\Device\…` quedan sustituidos tras la pasada de anonimización.
- [X] T302 [P] [US2] Test de la reserva FR-015: sin evento disparador o sin contenido legible → `sin_suceso = true`, no `Err`.

### Implementación

- [X] T310 [US2] Implementar `extraer_contenido_suceso` (lector acotado sobre `raw_xml`: localizar `<EventData>`, recorrer `<Data ...>`; sin parser XML ni dependencia nueva). Hacer pasar T300.
- [X] T311 [US2] En `src-tauri/src/persistence/repo_alertas.rs` (o donde vivan las ocurrencias), asegurar un acceso al `triggering_event_id` de la ocurrencia más reciente de un grupo (ya existe `list_occurrences`; usar su primer elemento). Sin cambio de esquema.
- [X] T312 [US2] En `src-tauri/src/commands/mod.rs`, `reunir_datos_explicacion` rama `Alerta` cuando `rule_key` **no** es `smart.*`/`temp.*`/`nvme.*` (regla de suceso): `repo_varios::get_event_by_id(conn, triggering_event_id)` → `extraer_contenido_suceso(...)` → pasar el resultado **en bruto** a `ia::componer_consulta(..., volcado, Some(&suceso_bruto))`. Fallo/None → `sin_suceso = true`, `Some(&suceso)` pasa a `None`. (Una alerta de regla de suceso con disco asociado puede llevar volcado **y** suceso: ambos van.)
- [X] T313 [US2] En `src-tauri/src/commands/mod.rs`: verificar que la única pasada de anonimización de T212 (`anon.aplicar` + `redactar_identificadores` + `recortar` + `barrer_texto_residual` sobre el `user_crudo` completo) cubre también el suceso ensamblado. Sin código nuevo si T212 se hizo sobre el texto completo; si no, corregir T212. Propagar `sin_suceso`.
- [X] T314 [US2] En el modal (`src/routes/alerts/+page.svelte` / store): si `sinSuceso`, mostrar `t("ia.explain.withoutEvent")`. Afinar el texto `es`/`en`.

**Checkpoint**: explicar una alerta de suceso envía el contenido del evento anonimizado, sin metadatos de sistema.

---

## Phase 5: User Story 3 — Detalle SMART de un disco con el volcado crudo (P2)

**Goal**: la acción «Explícamelo» del detalle SMART de un disco envía también el volcado crudo.

**Independent Test**: abrir el detalle SMART de un disco, pulsar la acción, ver el volcado
anonimizado en el texto y una explicación apoyada en sus cifras.

### Tests

- [X] T400 [P] [US3] Test de `reunir_datos_explicacion` para `tipo: Smart`: el `user_crudo` incluye la lista de contadores SMART (como en la 005) **y** el bloque de volcado en bruto, y devuelve la serie/WWN extraídos. La sustitución efectiva se comprueba a nivel `explicar_detalle_tecnico` (T103).

### Implementación

- [X] T410 [US3] En `src-tauri/src/commands/mod.rs`, `reunir_datos_explicacion` rama `TipoOrigen::Smart`: obtener el volcado con `query_device_json` del `device_id` directo; pasarlo **en bruto** a `componer_consulta(Detalle::Smart(...), ctx, Some(&volcado_bruto), None)` y devolver serie/WWN extraídos. La pasada única de anonimización de T212 en `explicar_detalle_tecnico` cubre este camino igual que el de US1 (mismo `user_crudo`). Extraer a función común el trozo de «obtener volcado + extraer ids» si US1 y US3 lo comparten, sin sobre-diseñar.
- [X] T411 [US3] En `src/routes/disks/[id]/+page.svelte` (o donde esté la acción del detalle SMART): mismo aviso `ia.explain.withoutDump` si `sinVolcado`.

**Checkpoint**: US1 y US3 comparten el camino del volcado; el detalle SMART lo envía.

---

## Phase 6: User Story 4 — Modo «enviar sin revisar» (P2)

**Goal**: ajuste opt-in que omite la pantalla de revisión de fragmentos (no la vista previa), con
consentimiento de riesgo.

**Independent Test**: activar el `Switch` en Ajustes → confirmar el diálogo → lanzar una consulta
con fragmento dudoso → no aparece la pantalla de revisión; la vista previa de la primera vez sí.

### Tests

- [X] T500 [P] [US4] En `src-tauri/src/commands/mod.rs` (`#[cfg(test)]`), test de `explicar_detalle_tecnico`: con `settings.ai.send_without_review = true` y un texto que `barrer_texto_residual` marcaría, el resultado **no** es `ResultadoExplicacion::Revision` por fragmentos; pero si `preview_acknowledged = false`, **sí** devuelve la vista previa.
- [X] T501 [P] [US4] Test de `borrar_clave_ia`: deja `send_without_review = false` además de `preview_acknowledged = false`.
- [X] T502 [P] [US4] Test de `estado_ia`: refleja `settings.ai.send_without_review` en `send_without_review` (fábrica `false`).
- [X] T503 [P] [US4] Cobertura del `Switch` + `ConfirmDialog` de «enviar sin revisar»: se traslada a e2e (T606); no hay infraestructura de test de componente para `/settings` y el flujo se prueba mejor de punta a punta.

### Implementación (backend)

- [X] T510 [US4] En `src-tauri/src/domain/ia.rs`, añadir `send_without_review: bool` a `EstadoIaWire` (y a `AiSettingsWire` si ese wire expone el grupo). Regenerar `.ts` con `ts-rs`; actualizar `src/lib/api/schemas.ts`.
- [X] T511 [US4] En `src-tauri/src/commands/mod.rs`, `estado_ia` (ambas construcciones del wire, líneas ~727 y ~756): leer `leer_ajuste_bool(conn, "settings.ai.send_without_review", false)`.
- [X] T512 [US4] En `src-tauri/src/commands/mod.rs`, `borrar_clave_ia`: añadir `guardar_ajuste(&conn, "settings.ai.send_without_review", &false, &ahora)?` junto al de `preview_acknowledged`.
- [X] T513 [US4] En `src-tauri/src/commands/mod.rs`, `claves_por_ambito`: añadir `"settings.ai.send_without_review"` a `const AI`.
- [X] T514 [US4] En `src-tauri/src/commands/mod.rs`, comando nuevo `establecer_envio_sin_revision(state, activar: bool) -> AppResult<EstadoIaWire>`: sin credencial → `AppError { code: "ia.no_key" }`; si no, `guardar_ajuste("settings.ai.send_without_review", activar)`; devuelve `estado_ia()`. `tracing::debug!` solo con `activar`.
- [X] T515 [US4] Registrar `establecer_envio_sin_revision` en `src-tauri/src/lib.rs` (`generate_handler!`) y exponerlo en `src/lib/api/client.ts` + `src/lib/api/index` (patrón de los otros comandos IA).
- [X] T516 [US4] En `src-tauri/src/commands/mod.rs`, `explicar_detalle_tecnico`: leer `send_without_review`; si `true`, en el `match origen.revision` saltar la rama que devuelve `ResultadoExplicacion::Revision` por `fragmentos` (seguir con el texto anonimizado). **No** tocar la rama de vista previa (FR-009).

### Implementación (frontend)

- [X] T517 [US4] En `src/lib/stores/ia.svelte.ts`: getter `get sendWithoutReview(): boolean` desde `this.estado`.
- [X] T518 [US4] En `src/routes/settings/+page.svelte`, tarjeta «Ayuda con IA» (bloque `ia.estado?.activa`): añadir un `Switch` con `label`/`hint` de `settings.ai.sendWithoutReview.*`, `checked={ia.sendWithoutReview}`. Al pasar a `true`, abrir un `ConfirmDialog` (`title/body/impact/confirmLabel` de las claves `confirm*`); `onconfirm` → `establecerEnvioSinRevision(true)` + `ia.refrescar()`. Apagar → llamada directa con `false`.
- [X] T519 [US4] Afinar el texto `es`/`en` de las seis claves `settings.ai.sendWithoutReview.*`.

**Checkpoint**: el modo se activa con confirmación, se resetea al desactivar la función, y salta solo la revisión de fragmentos.

---

## Phase 7: Polish & Cross-Cutting

**Purpose**: gobernanza, documentación normativa, verificación integral. Puertas de cierre.

- [X] T600 Verificar que la enmienda del principio XVI (T000) sigue aplicada y coherente: `grep -n "1.9.0" .specify/memory/constitution.md`, y que ADR-047 (T601) queda referenciado desde el principio y desde ADR-046. (La aplicación en sí es T000, puerta de la Fase 0.)
- [X] T601 [P] Escribir **ADR-047** en `docs/decisions.md` (usar la skill `adr`): payload ampliado (volcado crudo + contenido del suceso), anonimización en capas (extracción de campos conocidos + barrido de patrones + revisión residual), modo «enviar sin revisar». Alternativas descartadas: crate `regex`, enviar sin capa de patrones, anonimizar en el front.
- [X] T602 [P] Retocar la viñeta «Datos enviados» de **ADR-046** en `docs/decisions.md` para remitir a ADR-047 en cuanto al alcance del dato.
- [X] T603 [P] Actualizar `docs/data-model.md` (sección «Ayuda con IA»: de tres a cuatro claves, viñeta `send_without_review`).
- [X] T604 [P] Actualizar `docs/ui-contract.md` (comando `establecer_envio_sin_revision`, campo `sendWithoutReview` en `EstadoIaWire`, nota de que `explicar_detalle_tecnico` incluye volcado + suceso).
- [X] T605 [P] Actualizar `docs/open-questions.md`: `MAX_DETALLE_CHARS` pasa de 8 000 (propuesto) a 40 000 (decidido para 006), con la medición de referencia.
- [X] T606 Extender `e2e/ui/ia.spec.ts` (`pnpm test:e2e`): explicar una alerta SMART y comprobar por el mock del transporte que el texto lleva el volcado con serie/WWN sustituidos; activar «enviar sin revisar» y comprobar que una consulta con fragmento dudoso no muestra la pantalla de revisión.
- [X] T607 [P] `pnpm test:a11y` sobre el `ConfirmDialog` nuevo de Ajustes (contenido y foco).
- [X] T608 Verificación integral: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`; raíz: `pnpm check && pnpm lint && pnpm test && pnpm test:component && pnpm verify`.
- [ ] T609 **Pendiente (manual, requiere entorno)**: ejecutar los 7 escenarios de `quickstart.md` sobre la app real (`pnpm app:dev`, UAC + clave de OpenRouter real). No ejecutable en este entorno; la lógica está cubierta por `cargo test` (T103, T200–T302, T500) y `pnpm test:e2e` (T606).
- [X] T610 Cierre con la skill `cierre-tarea`: `historias.md` regenerado (`pnpm docs:build`), nueve puertas en verde (`check`/`lint`/`verify`/`test`/`build`/`docs:check` + `cargo fmt --check`/`clippy -D warnings`/`test`), sin `svelte-ignore`/`eslint-disable` nuevos.

---

## Dependencies & Execution Order

### Fases

- **Gobernanza (F0)**: T000. **Bloquea todo lo demás.** Sin la enmienda XVI aplicada no se toca `src/` ni `src-tauri/`.
- **Setup (F1)**: tras F0.
- **Foundational (F2)**: tras F1. **Bloquea US1, US2, US3**. US4 solo depende de F2 para T516 (punto de corte en `explicar_detalle_tecnico`), que puede quedar el último de US4.
- **US1 (F3)** y **US2 (F4)**: tras F2. Independientes entre sí (ramas distintas de `reunir_datos_explicacion`), aunque comparten fichero → coordinar los edits en `commands/mod.rs`.
- **US3 (F5)**: tras F2; reutiliza el helper del volcado de US1 (más limpio hacer US1 antes, pero no es un bloqueo duro).
- **US4 (F6)**: tras F2; T516 después de F2. Resto de US4 independiente de US1–US3.
- **Polish (F7)**: T601–T605 pueden empezar en cuanto la forma esté clara; T600, T606–T610 al final.

### MVP

**US1 sola** (F0 + F1 + F2 + F3) ya entrega valor: la explicación de una alerta SMART deja de ser
genérica. T000 (enmienda XVI) es **puerta previa** incluso para el MVP; ADR-047 (T601) es puerta de
cierre, porque el envío del volcado es lo que la enmienda autoriza.

### Paralelo

- F2: T100–T103 en paralelo (tests, distinto foco); luego T110, T111 en paralelo; T112 tras T110–T111; T114 independiente.
- F7: T601–T605, T607 en paralelo (ficheros distintos).
- US1 y US4 backend pueden avanzar en paralelo con dos personas si se coordina `commands/mod.rs`.

---

## Notas

- `[P]` = fichero distinto, sin dependencia pendiente.
- Los tests marcados «FALLA» (T100–T103, T300, T500…) se escriben y se ven fallar antes de implementar (constitución §VIII para el barrido; el resto por coherencia).
- Commit por tarea o grupo lógico. **No** hacer commit/push ni abrir PR sin pedirlo (AGENTS.md).
- Nada de `regex`, nada de dependencias, permisos ni componentes de catálogo nuevos.
- El contenido de peticiones y respuestas nunca al log (FR-016) — vigilar en cada `tracing::*` nuevo.
