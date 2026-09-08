# Phase 0 — Investigación y decisiones técnicas

Feature: Ignorar una alerta de forma permanente (`004-ignorar-alertas`)

No había marcadores `NEEDS CLARIFICATION` en la spec. Las decisiones de abajo resuelven las
preguntas de diseño que surgen al mapear la spec sobre el código existente.

---

## R1. Cómo ampliar el `CHECK (status IN ...)` de `alert_groups` sin perder la cronología

> **Actualizado durante la implementación (2026-09-08)**: el plan inicial usaba
> `PRAGMA legacy_alter_table` para renombrar `alert_groups` sin arrastrar la clave ajena de
> `alert_occurrences`. **No funciona**: dentro de la transacción que abre el runner por cada
> migración, `PRAGMA legacy_alter_table` es un no-op igual que `PRAGMA foreign_keys`, así que el
> `ALTER TABLE ... RENAME` usa el comportamiento moderno, reescribe la FK de `alert_occurrences`
> hacia `alert_groups_old`, y el `DROP TABLE alert_groups_old` cascadea y **borra la cronología**
> (lo detectó la prueba `la_0003_amplia_alert_groups_sin_perder_la_cronologia`). Enfoque final
> abajo: desenganchar la cronología de la FK, reconstruir `alert_groups`, recrear
> `alert_occurrences` con la FK y devolver las filas. Sin tocar el runner.

**Decisión (final)**: migración `0003` en tres pasos, todo dentro de la transacción del runner:

1. `CREATE TABLE alert_occurrences_tmp` **sin** la clave ajena `alert_group_id → alert_groups`;
   copiar filas (con `id` explícito); `DROP TABLE alert_occurrences`.
2. `ALTER TABLE alert_groups RENAME TO alert_groups_old`; `CREATE TABLE alert_groups` con el `CHECK`
   ampliado y `ignored_at_utc`; `INSERT ... SELECT ..., NULL FROM alert_groups_old`;
   `DROP TABLE alert_groups_old` (ya no la referencia nadie → no cascadea); recrear sus dos índices.
3. `CREATE TABLE alert_occurrences` con la clave ajena de vuelta; `INSERT ... SELECT ... FROM
   alert_occurrences_tmp` (id explícito); `DROP TABLE alert_occurrences_tmp`; recrear
   `idx_alert_occurrences_group`.

**Descartado (documentado arriba)**: `PRAGMA legacy_alter_table` dentro de la transacción del
runner.

---

### Nota histórica — planteamiento inicial (no válido)

Este era el orden de sentencias que se creyó suficiente y que la prueba tumbó:

```sql
PRAGMA legacy_alter_table = ON;
ALTER TABLE alert_groups RENAME TO alert_groups_old;
PRAGMA legacy_alter_table = OFF;

CREATE TABLE alert_groups ( ...  -- idéntica, salvo:
  status TEXT NOT NULL CHECK (status IN ('active','acknowledged','resolved','archived','ignored')),
  ...
  ignored_at_utc TEXT,           -- NUEVA columna, tras archived_at_utc
  ...
);

INSERT INTO alert_groups (<todas las columnas viejas>, ignored_at_utc)
  SELECT <todas las columnas viejas>, NULL FROM alert_groups_old;

DROP TABLE alert_groups_old;

CREATE INDEX idx_alert_groups_status ON alert_groups (status);
CREATE INDEX idx_alert_groups_target ON alert_groups (target_device_id, target_volume_id);
```

**Rationale**:

- El runner (`persistence/migrations.rs`) envuelve **cada** migración en `conn.transaction()`, y
  `PRAGMA foreign_keys` es un no-op dentro de una transacción. No se puede desactivar la
  comprobación de claves ajenas desde el SQL de la migración, y el runner es código sensible que no
  toca esta feature.
- Con `foreign_keys = ON`, un `DROP TABLE alert_groups` haría un `DELETE` implícito que dispara el
  `ON DELETE CASCADE` de `alert_occurrences.alert_group_id` → **se borraría toda la cronología**.
  Inaceptable (constitución §V: la retención nunca borra ocurrencias).
- `PRAGMA legacy_alter_table = ON` **sí** es válido dentro de una transacción. Hace que
  `ALTER TABLE ... RENAME TO` cambie solo el nombre de la tabla y **no** reescriba las referencias
  de clave ajena de otras tablas. Tras el rename, la FK de `alert_occurrences` sigue apuntando por
  nombre a `"alert_groups"` (no a `"alert_groups_old"`), así que `DROP TABLE alert_groups_old` no
  tiene ninguna FK entrante y no cascadea nada.
- Los índices `idx_alert_groups_*` viajan con la tabla renombrada; se recrean **después** del
  `DROP` para no colisionar por nombre.
- No hay disparadores ni vistas sobre `alert_groups` en el esquema (verificado en
  `0001_esquema_inicial.sql`), así que el rename no arrastra nada más.

**Alternativas consideradas**:

- *Modificar el runner para desactivar `foreign_keys` alrededor del lote de migraciones y validar
  con `PRAGMA foreign_key_check`*: es el procedimiento oficial de SQLite de 12 pasos, pero cambia
  infraestructura de persistencia protegida y exigiría su propio ADR. Se reserva por si una
  migración futura no puede resolverse con el truco de `legacy_alter_table`.
- *Quitar el `CHECK` de `status`*: va contra el estilo del esquema (defensa en profundidad con
  `CHECK` en casi toda columna enumerada). Rechazada.
- *No añadir `ignored_at_utc` y reutilizar `archived_at_utc`*: mezcla dos estados distintos y
  rompe la simetría de «una fecha por transición de usuario». Rechazada.

**Prueba**: `persistence::migrations::tests` gana un caso que, partiendo de una base en versión 2
con un grupo y varias ocurrencias, aplica la 3 y verifica que (a) el grupo y **todas** sus
ocurrencias siguen ahí, (b) `INSERT ... status='ignored'` ahora se acepta y antes fallaba,
(c) `PRAGMA foreign_key_check` no devuelve filas.

---

## R2. Dónde y cómo se representa el estado `ignored` en el motor

**Decisión**: `AlertStatus::Ignored` en `domain/tipos.rs` (`#[serde rename_all = "snake_case"]` ya
produce `"ignored"`; `#[ts(export)]` regenera `AlertStatus.ts` en `cargo test`).
`repo_alertas::status_to_str` / `status_from_str` ganan el par `"ignored"`.

En `alerts::agrupacion::procesar`, el `match (existente, ev.severity_si_activa)` gana dos ramas
**antes** de la rama catch-all de reactivación:

```rust
// Ignorado que se vuelve a cumplir: se apunta la ocurrencia y sube la severidad al peor
// valor visto, pero NO se incrementa `cycle` ni se reactiva (spec FR-003, FR-006).
(Some(g), Some(severidad)) if g.status == AlertStatus::Ignored => {
    repo_alertas::record_occurrence(conn, &g.id, g.cycle, &ev.occurred_at_utc, ev.value, None)?;
    if severidad_sube(severidad, g.severity) {
        repo_alertas::set_severity(conn, &g.id, severidad)?;
    }
    Ok(Transicion::OcurrenciaIgnorada)
}
```

La rama catch-all de reactivación pasa de `(Some(g), Some(severidad)) => reopen_as_new_cycle` a
`(Some(g), Some(severidad)) if matches!(g.status, Resolved | Archived) => reopen_as_new_cycle`, y
lo demás cae en `_ => SinCambio`. Así un grupo `ignored` nunca entra en `reopen_as_new_cycle`.

El caso `(Some(g), None)` con `g.status == Ignored` ya lo cubre el `_ => SinCambio` existente (solo
`Active`/`Acknowledged` + `ev.resuelto` hacen algo): un grupo ignorado **no** transiciona a
`resolved` solo porque su condición deje de cumplirse.

**Rationale**: mantiene `procesar` como única autoridad del ciclo de vida; el compilador obliga a
tratar la nueva `Transicion` en `notificaciones::debe_enviar` (se mapea a `false`) y en cualquier
`match` exhaustivo. `severidad_sube` ya existe y solo devuelve `true` en `Warning → Critical`, que
es justo «peor valor visto» para dos niveles.

**Alternativas consideradas**:

- *Devolver `Transicion::SinCambio` tras registrar la ocurrencia del grupo ignorado*: mentiría
  («no hubo ocurrencia nueva») y haría frágil cualquier consumidor de `Transicion`. Rechazada a
  favor de una variante explícita.
- *No registrar ocurrencias mientras está `ignored`*: contradice la aclaración Q3 y la spec
  (US-1 escenario 2). Rechazada.

---

## R3. Semántica de «Ignorar» y «Dejar de ignorar» como acciones de usuario

**Decisión**: en `alerts::ciclo`, simétrico a `archivar` / `reconocer`:

```rust
pub fn ignorar(conn, id, ahora_utc) -> Result<(), IgnorarError> {
    let g = repo_alertas::get_group(conn, id)?.ok_or(IgnorarError::NoExiste)?;
    if !regla_es_ignorable(&g.rule_key) {
        return Err(IgnorarError::ReglaNoIgnorable);
    }
    repo_alertas::set_status(conn, id, AlertStatus::Ignored, ahora_utc)  // set ignored_at_utc
}

pub fn dejar_de_ignorar(conn, id) -> rusqlite::Result<()> {
    // A `resolved`: si la condición sigue, el motor lo sube a `active` en el ciclo siguiente
    // (rama de reactivación de `procesar`); si no, se queda resuelto. (spec FR-011)
    repo_alertas::dejar_de_ignorar(conn, id)  // status='resolved', resolved_at_utc=ahora, ignored_at_utc=NULL
}
```

`repo_alertas::set_status` gana un brazo `AlertStatus::Ignored => UPDATE ... status=?, ignored_at_utc=?`.
El brazo `AlertStatus::Active` (que ya limpia `resolved_at_utc`/`archived_at_utc`) añade
`ignored_at_utc = NULL` para cubrir el caso de reactivación desde `ignored` vía motor — aunque el
camino normal de salida es `dejar_de_ignorar`.

**Rationale**: «dejar de ignorar» → `resolved` reutiliza la rama de reactivación existente de
`procesar` sin tocar el motor. La spec ya asume «no fuerza una evaluación inmediata».

**Alternativas consideradas**:

- *«Dejar de ignorar» pone `active` directamente*: obligaría a re-evaluar la condición en el
  comando (leer series de `metric_samples`, replicar `motor.rs`) o arriesgarse a mostrar `active`
  una alerta que ya no aplica. Rechazada: `resolved` + el ciclo natural es más honesto y más
  simple.
- *Estado intermedio «pendiente de re-evaluar»*: complejidad sin valor observable. Rechazada.

---

## R4. Dónde vive el conjunto de reglas no ignorables

**Decisión**: `src-tauri/src/alerts/reglas.rs` (módulo nuevo y pequeño) con:

```rust
/// Reglas que señalan daño físico o fallo inminente del disco: nunca se pueden ignorar
/// (spec FR-012, docs/alert-rules.md, ADR-044).
pub const REGLAS_NO_IGNORABLES: &[&str] = &[
    "smart.health.failed",
    "nvme.critical_warning",
    "smart.wear_high",
    "smart.spare_below_threshold",
    "smart.media_errors",
    "smart.error_log",
    "events.disk_predictive",
];

pub fn regla_es_ignorable(rule_key: &str) -> bool {
    !REGLAS_NO_IGNORABLES.contains(&rule_key)
}
```

Es una decisión pura sobre un `&str`; encaja con `alerts::notificaciones::politica_notificacion`,
que también clasifica por `rule_key` y vive en `alerts/`. El frontend **no** replica la lista: el
contrato expone `ruleIgnorable: bool` en `AlertDetail` (R5).

**Rationale**: una sola fuente de verdad en Rust, verificable con una prueba que recorre la tabla
de `docs/alert-rules.md`. `alerts/` sobre `domain/` porque el resto de lógica por `rule_key` ya
está en `alerts/` y `domain/` no tiene hoy ningún módulo de reglas de alerta.

**Alternativas consideradas**:

- *Columna/flag configurable por el usuario*: la spec lo descarta explícitamente — es regla de
  negocio, no preferencia. Rechazada.
- *Constante en `docs/` parseada en runtime*: `alert-rules.md` es normativo pero no es un formato
  de datos (constitución/`backend-rust.md`: los documentos no alimentan decisiones). La prueba es
  la que mantiene el doc y el código sincronizados.

---

## R5. Cómo sabe el frontend si una alerta es ignorable

**Decisión**: `AlertDetail` (el objeto que consume el panel de detalle) gana un campo de nivel
superior `rule_ignorable: bool` (→ `ruleIgnorable` en el contrato camelCase), calculado en
`get_alert_detail_impl` con `regla_es_ignorable(&grupo.rule_key)`. `schemas.ts::alertDetail` lo
añade como `z.boolean()`.

El botón «Ignorar» de `+page.svelte`:

- se muestra cuando `detail.status !== "ignored"` (y no `archived`);
- va **deshabilitado** con `disabledReason={t("alerts.ignore.notIgnorable")}` cuando
  `!detail.ruleIgnorable`;
- abre `ConfirmDialog` cuando está activo.

**Rationale**: mantener la lista de reglas solo en Rust (§IV: ningún componente decide reglas de
negocio). `AlertDetail` y no `AlertGroupWire` porque la acción vive en el panel de detalle; la
tarjeta de la lista no necesita el dato. `AlertGroupWire` sigue sin cambios de forma salvo el
`status` ampliado.

**Alternativas consideradas**:

- *Campo en `AlertGroupWire`*: lo cargaría en cada tarjeta de la lista y en la carga de `alerts`
  completa sin necesidad. Rechazada.
- *Comando aparte `is_rule_ignorable(ruleKey)`*: un viaje extra por dato que ya viaja con el
  detalle. Rechazada.

---

## R6. Pestaña «Ignoradas» y estados de la pantalla

**Decisión**: quinto segmento en el `SegmentedControl` de `/alerts`, siempre visible, junto a
Activas / Resueltas / Archivadas / Todas:

- `Filtro` gana `"ignored"`; `ESTADOS_POR_FILTRO.ignored = ["ignored"]`.
- `all` sigue siendo `null` (muestra todo, incluidas las ignoradas).
- El estado vacío reutiliza `alerts.empty.title` / `alerts.empty.body` (ya genérico: «No hay
  alertas que coincidan con este filtro»).
- `AlertCard` ya compone `t(\`alert.status.${status}\`)` — basta con la clave `alert.status.ignored`.

**Rationale**: coherencia total con el patrón existente de filtros; `SegmentedControl`
(`px-3 py-1 text-xs`, 5 segmentos) entra de sobra en la ventana mínima de 1024. Sin estado nuevo
que diseñar.

**Alternativas consideradas**:

- *Pestaña oculta cuando no hay ignoradas*: rompería la posición estable de los segmentos y haría
  que la acción «Ignorar» pareciera no tener consecuencia visible la primera vez. Rechazada.
- *Sección aparte fuera del `SegmentedControl`*: incoherente con Archivadas, que sí es un segmento.
  Rechazada.

---

## R7. Retención

**Decisión**: sin cambios. `domain::retencion` solo compacta `metric_samples`; nunca toca
`alert_groups` ni `alert_occurrences`. FR-016 se cumple por construcción. Se añade una asсерción a
la prueba de retención existente solo si resulta barato; si no, se documenta que no hay ruta de
borrado.

---

## R8. Notificaciones

**Decisión**: `notificaciones::debe_enviar` gana el brazo `Transicion::OcurrenciaIgnorada => false`
(forzado por el compilador). El resto de la función no cambia. Un grupo `ignored` nunca llega a
`enviar_y_registrar` porque `procesar` nunca emite para él otra `Transicion` que notifique.

---

## Resumen de decisiones

| # | Decisión |
|---|---|
| R1 | Migración `0003` reconstruye `alert_groups` con `PRAGMA legacy_alter_table`; sin tocar el runner |
| R2 | `AlertStatus::Ignored` + dos ramas en `agrupacion::procesar` + `Transicion::OcurrenciaIgnorada`; `cycle` no avanza |
| R3 | `ciclo::ignorar` (rechaza reglas vetadas) / `ciclo::dejar_de_ignorar` → `resolved`, sin re-evaluación inmediata |
| R4 | `alerts::reglas::REGLAS_NO_IGNORABLES` + `regla_es_ignorable`, única fuente de verdad, con prueba contra `alert-rules.md` |
| R5 | `AlertDetail.ruleIgnorable: bool` calculado en backend; el frontend no replica la lista |
| R6 | 5.º segmento «Ignoradas» siempre visible; estados vacíos reutilizados |
| R7 | Retención sin cambios (ya nunca borra alertas) |
| R8 | `debe_enviar`: `OcurrenciaIgnorada => false` |
