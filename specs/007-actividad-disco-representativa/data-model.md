# Fase 1 — Modelo de datos: Actividad de disco representativa

No hay cambios de esquema SQLite. Las entidades nuevas son **estado en memoria** del backend y un
**tipo de contrato**. La serie histórica existente cambia de semántica, no de forma.

---

## 1. Entidades en memoria (dominio y estado del proceso)

### 1.1 `MuestraActividad` — `domain/actividad.rs`

Una lectura instantánea del contador, ya derivada.

| Campo | Tipo | Notas |
|---|---|---|
| `instante` | `std::time::Instant` | Reloj **monotónico** (FR-006). Fecha de la muestra. |
| `valor` | `f64` | `activity_percent` 0–100 (`derivar_activity_percent`, ya acotado). |

No se persiste. No cruza la frontera IPC.

### 1.2 `VentanaActividad` — `domain/actividad.rs` (puro, test-first)

Ventana deslizante por disco. **Sin** dependencias de PDH, Tauri ni SQLite (constitución §IV).

| Campo | Tipo | Notas |
|---|---|---|
| `ventana` | `Duration` | = `schedule.metrics_fast_seconds`. Periodo que la ventana pretende cubrir. |
| `umbral_hueco` | `Duration` | 3 × intervalo de muestreo (R3). |
| `muestras` | `VecDeque<MuestraActividad>` | Orden temporal; se purga por el frente. |

Operaciones:

- `registrar(&mut self, ahora: Instant, valor: f64)`:
  1. Si `muestras` no está vacía y `ahora − última.instante > umbral_hueco` → **vaciar** `muestras`
     (corte por hueco, FR-007).
  2. Empujar `MuestraActividad { instante: ahora, valor }`.
  3. Descartar por el frente toda muestra con `ahora − instante > ventana`.
- `agregado(&self, ahora: Instant) -> AgregadoActividad` — ver 1.3.
- `purgar(&mut self, ahora: Instant)` — descarta muestras fuera de ventana sin registrar una nueva
  (para recalcular el agregado entre muestreos si hiciera falta).

Reglas de validación / invariantes probadas (FR-021):

- Media = media aritmética de `valor` de las muestras en ventana. Pico = máximo.
- `agregado` sobre ventana vacía → `EstadoActividad::NoDisponible`, medias `None`.
- `agregado` cuando `ahora − muestras.front().instante < ventana` (menos una tolerancia) →
  `EstadoActividad::Parcial`, con media y pico de lo que haya.
- En cualquier otro caso → `EstadoActividad::Valido`.
- Una muestra fallida **nunca** se registra como `0.0` (el colector no llama a `registrar` en ese
  caso; ver 2).
- Un cambio de hora del sistema no altera la ventana (usa `Instant`).

### 1.3 `AgregadoActividad` / `EstadoActividad` — `domain/actividad.rs`, viaja al contrato

| Campo | Tipo | Notas |
|---|---|---|
| `estado` | `EstadoActividad` | `Valido` \| `Parcial` \| `NoDisponible`. |
| `media_percent` | `Option<f64>` | `Some` si `estado != NoDisponible`. |
| `pico_percent` | `Option<f64>` | idem. |
| `muestras` | `u32` | nº de muestras que respaldan la ventana. |
| `ventana_segundos` | `u32` | `ventana.as_secs()`. |

### 1.4 `AppState.actividad` — `persistence/db.rs`

```rust
pub actividad: std::sync::Mutex<std::collections::HashMap<String, AgregadoActividad>>,
```

- Clave: `Device.id` (identificador interno, no número de serie — §XV).
- Escrito por el hilo del planificador tras cada muestreo (publicación del agregado por disco).
- Leído por `enrich_with_smart_data` (comandos `get_devices`, `get_device_detail`, y
  `emitir_metrics_updated`).
- **No se persiste**, como `paused`, `source_health`, `notified_at`: un reinicio arranca con el
  mapa vacío → primeros ~30 s en `Parcial`/`NoDisponible` (constitución §I, FR-004).

### 1.5 `ConsultaActividad` — `collectors/perf_counters.rs` (solo Windows)

Consulta PDH persistente. Vive **exclusivamente** en el hilo del planificador (los `HQUERY`/
`HCOUNTER` no cruzan de hilo).

| Campo | Tipo | Notas |
|---|---|---|
| `h_query` | `PDH_HQUERY` | Abierta una vez; cerrada al reconstruir o al terminar. |
| `contadores` | `Vec<(i64 /*disk_number*/, String /*device_id*/, PDH_HCOUNTER)>` | Un `% Idle Time` por disco. |
| `primera_recogida_hecha` | `bool` | La 1.ª recogida no produce valores válidos (R1). |

Operaciones:

- `reconstruir(discos: &[(String, i64)]) -> Result<ConsultaActividad, ErrorPdh>` — abre `HQUERY`,
  añade un contador por disco (`\PhysicalDisk(<n> *)\% Idle Time`, `PdhAddEnglishCounterW`).
- `muestrear(&mut self) -> HashMap<String, Result<f64, ErrorPdh>>` — un `PdhCollectQueryData`;
  si es la primera, marca la bandera y devuelve el mapa vacío; si no, formatea cada contador y
  aplica `derivar_activity_percent`. `c_status` distinto de válido → `Err` para ese disco.
- `Drop` → `PdhCloseQuery`.

### 1.6 `LecturaRendimiento` — cambio

Se **elimina** el campo `activity_percent`. Queda con `read_bytes_per_second`,
`write_bytes_per_second`, `read_latency_ms`, `write_latency_ms`. `leer()` deja de añadir el contador
`% Idle Time` y de derivar la actividad; sigue con su `sleep(1 s)` para las tasas.

`derivar_activity_percent` **se conserva** (la usa ahora `ConsultaActividad::muestrear`); sus
pruebas siguen válidas.

---

## 2. Flujo de una muestra

```
bucle del planificador (cada 1 s, o cada 4 s en batería; nunca en pausa)
  └─ ¿toca refrescar el conjunto de discos? (1×/ciclo de métricas rápidas, o tras alta/baja)
  │     └─ conjunto cambió → ConsultaActividad::reconstruir(...)
  └─ mapa = ConsultaActividad::muestrear()
  └─ para cada (device_id, resultado) del mapa:
  │     ├─ Ok(valor)  → ventana[device_id].registrar(Instant::now(), valor)
  │     └─ Err(_)     → NO se registra nada (la ventana envejece); se cuenta el fallo
  └─ para cada device_id vigilado:
  │     └─ AppState.actividad[device_id] = ventana[device_id].agregado(Instant::now())
  └─ (si TODOS fallaron este ciclo de métricas rápidas) → actualizar_source_health(PerformanceCounter, …)

ciclo de «métricas rápidas» (cada ~30 s), fase 3 de refresh_metricas_rendimiento:
  └─ para cada disco:
        agregado = AppState.actividad[device_id]
        si agregado.estado == Valido → persistir metric_sample("activity_percent", agregado.media_percent)
        si no                        → NO se escribe fila (FR-010a)

emitir_metrics_updated / enrich_with_smart_data:
  └─ resumen.activity = AppState.actividad[device_id]  (o ActividadDisco::no_disponible() si falta)
```

---

## 3. Serie histórica `activity_percent` (`metric_samples`) — semántica revisada

| Antes | Después |
|---|---|
| Valor instantáneo de una ventana PDH de 1 s, una fila cada ~30 s. | **Media** de la ventana deslizante en el instante del ciclo, una fila cada ~30 s. |
| Siempre se escribía fila si `leer()` tuvo éxito. | **No se escribe fila** si la ventana está `Parcial`/`NoDisponible` (FR-010a) → hueco en la gráfica, dibujado como hueco (E.1), nunca interpolado. |
| Unidad `percent`, rango 0–100, en bruto. | Sin cambios. |

`docs/data-model.md` §3, viñeta `activity_percent`: se actualiza el texto para decir «media de una
ventana deslizante del tamaño de la cadencia de métricas rápidas», manteniendo la nota de que **no**
se usa `% Disk Time`.

---

## 4. Contrato IPC — resumen (detalle en `contracts/disk-activity.md`)

| Tipo | Antes | Después |
|---|---|---|
| `DiskSummary` (Rust + `generated/DiskSummary.ts` + `design/types.ts`) | `activity_percent: Option<f64>` / `activityPercent: number \| null` | `activity: ActividadDisco` / `activity: ActividadDisco` |
| `DeviceDetail` | hereda `activity_percent` por `#[serde(flatten)]` | hereda `activity` |
| `generated/ActividadDisco.ts`, `generated/EstadoActividad.ts` | — | nuevos (los genera `cargo test` vía `ts-rs`) |
| `schemas.ts` `diskSummary` | `activityPercent: nullableNumber` | `activity: actividadDisco` (objeto Zod nuevo) + prueba de rechazo |
| Evento `metrics:updated` | `devices: DiskSummary[]` | igual forma; cada `DiskSummary` trae `activity` en vez de `activityPercent` |
