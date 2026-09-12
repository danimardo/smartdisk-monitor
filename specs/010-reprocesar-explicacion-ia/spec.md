# Feature Specification: Reprocesar la explicación con IA con otro modelo gratuito

**Feature Branch**: `[010-reprocesar-explicacion-ia]`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "Reprocesar la explicación con IA eligiendo otro modelo gratuito — en el modal de ayuda con IA (`ExplicacionModal.svelte`), tras ver una respuesta o un error, poder elegir otro modelo gratuito en un selector y reprocesar la misma petición; ver la respuesta anterior sin que ocupe espacio; y poder fijar el modelo elegido como modelo por defecto para toda la aplicación. Los modelos de pago se muestran deshabilitados mientras esté activa la clave de demostración compartida (enmienda a ADR-054)."

## Clarifications

### Session 2026-09-12

- Q: ¿Debe el historial de respuestas de una sesión de explicación crecer sin límite mientras el
  modal esté abierto, o conviene fijar un máximo de intentos guardados (descartando el más
  antiguo)? → A: Sin límite: el historial crece mientras el modal esté abierto, se pierde entero al
  cerrarlo.
- Q: ¿"Fijar como predeterminado" debe poder usarse también sobre la primerísima respuesta del modo
  automático, o solo sobre respuestas obtenidas eligiendo explícitamente un modelo? → A: Solo sobre
  respuestas obtenidas reprocesando con un modelo elegido explícitamente en el selector.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Probar otro modelo gratuito sobre la misma pregunta (Priority: P1)

Una persona pide "Explícamelo en lenguaje claro" sobre una alerta o un detalle técnico. La
respuesta del modelo automático le resulta poco clara, incompleta o confusa. En vez de cerrar la
ventana y no saber qué más hacer, quiere poder elegir otro modelo gratuito concreto y pedir que se
responda la misma pregunta con ese modelo, sin repetir el gesto original ni volver a revisar qué
datos se envían.

**Por qué esta prioridad**: es el motivo de ser de la función. Sin esto, el resto de historias no
tiene nada sobre lo que operar.

**Prueba independiente**: se puede probar por completo pidiendo una explicación, comprobando que
aparece un selector de modelo y un botón para reprocesar junto a la respuesta, eligiendo un modelo
gratuito distinto del que respondió y pulsando el botón: debe aparecer una respuesta nueva sin
cerrar el modal.

**Escenarios de aceptación**:

1. **Given** el modal muestra la respuesta de un modelo, **When** la persona elige otro modelo
   gratuito en el selector y pulsa reprocesar, **Then** se envía de nuevo exactamente la misma
   petición ya construida (mismo texto, misma alerta o detalle) al modelo elegido y se muestra su
   respuesta cuando llega.
2. **Given** la primera llamada terminó en error, **When** la persona elige otro modelo gratuito y
   pulsa reprocesar, **Then** se intenta la misma petición con el modelo nuevo en vez de repetirla
   con el modelo que falló.
3. **Given** la petición original necesitó pasar por la vista previa o por la revisión de
   fragmentos sensibles antes de enviarse, **When** se reprocesa, **Then** se pasa por el mismo
   paso de revisión antes de la llamada, igual que la primera vez.
4. **Given** el reproceso está en curso, **When** la persona cierra el modal, **Then** la llamada en
   curso se cancela igual que ocurre hoy con la primera llamada.

---

### User Story 2 - Fijar el modelo preferido como modelo por defecto (Priority: P2)

Tras comparar respuestas de varios modelos, la persona encuentra uno que le convence más. Quiere
que ese sea el modelo que se use a partir de ahora en toda la aplicación, sin tener que ir a
Ajustes a buscarlo y seleccionarlo de nuevo a mano.

**Por qué esta prioridad**: es el cierre natural de la historia 1 — probar modelos solo aporta
valor duradero si el resultado se puede adoptar sin fricción.

**Prueba independiente**: se puede probar pidiendo una explicación, reprocesando con un modelo
distinto, marcándolo como predeterminado, y comprobando que Ajustes → asistencia con IA refleja ese
modelo, y que una nueva explicación (o un informe con resumen de IA) usa ese modelo salvo que se
cambie de nuevo.

**Escenarios de aceptación**:

1. **Given** el modal muestra una respuesta obtenida al reprocesar con un modelo elegido
   explícitamente en el selector, **When** la persona marca esa respuesta como predeterminada,
   **Then** el ajuste general de modelo de la aplicación pasa a ser el de esa respuesta.
2. **Given** se acaba de fijar un modelo como predeterminado desde este modal, **When** se abre
   Ajustes → asistencia con IA, **Then** el selector de esa pantalla muestra el mismo modelo
   seleccionado.
3. **Given** se acaba de fijar un modelo como predeterminado desde este modal, **When** se genera un
   informe con resumen de IA, **Then** ese informe usa el mismo modelo fijado.

---

### User Story 3 - Consultar respuestas anteriores sin perder espacio (Priority: P3)

Después de reprocesar una o varias veces, la persona quiere poder volver a leer una respuesta
anterior para comparar matices, sin que el modal se llene de texto ni tenga que reprocesar otra vez
para volver a verla.

**Por qué esta prioridad**: mejora la comparación de la historia 1, pero la función ya aporta valor
sin ella (se podría comparar de memoria).

**Prueba independiente**: se puede probar reprocesando dos o más veces y comprobando que cada
respuesta que deja de ser la actual queda visible como una fila plegada con el modelo que la
generó, que se puede desplegar para releerla y volver a plegar.

**Escenarios de aceptación**:

1. **Given** se acaba de reprocesar y hay una respuesta nueva, **When** se muestra el modal,
   **Then** la respuesta que estaba visible justo antes pasa a mostrarse como una fila plegada,
   identificada por el modelo que la generó.
2. **Given** hay una o más filas plegadas de respuestas anteriores, **When** la persona despliega
   una de ellas, **Then** puede leer esa respuesta completa y volver a plegarla sin perder las
   demás.
3. **Given** el modal se cierra, **When** se vuelve a abrir para una explicación distinta, **Then**
   no queda ningún historial de la sesión anterior.

---

### User Story 4 - El selector no ofrece modelos que fallarán con la clave activa (Priority: P4)

La clave de demostración compartida solo tiene acceso a modelos gratuitos. Si la persona elige un
modelo de pago con esa clave activa, la llamada fallará. En vez de dejar que lo intente y reciba un
error de OpenRouter, el selector debe dejarle ver que esos modelos existen pero no están disponibles
ahora mismo, y por qué.

**Por qué esta prioridad**: evita un error confuso, pero no bloquea el valor principal de la
función (que ya funciona con modelos gratuitos).

**Prueba independiente**: se puede probar activando la clave de demostración y comprobando que los
modelos de pago aparecen en el selector mismo pero deshabilitados; y volviendo a probar tras
configurar una clave propia, comprobando que esos mismos modelos pasan a poder elegirse. Se
comprueba en el selector de este modal y en el de Ajustes.

**Escenarios de aceptación**:

1. **Given** la credencial activa es la clave de demostración compartida, **When** se abre
   cualquiera de los dos selectores de modelo de la aplicación (el de este modal y el de Ajustes),
   **Then** los modelos de pago aparecen listados pero no se pueden seleccionar.
2. **Given** la persona configura su propia clave de OpenRouter, **When** vuelve a abrir cualquiera
   de los dos selectores, **Then** los modelos de pago aparecen seleccionables con normalidad.

---

### Edge Cases

- Si la persona elige en el selector el mismo modelo que ya generó la respuesta visible y pulsa
  reprocesar, la petición se envía igual: dos llamadas al mismo modelo no tienen por qué devolver
  el mismo texto.
- Si el reproceso falla (error de red, de la credencial o del proveedor), se comporta como
  cualquier llamada fallida de esta función: se ofrece reintentar o elegir otro modelo, y la
  respuesta anterior sigue disponible plegada, sin perderse.
- Si no se puede obtener el catálogo de modelos (por ejemplo, sin conexión), el selector lo
  refleja sin bloquear la lectura de la respuesta ya obtenida.
- Fijar un modelo como predeterminado puede fallar al guardar el ajuste; en ese caso se avisa del
  error sin descartar la respuesta que se estaba viendo.
- Un informe con resumen de IA que ya estaba en curso cuando se cambia el modelo por defecto no
  se ve afectado a mitad de camino: el cambio rige para las próximas consultas de IA, no para las
  que ya estén en marcha.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El sistema DEBE permitir elegir un modelo distinto del que respondió (o del que
  falló) directamente desde el resultado o el error de una explicación con IA, sin repetir el
  gesto que la originó.
- **FR-002**: El sistema DEBE ofrecer, en ese mismo lugar, una acción para reprocesar la petición
  con el modelo elegido.
- **FR-003**: Reprocesar DEBE reutilizar exactamente el mismo contenido ya construido para la
  petición original (mismo texto, misma alerta o detalle técnico), cambiando únicamente el modelo
  de destino.
- **FR-004**: Si la petición original requirió una vista previa o una revisión de fragmentos antes
  de enviarse, reprocesar DEBE pasar por el mismo paso antes de la nueva llamada.
- **FR-005**: La acción de elegir otro modelo y reprocesar DEBE estar disponible tanto cuando la
  última llamada tuvo éxito como cuando terminó en error.
- **FR-006**: El selector de modelo de esta función DEBE excluir el modo automático (el que
  OpenRouter resuelve de forma distinta en cada llamada), por no ser comparable de una vez a otra.
- **FR-007**: Cada respuesta que deja de ser la que se muestra en primer plano (por haberse
  reprocesado) DEBE seguir siendo consultable dentro de la misma sesión de explicación, identificada
  por el modelo que la generó, sin ocupar espacio visible mientras no se consulta. El historial de
  una sesión no tiene un máximo: conserva todos los intentos hechos mientras el modal permanece
  abierto.
- **FR-008**: El sistema DEBE ofrecer una acción, sobre una respuesta obtenida al reprocesar con un
  modelo elegido explícitamente en el selector, para fijar ese modelo como el modelo por defecto de
  la aplicación. Esta acción NO DEBE ofrecerse sobre la respuesta inicial del modo automático, para
  no fijar por accidente el modelo puntual que ese modo haya resuelto esa vez.
- **FR-009**: Fijar un modelo como predeterminado desde esta función DEBE cambiar el mismo ajuste
  general de modelo que ya existe en la configuración de la aplicación, de modo que rija también
  para el resumen con IA de los informes y para cualquier otra consulta de IA futura.
- **FR-010**: Mientras la credencial de OpenRouter activa sea la clave de demostración compartida,
  los modelos de pago DEBEN mostrarse en los selectores de modelo (el de esta función y el de la
  configuración general) visibles pero no seleccionables.
- **FR-011**: En cuanto la credencial activa deja de ser la clave de demostración compartida, los
  modelos de pago DEBEN volver a poder elegirse con normalidad en ambos selectores.
- **FR-012**: El historial de respuestas de una sesión de explicación NO DEBE conservarse más allá
  de esa sesión: al cerrar y volver a abrir la función para otro caso, no debe quedar rastro de las
  respuestas anteriores.
- **FR-013**: Cancelar una llamada de reproceso en curso DEBE ocultar el modal igual que cancelar la
  llamada original; no se pierde lo que ya se estaba mostrando antes de reprocesar, y una respuesta
  que llegue después de cancelar no debe alterar la pantalla ni reabrir el modal.

### Key Entities

- **Intento de explicación**: una llamada concreta de esta función, con el modelo con el que se
  pidió, el resultado obtenido (texto de la respuesta, o el error) y si es la que está en primer
  plano o ha quedado plegada. Vive solo mientras dura la sesión de explicación, sin un máximo de
  intentos acumulables; no se persiste.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Desde que se ve una respuesta o un error de una explicación con IA, se puede obtener
  una respuesta de otro modelo gratuito con dos acciones como máximo (elegir modelo, reprocesar),
  sin cerrar la ventana.
- **SC-002**: El 100% de las respuestas obtenidas en una misma sesión de explicación siguen siendo
  legibles bajo demanda, y ninguna ocupa espacio visible salvo la que está en primer plano.
- **SC-003**: Fijar un modelo como predeterminado desde el resultado se refleja de inmediato tanto
  en la configuración general como en la siguiente consulta de IA de cualquier parte de la
  aplicación, sin pasos adicionales.
- **SC-004**: Con la clave de demostración activa, el 100% de los modelos de pago listados se
  muestran identificables como no disponibles en vez de desaparecer, en los dos selectores de
  modelo de la aplicación.
- **SC-005**: Ningún reproceso envía a la red un contenido distinto del que la persona ya vio y
  aceptó en la petición original.

## Assumptions

- El modelo por defecto sigue siendo un único ajuste global, compartido entre esta función y el
  resumen con IA de los informes; esta función no introduce un valor por defecto independiente
  para cada uso de la IA.
- El historial de respuestas dentro de una sesión de explicación es efímero, igual que el resto del
  contenido de esta función (no se guarda en base de datos ni sobrevive a cerrar el modal).
- No se introduce ningún control de interfaz nuevo: la manera de consultar respuestas anteriores
  reutiliza el mismo patrón de fila plegable que esta función ya usa para el detalle técnico de un
  error.
- El criterio de deshabilitar los modelos de pago según la credencial activa se aplica por igual en
  el selector de esta función y en el que ya existe en la configuración general, para que ambos se
  comporten de la misma manera.
- Esta función no modifica el contenido ni el proceso de generación del informe HTML por disco más
  allá de heredar el modelo por defecto que aquí se pueda cambiar.
