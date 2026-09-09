# Quickstart — Validación de «Actividad de disco representativa»

Guía para comprobar que la feature funciona de extremo a extremo. No contiene código de
implementación; para el diseño ver `plan.md`, `data-model.md` y `contracts/disk-activity.md`.

## Prerrequisitos

- Windows real (el muestreo PDH no funciona en CI ni en Linux).
- Terminal **elevado** (la app va con `requireAdministrator`).
- `pnpm install` hecho; toolchain de Rust 1.94.0.

## 1. Puertas automáticas (sin hardware)

```powershell
cargo test                       # incl. domain::actividad (media, pico, parcial, hueco, muestra fallida)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
pnpm check                       # cero errores y cero avisos
pnpm test                        # incl. schemas.test.ts: prueba de rechazo del objeto `activity`
pnpm test:component              # DiskCard / MetricCard en estados valido / parcial / no_disponible
pnpm lint
pnpm verify                      # hashes, tokens, i18n sincronizado, fronteras
pnpm docs:check                  # historias.md al día tras regenerar
```

Esperado: todo en verde. `cargo test` regenera `src/lib/api/generated/*.ts`; una diferencia con lo
versionado es un fallo de la puerta de tipos del contrato (constitución, Puertas de calidad).

## 2. Validación manual en la app (`pnpm app:dev`)

### 2.1 La actividad sube con carga real (US1, SC-001)

1. Arranca la app. Espera ~40 s (una cadencia de métricas rápidas + margen).
2. En otra ventana, genera carga sostenida sobre un disco, p. ej.:
   `Get-ChildItem C:\ -Recurse -ErrorAction SilentlyContinue | Out-Null` (lectura intensa), o una
   copia grande de archivos hacia/desde ese disco.
3. **Panel general**: la cifra de actividad de ese disco sube a un valor claramente alto
   (no 0–1 %) en menos de 30 s, y se mantiene mientras dura la carga.
4. Repite 3–4 veces arrancando la carga en momentos distintos: el resultado es estable, no depende
   de «acertar» el instante de muestreo.
5. Para la carga → la cifra baja hacia valores bajos en la ventana siguiente.

### 2.2 Media vs pico (US2)

1. Con la app abierta, provoca una ráfaga corta (2–3 s) de actividad y luego deja el disco en calma.
2. Abre el **detalle** de ese disco. Durante ~30 s: el **pico** refleja la ráfaga, la **media** es
   bastante menor y va bajando. Pasados ~30 s ambos vuelven a valores bajos.
3. La ayuda contextual de «Actividad» dice que es un agregado de los últimos ~30 s (media y pico),
   no una lectura instantánea.

### 2.3 Arranque: nada inventado (SC-003, FR-004)

1. Cierra y reabre la app. Durante los primeros ~30 s, la actividad de cada disco se muestra como
   **parcial** («midiendo…») o **no disponible**, nunca como un `0 %` ni como un número «normal».
2. La serie histórica (`gráfica de actividad` en el detalle, 24 h): tras el arranque hay un **hueco**
   de como mucho una cadencia, dibujado como hueco, no interpolado (FR-010a).

### 2.4 Fuente degradada no cae a cero (SC-004)

- No hay forma cómoda de forzar un fallo de PDH a mano; se comprueba por inspección de log a nivel
  `debug`: un `warn` puntual «no se pudo leer los contadores de rendimiento» **no** produce una
  caída a 0 % en el panel ni una fila `activity_percent` a 0 en la base. Si la fuente se degrada de
  verdad, la tarjeta muestra el estado de fuente degradada (comportamiento ya existente).

### 2.5 Batería (FR-017, D.2)

- Con el portátil desenchufado, el muestreo pasa a ~4 s. La actividad sigue actualizándose en cada
  ciclo de métricas rápidas (que en batería también se espacia ×4). Al enchufar, se restaura.

### 2.6 Coste en reposo (SC-005)

- Con los discos en reposo, el uso de CPU de `SmartDisk Monitor.exe` en el Administrador de tareas
  no es distinguible del de la versión anterior (una lectura de contador por segundo es
  despreciable).

## 3. Comprobaciones de «definición de terminado» de interfaz (`docs/ui-design.md` §8)

Sobre `DiskCard` y el detalle de disco: tema claro y oscuro; acento propio y del sistema;
1024 × 560 y 1280 × 720; escalado 125/150/200 %; estados **valido / parcial / no disponible** de la
actividad; teclado y foco en el `MetricCard` del detalle; textos en español e inglés.

## 4. Documentación (al cerrar, skill `cierre-tarea`)

- `docs/open-questions.md`: nueva subsección **D.4** con los valores adoptados (intervalo de
  muestreo, tamaño de ventana, umbral de hueco, comportamiento en batería, hueco en histórico),
  estado `PROPUESTO`.
- `docs/architecture.md` «Colector Windows»: la lectura de actividad pasa a describir la consulta
  PDH persistente y la ventana deslizante.
- `docs/data-model.md` §3 (`activity_percent`): «media de una ventana deslizante…».
- `docs/ui-contract.md` §3.2 y la nota de `src/lib/design/types.ts`: `activity: ActividadDisco`.
- `docs/decisions.md`: **ADR-050** — consulta PDH persistente para la actividad de disco (cambio de
  arquitectura de recogida + cambio de contrato).
- `pnpm docs:build` para regenerar `historias.md`; confirmar los dos cambios juntos.
