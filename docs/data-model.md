# Modelo de datos

## 1. Principios

- Separar identidad física, topología mutable y observaciones temporales.
- Conservar procedencia y calidad de cada métrica.
- Guardar contadores absolutos para calcular incrementos de forma fiable.
- No asumir que una letra de unidad identifica un disco.
- No marcar como fallo la ausencia de una métrica.

## 2. Entidades principales

### `devices`

- `id`: UUID interno.
- `fingerprint`: identidad calculada estable. Se compone como
  `sha256(model | capacity_bytes | bus_type | wwn_o_pnp_device_id)`. **El firmware queda fuera a
  propósito**: si formara parte de la huella, actualizarlo partiría el historial del disco en dos
  entidades, y la arquitectura pide justo lo contrario, detectar el cambio sobre la misma entidad.
- `identity_confidence`: `serial` o `fingerprint`. La interfaz marca como identidad inferida los
  discos sin número de serie.
- `serial_number`: nullable.
- `model`, `manufacturer`, `firmware`.
- `device_type`: NVMe, SATA SSD, HDD, USB, virtual, RAID logical, unknown.
- `bus_type` y `smartctl_path`.
- `capacity_bytes`.
- `alias`.
- `monitoring_enabled`.
- `first_seen_at`, `last_seen_at`, `removed_at`.
- `capabilities_json`.

### `volumes`

- `id`, `volume_guid`, `label`, `filesystem`.
- `drive_letters_json`.
- `capacity_bytes`, `free_bytes`.
- `device_mapping_confidence`.
- `first_seen_at`, `last_seen_at`.

### `device_volume_links`

- Relación entre discos físicos y volúmenes.
- Permite múltiples discos por volumen y múltiples volúmenes por disco.
- Incluye procedencia y confianza de la asociación.

### `metric_samples`

- `device_id` o `volume_id`.
- Exactamente uno de `device_id` / `volume_id` es no nulo, garantizado por una restricción `CHECK`.
- `metric_key`, `value_real`, `value_integer`, `unit`.
- `sampled_at_utc`.
- `source`: smartctl, Windows Storage, performance counter, filesystem.
- `quality`: exact, inferred, vendor_specific, stale.
- `resolution`: raw, five_minutes, hourly.

### `smart_snapshots`

- `device_id`, `captured_at_utc`.
- Campos normalizados de salud.
- `smartctl_version`, `exit_status` y estado de consulta.
- Ruta o contenido comprimido de JSON bruto cuando proceda.

### `system_events`

- Identidad estable del canal/registro.
- `occurred_at_utc`, `provider`, `event_id`, `level`.
- `message`, `raw_xml` opcional.
- Disco o volumen asociado y confianza de asociación.
- Hash de deduplicación.

### `alert_occurrences`

- `alert_group_id`, `cycle`, `occurred_at_utc`, `value_real`, `context_json`.
- `triggering_event_id`: el `system_events.id` del evento de Windows que provocó esta ocurrencia.
  Lo rellenan las reglas `events.*` (spec `003-puente-eventos-alertas`); `null` para SMART y
  capacidad. El detalle de una alerta lo usa para enlazar al suceso en la pantalla de eventos.

### `alert_groups`

- `id`, `deduplication_key`, `rule_key`.
- Objeto afectado.
- Severidad y estado (`active`, `acknowledged`, `resolved`, `archived`).
- `muted_until`: fecha UTC, `null` o el valor especial de silencio indefinido. **No es un estado**:
  es ortogonal y convive con cualquiera de ellos.
- `cycle`: número de episodio. Un grupo resuelto que recae lo incrementa en vez de crear un grupo
  nuevo, para no perder el contador histórico.
- Primera y última ocurrencia.
- Contador.
- Fechas de reconocimiento, resolución y archivo.
- Último valor y contexto.

### `alert_occurrences`

- `alert_group_id`, `cycle`, fecha, valor, evento y contexto de la ocurrencia.

### `test_runs`

- Tipo: benchmark, chkdsk_scan o smart_short.
- Disco/volumen objetivo.
- Estado: pending, running, cancelling, completed, failed, cancelled, interrupted.
- Inicio, fin, progreso y resultado.
- Parámetros y resumen de métricas.
- Ruta temporal solo mientras sea necesaria.

### `settings`

- Claves tipadas y versionadas.
- Preferencias globales, de disco y de volumen.
- Idioma, tema, frecuencias, retención, cierre y umbrales.
- `storage.free_space_warn_bytes` / `storage.free_space_halt_bytes`: umbrales de espacio libre del
  volumen donde reside el historial. Al cruzar el de aviso se notifica; al cruzar el de parada se
  detiene la escritura de historial sin afectar a la monitorización ni a las alertas en vivo. Valor
  por defecto 1 GB / 256 MB, no medido (`open-questions.md` J.13).
- `logging.verbose`: booleano, modo detallado de registro de actividad (US-071).
- `notifications.enabled`: booleano, **fábrica `true`** (ADR-037). `false` oculta el toast nativo sin
  necesidad de pausar la recopilación. La alerta sigue existiendo y contando para el color de salud.
- `lifecycle.start_with_system`: booleano, **fábrica `false`** (ADR-038). Al activarlo, el backend
  registra una tarea programada elevada (`schtasks /SC ONLOGON /RL HIGHEST`); al desactivarlo o al
  hacer `reset_settings("all")`, la borra. Fuente de verdad = esta clave, no el estado real de la
  tarea.
- **`window.width` / `window.height` / `window.x` / `window.y` / `window.maximized`** (ADR-040):
  geometría de la ventana principal de la última sesión, en **píxeles lógicos**. Enteros y un
  booleano. Las escribe **solo el backend** (al cerrar y al salir), nunca el frontend ni
  `set_setting`. Ausentes ⇒ se usa `tauri.conf.json` (primer arranque: 1695 × 988 centrada). Si la
  posición guardada queda fuera de todo monitor actual, se ignora y la ventana abre centrada. Con
  `window.maximized` activo no se tocan tamaño ni posición: se conservan los previos a maximizar.
  `reset_settings` (ámbito «resto» o «all») las borra.
- **`alerts.profile`** (`cautious` | `balanced` | `quiet` | `custom`) y los umbrales que un perfil
  escribe (ADR-036, `cambios/08b-perfiles-de-alerta.md`). Fábrica: `balanced`. Umbrales nuevos frente
  a v2, con su rango de edición y su valor de fábrica (perfil Equilibrado):

  | Clave | Rango | Fábrica |
  |---|---|---|
  | `alerts.temp_configured_warn_c` | 40–95 | **60** (baja de 70) |
  | `alerts.temp_configured_crit_c` | warn–100 | **70** (baja de 80) |
  | `alerts.wear_warn_percent` | 50–99 | 80 |
  | `alerts.wear_crit_percent` | warn–100 | 90 |
  | `alerts.media_errors_warn_per24h` | 1–1000 | 1 — es el incremento del contador que basta para avisar, **no** una ventana de 24 h (clarify Q1) |
  | `alerts.media_errors_crit_per24h` | warn–1000 | 5 |
  | `alerts.driver_retry_warn_per24h` | 1–1000 | 5 — **aún sin consumidor** (necesita el colector de eventos, Historia 4) |
  | `alerts.driver_retry_crit_per24h` | warn–1000 | 12 |

  Perfiles: Prudente 55/65 · 70/85 · … · Equilibrado (= fábrica) · Solo lo grave 70/80 · 90/95 · …
  Tabla completa en `specs/002-rediseno-v3/data-model.md` y en `cambios/08b`.
- **`settings.onboarding.completed_at`**: fecha ISO-8601 UTC o nula. Nula ⇒ el guardián de
  `+layout.ts` redirige al asistente inicial al arrancar, **salvo** que ya haya configuración previa
  observable (tema ≠ `system`, idioma forzado, perfil de alerta ≠ `balanced`, algún alias o alguna
  exclusión), en cuyo caso la graba y sigue sin mostrarlo (FR-043, sin migración). No la restaura
  `reset_settings`.
- **`volume_free_bytes`** (`metric_samples`, `MetricTarget::Volume`): muestra periódica del espacio
  libre de cada volumen monitorizado, persistida en el ciclo de descubrimiento (ADR-036). Antes la
  capacidad solo vivía como instantánea en `volumes.free_bytes`; ahora también como serie, para que
  `capacity.low`/`capacity.critical` tengan histéresis. La retención la compacta igual que el resto.
- `VolumeSummary` gana `is_system_volume` (booleano): `true` para el volumen donde vive Windows. Lo
  calcula el backend al leer (`GetSystemWindowsDirectoryW`), **sin columna nueva ni migración**.

### `event_cursors`

- Canal/proveedor y **bookmark** del registro de eventos, no un `RecordId` suelto: al limpiar un
  canal los identificadores se reinician, y un cursor numérico se quedaría por delante de los
  eventos nuevos y dejaría de importarlos sin dar ningún error.
- Evita duplicar eventos entre sesiones. La identidad de un evento es `(canal, RecordId)`, no su
  fecha, de modo que un cambio del reloj del sistema tampoco produce duplicados.

### `schema_migrations`

- Versión, fecha y checksum de cada migración aplicada.

### `metric_aggregates`

- `device_id` o `volume_id`, misma restricción de exactamente uno que `metric_samples`.
- `metric_key`, `bucket_start_utc`, `bucket_end_utc`, `resolution` (`five_minutes` o `hourly`).
- `value_min`, `value_max`, `value_avg`, `value_first`, `value_last`, `sample_count`, `unit`.
- El incremento de un contador acumulativo dentro del bucket es `value_last - value_first`; no
  lleva columna propia (`open-questions.md` J.14).

## 3. Métricas normalizadas iniciales

- `temperature_celsius`
- `temperature_sensor_N_celsius`
- `critical_warning`
- `health_passed`
- `percentage_used`
- `available_spare_percent`
- `available_spare_threshold_percent`
- `power_on_hours`
- `power_cycles`
- `unsafe_shutdowns`
- `media_errors_total`
- `error_log_entries_total`
- `data_read_bytes_total`
- `data_written_bytes_total`
- `read_bytes_per_second`
- `write_bytes_per_second`
- `read_latency_ms`
- `write_latency_ms`
- `activity_percent` — porcentaje de tiempo con al menos una operación en curso, derivado de
  `PhysicalDisk\% Idle Time` y acotado a 0–100. **No** se usa `% Disk Time` directamente, que en
  discos con varias operaciones simultáneas supera el 100 % y no es un porcentaje real.
- `volume_free_bytes`
- `volume_free_percent`
- `smart_query_ok` — 1.0 si `smartctl` pudo leer el disco ese ciclo, 0.0 si la consulta falló o
  devolvió algo irreconocible. **No** es una medida del disco: es el resultado del intento de
  consulta, y es la serie sobre la que se evalúa la regla `smart.unreadable` (`alert-rules.md`).
- `vendor_temp_limit_celsius` — límite operativo de temperatura que declara el fabricante
  (`temperature.op_limit_max` de `smartctl`, tabla SCT). Solo lo traen algunos discos SATA; ausente
  en la mayoría de NVMe. Es el umbral de `temp.above_vendor_limit` y de la línea de referencia de la
  gráfica de temperatura. **No** hay equivalente para el crítico del fabricante: `smartctl` no lo
  expone de forma fiable en el JSON (`open-questions.md` J.16).

Los campos no disponibles se omiten; no se almacenan como cero.

## 4. Retención

- Un trabajo diario compacta muestras antiguas dentro de una transacción.
- La agregación conserva mínimo, máximo, promedio, primera y última lectura, además de incrementos
  de contadores, en `metric_aggregates` (§2).
- **Tres periodos, uno por resolución** (US-071, `open-questions.md` J.14): pasado
  `retention.raw_days` (7 por defecto) las muestras `raw` se compactan a `five_minutes`; pasado
  `retention.five_minutes_days` (90) se compactan a `hourly`; pasado `retention.hourly_days` (730)
  se purgan. Valores de partida, no medidos.
- Antes de una migración se crea una copia consistente de SQLite.
- Se conservan las tres copias de migración más recientes.
- Alertas, ocurrencias críticas, eventos vinculados y ejecuciones de pruebas no se borran automáticamente.

## 5. Tiempo y unidades

- Persistencia en UTC.
- Presentación en hora local del sistema.
- Capacidades almacenadas en bytes, caudales en bytes por segundo y temperaturas en Celsius. La
  conversión a unidades legibles ocurre solo en la capa de presentación, en base 1024 con las
  etiquetas KB/MB/GB que usa el Explorador de Windows.
- La interfaz puede presentar unidades legibles sin alterar el dato original.

