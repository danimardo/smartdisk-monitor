# Fase 1 — Contratos

Feature: Puente del registro de eventos de Windows al motor de alertas.

Esta funcionalidad **no añade ningún comando Tauri ni ningún permiso**. Las alertas de eventos
salen por los canales que ya existen (`alerts:changed`, notificación nativa, bandeja). Los contratos
que cambian son: las claves de i18n de las reglas nuevas, una adición al DTO de ocurrencia de alerta
y un parámetro de consulta en la ruta `/events`.

---

## 1. Claves de i18n (`src/lib/i18n/es.json` y `en.json`)

Por cada regla que el motor puede emitir (`alert-rules.md` §4), en los dos idiomas:

```
alert.rule.<rule_key>.title      Titular corto, sin jerga. Dice QUÉ PASA, no qué contador se movió.
alert.rule.<rule_key>.summary    Una frase: qué significa y por qué importa.
```

`rule_key` nuevos a cubrir (13):

```
events.disk_error
events.filesystem_error
events.filesystem_repaired
events.filesystem_repair_storm
events.controller_reset
events.paging_error
events.io_retry
events.delayed_write
events.disk_predictive
events.storage_space_degraded
device.removed_unexpected
inventory.duplicate_id
```

(12 `rule_key`; `events.filesystem_error` cubre `Ntfs` 55 y 131 con un solo titular.)

Ejemplo de redacción conforme a §4 (borrador, se pule en implementación):

| `rule_key` | title (es) | summary (es) |
|---|---|---|
| `events.disk_error` | «Windows registró un error de disco» | «El sistema operativo anotó un fallo de lectura o escritura en este disco. Conviene revisarlo cuanto antes.» |
| `device.removed_unexpected` | «Un disco se desconectó sin avisar» | «Este disco desapareció del sistema sin una expulsión previa. Si no fuiste tú, puede haber un problema de cable o de alimentación.» |
| `inventory.duplicate_id` | «Dos discos comparten identificadores» | «Windows detectó dos discos con los mismos identificadores. Mientras dure, los datos que se muestran de ellos pueden confundirse.» |

`pnpm verify:i18n` exige que `es.json` y `en.json` tengan exactamente las mismas claves.

---

## 2. DTO de ocurrencia de alerta — campo nuevo

La cronología del detalle de alerta (`/alerts`) muestra las ocurrencias de un grupo. Cada ocurrencia
gana:

```
triggeringEventId: number | null
```

- `number`: el `system_events.id` del evento del sistema que provocó esa ocurrencia. Presente para
  las ocurrencias de reglas de eventos; `null` para las de SMART/capacidad.
- El detalle de alerta, para una fila con `triggeringEventId != null`, muestra un enlace
  «Ver el suceso» → `/events?focus=<id>` (ver §3).

**Validación de frontera (constitución §XI)**:
- Si el DTO de ocurrencia se genera con `ts-rs` desde Rust: añadir el campo al struct Rust, `cargo
  test` regenera el `.ts`, la puerta de tipos del contrato lo verifica.
- Si se mantiene a mano con Zod: añadir `triggeringEventId: z.number().int().nullable()` al esquema,
  con su prueba de rechazo (un valor no numérico → `ipc.schema_mismatch`).

`docs/ui-contract.md`: documentar el campo en la sección de la ocurrencia de alerta.

---

## 3. Ruta `/events` — parámetro de foco

`/events?focus=<system_events.id>`:

- Al cargar, la pantalla de sucesos **resalta** (o desplaza hasta) el evento con ese id, y opcionalmente
  ajusta el filtro para que sea visible.
- Si el id no existe (evento compactado, base distinta), la pantalla se comporta como sin parámetro
  y no muestra error: un foco perdido no es un fallo.
- El parámetro es de solo lectura de la interfaz; no cambia ningún comando ni consulta nueva
  (`get_system_events` ya devuelve el id).

`docs/ui-contract.md`: documentar el parámetro en la sección de la pantalla de eventos.

---

## 4. Contrato de comportamiento del motor (normativo: `docs/alert-rules.md` §2)

No se reproduce aquí. Cualquier valor que la tabla §2 no fije y que la implementación necesite se
añade a `docs/open-questions.md` §J **antes** de programarlo (research.md D2–D6 son el borrador de
esas entradas). Al cerrar la feature:

- `docs/alert-rules.md`: marcar como implementadas las filas de eventos + `device.removed_unexpected`
  + `inventory.duplicate_id`; anotar la mecánica de la ventana de 60 s en §3.5 si se precisó algo.
- `docs/open-questions.md` J.16: cerrar la parte de `events.*` (queda `temp.above_vendor_critical`).
- `docs/data-model.md` §2/§3: nota de que `alert_occurrences.triggering_event_id` ya se escribe.
