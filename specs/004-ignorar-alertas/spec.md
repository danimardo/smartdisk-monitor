# Feature Specification: Ignorar una alerta de forma permanente

**Feature Branch**: `004-ignorar-alertas`

**Created**: 2026-09-08

**Status**: Draft

**Input**: User description: "Botón «No volver a registrar este evento» en el informe de alertas: nuevo estado `ignored` para un grupo de alerta concreto (por clave de deduplicación: regla + disco + contexto). Al ignorar, la app sigue anotando ocurrencias en la cronología en segundo plano, pero la alerta nunca vuelve a notificar, nunca cuenta para el color del disco y nunca se reactiva sola a `active` (a diferencia de `archived`). Aunque estando ignorada la severidad suba de aviso a crítico, sigue sin avisar. El historial y la cronología previos se conservan; solo corta hacia el futuro. Nueva pestaña «Ignoradas» en el informe de alertas con un botón para dejar de ignorar (vuelve a `active` si la condición se cumple, o a `resolved` si no). Diálogo de confirmación al ignorar, como el de archivar. El botón aparece en todas las alertas EXCEPTO en un conjunto de reglas no ignorables que representan daño físico o fallo inminente del disco: smart.health.failed, nvme.critical_warning, smart.wear_high, smart.spare_below_threshold, smart.media_errors, smart.error_log, events.disk_predictive. Motivación: el usuario quiere silenciar de forma permanente falsos positivos o incidencias que ya ha evaluado y acepta, sin que reaparezcan; archivar no sirve porque se reactiva."

## Contexto

El informe de alertas ya ofrece tres acciones sobre un grupo: **reconocer**, **silenciar** (durante
un tiempo o indefinidamente) y **archivar**. Ninguna cubre el caso «he mirado esto, sé lo que es, lo
acepto y no quiero volver a verlo nunca»:

- **Reconocer** deja la alerta activa y el disco sigue con su color.
- **Silenciar indefinidamente** suprime la notificación, pero la alerta sigue en la lista y el disco
  mantiene su color.
- **Archivar** la retira de la vista y del color, pero **se reactiva sola** en cuanto la condición
  vuelve a cumplirse (`archived` → `active`, `cycle + 1`).

Hace falta una acción terminal que corte una regla concreta para un disco concreto de forma
duradera, sin dejar de registrar lo que pase por si el usuario quiere revisarlo más adelante, y sin
que el sistema pueda ocultar por esta vía un fallo físico grave.

## Clarifications

### Session 2026-09-08

- Q: ¿`events.filesystem_error` y `events.disk_error` entran también en el conjunto no ignorable? → A: No. Se quedan ignorables; el conjunto no ignorable son 7 reglas (solo SMART/NVMe + `events.disk_predictive`). Un error de E/S o de estructura NTFS puede venir del cable, la carcasa o el controlador; la muerte real del disco ya la cubren las siete reglas bloqueadas.
- Q: ¿Rótulo de la acción y de la pestaña? → A: Acción "Ignorar"; pestaña "Ignoradas". Encaja con Reconocer/Silenciar/Archivar y no promete que se deje de registrar nada; el matiz completo lo explica el diálogo de confirmación.
- Q: Mientras está `ignored` y la condición recae, ¿episodio nuevo o congelado? → A: Las ocurrencias se apuntan con su valor real y la severidad registrada del grupo se actualiza al peor valor visto, pero `cycle` no avanza (no hay frontera de episodio sin transición de estado). Al dejar de ignorar, el usuario ve la gravedad actual.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Dejar de ver para siempre una alerta aceptada (Priority: P1)

Una persona tiene un disco USB externo que provoca de vez en cuando una alerta de reinicio de
controladora al conectarlo en caliente. Sabe que es el conector y no le preocupa. Quiere que esa
alerta, para ese disco, no vuelva a aparecer ni a contar para el estado, pero que la aplicación siga
llevando la cuenta por si algún día empeora de verdad.

**Why this priority**: Es el motivo de la feature. Sin esto, la única salida es archivar una y otra
vez la misma alerta cada vez que reaparece, o silenciarla indefinidamente y convivir con el color
alterado. Entrega valor por sí sola.

**Independent Test**: Con una alerta activa de una regla ignorable, pulsar «Ignorar», confirmar, y
comprobar que (a) desaparece de la pestaña *Activas*, (b) el color del disco vuelve a reflejar solo
el resto de alertas, (c) aparece en la pestaña *Ignoradas*, y (d) tras forzar una nueva ocurrencia
de la misma condición no hay notificación, el color no cambia y la ocurrencia queda registrada en la
cronología del grupo.

**Acceptance Scenarios**:

1. **Given** un grupo de alerta en estado `active` de una regla ignorable, **When** el usuario pulsa
   «Ignorar» y confirma en el diálogo, **Then** el grupo pasa a estado `ignored`, sale de la pestaña
   *Activas* y deja de contar para el color del disco.
2. **Given** un grupo en estado `ignored`, **When** la condición de su regla se vuelve a cumplir en
   un ciclo posterior, **Then** se añade una ocurrencia a su cronología con su valor real, pero
   **no** se emite notificación, **no** cambia el estado del grupo, **no** avanza el contador de
   ciclo y **no** cambia el color del disco.
3. **Given** un grupo en estado `ignored` cuya severidad registrada era «advertencia», **When** una
   ocurrencia posterior alcanza el umbral «crítico», **Then** la ocurrencia se registra con su valor
   real, la severidad registrada del grupo pasa a «crítico», pero el grupo sigue `ignored` y sigue
   sin notificar ni teñir el disco.
4. **Given** un grupo en cualquier estado (`active`, `acknowledged`, `resolved`, `archived`),
   **When** el usuario lo ignora, **Then** el historial y la cronología previos se conservan
   íntegros y solo se corta el comportamiento hacia el futuro.

---

### User Story 2 - Revisar y revertir lo que se ha ignorado (Priority: P1)

La misma persona, meses después, quiere repasar qué alertas silenció y decidir si alguna vuelve a
ser relevante.

**Why this priority**: Ignorar sin poder revisar ni revertir sería una trampa: el usuario perdería
para siempre la visibilidad de algo que un día quiso posponer. La conversación de diseño lo fijó
como requisito explícito.

**Independent Test**: Abrir la pestaña *Ignoradas*, seleccionar un grupo, pulsar «Dejar de
ignorar», y comprobar que el grupo vuelve a la vida normal: si su condición se cumple ahora, aparece
en *Activas* y vuelve a contar para el color; si no, queda como resuelto.

**Acceptance Scenarios**:

1. **Given** uno o más grupos en estado `ignored`, **When** el usuario abre la pestaña *Ignoradas*,
   **Then** ve la lista de esos grupos con su regla, su disco, su severidad registrada y su última
   ocurrencia.
2. **Given** un grupo `ignored` cuya condición **se sigue cumpliendo**, **When** el usuario pulsa
   «Dejar de ignorar», **Then** en el siguiente ciclo de evaluación el grupo aparece en *Activas*,
   cuenta para el color y puede volver a notificar.
3. **Given** un grupo `ignored` cuya condición **ya no se cumple**, **When** el usuario pulsa «Dejar
   de ignorar», **Then** el grupo queda en estado `resolved` y aparece en la pestaña *Resueltas*.

---

### User Story 3 - No se puede ignorar un disco que se está muriendo (Priority: P1)

Una persona ve una alerta de que el disco reporta un fallo de salud SMART. Le agobia y quiere
quitársela de en medio.

**Why this priority**: Permitir ignorar señales de fallo físico convertiría la aplicación en un
monitor que oculta justo lo que debe vigilar. Es una restricción de seguridad del producto, no un
detalle. Debe entrar con la feature, no después.

**Independent Test**: Abrir el detalle de una alerta de una regla no ignorable y comprobar que la
acción de ignorar no está disponible y que el motivo es visible para el usuario.

**Acceptance Scenarios**:

1. **Given** un grupo de alerta de una regla del conjunto no ignorable, **When** el usuario abre su
   detalle, **Then** la acción «Ignorar» aparece deshabilitada con un texto que explica por qué no
   se puede ignorar esa alerta.
2. **Given** un intento de ignorar un grupo de una regla no ignorable por cualquier vía, **When** se
   procesa, **Then** se rechaza con un error comprensible y el grupo no cambia de estado.

---

### Edge Cases

- **Grupo ya silenciado que se ignora**: el silencio (`muted_until`) queda irrelevante mientras el
  grupo está `ignored`; al dejar de ignorar, se respeta el `muted_until` que hubiera si aún está en
  el futuro.
- **Ignorar desde la pestaña «Todas»**: la acción funciona igual con independencia de la pestaña
  desde la que se abra el detalle.
- **Un grupo `ignored` que también estaba `archived`**: `ignored` es el estado efectivo; al dejar de
  ignorar vuelve a `resolved` o `active` según la condición, no a `archived`.
- **La regla de un grupo `ignored` deja de existir o cambia de identificador**: el grupo permanece
  visible en *Ignoradas* con su último dato conocido; no genera error.
- **Retención de datos**: la limpieza periódica no borra un grupo `ignored` ni sus ocurrencias
  mientras siga ignorado (coherente con que la retención nunca borra alertas).
- **Reevaluación tras «dejar de ignorar»**: si el usuario deja de ignorar y cierra la aplicación
  antes del siguiente ciclo, al reabrir el grupo se evalúa con normalidad.
- **El conjunto no ignorable y una alerta ya `ignored` de esas reglas antes de esta feature**: no
  puede existir, porque la feature nace con la restricción; no hay migración de datos que contemple
  ese caso.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema MUST ofrecer una acción rotulada **«Ignorar»** sobre un grupo de alerta
  que lo lleve a un estado nuevo, distinto de `active`, `acknowledged`, `resolved` y `archived`,
  cuyo nombre a efectos de esta spec es `ignored`.
- **FR-002**: La acción «Ignorar» MUST pedir confirmación en un diálogo que declare su impacto,
  igual que hace «Archivar», antes de aplicarse. El texto del diálogo MUST aclarar que la alerta se
  sigue registrando en segundo plano pero deja de avisar y de contar para el color, y que es
  reversible desde la pestaña «Ignoradas».
- **FR-003**: Un grupo en estado `ignored` MUST seguir registrando en su cronología cada nueva
  ocurrencia de su condición, con su valor real. La severidad registrada del grupo MUST actualizarse
  al peor valor visto mientras está ignorado, pero el contador de ciclo (`cycle`) MUST NOT avanzar:
  no hay frontera de episodio nuevo sin transición de estado.
- **FR-004**: Un grupo en estado `ignored` MUST NOT generar ninguna notificación, con independencia
  de la severidad de las ocurrencias posteriores, incluida una subida de «advertencia» a «crítico».
- **FR-005**: Un grupo en estado `ignored` MUST NOT contar para el color de estado del disco ni para
  el estado global de la aplicación.
- **FR-006**: Un grupo en estado `ignored` MUST NOT reactivarse automáticamente a `active` por
  cumplirse de nuevo su condición. Solo una acción explícita del usuario lo saca de `ignored`.
- **FR-007**: Ignorar un grupo MUST conservar íntegros su contador histórico, su cronología y todas
  sus ocurrencias previas. La acción solo afecta al comportamiento futuro.
- **FR-008**: El sistema MUST permitir ignorar un grupo esté en el estado que esté (`active`,
  `acknowledged`, `resolved`, `archived`).
- **FR-009**: El informe de alertas MUST ofrecer una pestaña rotulada **«Ignoradas»**, junto a las
  existentes, que liste los grupos en estado `ignored` con, al menos, su regla, su disco, su
  severidad registrada y su última ocurrencia.
- **FR-010**: El sistema MUST ofrecer una acción **«Dejar de ignorar»** sobre un grupo en estado
  `ignored`.
- **FR-011**: Al dejar de ignorar un grupo, si su condición se sigue cumpliendo en la siguiente
  evaluación, el grupo MUST volver a `active`, contar para el color y poder notificar; si no se
  cumple, MUST quedar en `resolved`.
- **FR-012**: El sistema MUST definir un conjunto cerrado de reglas **no ignorables** que
  representan daño físico o fallo inminente del disco. La versión inicial de ese conjunto es:
  `smart.health.failed`, `nvme.critical_warning`, `smart.wear_high`,
  `smart.spare_below_threshold`, `smart.media_errors`, `smart.error_log`, `events.disk_predictive`.
- **FR-013**: Para un grupo de una regla no ignorable, la acción «Ignorar» MUST presentarse
  deshabilitada, con un texto visible que explique que ese tipo de alerta no se puede ignorar.
- **FR-014**: Cualquier intento de llevar a `ignored` un grupo de una regla no ignorable MUST
  rechazarse con un error comprensible, sin cambiar el estado del grupo.
- **FR-015**: La documentación normativa de reglas de alerta (ciclo de vida, transiciones, qué
  cuenta para el color) MUST actualizarse para incluir el estado `ignored` y la restricción de
  reglas no ignorables.
- **FR-016**: La retención periódica de datos MUST NOT borrar un grupo en estado `ignored` ni sus
  ocurrencias mientras siga ignorado.
- **FR-017**: Los textos nuevos de interfaz (acción, diálogo de confirmación, nombre de la pestaña,
  motivo de deshabilitado, acción de revertir) MUST existir en los dos idiomas de la aplicación.

### Key Entities *(include if feature involves data)*

- **Grupo de alerta**: unidad de deduplicación existente (regla + tipo y id de objetivo +
  contexto). Gana un estado posible más, `ignored`, y las transiciones asociadas. Conserva contador
  histórico, severidad registrada y cronología.
- **Estado `ignored`**: estado terminal por decisión del usuario. No cuenta para el color, no
  notifica, no se abandona solo. Se sale de él únicamente con «Dejar de ignorar».
- **Conjunto de reglas no ignorables**: lista cerrada de `rule_key` para las que la acción «Ignorar»
  está vetada. Es una regla de negocio, no una preferencia configurable.
- **Ocurrencia**: entrada de cronología existente. Se sigue creando para grupos `ignored`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Tras ignorar una alerta de una regla ignorable, esa alerta no vuelve a aparecer en la
  pestaña *Activas* ni a alterar el color del disco en ningún momento posterior, hasta que el
  usuario la saque de *Ignoradas*.
- **SC-002**: Después de ignorar una alerta, el usuario no recibe ninguna notificación de ella,
  aunque su condición se repita o empeore.
- **SC-003**: El 100 % de las ocurrencias de una alerta ignorada que ocurren mientras está ignorada
  quedan registradas y son consultables en su cronología.
- **SC-004**: Un usuario puede localizar todas sus alertas ignoradas y revertir cualquiera de ellas
  en menos de 15 segundos desde el informe de alertas.
- **SC-005**: Para las siete reglas del conjunto no ignorable, no existe ninguna secuencia de
  acciones en la interfaz que deje la alerta en estado `ignored`.
- **SC-006**: Ninguna alerta ni ocurrencia previa se pierde ni se altera al ignorar o al dejar de
  ignorar un grupo.

## Assumptions

- **Ámbito de «Ignorar» = un grupo concreto**, identificado por su clave de deduplicación (regla +
  disco + contexto). No existe «ignorar esta regla para todos los discos» en esta versión.
- **La pestaña «Ignoradas» es un filtro más** del informe de alertas, junto a *Activas*,
  *Resueltas*, *Archivadas* y *Todas*. La pestaña *Todas* incluye también los grupos `ignored`.
- **Las reglas no ignorables se presentan como acción deshabilitada con motivo**, no ocultando la
  acción, por coherencia con cómo la aplicación trata el resto de acciones no disponibles.
- **`events.filesystem_error` y `events.disk_error` quedan como reglas ignorables** (aclarado
  2026-09-08): pueden tener causas ajenas al disco y las señales de muerte real del disco ya están
  bloqueadas por las siete reglas SMART/NVMe del conjunto.
- **«Dejar de ignorar» no fuerza una evaluación inmediata**: el grupo se normaliza en el siguiente
  ciclo del recopilador correspondiente. El usuario puede ver un breve retardo hasta que la alerta
  reaparece en *Activas* si su condición se cumple.
- **Ignorar es independiente de silenciar y de reconocer**: son mecanismos ortogonales; ignorar no
  borra `muted_until` ni el reconocimiento previo, simplemente los hace irrelevantes mientras dure.
- **No hay migración de datos**: la feature introduce un estado nuevo; los grupos existentes no
  cambian hasta que el usuario los ignore.
- **La acción de ignorar, como el resto de acciones sobre alertas, pasa por la capa de comandos de
  la aplicación** y no la ejecuta la interfaz directamente.
