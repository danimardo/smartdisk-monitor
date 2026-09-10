# Feature Specification: Benchmark de disco con DiskSpd (perfiles estilo CrystalDiskMark)

**Feature Branch**: `008-benchmark-diskspd`

**Created**: 2026-09-10

**Status**: Draft

**Input**: User description: "Sustituir el benchmark de disco actual por una prueba de rendimiento basada en Microsoft DiskSpd. DiskSpd se redistribuye como binario independiente (licencia MIT, solo amd64, ~200 KB), invocado como proceso externo igual que smartctl.exe. La prueba ejecuta 3-4 perfiles fijos con nombre estilo CrystalDiskMark —SEQ1M Q8T1, SEQ1M Q1T1, RND4K Q32T1, RND4K Q1T1— en lectura y escritura sobre un archivo temporal en la carpeta controlada del volumen, y muestra en una tabla los resultados: MB/s, IOPS y latencia media por perfil. Se conservan las guardias existentes (espacio, térmica, cancelación, exclusión con el autotest SMART, borrado del archivo). Se ELIMINA el motor de benchmark propio y la verificación de patrón byte a byte. El objetivo es dar cifras comparables con CrystalDiskMark."

## Contexto

La pantalla **Pruebas y diagnóstico** ofrece hoy una prueba de «Lectura y escritura» (US-020,
`docs/product-specification.md` §6): crea un archivo temporal de 1 GiB en una carpeta controlada del
volumen, lo escribe en bloques de 1 MiB con un patrón comprobable, lo sincroniza, lo lee entero y
verifica el contenido byte a byte, y borra el archivo al terminar. Mide caudal aproximado (bytes/s)
y latencia media, y detecta una escritura que el disco no persiste de verdad.

Como ya no verifica contenido, esta prueba pasa a llamarse **«Rendimiento»** («Performance» en
inglés).

Sus límites, reconocidos en el propio código y en `docs/open-questions.md` J.29:

- La E/S es **síncrona y de una sola operación en vuelo** (profundidad de cola efectiva 1) y un solo
  hilo. Eso no refleja cómo trabaja un SSD moderno bajo carga real: un NVMe da su rendimiento
  cuando se le encolan decenas de operaciones a la vez.
- Solo hay un acceso **secuencial**; no hay una prueba **aleatoria de bloque pequeño** (4 KiB), que
  es la que de verdad separa un disco bueno de uno malo para el uso cotidiano.
- No reporta **IOPS**, solo MB/s y latencia.
- La ejecución real nunca se ha verificado con el tamaño de producción ni sobre un volumen de
  usuario (decisión de alcance explícita de la sesión que lo construyó).

Consecuencia: quien conoce **CrystalDiskMark** —el benchmark de disco de referencia en Windows— y
compara sus cifras con las de esta prueba no reconoce los números, y la prueba parece de juguete.
CrystalDiskMark en sus versiones recientes usa **Microsoft DiskSpd** como motor de medición; DiskSpd
es una herramienta de Microsoft, de línea de comandos, con licencia permisiva, que mide caudal,
IOPS y latencia con E/S asíncrona real y perfiles configurables de cola e hilos.

Decisión de producto adoptada con el dueño: **se sustituye el motor propio por DiskSpd**, invocado
como proceso externo del mismo modo que `smartctl.exe` (nunca enlazado, comunicación por línea de
comandos y salida estándar). La prueba pasa a ejecutar un **conjunto fijo de perfiles con nombre**
al estilo de CrystalDiskMark, y presenta los resultados en una **tabla**: por perfil y sentido
(lectura / escritura), su caudal en MB/s, sus IOPS y su latencia media. Se **elimina** el motor de
benchmark propio y, con él, la verificación de patrón byte a byte del contenido escrito: no se
conserva ni se sustituye por otra comprobación de integridad (decisión explícita del dueño).

Se conservan **todas las salvaguardas actuales**: comprobación previa de espacio libre contra la
reserva de seguridad, guardia térmica que detiene la prueba si el disco alcanza su límite crítico,
cancelación por el usuario en cualquier momento, exclusión mutua con el autotest SMART corto del
mismo disco físico, y borrado del archivo temporal al terminar, cancelar o fallar.

Esta especificación describe **qué** debe pasar y **por qué**. El *cómo* —versión exacta de DiskSpd,
la línea de comandos de cada perfil, el formato de salida que se parsea, el tamaño y la duración de
cada perfil, la forma exacta del contrato y del resultado— se fija en el plan, y todo valor numérico
nuevo se registra en `docs/open-questions.md` con su valor propuesto **antes** de programarse
(constitución, flujo de desarrollo §1). El binario redistribuido y su licencia exigen un ADR
(constitución §III).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Medir el rendimiento de un disco con cifras reconocibles (Priority: P1)

Alguien quiere saber cómo de rápido es un disco: para decidir si el SSD que acaba de instalar rinde
lo que promete, para comparar dos unidades, o para documentar un problema de lentitud. Elige el
volumen, lanza la prueba, y al terminar ve una tabla con varias filas —acceso secuencial de bloque
grande y acceso aleatorio de bloque pequeño, cada uno a cola baja y a cola alta, en lectura y en
escritura— y, para cada una, los **MB/s**, los **IOPS** y la **latencia media**. Los números son del
mismo orden que los que daría CrystalDiskMark sobre ese mismo disco.

**Why this priority**: es el motivo de la funcionalidad. La prueba actual da un número que quien
sabe de discos no reconoce; una prueba de rendimiento que no se puede comparar con la referencia del
sector no aporta. Sin este escenario resuelto, el resto no importa.

**Independent Test**: se lanza la prueba sobre un volumen real, se deja terminar, y se comprueba que
la tabla de resultados trae, por cada perfil y sentido, un caudal, unos IOPS y una latencia
plausibles y del mismo orden de magnitud que CrystalDiskMark para el perfil equivalente en ese
disco.

**Acceptance Scenarios**:

1. **Given** un volumen con espacio de sobra y el disco a temperatura normal, **When** el usuario
   lanza la prueba y la deja terminar, **Then** se muestra una tabla con una fila por perfil y
   sentido, cada una con MB/s, IOPS y latencia media, y el estado de la ejecución es «completada».
2. **Given** una ejecución completada, **When** el usuario mira los resultados, **Then** se indica
   qué herramienta y qué versión los produjo (para poder comparar entre ejecuciones y con otras
   herramientas).
3. **Given** una ejecución completada, **When** el usuario compara el perfil secuencial de lectura
   con el de CrystalDiskMark sobre el mismo disco, **Then** las cifras difieren en un margen
   pequeño, no en un orden de magnitud.

---

### User Story 2 - Que la prueba no dañe ni bloquee el equipo (Priority: P1)

La prueba escribe y lee gigabytes a máxima velocidad; puede calentar el disco, consumir espacio y
ralentizar el equipo mientras dura. El usuario necesita que la prueba se comporte: que avise antes,
que no se coma el espacio libre, que se pare sola si el disco se calienta demasiado, que se pueda
cancelar, y que no deje basura.

**Why this priority**: una prueba destructiva del rendimiento en un proceso elevado, sin
salvaguardas, es un riesgo inaceptable en una herramienta cuyo objetivo es *cuidar* los discos
(constitución §I, §II). Estas garantías ya existen para la prueba actual y no pueden perderse al
cambiar de motor.

**Independent Test**: se fuerza cada condición de parada (poco espacio, temperatura alta simulada,
cancelación del usuario) y se comprueba que la prueba se detiene con la razón correcta, conserva los
resultados parciales que hubiera, y no deja el archivo temporal atrás.

**Acceptance Scenarios**:

1. **Given** un volumen donde el archivo de la prueba no cabe tras descontar la reserva de seguridad
   (2 GiB o el 5 % del volumen, la mayor), **When** el usuario lanza la prueba, **Then** la prueba
   no empieza y se explica por qué.
2. **Given** una prueba en curso, **When** el disco alcanza su límite de temperatura crítico (el del
   fabricante si lo declara, si no el configurado), **Then** la prueba se detiene, se conserva lo
   medido hasta ese punto, y el estado es «detenida por temperatura».
3. **Given** una prueba en curso, **When** el usuario pulsa cancelar, **Then** la E/S se detiene en
   pocos segundos, se conserva lo medido, y el archivo temporal se borra.
4. **Given** una prueba que termina, se cancela o falla, **When** se resuelve, **Then** el archivo
   temporal se elimina; si el borrado no fuera posible, su ruta queda visible para limpieza manual.
5. **Given** un autotest SMART corto en curso sobre un disco, **When** el usuario intenta lanzar la
   prueba de rendimiento sobre el mismo disco físico, **Then** se rechaza con un aviso de que ya hay
   una prueba en ese disco (y a la inversa).

---

### User Story 3 - Consultar y comparar resultados anteriores (Priority: P2)

El usuario ha corrido la prueba varias veces —en discos distintos, o en el mismo disco antes y
después de un cambio— y quiere revisar el historial para comparar.

**Why this priority**: el valor de un benchmark crece si se pueden comparar ejecuciones. El
historial de pruebas ya existe en la aplicación; esta funcionalidad solo tiene que alimentarlo con
un resultado más rico.

**Independent Test**: se lanzan dos ejecuciones sobre volúmenes distintos y se comprueba que ambas
quedan en el historial con sus resultados, sus parámetros y su estado, y que se pueden abrir para
ver la tabla de cifras.

**Acceptance Scenarios**:

1. **Given** una ejecución terminada, **When** el usuario abre el historial de pruebas, **Then** la
   ejecución aparece con su fecha, el volumen probado, el estado y un resumen de las cifras.
2. **Given** una ejecución detenida por temperatura o cancelada, **When** aparece en el historial,
   **Then** se distingue de una completada y muestra los resultados parciales que hubiera.

---

### Edge Cases

- **La herramienta de benchmark falta o no arranca** (fichero ausente, corrupto, bloqueado por un
  antivirus, incompatible con la versión de Windows): la prueba falla con un error claro, y el resto
  de la pantalla de pruebas (chkdsk, autotest SMART) sigue funcionando.
- **La herramienta devuelve una salida que no se entiende** (versión inesperada, formato cambiado,
  salida truncada): la ejecución se marca como fallida con el detalle técnico conservado, nunca se
  inventan cifras.
- **El volumen no tiene letra de unidad asignada**: la prueba no puede crear su archivo; se explica.
- **El disco se retira o el volumen se desmonta a mitad de prueba**: la ejecución termina en error,
  el archivo (si sigue accesible) se intenta borrar, y se registra.
- **Cancelación o parada térmica a mitad de la matriz de perfiles**: se conservan los perfiles ya
  medidos; los que no llegaron a correr se muestran como no ejecutados, no como cero.
- **Volumen extraíble o de red**: se permite lanzar la prueba, pero los resultados serán ruidosos;
  no se le da un trato especial más allá de lo que ya hacen las salvaguardas de espacio.
- **El equipo entra en suspensión durante la prueba**: al reanudar, la prueba se considera fallida
  (no se puede garantizar que las cifras del tramo afectado sean válidas).
- **Escrituras acumuladas**: la matriz completa incluye varios perfiles de escritura; el usuario
  debe entender antes de empezar cuántos datos se van a escribir en total sobre el SSD.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: La prueba DEBE ejecutar un **conjunto fijo de perfiles con nombre**, que cubra como
  mínimo: acceso secuencial de bloque grande y acceso aleatorio de bloque pequeño, cada uno a
  profundidad de cola baja y a profundidad de cola alta. Cada perfil se mide en **lectura** y en
  **escritura**. El usuario no configura cola, hilos ni bloque.
- **FR-002**: Para cada combinación de perfil y sentido, la prueba DEBE reportar **caudal en MB/s**,
  **IOPS** y **latencia media en milisegundos**.
- **FR-003**: La prueba DEBE escribir sobre un **archivo temporal** en la carpeta controlada del
  volumen probado (`<raíz del volumen>\SmartDisk Monitor Benchmark\`), con un nombre que no colisione
  y **comprobando activamente que no existe** antes de crearlo; **nunca** sobrescribe un archivo
  existente.
- **FR-004**: La prueba DEBE **borrar el archivo temporal** al completarse, al cancelarse y al
  fallar. Si el borrado no es posible, la ruta DEBE quedar registrada y visible para limpieza
  manual.
- **FR-005**: La prueba DEBE **rechazar el arranque** si el archivo que necesita el perfil no cabe
  en el volumen tras descontar la **reserva de seguridad** de 2 GiB o el 5 % de la capacidad del
  volumen, la mayor de las dos.
- **FR-006**: La prueba DEBE **detenerse automáticamente** si el disco alcanza su límite de
  temperatura crítico —el del fabricante si lo declara, si no el configurado en Ajustes—,
  conservando los resultados de los perfiles ya medidos y registrando «detenida por temperatura».
- **FR-007**: El usuario DEBE poder **cancelar** la prueba en cualquier momento; la E/S DEBE cesar en
  pocos segundos y los resultados parciales DEBEN conservarse.
- **FR-008**: La prueba NO DEBE ejecutarse a la vez que un autotest SMART corto sobre el **mismo
  disco físico**, ni a la vez que otra prueba de rendimiento sobre el mismo disco.
- **FR-009**: Mientras corre, la prueba DEBE mostrar **progreso en vivo**: qué perfil se está
  midiendo y un porcentaje de avance de la matriz completa.
- **FR-010**: Antes de empezar, la prueba DEBE **advertir del impacto**: bajada temporal del
  rendimiento del equipo, calentamiento del disco y **cuántos datos se van a escribir en total**
  sobre el SSD.
- **FR-011**: Cada ejecución DEBE quedar registrada en el **historial de pruebas** con sus
  resultados, sus parámetros, su estado y —si aplica— su razón de parada.
- **FR-012**: El resultado DEBE **declarar qué herramienta y qué versión** lo produjo, para que las
  cifras se puedan comparar entre ejecuciones y con otras herramientas.
- **FR-013**: Si la herramienta de benchmark **falta, no se puede ejecutar, o devuelve una salida
  que no se puede interpretar**, la ejecución DEBE fallar con un error comprensible y el detalle
  técnico conservado; el resto de la aplicación DEBE seguir funcionando (se degrada la función, no
  la aplicación).
- **FR-014**: La prueba NO DEBE realizar **ninguna comunicación de red** en ningún momento.
- **FR-015**: La herramienta de benchmark redistribuida DEBE acompañarse de su **licencia y avisos
  de autoría** en la instalación, y su integridad DEBE verificarse en la compilación igual que la de
  los demás binarios redistribuidos.
- **FR-016**: El motor de benchmark propio actual y la **verificación de patrón byte a byte** del
  contenido escrito **se eliminan**. Esta comprobación de integridad **no se conserva ni se
  sustituye**.
- **FR-017**: Cada perfil se acota **por tiempo con un tope de datos**: corre durante una duración
  objetivo (del orden de unos segundos) **o** hasta mover un volumen máximo de datos, lo que llegue
  antes. Esto da cifras comparables en discos normales y **acota el desgaste del SSD** en los muy
  rápidos. Cuando un perfil termina por el tope de datos antes de agotar su tiempo, el resultado lo
  DEBE indicar (p. ej. «medido durante 2,1 s, tope de datos alcanzado»). Los valores concretos de
  duración objetivo y tope se fijan en el plan y se registran en `docs/open-questions.md`.

### Key Entities *(include if feature involves data)*

- **Ejecución de benchmark**: una corrida completa de la matriz de perfiles sobre un volumen. Tiene
  fecha de inicio y fin, volumen y disco objetivo, estado (en curso / completada / cancelada /
  fallida / detenida por temperatura), razón de parada, parámetros usados (perfiles, tamaño,
  herramienta y versión) y el conjunto de resultados. Es una fila más del historial de pruebas
  existente.
- **Perfil**: una configuración con nombre de acceso (secuencial/aleatorio), tamaño de bloque y
  profundidad de cola. El conjunto es **fijo** y no editable por el usuario.
- **Fila de resultado**: por cada perfil y sentido (lectura/escritura), el caudal (MB/s), los IOPS y
  la latencia media (ms). Un perfil que no llegó a ejecutarse (parada anticipada) no tiene fila, o
  la tiene marcada como «no ejecutado» — nunca a cero.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Sobre un SSD típico y con el tamaño de archivo por defecto, la matriz completa de
  perfiles (secuencial y aleatorio, cola baja y alta, lectura y escritura) termina en **menos de
  ~2 minutos**.
- **SC-002**: Para un perfil dado, el caudal y los IOPS que reporta la prueba están **dentro de un
  ±10 %** de los que da CrystalDiskMark para el perfil equivalente sobre el mismo disco (verificable
  a mano comparando ambas herramientas).
- **SC-003**: Cancelar una prueba en curso detiene toda la E/S en **≤ 3 segundos** y el archivo
  temporal desaparece.
- **SC-004**: Una parada térmica deja el disco **por debajo de su límite crítico** y la ejecución
  queda marcada «detenida por temperatura» con los perfiles que sí se completaron.
- **SC-005**: Con el binario de la herramienta retirado del sistema, lanzar la prueba muestra un
  error y las otras dos pruebas de la pantalla (chkdsk, autotest SMART) **siguen funcionando**.
- **SC-006**: En ninguna circunstancia se crea un archivo temporal que invada la reserva de
  seguridad del volumen.
- **SC-007**: Antes de la primera ejecución, el usuario ve **cuántos GB se van a escribir en total**
  sobre el SSD y puede cancelar sin que se escriba nada.

## Assumptions

- **Plataforma**: Windows x64 únicamente, coherente con lo que declara el proyecto hoy (`README.md`,
  `tauri.conf.json`). El binario de la herramienta se empaqueta solo para amd64.
- **Herramienta**: Microsoft **DiskSpd**, redistribuido bajo su licencia MIT del mismo modo que
  `smartctl.exe` (proceso independiente, nunca enlazado). La versión exacta la fija su ADR.
- **Conjunto de perfiles**: por defecto los cuatro «de pico» de CrystalDiskMark —secuencial 1 MiB a
  cola 8 y a cola 1; aleatorio 4 KiB a cola 32 y a cola 1, un hilo en todos—. La lista definitiva se
  cierra en el plan; que sean **fijos** (no configurables) es decisión del dueño.
- **Tamaño de archivo por defecto**: del orden de **1 GiB** (como hoy). El tamaño, el modo y la cola
  **no** son configurables por el usuario en esta entrega (decisión del dueño); un modo avanzado
  configurable queda fuera de alcance.
- **Acotado de cada perfil** (FR-017): por tiempo con tope de datos. Referencia de partida a
  afinar en el plan: ~5 s de duración objetivo por perfil y un tope del orden de unos pocos GiB por
  perfil de escritura, de modo que la matriz completa escriba a lo sumo unas dos decenas de GB. El
  aviso previo (FR-010/SC-007) muestra el máximo real según esos valores.
- **Guardia de espacio durante la ejecución**: el archivo se reserva entero al principio, así que un
  agotamiento de espacio a mitad de prueba no es un caso a manejar (igual que hoy, J.29).
- **Volúmenes extraíbles y de red**: se permiten; no reciben trato especial.
- **La verificación de integridad byte a byte se elimina con el visto bueno del dueño**; si en el
  futuro se quisiera «¿está el disco mintiendo sobre lo que escribe?», sería una funcionalidad
  aparte.
- **El historial de pruebas y su tabla `test_runs` ya existen** (US-020, `data-model.md`); esta
  funcionalidad amplía la forma del resumen de resultados, no crea una entidad nueva.
- **La guardia térmica lee la última muestra SMART persistida del disco** (como hoy); si el disco no
  expone temperatura, la prueba corre sin guardia térmica, nunca se rechaza por eso.

## Out of Scope

- Modo avanzado con parámetros configurables (tamaño, bloque, cola, hilos, duración, número de
  pasadas).
- Cualquier comprobación de integridad del contenido escrito.
- Soporte de arquitecturas distintas de amd64.
- Benchmark de disco físico completo (raw device); la prueba siempre opera sobre un archivo en un
  volumen con sistema de archivos.
- Programación automática o recurrente de benchmarks.
- Comparación visual entre ejecuciones dentro de la aplicación (más allá de poder abrir cada una en
  el historial).
