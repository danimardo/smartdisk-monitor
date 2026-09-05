-- Migración 0002 — agregados de retención.
--
-- `docs/data-model.md` §4 exige conservar mínimo, máximo, promedio, primera y última lectura al
-- compactar muestras antiguas; `metric_samples` solo tiene una columna de valor por fila. Diseño
-- registrado en `docs/open-questions.md` J.14 antes de esta migración.

CREATE TABLE metric_aggregates (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT REFERENCES devices (id) ON DELETE CASCADE,
  volume_id TEXT REFERENCES volumes (id) ON DELETE CASCADE,
  metric_key TEXT NOT NULL,
  bucket_start_utc TEXT NOT NULL,
  bucket_end_utc TEXT NOT NULL,
  resolution TEXT NOT NULL CHECK (resolution IN ('five_minutes', 'hourly')),
  value_min REAL,
  value_max REAL,
  value_avg REAL,
  value_first REAL,
  value_last REAL,
  sample_count INTEGER NOT NULL,
  unit TEXT NOT NULL,
  CHECK ((device_id IS NOT NULL) <> (volume_id IS NOT NULL))
);

CREATE INDEX idx_metric_aggregates_device_key_time
  ON metric_aggregates (device_id, metric_key, bucket_start_utc);
CREATE INDEX idx_metric_aggregates_volume_key_time
  ON metric_aggregates (volume_id, metric_key, bucket_start_utc);
