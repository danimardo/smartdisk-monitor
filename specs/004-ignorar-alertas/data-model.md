# Phase 1 — Modelo de datos

Feature: Ignorar una alerta de forma permanente (`004-ignorar-alertas`)

Esta feature **no crea entidades nuevas**. Amplía `alert_groups` y su máquina de estados.

---

## `alert_groups` (modificada)

### Cambios de esquema (migración `0003_alerta_ignored.sql`)

| Cambio | Antes | Después |
|---|---|---|
| `CHECK` de `status` | `IN ('active','acknowledged','resolved','archived')` | `IN ('active','acknowledged','resolved','archived','ignored')` |
| Columna nueva | — | `ignored_at_utc TEXT` (nullable), tras `archived_at_utc` |

Todo lo demás (columnas, tipos, `UNIQUE(deduplication_key)`, índices
`idx_alert_groups_status` / `idx_alert_groups_target`, FKs a `devices`/`volumes`) se conserva
idéntico. La reconstrucción de tabla se hace con el procedimiento de R1 (research.md) para no
disparar el `ON DELETE CASCADE` de `alert_occurrences`.

### Campo `ignored_at_utc`

- **Qué es**: instante UTC (RFC 3339) en que el usuario ignoró el grupo por última vez.
- **Se escribe**: en la transición `→ ignored` (`repo_alertas::set_status(Ignored, ahora)`).
- **Se limpia** (`NULL`): al `dejar_de_ignorar` y en cualquier reactivación vía motor
  (`set_status(Active, …)` ya limpia las otras fechas de transición; se le suma `ignored_at_utc`).
- **Presentación**: hora local; en la pestaña «Ignoradas», como «Ignorada el …». No decide color
  ni notificación.

### `severity` mientras el grupo está `ignored`

- Puede **subir** al peor valor visto (`Warning → Critical`) por ocurrencias posteriores
  (`severidad_sube`), nunca baja mientras está ignorado.
- No cuenta para `deviceState()` (ni `active` ni `acknowledged`), así que ese ascenso no tiñe el
  disco.

### `cycle` mientras el grupo está `ignored`

- **No avanza.** No hay frontera de episodio sin transición de estado (aclaración Q3). Las
  ocurrencias registradas mientras el grupo está `ignored` llevan el `cycle` congelado que tenía
  al ignorarlo.

---

## Máquina de estados (`docs/alert-rules.md` §1, ampliada)

### Estados

`active` · `acknowledged` · `resolved` · `archived` · **`ignored`** (nuevo)

### Transiciones nuevas o modificadas

| Desde | Evento | Hasta | Efecto |
|---|---|---|---|
| `active` / `acknowledged` / `resolved` / `archived` | el usuario **ignora** (solo reglas ignorables) | `ignored` | `ignored_at_utc = ahora`; deja de contar para el color; deja de notificar |
| `ignored` | la condición se vuelve a cumplir | `ignored` (sin cambio de estado) | se registra la ocurrencia; `severity` sube si procede; `cycle` **no** avanza; **no** notifica |
| `ignored` | la condición deja de cumplirse | `ignored` (sin cambio) | nada — no pasa a `resolved` solo |
| `ignored` | el usuario **deja de ignorar** | `resolved` | `ignored_at_utc = NULL`, `resolved_at_utc = ahora` |
| `resolved` (venido de «dejar de ignorar») | la condición sigue cumpliéndose, siguiente ciclo | `active`, `cycle + 1` | rama de reactivación existente (`reopen_as_new_cycle`) |

### Transición que se restringe

| Desde | Evento | Antes | Después |
|---|---|---|---|
| `archived` **o `resolved`** | vuelve a cumplirse | cualquier estado no `active`/`acknowledged` reabría | **solo** `archived` y `resolved` reabren; `ignored` **no** |

### Qué cuenta para el color

Sin cambios de principio: `active` y `acknowledged`. `resolved`, `archived` **y `ignored`** no
cuentan. El silencio (`muted_until`) sigue siendo ortogonal y nunca decide el color.

---

## `Transicion` (enum interno de `alerts::agrupacion`)

| Variante | Nueva | Notifica (T053) |
|---|---|---|
| `SinCambio` | | no |
| `CreadaActiva` | | sí |
| `OcurrenciaRepetida` | | según cooldown |
| `Escalada` | | sí |
| `Reactivada` | | sí |
| `Resuelta` | | no |
| **`OcurrenciaIgnorada`** | ✅ | **no** |

---

## Contrato con la interfaz (resumen; detalle en `contracts/comandos-alertas.md`)

- `AlertStatus` (TS, generado): `"active" | "acknowledged" | "resolved" | "archived" | "ignored"`.
- `AlertDetail`: campo nuevo `ruleIgnorable: boolean`.
- `AlertGroup` / `AlertGroupWire`: sin campos nuevos; `status` puede valer `"ignored"`.
- Eventos `alerts:changed`: sin cambios de forma; un grupo recién ignorado viaja en `changed` con
  su `status` nuevo.

---

## Reglas de validación

| Regla | Dónde se aplica |
|---|---|
| Solo se puede pasar a `ignored` si `regla_es_ignorable(rule_key)` | `alerts::ciclo::ignorar` → `AppError alert.rule_not_ignorable` |
| `status` solo admite los cinco valores | `CHECK` en `alert_groups` + `status_from_str` (defensa en profundidad) |
| Un grupo `ignored` no se reactiva por el motor | rama guardada en `agrupacion::procesar` |
| La cronología previa y el contador histórico se conservan al ignorar | ninguna sentencia los borra; `set_status` solo toca `status` y `ignored_at_utc` |
