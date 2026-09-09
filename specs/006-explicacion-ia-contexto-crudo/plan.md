# Implementation Plan: Contexto crudo para la explicación con IA

**Branch**: `006-explicacion-ia-contexto-crudo` | **Date**: 2026-09-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/006-explicacion-ia-contexto-crudo/spec.md`

## Summary

Incremento sobre la feature **005** (ya fusionada). Hoy la ayuda con IA envía al modelo una
reconstrucción mínima: identificador interno de la regla + valor + tendencia + contexto del disco
(`domain::ia::componer_consulta`). No envía el detalle técnico real que la persona ve en pantalla.

Este incremento añade a la consulta, ya anonimizados:

- el **volcado crudo de `smartctl -a -j`** del disco implicado, en alertas `smart.*`/`temp.*`/
  `nvme.*` y en el detalle SMART de un disco (`get_alert_smart_raw_json` / la salida que ya expone
  el botón «Ver detalle técnico»);
- el **contenido legible del suceso de Windows** que disparó la alerta —su `message` renderizado y
  los campos `EventData` de su `raw_xml`—, sin el bloque `<System>`.

Y añade un ajuste opcional **«enviar sin revisar»** (cuarta clave de `settings.ai`, apagado de
fábrica): con él activo se omite la pantalla de revisión de fragmentos dudosos (FR-026 de la 005) y
se envía el texto ya anonimizado; activarlo exige una confirmación de riesgo, distinta de la vista
previa de FR-010.

**Enfoque técnico**: nada de red nueva, nada de dependencias nuevas, nada de permisos de Tauri
nuevos. Toda la mecánica de la 005 se reutiliza. El trabajo es:

1. **Recolección** (`commands::reunir_datos_explicacion`): además del resumen estructurado, obtener
   el volcado `smartctl` en el momento (mismo camino que `get_alert_smart_raw_json_impl`) y, para
   alertas de suceso, el evento disparador (`repo_varios::get_event_by_id` desde la ocurrencia) y
   extraer de él el `message` + los `EventData`.
2. **Anonimización en capas** (dominio, Rust, `reporting::anonimizar` + `domain::ia`):
   - **Extracción estructurada primero**: del JSON de `smartctl` se leen `serial_number` y `wwn`;
     se pasan como sustituciones literales al `Anonimizador` (que ya hace eso bien y de forma
     consistente). Marca, modelo y firmware **no** se sustituyen (FR-005).
   - **Barrido por patrones después** (nueva función pura en `domain::ia`, mismo estilo byte-a-byte
     que `barrer_texto_residual`, **sin crate `regex`**): sustituye SID (`S-1-5-…`), rutas NT de
     dispositivo (`\Device\…`) y WWN hexadecimales (`0x` + 16 hex) que la extracción literal no
     haya cubierto.
   - **Barrido residual al final**: `barrer_texto_residual` (ya existe) sobre el texto ensamblado.
3. **Ensamblado y recorte** (`domain::ia::componer_consulta`): `resumen → volcado → suceso`, en ese
   orden, y `recortar(total, MAX_DETALLE_CHARS)` con `MAX_DETALLE_CHARS` elevado. Como el suceso va
   al final, es lo primero que se recorta (FR-013); el resumen, al principio, siempre sobrevive.
4. **Ajuste nuevo**: `settings.ai.send_without_review` (bool, fábrica `false`). Lo escribe un
   comando dedicado de la familia IA (no `set_setting`, igual que `enabled`/`preview_acknowledged`).
   `borrar_clave_ia` lo devuelve a `false` (FR-012). `estado_ia` lo expone. `explicar_detalle_tecnico`
   lo consulta: si es `true`, salta la respuesta `Revision` por fragmentos (no la de vista previa).
5. **Frontend**: `EstadoIaWire` gana `sendWithoutReview`; la tarjeta de Ajustes de IA gana un
   `Switch` que, al activarse, abre un `ConfirmDialog` con el aviso de riesgo; el store `ia` y el
   `explicacion.svelte` no cambian su forma (el backend sigue devolviendo `ResultadoExplicacion`).
6. **Gobernanza**: enmienda al principio XVI (versión 1.9.0) — la aplica la persona con
   `parche-constitucion.md`. **ADR-047** nuevo: payload ampliado + anonimización en capas + modo
   «enviar sin revisar». Actualización puntual de la viñeta «Datos enviados» de ADR-046.

## Technical Context

**Language/Version**: Rust 1.77.2 (edición 2021) · TypeScript 5.9 · Svelte 5 (runes)

**Primary Dependencies**: **ninguna nueva.** `reqwest` y el FFI a `advapi32` de la 005 se
reutilizan sin cambios. El barrido por patrones se hace a mano (mismo estilo que
`barrer_texto_residual`), no con `regex`.

**Storage**: SQLite (`settings` k/v). **Una** clave nueva: `settings.ai.send_without_review`
(bool, fábrica `false`). Sin migración: fila nueva en tabla existente. La clave de API sigue fuera
de SQLite. El volcado y el contenido del suceso son **efímeros**: no se persisten para esta función
(el suceso ya vive en `system_events`; el volcado se obtiene al momento, como en la 005).

**Testing**:
- `cargo test` — dominio, test-first: barrido por patrones (SID, `\Device\`, WWN hex) señala y
  sustituye; extracción de `serial_number`/`wwn` del JSON de `smartctl`; extracción de `message` +
  `EventData` del `raw_xml`; ensamblado y orden de recorte (suceso primero); `send_without_review`
  salta la revisión de fragmentos pero no la vista previa.
- `pnpm test` — n/a (sin lógica de front nueva relevante).
- `pnpm test:component` — tarjeta de Ajustes de IA con el `Switch` nuevo y su `ConfirmDialog`
  (estados: apagado, encendido, diálogo abierto, confirmado).
- `pnpm test:e2e` — extender `e2e/ui/ia.spec.ts`: explicar una alerta SMART y comprobar (por el
  IPC de pruebas / mock del transporte) que el texto lleva el volcado con serie y WWN sustituidos;
  activar «enviar sin revisar» y comprobar que una consulta con fragmento dudoso no muestra la
  pantalla de revisión.
- `pnpm test:a11y` — el `ConfirmDialog` nuevo (ya cubierto por el componente de catálogo; verificar
  el contenido).

**Target Platform**: Windows x64, aplicación de escritorio elevada (`requireAdministrator`).

**Project Type**: desktop-app (frontend SvelteKit estático + backend Rust/Tauri).

**Performance Goals**: sin ruta caliente afectada. El volcado se obtiene al pulsar «Explícamelo»,
en el mismo comando `async` que ya llama a la red; `smartctl -a -j` tarda ~0,3–1 s (igual que «Ver
detalle técnico», ya aceptado en la 005). SC-002 de la 005 (≤20 s) sigue aplicando; un prompt
mayor consume más tokens pero no cambia el orden de magnitud.

**Constraints**: cero valores visuales literales · cero literales de interfaz (claves en `es` y
`en`, incluidos el aviso de riesgo y el `Switch`) · toda llamada de red iniciada por gesto
explícito (principio XVI) · el contenido de peticiones/respuestas nunca al log (FR-016) · la
respuesta del LLM sigue siendo contenido no confiable · el volcado y el suceso salen del proceso
**solo** tras la anonimización en el dominio · correcto en tema claro/oscuro y en 1024 × 560.

**Scale/Scope**: Backend — `domain/ia.rs`: 1 función pura nueva (barrido por patrones) + cambios en
`componer_consulta` (acepta volcado y suceso) y en los tipos `Detalle`/`ContextoDisco`;
`commands/mod.rs`: `reunir_datos_explicacion` obtiene volcado + suceso, 1 comando nuevo
(`establecer_envio_sin_revision`), cambios en `estado_ia`/`borrar_clave_ia`/`explicar_detalle_tecnico`;
`platform/ia_openrouter.rs`: `MAX_DETALLE_CHARS` elevado. 2 DTO tocados (`EstadoIaWire` +
`sendWithoutReview`; `ExplicacionIaWire` + `sinVolcado`/`sinSuceso`). Frontend —
1 `Switch` + 1 `ConfirmDialog` en `/settings`, campo nuevo en el store `ia`. ~10 claves i18n
nuevas. Docs — enmienda XVI, ADR-047, retoque ADR-046, `data-model.md`, `ui-contract.md`,
`open-questions.md` (MAX nuevo), `historias.md` (regenerado).

## Constitution Check

*GATE: pasa antes de Phase 0. Re-evaluado tras Phase 1.*

**Nota previa**: este incremento **requiere una enmienda al principio XVI** (versión 1.9.0). La
tabla evalúa la feature *bajo el texto enmendado*; sin la enmienda aplicada, la feature está en
conflicto con la redacción actual de dos viñetas de XVI y no puede darse por terminada (registrado
en `spec.md` §Assumptions y en `parche-constitucion.md`).

| Principio | Efecto de esta feature | Veredicto |
|---|---|---|
| I. Veracidad del dato | El volcado y el suceso solo alimentan una explicación orientativa; no cambian ningún color, regla ni retención (FR-017, extiende SC-007 de la 005). El modal sigue rotulando la explicación como orientación por IA. | ✅ |
| II. Orden de prioridades | Privacidad antes que comodidad: la anonimización en capas es obligatoria y previa a la salida; «enviar sin revisar» es **opt-in**, apagado de fábrica, con consentimiento de riesgo explícito (FR-007, FR-008). El caso por defecto conserva la revisión manual (FR-010). | ✅ |
| III. Pila fija / cero red | **Sin red nueva, sin dependencias nuevas, sin permisos nuevos.** Misma ruta saliente única de la 005, bajo el principio XVI (ya enmendado en 1.9.0 para el alcance del dato). | ✅ (bajo XVI 1.9.0) |
| IV. Dominio / presentación | El barrido por patrones y el ensamblado viven en `domain/ia.rs` (puro, test-first). La obtención del volcado y del suceso vive en `commands`/`platform`/`persistence`, no en el dominio. La interfaz no parsea XML de eventos ni construye órdenes de `smartctl` (principio IV, viñeta explícita): recibe `ResultadoExplicacion` ya compuesto. | ✅ |
| V. Persistencia local | 1 fila nueva en `settings` (`send_without_review`, bool). Sin migración. El volcado y el suceso no se persisten para esto. `reset_settings` ámbito `ai`/`all` la borra junto a las otras. | ✅ |
| VI. Sistema de diseño | Solo se reutilizan `Switch` y `ConfirmDialog` del catálogo. Sin componentes nuevos, sin valores literales, claves en los dos diccionarios. | ✅ (verificar en implementación) |
| VII. Accesibilidad AA | El `Switch` nuevo con `label`+`hint`; el `ConfirmDialog` ya cumple foco atrapado y `Escape`. Sin cambios en el modal de explicación. | ✅ |
| VIII. Testeabilidad | El barrido por patrones es un parser de privacidad → **test-first** (constitución VIII: «parsers» y «donde el fallo es silencioso»). Casos con SID, `\Device\`, WWN, y textos que no deben marcarse (evitar falsos positivos que rompan la explicación). | ✅ |
| IX. Seguridad y privacidad | Cero telemetría (FR-016). La superficie de datos que sale del equipo **crece** (volcado + suceso), y por eso: anonimización en capas obligatoria, extracción estructurada de serie/WWN antes que heurística, «enviar sin revisar» apagado de fábrica y con doble consentimiento, y **ADR-047** que lo documenta. Sin permiso ni dependencia nueva. | ✅ (con ADR-047 y enmienda XVI) |
| X. Errores comprensibles | Volcado no disponible o suceso ilegible → no es error: la consulta sigue con el resumen y el modal avisa (FR-014, FR-015). Los `AppError` de la 005 no cambian. | ✅ |
| XI. Validación de fronteras | El JSON de `smartctl` se parsea con tipos `serde` explícitos para leer `serial_number`/`wwn` (no `Value` para decidir); el `raw_xml` del evento se recorre para extraer `EventData` con un lector acotado, no se interpreta como documento de confianza. El DTO `EstadoIaWire` sigue validándose con Zod. | ✅ |
| XII. Configuración sin env | Nada en `process.env`. `MAX_DETALLE_CHARS` es constante de código. | ✅ |
| XIII. Tipos antes que nada | `EstadoIaWire` y `ExplicacionIaWire` regenerados con `ts-rs`; Zod al día; cada campo nuevo rompe en el mapeo Rust→TS y en el schema, un sitio cada uno, no en once. | ✅ |
| XIV. SvelteKit idiomático | El `Switch` usa el patrón `load` + acción; `$derived` para el estado del diálogo; sin `invoke` directo. | ✅ |
| XV. Registro de actividad | FR-016: ni el volcado ni el suceso ni la respuesta se escriben en el log; como mucho, los metadatos ya existentes (que hubo consulta, modelo, error). | ✅ |
| XVI. Asistencia con IA (**enmendado 1.9.0**) | La enmienda amplía «detalle técnico visible» al volcado y al contenido del suceso (ambos abribles en pantalla) y refuerza la cláusula de anonimización, reconociendo «enviar sin revisar» como consentimiento de segundo nivel. Todo lo demás de XVI se mantiene: apagada de fábrica, gesto explícito, destino único, clave en el almacén del SO, llamada desde el backend, respuesta no confiable, fallo que degrada solo la función, cero telemetría. | ✅ (requiere `parche-constitucion.md` aplicado) |

**Resultado del gate**: PASA bajo el texto enmendado (XVI 1.9.0), con dos condiciones:

- **(a) La enmienda XVI 1.9.0 se aplica ANTES de escribir código** — no es puerta de cierre, es
  **puerta previa** (`tasks.md` Fase 0, T000). Lo exige la §Gobernanza de la constitución
  («toda enmienda se hace en este documento y antes de escribir el código que la necesita»).
  Corrección respecto a una versión anterior de este plan que la situaba como puerta de cierre.
- **(b) ADR-047 redactado** — puede ir en paralelo a la implementación; es puerta de cierre
  (`tasks.md` T601).

## Project Structure

### Documentation (this feature)

```text
specs/006-explicacion-ia-contexto-crudo/
├── plan.md                  # Este fichero
├── research.md              # Phase 0
├── data-model.md            # Phase 1
├── quickstart.md            # Phase 1
├── contracts/
│   └── comandos-ia.md       # Delta al contrato de la 005
├── parche-constitucion.md   # Enmienda XVI 1.9.0 — la aplica la persona (hook)
├── checklists/
│   └── requirements.md      # Ya creado por /speckit-specify
└── tasks.md                 # /speckit-tasks (aún no)
```

### Source Code (repository root)

```text
src-tauri/src/
├── domain/
│   └── ia.rs                # + barrido por patrones (SID, \Device\, WWN); componer_consulta
│                            #   acepta volcado + suceso; tipos Detalle/ContextoDisco ampliados
├── commands/
│   └── mod.rs               # reunir_datos_explicacion: obtiene volcado + evento disparador;
│                            #   estado_ia / borrar_clave_ia / explicar_detalle_tecnico;
│                            #   + comando establecer_envio_sin_revision
├── platform/
│   └── ia_openrouter.rs     # MAX_DETALLE_CHARS elevado
├── persistence/
│   └── repo_varios.rs       # (reutilizado: get_event_by_id ya existe)
└── collectors/
    ├── smartctl.rs          # (reutilizado: query_device_json ya existe)
    └── event_log.rs         # posible helper para extraer EventData del raw_xml (o en domain)

src/
├── lib/
│   ├── api/generated/EstadoIaWire.ts   # regenerado
│   ├── api/schemas.ts                  # + sendWithoutReview
│   └── stores/ia.svelte.ts             # + getter sendWithoutReview
└── routes/settings/+page.svelte        # + Switch «enviar sin revisar» + ConfirmDialog

src/lib/i18n/{es,en}.json               # ~10 claves nuevas
```

**Structure Decision**: sin estructura nueva. El incremento toca los mismos cuatro módulos de la
005 (`domain/ia.rs`, `commands/mod.rs`, `platform/ia_openrouter.rs`, `routes/settings`) más lectura
de `system_events` que ya existe. El único fichero de dominio con lógica nueva sustancial es
`domain/ia.rs` (barrido por patrones), y es donde se concentra el test-first.

## Complexity Tracking

> Sin violaciones del Constitution Check que justificar. La ampliación de la superficie de datos
> (principio IX) no es una violación: es el objeto del incremento, está autorizada por la enmienda
> XVI 1.9.0 y mitigada por la anonimización en capas + ADR-047. No hay dependencias, permisos ni
> componentes nuevos.
