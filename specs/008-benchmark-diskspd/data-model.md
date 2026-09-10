# Fase 1 — Modelo de datos: Benchmark de disco con DiskSpd

**Sin cambio de esquema SQLite.** `test_runs` (migración 0001) ya tiene todo lo necesario. Lo que
cambia es la **forma del resumen del resultado** dentro de `result_summary_json` (columna libre por
diseño, `open-questions.md` J.29) y los **parámetros** de `start_benchmark`.

---

## 1. Entidades de dominio (puras, `src-tauri/src/tests/diskspd.rs`)

### 1.1 `Perfil` — configuración fija de una prueba

| Campo | Tipo | Notas |
|---|---|---|
| `clave` | `&'static str` | `"seq1m_q8"`, `"seq1m_q1"`, `"rnd4k_q32"`, `"rnd4k_q1"`. Estable, para i18n y JSON. |
| `acceso` | `Acceso` | `Secuencial` \| `Aleatorio` |
| `bloque_bytes` | `u64` | 1 048 576 o 4096 |
| `cola` | `u32` | `-o`: 8, 1, 32, 1 |

Constante: `PERFILES: [Perfil; 4]`. **No** editable por el usuario (spec, decisión del dueño).

### 1.2 `Medicion` — un perfil en un sentido, ya con su línea de comandos y su `-d`

| Campo | Tipo | Notas |
|---|---|---|
| `perfil` | `&'static Perfil` | |
| `sentido` | `Sentido` | `Lectura` (`-w0`) \| `Escritura` (`-w100`) |
| `duracion_objetivo_s` | `u32` | `D_OBJETIVO` (5) o el recorte por tope de datos (D3). Suelo `D_MIN` (2). |
| `tope_datos_bytes` | `Option<u64>` | Solo escritura: `TOPE_DATOS` (~4 GiB). `None` en lectura. |

- `args(&self, ruta_archivo, tamano_archivo) -> Vec<String>`: **pura**. Construye la línea de DiskSpd
  (`-b`, `-o`, `-r`/`-s`, `-w`, `-d`, `-W`, `-Sh`, `-Z1M`, `-L`, `-Rxml`, `-c`, ruta). Probada con
  fixtures sin lanzar nada.
- `duracion_efectiva(caudal_referencia_bps: Option<f64>) -> (u32, bool)`: **pura**. Devuelve
  `(segundos, tope_alcanzado)` según D3 (`clamp(tope/caudal, D_MIN, D_OBJETIVO)`; si `tope/caudal <
  D_MIN` → `(D_MIN, true)`).

### 1.3 `FilaResultado` — la salida de una `Medicion`

| Campo | Tipo | Notas |
|---|---|---|
| `perfil` | `String` | `Perfil.clave` |
| `sentido` | `String` | `"lectura"` \| `"escritura"` |
| `mb_por_segundo` | `f64` | `bytes / TestTimeSeconds / 1_000_000` (MB decimal, como CDM) |
| `iops` | `f64` | `ops / TestTimeSeconds` |
| `latencia_media_ms` | `f64` | del XML de DiskSpd |
| `duracion_real_s` | `f64` | `TestTimeSeconds` del XML |
| `bytes_movidos` | `u64` | del XML |
| `tope_alcanzado` | `bool` | true si la duración se recortó por `TOPE_DATOS` (D3) |

Una medición que **no llegó a correr** (parada térmica / cancelación antes de su turno) → **no** hay
`FilaResultado`, o se marca aparte como no ejecutada. Nunca a cero (§I, FR-017/spec).

---

## 2. Contrato IPC — `TestResult` del benchmark

### Antes (`ui-contract.md` §3.6)

```ts
interface TestResult {
  passed: boolean | null;
  readBytesPerSecond: number | null;
  writeBytesPerSecond: number | null;
  readLatencyMs: number | null;
  writeLatencyMs: number | null;
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "space" | "error" | null;
}
```

### Después

El benchmark deja de usar los campos planos y estrena `benchmark`. chkdsk y autotest siguen usando
`passed`/`output`. (La forma exacta la fija `contracts/pruebas-benchmark.md`; resumen:)

```ts
interface TestResult {
  passed: boolean | null;                 // chkdsk / autotest; null en benchmark
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "error" | null;  // "space" ya no aplica en ejecución (rechazo previo)
  /** Solo en type === "benchmark". */
  benchmark: {
    tool: "diskspd";
    toolVersion: string;                  // del XML de DiskSpd (FR-012)
    fileSizeBytes: number;
    rows: {
      profile: "seq1m_q8" | "seq1m_q1" | "rnd4k_q32" | "rnd4k_q1";
      direction: "read" | "write";
      mbPerSecond: number;
      iops: number;
      avgLatencyMs: number;
      actualDurationS: number;
      bytesMoved: number;
      dataCapHit: boolean;
    }[];
    /** Perfiles+sentidos que no llegaron a ejecutarse por parada anticipada. */
    notRun: { profile: string; direction: "read" | "write" }[];
  } | null;
}
```

- Se generan con `ts-rs` desde los structs Rust; el Zod se infiere con `z.infer` y lleva **prueba
  de rechazo** (fila con `mbPerSecond` ausente, `profile` fuera del enum → `ipc.schema_mismatch`).
- `docs/data-model.md` §3 (test_runs) documenta que `result_summary_json` de un benchmark lleva esta
  forma; sigue sin ser una tabla nueva.

---

## 3. Parámetros de `start_benchmark`

### Antes

```ts
invoke<string>("start_benchmark", { volumeId, sizeBytes, blockSizeBytes, mode, passes })
```

### Después

```ts
invoke<string>("start_benchmark", { volumeId })   // corre la matriz fija
```

`command` (el string literal para el `ConfirmDialog` y el historial) pasa a describir la matriz
(«DiskSpd — 4 perfiles, lectura y escritura, archivo de 1 GiB»), no un comando de shell único.
`parameters` (JSON) guarda `{ tool, toolVersion, fileSizeBytes, profiles: [...] }`.

---

## 4. XML de DiskSpd — struct de parseo (`tests/diskspd_xml.rs`)

Se parsea **a mano** (patrón de `collectors/event_log.rs`), en un struct explícito. Del bloque
agregado `<Results><TimeSpan>` se extraen, por objetivo/agregado:

| Elemento XML (aprox.) | Campo Rust |
|---|---|
| `TestTimeSeconds` | `test_time_s: f64` |
| `ReadBytes` / `WriteBytes` (o `BytesCount` + sentido) | `bytes: u64` |
| `ReadCount` / `WriteCount` (o `IOCount`) | `ops: u64` |
| `AverageReadLatencyMilliseconds` / `AverageWriteLatencyMilliseconds` | `avg_latency_ms: f64` |
| versión (atributo del elemento raíz o `<System>`) | `tool_version: String` |

Un XML que no trae estos campos, o que no se puede recorrer → `Err(SalidaIlegible)` → la ejecución
falla con `test.tool_output_unreadable`, **nunca** una fila con ceros.

**Fixtures**: capturas de `-Rxml` reales del `diskspd.exe` redistribuido, una por perfil/sentido y
una de una ejecución que DiskSpd abortó, guardadas en `src-tauri/src/tests/fixtures/` (como las de
`smartctl_parser`).
