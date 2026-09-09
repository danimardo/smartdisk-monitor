# Implementation Plan: Actividad de disco representativa mediante ventana continua

**Branch**: `007-actividad-disco-representativa` | **Date**: 2026-09-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/007-actividad-disco-representativa/spec.md`

## Summary

La actividad de disco pasa de ser una **instantánea de 1 s tomada una vez cada 30 s** a un
**agregado de una ventana deslizante** alimentada por muestreo continuo. El bucle en segundo plano
que ya despierta cada 1 s (`iniciar_planificador`) toma una muestra del contador `% Idle Time` de
cada disco físico monitorizado en cada tick, desde una **consulta PDH persistente** (abierta una
vez, no abierta-y-cerrada por lectura). Cada disco tiene una ventana en memoria del tamaño de la
cadencia de «métricas rápidas» (30 s por defecto); de ella se derivan **media** y **pico**.

- **Panel general**: muestra el **pico** de la ventana (Q1 → A).
- **Detalle de disco**: muestra **media y pico**, con procedencia y antigüedad.
- **Serie histórica** (`activity_percent`): una fila por ciclo de «métricas rápidas` con **la media
  de la ventana**, y **sin fila** cuando la ventana aún está incompleta (Q3 → A, FR-010a).
- **Contrato**: `activityPercent: number | null` se **sustituye** por un valor estructurado
  `{ estado, mediaPercent, picoPercent, muestras, ventanaSegundos }` en `DiskSummary` /
  `DeviceDetail` (Q2 → A / FR-012a).
- **Caudal (bytes/s) y latencias**: sin cambios, siguen en `perf_counters::leer` a 30 s (Q2 → A).
- **Sin dependencias nuevas, sin permisos de Tauri nuevos**: se extiende la FFI a `pdh.dll` que ya
  vive en `collectors/perf_counters.rs`.

## Technical Context

**Language/Version**: Rust 1.94.0 (edición 2021, MSRV 1.77) en el backend; TypeScript 5.9.3 /
Svelte 5.57 (runes) en la interfaz. Versiones fijadas por la constitución.

**Primary Dependencies**: Ninguna nueva. Backend: FFI directa a `pdh.dll` (ya presente), `tracing`,
`rusqlite`, `time`, `ts-rs`. Interfaz: Zod 4.5.4, catálogo propio de componentes.

**Storage**: SQLite (`metric_samples`), sin cambios de esquema. La ventana deslizante vive **solo en
memoria** del proceso backend (`AppState`), no se persiste.

**Testing**: `cargo test` (dominio de la ventana: media, pico, parcial, hueco, muestra fallida);
`pnpm test` (esquema Zod nuevo + su prueba de rechazo, `metricHelp`); `pnpm test:component`
(`DiskCard`, `MetricCard` con los estados válido / parcial / no disponible); `pnpm check` (tipos).

**Target Platform**: Windows 10 1809+ / Server 2016+, x64, proceso elevado.

**Project Type**: Aplicación de escritorio (Tauri 2 + SvelteKit `adapter-static`, SSR off).

**Performance Goals**: coste de CPU del muestreo imperceptible en reposo (SC-005); una lectura PDH
por tick de 1 s y por disco (lo mismo que hace el Administrador de tareas de forma continua). Panel
del inventario sigue pintándose rápido (`open-questions.md` J.59).

**Constraints**: `activity_percent` nunca se inventa (constitución §I): ventana incompleta → estado
`parcial` o `no_disponible`, jamás 0. Reloj **monotónico** (`std::time::Instant`), no hora de
pared. La consulta PDH y sus manejadores **no cruzan de hilo**: viven en el hilo del planificador.
Ningún `console.*`/`println!`; el muestreo no añade una línea de log por tick (§XV).

**Scale/Scope**: 1–~20 discos físicos. Ventana de ~30 muestras por disco (30 s a 1 s/muestra).
Superficie del cambio: 1 módulo de dominio nuevo (`domain/actividad.rs`), extensión de
`collectors/perf_counters.rs`, cambios en `commands/mod.rs` (bucle, `enrich_with_smart_data`,
`refresh_metricas_rendimiento`, DTO), `AppState`, 3 componentes/pantallas de interfaz, esquemas Zod,
diccionarios i18n, y 5 documentos normativos + ADR-050.

## Constitution Check

*GATE: debe pasar antes de la Fase 0 y volver a comprobarse tras la Fase 1.*

| Principio | Aplicación en esta feature | Estado |
|---|---|---|
| **I. Veracidad del dato** | Ventana incompleta → `parcial`/`no_disponible`, nunca 0. Muestra PDH fallida no entra como 0 (FR-005). Histórico con hueco cuando la ventana es parcial (FR-010a), y la gráfica ya dibuja huecos sin interpolar. Media y pico llevan procedencia y antigüedad. | ✅ Pasa |
| **II. Orden de prioridades** | Veracidad (estado explícito) por encima de la comodidad de tener siempre un número. Simplicidad: reconstruir la consulta PDH al cambiar el inventario en vez de gestionar contadores vivos uno a uno; caudal/latencias **no** entran en la ventana (Q2 → A). | ✅ Pasa |
| **III. Pila fija / cero red / sin deps** | Cero red. **Ninguna dependencia nueva**: se añade un símbolo FFI (`PdhRemoveCounter`) al bloque `extern` que ya existe, o se evita reconstruyendo la consulta. Sin biblioteca nueva. | ✅ Pasa |
| **IV. Dominio ↔ presentación** | La matemática de la ventana (media, pico, clasificación de estado) vive en `domain/actividad.rs`, **pura**, sin PDH ni Tauri: se prueba con fixtures. El colector solo muestrea; el comando solo traduce. Ningún componente recalcula el estado. | ✅ Pasa |
| **V. Persistencia local** | Sin cambio de esquema SQLite. La ventana es estado en memoria (como `paused`, `source_health`): un dato de 30 s no debe sobrevivir a un reinicio. `activity_percent` sigue en `metric_samples` en bruto (porcentaje 0–100). UTC en persistencia. | ✅ Pasa |
| **VI. Sistema de diseño** | Estados válido / parcial / no disponible con texto o icono, nunca solo color. Cero literales visuales y cero literales de interfaz: claves nuevas en `es.json` y `en.json`. `MetricCard` y `DiskCard` son del catálogo; no se crea componente nuevo (la sparkline de US3 ya existe: `Sparkline.svelte`). | ✅ Pasa |
| **VII. Accesibilidad** | La cifra del panel y del detalle siguen el patrón ya existente (tooltip con ratón en `DiskCard`, `MetricCard` con teclado en el detalle). La sparkline opcional reutiliza `Sparkline` con su `role="img"` y lectura textual. | ✅ Pasa |
| **VIII. Testeabilidad** | La agregación es «área donde el error es silencioso» → **test-first** para `domain/actividad.rs` (FR-021). Cobertura de `domain/` ≥ 90 %. Estados de componente (`DiskCard`, `MetricCard`): válido / parcial / no disponible. | ✅ Pasa |
| **X. Errores** | Un fallo del subsistema de rendimiento degrada `MetricSource::PerformanceCounter` con su `AppError` (código estable, clave i18n, detalle) por el mecanismo ya existente (`actualizar_source_health`), en el ciclo de «métricas rápidas», no en cada tick. La tarjeta se degrada, la aplicación sigue. | ✅ Pasa |
| **XI. Validación de fronteras** | El DTO nuevo se genera con `ts-rs`; su esquema Zod se infiere con `z.infer` y lleva **prueba de rechazo** (dato con forma inesperada → `ipc.schema_mismatch`). Nada de `as` sobre datos de `invoke`/`listen`. | ✅ Pasa |
| **XIII. `svelte-check`** | `pnpm check` con cero errores y cero avisos tras adaptar los consumidores del contrato. | ✅ Pasa (se verifica al implementar) |
| **XIV. SvelteKit idiomático** | El valor llega por el evento `metrics:updated` que ya existe (el backend empuja; la interfaz no sondea, ADR-015). Sin `setInterval` de datos. `$derived` para la clasificación en pantalla. Suscripciones ya se sueltan en el layout. | ✅ Pasa |
| **XV. Registro** | El muestreo por tick **no** loguea (§XV: lo que se repite cada ciclo es `debug`/`trace`). Un fallo del contador se registra `warn` una vez por flanco, como ya hace `source_health`. Sin números de serie ni rutas de perfil. | ✅ Pasa |
| **Límites duros (`AGENTS.md`)** | **Cambia el contrato** `DiskSummary`/`DeviceDetail` → exige regenerar `ts-rs`, actualizar `docs/ui-contract.md` §3.2 y su ADR (ADR-050). El hook `proteger-rutas.mjs` puede pedir confirmación al editar `docs/decisions.md`/contratos durante la implementación: es esperado, se pide autorización entonces. **Sin permiso de Tauri nuevo, sin dependencia nueva.** | ⚠️ Requiere ADR-050 y actualización de documentos normativos (previsto en Fase 1 y en tasks) |

**Veredicto**: sin violaciones. El único punto de fricción (cambio de contrato) está dentro del
proceso normal, con su ADR y su regeneración de `ts-rs`, igual que ADR-041 en su día.

## Project Structure

### Documentation (this feature)

```text
specs/007-actividad-disco-representativa/
├── plan.md              # Este fichero
├── research.md          # Fase 0 — decisiones técnicas resueltas
├── data-model.md        # Fase 1 — entidades (ventana, agregado, DTO, serie)
├── quickstart.md        # Fase 1 — cómo validar la feature de extremo a extremo
├── contracts/
│   └── disk-activity.md  # Fase 1 — contrato UI↔backend del campo de actividad + API interna Rust
└── tasks.md             # Fase 2 — /speckit-tasks (NO lo crea este comando)
```

### Source Code (repository root)

```text
src-tauri/src/
├── domain/
│   ├── actividad.rs            # NUEVO — VentanaActividad, AgregadoActividad, EstadoActividad.
│   │                           #         Puro: media, pico, clasificación de estado, corte por hueco.
│   └── mod.rs                  # + pub mod actividad;
├── collectors/
│   └── perf_counters.rs        # + ConsultaActividad: PDH_HQUERY persistente con un contador
│                               #   `% Idle Time` por disco; muestrear() -> HashMap<i64, Result<f64>>.
│                               #   `leer()` deja de devolver activity_percent (solo caudal + latencias).
├── persistence/
│   └── db.rs                   # + AppState.actividad: Mutex<HashMap<String, AgregadoActividad>>
└── commands/
    └── mod.rs                  # - iniciar_planificador: muestreo por tick + publicación del agregado
                                # - ejecutar_ciclo / refresh_metricas_rendimiento: persistir media si `valido`
                                # - enrich_with_smart_data: rellena el campo estructurado desde AppState.actividad
                                # - DiskSummary DTO: activity_percent -> activity: ActividadDisco
                                # - LecturaRendimiento: sin activity_percent

src/
├── lib/api/
│   ├── generated/DiskSummary.ts, DeviceDetail.ts, ActividadDisco.ts   # regenerados por cargo test
│   ├── schemas.ts             # diskSummary: activityPercent -> activity: actividadDisco (objeto)
│   └── schemas.test.ts        # + prueba de rechazo del objeto de actividad
├── lib/design/
│   ├── types.ts               # DiskSummary.activityPercent -> activity: ActividadDisco
│   └── metricHelp.ts          # veredicto "activity": media/pico/parcial/no disponible
├── lib/components/
│   ├── DiskCard.svelte        # magnitud "activity": pico de la ventana; estado parcial/no disponible
│   └── (MetricCard.svelte)    # detalle: media + pico; sin componente nuevo
└── routes/
    ├── +page.svelte           # HeroPanel: hecho "Actividad" -> pico
    └── disks/[id]/+page.svelte# MetricCard de actividad: media + pico; ayuda contextual

src/lib/i18n/es.json, en.json  # claves nuevas: estado parcial, "media"/"pico", cuerpo de ayuda revisado

docs/                          # open-questions.md (D.4 nuevo), architecture.md, data-model.md,
                               # ui-contract.md §3.2, decisions.md (ADR-050); historias.md regenerado
```

**Structure Decision**: proyecto único Tauri + SvelteKit ya establecido. El cambio respeta las
tres capas de la constitución §IV: matemática de la ventana en `domain/` (pura, test-first),
acceso al hardware en `collectors/`, traducción en `commands/`, presentación en `src/`.

## Complexity Tracking

*Sin violaciones de la Constitution Check que justificar.* La única complejidad añadida —una
consulta PDH con estado que sobrevive entre ciclos— es exactamente lo que la cabecera de
`perf_counters.rs` ya anticipaba como pendiente («…porque el planificador en segundo plano todavía
no existe para mantener una consulta abierta entre ciclos»), y es más simple que la alternativa
(abrir/cerrar por lectura y dormir 1 s), no menos.

## Phase 0 — Research

Ver [research.md](./research.md). Resuelve: ciclo de vida de la consulta PDH persistente y la
semántica de `% Idle Time` entre dos `PdhCollectQueryData`; dónde viven el muestreo y la ventana;
intervalo de muestreo, tamaño de ventana y umbral de hueco (→ `open-questions.md` D.4); reloj
monotónico y suspensión; forma del tipo estructurado; regla de persistencia; degradación de fuente;
confirmación de «sin deps ni permisos nuevos».

## Phase 1 — Design & Contracts

- [data-model.md](./data-model.md): `MuestraActividad`, `VentanaActividad`, `AgregadoActividad`,
  `EstadoActividad`, el DTO `ActividadDisco`, el campo `AppState.actividad`, y la semántica revisada
  de la serie `activity_percent`.
- [contracts/disk-activity.md](./contracts/disk-activity.md): el cambio de `DiskSummary` /
  `DeviceDetail` (antes/después), el `metrics:updated` (forma estable, campo más rico), el esquema
  Zod y su prueba de rechazo, y la API interna de Rust (`ConsultaActividad`, `VentanaActividad`).
- [quickstart.md](./quickstart.md): guía de validación de extremo a extremo (carga real, reinicio,
  batería, pruebas de dominio, regeneración de documentación).

### Post-Design Constitution Re-Check

Tras el diseño de la Fase 1, la Constitution Check se mantiene: el DTO estructurado refuerza el
principio I (el estado del dato viaja explícito en vez de colarse como `null` ambiguo), la matemática
aislada en `domain/` cumple IV y VIII, y no aparece ninguna dependencia ni permiso nuevo. El cambio
de contrato queda cubierto por ADR-050 + `ts-rs` + prueba de rechazo Zod.
