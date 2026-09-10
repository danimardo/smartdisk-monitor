# Contrato — Benchmark de disco (DiskSpd)

Reemplaza la parte de `docs/ui-contract.md` §3.6 relativa al benchmark. chkdsk y autotest SMART
**no cambian**.

## Comandos

```ts
// Corre la matriz fija de 8 mediciones sobre un archivo de 1 GiB en la carpeta controlada del
// volumen. Devuelve el testRunId. Sin parámetros de perfil (decisión de producto).
invoke<string>("start_benchmark", { volumeId: string })

invoke<void>("cancel_test", { testRunId: string })          // SIN CAMBIOS — mata el proceso DiskSpd
invoke<TestRun[]>("get_test_runs", { deviceId?: string, limit?: number })   // SIN CAMBIOS de firma
```

### Errores de `start_benchmark`

| Código | Cuándo | Reintentable |
|---|---|---|
| `volume.not_found` | el `volumeId` no existe | no |
| `path.invalid` | el volumen no tiene letra de unidad | no |
| `test.busy` | ya hay una prueba (benchmark o autotest SMART) en ese disco físico | no |
| `test.insufficient_space` | 1 GiB no cabe tras la reserva de seguridad (2 GiB o 5 %, la mayor) | no |
| `test.tool_missing` | `diskspd.exe` no está o no se puede ejecutar | no |
| `test.io_failed` | fallo al crear la carpeta / el archivo | sí |

Errores que aparecen **en el resultado** (la ejecución arranca y luego falla), no en la llamada:

| `stoppedReason` / código en `output` | Cuándo |
|---|---|
| `test.tool_output_unreadable` | DiskSpd terminó pero su `-Rxml` no se puede interpretar |
| `stoppedReason: "error"` | DiskSpd devolvió un código de salida distinto de 0 en una medición |
| `stoppedReason: "thermal"` | el disco alcanzó su límite crítico durante la matriz |
| `stoppedReason: "cancelled"` | el usuario canceló |

## Tipos

```ts
type TestStatus = "pending" | "running" | "completed" | "cancelled" | "failed";

interface TestRun {
  id: string;
  type: "benchmark" | "chkdsk_scan" | "smart_short";
  deviceId: string | null;
  volumeId: string | null;
  status: TestStatus;
  startedAt: string;                      // UTC ISO-8601
  finishedAt: string | null;
  progressPercent: number | null;         // 0–100; en benchmark = (mediciones terminadas / 8) * 100
  command: string | null;                 // descripción legible de la matriz, no un comando de shell
  parameters: Record<string, unknown>;    // { tool, toolVersion, fileSizeBytes, profiles }
  result: TestResult | null;
  output: string | null;                  // texto de diagnóstico si falló; nunca el volcado completo de DiskSpd
  outputEncoding: string | null;
  orphanPath: string | null;              // ruta del archivo temporal si no se pudo borrar
}

interface TestResult {
  passed: boolean | null;                 // chkdsk / autotest; siempre null en benchmark
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "error" | null;
  benchmark: BenchmarkResult | null;      // no null sólo si type === "benchmark" y hubo al menos una fila
}

interface BenchmarkResult {
  tool: "diskspd";
  toolVersion: string;
  fileSizeBytes: number;
  rows: BenchmarkRow[];
  notRun: { profile: BenchmarkProfile; direction: "read" | "write" }[];
}

type BenchmarkProfile = "seq1m_q8" | "seq1m_q1" | "rnd4k_q32" | "rnd4k_q1";

interface BenchmarkRow {
  profile: BenchmarkProfile;
  direction: "read" | "write";
  mbPerSecond: number;                    // MB decimales / s (como CrystalDiskMark)
  iops: number;
  avgLatencyMs: number;
  actualDurationS: number;
  bytesMoved: number;
  dataCapHit: boolean;                    // true si la duración se recortó por el tope de datos
}
```

## Reglas

- **La interfaz nunca construye la línea de comandos.** El backend arma cada invocación de DiskSpd,
  la lanza por `platform::proceso_externo` (`CREATE_NO_WINDOW`), parsea el `-Rxml` y compone
  `BenchmarkResult`.
- **Orden de la matriz**: para cada perfil, primero **lectura**, luego **escritura** (la lectura da
  el caudal de referencia para dimensionar la `-d` de la escritura, D3). Los 4 perfiles en el orden
  de `PERFILES`.
- **Progreso**: `progressPercent` avanza `100/8` por cada medición terminada; el evento
  `test:progress` lleva además qué perfil/sentido está en curso (clave i18n, no texto).
- **Parada anticipada**: las mediciones ya hechas quedan en `rows`; las que faltaban, en `notRun`.
  `status` = `cancelled` o `failed`; `stoppedReason` lo precisa.
- **`command`** y **`parameters.toolVersion`** hacen la ejecución comparable con otra y con
  CrystalDiskMark (FR-012).
- **Cero red** en todo el flujo (FR-014).
