# Feature Specification: Explicación en lenguaje llano con IA

**Feature Branch**: `005-explicacion-ia`

**Created**: 2026-09-08

**Status**: Draft

**Input**: User description: "Integración opcional con OpenRouter para traducir el detalle técnico de alertas y otras secciones a lenguaje llano mediante un LLM gratuito, con clave API opcional guardada en el gestor de credenciales de Windows, selección de modelo, anonimización de datos privados, y resultado mostrado en un modal en markdown con indicador de progreso."

## Contexto normativo

Esta feature se apoya en:

- **Principio XVI de la constitución** (enmienda 1.8.0): asistencia con IA en la nube, opcional,
  explícita y sin datos identificables. Fija las condiciones innegociables.
- **ADR-046**: OpenRouter como proveedor único, endpoint `https://openrouter.ai/api/v1`, modelo
  `openrouter/free` por defecto, llamada desde el backend.

Cualquier requisito de abajo que contradiga esos dos documentos está mal y se corrige hacia ellos.

## Clarifications

### Session 2026-09-08

- Q: ¿El selector de modelo lista solo los gratuitos o también los de pago? → A: Lista todos los
  modelos disponibles; al elegir uno de pago se muestra un aviso explícito de posibles cargos y se
  pide confirmación.
- Q: ¿Se envían al modelo la marca/modelo del disco y su firmware? → A: Sí. Se envían marca,
  modelo, tipo/interfaz (HDD/SSD/NVMe), firmware, antigüedad y horas de encendido. Solo el número
  de serie y los datos del equipo/usuario se sustituyen.
- Q: ¿Cuánto espera como máximo la aplicación por una respuesta? → A: 60 segundos; superado ese
  tiempo se cancela y se ofrece reintentar.
- Q: ¿La vista previa del texto a enviar se repite o se muestra una sola vez? → A: Se muestra una
  vez, tras activar la función; un indicador persistido recuerda que ya se mostró. La configuración
  pasa a guardar tres datos (activación, modelo elegido, vista previa mostrada).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Activar la ayuda con IA y gestionar la clave (Priority: P1)

Una persona que instala SmartDisk quiere poder pedir explicaciones en lenguaje claro. Durante el
asistente inicial se le ofrece, como paso **opcional**, introducir una clave de API de OpenRouter,
con una explicación de para qué sirve, qué datos se enviarán y a dónde. Puede saltarse el paso sin
consecuencias. Más tarde puede añadir, cambiar o borrar la clave desde la configuración, en un
apartado propio.

**Why this priority**: sin este paso la función no existe. Es el cimiento y además el punto donde
se obtiene el consentimiento informado que exige el principio XVI.

**Independent Test**: se puede probar por completo introduciendo una clave en el asistente o en la
configuración, comprobando que queda guardada de forma segura (no en claro en disco ni en la base
de datos), que una clave inválida se detecta y se comunica, y que al borrarla la función vuelve a
quedar inactiva. Con la función inactiva, la aplicación no realiza ninguna conexión de red.

**Acceptance Scenarios**:

1. **Given** el asistente inicial en el paso de la ayuda con IA, **When** la persona lo omite,
   **Then** la aplicación termina el asistente con la función desactivada y sin haber contactado
   con ninguna red.
2. **Given** el asistente inicial en el paso de la ayuda con IA, **When** la persona introduce una
   clave válida y continúa, **Then** la función queda activada y la clave se guarda en el almacén
   de credenciales de Windows.
3. **Given** una clave introducida, **When** la clave no es válida o es rechazada por el proveedor,
   **Then** la aplicación lo comunica con un mensaje comprensible y conserva el detalle técnico, y
   no deja la función a medias.
4. **Given** la función activada, **When** la persona borra la clave desde la configuración,
   **Then** la función queda desactivada, la credencial se elimina del almacén y las acciones de
   explicación desaparecen de la interfaz.
5. **Given** cualquier estado, **When** se inspecciona la base de datos y los ficheros de la
   aplicación, **Then** la clave de API no aparece en ninguno.

---

### User Story 2 - Entender una alerta sin ser técnico (Priority: P1)

Una persona ve una alerta con su detalle técnico (por ejemplo, un recuento de sectores
reasignados). No sabe si es grave ni qué hacer. Pulsa una acción del tipo «Explícamelo en lenguaje
claro». Mientras espera ve un indicador de progreso. Al terminar, aparece un modal con una
explicación redactada para alguien no técnico: qué significa, si debe preocuparse y, si procede,
qué pasos puede dar. El texto se muestra con formato (markdown) y deja claro que es una orientación
generada por IA, no un veredicto de la aplicación.

**Why this priority**: es el valor central de la feature; lo que pidió el usuario.

**Independent Test**: con una clave configurada, abrir una alerta, pulsar la acción, ver el
indicador de progreso y recibir un modal con la explicación en lenguaje llano y con formato. Se
puede verificar también que antes del envío se anonimiza el contenido y que un fallo de red no
rompe la pantalla.

**Acceptance Scenarios**:

1. **Given** una alerta con detalle técnico y la función activada, **When** la persona pulsa
   «Explícamelo en lenguaje claro», **Then** aparece un indicador de progreso y, al completarse,
   un modal con la explicación en lenguaje no técnico renderizada como markdown.
2. **Given** que es la primera consulta desde que se activó la función, **When** la persona pulsa
   la acción, **Then** antes de enviar nada se le muestra el texto exacto que saldrá del equipo y a
   dónde va, y debe confirmar.
3. **Given** una alerta cuyo detalle contiene el número de serie del disco, el nombre del equipo o
   una ruta con el perfil de usuario, **When** se prepara la consulta, **Then** esos datos se
   sustituyen por marcadores antes de salir del proceso, de forma consistente dentro de la
   petición.
4. **Given** la función activada pero sin conexión, o con la cuota del proveedor agotada, **When**
   la persona pulsa la acción, **Then** se muestra un error comprensible con su detalle técnico
   conservado y el resto de la aplicación sigue funcionando.
5. **Given** una explicación mostrada en el modal, **When** la persona la lee, **Then** el modal
   indica el modelo que la generó y que es una orientación por IA, no un dato de salud.
6. **Given** el modal abierto, **When** la persona pulsa `Escape` o el botón de cierre, **Then** el
   modal se cierra y devuelve el foco al elemento que lo abrió.

---

### User Story 3 - Elegir el modelo de IA (Priority: P2)

Una persona con conocimientos quiere usar un modelo concreto en vez del automático. En el asistente
y en la configuración puede elegir entre «modelo gratuito automático» (por defecto) y cualquier
modelo de la lista que se obtiene del proveedor. Si el modelo elegido es de pago, la aplicación lo
advierte y pide confirmación antes de guardarlo. La opción por defecto no exige entender nada.

**Why this priority**: mejora la función pero no es imprescindible para el MVP; el modelo
automático cubre el caso general.

**Independent Test**: abrir la configuración, ver el selector con «automático» por defecto,
desplegar la lista de modelos disponibles, elegir uno y comprobar que las siguientes explicaciones
lo usan y que el modal lo refleja.

**Acceptance Scenarios**:

1. **Given** la configuración de la ayuda con IA, **When** la persona abre el selector de modelo,
   **Then** ve «modelo gratuito automático» seleccionado y una lista de modelos obtenida del
   proveedor.
2. **Given** un modelo elegido a mano, **When** se realiza una explicación, **Then** se usa ese
   modelo y el modal lo indica.
3. **Given** que la lista de modelos no se puede obtener (sin red, proveedor caído), **When** la
   persona abre el selector, **Then** puede seguir usando «automático» y se le informa de que la
   lista no está disponible ahora.
4. **Given** el selector de modelo abierto, **When** la persona elige un modelo de pago, **Then** la
   aplicación muestra un aviso de que puede generar cargos en su cuenta de OpenRouter y no guarda la
   elección hasta que la persona la confirma.

---

### User Story 4 - Explicar el detalle SMART de un disco (Priority: P2)

Además de en las alertas, la persona encuentra detalle técnico en el detalle SMART de un disco.
Quiere la misma acción de «Explícamelo» ahí.

**Why this priority**: entra en la primera versión junto con las alertas, pero se construye después
del flujo de alerta porque comparte con él toda la mecánica (gesto, anonimización, modal); una vez
hecho ese flujo, extenderlo al detalle SMART es incremental.

**Independent Test**: abrir el detalle SMART de un disco, pulsar la acción de explicación y recibir
el modal con la misma mecánica que en las alertas.

**Acceptance Scenarios**:

1. **Given** la vista de detalle SMART de un disco y la función activada, **When** la persona pulsa
   «Explícamelo en lenguaje claro», **Then** recibe un modal con la explicación en lenguaje llano
   de esas métricas.
2. **Given** la función desactivada, **When** la persona abre cualquier vista con detalle técnico,
   **Then** no aparece ninguna acción de explicación.

---

### Edge Cases

- **Detalle muy largo**: si el detalle técnico supera un tamaño razonable, se acota antes de
  enviarlo y se avisa de que se ha enviado un extracto.
- **Respuesta vacía o incoherente del modelo**: se trata como error de la función, con mensaje
  comprensible; no se muestra un modal vacío.
- **Respuesta que tarda demasiado**: el tiempo máximo de espera es de 60 segundos; superado, se
  cancela y se ofrece reintentar.
- **La persona pulsa la acción dos veces**: no se lanzan dos consultas simultáneas para el mismo
  detalle.
- **Clave válida pero sin saldo/cuota gratuita agotada**: mensaje específico que distingue «clave
  incorrecta» de «límite alcanzado, inténtalo más tarde».
- **La anonimización no puede garantizar la limpieza de un fragmento de texto libre** (p. ej. la
  descripción de un evento de Windows): se muestra el texto exacto con el fragmento dudoso
  resaltado y la persona elige entre enviarlo, quitar el fragmento o cancelar (FR-026).
- **Cambio de idioma de la aplicación**: la explicación se pide en el idioma activo de la interfaz.
- **La respuesta del modelo intenta incluir marcado HTML o enlaces**: se renderiza como texto/
  markdown seguro, sin ejecutar nada.
- **El modelo automático enruta a un modelo distinto en cada llamada**: aceptable; el modal siempre
  refleja el que se usó realmente.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE ofrecer, en el asistente inicial, un paso opcional para activar la
  ayuda con IA introduciendo una clave de API de OpenRouter, con texto que explique para qué sirve,
  qué se enviará y a dónde.
- **FR-002**: El sistema DEBE permitir omitir ese paso y terminar el asistente con la función
  desactivada.
- **FR-003**: El sistema DEBE ofrecer un apartado propio en la configuración para añadir, cambiar,
  probar y borrar la clave de API.
- **FR-004**: El sistema DEBE guardar la clave de API en el almacén de credenciales de Windows y
  NUNCA en la base de datos, en `localStorage` ni en ningún fichero.
- **FR-005**: Con la función desactivada (sin clave), el sistema NO DEBE realizar ninguna conexión
  de red: ni cliente, ni conexión, ni resolución de nombres.
- **FR-006**: El sistema DEBE mostrar una acción de explicación («Explícamelo en lenguaje claro»)
  en el detalle de una alerta cuando la función está activada, y ocultarla cuando está desactivada.
- **FR-007**: El sistema DEBE realizar la llamada al proveedor únicamente tras un gesto explícito
  de la persona; nunca de forma automática, en segundo plano, al arrancar ni al detectarse una
  alerta.
- **FR-008**: El sistema DEBE enviar solo el detalle técnico visible en ese momento y un contexto
  acotado del disco al que se refiere: marca, modelo, tipo/interfaz (HDD/SSD/NVMe), versión de
  firmware, antigüedad y horas de encendido. Nunca el inventario completo, el historial de otras
  métricas, la configuración ni datos de otras vistas.
- **FR-009**: El sistema DEBE sustituir por marcadores, antes de que el texto salga del proceso, el
  número de serie del disco y los datos del equipo y de la persona (nombre del equipo, nombre de
  usuario, rutas con perfil de usuario, etiquetas de volumen), con sustitución consistente dentro
  de una misma petición. La marca, el modelo, la interfaz y el firmware del disco NO se sustituyen:
  no identifican a una persona y son contexto necesario para la explicación.
- **FR-010**: El sistema DEBE mostrar, la primera vez que se va a consultar tras activar la
  función, el texto exacto que se enviará y a dónde va, y requerir confirmación. Una vez mostrada y
  confirmada, no se repite (ver FR-027).
- **FR-011**: El sistema DEBE mostrar un indicador de progreso mientras espera la respuesta.
- **FR-012**: El sistema DEBE mostrar la respuesta en un modal, renderizada como markdown seguro,
  sin ejecutar ni interpretar HTML.
- **FR-013**: El modal DEBE indicar el modelo que generó la explicación y advertir de que es una
  orientación por IA, no un dato de salud.
- **FR-014**: El sistema DEBE pedir al modelo una respuesta en formato markdown y en el idioma
  activo de la interfaz, que incluya qué significa el detalle, si debe preocupar y, si procede,
  posibles pasos a seguir.
- **FR-015**: El sistema DEBE usar por defecto el identificador de «modelo gratuito automático»
  (`openrouter/free`) y permitir elegir otro modelo de la lista completa obtenida del proveedor,
  tanto en el asistente como en la configuración.
- **FR-015a**: Cuando la persona elija un modelo de pago, el sistema DEBE advertir de que puede
  generar cargos en su cuenta de OpenRouter y NO DEBE guardar la elección hasta que la persona la
  confirme.
- **FR-016**: El sistema DEBE registrar internamente y mostrar en el modal el modelo realmente
  utilizado en cada respuesta.
- **FR-017**: El sistema DEBE tratar cualquier fallo (sin red, clave inválida, cuota agotada,
  tiempo excedido, respuesta vacía) como un error de la función con mensaje comprensible y detalle
  técnico conservado, sin afectar al resto de la aplicación.
- **FR-018**: El sistema DEBE distinguir en el mensaje de error entre «clave incorrecta» y «límite
  de uso alcanzado».
- **FR-019**: El sistema DEBE aplicar un tiempo máximo de 60 segundos a la espera de la respuesta;
  superado, cancela la consulta y ofrece reintentar.
- **FR-020**: El sistema NO DEBE lanzar dos consultas simultáneas para el mismo detalle.
- **FR-021**: El sistema DEBE acotar el detalle técnico que exceda un tamaño razonable antes de
  enviarlo, e informar de que se envió un extracto.
- **FR-022**: El sistema NO DEBE escribir en el registro de actividad el contenido de las peticiones
  ni de las respuestas; solo metadatos no identificables (que hubo una consulta, el modelo, si hubo
  error).
- **FR-023**: El sistema NO DEBE usar la respuesta del modelo para tomar ninguna decisión de la
  aplicación (color de estado, alertas, cualquier regla); es solo texto para la persona.
- **FR-024**: El sistema DEBE guardar en la configuración únicamente tres datos: el estado de
  activación de la función, el modelo elegido y si la vista previa del texto a enviar ya se mostró.
- **FR-027**: El sistema DEBE recordar de forma persistente que la vista previa de FR-010 ya se
  mostró y se confirmó, y NO DEBE volver a mostrarla en consultas posteriores ni tras reiniciar la
  aplicación. Si la función se desactiva y se vuelve a activar, la vista previa se muestra de nuevo.
- **FR-025**: El sistema DEBE ofrecer la acción de explicación en el detalle de una alerta y en el
  detalle SMART de un disco. Otras vistas y los mensajes de error (`AppError`) quedan fuera de la
  primera versión.
- **FR-026**: Cuando la anonimización no pueda garantizar que un fragmento de texto libre (por
  ejemplo, la descripción de un evento de Windows) esté limpio de datos identificables, el sistema
  DEBE mostrar a la persona el texto exacto que saldría, resaltando el fragmento dudoso, y ofrecer
  tres opciones: enviarlo tal cual, quitar ese fragmento y enviar el resto, o cancelar la consulta.

### Key Entities *(include if feature involves data)*

- **Clave de API de OpenRouter**: secreto que habilita la función. Vive en el almacén de
  credenciales de Windows. Atributos observables: existe / no existe; válida / inválida.
- **Preferencia de la ayuda con IA**: en la configuración. Tres datos: estado de activación, modelo
  elegido («automático» o un identificador concreto) y si la vista previa del texto a enviar ya se
  mostró y confirmó.
- **Consulta de explicación**: efímera. Detalle técnico de origen (con el número de serie y los
  datos de equipo/usuario ya sustituidos), contexto acotado del disco (marca, modelo, interfaz,
  firmware, antigüedad, horas de encendido), idioma y modelo solicitado. No se persiste.
- **Explicación devuelta**: efímera. Texto en markdown y modelo realmente usado. Se muestra en el
  modal y no se guarda.
- **Catálogo de modelos**: lista de modelos disponibles obtenida del proveedor para poblar el
  selector. No se persiste; se consulta al abrir el selector.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Con la función desactivada, una inspección de red durante un día de uso normal
  muestra **cero** conexiones salientes originadas por la aplicación.
- **SC-002**: Una persona no técnica, ante una alerta que no entiende, obtiene una explicación
  legible en menos de 20 segundos desde que pulsa la acción (incluida la confirmación de la primera
  vez), en condiciones de red normales.
- **SC-003**: En el 100 % de las consultas, el texto enviado al proveedor no contiene números de
  serie, nombres de equipo o de usuario, ni rutas con perfil de usuario, verificado sobre un
  conjunto de casos de prueba que sí los contienen en origen.
- **SC-004**: La clave de API no aparece en la base de datos, en `localStorage` ni en ningún
  fichero de la aplicación, verificado por inspección.
- **SC-005**: Ante un fallo del proveedor (sin red, cuota agotada, tiempo excedido), la aplicación
  sigue mostrando el resto de sus pantallas sin degradación y presenta un mensaje que la persona
  entiende.
- **SC-006**: En el 100 % de las respuestas mostradas, el modal indica el modelo usado y la
  advertencia de que es orientación por IA.
- **SC-007**: Ninguna decisión de la aplicación (color de estado, activación de alertas, retención)
  cambia como consecuencia del contenido de una respuesta del modelo.
- **SC-008**: En el 100 % de los intentos de elegir un modelo de pago, la aplicación muestra el
  aviso de posibles cargos y no guarda la elección sin confirmación.
- **SC-009**: Tras confirmar la vista previa una vez, ninguna consulta posterior (ni tras reiniciar
  la aplicación) vuelve a mostrarla, mientras la función siga activada.

## Assumptions

- El público objetivo no es técnico; la explicación debe evitar jerga y traducir los términos.
- La función es una ayuda orientativa; la fuente de verdad sigue siendo la propia aplicación
  (principio I).
- La explicación devuelta es **efímera**: no se guarda ni se cachea. Volver a pedirla sobre el
  mismo detalle lanza una consulta nueva. (Se asume por simplicidad y por la cuota gratuita; puede
  revisarse si el gasto de cuota resulta molesto.)
- El texto que se envía se muestra íntegro **una sola vez**, la primera consulta tras activar la
  función; a partir de ahí basta con el indicador de progreso, porque el contenido enviado es del
  mismo tipo y ya se consintió. Desactivar y reactivar la función vuelve a mostrarlo.
- La llamada de red la hace el backend, no la interfaz (ADR-046, principio XVI).
- Los límites de cuota de OpenRouter (p. ej. 50 peticiones/día en cuenta gratuita) no se codifican
  en la aplicación; se maneja el error cuando llega.
- La lista de modelos para el selector se obtiene en el momento de abrirlo; no se mantiene una copia
  local. El proveedor indica en cada modelo si tiene coste, y la aplicación usa ese dato para
  decidir si mostrar el aviso de posibles cargos.
- Se asume que la persona que introduce una clave acepta que el detalle técnico anonimizado salga
  del equipo hacia OpenRouter; ese es el consentimiento que recoge FR-001 y FR-010.
- El modo oscuro/claro, el tamaño mínimo de ventana (1024×560) y la accesibilidad AA aplican al
  nuevo modal y a los nuevos apartados de configuración y del asistente, como a cualquier pantalla.
