# Contrato — Actividad de disco

Cubre la frontera **UI ↔ backend** (normativa: `docs/ui-contract.md`) y la **API interna de Rust**
que la alimenta. La forma canónica la fija el `.ts` generado por `ts-rs`; este documento describe la
intención y el antes/después.

---

## 1. Frontera UI ↔ backend

### 1.1 Tipo nuevo `ActividadDisco`

```ts
// src/lib/api/generated/ActividadDisco.ts  (generado)
export type EstadoActividad = "valido" | "parcial" | "no_disponible";

export type ActividadDisco = {
  estado: EstadoActividad;
  /** Media de la ventana deslizante, 0–100. null solo si estado === "no_disponible". */
  mediaPercent: number | null;
  /** Pico (máximo) de la ventana, 0–100. null solo si estado === "no_disponible". */
  picoPercent: number | null;
  /** Nº de muestras que respaldan la ventana en este instante. */
  muestras: number;
  /** Periodo que la ventana pretende cubrir, en segundos (= cadencia de métricas rápidas). */
  ventanaSegundos: number;
};
```

Rust (`commands/mod.rs`), fuente de la generación:

```rust
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct ActividadDisco {
    pub estado: EstadoActividad,
    pub media_percent: Option<f64>,
    pub pico_percent: Option<f64>,
    pub muestras: u32,
    pub ventana_segundos: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum EstadoActividad { Valido, Parcial, NoDisponible }
```

Constructor de conveniencia: `ActividadDisco::no_disponible(ventana_segundos)` para el disco que aún
no tiene ventana (arranque) o cuya fuente está degradada.

**Sin campo de procedencia ni de marca de tiempo** (`research.md` R9): la procedencia es siempre
«contadores de rendimiento» (rótulo fijo en la interfaz) y el agregado es actual por construcción,
así que `estado` es la única señal de fiabilidad que se transporta.

### 1.2 Cambio en `DiskSummary` / `DeviceDetail`

| | Antes | Después |
|---|---|---|
| Rust `DiskSummary` | `pub activity_percent: Option<f64>,` | `pub activity: ActividadDisco,` |
| `generated/DiskSummary.ts` | `activityPercent: number \| null` | `activity: ActividadDisco` |
| `generated/DeviceDetail.ts` | hereda `activityPercent` | hereda `activity` |
| `src/lib/design/types.ts` `DiskSummary` | `activityPercent: number \| null;` | `activity: ActividadDisco;` |

`device_to_summary` (el constructor «vacío») pasa a poner
`activity: ActividadDisco::no_disponible(cadencia)` en vez de `activity_percent: None`.

### 1.3 Esquema Zod (`src/lib/api/schemas.ts`)

```ts
export const estadoActividad = z.enum(["valido", "parcial", "no_disponible"]);

export const actividadDisco = z.object({
  estado: estadoActividad,
  mediaPercent: nullableNumber,
  picoPercent: nullableNumber,
  muestras: z.number().int().nonnegative(),
  ventanaSegundos: z.number().int().positive()
});

export const diskSummary = z.object({
  // …
  activity: actividadDisco,          // sustituye a  activityPercent: nullableNumber
  // …
});
```

**Prueba de rechazo obligatoria** (constitución §XI, `schemas.test.ts`): un `DiskSummary` cuyo
`activity` sea un número suelto, o cuyo `estado` sea `"desconocido"`, o al que le falte
`ventanaSegundos`, debe fallar la validación con `ipc.schema_mismatch` y la ruta del campo en el
detalle. Actualizar el fixture `discoValido` de `schemas.test.ts` (hoy `activityPercent: 12`).

### 1.4 Evento `metrics:updated`

Sin cambio de forma: `{ emittedAt, devices: DiskSummary[], sources: SourceHealth[],
historyWriteHalted }`. Cada `DiskSummary` del array trae ahora `activity: ActividadDisco`. La
cadencia de emisión **no cambia** (una vez por ciclo de recopilación; FR-022). No se añade evento.

### 1.5 Presentación (no es contrato, orienta a tasks)

- **`DiskCard`** (panel): la magnitud «actividad» muestra `activity.picoPercent` con
  `formatPercent`. `estado === "no_disponible"` → «—» discreto (patrón `DiskCard.md`), con el texto
  completo en `title` vía `t()`. `estado === "parcial"` → se muestra el valor con una marca de
  «midiendo aún» (icono o sufijo, nunca solo color; §VI).
- **`MetricCard`** (detalle): valor principal = media; el pico se muestra como dato secundario
  («pico N %»). La sparkline de US3 (si entra) usa las muestras de la ventana.
- **`metricHelp.ts`** veredicto `activity`: nuevos casos `parcial` y `no_disponible`; el caso
  `info` pasa a hablar de «media de los últimos {ventanaSegundos} s, pico {pico} %».
- **i18n** (`es.json` / `en.json`, mismas claves): `metric.help.activity.body` revisado;
  `metric.help.activity.verdict.info` con `{media}` `{pico}` `{ventana}`;
  `metric.help.activity.verdict.parcial`; `metric.help.activity.verdict.unknown` ya existe.
  Etiquetas `disk.activityPeak` / `disk.activityMean` si el detalle las necesita.

---

## 2. API interna de Rust

### 2.1 `domain::actividad` (puro — test-first, constitución §VIII)

```rust
pub struct MuestraActividad { pub instante: Instant, pub valor: f64 }

pub struct VentanaActividad { /* ventana, umbral_hueco, muestras: VecDeque<MuestraActividad> */ }

impl VentanaActividad {
    pub fn nueva(ventana: Duration, umbral_hueco: Duration) -> Self;
    pub fn registrar(&mut self, ahora: Instant, valor: f64);   // corta por hueco, empuja, purga
    pub fn agregado(&self, ahora: Instant) -> AgregadoActividad;
}

pub struct AgregadoActividad {
    pub estado: EstadoActividad,
    pub media_percent: Option<f64>,
    pub pico_percent: Option<f64>,
    pub muestras: u32,
    pub ventana_segundos: u32,
}
```

`AgregadoActividad` es también el cuerpo del DTO `ActividadDisco` (o se convierte a él con un `From`
trivial si se quiere separar dominio de wire; decisión de estilo para tasks).

Pruebas mínimas (FR-021), todas con `Instant` sintético:
1. media y pico correctos sobre una ventana de muestras conocida;
2. ventana que aún no cubre la cadencia → `Parcial` con media/pico de lo que hay;
3. ventana vacía → `NoDisponible`, medias `None`;
4. una muestra separada > umbral de hueco de la anterior → la ventana se vacía antes de agregar;
5. las muestras viejas (> ventana) se descartan por el frente;
6. no se puede colar un `0.0` «de relleno»: `registrar` solo se llama con lecturas reales.

### 2.2 `collectors::perf_counters` (solo Windows)

```rust
pub struct ConsultaActividad { /* h_query, contadores, primera_recogida_hecha */ }

impl ConsultaActividad {
    pub fn reconstruir(discos: &[(String /*device_id*/, i64 /*disk_number*/)])
        -> Result<Self, ErrorPdh>;
    pub fn muestrear(&mut self) -> HashMap<String, Result<f64, ErrorPdh>>;
    pub fn discos(&self) -> Vec<(String, i64)>;   // conjunto actual, para detectar cambios
}
impl Drop for ConsultaActividad { /* PdhCloseQuery */ }
```

- Constantes FFI nuevas: `PDH_CSTATUS_NEW_DATA` (0), `PDH_CSTATUS_VALID_DATA` (0),
  `PDH_CSTATUS_INVALID_DATA` (0xC0000BC6), `PDH_CSTATUS_NO_DATA` (0x800007D5). Ningún símbolo de
  función nuevo (reconstrucción total, R2).
- No probable en `cargo test` sin hardware; se cubre con la validación manual de `quickstart.md` y
  con la prueba de `domain::actividad` para toda la lógica sin PDH.

### 2.3 `commands` — puntos de cambio

| Función | Cambio |
|---|---|
| `iniciar_planificador` | Contador de ticks; cada tick (o 1 de cada 4 en batería, no en pausa): reconstruir `ConsultaActividad` si el conjunto cambió, `muestrear`, `registrar` por disco, publicar `AgregadoActividad` en `AppState.actividad`. Estado local del hilo: `Option<ConsultaActividad>` + `HashMap<String, VentanaActividad>`. |
| `refresh_metricas_rendimiento` | Fase 3: además de persistir caudal/latencias de `leer()`, persistir `activity_percent = media` **solo si** `AppState.actividad[id].estado == Valido`. Contabilizar el fallo de muestreo de actividad para `source_health` si procede. |
| `enrich_with_smart_data` | Sustituir la lectura de `latest_device_sample("activity_percent")` por `AppState.actividad.get(&d.id).cloned().unwrap_or(ActividadDisco::no_disponible(cadencia))`. Nota: necesita acceso a `AppState` (hoy recibe `&Connection`); pasar el agregado como parámetro o el mapa ya bloqueado. |
| `DiskSummary` / `device_to_summary` / `MetricsUpdatedEvent` | Campo `activity`. |
| `LecturaRendimiento` / `persist_perf_reading` / `perf_counters::leer` | Quitar `activity_percent`. |
| Tests de `commands` que fijan `activity_percent` (`la_actividad_de_rendimiento_se_refleja...`, `persist_perf_reading_*`) | Adaptar al nuevo camino (agregado en memoria en vez de muestra persistida). |
