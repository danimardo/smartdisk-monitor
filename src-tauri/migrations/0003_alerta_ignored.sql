-- Migración 0003 — estado `ignored` para un grupo de alerta.
--
-- La feature `specs/004-ignorar-alertas` añade un estado terminal `ignored` (ADR-044): una alerta
-- ignorada deja de notificar y de contar para el color, y **no** se reactiva sola. Requiere ampliar
-- el `CHECK` de `alert_groups.status` y una columna de fecha `ignored_at_utc`.
--
-- SQLite no permite alterar un `CHECK`: hay que reconstruir `alert_groups`. El obstáculo es que
-- `alert_occurrences.alert_group_id` la referencia con `ON DELETE CASCADE`, y el runner
-- (`persistence/migrations.rs`) corre cada migración dentro de una transacción, donde
-- `PRAGMA foreign_keys` y `PRAGMA legacy_alter_table` son no-ops: no se puede desactivar la
-- comprobación de claves ajenas desde aquí. Con las claves activas, cualquier `DROP TABLE` de una
-- tabla referenciada dispara un DELETE implícito que cascadea y **borra la cronología**
-- (inaceptable, constitución §V).
--
-- Solución sin tocar el runner: (1) mover `alert_occurrences` a una tabla temporal **sin** la clave
-- ajena, (2) reconstruir `alert_groups` ya sin nadie que la referencie, (3) recrear
-- `alert_occurrences` con su clave ajena y devolverle las filas. Los `id` se conservan explícitos.
-- Verificado que no hay disparadores ni vistas sobre ninguna de las dos tablas (0001).

-- (1) Desengancha la cronología de la clave ajena.
CREATE TABLE alert_occurrences_tmp (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  alert_group_id TEXT NOT NULL,
  cycle INTEGER NOT NULL,
  occurred_at_utc TEXT NOT NULL,
  value_real REAL,
  triggering_event_id INTEGER REFERENCES system_events (id) ON DELETE SET NULL,
  context_json TEXT
);
INSERT INTO alert_occurrences_tmp (id, alert_group_id, cycle, occurred_at_utc, value_real, triggering_event_id, context_json)
  SELECT id, alert_group_id, cycle, occurred_at_utc, value_real, triggering_event_id, context_json
  FROM alert_occurrences;
DROP TABLE alert_occurrences;

-- (2) Reconstruye alert_groups: nadie la referencia ya, así que el DROP no cascadea nada.
ALTER TABLE alert_groups RENAME TO alert_groups_old;

CREATE TABLE alert_groups (
  id TEXT PRIMARY KEY,
  deduplication_key TEXT NOT NULL UNIQUE,
  rule_key TEXT NOT NULL,
  target_device_id TEXT REFERENCES devices (id) ON DELETE CASCADE,
  target_volume_id TEXT REFERENCES volumes (id) ON DELETE CASCADE,
  severity TEXT NOT NULL CHECK (severity IN ('warning', 'critical')),
  status TEXT NOT NULL
    CHECK (status IN ('active', 'acknowledged', 'resolved', 'archived', 'ignored')),
  -- Ortogonal al estado: el silencio nunca decide el color (constitución §I, docs/ui-design.md).
  muted_until TEXT,
  cycle INTEGER NOT NULL DEFAULT 1,
  first_occurrence_at_utc TEXT NOT NULL,
  last_occurrence_at_utc TEXT NOT NULL,
  occurrence_count INTEGER NOT NULL DEFAULT 1,
  acknowledged_at_utc TEXT,
  resolved_at_utc TEXT,
  archived_at_utc TEXT,
  ignored_at_utc TEXT,
  last_value_real REAL,
  context_json TEXT
);

INSERT INTO alert_groups (
  id, deduplication_key, rule_key, target_device_id, target_volume_id, severity, status,
  muted_until, cycle, first_occurrence_at_utc, last_occurrence_at_utc, occurrence_count,
  acknowledged_at_utc, resolved_at_utc, archived_at_utc, ignored_at_utc, last_value_real, context_json
)
SELECT
  id, deduplication_key, rule_key, target_device_id, target_volume_id, severity, status,
  muted_until, cycle, first_occurrence_at_utc, last_occurrence_at_utc, occurrence_count,
  acknowledged_at_utc, resolved_at_utc, archived_at_utc, NULL, last_value_real, context_json
FROM alert_groups_old;

DROP TABLE alert_groups_old;

CREATE INDEX idx_alert_groups_status ON alert_groups (status);
CREATE INDEX idx_alert_groups_target ON alert_groups (target_device_id, target_volume_id);

-- (3) Recrea alert_occurrences con su clave ajena y devuelve las filas.
CREATE TABLE alert_occurrences (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  alert_group_id TEXT NOT NULL REFERENCES alert_groups (id) ON DELETE CASCADE,
  cycle INTEGER NOT NULL,
  occurred_at_utc TEXT NOT NULL,
  value_real REAL,
  triggering_event_id INTEGER REFERENCES system_events (id) ON DELETE SET NULL,
  context_json TEXT
);
INSERT INTO alert_occurrences (id, alert_group_id, cycle, occurred_at_utc, value_real, triggering_event_id, context_json)
  SELECT id, alert_group_id, cycle, occurred_at_utc, value_real, triggering_event_id, context_json
  FROM alert_occurrences_tmp;
DROP TABLE alert_occurrences_tmp;

CREATE INDEX idx_alert_occurrences_group ON alert_occurrences (alert_group_id, cycle);
