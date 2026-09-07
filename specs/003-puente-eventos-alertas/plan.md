# Implementation Plan: Puente del registro de eventos de Windows al motor de alertas

**Branch**: `003-puente-eventos-alertas` (dir de spec; el trabajo sale hoy de `002-rediseno-v3`, sin
rama nueva salvo que se decida crearla) | **Date**: 2026-09-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/003-puente-eventos-alertas/spec.md`

## Summary

Añadir al motor de alertas la evaluación de las reglas cuya fuente es el registro de eventos de
Windows (`docs/alert-rules.md` §2): las 10 reglas `events.*`, más `device.removed_unexpected` e
`inventory.duplicate_id`. La ingesta de eventos (`event_log.rs`), la correlación con disco
(`domain/correlacion.rs`) y la persistencia (`system_events`) ya existen y no se rehacen; lo que
falta es la capa de evaluación encima.

Enfoque técnico: funciones puras de evaluación por familia de regla en `alerts/motor.rs` sobre una
vista ligera del evento; un orquestador `alerts::evaluar_eventos` análogo a `evaluar_smart` que se
llama desde `refresh_events` con los eventos **recién insertados** del ciclo, más un barrido de
resolución temporal de los grupos de eventos abiertos; una tabla estática `(proveedor, id) → regla`
que refleja `alert-rules.md` §3.2; y la ventana de correlación de 60 s como función pura que decide
qué eventos de una ráfaga se colapsan bajo una causa `disk` 157 y cuáles no. La deduplicación, el
ciclo de vida y las notificaciones reutilizan `alerts::agrupacion` y `alerts::notificaciones` tal
cual, ampliando solo `politica_notificacion` con las nuevas reglas.

## Technical Context

**Language/Version**: Rust 1.94.0 (edición 2021, MSRV 1.77) para el motor; TypeScript 5.9.3 /
Svelte 5.57 para el enlace mínimo de interfaz.

**Primary Dependencies**: ninguna nueva. Se usan `rusqlite` 0.40 (`bundled`), `time` 0.3, `sha2`
0.10, `serde` 1, y en el frontend Zod 4.5. El acceso al Event Log ya está resuelto con FFI directo a
`wevtapi.dll` en `collectors/event_log.rs` (sin el crate `windows`).

**Storage**: SQLite en `%ProgramData%\SmartDisk Monitor\`. Tablas implicadas, todas ya existentes:
`system_events`, `alert_groups`, `alert_occurrences` (con `triggering_event_id` ya en el esquema,
sin usar), `event_cursors`, `settings`. **Sin migración nueva prevista** (ver research.md).

**Testing**: `cargo test` (motor y orquestador con fixtures, sin hardware); `pnpm test` /
`test:component` / `test:e2e` / `test:a11y` para el enlace de interfaz. Test-first obligatorio en el
motor de alertas (constitución §VIII, `alert-rules.md` §5).

**Target Platform**: Windows 10 1809+ / Server 2016+, x64. La evaluación de reglas es pura y
multiplataforma; solo la ingesta (`refresh_events`) es `#[cfg(windows)]`.

**Project Type**: aplicación de escritorio Tauri 2 + SvelteKit (`adapter-static`, SSR off). Cambio
concentrado en el backend (`src-tauri/src/alerts/`, `src-tauri/src/domain/`), con un toque en
`src-tauri/src/commands/` (llamar al orquestador) y otro mínimo en la interfaz de alertas.

**Performance Goals**: la evaluación de un ciclo de eventos (lote típico 0–20 eventos, cada 30 s) no
debe añadir latencia perceptible al ciclo de recopilación; las consultas de ventana de frecuencia
(«≥ N en 1 h») usan el índice `idx_system_events_time` / `idx_system_events_device` ya existentes.
SC-007: habilitar el puente sobre meses de histórico produce cero alertas por eventos antiguos.

**Constraints**: veracidad del dato (constitución §I): nunca se alerta por ausencia de evento; una
atribución inferida nunca se presenta como certeza; un evento no compatible/informativo (`§3.3`)
nunca genera alerta. La aplicación sigue respondiendo aunque el colector de eventos falle un ciclo
(§II.3, SC-008). Cobertura ≥ 90 % en `src-tauri/src/alerts/` y `src-tauri/src/domain/`.

**Scale/Scope**: ~13 reglas nuevas × 5 pruebas mínimas cada una + pruebas de ventana de correlación
y de umbrales de frecuencia. Registro de referencia: 2.038 eventos de almacenamiento en 180 días
(`alert-rules.md` §3).

## Constitution Check

*GATE: debe pasar antes de la Fase 0. Re-evaluado tras la Fase 1.*

| Principio | Cómo lo cumple este plan | Riesgo |
|---|---|---|
| **I — Veracidad del dato** | No se alerta por ausencia de evento (FR-006). La atribución `desconocida` produce alerta **sin objeto** y el detalle lo dice (FR-004a), nunca una atribución inventada. Los eventos de `§3.3` (informativos, VSS, volcado) se excluyen explícitamente (FR-005). | Bajo. Es el corazón de la spec. |
| **II — Orden de prioridades** | Un fallo del colector de eventos degrada solo esa fuente; el resto del ciclo produce sus alertas (FR-015). | Bajo. Patrón ya existente (`refresh_events` es tolerante). |
| **III — Pila fija** | Cero dependencias nuevas. FFI a `wevtapi.dll` ya existe. | Ninguno. |
| **IV — Dominio ↔ presentación** | La evaluación vive en `alerts/` y `domain/`, pura y probable con fixtures. Los comandos solo orquestan. La interfaz no interpreta XML de eventos. | Bajo. |
| **V — Persistencia** | Sin almacén nuevo. UTC en base. La retención ya no borra eventos vinculados ni ocurrencias críticas (`§V`). Se empieza a escribir `alert_occurrences.triggering_event_id`, columna ya existente. | Bajo. Confirmar en research.md que no hace falta migración. |
| **VI — Interfaz** | Titulares por `t()` en los dos idiomas, redactados según `alert-rules.md` §4 (FR-017). El detalle renderiza el texto del evento como texto plano (FR-016). El color lo decide `deviceState()`, no un componente. | Bajo. |
| **VII — Accesibilidad** | El enlace alerta→evento en el detalle usa navegación por `<a href>` a `/events`, foco visible, `role` correctos. `pnpm test:a11y` en la pantalla de alertas. | Bajo. |
| **VIII — Test-first** | Motor de alertas: prueba antes que código, ciclo rojo-verde. Las 5 pruebas de `alert-rules.md` §5 por regla (FR-019) + correlación de ráfaga (FR-020) + umbrales de frecuencia en el borde. Cobertura ≥ 90 % en `alerts/` y `domain/`. | Medio: volumen de pruebas alto (~70). Mitigación: helpers de fixture compartidos, un `mod tests` por familia. |
| **IX — Seguridad y privacidad** | Sin telemetría, sin red, sin permiso de Tauri nuevo (leer el Event Log ya está cubierto por la ejecución elevada). El texto de eventos que ya se persiste no cambia. | Bajo. Confirmar «sin permiso nuevo» en research.md. |
| **X — Errores** | Un fallo de evaluación de una regla se registra con `tracing::warn!` y no tumba el ciclo (patrón de `evaluar_unreadable`). | Bajo. |
| **XI — Validación de fronteras** | El XML del evento ya se parsea a tipos explícitos en `event_log.rs`. Si la interfaz gana un campo nuevo (`triggeringEventId` en la ocurrencia), su esquema Zod y su prueba de rechazo. | Bajo. |
| **XV — Registro** | Decisiones de regla a `debug` (`alert-rules.md` §…: «decisión de una regla» es ejemplo explícito de `debug`). Nada personal en el log: se usa el id interno del disco, nunca el identificador del evento con rutas de perfil. | Bajo. |

**Veredicto (pre-Fase 0)**: sin violaciones. Sin entradas en Complexity Tracking. La única tensión
real es el **volumen de pruebas** (§VIII), coste esperado de 13 reglas, no una desviación.

Puntos que cerró la Fase 0 (`research.md`):

1. **D1** — sin migración: `alert_occurrences.triggering_event_id` ya existe en el esquema.
2. **D2** — `device.removed_unexpected` se activa por `disk` 157 correlacionado **o** por baja de
   inventario de un disco no-USB; USB solo con `disk` 157; la ventana de 60 s evita el doble grupo.
3. **D3** — la tabla `(proveedor, id) → regla` vive en código (`alerts/reglas_eventos.rs`) con
   prueba de completitud contra `alert-rules.md` §3.2/§3.3. `PROVEEDORES_VIGILADOS` (filtro de
   ingesta) se queda en `event_log.rs`.
4. **D4** — la correlación consulta `system_events` 60 s hacia atrás; caso de orden invertido
   (`disk` 157 tardío) resuelto por reasignación de grupos derivados.
5. **D5** — `triggeringEventId` en el DTO de ocurrencia + enlace del detalle de alerta a
   `/events?focus=<id>`.
6. **D6** — «solo hacia delante» sin marcador nuevo: se apoya en `insert_event_if_new` + bookmark.
7. **D7** — ningún permiso de Tauri nuevo.

### Re-evaluación tras la Fase 1 (Design & Contracts)

Sin cambios en el veredicto. El diseño confirma:

- **§I / §XI**: `EventoParaRegla` es una vista tipada; la atribución `unknown` produce alerta sin
  objeto (nunca inventada). La tabla `reglas_eventos` en código, con serde/tipos, no
  `serde_json::Value`.
- **§IV**: dos módulos nuevos puros en `alerts/`; `commands/mod.rs` solo orquesta (`refresh_events`
  pasa a devolver transiciones, `post_procesar_ciclo` las procesa).
- **§V**: sin almacén ni migración nuevos; `triggering_event_id` es columna existente.
- **§VIII**: el barrido de resolución temporal (data-model.md §4) puede además cerrar `J.17`
  (`smart.media_errors`/`smart.error_log` sin resolución temporal) si se hace genérico — decisión
  menor para `/speckit-tasks`, no amplía el alcance obligatorio.
- **§IX**: `contracts/` confirma cero comandos y cero permisos nuevos; la única superficie de
  interfaz nueva es el parámetro `?focus=` de `/events`.

Complexity Tracking sigue vacío.

## Project Structure

### Documentation (this feature)

```text
specs/003-puente-eventos-alertas/
├── plan.md              # Este fichero
├── research.md          # Fase 0 — decisiones (los 5 puntos de arriba)
├── data-model.md        # Fase 1 — entidades y transiciones
├── quickstart.md        # Fase 1 — guía de validación end-to-end
├── contracts/
│   └── eventos-alertas.md   # Fase 1 — reglas, claves i18n, adiciones de contrato de interfaz
├── checklists/
│   └── requirements.md      # de /speckit-specify
└── tasks.md             # Fase 2 — /speckit-tasks (NO lo crea este comando)
```

### Source Code (repository root)

```text
src-tauri/src/
├── alerts/
│   ├── motor.rs               # + funciones puras de evaluación por familia de regla de eventos
│   ├── reglas_eventos.rs      # NUEVO — tabla (proveedor, id) → regla + clasificación (§3.2)
│   ├── correlacion_rafaga.rs  # NUEVO — ventana de 60 s: qué se colapsa bajo una causa disk 157
│   ├── mod.rs                 # + evaluar_eventos(...) orquestador + barrido de resolución temporal
│   ├── agrupacion.rs          # + triggering_event_id en EvaluacionAlerta y en los INSERT de ocurrencia
│   └── notificaciones.rs      # + arms de politica_notificacion para las ~13 reglas nuevas
├── domain/
│   └── correlacion.rs         # sin cambios de fondo (ya resuelve evento→disco); se reutiliza
├── commands/
│   └── mod.rs                 # refresh_events pasa a devolver transiciones; post_procesar_ciclo
│                              #   las procesa; device.removed_unexpected se engancha a la baja de inventario
├── persistence/
│   ├── repo_alertas.rs        # + escribir triggering_event_id; consulta de grupos de eventos abiertos
│   └── repo_varios.rs         # + consulta de eventos por (regla/proveedor, device, ventana temporal)
└── (migrations/)              # solo si research.md concluye que hace falta (previsto que NO)

src/
├── lib/i18n/{es,en}.json      # + alert.rule.<rule_key>.{title,summary} de las ~13 reglas
├── lib/api/                   # + esquema Zod si la ocurrencia gana triggeringEventId (contrato)
└── routes/alerts/+page.svelte # enlace del detalle de alerta a su(s) evento(s) en /events

e2e/ui/alerts.spec.ts          # + títulos traducidos de las reglas de eventos; enlace al evento
```

**Structure Decision**: cambio concentrado en `src-tauri/src/alerts/` (dos módulos nuevos, tres
ampliados), siguiendo el patrón ya establecido por `evaluar_smart` / `evaluar_capacidad` /
`evaluar_collector_stalled`: función pura en `motor.rs`, orquestación en `mod.rs`, ciclo de vida en
`agrupacion.rs`, notificación en `notificaciones.rs`. La interfaz cambia lo mínimo: i18n y un enlace.

## Complexity Tracking

> Sin violaciones de la Constitution Check. Nada que justificar aquí.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
