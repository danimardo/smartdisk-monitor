# Quickstart — Validación de «Benchmark de disco con DiskSpd»

Guía para comprobar la feature de extremo a extremo. No contiene código de implementación; para el
diseño ver `plan.md`, `research.md`, `data-model.md` y `contracts/pruebas-benchmark.md`.

## Prerrequisitos

- Windows real x64. **Terminal elevado** (la app va con `requireAdministrator`).
- `pnpm install` hecho; toolchain de Rust 1.94.0.
- **`third-party/diskspd/bin/diskspd.exe`** presente (lo coloca el dueño: descarga de
  `https://aka.ms/getdiskspd`, extrae `amd64/diskspd.exe`, verifica que es PE `0x8664`, y aporta su
  SHA-256 para `scripts/verify-assets.mjs` y `THIRD_PARTY_NOTICES.md`). Su licencia MIT en
  `third-party/diskspd/licenses/LICENSE.txt`.
- Un volumen de pruebas con **al menos ~3 GiB libres** por encima de la reserva de seguridad. **No
  usar el volumen del sistema** para las primeras pruebas.
- Idealmente, **CrystalDiskMark** instalado aparte, para comparar cifras (SC-002).

## 1. Puertas automáticas (sin hardware)

```powershell
cd src-tauri; cargo test          # incl. tests::diskspd (args por perfil, cálculo de -d) y
                                  # tests::diskspd_xml (parseo contra fixtures reales, incl. XML abortado)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cd ..; pnpm check                 # cero errores y cero avisos
pnpm test                         # incl. schemas.test.ts: prueba de rechazo de BenchmarkRow / TestResult
pnpm test:component               # la tabla de resultados: completado, parcial (notRun), herramienta ausente
pnpm lint
pnpm verify                       # incl. verify:assets con el hash de diskspd.exe
pnpm test:e2e                     # tests.spec.ts con el fixture del benchmark nuevo
pnpm docs:check                   # historias.md al día
```

Esperado: todo en verde. `cargo test` regenera `src/lib/api/generated/*.ts`; una diferencia con lo
versionado es un fallo de la puerta de tipos del contrato.

## 2. Validación manual en la app (`pnpm app:dev`, muestra UAC)

### 2.1 Ejecución normal

1. Pruebas y diagnóstico → elegir un **volumen que no sea el del sistema** → «Rendimiento»
   → «Configurar y ejecutar».
2. El `ConfirmDialog` debe indicar: los 4 perfiles, que corre lectura **y** escritura, el archivo de
   1 GiB, y **cuántos GB se van a escribir en total** (worst-case según D3). Cancelar aquí **no
   escribe nada** (SC-007).
3. Confirmar. Mientras corre:
   - la barra de progreso avanza en 8 tramos;
   - el texto dice qué perfil/sentido está midiendo;
   - se puede **Cancelar** → la E/S para en ≤ 3 s (SC-003), el archivo temporal **desaparece** de
     `<raíz del volumen>\SmartDisk Monitor Benchmark\`, y la ejecución queda «cancelada» con las
     filas ya medidas.
4. Dejar terminar: aparece una **tabla** con 8 filas (perfil × sentido) y, por fila, **MB/s**,
   **IOPS** y **latencia media (ms)**. Debajo, «Medido con DiskSpd `<versión>`».
5. El archivo temporal se ha borrado. La carpeta `SmartDisk Monitor Benchmark\` puede quedar vacía.

### 2.2 Comparar con CrystalDiskMark (SC-002)

- Correr CrystalDiskMark sobre el **mismo volumen** con su perfil «Peak Performance».
- Las cifras de SmartDisk para SEQ1M Q8T1 y RND4K Q32T1 (lectura y escritura) deben quedar **dentro
  de ±10 %** de las de CDM. Un orden de magnitud de diferencia = bug.

### 2.3 Guardias

| Qué probar | Cómo | Esperado |
|---|---|---|
| **Espacio insuficiente** | volumen con < (1 GiB + reserva) libres | la prueba **no arranca**; error explicado (FR-005). |
| **Parada térmica** | disco cerca de su límite, o bajar `settings.alerts` de temp. crítica a un valor que el disco supere bajo carga | la prueba se **detiene**, estado «detenida por temperatura», la tabla muestra los perfiles ya medidos (FR-006/SC-004). El disco queda por debajo del límite. |
| **Herramienta ausente** | renombrar `diskspd.exe` temporalmente | «Rendimiento» falla con un error claro; **chkdsk y autotest SMART siguen funcionando** (SC-005). |
| **Salida ilegible** | (difícil de forzar a mano; lo cubre el fixture de `cargo test`) | ejecución fallida, detalle técnico conservado, **ninguna cifra inventada**. |
| **Exclusión con autotest SMART** | lanzar el autotest SMART y luego el benchmark en el mismo disco | rechazado con «ya hay una prueba en ese disco» (FR-008). |
| **Reserva nunca invadida** | tras varias ejecuciones, comprobar el espacio libre del volumen | nunca baja de la reserva; no queda ningún `.tmp` (SC-006). |

### 2.4 Historial

- Cada ejecución (completada, cancelada, térmica) aparece en el **historial de pruebas** con fecha,
  volumen, estado y un resumen de cifras; se puede abrir para ver la tabla (US3).

## 3. Verificación del empaquetado (por versión publicada)

- `pnpm app:build` → abrir el instalador NSIS → confirmar que `diskspd.exe` y su licencia llegan a
  la ruta que asume `resolve_diskspd_path()` (`bin/diskspd.exe` junto al ejecutable). Mismo trato
  que la nota «sin verificar contra una compilación empaquetada real» de `smartctl`.
- Con el instalador puesto en una máquina **sin red**, correr el benchmark y confirmar con un
  monitor de paquetes que **no sale ni un byte** (FR-014, mismo criterio que J.33).
- Comprobar si **Control de acceso a carpetas** de Defender interfiere con la escritura en la raíz
  del volumen (research D5 dice que no debería; confirmarlo).
