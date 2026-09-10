# Implementation Plan: Benchmark de disco con DiskSpd (perfiles estilo CrystalDiskMark)

**Branch**: `008-benchmark-diskspd` (el repositorio está en `main`; no se creó rama porque no hay
hook de git de SpecKit configurado) | **Date**: 2026-09-10 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/008-benchmark-diskspd/spec.md`

## Summary

La prueba de «Lectura y escritura» (que pasa a llamarse **«Rendimiento»**) deja de usar un motor
propio (bucle síncrono, cola 1, solo secuencial, verificación de patrón) y pasa a orquestar
**Microsoft DiskSpd**, redistribuido como binario independiente igual que `smartctl.exe`. La prueba corre una **matriz fija de 8 mediciones**
—4 perfiles CrystalDiskMark (secuencial 1 MiB a cola 8 y 1; aleatorio 4 KiB a cola 32 y 1) × lectura
y escritura— sobre un archivo de 1 GiB en la carpeta controlada del volumen, y presenta una **tabla**
con MB/s, IOPS y latencia media por fila.

- **Acotado (FR-017, opción C)**: cada perfil corre `-d` segundos (objetivo 5 s, suelo 2 s) o hasta
  un **tope de datos** de escritura (~4 GiB/perfil), lo que llegue antes. El envoltorio calcula `-d`
  a partir del caudal de lectura ya medido del mismo perfil; si el disco es tan rápido que el tope
  se agota antes del suelo, se mide el suelo y el resultado se etiqueta.
- **Salvaguardas**: se reutilizan `tests::rutas` (carpeta, no-sobrescritura, reserva de espacio) y
  `tests::guardia` (límite térmico). Guardia térmica y cancelación pasan a **matar el proceso
  DiskSpd**; el archivo se borra al terminar o al matar.
- **Se elimina** `tests::benchmark` y `tests::patron` y con ellos la verificación byte a byte.
- **Contrato**: `start_benchmark({ volumeId })` — sin `sizeBytes`/`blockSizeBytes`/`mode`/`passes`.
  El `TestResult` del benchmark gana una **lista de filas de perfil** y la procedencia de la
  herramienta.
- **Dependencias**: **ninguna de código** (el XML de DiskSpd se parsea a mano, como el del Event
  Log). **Un binario redistribuido nuevo** (`diskspd.exe`, MIT) → ADR + `THIRD_PARTY_NOTICES` +
  `verify:assets`. **Sin permiso de Tauri nuevo.**

## Technical Context

**Language/Version**: Rust 1.94.0 (edición 2021) en el backend; TypeScript 5.9.3 / Svelte 5.57
(runes) en la interfaz. Versiones fijadas por la constitución.

**Primary Dependencies**: **Ninguna nueva de código.** Backend: `platform::proceso_externo`
(ya existe, lanza procesos con `CREATE_NO_WINDOW` y drena pipes), `tracing`, `rusqlite`, `time`,
`ts-rs`, `serde`. Parseo del XML de DiskSpd **a mano** (patrón de `collectors/event_log.rs`).
Interfaz: Zod 4.5.4, catálogo propio de componentes.

**Recurso redistribuido nuevo**: `third-party/diskspd/bin/diskspd.exe` (amd64, MIT, ~200 KB) + su
licencia. Empaquetado por `tauri.conf.json` → `bundle.resources`, hash en `scripts/verify-assets.mjs`
y `THIRD_PARTY_NOTICES.md`. **Lo coloca el dueño** (el hook `proteger-rutas.mjs` impide al agente
escribir en `third-party/`).

**Storage**: SQLite `test_runs` (migración 0001), **sin cambio de esquema**. El resumen del
resultado vive en `result_summary_json` (columna libre por diseño, J.29); cambia su forma, no la
tabla. Sin estado en memoria persistente nuevo (la cancelación reutiliza `test_cancel_flags` de
`AppState`).

**Testing**: `cargo test` — parseo del XML de DiskSpd contra **fixtures reales** (como
`smartctl_parser.rs`); construcción de la línea de comandos de cada perfil (pura); cálculo de `-d`
efectivo (puro); `tests::guardia` y `tests::rutas` ya cubiertos. `pnpm test` — esquema Zod nuevo del
`TestResult` + su prueba de rechazo. `pnpm test:component` — la tabla de resultados del benchmark
(estados: completado, parcial por parada térmica/cancelación, herramienta ausente). `pnpm test:e2e`
— la pantalla de pruebas con el fixture del benchmark nuevo; `capturas.spec.ts` regenera la captura
de `pruebas`. `pnpm check`.

**Target Platform**: Windows 10 1809+ / Server 2016+, **x64 únicamente** (DiskSpd solo amd64),
proceso elevado.

**Project Type**: Aplicación de escritorio (Tauri 2 + SvelteKit `adapter-static`, SSR off).

**Performance Goals**: la matriz completa termina en < ~2 min sobre un SSD típico (SC-001);
cancelar detiene la E/S en ≤ 3 s (SC-003, = matar el proceso). Una invocación de DiskSpd por
medición (8), no una sola larga, para que el progreso sea real y un cuelgue no arrastre al resto.

**Constraints**: cero red (FR-014, DiskSpd corre offline). Cifras nunca inventadas: una salida de
DiskSpd ilegible → ejecución fallida con el detalle conservado, jamás un número a ojo (§I). El
archivo del benchmark **nunca** sobrescribe uno existente ni invade la reserva de seguridad. Ningún
`println!`; el volcado de DiskSpd **no** entra en el log (puede llevar rutas). El proceso DiskSpd se
lanza siempre por `platform::proceso_externo` (`CREATE_NO_WINDOW`, sin ventana de consola).

**Scale/Scope**: 8 mediciones por ejecución, 1 archivo de 1 GiB. Superficie: **se borra**
`tests/benchmark.rs` + `tests/patron.rs`; **se añade** `tests/diskspd.rs` (orquestación) +
`tests/diskspd_xml.rs` (parseo) + `platform/diskspd.rs` (ruta del binario); cambios en
`commands/mod.rs` (`start_benchmark`, el hilo de ejecución, `ResumenPruebaJson`); `tauri.conf.json`,
`verify-assets.mjs`, `THIRD_PARTY_NOTICES.md`; interfaz (`routes/tests/+page.svelte` + un componente
de tabla o composición con `DataRow`); esquemas Zod; i18n; y **6 documentos normativos + ADR-053**
(product-specification §6, ui-contract §3.6, data-model, open-questions, roadmap, testing-strategy si
cambia una suite).

## Constitution Check

*GATE: debe pasar antes de la Fase 0 y volver a comprobarse tras la Fase 1.*

| Principio | Aplicación en esta feature | Estado |
|---|---|---|
| **I. Veracidad del dato** | Salida de DiskSpd ilegible o herramienta ausente → ejecución **fallida** con detalle técnico, nunca cifras inventadas (FR-013). Un perfil que no llegó a correr por parada anticipada → sin fila o «no ejecutado», nunca 0 (FR/spec). El resultado declara herramienta y versión (FR-012). Si el tope de datos fuerza una medición corta, el resultado lo dice (D3). | ✅ Pasa |
| **II. Orden de prioridades** | Seguridad del disco (reserva de espacio, guardia térmica, tope de datos) por encima de la exhaustividad de la medición. Simplicidad: **reutilizar el motor estándar** (DiskSpd) en vez de reimplementar E/S asíncrona; parsear XML a mano en vez de añadir un crate. | ✅ Pasa |
| **III. Pila fija / cero red / dependencias** | **Cero red** (DiskSpd offline, FR-014). **Cero dependencia de código.** **Un binario redistribuido nuevo** (`diskspd.exe`): amplía la superficie de un binario privilegiado → **exige ADR-053 y justificación escrita** (`AGENTS.md`, límites duros), y entrada en `THIRD_PARTY_NOTICES` + `verify:assets`. La licencia MIT no impone obligación de fuente (a diferencia de la GPLv2 de smartmontools). | ⚠️ Requiere **ADR-053** (binario redistribuido nuevo) |
| **IV. Dominio ↔ presentación** | La construcción de la línea de comandos, el cálculo de `-d` efectivo y el parseo del XML son **puros** (`tests/diskspd*.rs`), probables con fixtures sin lanzar nada. El comando `start_benchmark` valida y delega. La tabla de la interfaz solo pinta el resultado; no lo recalcula. | ✅ Pasa |
| **V. Persistencia local** | Sin cambio de esquema SQLite. El resumen cambia de forma dentro de `result_summary_json` (columna libre, J.29). El archivo temporal vive en el volumen probado, se borra siempre que se puede, y su ruta queda en `temp_path`/`orphanPath` si no. UTC en persistencia. | ✅ Pasa |
| **VI. Sistema de diseño** | Tabla de resultados: si compone del catálogo (`Card` + `DataRow` o similar) no hace falta componente nuevo; si no, uno nuevo con el criterio de `ui-design.md` §3 (aparece en ≥3 pantallas → **no**, así que se compone). Cero literales visuales, cero literales de interfaz: claves nuevas en `es.json` y `en.json`. Correcta en claro/oscuro y a 1024×560. | ✅ Pasa (componer, no crear) |
| **VII. Accesibilidad** | La tabla de resultados con encabezados de columna reales (`<th>` / `scope`), navegable por teclado. El aviso previo (impacto + datos a escribir) en el `ConfirmDialog` ya existente. Progreso en vivo con texto, no solo barra. | ✅ Pasa |
| **VIII. Testeabilidad** | Parseo del XML y cálculo de `-d` son «área donde el error es silencioso» → **test-first** con fixtures. Cobertura de `domain/`/`tests/` puros ≥ 90 %. La orquestación real (lanzar DiskSpd) queda como el resto de E/S real del proyecto: no se prueba en `cargo test`, se valida en `quickstart.md`. | ✅ Pasa |
| **X. Errores** | Herramienta ausente → `AppError` con código estable (`test.tool_missing` o similar), clave i18n, detalle. Salida ilegible → `test.tool_output_unreadable`. DiskSpd devuelve error → `test.io_failed` con su stderr en el detalle (nunca en el log). Se degrada la prueba; chkdsk y autotest siguen (FR-013, SC-005). | ✅ Pasa |
| **XI. Validación de fronteras** | `TestResult` nuevo se genera con `ts-rs`; su Zod se infiere con `z.infer` y lleva **prueba de rechazo** (fila de perfil malformada → `ipc.schema_mismatch`). Nada de `as` sobre datos de `invoke`. El XML de DiskSpd se deserializa en un struct explícito, no `serde_json::Value`. | ✅ Pasa |
| **XII. Sin variables de entorno** | Ninguna. La ruta de `diskspd.exe` se resuelve por `cfg!(debug_assertions)` + `current_exe()` (patrón de `resolve_smartctl_path`), no por env. | ✅ Pasa |
| **XIII. `svelte-check`** | `pnpm check` cero errores y cero avisos tras adaptar los consumidores del contrato. | ✅ Pasa (se verifica al implementar) |
| **XIV. SvelteKit idiomático** | El progreso llega por el evento `test:progress` que ya existe (el backend empuja; la interfaz no sondea). El resultado, por `get_test_runs`/`test:progress` al terminar. `$derived` para la tabla. | ✅ Pasa |
| **XV. Registro** | El sondeo de temperatura durante la prueba no loguea por tick. Un fallo de DiskSpd se registra `warn` una vez, **sin** su volcado (puede llevar rutas). Ningún número de serie ni ruta de perfil de usuario. | ✅ Pasa |
| **Límites duros (`AGENTS.md`)** | **Cambia el contrato** (`start_benchmark`, `TestResult`) → regenerar `ts-rs`, actualizar `ui-contract.md` §3.6 + ADR. **Binario nuevo** → ADR-053, `THIRD_PARTY_NOTICES`, `verify:assets`. **El binario lo coloca el dueño** (hook `proteger-rutas.mjs` sobre `third-party/`). Reescribe `product-specification.md` §6. **Sin permiso de Tauri nuevo, sin dependencia de código nueva.** El hook puede pedir confirmación al editar `docs/decisions.md`/contratos durante la implementación: esperado. | ⚠️ ADR-053 + contratos + `THIRD_PARTY_NOTICES` (previsto en tasks) |

**Veredicto**: sin violaciones. Dos puntos de fricción, ambos dentro del proceso normal:
1. **Binario redistribuido nuevo** → ADR-053, como ADR-005 en su día para smartctl. Justificación:
   el objetivo de producto (cifras profesionales comparables con CrystalDiskMark) no se alcanza sin
   E/S asíncrona real, y reimplementarla bien es un coste permanente y arriesgado (§I); DiskSpd es
   el motor estándar del sector, MIT, sin obligación de fuente, ~200 KB.
2. **Cambio de contrato** → regeneración `ts-rs` + `ui-contract.md`, como ADR-050.

**Re-comprobación tras la Fase 1**: el diseño (research.md, data-model.md, contracts/, quickstart.md)
no introduce ninguna violación nueva. El parseo del XML a mano (D4) evita la dependencia que se
había marcado como incógnita. `serde_json::Value` no aparece como tipo de dominio. La tabla de
resultados se compone del catálogo, no crea componente. Sin NEEDS CLARIFICATION pendientes. Los dos
⚠️ (ADR-053, contrato) siguen siendo trabajo previsto en `tasks.md`, no bloqueos.

## Project Structure

### Documentation (this feature)

```text
specs/008-benchmark-diskspd/
├── plan.md              # Este fichero
├── research.md          # Fase 0 — decisiones D1–D7
├── data-model.md        # Fase 1 — forma nueva del resultado y del contrato
├── quickstart.md        # Fase 1 — validación manual end-to-end
├── contracts/
│   └── pruebas-benchmark.md   # Fase 1 — el comando y el DTO nuevos
├── checklists/
│   └── requirements.md
└── tasks.md             # Fase 2 (/speckit-tasks) — no lo crea este comando
```

### Source Code (repository root)

```text
src-tauri/src/
├── platform/
│   ├── diskspd.rs              # NUEVO: resolve_diskspd_path() — calcado de resolve_smartctl_path
│   └── proceso_externo.rs      # SIN CAMBIOS: se reutiliza ejecutar_con_limite
├── tests/
│   ├── diskspd.rs              # NUEVO: perfiles, línea de comandos por perfil, cálculo de -d,
│   │                           #        orquestación de la matriz (pura salvo el lanzamiento)
│   ├── diskspd_xml.rs          # NUEVO: parseo a mano del XML de DiskSpd → struct explícito
│   ├── rutas.rs                # SIN CAMBIOS: carpeta, no-sobrescritura, reserva de espacio
│   ├── guardia.rs              # SIN CAMBIOS: límite térmico
│   ├── benchmark.rs            # ❌ BORRAR (motor propio)
│   ├── patron.rs               # ❌ BORRAR (verificación byte a byte)
│   └── mod.rs                  # quita `mod benchmark; mod patron;`, añade los nuevos
├── commands/
│   └── mod.rs                  # start_benchmark: quita parámetros, corre la matriz;
│                               # ResumenPruebaJson: lista de filas de perfil + procedencia;
│                               # el hilo de ejecución: 8 invocaciones, guardia y cancelación
│                               # por muerte del proceso
└── ...

third-party/diskspd/            # NUEVO — lo coloca el dueño
├── bin/diskspd.exe
├── licenses/LICENSE.txt
└── README.md

src/
├── lib/api/
│   ├── schemas.ts              # TestResult nuevo (filas de perfil) + prueba de rechazo
│   └── generated/              # regenerado por ts-rs
├── lib/components/
│   └── (tabla de resultados: componer con Card + DataRow, o BenchmarkResults.svelte si no compone)
├── lib/i18n/{es,en}.json       # claves nuevas; tests.confirm.benchmark.body reescrito
└── routes/tests/+page.svelte   # start_benchmark sin parámetros; render de la tabla al terminar

e2e/ui/fixtures/respuestas.ts   # fixture del benchmark nuevo (TestRun con filas de perfil)
e2e/ui/tests.spec.ts            # ajuste del test del benchmark
scripts/verify-assets.mjs       # entrada de diskspd.exe con SHA-256
src-tauri/tauri.conf.json       # bundle.resources: diskspd.exe + su licencia
THIRD_PARTY_NOTICES.md          # sección DiskSpd (MIT)
docs/                           # product-specification §6, ui-contract §3.6, data-model,
                                # open-questions, roadmap, decisions (ADR-053)
```

**Structure Decision**: aplicación de escritorio, un solo proyecto backend (`src-tauri/`) + un
frontend (`src/`). El benchmark vive en `src-tauri/src/tests/` (ya existe para las tres pruebas) y
en `src-tauri/src/platform/` (resolución del binario, como `smartctl`). La interfaz reutiliza la
pantalla de pruebas.

## Complexity Tracking

| Violación | Por qué se necesita | Alternativa más simple rechazada porque |
|---|---|---|
| **Binario redistribuido nuevo (`diskspd.exe`)** | Cifras de rendimiento comparables con CrystalDiskMark exigen E/S asíncrona real (colas 8/32, medición estable). DiskSpd es el motor estándar del sector, el que usa CDM. | Un motor propio con overlapped I/O / IOCP vía FFI: mucho código `unsafe`, y clavar las cifras para que no engañen (§I) es difícil y un coste permanente. Se valoró y descartó con el dueño. La extensión modesta del motor actual (cola 1, sin IOPS) no cumple el objetivo de producto. |
