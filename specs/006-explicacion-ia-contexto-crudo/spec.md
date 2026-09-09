# Feature Specification: Contexto crudo para la explicación con IA

**Feature Branch**: `006-explicacion-ia-contexto-crudo`

**Created**: 2026-09-08

**Status**: Draft

**Input**: User description: "Incremento sobre la feature 005 (ya fusionada): ampliar el contexto
que la ayuda con IA envía al LLM al pulsar «Explícamelo en lenguaje claro» — volcado crudo de
`smartctl` en alertas y detalle SMART, contenido del suceso en alertas de eventos de Windows,
anonimización reforzada y un modo «enviar sin revisar» opcional."

## Contexto normativo

Este incremento se apoya en la feature **005-explicacion-ia** (ya implementada y fusionada) y en:

- **Principio XVI de la constitución**: asistencia con IA en la nube, opcional, explícita y sin
  datos identificables. **Este incremento requiere una enmienda a ese principio** (versión
  1.8.1 → 1.9.0): la cláusula «Solo el detalle técnico que la persona ya tiene delante» se
  reescribe para incluir de forma explícita el volcado crudo de `smartctl` y el contenido del
  suceso de Windows como parte del detalle técnico visible (la persona los abre en la propia
  pantalla), y la cláusula de anonimización se amplía con el barrido reforzado y con el
  reconocimiento del modo «enviar sin revisar» como consentimiento de segundo nivel. Sin la
  enmienda aplicada, la feature no se da por terminada.
- **ADR-046**: OpenRouter como proveedor único; la llamada la hace el backend. No cambia.

Toda la mecánica de la 005 sigue vigente: clave en el almacén de credenciales de Windows, proveedor
y destino únicos, gesto explícito, anonimización en el dominio, modal en markdown seguro, vista
previa de la primera consulta, tiempo máximo de 60 s, cero telemetría. Este incremento **solo
amplía qué información se recopila para la consulta y añade un ajuste**.

## Clarifications

### Session 2026-09-08

- Q: ¿El WWN (identificador mundial) del disco se envía o se anonimiza? → A: Se anonimiza. Identifica
  la unidad física de forma única, igual que el número de serie. Marca, modelo, tipo/interfaz y
  firmware sí se conservan (contexto necesario, no identifican a una persona).
- Q: ¿Del suceso de Windows se envía el XML completo o solo su contenido? → A: Solo el contenido
  legible: el mensaje descriptivo y los campos de datos del suceso. El bloque de metadatos de
  sistema (identificadores del proveedor, de proceso e hilo, nombre del equipo, principal de
  seguridad) se descarta: no aporta a la explicación y concentra identificadores.
- Q: ¿El modo «enviar sin revisar» también omite la vista previa de la primera consulta (FR-010 de
  la 005)? → A: No. La vista previa de la primera consulta se sigue mostrando siempre. «Enviar sin
  revisar» solo omite la pantalla de revisión de fragmentos dudosos.
- Q: Si la ayuda con IA se desactiva y se vuelve a activar, ¿«enviar sin revisar» se conserva? → A:
  No. Vuelve a su estado de fábrica (desactivada), igual que la vista previa (FR-027 de la 005). El
  consentimiento de riesgo se vuelve a pedir.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Explicación de una alerta SMART apoyada en los datos reales del disco (Priority: P1)

Una persona ve una alerta de disco (sectores reasignados, desgaste, temperatura, error NVMe…).
Pulsa «Explícamelo en lenguaje claro». Hoy el modelo recibe solo el identificador interno de la
regla, la cifra que la disparó y su tendencia; explica a ciegas. Con este incremento, el modelo
recibe además el **volcado técnico completo del disco** —la misma salida que la persona puede abrir
con «Ver detalle técnico»—, ya anonimizado. La explicación resultante cita valores concretos: el
atributo afectado, su valor y su umbral, si hay entradas en el registro de errores del disco, las
horas de encendido reales.

**Why this priority**: es el valor central del incremento. Sin los datos reales, la explicación es
genérica y poco fiable; con ellos, orienta de verdad.

**Independent Test**: con la ayuda con IA activada, abrir una alerta `smart.*`/`temp.*`/`nvme.*`,
pulsar la acción, confirmar la vista previa si es la primera vez, y comprobar que el texto que sale
del equipo incluye el volcado del disco con el número de serie y el WWN sustituidos por marcadores,
y que la explicación del modal se apoya en cifras concretas del disco.

**Acceptance Scenarios**:

1. **Given** una alerta `smart.reallocated_high` y la función activada, **When** la persona pulsa
   la acción y confirma, **Then** la consulta enviada contiene el resumen estructurado (regla,
   valor, tendencia, contexto del disco) **y** el volcado técnico completo del disco.
2. **Given** un volcado que contiene el número de serie del disco y su WWN, **When** se prepara la
   consulta, **Then** ambos aparecen sustituidos por marcadores consistentes antes de que el texto
   salga del proceso; la marca, el modelo y el firmware se conservan.
3. **Given** que el disco de la alerta ya no está conectado al pulsar la acción, **When** el
   volcado no puede obtenerse, **Then** la consulta se envía solo con el resumen estructurado y el
   modal indica que la explicación se hizo sin el volcado.
4. **Given** un volcado que, sumado al resto, supera el tamaño máximo admitido, **When** se prepara
   la consulta, **Then** se envía un extracto y el modal avisa de que el detalle se acotó.

---

### User Story 2 - Explicación de una alerta de suceso de Windows con el contenido del evento (Priority: P1)

Una alerta nace de un suceso del registro de eventos de Windows (por ejemplo, un error de disco o
una corrupción de sistema de archivos). La persona pulsa «Explícamelo en lenguaje claro». Con este
incremento, el modelo recibe el **contenido legible del suceso** que disparó la alerta —su mensaje
descriptivo y sus campos de datos—, ya anonimizado, además del resumen estructurado. La explicación
puede así decir qué componente informó del problema, sobre qué volumen o dispositivo, y con qué
código.

**Why this priority**: para las alertas de sucesos, el identificador de regla por sí solo no dice
casi nada; el contenido del suceso es donde está la información.

**Independent Test**: con la función activada, abrir una alerta originada por un suceso de Windows,
pulsar la acción, y comprobar que el texto enviado incluye el mensaje del suceso y sus campos de
datos, sin el bloque de metadatos de sistema (nombre del equipo, SID, identificadores de proceso),
y con las rutas de dispositivo y los nombres de equipo sustituidos.

**Acceptance Scenarios**:

1. **Given** una alerta originada por un suceso de Windows y la función activada, **When** la
   persona pulsa la acción y confirma, **Then** la consulta contiene el mensaje descriptivo del
   suceso y sus campos de datos, además del resumen estructurado.
2. **Given** el contenido de un suceso que incluye el nombre del equipo, un SID y una ruta
   `\Device\HarddiskVolumeN`, **When** se prepara la consulta, **Then** esos fragmentos se
   sustituyen por marcadores antes de salir del proceso.
3. **Given** un suceso cuyo mensaje descriptivo contiene texto libre que el barrido no puede
   clasificar como limpio, **When** «enviar sin revisar» está desactivado, **Then** se muestra la
   pantalla de revisión con el fragmento resaltado y las tres opciones (enviar, quitar, cancelar).
4. **Given** que el suceso que originó la alerta ya no está disponible, **When** la persona pulsa
   la acción, **Then** la consulta se envía solo con el resumen estructurado y el modal indica que
   se hizo sin el contenido del suceso.

---

### User Story 3 - Explicación del detalle SMART de un disco con el volcado crudo (Priority: P2)

Además de en las alertas, la persona abre el detalle SMART de un disco y pulsa «Explícamelo en
lenguaje claro». Con este incremento, el modelo recibe también aquí el volcado técnico completo del
disco, con el mismo tratamiento que en las alertas.

**Why this priority**: comparte toda la mecánica con la US1; una vez hecha esa, extenderla al
detalle SMART es incremental. Entra en la misma versión.

**Independent Test**: abrir el detalle SMART de un disco con la función activada, pulsar la acción,
y comprobar que la consulta incluye el volcado anonimizado y que la explicación se apoya en sus
cifras.

**Acceptance Scenarios**:

1. **Given** la vista de detalle SMART de un disco y la función activada, **When** la persona pulsa
   la acción, **Then** la consulta contiene la lista de contadores SMART (como en la 005) **y** el
   volcado técnico completo del disco, anonimizado.
2. **Given** cualquier estado, **When** el volcado no puede obtenerse, **Then** aplica el mismo
   comportamiento de reserva y aviso que en la US1.

---

### User Story 4 - Modo «enviar sin revisar» para quien acepta el riesgo (Priority: P2)

Con volcados y sucesos completos, la pantalla de revisión de fragmentos dudosos aparece con
frecuencia, porque el texto libre de un suceso rara vez puede garantizarse limpio del todo. Una
persona que entiende y acepta ese riesgo activa en la configuración un modo «enviar sin revisar».
A partir de entonces, las consultas se envían con el texto ya anonimizado sin pasar por la pantalla
de revisión. La vista previa de la primera consulta se sigue mostrando.

**Why this priority**: mejora la ergonomía para un usuario avanzado, pero la función es plenamente
usable sin ella (con la revisión manual). No es imprescindible para el MVP del incremento.

**Independent Test**: en la configuración de la ayuda con IA, activar «enviar sin revisar»,
comprobar que exige una confirmación explícita con aviso de riesgo, y luego lanzar una consulta con
un fragmento dudoso y verificar que no aparece la pantalla de revisión y que el texto enviado va
anonimizado.

**Acceptance Scenarios**:

1. **Given** la configuración de la ayuda con IA, **When** la persona activa «enviar sin revisar»,
   **Then** se muestra un aviso de que fragmentos de texto libre podrían salir del equipo sin
   revisión manual y se pide confirmación; sin confirmar, la opción queda desactivada.
2. **Given** «enviar sin revisar» activado, **When** la persona lanza una consulta cuyo texto
   contiene un fragmento no clasificable, **Then** no aparece la pantalla de revisión y la consulta
   se envía con el texto anonimizado.
3. **Given** «enviar sin revisar» activado y una consulta que es la primera desde que se activó la
   función, **When** la persona pulsa la acción, **Then** la vista previa del texto a enviar se
   muestra igualmente y debe confirmarse.
4. **Given** «enviar sin revisar» activado, **When** la persona desactiva la ayuda con IA y la
   vuelve a activar, **Then** «enviar sin revisar» vuelve a estar desactivado y su aviso se pide de
   nuevo la próxima vez que se active.
5. **Given** «enviar sin revisar» desactivado (estado de fábrica), **When** una consulta contiene
   un fragmento dudoso, **Then** se muestra la pantalla de revisión, como en la 005.

---

### Edge Cases

- **Volcado disponible pero el disco no expone WWN**: se anonimiza lo que haya (número de serie) y
  se envía el resto.
- **Suceso sin mensaje descriptivo renderizado**: se envían solo los campos de datos del suceso; si
  tampoco los hay, aplica la reserva de FR-015.
- **Volcado y suceso juntos superan de largo el tamaño máximo**: se prioriza el resumen
  estructurado y el volcado; el contenido del suceso se acota primero. El modal avisa del extracto.
- **La respuesta del modelo cita un valor que no estaba en el volcado**: sigue siendo orientación,
  no dato de salud (principio I); el modal ya advierte de ello (FR-013 de la 005).
- **La persona activa «enviar sin revisar» y luego borra la clave**: la función queda inactiva; al
  volver a configurar una clave, «enviar sin revisar» está desactivado.
- **Doble pulsación de la acción**: no se lanzan dos consultas simultáneas para el mismo detalle
  (FR-020 de la 005, sin cambios).
- **El volcado contiene una etiqueta de volumen o una ruta con el perfil de usuario**: se sustituye
  por marcadores, como ya exige FR-009 de la 005.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Para una alerta cuya regla es `smart.*`, `temp.*` o `nvme.*`, el sistema DEBE incluir
  en la consulta al modelo el volcado técnico completo del disco implicado —la misma salida que la
  persona puede abrir con «Ver detalle técnico»—, además del resumen estructurado que ya se enviaba
  (regla, valor, tendencia, contexto del disco).
- **FR-002**: Para una alerta originada por un suceso del registro de eventos de Windows, el
  sistema DEBE incluir en la consulta el contenido legible del suceso que la disparó: su mensaje
  descriptivo y sus campos de datos. El sistema NO DEBE incluir los metadatos de sistema del suceso
  (identificadores internos del proveedor, identificadores de proceso e hilo, nombre del equipo,
  principal de seguridad).
- **FR-003**: En la vista de detalle SMART de un disco, el sistema DEBE incluir el volcado técnico
  completo del disco en la consulta, con el mismo tratamiento que en FR-001.
- **FR-004**: Antes de que el texto salga del proceso, el sistema DEBE sustituir por marcadores,
  además de lo que ya exige la 005 (número de serie del disco, nombre del equipo, nombre de
  usuario, rutas con perfil de usuario, etiquetas de volumen): el identificador mundial del disco
  (WWN), los identificadores de seguridad (SID) y nombres de cuenta, los nombres de otros equipos y
  las rutas internas de dispositivo del sistema. La sustitución es consistente dentro de una misma
  petición.
- **FR-005**: La marca, el modelo, el tipo/interfaz (HDD/SSD/NVMe) y la versión de firmware del
  disco DEBEN conservarse sin sustituir, coherente con FR-009 de la 005: son contexto necesario
  para la explicación y no identifican a una persona.
- **FR-006**: El sistema DEBE aplicar el barrido de fragmentos residuales (FR-026 de la 005)
  también sobre el volcado técnico y el contenido del suceso, y DEBE tratar como dudoso todo
  fragmento de texto libre que no pueda clasificar como limpio.
- **FR-007**: El sistema DEBE ofrecer en el apartado de configuración de la ayuda con IA una opción
  «enviar sin revisar», desactivada de fábrica.
- **FR-008**: Al activar «enviar sin revisar», el sistema DEBE mostrar un aviso explícito de que,
  con esta opción, fragmentos de texto libre del volcado o del suceso podrían salir del equipo sin
  una revisión manual, y DEBE requerir una confirmación. Este consentimiento es independiente y
  adicional al de la vista previa de la primera consulta (FR-010 de la 005).
- **FR-009**: Con «enviar sin revisar» activada, el sistema DEBE omitir la pantalla de revisión de
  fragmentos dudosos (FR-026 de la 005) y enviar el texto ya anonimizado. La vista previa de la
  primera consulta (FR-010 de la 005) se sigue mostrando.
- **FR-010**: Con «enviar sin revisar» desactivada (estado de fábrica), el sistema DEBE mostrar la
  pantalla de revisión siempre que el barrido encuentre un fragmento dudoso, con las tres opciones
  de la 005 (enviar tal cual, quitar el fragmento, cancelar).
- **FR-011**: El sistema DEBE guardar en la configuración un cuarto dato en el grupo de la ayuda
  con IA: el estado de «enviar sin revisar». Los otros tres no cambian: activación, modelo elegido
  y vista previa mostrada (FR-024 de la 005).
- **FR-012**: Si la función de ayuda con IA se desactiva y se vuelve a activar, «enviar sin
  revisar» DEBE volver a su estado de fábrica (desactivada), igual que la vista previa (FR-027 de
  la 005).
- **FR-013**: El sistema DEBE acotar el conjunto volcado + suceso + resumen cuando exceda el
  tamaño máximo admitido, enviar un extracto e informarlo en el modal. El tamaño máximo se eleva
  respecto al de la 005 para dar cabida a un volcado técnico completo. Al acotar, se prioriza el
  resumen estructurado y el volcado; el contenido del suceso se acota primero.
- **FR-014**: Si en el momento de la consulta el volcado técnico del disco no puede obtenerse
  (disco ausente, herramienta no disponible), el sistema DEBE continuar con el resumen estructurado
  y advertir en el modal de que la explicación se hizo sin el volcado.
- **FR-015**: Si el suceso de Windows que originó la alerta ya no está disponible o no tiene
  contenido legible, el sistema DEBE continuar con el resumen estructurado y advertir de que se
  hizo sin el contenido del suceso.
- **FR-016**: El sistema DEBE seguir sin escribir en el registro de actividad el contenido de las
  consultas ni de las respuestas (FR-022 de la 005); añadir el volcado y el contenido del suceso no
  cambia esa regla.
- **FR-017**: El contenido añadido (volcado, suceso) NO DEBE alterar ninguna decisión de la
  aplicación (FR-023 de la 005): es solo material para construir la explicación que lee la persona.
- **FR-018**: El sistema DEBE seguir enviando la consulta únicamente tras un gesto explícito de la
  persona (FR-007 de la 005); el contenido añadido no introduce ninguna recopilación adicional
  fuera de ese gesto.

### Key Entities *(include if feature involves data)*

- **Consulta de explicación (ampliada)**: efímera. Además de lo de la 005 (resumen estructurado
  con número de serie y datos de equipo/usuario ya sustituidos), ahora incluye el volcado técnico
  del disco y el contenido del suceso de Windows, ambos anonimizados. No se persiste.
- **Preferencia de la ayuda con IA (ampliada)**: cuatro datos en lugar de tres. Se añade el estado
  de «enviar sin revisar» (por defecto, desactivado).
- **Volcado técnico del disco**: la salida completa de diagnóstico del disco, obtenida en el
  momento de la consulta. No se persiste para esta función.
- **Contenido del suceso de Windows**: el mensaje descriptivo y los campos de datos del suceso que
  disparó la alerta. Se obtiene de lo que la aplicación ya guarda del suceso. No incluye los
  metadatos de sistema del suceso.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: En una muestra de alertas `smart.*`/`temp.*`/`nvme.*` reales, la explicación del
  modal cita al menos un valor concreto del disco (un atributo, un umbral, una cifra del volcado)
  en el 90 % de los casos, frente al 0 % de la implementación actual.
- **SC-002**: Sobre un conjunto de volcados y sucesos que contienen número de serie, WWN, SID,
  nombres de cuenta, nombres de equipo y rutas de dispositivo, el texto enviado al proveedor no
  contiene ninguno de esos datos en el 100 % de los casos (extiende SC-003 de la 005).
- **SC-003**: Con «enviar sin revisar» desactivada, el 100 % de las consultas cuyo texto contiene
  un fragmento no clasificable pasan por la pantalla de revisión antes de enviarse.
- **SC-004**: Con «enviar sin revisar» activada, ninguna consulta muestra la pantalla de revisión
  de fragmentos; la vista previa de la primera consulta tras activar la función sí se muestra en el
  100 % de los casos.
- **SC-005**: Activar «enviar sin revisar» exige una confirmación explícita en el 100 % de los
  intentos; sin confirmación, la opción queda desactivada.
- **SC-006**: Ante un volcado no disponible o un suceso ilegible, el 100 % de las consultas se
  completan con el resumen estructurado y el modal indica la limitación; el resto de la aplicación
  no se degrada.
- **SC-007**: El contenido de las consultas y de las respuestas —incluidos el volcado y el
  contenido del suceso— no aparece en el registro de actividad, verificado por inspección.
- **SC-008**: Tras desactivar y reactivar la ayuda con IA, «enviar sin revisar» está desactivada en
  el 100 % de los casos.

## Assumptions

- **Depende de la enmienda al principio XVI de la constitución** (versión 1.8.1 → 1.9.0), que
  autoriza enviar el volcado crudo de `smartctl` y el contenido del suceso de Windows como parte
  del detalle técnico visible, y reconoce «enviar sin revisar» como consentimiento de segundo
  nivel. La aplica la persona (el fichero está protegido por hook), con el patrón
  `parche-constitucion-*.md` ya usado en la 005. Sin la enmienda aplicada, la feature no se da por
  terminada.
- Toda la mecánica de la 005 se reutiliza sin cambios: clave en el almacén de credenciales,
  proveedor y destino únicos, anonimización en el dominio (Rust), modal en markdown seguro, vista
  previa de la primera consulta, tiempo máximo de 60 s, cero telemetría, la respuesta como
  contenido no confiable.
- El volcado técnico se obtiene en el momento de la consulta, no de un histórico: mismo patrón que
  el botón «Ver detalle técnico» de la 005 y que el detalle de sucesos. Volver a pedir la
  explicación relanza la obtención.
- El WWN se trata como dato identificable de la unidad física; marca, modelo, tipo/interfaz y
  firmware, no.
- Del suceso se toma el mensaje descriptivo renderizado y sus campos de datos; el bloque de
  metadatos de sistema del suceso se descarta por no aportar a la explicación y concentrar
  identificadores.
- El público objetivo (no técnico) y el carácter orientativo de la explicación (principio I) no
  cambian respecto a la 005.
- El modo oscuro/claro, el tamaño mínimo de ventana (1024×560) y la accesibilidad AA aplican al
  nuevo aviso de confirmación y al nuevo control de configuración, como a cualquier pantalla.
- Los límites de cuota de OpenRouter no se codifican; un volcado más grande consume más tokens y
  puede acercar antes el límite gratuito, pero el error se maneja cuando llega (FR-018 de la 005).
