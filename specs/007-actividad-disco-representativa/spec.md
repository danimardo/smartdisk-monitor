# Feature Specification: Actividad de disco representativa mediante ventana continua de contadores de rendimiento

**Feature Branch**: `007-actividad-disco-representativa`

**Created**: 2026-09-09

**Status**: Draft

**Input**: User description: "Actividad de disco representativa mediante consulta PDH persistente. Hoy `activity_percent` sale de `perf_counters::leer`, que abre una consulta de contadores, toma una ventana de 1 s y la cierra, y solo se ejecuta en el trabajo METRICAS_RAPIDAS (30 s por defecto). Resultado: el usuario ve 0-1 % aunque el disco esté trabajando, porque es una instantánea de 1 s de hace hasta 30 s, no un agregado del intervalo. Opción A acordada: mantener una consulta abierta de forma permanente, muestrear cada ~2 s desde el bucle en segundo plano que ya corre cada 1 s, guardar una ventana deslizante y exponer y mostrar la MEDIA y el PICO de la actividad sobre la ventana de los últimos ~30 s. Sin dependencias ni permisos nuevos. Cambio relevante: entrada en open-questions.md y probablemente ADR."

## Contexto

La aplicación muestra, por cada disco, un porcentaje de **actividad** en el panel general y en el
detalle. Ese número sale de los contadores de rendimiento de Windows del objeto `PhysicalDisk`
(`% Idle Time`, del que se deriva `activity_percent = 100 − idle`, acotado a 0–100;
`docs/architecture.md`, `docs/data-model.md` §3). No procede de los datos SMART.

Hoy la lectura es **autónoma y efímera**: en cada ciclo del trabajo «métricas rápidas» (30 s por
defecto; `docs/open-questions.md` D.1) se abre una consulta de contadores, se toman dos muestras
separadas 1 s —el mínimo que exige una tasa—, se deriva el valor y se cierra la consulta. El
resultado es **una fotografía de ~1 segundo tomada una vez cada 30 segundos**. La cabecera del
propio módulo lo reconoce como provisional: se hizo así "porque el planificador en segundo plano
todavía no existe para mantener una consulta abierta entre ciclos". Ese planificador ya existe: es
un bucle que despierta cada 1 s (`iniciar_planificador` en `commands/mod.rs`).

Consecuencia observada por el usuario: con el disco claramente trabajando (una copia grande, una
copia de seguridad, una indexación), la actividad sigue marcando 0–1 %, porque la ventana de 1 s
cae en un hueco entre operaciones o el instante de muestreo llega 25 s después de la ráfaga. La
cifra no representa el intervalo que dice representar.

El Administrador de tareas de Windows usa **el mismo contador y la misma fórmula**; su única
diferencia es que mantiene la consulta abierta y muestrea cada ~1 s de forma continua. Esta
funcionalidad traslada esa técnica al backend: una consulta de contadores viva durante toda la
ejecución, muestreada a intervalo corto, con una **ventana deslizante** por disco de la que se
derivan **media y pico** del periodo. No hay herramienta ni comando de Windows que ofrezca un dato
más «en tiempo real» que ese contador; PowerShell (`Get-Counter`) lee exactamente el mismo, y
lanzarlo como proceso cada pocos segundos sería más caro y más frágil que la vía de acceso directo
que ya usa el proyecto.

Esta especificación describe **qué** debe pasar y **por qué**. El *cómo* (hilo dedicado o
reutilización del bucle de 1 s, forma exacta del contrato, intervalo de muestreo definitivo) se fija
en el plan, y todo valor numérico nuevo se registra en `docs/open-questions.md` con su valor
propuesto **antes** de programarse (constitución, flujo de desarrollo §1).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver que un disco está trabajando de verdad, ahora (Priority: P1)

Un administrador lanza una operación que castiga un disco: una copia de varios gigabytes, una copia
de seguridad, una reindexación. Abre el panel general para confirmar que el disco correcto es el que
está bajo carga. Quiere que la actividad de ese disco lo refleje —un valor alto y coherente con lo
que está pasando— y no un 0–1 % porque la última instantánea de 1 s cayó en un microhueco entre
operaciones.

**Why this priority**: es el motivo de la funcionalidad. Una cifra de actividad que no sube cuando
el disco trabaja no aporta información; peor, induce a error (constitución §I: un dato engañoso es
peor que ningún dato). Sin este escenario resuelto, el resto no importa.

**Independent Test**: se genera carga de disco sostenida sobre un disco durante 60–120 s y se
comprueba que la actividad mostrada en el panel sube de forma acorde en menos de una cadencia de
«métricas rápidas», y que el resultado **no depende** de si el instante de muestreo coincidió con
una ráfaga concreta (se repite varias veces y es estable).

**Acceptance Scenarios**:

1. **Dado** un disco bajo carga de lectura/escritura sostenida, **cuando** el administrador mira el
   panel general, **entonces** la actividad de ese disco muestra un valor alto coherente con la
   carga, no 0–1 %.
2. **Dado** ese mismo disco, **cuando** la carga termina, **entonces** la actividad desciende hacia
   valores bajos dentro de la ventana siguiente, sin quedarse «pegada» al valor alto anterior.
3. **Dado** un disco en reposo real (sin actividad), **cuando** el administrador mira el panel,
   **entonces** la actividad muestra un valor bajo (0–algún %), como hoy.
4. **Dado** un disco cuya carga es intermitente (ráfagas cortas separadas por calma), **cuando** el
   administrador mira el panel a lo largo de un minuto, **entonces** la cifra refleja el conjunto
   del periodo y no oscila entre 0 % y 100 % según el azar del instante de muestreo.

---

### User Story 2 - Distinguir el pico reciente del promedio (Priority: P2)

El administrador investiga por qué un disco «va lento a ratos». Quiere ver no solo cuánta actividad
media ha tenido en el último medio minuto, sino si ha llegado a saturarse en algún momento de ese
periodo, aunque ahora mismo esté tranquilo.

**Why this priority**: separar media y pico es lo que convierte la cifra en diagnóstico y no solo en
un indicador de «encendido/apagado». Es coherente con el principio §I («un máximo promediado no es un
pico»). Va después de US1 porque US1 ya entrega valor por sí sola: una media representativa de la
ventana ya resuelve la queja original.

**Independent Test**: se provoca una ráfaga corta (2–3 s) de actividad al máximo seguida de ~30 s de
calma y se comprueba que el **pico** de la ventana refleja la ráfaga durante todo el tiempo que
permanece dentro de la ventana, mientras la **media** desciende de forma progresiva conforme la
ráfaga se aleja.

**Acceptance Scenarios**:

1. **Dado** un disco con una ráfaga breve de actividad seguida de calma, **cuando** el administrador
   abre el detalle del disco, **entonces** ve la actividad media y el pico del periodo como dos
   valores distintos.
2. **Dado** que la ráfaga ya salió por completo de la ventana, **cuando** el administrador vuelve a
   mirar, **entonces** tanto la media como el pico han vuelto a valores bajos.
3. **Dado** el detalle de disco, **cuando** el administrador consulta la ayuda contextual de la
   actividad, **entonces** el texto explica que es un agregado de los últimos ~30 s (media y pico),
   no una lectura instantánea.

---

### User Story 3 - Ver la forma de la actividad reciente de un vistazo (Priority: P3)

En el detalle de disco, junto a la cifra de actividad, el administrador quiere una minigráfica
(sparkline) con las últimas muestras de la ventana, para ver la «forma» de la actividad reciente sin
abrir la gráfica histórica.

**Why this priority**: es una mejora de lectura, no un requisito para que la cifra sea correcta.
Puede entregarse después o quedar fuera del primer lote sin degradar US1 ni US2.

**Independent Test**: con una ventana de muestras conocida, la `MetricCard` de actividad del detalle
de disco dibuja una sparkline con esas muestras; con la ventana vacía o parcial, no dibuja nada
engañoso.

**Acceptance Scenarios**:

1. **Dado** un disco con ventana de actividad llena, **cuando** el administrador abre su detalle,
   **entonces** la tarjeta de actividad muestra una sparkline de las muestras recientes.
2. **Dado** un disco cuya ventana aún no tiene datos suficientes, **cuando** el administrador abre su
   detalle, **entonces** la tarjeta no dibuja una sparkline a medias que parezca un dato completo.

---

### Edge Cases

- **Arranque de la aplicación.** Durante los primeros segundos la ventana no cubre aún la cadencia
  de «métricas rápidas». La cifra se presenta como **parcial** (indicando sobre qué periodo o cuántas
  muestras se calcula) o como **no disponible**, nunca como una media «normal» ni como 0 %
  (constitución §I).
- **Fallo puntual de lectura de contadores.** Si en un instante de muestreo la lectura falla
  (contador no disponible, error del subsistema de rendimiento), la muestra **no entra en la ventana
  como 0**: se conserva la ventana previa, la fuente se marca degradada con su `AppError`, y el
  muestreo se reanuda en el siguiente instante.
- **Disco que desaparece del inventario** mientras su instancia de contador está viva: se deja de
  muestrear ese disco y se libera su ventana; el fallo de resolución de su instancia no afecta al
  muestreo del resto de discos.
- **Disco nuevo detectado**: se empieza a incluir en el muestreo continuo sin perder la
  representatividad de los discos ya vigilados. Un cambio de inventario **reinicia brevemente la
  ventana de todos los discos** (estado `parcial` durante como mucho una cadencia): es un compromiso
  aceptado (`research.md` R2) para no gestionar contadores PDH vivos uno a uno, y el estado `parcial`
  es honesto. Un cambio de inventario es infrecuente (enchufar un USB).
- **Suspensión y reanudación del equipo**, o cualquier hueco de muestreo grande: las muestras
  anteriores al hueco se **descartan** en lugar de promediar a través de él (un promedio que cruza
  cuatro horas de suspensión no significa nada).
- **Cambio de la hora del sistema**: la ventana se mide con un reloj monotónico, no con la hora de
  pared; adelantar o atrasar el reloj no altera su contenido ni su tamaño.
- **Equipo a batería**: el muestreo continuo se ralentiza según la política de `docs/open-questions.md`
  D.2 (intervalo corto × 4), sin cerrar la consulta. La actividad sigue siendo una métrica no
  crítica.
- **Recopilación pausada** (`docs/open-questions.md` D.3): el muestreo continuo se detiene también;
  al reanudar, la ventana arranca vacía y se marca parcial hasta llenarse.
- **Cadencia de «métricas rápidas» reconfigurada** (10 s–5 min): el tamaño de la ventana sigue a esa
  cadencia, de modo que la cifra siempre agrega «lo que va del último intervalo mostrado».
- **Varios discos físicos**: el coste del muestreo crece de forma lineal con el número de discos y
  debe seguir siendo imperceptible.

## Requirements *(mandatory)*

### Functional Requirements

#### Recogida continua y ventana

- **FR-001**: El sistema DEBE mantener una consulta de contadores de rendimiento abierta de forma
  continua mientras la recopilación está activa, y muestrear la actividad de cada disco físico
  monitorizado a un **intervalo corto** (≈1 s en red eléctrica, ≈4 s en batería), independiente de
  la cadencia del trabajo «métricas rápidas». El valor exacto y su comportamiento en batería se
  fijan en `docs/open-questions.md` D.4 antes de programarlos.
- **FR-002**: El sistema DEBE conservar, por disco, una **ventana deslizante** en memoria de las
  muestras de actividad recientes que cubra como mínimo la cadencia configurada de «métricas
  rápidas» (30 s por defecto; `docs/open-questions.md` D.1).
- **FR-003**: De cada ventana el sistema DEBE derivar, como mínimo, la **media** y el **máximo**
  (pico) del periodo, y exponer ambos por separado hacia la interfaz. La emisión hacia la interfaz
  se hace **en cada ciclo de «métricas rápidas»** (~30 s por defecto), por los mismos canales que
  hoy; el valor emitido es el agregado de la ventana en ese instante (Q → A). Esta funcionalidad NO
  añade un evento nuevo ni un canal de refresco más rápido (ADR-015).
- **FR-004**: Mientras la ventana no cubra la cadencia de «métricas rápidas» (arranque, reanudación
  tras pausa, tras un hueco de muestreo), el sistema DEBE señalar que el dato es **parcial** e
  indicar sobre qué periodo o cuántas muestras se calcula. NUNCA presenta una media de pocos
  segundos como si cubriera la ventana completa, ni rellena los huecos con ceros (constitución §I).
- **FR-005**: Si la lectura de contadores falla en un instante de muestreo, el sistema DEBE
  conservar la ventana previa, marcar la fuente de la métrica como degradada con su `AppError`
  (constitución §X) y reanudar en el siguiente instante. Una muestra fallida NO entra en la ventana
  como 0.
- **FR-006**: La ventana DEBE medirse con un **reloj monotónico**, no con la hora de pared: un
  ajuste de la hora del sistema no altera ni su contenido ni su tamaño.
- **FR-007**: Ante un hueco de muestreo mayor que un umbral definido (suspensión del equipo,
  bloqueo temporal del subsistema de rendimiento), el sistema DEBE descartar las muestras anteriores
  al hueco en lugar de calcular media o pico a través de él.

#### Discos que entran y salen del inventario

- **FR-008**: Cuando un disco pasa a estar monitorizado, el sistema DEBE incluirlo en el muestreo
  continuo sin perder la ventana de los discos ya vigilados.
- **FR-009**: Cuando un disco deja de estar en el inventario, el sistema DEBE dejar de muestrearlo y
  liberar su ventana; el fallo de resolución de la instancia de un disco retirado NO DEBE afectar al
  muestreo del resto.

#### Persistencia e histórico

- **FR-010**: El sistema DEBE seguir registrando la serie histórica de actividad
  (`activity_percent` en `metric_samples`) a la cadencia de «métricas rápidas» —una muestra por
  ciclo—, sin multiplicar el número de filas por día por efecto del muestreo corto. El valor
  persistido de cada ciclo es **la media de la ventana** en el momento del ciclo (Q3 → A): una sola
  serie, sin cambio de modelo de datos, y coherente con lo que se mostró en vivo. No se persiste el
  pico en el histórico en esta entrega (ampliación aditiva futura si se necesita).
- **FR-010a**: Cuando un ciclo de «métricas rápidas» llega con la ventana aún incompleta (arranque,
  reanudación tras pausa, tras un hueco), el sistema NO DEBE escribir fila en la serie histórica
  para ese ciclo (Q → A). El histórico queda con un hueco, nunca con una media parcial presentada
  como valor del ciclo (constitución §I).
- **FR-011**: La gráfica histórica de actividad DEBE seguir declarando su resolución y dibujando los
  huecos como huecos (constitución §I; `docs/open-questions.md` E.1), sin más cambio que el origen
  del valor de cada punto y los huecos de arranque de FR-010a.

#### Presentación

- **FR-012**: El panel general DEBE mostrar, por disco, una **única** cifra de actividad que
  responda a la pregunta «¿está trabajando este disco?». Esa cifra es el **pico de la ventana**
  (Q1 → A): un disco que tuvo una ráfaga breve se ve como activo mientras esa ráfaga permanece en la
  ventana, que es la señal más directa para la pregunta que el panel responde. La media queda para
  el detalle (FR-013).
- **FR-012a**: El campo `activityPercent` (número suelto) de `DiskSummary` y `DeviceDetail` DEBE
  sustituirse por un valor **estructurado** que transporte la media de la ventana, el pico de la
  ventana, el estado del dato (válido / parcial / no disponible), y cuántas muestras / qué periodo
  respaldan la ventana. No se conserva el número instantáneo actual. Todos los consumidores y sus
  esquemas de validación se adaptan en el mismo lote; la puerta de tipos del contrato y las pruebas
  de rechazo Zod (constitución §XI) lo verifican. La forma exacta se fija en el plan, con
  actualización de `docs/ui-contract.md` §3.2.
- **FR-013**: El detalle de disco DEBE mostrar la actividad con su **media** y su **pico** de la
  ventana. La **procedencia** es siempre «contadores de rendimiento» para esta métrica: se rotula
  fija en la interfaz (no la decide un cálculo). La **antigüedad** la cubre el propio estado del
  dato: el agregado se recalcula en cada emisión desde la ventana viva, así que es **actual**
  (`válido`/`parcial`) o **`no disponible`** (fuente degradada o ventana vaciada por un hueco); no
  hay un estado intermedio «válido pero obsoleto» como en SMART (`research.md` R9). La ayuda
  contextual DEBE explicar que es un agregado de los últimos ~30 s (media y pico sobre N muestras),
  no una instantánea.
- **FR-014**: Toda cifra de actividad mostrada DEBE poder distinguirse como **dato válido**, **dato
  parcial** (ventana incompleta) o **no disponible** (fuente degradada), con texto o icono además
  del valor, nunca solo por color (constitución §VI).
- **FR-015** *(opcional, US3)*: El detalle de disco PUEDE mostrar una sparkline de las últimas
  muestras de la ventana, y NO la dibuja cuando la ventana está vacía o es parcial.

#### Alcance de los demás contadores de rendimiento

- **FR-016**: El caudal de lectura/escritura (bytes/s) y las latencias medias **NO** entran en la
  ventana continua en esta entrega (Q2 → A): siguen leyéndose una vez por ciclo de «métricas
  rápidas» como hoy. Son tasas ya medidas sobre su propia ventana de muestreo, menos engañosas que
  la actividad instantánea; ampliarlos a la ventana continua es una mejora aditiva posterior. La
  consulta persistente PUEDE, por implementación, seguir leyéndolos en el mismo momento que hoy sin
  que eso implique ventana ni agregados nuevos para ellos.

#### Batería y pausa

- **FR-017**: Con el equipo a batería, el muestreo continuo DEBE ralentizarse de forma coherente con
  `docs/open-questions.md` D.2 (el intervalo corto se multiplica por 4 → ≈4 s), **sin cerrar** la
  consulta. SMART completo y eventos de Windows no se ven afectados por esta funcionalidad.
- **FR-018**: Al pausar la recopilación (`docs/open-questions.md` D.3), el muestreo continuo DEBE
  detenerse; al reanudar, la ventana arranca vacía y se marca parcial hasta llenarse.

#### Sin ampliación de superficie ni de ruido

- **FR-019**: La funcionalidad NO DEBE añadir dependencias nuevas ni permisos de Tauri nuevos: se
  implementa con la biblioteca estándar y el acceso a contadores de rendimiento que ya existe
  (constitución §III; `AGENTS.md`, límites duros).
- **FR-020**: El coste de CPU añadido DEBE ser imperceptible en uso normal, y el registro de
  actividad NO DEBE ganar una entrada por cada muestra corta (constitución §XV: lo que se repite
  cada ciclo es `debug` o `trace`, no `info`).

#### Cadencia de emisión hacia la interfaz

- **FR-022**: El valor de actividad (pico para el panel, media y pico para el detalle) DEBE llegar a
  la interfaz por el mismo evento y con la misma cadencia que el resto de métricas rápidas hoy
  (~30 s, configurable). Entre dos emisiones la interfaz muestra el último valor recibido sin
  refrescos intermedios. Si entre dos emisiones dejaran de llegar ciclos (recopilación colgada,
  fuente caída), la ausencia de dato fresco se refleja por la marca de obsolescencia ya vigente del
  panel (`Toolbar` / frescura), igual que el resto de métricas; el propio `estado` del agregado pasa
  a `no disponible` si el problema es de la fuente de actividad (FR-005, FR-013).

#### Pruebas

- **FR-021**: DEBE haber pruebas de dominio, sin hardware, que cubran: el cálculo de media y pico
  sobre una ventana de muestras fixture; el marcado de ventana parcial mientras no cubre la cadencia;
  el descarte de muestras a través de un hueco (FR-007); que una muestra fallida no se incorpora
  como 0 (FR-005); y que un ciclo con ventana parcial no produce fila en la serie histórica
  (FR-010a). Estas pruebas se escriben antes que el código de esa lógica (constitución §VIII: la
  agregación es un área donde el error es silencioso).

### Key Entities *(include if feature involves data)*

- **Muestra de actividad instantánea**: un valor 0–100 con su marca de tiempo monotónica. Vive en
  memoria del backend; no se persiste individualmente.
- **Ventana deslizante de actividad** (por disco): la colección de muestras recientes que cubre al
  menos la cadencia de «métricas rápidas», de la que se derivan la media y el pico del periodo. Es
  **estado en memoria del proceso**: se pierde al reiniciar y arranca vacía (marcada parcial).
- **Agregado de actividad de la ventana**: media y pico del periodo, el estado del dato (válido /
  parcial / no disponible), y cuántas muestras / qué periodo lo respaldan. Es lo que cruza hacia la
  interfaz, y **sustituye** al campo `activityPercent` (número suelto) en `DiskSummary` y
  `DeviceDetail` (FR-012a). No lleva marca de tiempo propia: siempre es «actual» por construcción
  (FR-013, `research.md` R9).
- **Muestra histórica de actividad** (`activity_percent` en `metric_samples`, ya existe): una fila
  por ciclo de «métricas rápidas». Esta funcionalidad cambia **de qué se deriva** su valor (ver Q3),
  no su cadencia.
- **Lectura de rendimiento** (`LecturaRendimiento`, ya existe): sigue siendo la unidad para caudal y
  latencias, que se recogen una vez por ciclo de «métricas rápidas» y no tienen ventana ni agregados
  nuevos en esta entrega (Q2 → A).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Con carga de disco sostenida durante 60 s sobre un disco, la actividad mostrada en el
  panel general refleja esa carga (valor claramente alto) en menos de una cadencia de «métricas
  rápidas», en al menos 9 de cada 10 repeticiones —frente al comportamiento actual, donde una
  instantánea de 1 s puede caer en un hueco y marcar 0–1 %.
- **SC-002**: Una ráfaga de ~3 s de actividad máxima seguida de ~30 s de calma queda reflejada en el
  **pico** de la ventana durante todo el tiempo que la ráfaga permanece dentro de la ventana,
  mientras la **media** desciende de forma progresiva.
- **SC-003**: Tras reiniciar la aplicación, ninguna cifra de actividad aparece como un número normal
  antes de que su ventana tenga datos suficientes: se muestra como parcial o como no disponible.
- **SC-004**: Un fallo puntual de lectura de contadores no produce ninguna caída a 0 % ni en la
  cifra mostrada ni en la serie histórica.
- **SC-005**: El uso de CPU de la aplicación con los discos en reposo no aumenta de forma medible
  respecto a la versión anterior.
- **SC-006**: La serie histórica de `activity_percent` no crece en número de filas por día respecto
  a la versión anterior.
- **SC-007**: No hay entradas nuevas en `Cargo.toml` ni permisos nuevos en la configuración de
  Tauri respecto a la versión anterior.
- **SC-008**: Las pruebas de FR-021 existen y pasan; la cobertura de `src-tauri/src/domain/` (o del
  módulo donde viva la lógica de ventana) se mantiene por encima del mínimo de la constitución §VIII.

## Clarifications

### Session 2026-09-09

- **Q1 — ¿Qué cifra de actividad muestra el panel general (una sola por disco)?**
  → **A: el pico de la ventana.** Es la señal más directa para «¿ha estado ocupado este disco hace
  poco?»; un disco con una ráfaga breve se ve activo mientras la ráfaga sigue en la ventana. La
  media se muestra en el detalle. Recogido en FR-012 y FR-013.

- **Q2 — ¿El caudal (bytes/s) y las latencias entran también en la ventana continua?**
  → **A: no, solo la actividad.** Caudal y latencias siguen leyéndose una vez por ciclo de «métricas
  rápidas»: ya son tasas medidas sobre su propia ventana, menos engañosas que la actividad
  instantánea. Ampliarlos es una mejora aditiva futura. Recogido en FR-016 y en Key Entities.

- **Q3 — ¿Qué valor de la ventana se persiste en la serie histórica `activity_percent`?**
  → **A: la media de la ventana.** Una sola serie, sin cambio de modelo de datos ni de retención, y
  coherente con lo que se mostró en vivo. El pico no se persiste en el histórico en esta entrega.
  Recogido en FR-010 y FR-011.

- **Q — ¿Con qué frecuencia llega a la interfaz el valor de actividad actualizado?**
  → **A: en cada ciclo de «métricas rápidas» (~30 s), como hoy.** El backend sigue emitiendo el
  valor una vez por ciclo; lo que cambia es que el valor emitido es el agregado de la ventana (pico
  para el panel, media y pico para el detalle) en ese instante, no una instantánea de 1 s. No se
  añade ningún evento nuevo ni se toca el contrato de eventos (ADR-015: la interfaz no sondea).
  El «modo en vivo» queda descartado en esta entrega. Recogido en FR-003 y FR-022.

- **Q — ¿Cómo cambia el contrato de `DiskSummary` / `DeviceDetail` para llevar media y pico?**
  → **A: se sustituye `activityPercent` por un valor estructurado** con media, pico y estado
  (válido / parcial / no disponible). No se conserva el número suelto actual (era la instantánea que
  se está eliminando). Todos los consumidores (panel, detalle, esquemas Zod) se adaptan en el mismo
  lote; la puerta de tipos del contrato (`ts-rs`) y las pruebas de rechazo Zod garantizan que no
  queda ninguno sin adaptar. La forma exacta del tipo se fija en el plan, con actualización de
  `docs/ui-contract.md` §3.2 y ADR si el cambio de modelo lo amerita. Recogido en FR-012a y en Key
  Entities.

- **Q — ¿Qué se persiste en el histórico cuando el ciclo llega con la ventana incompleta?**
  → **A: no se escribe fila para ese ciclo.** La serie histórica queda con un hueco, que la gráfica
  dibuja como hueco sin interpolar (constitución §I; `docs/open-questions.md` E.1). Un arranque deja
  un hueco de como mucho una cadencia, irrelevante para el análisis de tendencias, y no se inventa
  un valor de ciclo a partir de pocos segundos. Recogido en FR-010a.

## Assumptions

- **El bucle en segundo plano ya existe** (`iniciar_planificador`, `commands/mod.rs`) y despierta
  cada 1 s. Es el punto natural para disparar el muestreo corto; que se haga desde ese bucle o desde
  un hilo dedicado se decide en el plan.
- **La ventana vive solo en memoria del backend y no se persiste.** Al reiniciar arranca vacía y se
  marca parcial. Es coherente con §I (se declara parcial) y con §V (SQLite es para lo que debe
  sobrevivir; una ventana de 30 s no lo es).
- **El intervalo de muestreo corto es ≈1 s** (≈4 s en batería), un muestreo por tick del bucle en
  segundo plano que ya despierta cada 1 s. El valor exacto y el umbral de «hueco» de FR-007 se fijan
  en `docs/open-questions.md` D.4 con su valor propuesto antes de programarse (constitución, flujo
  de desarrollo §1; `research.md` R3).
- **La cadencia de «métricas rápidas»** (30 s por defecto, configurable 10 s–5 min;
  `docs/open-questions.md` D.1) sigue definiendo el tamaño de la ventana y la cadencia de
  persistencia de la serie histórica.
- **En batería se aplica el multiplicador × 4 de D.2 también al muestreo corto.** SMART completo y
  eventos de Windows no se tocan.
- **El tamaño de la ventana igual a la cadencia de «métricas rápidas»** implica que reconfigurar esa
  cadencia (10 s–5 min) redimensiona la ventana; con 5 min, media y pico agregan cinco minutos.
- **La distinción válido / parcial / no disponible y los valores media/pico sustituyen a
  `activityPercent`** en el contrato de `DiskSummary` / `DeviceDetail` (Q → A, FR-012a). La forma
  exacta del tipo estructurado se decide en el plan; el contrato generado por `ts-rs` se regenera,
  se actualiza `docs/ui-contract.md` §3.2 y, si el cambio de modelo lo amerita, lleva su ADR
  (`docs/decisions.md`).
- **Es un cambio relevante.** Requiere, como mínimo: entrada(s) en `docs/open-questions.md`
  (sección D o una nueva) con los valores adoptados; actualización de `docs/architecture.md`
  (colector de contadores de rendimiento) y de `docs/data-model.md` (definición de
  `activity_percent`); y muy probablemente un ADR en `docs/decisions.md` por el cambio de
  arquitectura de recogida (de consulta efímera a consulta persistente). El consolidado
  `historias.md` se regenera con `pnpm docs:build`.
- **Idioma**: toda la documentación generada, los textos de interfaz nuevos (en los dos
  diccionarios) y los comentarios de código van en español.

## Dependencies

- `docs/open-questions.md`: D.1 (límites de las frecuencias configurables), D.2 (comportamiento en
  batería), D.3 (qué hace «Pausar»), E.1 (tabla intervalo → resolución de gráficas).
- `docs/architecture.md`: descripción del colector de contadores `PhysicalDisk`.
- `docs/data-model.md` §2–§3: `activity_percent`, `metric_samples`, retención y agregación.
- `docs/decisions.md`: entrada sobre `refresh_metricas_rendimiento` y la espera de 1 s entre las dos
  muestras de una tasa de contador.
- `docs/ui-contract.md` §3.2: `DeviceDetail`; contrato de `DiskSummary`.
- `docs/ui-design.md`: `MetricCard`, tarjeta de disco, ayuda contextual de métricas; definición de
  terminado §8.
- Código existente: `src-tauri/src/collectors/perf_counters.rs`,
  `src-tauri/src/collectors/planificador.rs`, `iniciar_planificador` / `ejecutar_ciclo` /
  `refresh_metricas_rendimiento` en `src-tauri/src/commands/mod.rs`,
  `src/lib/components/DiskCard.svelte`, `src/routes/+page.svelte`,
  `src/routes/disks/[id]/+page.svelte`, `src/lib/design/metricHelp.ts`,
  `src/lib/api/generated/DiskSummary.ts` y `DeviceDetail.ts`.
