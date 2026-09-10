# Research: Benchmark de disco con DiskSpd

**Feature**: 008-benchmark-diskspd · **Fecha**: 2026-09-10

Resuelve las incógnitas del plan antes de la Fase 1. Todo valor numérico nuevo se propone aquí y se
traslada a `docs/open-questions.md` al implementar (constitución, flujo §1).

---

## D1 · Qué herramienta y cómo se redistribuye

**Decisión**: **Microsoft DiskSpd**, redistribuido como binario independiente, mismo patrón exacto
que `smartctl.exe` (ADR-005): proceso externo, nunca enlazado, comunicación por línea de comandos y
salida estándar.

- **Licencia**: MIT. Mucho más simple que la GPLv2 de smartmontools: **no hay obligación de código
  fuente**; basta incluir el texto de la licencia y el aviso de copyright en la instalación.
- **Versión**: la última estable en el momento de implementar (la serie es `2.0.20a`, `2.0.21a`,
  `2.1`, `2.2`, `2.3`). La fija el ADR con su hash, igual que «smartmontools 7.5». **Verificada como
  PE de máquina `0x8664` (AMD64)** antes de aceptarla, como se hizo con `smartctl.exe`.
- **Origen**: descarga oficial `https://aka.ms/getdiskspd` (redirige a la Release de
  `github.com/microsoft/diskspd`). El ZIP trae subcarpetas por arquitectura (`amd64/`, `arm64/`) y
  el fichero de licencia. **Solo se redistribuye `amd64/diskspd.exe`** (~200 KB) y la licencia.
- **Estructura en el repo** (espejo de `third-party/smartmontools/`):
  ```
  third-party/diskspd/
  ├── bin/diskspd.exe            # amd64, ~200 KB
  ├── licenses/LICENSE.txt       # MIT, íntegra
  └── README.md                  # versión, origen, qué se incluye y qué se deja fuera
  ```
- **Empaquetado** (`tauri.conf.json` → `bundle.resources`): `"../third-party/diskspd/bin/diskspd.exe":
  "bin/diskspd.exe"` y `"../third-party/diskspd/licenses/LICENSE.txt": "licenses/diskspd/LICENSE.txt"`.
- **Integridad**: entrada nueva en `scripts/verify-assets.mjs` con su SHA-256, y sección en
  `THIRD_PARTY_NOTICES.md` (con la nota de que la obligación 3(a) de la GPL **no aplica** aquí: es
  MIT).
- **Resolución de ruta en Rust**: función `resolve_diskspd_path()` calcada de
  `collectors::smartctl::resolve_smartctl_path()` (dev: `../third-party/diskspd/bin/diskspd.exe`;
  prod: `bin/diskspd.exe` junto al ejecutable). Mismo aviso: no verificado contra un instalador
  empaquetado real hasta abrir uno.

**Alternativas descartadas**:
- *Motor propio con E/S superpuesta (overlapped/IOCP vía FFI a `kernel32`)*: sin dependencia nueva,
  pero es mucho código `unsafe` y clavar las cifras para que coincidan con CrystalDiskMark es
  difícil; un benchmark que reporta mal choca con el principio I. Se valoró y descartó con el dueño.
- *`Get-Counter`/PowerShell*: no hace benchmark, solo lee contadores.
- *Empaquetar CrystalDiskMark*: GUI, no scriptable de forma limpia, licencia y peso mayores.

**El hook `proteger-rutas.mjs` bloquea escribir en `third-party/`**: el binario y su licencia los
**coloca el dueño** tras descargarlos y verificarlos, y aporta el SHA-256 para `verify-assets.mjs` y
`THIRD_PARTY_NOTICES.md`. El agente no descarga ni coloca binarios de terceros.

---

## D2 · Los perfiles fijos

**Decisión**: cuatro perfiles con nombre, los «de pico» de CrystalDiskMark, cada uno en lectura
(`-w0`) y escritura (`-w100`). Un hilo por objetivo en todos (`-t1`); un solo objetivo (el archivo).

| Nombre visible | Acceso | Bloque | Cola (`-o`) | Notación CDM |
|---|---|---|---|---|
| Secuencial (cola 8) | secuencial (`-s`) | 1 MiB (`-b1M`) | 8 | SEQ1M Q8T1 |
| Secuencial (cola 1) | secuencial (`-s`) | 1 MiB | 1 | SEQ1M Q1T1 |
| Aleatorio 4K (cola 32) | aleatorio (`-r4K`) | 4 KiB (`-b4K`) | 32 | RND4K Q32T1 |
| Aleatorio 4K (cola 1) | aleatorio (`-r4K`) | 4 KiB | 1 | RND4K Q1T1 |

- **Matriz**: 4 perfiles × 2 sentidos = **8 mediciones** por ejecución.
- **Caché**: `-Sh` (= `-Suw`: sin caché de software, con write-through) — equivale a
  `FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH`, exactamente lo que hacía el motor propio.
- **Buffer de escritura**: `-Z1M` (buffer de datos aleatorios) para que un SSD con compresión no dé
  cifras infladas.
- **Latencia**: `-L` en todos (media por sentido).
- **Archivo**: `-c<tamaño>` lo crea; DiskSpd itera sobre él, **no lo hace crecer**.

**Alternativas descartadas**:
- *Añadir SEQ128K / los perfiles «Real World» de CDM*: más filas, más tiempo, más escritura; los
  cuatro de pico ya dan el retrato que la gente reconoce. Modo avanzado configurable → fuera de
  alcance (spec).
- *Multihilo (`-t>1` o `-F`)*: CDM usa T1 en sus perfiles de pico modernos; añadir hilos complica la
  comparación y el reparto de CPU. Descartado.

---

## D3 · Acotado por tiempo con tope de datos (FR-017, opción C del dueño)

**Problema**: DiskSpd **no tiene** ningún parámetro para parar tras N bytes o N operaciones — solo
tiempo (`-d`, `-W`, `-C`). El tope de datos hay que imponerlo desde el envoltorio.

**Decisión**: el envoltorio calcula la duración efectiva de cada perfil **antes de lanzarlo**, a
partir del caudal ya medido, y elige `-d` de forma que se respete el tope de datos sin bajar de un
suelo de medición.

Valores propuestos (a `open-questions.md`):

| Parámetro | Valor propuesto | Razón |
|---|---|---|
| `D_OBJETIVO` | **5 s** | Ventana de medición cómoda, como CDM/DiskSpd por defecto (10 s es más de lo necesario). |
| `D_MIN` | **2 s** | Por debajo de esto la cifra es ruido. Suelo de medición. |
| `D_CALENTAMIENTO` (`-W`) | **2 s** | Descarta el arranque (caché SLC, colas frías). No cuenta como datos «medidos» pero sí escribe. |
| `TOPE_DATOS` por medición de **escritura** | **4 GiB** | Techo de desgaste por perfil de escritura. Lectura no desgasta → sin tope. |
| Tamaño del archivo (`-c`) | **1 GiB** | Como hoy. DiskSpd itera sobre él; no crece. Acota el espacio ocupado. |

**Cómo se calcula `-d` de un perfil de escritura**:
1. El mismo perfil en **lectura** corre **antes** en la matriz → da un caudal `C` (aproxima el de
   escritura lo bastante para dimensionar).
2. `d = clamp(TOPE_DATOS / C, D_MIN, D_OBJETIVO)`.
3. Si `TOPE_DATOS / C < D_MIN` (disco tan rápido que el tope se agota antes del suelo): se corre
   `D_MIN` igualmente y el resultado se **etiqueta** «disco muy rápido: se escribieron ~N GiB para
   poder medir con fiabilidad». Es la única forma honesta: una medición de 0,3 s no es un dato.
4. En cualquier caso el resultado reporta la `-d` real y los **bytes realmente escritos** (DiskSpd
   los da en su salida).

**Peso total de escritura de la matriz** (2 perfiles de escritura + sus calentamientos):
- HDD 150 MB/s: ~1,5 GiB. SATA SSD 500 MB/s: ~5 GiB. NVMe Gen4 5 GB/s: ~2 × 4 GiB tope +
  calentamiento ≈ 12–20 GiB. Gen5 12 GB/s: se supera el tope, ~30–50 GiB, etiquetado.
- **Contexto de desgaste**: incluso 50 GiB sobre un SSD de 600–1200 TBW es < 0,01 % de su
  resistencia. El tope es más por percepción («un benchmark no debería machacarme el disco») que por
  daño real. El aviso previo (FR-010) muestra el **máximo real** según estos valores y el caudal
  estimado.

**Alternativas descartadas**:
- *Calibración con una pasada corta previa dedicada*: otra pasada que escribe y tarda; se aprovecha
  mejor el resultado de lectura del mismo perfil, que ya corre.
- *Vigilar y matar DiskSpd por bytes*: DiskSpd no reporta progreso parseable a media ejecución
  (`-D` es un histograma de IOPS, no un contador de bytes). Inviable de forma limpia.
- *Fichero minúsculo para «dar vueltas» y limitar escritura*: un fichero pequeño mide la caché, no
  el disco (es la razón del `FILE_FLAG_NO_BUFFERING`, pero el tamaño del fichero también importa
  para el comportamiento del FTL). 1 GiB es el mínimo razonable.

---

## D4 · Formato de salida y parseo

**Decisión**: `-Rxml` (salida XML a stdout). Estructurada, estable entre versiones, tipable.

Campos que se extraen por medición (del bloque agregado de `TimeSpan`):
- Caudal: `BytesCount` / `TestTimeSeconds` → MB/s (o los campos `ReadBytes`/`WriteBytes` según el
  sentido).
- IOPS: `IOCount` / `TestTimeSeconds` (o `ReadCount`/`WriteCount`).
- Latencia media: `AverageReadLatencyMilliseconds` / `AverageWriteLatencyMilliseconds`.
- Duración real: `TestTimeSeconds`.
- `diskspd` version: atributo del elemento raíz o del bloque `System`.

**Parseo en Rust**: **a mano, sin dependencia nueva** (constitución §III). `src-tauri/Cargo.toml`
no tiene ningún parser XML, y `collectors/event_log.rs` ya deserializa el XML del Event Log a mano
con dos helpers (`extraer_texto`, `extraer_atributo`, basados en `str::find`), «porque el esquema es
fijo y lo produce el sistema». El bloque agregado de DiskSpd es igual de plano y estable: un módulo
`tests::diskspd_xml` con un puñado de `extraer_*` y un `struct` explícito de destino (no
`serde_json::Value` — `backend-rust.md`). Se prueba con **fixtures de salida XML real** capturadas
del `diskspd.exe` que se redistribuye, igual que `smartctl_parser.rs`.

**Alternativas descartadas**:
- *`-Rtext` (por defecto)*: legible para humanos, frágil de parsear (tablas con anchos variables).
- *`-Rjson`*: no existe en DiskSpd (confirmado en la wiki de parámetros).

---

## D5 · Salvaguardas: qué se reutiliza y qué cambia

| Salvaguarda | Hoy (motor propio) | Con DiskSpd |
|---|---|---|
| **Rutas / carpeta controlada** | `tests::rutas` (puro) | **Se reutiliza tal cual.** El archivo vive en `<raíz>\SmartDisk Monitor Benchmark\`, nombre con sufijo, `confirmar_no_sobrescribe` antes de crear. |
| **Reserva de espacio** | `tests::rutas::resolver_tamano_bytes` | **Se reutiliza.** Se comprueba contra el tamaño de `-c` (1 GiB) antes de lanzar. |
| **Guardia térmica** | entre bloques, `tests::guardia` | **Se reutiliza `tests::guardia`** (puro). El envoltorio **sondea la temperatura cada ~2 s** (como hoy) y, si supera el límite, **mata el proceso DiskSpd** y marca la ejecución «detenida por temperatura», conservando los perfiles ya medidos. Granularidad más gruesa (un perfil en curso se pierde), aceptable. |
| **Cancelación** | `AtomicBool` entre bloques | El envoltorio **mata el proceso DiskSpd** al ver la bandera. La E/S cesa al morir el proceso (≤ SC-003). |
| **Exclusión con autotest SMART** | `hay_prueba_activa` por disco físico (J.29) | **Se reutiliza tal cual.** |
| **Borrado del archivo** | `remove_file` tras `ejecutar` | **Se reutiliza.** El envoltorio borra tras terminar DiskSpd (todos los perfiles) o tras matarlo. `temp_path` conserva la ruta si el borrado falla. |
| **Verificación byte a byte** | `tests::patron` + comparación | **SE ELIMINA** (FR-016). No hay sustituto. |
| **Guardia de espacio durante la ejecución** | no existe (J.29) | Sigue sin existir: el archivo es de tamaño fijo (`-c`), no crece. Sin cambio. |
| **`CREATE_NO_WINDOW`** | — | El proceso DiskSpd se lanza con `platform::proceso_externo` (que ya aplica `CREATE_NO_WINDOW` y drena los pipes en hilos aparte, J.55/J.57). **Pero** DiskSpd corre 8 mediciones × ~7–10 s = **hasta ~90 s**, más que el límite de 15 s de `smartctl`: se lanza **una invocación por medición** con su propio límite generoso (p. ej. `D_OBJETIVO + D_CALENTAMIENTO + 30 s` de margen), no una sola invocación larga. Así el progreso es real (una medición terminada = un octavo) y un cuelgue de DiskSpd en un perfil no arrastra a los demás. |

**Controlled Folder Access de Defender (ADR-043)**: la carpeta del benchmark está en la **raíz del
volumen**, no es una carpeta protegida por CFA (Documentos, Escritorio, etc.). **No hace falta una
excepción** para `diskspd.exe` como la que necesitó `smartctl.exe`. Se confirma en el plan / prueba
manual; si resultara necesaria, el mecanismo de `proteccion_carpetas.rs` ya existe.

---

## D6 · Contrato y modelo de datos

**Decisión**: se **sustituye** el comando `start_benchmark` (con sus parámetros `size_bytes`,
`block_size_bytes`, `mode`, `passes`) por uno **sin parámetros de perfil** — solo `volume_id` — que
corre la matriz fija.

- `docs/ui-contract.md` §3.6 (pruebas): `start_benchmark({ volumeId })` — se van `sizeBytes`,
  `blockSizeBytes`, `mode`, `passes`.
- El **resumen del resultado** (`result_summary_json` de `test_runs`, hoy `ResumenPruebaJson` con
  `read_bytes_per_second`/`write_bytes_per_second`/`read_latency_ms`/`write_latency_ms`/
  `max_temperature_c`/`stopped_reason`) pasa a llevar una **lista de filas de perfil**:
  `{ perfil, sentido, mbPorSegundo, iops, latenciaMediaMs, duracionRealS, bytesMovidos, topeAlcanzado }[]`
  más `herramienta`, `herramientaVersion`, `maxTemperatureC`, `stoppedReason`.
- No hay tabla nueva: `test_runs` ya existe (`data-model.md` §, `ui-contract.md` §3.6). El JSON del
  resumen es libre por diseño (J.29). Se documenta su forma nueva en `data-model.md`.
- La interfaz: la pantalla de pruebas gana un **componente de tabla de resultados** para el
  benchmark terminado (o una `Card` con filas `DataRow`, si compone). Ver `plan.md` → decisión de
  componente.

---

## D7 · Qué código se borra

- `src-tauri/src/tests/benchmark.rs` — el motor entero (`ejecutar`, `BufferAlineado`,
  `orden_de_bloques`, `calcular_*`). **Se conserva** solo si algún helper puro sirve al nuevo
  parseo — a priori, nada.
- `src-tauri/src/tests/patron.rs` — el patrón comprobable. Entero.
- `src-tauri/src/tests/mod.rs` — quita los `mod benchmark; mod patron;`.
- Pruebas asociadas en esos módulos.
- `i18n`: `tests.confirm.benchmark.body` cambia (ya no «1 GiB en bloques de 1 MiB secuencial»);
  claves nuevas para la tabla de resultados y el aviso de datos a escribir.
- `docs/product-specification.md` §6 «Prueba de lectura y escritura» se reescribe.

**Se conserva**: `tests/rutas.rs`, `tests/guardia.rs`, `tests/autotest.rs`, `tests/chkdsk.rs`,
`tests/codificacion.rs`.
