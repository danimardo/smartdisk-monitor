# Feature Specification: Eventos de Windows en el detalle de disco

**Feature Branch**: `[012-eventos-por-disco]`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "Añadir en el detalle de disco (/disks/[id]) una sección de eventos de Windows recientes de ese disco. Debajo del bloque de gráficas + contadores: una Card con título, enlace \"Ver todos\" a /events?deviceId=<id>, y los últimos 5 eventos de ese disco (getSystemEvents({deviceId, limit: 5})) usando EventRow con href=\"/events?focus={id}\" para navegar al detalle/XML/explicación en la pantalla de Eventos existente. Estado vacío diseñado cuando no hay eventos. Sin filtros propios, sin paginación, sin panel de detalle inline: solo un resumen con enlace, igual que el widget \"Últimos eventos\" del panel general pero filtrado a este disco. No hay cambios de backend ni de contratos (get_system_events ya acepta deviceId)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver de un vistazo los últimos eventos de este disco (Priority: P1) 🎯 MVP

Al investigar un disco concreto (por ejemplo porque está en aviso o crítico, o porque su gráfica de
temperatura o actividad muestra algo raro), la persona quiere saber si Windows ha registrado algún
suceso relacionado con ese disco, sin tener que ir a la pantalla general de Eventos y filtrar a
mano por dispositivo.

**Por qué esta prioridad**: es el valor completo de la funcionalidad solicitada. Sin esto no hay
nada que entregar.

**Prueba independiente**: abrir el detalle de un disco que tenga eventos asociados y comprobar que
aparecen, ordenados del más reciente al más antiguo, con su nivel, proveedor, ID y hora.

**Escenarios de aceptación**:

1. **Given** un disco con eventos de Windows asociados, **When** la persona abre su pantalla de
   detalle, **Then** ve una sección con los eventos más recientes de ese disco (hasta 5),
   cada uno con su icono de nivel, mensaje, proveedor, ID de evento y hora.
2. **Given** un evento mostrado en esa sección con asociación inferida (no confirmada) al
   dispositivo, **When** la persona lo ve, **Then** el evento lleva la misma indicación de
   "asociación no confirmada" que ya usa la pantalla de Eventos, para no sugerir una certeza que no
   existe.
3. **Given** la sección de eventos, **When** llegan más de 5 eventos a lo largo del tiempo,
   **Then** solo se muestran los 5 más recientes de ese disco.

---

### User Story 2 - Profundizar en un evento concreto (Priority: P2)

Tras ver un evento en el resumen, la persona quiere consultar su detalle completo: el mensaje
íntegro, el XML crudo si existe, y si la IA está activa, pedir una explicación en lenguaje claro.

**Por qué esta prioridad**: sin esto el resumen sería un callejón sin salida; sin embargo, el valor
principal de la User Story 1 ya se cumple sin esta capacidad ampliada, porque la persona ya ve
qué ha pasado.

**Independent Test**: desde la sección de eventos del disco, pulsar sobre un evento y comprobar que
lleva a la pantalla de Eventos existente, con ese suceso concreto ya seleccionado y su detalle
abierto.

**Acceptance Scenarios**:

1. **Given** la sección de eventos de un disco, **When** la persona pulsa sobre uno de los
   eventos listados, **Then** se abre la pantalla de Eventos con ese suceso ya resaltado y su panel
   de detalle (mensaje completo, XML si existe, y el botón "Explícamelo con IA" si la IA está
   activa) ya abierto.

---

### User Story 3 - Ver todos los eventos de este disco (Priority: P2)

Cuando el resumen de 5 eventos no es suficiente porque hay más historial que investigar, la persona
quiere pasar a la pantalla completa de Eventos, ya filtrada a este disco, sin tener que volver a
aplicar el filtro a mano.

**Por qué esta prioridad**: extiende la User Story 1 a un caso de uso real de investigación más
profunda, pero no es imprescindible para el valor mínimo (ver los eventos recientes ya lo es).

**Independent Test**: desde el detalle de un disco, pulsar el enlace "Ver todos" de la sección de
eventos y comprobar que la pantalla de Eventos se abre con el filtro de dispositivo ya aplicado a
ese disco.

**Acceptance Scenarios**:

1. **Given** la sección de eventos de un disco, **When** la persona pulsa "Ver todos", **Then** se
   abre la pantalla de Eventos con el filtro de dispositivo preaplicado a ese disco.

---

### Edge Cases

- **Disco sin ningún evento asociado**: la sección se sigue mostrando (no desaparece), con un
  estado vacío diseñado que indica que no hay eventos recientes para ese disco.
- **La consulta de eventos falla** (p. ej. el registro de sucesos de Windows no responde): la
  sección muestra su propio estado de error, sin afectar al resto de la pantalla (cabecera,
  métricas, gráficas y contadores siguen funcionando cada uno por su cuenta, mismo patrón que ya
  usan las dos gráficas históricas de esta misma pantalla).
- **Disco no compatible con SMART**: la sección de eventos se muestra igual, porque no depende de
  SMART, solo del registro de eventos de Windows asociado a ese dispositivo.
- **Evento con asociación "inferida" o "desconocida"**: solo se listan aquí los eventos que el
  backend ya asocia a este dispositivo (asociación exacta o inferida); un evento sin asociación a
  ningún disco no aparece en la sección de ningún disco concreto, solo en el listado general de
  Eventos.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: La pantalla de detalle de un disco DEBE mostrar una sección con los eventos de
  Windows más recientes asociados a ese disco, situada debajo del bloque de gráficas y contadores.
- **FR-002**: La sección DEBE mostrar como máximo los 5 eventos más recientes del disco, ordenados
  del más reciente al más antiguo.
- **FR-003**: Cada evento listado DEBE mostrar su nivel, mensaje, proveedor, identificador de
  evento y hora, con el mismo formato visual que usa la pantalla de Eventos para una fila.
- **FR-004**: Un evento cuya asociación al disco no sea exacta DEBE mostrar la misma indicación de
  asociación no confirmada que ya usa la pantalla de Eventos.
- **FR-005**: Al seleccionar un evento de la sección, el sistema DEBE llevar a la persona a la
  pantalla de Eventos con ese suceso ya resaltado y su detalle abierto.
- **FR-006**: La sección DEBE incluir un enlace que lleve a la pantalla de Eventos con el filtro de
  dispositivo ya aplicado a ese disco.
- **FR-007**: Cuando el disco no tenga eventos recientes, la sección DEBE mostrar un estado vacío
  diseñado en vez de ocultarse u omitirse.
- **FR-008**: Cuando la consulta de eventos falle, la sección DEBE mostrar su propio estado de
  error sin impedir que el resto de la pantalla de detalle funcione.
- **FR-009**: La sección NO DEBE incluir filtros propios de nivel o proveedor, ni paginación, ni un
  panel de detalle con el XML crudo: esas capacidades ya existen en la pantalla de Eventos y no se
  duplican aquí.
- **FR-010**: La sección DEBE funcionar igual para un disco compatible con SMART y para uno que no
  lo sea, porque no depende de los datos SMART.

### Key Entities

- **Evento de Windows** (ya existente): suceso del registro de eventos de Windows, con nivel,
  proveedor, identificador, hora, mensaje y grado de confianza de su asociación a un disco. Esta
  funcionalidad no añade campos nuevos, solo consulta y muestra los ya existentes filtrados por
  disco.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Desde el detalle de un disco con eventos asociados, una persona identifica el evento
  de Windows más reciente de ese disco sin necesidad de navegar a ninguna otra pantalla ni aplicar
  ningún filtro manual.
- **SC-002**: Desde el detalle de un disco, una persona llega al historial completo de eventos de
  ese disco, ya filtrado, en una sola interacción (un clic o pulsación).
- **SC-003**: Un disco sin eventos asociados comunica claramente esa ausencia, en vez de dejar a la
  persona sin saber si la sección ha fallado o simplemente no hay nada que mostrar.

## Assumptions

- La asociación entre un evento de Windows y un disco concreto ya la resuelve el backend existente
  (parámetro `deviceId` de la consulta de eventos); esta funcionalidad no cambia esa lógica, solo
  la consume filtrada a un disco.
- El límite de 5 eventos en el resumen es un valor de partida razonable para no saturar la pantalla
  de detalle, coherente con el resumen de 3 eventos (sin filtrar por disco) que ya muestra el panel
  general; puede ajustarse si al validarlo en pantalla real resulta corto o largo.
- No se requiere ninguna capacidad nueva de backend ni cambio de contrato: la consulta de eventos ya
  admite filtrar por disco, y la navegación a la pantalla de Eventos con un dispositivo o un suceso
  preaplicado ya existe.
- Fuera de alcance: filtros propios de nivel/proveedor en esta sección, paginación, panel de
  detalle inline con el XML crudo, y cualquier acción de IA directamente desde esta sección (todo
  eso ya vive en la pantalla de Eventos, a un clic de distancia).
