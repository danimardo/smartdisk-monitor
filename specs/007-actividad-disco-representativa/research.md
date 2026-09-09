# Fase 0 — Research: Actividad de disco representativa

Todas las incógnitas del plan quedan resueltas aquí. Formato por decisión: **Decisión / Razón /
Alternativas consideradas**.

---

## R1 · Ciclo de vida de la consulta PDH y semántica de `% Idle Time`

**Decisión**: una única `PDH_HQUERY` abierta durante toda la ejecución, con **un contador
`\PhysicalDisk(<n> *)\% Idle Time` por disco físico monitorizado**. En cada muestreo se llama a
`PdhCollectQueryData` **una vez** y luego a `PdhGetFormattedCounterValue` por contador. El valor
formateado de `% Idle Time` en ese instante es el porcentaje de tiempo inactivo **en el intervalo
transcurrido desde el `PdhCollectQueryData` anterior**; `activity_percent = clamp(100 − idle, 0, 100)`
(se reutiliza `derivar_activity_percent`, ya probada).

La **primera** recogida tras abrir la consulta (o tras añadir un contador) devuelve
`PDH_CSTATUS_INVALID_DATA` / `PDH_CSTATUS_NEW_DATA` en `c_status`: no hay intervalo previo con el que
comparar. Esa muestra **se descarta** (no entra en la ventana), igual que hoy `perf_counters::leer`
hace dos recogidas y usa solo la segunda.

**Razón**: `% Idle Time` es un contador de tipo temporal (`PERF_100NSEC_TIMER_INV`): PDH calcula el
valor a partir del delta de dos muestras crudas. Con una consulta **persistente**, la «segunda
muestra» es simplemente la recogida anterior del bucle: no hace falta el `thread::sleep(1 s)` que
hoy paga `leer()` por ser autónoma. Es exactamente cómo `perfmon` y el Administrador de tareas
mantienen su lectura continua.

**Alternativas consideradas**:
- *Mantener el patrón abrir/muestrear-dos-veces/cerrar por tick*: obliga a dormir 1 s dentro del
  hilo del planificador en cada tick, que rompe la cadencia de 1 s del bucle. Descartada.
- *`Get-Counter` de PowerShell en subproceso*: mismos contadores, más coste (arranque de proceso),
  y arrastra la trampa de la ventana de consola (`.claude/rules/backend-rust.md` J.57). Descartada.

## R2 · Reconstrucción de la consulta al cambiar el inventario

**Decisión**: cuando cambia el conjunto de discos físicos monitorizados, **se reconstruye la
consulta entera** (cerrar `HQUERY`, abrir una nueva con los contadores del conjunto actual). El
conjunto objetivo se refresca una vez por ciclo de «métricas rápidas» (~30 s) y en el
post-proceso de una alta/baja de inventario, no en cada tick.

**Consecuencia**: tras una reconstrucción, **todos** los discos quedan en estado `parcial` hasta que
su ventana vuelve a cubrir la cadencia (≈30 s). Un cambio de inventario (enchufar un USB) es raro y
el estado `parcial` es honesto (constitución §I), así que es aceptable.

**Razón**: gestionar contadores vivos uno a uno (`PdhAddEnglishCounterW` / `PdhRemoveCounter`
sobre una consulta abierta, con un `HashMap<i64, HCOUNTER>` y su bookkeeping) es más código y más
estados por probar para ahorrar ~30 s de `parcial` en un evento infrecuente. Constitución §II.5:
simplicidad antes que generalidad.

**Alternativa considerada**: `PdhRemoveCounter` + añadir incremental. Mantiene el histórico de PDH
de los discos no afectados. Se puede adoptar más adelante si medimos que las reconstrucciones
molestan; hoy no hay evidencia de que lo hagan. Si se adopta, es **un** símbolo FFI nuevo
(`PdhRemoveCounter`) en el bloque `extern "system"` que ya existe, sin dependencia.

## R3 · Intervalo de muestreo, tamaño de ventana y umbral de hueco  → `docs/open-questions.md` D.4

**Decisión** (valores adoptados, a registrar en `open-questions.md` D.4 con estado `PROPUESTO`):

| Parámetro | Valor | Nota |
|---|---|---|
| Intervalo de muestreo | **1 s** (un muestreo por tick del bucle del planificador) | En batería, 1 de cada 4 ticks → **4 s** (coherente con D.2, `MULTIPLICADOR_BATERIA`). |
| Tamaño de la ventana | **= `schedule.metrics_fast_seconds`** (30 s de fábrica; 10–300 s) | La cifra agrega «lo que va del último intervalo mostrado». |
| Umbral de hueco | **> 3 × intervalo de muestreo** (≈3 s en red, ≈12 s en batería) | Por encima, se descartan las muestras anteriores al hueco antes de agregar (FR-007). |
| Estado `valido` | la muestra más antigua de la ventana tiene una antigüedad ≥ tamaño de la ventana | Antes de eso, `parcial` (con datos) o `no_disponible` (vacía). |

**Razón**: 1 s es lo que hace el Administrador de tareas, no cuesta nada medible (SC-005) y evita un
temporizador secundario dentro del bucle. Ligar la ventana a la cadencia configurable mantiene la
coherencia entre lo que se ve en vivo y lo que se persiste. El umbral de hueco a 3× reproduce a
escala de muestreo la idea de `open-questions.md` E.1 («un salto es un hueco a partir de 2,5× la
cadencia»).

**Alternativas consideradas**: muestreo a 2 s (lo que decía el borrador) — 1 s es más simple (sin
subcontador) y más fiel. Ventana fija de 30 s independiente de la cadencia — incoherente si el
usuario sube la cadencia a 5 min.

## R4 · Reloj monotónico y suspensión del equipo

**Decisión**: la ventana usa `std::time::Instant` para fechar cada muestra y para todos los
cálculos de antigüedad. La detección de «hueco» de R3 cubre la suspensión: al reanudar, el primer
`saturating_duration_since` de la última muestra supera el umbral y la ventana se vacía.

**Razón**: `Instant` en Windows se apoya en `QueryPerformanceCounter`; es monotónico y no lo afecta
un cambio de hora del sistema (FR-006). En Windows 10+ `Instant` **sí** avanza durante la
suspensión, de modo que el hueco se detecta de forma natural sin depender de un evento de
reanudación.

**Alternativa considerada**: `SystemTime` / hora de pared — la descarta FR-006 (un cambio de hora
falsearía la ventana).

## R5 · Forma del tipo estructurado que sustituye a `activityPercent`

**Decisión**: en `DiskSummary` (y por herencia `DeviceDetail`), `activity_percent: Option<f64>` se
sustituye por:

```rust
pub struct ActividadDisco {
    pub estado: EstadoActividad,          // "valido" | "parcial" | "no_disponible"
    pub media_percent: Option<f64>,       // Some cuando estado != no_disponible
    pub pico_percent: Option<f64>,        // Some cuando estado != no_disponible
    pub muestras: u32,                    // nº de muestras que respaldan la ventana
    pub ventana_segundos: u32,            // periodo que la ventana pretende cubrir
}
pub enum EstadoActividad { Valido, Parcial, NoDisponible }
```

`serde(rename_all = "camelCase")`; `ts-rs` exporta `ActividadDisco.ts` y `EstadoActividad.ts`. El
campo en el DTO se llama `activity`.

**Razón**: el número suelto actual es justo la instantánea que se elimina; conservarlo obligaría a
decidir qué poner ahí y arrastraría la ambigüedad que el principio I prohíbe. Con el estado
explícito, `parcial` y `no_disponible` dejan de confundirse con `null`. `muestras` y
`ventana_segundos` permiten a la interfaz explicar «media de los últimos N s» sin inventar.

**Alternativas consideradas** (ambas descartadas en Q2 → A de la spec):
- Conservar `activityPercent` como pico y añadir campos: el significado del campo cambia en
  silencio; un consumidor no adaptado sigue compilando.
- Objeto nuevo dejando `activityPercent` como instantáneo: doble fuente temporal del mismo dato.

## R6 · Regla de persistencia en la serie histórica

**Decisión**: en el ciclo de «métricas rápidas», por cada disco:
- si el agregado de su ventana es `valido` → se persiste **una** muestra `activity_percent` con
  `media_percent` (valor 0–100, en bruto, como hoy);
- si es `parcial` o `no_disponible` → **no se escribe fila** (FR-010a).

`perf_counters::leer` **deja de leer y de devolver** `activity_percent`: `LecturaRendimiento`
pierde ese campo y `persist_perf_reading` deja de escribir la serie desde ahí. `leer()` sigue
haciendo su `sleep(1 s)` para las tasas (caudal) y latencias, que no cambian (Q2 → A).

**Razón**: una sola serie, sin tocar el esquema ni la retención; el histórico pasa a ser coherente
con lo que se mostró en vivo. Escribir una media de pocos segundos como valor del ciclo violaría §I.

**Alternativa considerada**: persistir media y pico como dos series — cambia el modelo de datos,
la retención y exige migración; se deja como ampliación aditiva futura (Q3 → A).

## R7 · Degradación de la fuente ante un fallo de muestreo

**Decisión**: si en un tick fallan **todos** los contadores (error de PDH, subsistema de
rendimiento caído), la muestra no entra en ninguna ventana (las ventanas conservan lo que tenían) y
se cuenta el fallo. La transición de `MetricSource::PerformanceCounter` a `timeout`/`error` se
publica por el mecanismo existente (`actualizar_source_health` → evento `source:degraded`) **en el
ciclo de «métricas rápidas»**, no en cada tick. Mientras el fallo persiste, las ventanas envejecen;
cuando su muestra más reciente sale del umbral de hueco, el agregado pasa a `no_disponible`.

**Razón**: cumple §X (degradar la tarjeta, no la aplicación) y §XV (no un `warn` por segundo:
el flanco se registra una vez, como ya hace `source_health`). El `AppError` reutiliza el código
`perf_counters.read_failed` / clave `error.perfCountersFailed` que ya existe.

**Alternativa considerada**: emitir `source:degraded` en cuanto falla un tick — genera ruido de
eventos y de log si el subsistema parpadea.

## R8 · Sin dependencias ni permisos nuevos — confirmación

**Decisión**: la feature se implementa con:
- La FFI a `pdh.dll` **ya presente** en `collectors/perf_counters.rs`: `PdhOpenQueryW`,
  `PdhAddEnglishCounterW`, `PdhCollectQueryData`, `PdhGetFormattedCounterValue`, `PdhCloseQuery`.
  Solo se añaden **constantes** (`PDH_CSTATUS_VALID_DATA` = 0, `PDH_CSTATUS_NEW_DATA` = 0,
  `PDH_CSTATUS_INVALID_DATA`, `PDH_CSTATUS_NO_DATA`) para poder descartar la primera muestra; ningún
  símbolo de función nuevo si se adopta la reconstrucción de R2.
- `std` (`VecDeque`, `Instant`, `Mutex`, `HashMap`) para la ventana y el estado compartido.
- `ts-rs` para el DTO (ya en la pila).
- Zod (ya en la pila) para el esquema y su prueba de rechazo.

**No** hay: crate nuevo en `Cargo.toml`, permiso/capability de Tauri nuevo (la llamada PDH es
backend puro, no toca las *capabilities* del WebView), ni endpoint de red.

**Razón**: `AGENTS.md` límites duros y constitución §III. El acceso a contadores de rendimiento ya
está cubierto por la ejecución elevada existente.

## R9 · Procedencia y antigüedad del agregado de actividad

**Decisión**: `ActividadDisco` **no** lleva ni campo de procedencia ni marca de tiempo.

- **Procedencia**: la actividad siempre viene de `MetricSource::PerformanceCounter`. Es invariante
  para esta métrica, así que la interfaz la **rotula fija** («contadores de rendimiento») sin
  consultar un dato. No es un componente «decidiendo una regla» (constitución §IV): es una etiqueta
  constante conocida.
- **Antigüedad**: el agregado se recalcula en cada emisión (`emitir_metrics_updated`, ~30 s) a
  partir de la ventana viva en memoria. Por construcción es **actual**: o hay muestras recientes
  (`Valido`/`Parcial`) o la ventana se ha vaciado por un hueco / la fuente está degradada
  (`NoDisponible`). **No existe** un estado «válido pero de hace 10 minutos» como en las lecturas
  SMART persistidas, así que no hay una antigüedad que transportar. La marca de obsolescencia
  general del panel (`Toolbar` / frescura) cubre el caso de que dejen de llegar ciclos enteros.

**Razón**: añadir una marca de tiempo que siempre valdría «ahora» es ruido en el contrato
(constitución §II.4: claridad antes que densidad). El `estado` ya responde la única pregunta
relevante: ¿me puedo fiar de este número?

**Alternativa considerada**: `ActividadDisco { …, medido_en: String }` — descartada: el valor sería
siempre el instante de la emisión, sin información añadida.

---

## Incógnitas restantes → decididas en implementación, no bloquean

- Nombre exacto de los campos del DTO en `camelCase` y del módulo (`domain/actividad.rs` vs
  `domain/actividad_disco.rs`): estilo, se resuelve al escribir.
- Si la sparkline de US3 (P3) entra en este lote o en uno posterior: decisión de `/speckit-tasks`
  según el tamaño del lote.
- Umbral exacto de «parcial» (¿tolerancia del 10 % sobre el tamaño de la ventana?): se fija en
  `domain/actividad.rs` con su prueba; candidato a matiz en `open-questions.md` D.4.
