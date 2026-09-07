# Feature Specification: Puente del registro de eventos de Windows al motor de alertas

**Feature Branch**: `003-puente-eventos-alertas`

**Created**: 2026-09-07

**Status**: Draft

**Input**: User description: "Puente del registro de eventos de Windows al motor de alertas (Historia 4 de specs/001-monitor-discos-windows). Los eventos de Windows ya se leen (event_log.rs, T067), se correlacionan con discos (domain/correlacion.rs, T069) y se muestran en el panel «Sucesos del sistema», pero NO alimentan el motor de alertas. Falta implementar: las 18 reglas events.* de docs/alert-rules.md §2, la regla device.removed_unexpected, la regla inventory.duplicate_id, y la ventana de correlación de ráfaga de 60 s. Cada regla con su activación, severidad, resolución (histéresis temporal «N h/días sin repetición»), cooldown de notificación y contexto de deduplicación según la tabla normativa. Cinco pruebas por regla según alert-rules.md §5."

## Contexto

La aplicación ya **lee** el registro de eventos de Windows relativos al almacenamiento, los
**correlaciona** con los discos del inventario (con un grado de certeza declarado: exacta, inferida o
desconocida) y los **muestra** en la pantalla «Sucesos del sistema». Lo que falta es que esos
eventos **produzcan alertas**: hoy un administrador puede ver en la lista de sucesos que Windows
registró un daño de sistema de archivos, pero el panel general seguirá en verde y no habrá ninguna
notificación.

El motor de alertas actual (`docs/alert-rules.md` §2) cubre las reglas que se evalúan sobre datos de
`smartctl` y de capacidad ya persistidos. Las reglas cuya fuente es «registro de eventos» —más
`device.removed_unexpected` e `inventory.duplicate_id`— quedaron explícitamente fuera de esa primera
entrega (`docs/open-questions.md` J.16) porque necesitaban el colector de eventos, que ya existe.
Esta funcionalidad cierra ese hueco.

`docs/alert-rules.md` es la **fuente normativa**: la tabla §2 (activación, severidad, resolución,
cooldown, contexto de deduplicación por regla), la lista de proveedores y eventos vigilados §3.2, lo
que explícitamente no genera alerta §3.3, la ventana de correlación de ráfagas §3.5 y las cinco
pruebas exigidas por regla §5. Esta especificación describe **qué** debe pasar y **por qué**; no
reproduce la tabla, se apoya en ella.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Saber que un disco se ha desconectado sin avisar (Priority: P1)

Un administrador tiene un disco externo conectado a un equipo que monitoriza. El disco se
desconecta —un cable suelto, un fallo de alimentación, alguien que lo retira sin expulsarlo—. Windows
registra la extracción imprevista y, en la misma ráfaga, varios errores derivados (paginación,
escritura demorada, registro de transacción). El administrador quiere **una** alerta que diga «este
disco se ha desconectado de forma imprevista», no cuatro alertas distintas sobre síntomas del mismo
suceso.

**Why this priority**: es el escenario que motivó el rediseño del motor
(`docs/open-questions.md` P.5). Una desconexión imprevista es lo más parecido a «se ha roto algo» que
un administrador necesita ver de inmediato, y es también el caso donde el ruido correlacionado haría
más daño: sin agrupar, un solo cable suelto llenaría la pantalla de alertas.

**Independent Test**: se inyecta una ráfaga de eventos fixture de un mismo disco dentro de 60
segundos que incluye una extracción imprevista (`disk` 157) y se comprueba que se crea **un solo**
grupo de alerta `device.removed_unexpected`, que los demás eventos de la ráfaga figuran en su
cronología como ocurrencias suyas y no como grupos propios, y que la alerta se resuelve cuando el
mismo disco vuelve a aparecer en el inventario.

**Acceptance Scenarios**:

1. **Dado** un disco fijo monitorizado, **cuando** desaparece del inventario, **entonces** se crea
   una alerta crítica `device.removed_unexpected` para ese disco (un disco fijo no desaparece en
   operación normal).
2. **Dado** un disco monitorizado cuyo bus es USB, **cuando** llega un evento `disk` 157 para él,
   **entonces** se crea una alerta de nivel advertencia, no crítica (un disco USB que se quita es
   más esperable).
3. **Dado** que en 60 segundos llegan del mismo disco una extracción imprevista y tres errores
   derivados, **cuando** el motor los procesa, **entonces** existe un único grupo
   `device.removed_unexpected` y su cronología lista los cuatro eventos con su hora.
4. **Dado** un grupo `device.removed_unexpected` activo, **cuando** reaparece en el inventario un
   disco con la misma identidad estable, **entonces** el grupo pasa a resuelto.
5. **Dado** un disco USB que desaparece del inventario **sin** un evento `disk` 157 asociado,
   **cuando** el motor lo evalúa, **entonces** no se crea ninguna alerta (se trata como retirada
   esperada, research.md D2).

---

### User Story 2 - Enterarme de un error de disco o de sistema de archivos que registró Windows (Priority: P1)

Cuando Windows registra un error grave de almacenamiento —un bloque defectuoso, un daño en la
estructura NTFS que no se puede reparar solo, un comando NVMe completado con error, un disco físico
de Storage Spaces con metadatos inválidos— el administrador quiere una alerta crítica que lo diga en
lenguaje llano y que enlace al evento exacto para poder adjuntarlo a un parte de incidencia.

**Why this priority**: es el núcleo del puente. Estos eventos son evidencia directa de un problema
real que las métricas de `smartctl` no siempre capturan, y son de los que un administrador tiene que
ver el mismo día.

**Independent Test**: se inyecta un evento fixture de cada familia crítica (`Ntfs` 55/131, `disk` 7,
`NvmeDisk` 500, `StorageSpaces-Driver` 202/203/209 y 300–311, `Ntfs` 50 sobre volumen no extraíble) y
se comprueba que cada uno crea su grupo de alerta con la severidad de la tabla §2, con el título y
resumen traducidos (nunca la clave técnica), y que el detalle del grupo enlaza al evento con su grado
de certeza de atribución.

**Acceptance Scenarios**:

1. **Dado** un evento `Ntfs` 131 (estructura irreparable) atribuido a un volumen, **cuando** el
   motor lo procesa, **entonces** se crea una alerta crítica `events.filesystem_error` para ese
   volumen.
2. **Dado** un evento `Microsoft-Windows-NvmeDisk` 500 atribuido a un disco, **cuando** el motor lo
   procesa, **entonces** se crea una alerta crítica `events.disk_error`.
3. **Dado** cualquier alerta creada por una regla de eventos, **cuando** se muestra en la lista,
   **entonces** aparece su titular en lenguaje llano y **nunca** la `rule_key` ni el
   `provider:event_id`.
4. **Dado** el detalle de una alerta de evento, **cuando** el administrador lo abre, **entonces**
   puede ver el o los eventos del sistema que la dispararon, con su marca de tiempo, su texto
   original y la etiqueta de atribución (exacta o inferida).
5. **Dado** un evento `Microsoft-Windows-Ntfs` 98 (nivel Información: «el volumen está bien»),
   **cuando** el motor lo procesa, **entonces** no se crea ninguna alerta.
6. **Dado** un evento crítico ya alertado, **cuando** no vuelve a repetirse durante 24 horas,
   **entonces** el grupo pasa a resuelto; **cuando** vuelve a repetirse después, **entonces** el
   grupo se reactiva e incrementa su número de episodio conservando el contador histórico.

---

### User Story 3 - No recibir alertas por el ruido normal de Windows (Priority: P2)

El registro de Windows de un equipo perfectamente sano contiene cientos de eventos de almacenamiento
benignos: errores de paginación que el sistema resuelve solo, reintentos de E/S puntuales,
restablecimientos aislados de controladora. El administrador quiere que la aplicación **solo** avise
cuando esos eventos se acumulan por encima de un umbral de frecuencia sobre un mismo disco fijo, y
nunca por un evento aislado.

**Why this priority**: sin esto el producto es inservible. En la observación real, un solo evento
(`disk` 51) apareció 839 veces en 180 días en un equipo sin ningún problema
(`docs/open-questions.md` P.2). Tratarlo como alerta directa produciría 839 falsos críticos.

**Independent Test**: se inyecta una serie de eventos fixture del mismo tipo y disco por debajo del
umbral de frecuencia de la tabla §2 y se comprueba que no se crea ninguna alerta; al superar el
umbral dentro de la ventana, se crea exactamente un grupo.

**Acceptance Scenarios**:

1. **Dado** un disco no extraíble con 9 eventos `disk` 51 (paginación) en la última hora, **cuando**
   el motor los evalúa, **entonces** no hay ninguna alerta.
2. **Dado** ese mismo disco, **cuando** llega el décimo evento `disk` 51 dentro de la hora,
   **entonces** se crea una alerta de advertencia `events.paging_error`, y nunca crítica.
3. **Dado** un evento `disk` 51 sobre un medio extraíble, **cuando** el motor lo evalúa, **entonces**
   no cuenta para el umbral ni crea alerta.
4. **Dado** un disco fijo con 3 restablecimientos de controladora (`disk` 11 / `storahci` 129) en
   una hora, **cuando** el motor los evalúa, **entonces** la alerta `events.controller_reset` sube
   de advertencia a crítica.
5. **Dado** un umbral de frecuencia superado y luego 24 horas sin nuevos eventos de ese tipo,
   **cuando** el motor lo evalúa, **entonces** el grupo pasa a resuelto.

---

### User Story 4 - Advertencias leves: reparaciones automáticas y predicción de fallo (Priority: P2)

Windows a veces repara solo la estructura de un sistema de archivos, o avisa de que un disco podría
fallar pronto. No es una emergencia, pero el administrador quiere que quede registrado como una
advertencia con un cooldown de notificación largo, para no recibir el mismo aviso a diario.

**Why this priority**: aporta señal útil de mantenimiento preventivo sin competir por la atención con
los eventos críticos.

**Independent Test**: se inyectan eventos fixture `Ntfs` 130, `Ntfs` 132 y `disk` 52 y se comprueba
que crean grupos con la severidad y el cooldown de la tabla §2, y que su histéresis de resolución es
la ventana larga («7 días sin repetición»).

**Acceptance Scenarios**:

1. **Dado** un evento `Ntfs` 130 (la estructura se reparó sola) sobre un volumen, **cuando** el
   motor lo procesa, **entonces** se crea una advertencia `events.filesystem_repaired` con cooldown
   de notificación de 24 horas.
2. **Dado** un evento `Ntfs` 132 (demasiadas reparaciones seguidas), **cuando** el motor lo procesa,
   **entonces** se crea una alerta crítica `events.filesystem_repair_storm`.
3. **Dado** un evento `disk` 52 (fallo predictivo), **cuando** el motor lo procesa, **entonces** se
   crea una advertencia `events.disk_predictive`.
4. **Dado** cualquiera de estas advertencias, **cuando** no se repite el evento durante 7 días,
   **entonces** el grupo pasa a resuelto.

---

### User Story 5 - Detectar que dos discos comparten identidad (Priority: P3)

Windows puede registrar que dos discos presentan los mismos identificadores (evento `disk` 158). Esto
compromete la fiabilidad de todo lo que la aplicación atribuye a esos discos. El administrador quiere
un aviso, una sola vez por pareja de discos.

**Why this priority**: es un problema de integridad de datos poco frecuente pero real (observado 63
veces, `docs/open-questions.md` P.4). Va el último porque su impacto sobre el usuario medio es menor
que el de un error de disco.

**Independent Test**: se inyecta un evento fixture `disk` 158 y se comprueba que crea un grupo
`inventory.duplicate_id` cuyo contexto de deduplicación es la pareja de discos, y que un segundo
evento equivalente no crea un grupo nuevo.

**Acceptance Scenarios**:

1. **Dado** un evento `disk` 158 que señala dos discos con identificadores compartidos, **cuando** el
   motor lo procesa, **entonces** se crea una advertencia `inventory.duplicate_id` para esa pareja.
2. **Dado** un grupo `inventory.duplicate_id` ya activo para una pareja, **cuando** llega otro evento
   `disk` 158 de la misma pareja, **entonces** se registra como ocurrencia del grupo existente, no
   como grupo nuevo.
3. **Dado** un grupo `inventory.duplicate_id`, **cuando** el aviso deja de repetirse durante 7 días,
   **entonces** el grupo pasa a resuelto.

---

### Edge Cases

- **Evento sin disco atribuible.** Si la atribución de un evento a un disco es `desconocida`
  (identificadores no resolubles contra el inventario), la alerta se crea **igualmente, sin objeto
  asociado**, y se deduplica por `provider:event_id`; su detalle indica que Windows registró el
  problema pero no se pudo determinar el disco. El panel general no colorea ningún disco por esa
  alerta (Q2 → A).
- **Ráfaga sin causa clara.** Cuando varios eventos correlacionables del mismo disco ocurren en 60
  segundos sin ninguna extracción imprevista (`disk` 157) entre ellos, **cada regla dispara su
  propio grupo**: no se colapsa nada. Un `Ntfs` 55 y un `disk` 7 simultáneos crean dos grupos, uno
  por regla, porque son problemas de naturaleza distinta (Q3 → A). El colapso de la ráfaga en un
  solo suceso solo se aplica cuando hay un `disk` 157 que actúe de causa (FR-009).
- **Reactivación tras archivar.** Un grupo que el administrador archivó vuelve a cumplirse: se
  reactiva e incrementa el episodio, igual que desde resuelto (`docs/alert-rules.md` §1).
- **Evento antiguo re-leído.** Al reabrir la aplicación se releen eventos; el cursor persistente
  (`docs/open-questions.md` J.7) evita duplicar eventos ya ingeridos, y por tanto evita re-disparar
  alertas por ellos.
- **Reloj del sistema movido hacia atrás.** La identidad de un evento es `(canal, RecordId)`, no su
  fecha; un cambio de hora no produce ni eventos ni alertas duplicadas.
- **Evento cuyo nivel es Información pero cuyo id está en la lista de una regla.** Prevalece la regla
  de §3.3: `Microsoft-Windows-Ntfs` 98, `NvmeDisk` 501 y los de `Volsnap`/`volmgr` nunca generan
  alerta aunque compartan proveedor con un evento vigilado.
- **La lectura del registro de eventos falla en un ciclo.** El fallo del colector de eventos no debe
  impedir que el resto del ciclo (SMART, capacidad, inventario) produzca sus alertas, ni al revés
  (misma tolerancia a fallos por fuente que ya aplica al resto, SC-008 de la spec 001).
- **Storage Spaces / RAID por hardware.** Los eventos de estos proveedores están tomados de los
  manifiestos, no observados en hardware real (`docs/alert-rules.md` §3.8). Las reglas se
  implementan con fixtures del manifiesto; el contraste en hardware real queda como tarea de Fase 0,
  no bloquea esta entrega.
- **Primera activación del puente sobre un registro histórico.** Al habilitarse por primera vez,
  puede haber meses de eventos ya ingeridos en la base; ninguno de ellos genera alerta (FR-018): el
  puente solo evalúa lo que llega desde su primer arranque.

## Requirements *(mandatory)*

### Functional Requirements

#### Evaluación de reglas de eventos

- **FR-001**: El sistema DEBE evaluar cada evento de almacenamiento ingerido contra las reglas cuya
  fuente es «registro de eventos» en `docs/alert-rules.md` §2 (`events.disk_error`,
  `events.filesystem_error`, `events.filesystem_repaired`, `events.filesystem_repair_storm`,
  `events.controller_reset`, `events.paging_error`, `events.io_retry`, `events.delayed_write`,
  `events.disk_predictive`, `events.storage_space_degraded`) usando la correspondencia
  proveedor/id → regla de §3.2.
- **FR-002**: El sistema DEBE evaluar la regla `device.removed_unexpected`. Se activa (a) al llegar
  un evento `disk` 157 correlacionado con un disco del inventario, o (b) cuando un disco **no USB**
  monitorizado desaparece del inventario —un disco fijo no desaparece en operación normal y Windows
  no deja un rastro fiable de «expulsión limpia» (research.md D2)—. Un disco USB que desaparece sin
  `disk` 157 **no** activa la regla. Severidad crítica, salvo bus USB → advertencia. Resolución:
  reaparece en el inventario el mismo `fingerprint`.
- **FR-003**: El sistema DEBE evaluar la regla `inventory.duplicate_id` a partir del evento
  `disk` 158, creando como mucho un grupo por pareja de discos afectada.
- **FR-004**: Para cada regla, la activación, la severidad (incluida la escalada a crítico por
  umbral de frecuencia donde la tabla lo indica), la condición de resolución, el cooldown de
  notificación y el contexto de deduplicación DEBEN ser exactamente los de la fila correspondiente
  de `docs/alert-rules.md` §2. Si la implementación necesita un valor que la tabla no fija, se
  registra en `docs/open-questions.md` con su valor propuesto **antes** de programarlo.
- **FR-004a**: Cuando la atribución de un evento a un disco o volumen es `desconocida`, la regla
  DEBE crear la alerta igualmente, sin objeto asociado, deduplicando por `provider:event_id`; el
  detalle DEBE dejar claro que no se pudo determinar el disco. Una alerta sin objeto NO contribuye
  al color de ningún disco en el panel general.
- **FR-005**: El sistema NO DEBE crear ninguna alerta a partir de los eventos listados en
  `docs/alert-rules.md` §3.3 (informativos y de VSS/volcado de memoria), aunque compartan proveedor
  con un evento vigilado.
- **FR-006**: El sistema NO DEBE crear ninguna alerta por ausencia de dato: un evento que nunca ha
  llegado no genera nada (constitución §I; `alert-rules.md` §5, prueba 2).
- **FR-007**: Las reglas con umbral de frecuencia (`events.paging_error` ≥ 10/h,
  `events.io_retry` ≥ 5/h, escalada de `events.controller_reset` ≥ 3/h) DEBEN contar solo eventos
  del mismo tipo sobre el mismo disco, y `events.paging_error` DEBE contar solo discos no extraíbles.

#### Ventana de correlación de ráfagas

- **FR-008**: El sistema DEBE tratar como un único suceso los eventos del mismo disco que ocurren
  dentro de una ventana de 60 segundos (`docs/alert-rules.md` §3.5).
- **FR-009**: Cuando en una ráfaga correlacionada hay una extracción imprevista (`disk` 157), esa
  DEBE ser la causa: los demás eventos de la ráfaga se registran como ocurrencias del grupo
  `device.removed_unexpected` y **no** crean grupos propios.
- **FR-009a**: Cuando una ráfaga correlacionada NO contiene una extracción imprevista (`disk` 157),
  NO se colapsa: cada evento se evalúa contra su regla y crea o alimenta su propio grupo con
  normalidad.
- **FR-010**: La cronología de un grupo creado a partir de una ráfaga DEBE conservar todos los
  eventos de la ráfaga con su marca de tiempo individual, para poder reconstruir la secuencia.

#### Ciclo de vida e histéresis temporal

- **FR-011**: Las reglas de eventos DEBEN resolverse por **tiempo transcurrido sin repetición** (24
  horas o 7 días según la tabla §2), no por número de ciclos de recopilación. El sistema DEBE
  persistir la información necesaria para saber cuándo se repitió por última vez un evento de cada
  grupo.
- **FR-012**: Un grupo de alerta de evento resuelto o archivado que vuelve a cumplirse DEBE
  reactivarse e incrementar su número de episodio conservando el contador histórico de ocurrencias
  (`docs/alert-rules.md` §1).
- **FR-013**: El cooldown de notificación DEBE suprimir la notificación al usuario, nunca la
  ocurrencia en la cronología ni la presencia del grupo en la lista ni su contribución al color del
  panel.

#### Integración con lo que ya existe

- **FR-014**: La evaluación de reglas de eventos DEBE ejecutarse como parte del ciclo de
  recopilación, después de que el colector de eventos haya ingerido y correlacionado los eventos
  nuevos, y sus resultados DEBEN notificarse y emitirse hacia la interfaz por los mismos canales que
  el resto de las alertas (`alerts:changed`, notificación nativa, icono de bandeja).
- **FR-015**: Un fallo del colector de eventos en un ciclo NO DEBE impedir que el resto del ciclo
  produzca alertas, ni un fallo de otra fuente DEBE impedir la evaluación de las reglas de eventos.
- **FR-016**: El detalle de una alerta creada por una regla de eventos DEBE permitir al
  administrador llegar al evento o eventos del sistema que la dispararon, mostrando para cada uno su
  marca de tiempo, su texto original (como texto plano, nunca interpretado como contenido con
  formato) y el grado de certeza de su atribución a un disco (exacta o inferida).
- **FR-017**: Cada regla que el motor puede emitir DEBE tener su titular y su resumen en español y
  en inglés (`alert.rule.<rule_key>.title` / `.summary`), con el titular redactado según la norma de
  `docs/alert-rules.md` §4 (dice qué pasa, no qué contador se movió).

#### Alcance de la primera activación

- **FR-018**: Al habilitarse el puente, el sistema DEBE evaluar **solo los eventos ingeridos a
  partir de su primer arranque**. Los eventos ya presentes en la base con anterioridad se marcan
  como «ya vistos» y no se evalúan (Q1 → A). El sistema DEBE persistir un punto de corte que
  sobreviva a reinicios, de modo que reabrir la aplicación no vuelva a evaluar eventos anteriores a
  ese punto.

#### Pruebas

- **FR-019**: Cada regla de la tabla §2 cubierta por esta funcionalidad DEBE tener, como mínimo, las
  cinco pruebas de `docs/alert-rules.md` §5: activación con fixture real, no activación ante dato
  ausente, histéresis (oscilar el umbral no produce más de un grupo), deduplicación (N evaluaciones
  equivalentes → 1 grupo y N ocurrencias) y ciclo de recaída (resolver y recaer incrementa el
  episodio y conserva el contador). Las pruebas se escriben **antes** que el código de cada regla
  (constitución §VIII).
- **FR-020**: DEBE existir al menos una prueba de la ventana de correlación: una ráfaga con
  `disk` 157 produce un solo grupo `device.removed_unexpected` con los demás eventos como
  ocurrencias; una ráfaga equivalente sin `disk` 157 produce un grupo por regla afectada (FR-009a).

### Key Entities *(include if feature involves data)*

- **Evento del sistema** (`system_events`, ya existe): un registro del Event Log de Windows
  relativo al almacenamiento, con su canal y `RecordId` (identidad estable), proveedor, id, nivel,
  texto, disco o volumen asociado y grado de certeza de esa asociación. Es la entrada del motor.
- **Regla de evento**: la definición normativa de `docs/alert-rules.md` §2 — qué combinación de
  proveedor/id/frecuencia la activa, con qué severidad, cómo se resuelve, su cooldown y su
  discriminante de deduplicación.
- **Grupo de alerta** (`alert_groups`, ya existe): la condición sobre un objeto concreto que el
  administrador ve y gestiona. Esta funcionalidad crea grupos nuevos cuyo `rule_key` es una regla de
  eventos.
- **Ocurrencia de alerta** (`alert_occurrences`, ya existe): cada vez que la condición se vuelve a
  cumplir. Ya tiene un vínculo al evento que la disparó (`triggering_event_id`); esta funcionalidad
  lo rellena.
- **Ráfaga correlacionada**: el conjunto de eventos del mismo disco dentro de una ventana de 60
  segundos que el motor trata como un solo suceso. No es necesariamente una entidad persistida por
  separado; es un concepto de agrupación temporal.
- **Extracción imprevista vs. retirada esperada**: no es una entidad persistida. La distinción se
  infiere (research.md D2), porque Windows no deja una señal fiable de expulsión ordenada: cuenta
  como imprevisto un disco fijo que desaparece del inventario, o cualquier disco con un `disk` 157 en
  la ventana de correlación; una desaparición de disco USB sin `disk` 157 se trata como retirada
  esperada y no alerta.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Sobre el registro real de referencia (180 días de un equipo Windows 11 sano, 2.038
  eventos de almacenamiento, `docs/alert-rules.md` §3), el puente produce **cero** alertas críticas
  falsas y **cero** alertas de advertencia por eventos aislados por debajo de su umbral de
  frecuencia.
- **SC-002**: Una ráfaga de extracción imprevista de un disco (una causa `disk` 157 más tres o más
  eventos derivados en 60 segundos) produce exactamente **una** alerta, no una por evento.
- **SC-003**: Un evento crítico de almacenamiento registrado por Windows (daño de sistema de
  archivos, bloque defectuoso, error NVMe, disco de Storage Spaces degradado) aparece como alerta en
  el panel en el mismo ciclo de recopilación en que el evento se ingiere.
- **SC-004**: Desde el detalle de cualquier alerta de evento, el administrador puede abrir el evento
  o eventos del sistema que la originaron en como mucho una interacción, y ver su texto original y su
  etiqueta de certeza de atribución.
- **SC-005**: El 100 % de las reglas de la tabla §2 cubiertas por esta funcionalidad tienen las
  cinco pruebas exigidas por §5, y todas pasan.
- **SC-006**: Todas las reglas que el motor puede emitir tienen titular y resumen en los dos
  idiomas; la lista de alertas no muestra ninguna `rule_key` ni ninguna cadena técnica de
  proveedor/id.
- **SC-007**: Habilitar el puente sobre una base con meses de eventos históricos produce **cero**
  alertas por esos eventos anteriores al punto de corte (FR-018), y el equipo de referencia sano
  sigue cumpliendo SC-001.
- **SC-008**: Un grupo de alerta de evento se resuelve automáticamente cuando pasa su ventana sin
  repetición (24 h o 7 días según la regla) y se reactiva, conservando su contador histórico, si el
  evento vuelve a ocurrir después.

## Clarifications

### Session 2026-09-07

- **Q1 — ¿Qué hace el puente con los eventos ya ingeridos al activarse por primera vez?**
  → **A: solo hacia delante.** Evalúa únicamente eventos ingeridos a partir de su primer arranque;
  los históricos se marcan «ya vistos» sin evaluarlos, con un punto de corte persistente. Sin
  avalancha, y coherente con las ventanas de frecuencia y correlación, que necesitan tiempo real.
  Recogido en FR-018 y SC-007.

- **Q2 — ¿Qué pasa con un evento cuya atribución a un disco es `desconocida`?**
  → **A: alertar igualmente, sin objeto asociado.** La alerta se crea y se deduplica por
  `provider:event_id`; el detalle explica que Windows registró el problema pero no se pudo
  determinar el disco. No contribuye al color de ningún disco en el panel. Recogido en FR-004a y en
  Edge Cases.

- **Q3 — En una ráfaga de 60 s sin extracción imprevista (`disk` 157), ¿qué se hace con los eventos
  correlacionados?**
  → **A: cada regla dispara su propio grupo.** El colapso solo se aplica cuando hay un `disk` 157
  que actúe de causa. Un `Ntfs` 55 y un `disk` 7 simultáneos crean dos grupos, uno por regla.
  Recogido en FR-009a y FR-020.

## Assumptions

- **La pila de ingesta de eventos ya está y es suficiente.** El colector (`event_log.rs`), la
  correlación con confianza declarada (`domain/correlacion.rs`), la persistencia (`system_events`,
  con `UNIQUE(channel, record_id)` y cursor de bookmark) y la pantalla de sucesos ya existen y no se
  rehacen. Esta funcionalidad añade la capa de evaluación de reglas encima.
- **El esquema de datos ya soporta el vínculo alerta→evento.** `alert_occurrences.triggering_event_id`
  ya referencia `system_events`; esta funcionalidad lo empieza a rellenar. Cualquier campo nuevo que
  haga falta (p. ej. para la histéresis temporal por grupo o para «solo hacia delante») se decide en
  el plan y, si toca modelo de datos, con su ADR.
- **La lista de proveedores y eventos vigilados vive en `settings`** (`docs/alert-rules.md` §3), de
  modo que ampliarla no exige recompilar. Esta funcionalidad consume esa lista, no la sustituye.
- **La distinción «expulsión limpia» vs. «extracción imprevista»** está **resuelta en research.md
  D2**: no hay rastro fiable de expulsión ordenada, así que `device.removed_unexpected` se apoya en
  el evento `disk` 157 y en la premisa «un disco fijo no desaparece en operación normal». Un disco
  USB sin `disk` 157 no alerta.
- **Los eventos de Storage Spaces y RAID por hardware** se implementan y prueban con fixtures de los
  manifiestos de proveedor; su contraste contra hardware real es tarea de Fase 0 y no bloquea esta
  entrega (`docs/alert-rules.md` §3.8).
- **Frecuencias del recopilador de eventos**: 30 s por defecto (`EVENTOS_WINDOWS` en el
  planificador), inmune a batería. Las ventanas de «1 hora» de los umbrales de frecuencia y los «60
  s» de correlación son tiempo de reloj, no ciclos.
- **No hay cambios de permisos de Tauri**: leer el registro de eventos ya está cubierto por la
  ejecución elevada existente; esta funcionalidad no añade superficie nueva.
- **Idioma**: toda la documentación generada, los titulares y resúmenes de alerta y los comentarios
  de código van en español (y los textos, además, en inglés).

## Dependencies

- `docs/alert-rules.md` (normativo): tabla §2, proveedores §3.2, exclusiones §3.3, ventana de
  correlación §3.5, atribución §3.6, pruebas §5.
- `docs/open-questions.md`: J.7 (cursor de bookmark), J.16 (estas reglas quedaron fuera del primer
  motor), J.24 (`dedup_hash`), J.25 (confianza de correlación), J.37 (`SourceHealth`), P.1–P.6
  (hallazgos de la observación real del registro).
- Infraestructura existente de la spec 001 Historia 4: T067 (colector), T068–T069 (correlación),
  T070 (comandos y pantalla de eventos), y del motor de alertas: `alert_groups`,
  `alert_occurrences`, agrupación, notificaciones.
- `docs/data-model.md` §2 (`system_events`, `alert_occurrences`).
