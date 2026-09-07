# Fase 1 — Modelo de datos

Feature: Puente del registro de eventos de Windows al motor de alertas.

**Regla general**: no hay entidades persistidas nuevas ni migración. Se reutilizan `system_events`,
`alert_groups` y `alert_occurrences` tal cual (`docs/data-model.md` §2). Lo que sigue describe las
estructuras de dominio (en memoria, puras) que introduce el motor y cómo se mapean a lo persistido.

---

## 1. Entidades persistidas (sin cambios de esquema)

### `system_events` — entrada del motor

Ya existe. Campos relevantes para la evaluación: `channel`, `record_id` (identidad), `occurred_at_utc`
(UTC), `provider`, `event_id`, `level`, `message`, `raw_xml`, `device_id` (nullable),
`mapping_confidence` (`exact` | `inferred` | `unknown`).

El motor **solo evalúa** las filas que `repo_varios::insert_event_if_new` marca como nuevas en el
ciclo (D6). No modifica `system_events`.

### `alert_groups` — salida del motor

Ya existe. Esta feature crea grupos cuyo `rule_key` es una regla de eventos. Uso de campos:

| Campo | Para reglas de eventos |
|---|---|
| `rule_key` | `events.disk_error`, …, `device.removed_unexpected`, `inventory.duplicate_id` |
| `target_device_id` | el disco correlacionado, o `NULL` si la atribución es `unknown` (Q2 → A) |
| `target_volume_id` | el volumen, para reglas con contexto `volume_guid`, si se conoce |
| `deduplication_key` | `rule_key \| target \| context` — `context` es `provider:event_id`, o `volume_guid`, o el par de discos (ver §3) |
| `severity` | de la tabla `reglas_eventos` (§2), con posible escalada por frecuencia |
| `last_occurrence_at_utc` | **clave para la histéresis temporal**: `ahora - este valor > ventana` ⇒ resolver |
| `context_json` | opcional; para `inventory.duplicate_id`, el par de identidades; para un grupo absorbido por una ráfaga, nota de la causa |

### `alert_occurrences` — cronología

Ya existe. **Cambio de uso**: los tres puntos donde se inserta una ocurrencia
(`repo_alertas::create_group`, `record_occurrence`, `reopen_as_new_cycle`) empiezan a escribir
`triggering_event_id` con el `system_events.id` del evento que provocó esa ocurrencia (antes siempre
`NULL`). Para una ráfaga absorbida bajo `device.removed_unexpected`, cada evento de la ráfaga es una
fila de `alert_occurrences` con su `triggering_event_id` y su `occurred_at_utc` propio (FR-010).

### `event_cursors` — sin cambios

El bookmark del canal `System` sigue igual. Es lo que, junto con `UNIQUE(channel, record_id)`,
implementa «solo hacia delante» (D6).

---

## 2. Estructuras de dominio nuevas (puras, en memoria)

### `EventoParaRegla` (vista ligera del evento)

Lo que las funciones de `motor.rs` reciben, en vez de la fila entera:

```
EventoParaRegla {
    id: i64,                       // system_events.id, para triggering_event_id
    provider: &str,
    event_id: i64,
    occurred_at_utc: OffsetDateTime,
    level: EventLevel,
    device_id: Option<&str>,
    volume_guid: Option<&str>,
    mapping_confidence: MappingConfidence,
    es_extraible: bool,            // del inventario: disco USB / medio extraíble (para paging_error, delayed_write)
}
```

### `ClasificacionEvento` (tabla `reglas_eventos`, en código — D3)

Una fila por combinación proveedor/id vigilada, reflejo de `alert-rules.md` §3.2:

```
ClasificacionEvento {
    provider: &'static str,
    event_ids: &'static [i64],           // p. ej. StorageSpaces-Driver 202/203/209
    rule_key: &'static str,
    objetivo: Objetivo,                  // Dispositivo | Volumen | SinObjeto
    dedup_context: ContextoDedup,        // ProviderEventId | VolumeGuid | ParDeDiscos | Ninguno
    evaluacion: Evaluacion,
}

enum Evaluacion {
    Inmediata { severidad: AlertSeverity },                       // disk_error, filesystem_error, storage_space_degraded, disk_predictive, filesystem_repaired, filesystem_repair_storm
    PorFrecuencia { min_en_1h: u32, severidad: AlertSeverity,     // paging_error (10, warn), io_retry (5, warn)
                    escala_crit_en_1h: Option<u32> },             // controller_reset (base warn, escala crit a 3)
    CondicionalNoExtraible { severidad_extraible: Option<AlertSeverity>,   // delayed_write: warn siempre; crit si NO extraíble
                             severidad_fija: AlertSeverity },
}
```

Los eventos de `alert-rules.md` §3.3 (`Microsoft-Windows-Ntfs` 98, `NvmeDisk` 501, `Volsnap` *,
`volmgr` 161, `Microsoft-Windows-Disk` *) **no tienen fila**: la ausencia de fila es la exclusión
(FR-005). Una prueba de completitud verifica que cada fila «genera alerta» de §3.2 tiene entrada y
que ningún id de §3.3 la tiene.

### `Rafaga` (correlación de 60 s — D4)

```
Rafaga {
    device_id: Option<String>,
    eventos: Vec<EventoParaRegla>,        // ordenados por occurred_at_utc
    causa: Option<EventoParaRegla>,       // el disk 157, si lo hay
}
```

Función pura `correlacionar_rafaga(evento_nuevo, contexto_60s) -> ResultadoCorrelacion`:
- `contexto_60s` = eventos del mismo disco en `system_events` dentro de [t-60s, t].
- Si hay un `disk` 157 en la ventana ⇒ el evento nuevo es **ocurrencia** de
  `device.removed_unexpected` (no crea grupo propio).
- Si el evento nuevo **es** el `disk` 157 y hay grupos de evento del mismo disco creados en la
  ventana ⇒ marcar esos grupos para reasignación (resueltos con nota de causa; sus eventos
  re-registrados bajo `device.removed_unexpected`).
- Sin `disk` 157 ⇒ cada evento sigue su regla (FR-009a).

---

## 3. Clave de deduplicación por regla

Según `alert-rules.md` §2, columna «Contexto de dedup». Formato general
`rule_key | target_type:target_id | context`:

| Regla | target | context | Efecto |
|---|---|---|---|
| `events.disk_error` | `device:<id>` o `sin_objetivo` | `provider:event_id` | un grupo por (disco, tipo de evento) |
| `events.filesystem_error` | `volume:<guid>` o `sin_objetivo` | `provider:event_id` | |
| `events.filesystem_repaired` | `volume:<guid>` | `volume_guid` | un grupo por volumen |
| `events.filesystem_repair_storm` | `volume:<guid>` | `volume_guid` | |
| `events.controller_reset` | `device:<id>` | `provider:event_id` | escala a crítico ≥ 3/h |
| `events.paging_error` | `device:<id>` | `device_id` | solo discos no extraíbles, ≥ 10/h |
| `events.io_retry` | `device:<id>` | `device_id` | ≥ 5/h |
| `events.delayed_write` | `volume:<guid>` | `volume_guid` | crítico si el volumen no es extraíble |
| `events.disk_predictive` | `device:<id>` o `sin_objetivo` | `ninguno` | |
| `events.storage_space_degraded` | disco virtual | id. del disco virtual | |
| `device.removed_unexpected` | `device:<id>` | `ninguno` | crítico; warn si USB |
| `inventory.duplicate_id` | `sin_objetivo` | par de discos (identidades ordenadas) | una sola vez por par |

Cuando `mapping_confidence == unknown` y la regla pedía `device`/`volume`: `target = sin_objetivo`,
`context = provider:event_id`, y `context_json` anota que no se pudo atribuir (Q2 → A, FR-004a).

---

## 4. Transiciones de estado (reutiliza `alerts::agrupacion`)

Las mismas de `alert-rules.md` §1: `— → active` (primera evaluación positiva), `active/acknowledged
→ resolved` (histéresis cumplida), `resolved/archived → active, cycle+1` (recaída). Sin cambios en
`agrupacion::procesar`; solo se le pasa `triggering_event_id`.

**Novedad — resolución por tiempo, no por ciclos**: para las reglas de eventos, la condición de
resolución (`EvaluacionAlerta::resuelto`) la calcula un **barrido** en `evaluar_eventos`, no una
serie de valores:

```
resuelto = (ahora - grupo.last_occurrence_at_utc) > ventana_de_la_regla
```

donde `ventana_de_la_regla` es 24 h o 7 días según `alert-rules.md` §2. Este barrido recorre los
grupos de eventos `active`/`acknowledged` **cada ciclo**, haya o no eventos nuevos — igual que
`collector.stalled` se evalúa en `post_procesar_ciclo` sin depender de un ciclo de recopilación
concreto.

`smart.media_errors` / `smart.error_log` ya tienen el mismo problema sin resolver
(`docs/open-questions.md` J.17): esta feature **puede** cerrar J.17 de paso si el barrido temporal
se hace genérico, o dejarlo explícitamente fuera. → decisión menor para `/speckit-tasks`.

---

## 5. Reglas de validación

- Un evento cuyo `provider/event_id` no está en la tabla `reglas_eventos` **no se evalúa** (no es un
  error; es la mayoría de los eventos).
- Un evento de nivel `Information` cuyo id **sí** está en la tabla (no debería ocurrir con la tabla
  correcta) se registra a `warn` y no se evalúa: la tabla y `§3.3` mandan sobre el nivel del evento.
- `occurred_at_utc` que no parsea como RFC3339 ⇒ el evento se salta con `tracing::warn!`, no tumba el
  ciclo (constitución §X, §XI: campos de fuente externa validados).
- Ventana de frecuencia: se cuenta sobre `occurred_at_utc` (hora del evento), no sobre la hora de
  ingesta, para que un lote atrasado no falsee el umbral.
