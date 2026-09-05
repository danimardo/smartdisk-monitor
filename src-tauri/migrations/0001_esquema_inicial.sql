-- Migración 0001 — esquema inicial.
--
-- Doce entidades de `docs/data-model.md` §2. SQL numerado: una vez publicada, esta migración no se
-- edita — un cambio posterior es una migración nueva (constitución §V).

PRAGMA foreign_keys = ON;

-- Identidad estable de un disco físico. El firmware queda fuera del fingerprint a propósito:
-- si formara parte de él, actualizarlo partiría el historial en dos entidades.
CREATE TABLE devices (
  id TEXT PRIMARY KEY,
  fingerprint TEXT NOT NULL UNIQUE,
  identity_confidence TEXT NOT NULL CHECK (identity_confidence IN ('serial', 'fingerprint')),
  serial_number TEXT,
  model TEXT NOT NULL,
  manufacturer TEXT,
  firmware TEXT,
  device_type TEXT NOT NULL
    CHECK (device_type IN ('nvme', 'sata_ssd', 'hdd', 'usb', 'virtual', 'raid_logical', 'unknown')),
  bus_type TEXT,
  smartctl_path TEXT,
  capacity_bytes INTEGER,
  alias TEXT,
  monitoring_enabled INTEGER NOT NULL DEFAULT 1 CHECK (monitoring_enabled IN (0, 1)),
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  removed_at TEXT,
  capabilities_json TEXT
);

CREATE TABLE volumes (
  id TEXT PRIMARY KEY,
  volume_guid TEXT NOT NULL UNIQUE,
  label TEXT,
  filesystem TEXT,
  drive_letters_json TEXT,
  capacity_bytes INTEGER,
  free_bytes INTEGER,
  device_mapping_confidence TEXT
    CHECK (device_mapping_confidence IN ('exact', 'inferred', 'unknown')),
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL
);

-- Múltiples discos por volumen y múltiples volúmenes por disco: no se asume que una letra de
-- unidad identifique un disco (principio I).
CREATE TABLE device_volume_links (
  device_id TEXT NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
  volume_id TEXT NOT NULL REFERENCES volumes (id) ON DELETE CASCADE,
  confidence TEXT NOT NULL CHECK (confidence IN ('exact', 'inferred', 'unknown')),
  source TEXT NOT NULL,
  PRIMARY KEY (device_id, volume_id)
);

-- Exactamente uno de device_id / volume_id, garantizado por CHECK: es la restricción que
-- `docs/open-questions.md` J.3 fija como decidida.
CREATE TABLE metric_samples (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT REFERENCES devices (id) ON DELETE CASCADE,
  volume_id TEXT REFERENCES volumes (id) ON DELETE CASCADE,
  metric_key TEXT NOT NULL,
  value_real REAL,
  value_integer INTEGER,
  unit TEXT NOT NULL,
  sampled_at_utc TEXT NOT NULL,
  source TEXT NOT NULL
    CHECK (source IN ('smartctl', 'windows_storage', 'performance_counter', 'filesystem')),
  quality TEXT NOT NULL CHECK (quality IN ('exact', 'inferred', 'vendor_specific', 'stale')),
  resolution TEXT NOT NULL DEFAULT 'raw' CHECK (resolution IN ('raw', 'five_minutes', 'hourly')),
  CHECK ((device_id IS NOT NULL) <> (volume_id IS NOT NULL))
);

CREATE INDEX idx_metric_samples_device_key_time
  ON metric_samples (device_id, metric_key, sampled_at_utc);
CREATE INDEX idx_metric_samples_volume_key_time
  ON metric_samples (volume_id, metric_key, sampled_at_utc);

CREATE TABLE smart_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
  captured_at_utc TEXT NOT NULL,
  smartctl_version TEXT,
  exit_status INTEGER,
  query_status TEXT NOT NULL,
  raw_json_path TEXT,
  fields_json TEXT
);

CREATE INDEX idx_smart_snapshots_device_time ON smart_snapshots (device_id, captured_at_utc);

-- Identidad de un evento: (canal, RecordId), no su fecha — un cambio del reloj no produce
-- duplicados (`open-questions.md` J.7).
CREATE TABLE system_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  channel TEXT NOT NULL,
  record_id INTEGER NOT NULL,
  occurred_at_utc TEXT NOT NULL,
  provider TEXT NOT NULL,
  event_id INTEGER NOT NULL,
  level TEXT NOT NULL CHECK (level IN ('critical', 'error', 'warning', 'information')),
  message TEXT,
  raw_xml TEXT,
  device_id TEXT REFERENCES devices (id) ON DELETE SET NULL,
  volume_id TEXT REFERENCES volumes (id) ON DELETE SET NULL,
  mapping_confidence TEXT NOT NULL CHECK (mapping_confidence IN ('exact', 'inferred', 'unknown')),
  dedup_hash TEXT NOT NULL,
  UNIQUE (channel, record_id)
);

CREATE INDEX idx_system_events_time ON system_events (occurred_at_utc);
CREATE INDEX idx_system_events_device ON system_events (device_id);

CREATE TABLE alert_groups (
  id TEXT PRIMARY KEY,
  deduplication_key TEXT NOT NULL UNIQUE,
  rule_key TEXT NOT NULL,
  target_device_id TEXT REFERENCES devices (id) ON DELETE CASCADE,
  target_volume_id TEXT REFERENCES volumes (id) ON DELETE CASCADE,
  severity TEXT NOT NULL CHECK (severity IN ('warning', 'critical')),
  status TEXT NOT NULL CHECK (status IN ('active', 'acknowledged', 'resolved', 'archived')),
  -- Ortogonal al estado: el silencio nunca decide el color (constitución §I, docs/ui-design.md).
  muted_until TEXT,
  cycle INTEGER NOT NULL DEFAULT 1,
  first_occurrence_at_utc TEXT NOT NULL,
  last_occurrence_at_utc TEXT NOT NULL,
  occurrence_count INTEGER NOT NULL DEFAULT 1,
  acknowledged_at_utc TEXT,
  resolved_at_utc TEXT,
  archived_at_utc TEXT,
  last_value_real REAL,
  context_json TEXT
);

CREATE INDEX idx_alert_groups_status ON alert_groups (status);
CREATE INDEX idx_alert_groups_target ON alert_groups (target_device_id, target_volume_id);

CREATE TABLE alert_occurrences (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  alert_group_id TEXT NOT NULL REFERENCES alert_groups (id) ON DELETE CASCADE,
  cycle INTEGER NOT NULL,
  occurred_at_utc TEXT NOT NULL,
  value_real REAL,
  triggering_event_id INTEGER REFERENCES system_events (id) ON DELETE SET NULL,
  context_json TEXT
);

CREATE INDEX idx_alert_occurrences_group ON alert_occurrences (alert_group_id, cycle);

CREATE TABLE test_runs (
  id TEXT PRIMARY KEY,
  test_type TEXT NOT NULL CHECK (test_type IN ('benchmark', 'chkdsk_scan', 'smart_short')),
  target_device_id TEXT REFERENCES devices (id) ON DELETE SET NULL,
  target_volume_id TEXT REFERENCES volumes (id) ON DELETE SET NULL,
  status TEXT NOT NULL
    CHECK (status IN
      ('pending', 'running', 'cancelling', 'completed', 'failed', 'cancelled', 'interrupted')),
  started_at_utc TEXT,
  finished_at_utc TEXT,
  progress_percent INTEGER,
  result_summary_json TEXT,
  parameters_json TEXT,
  temp_path TEXT
);

CREATE INDEX idx_test_runs_target ON test_runs (target_device_id, target_volume_id);

-- Claves tipadas y versionadas, no un diccionario libre (`docs/ui-contract.md` §3.1).
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'global' CHECK (scope IN ('global', 'device', 'volume')),
  scope_id TEXT,
  schema_version INTEGER NOT NULL DEFAULT 1,
  updated_at_utc TEXT NOT NULL
);

-- Bookmark del registro de eventos, no un RecordId suelto: al limpiar un canal los identificadores
-- se reinician (`open-questions.md` J.7).
CREATE TABLE event_cursors (
  channel TEXT PRIMARY KEY,
  bookmark_blob BLOB,
  updated_at_utc TEXT NOT NULL
);

-- schema_migrations la crea el ejecutor antes de aplicar ninguna migración (persistence/migrations.rs).
