# Feature Specification: Informe HTML por disco (contenido útil + resumen con IA)

**Feature Branch**: `009-informe-mejorado`

**Created**: 2026-09-10

**Status**: Draft

**Input**: User description: "Rehacer el informe HTML imprimible (pantalla de Informes, US-050) para que sea un documento útil por disco en vez de una tabla de alertas casi vacía. […] Resumen con IA por disco (opt-in, apagado de fábrica) […] una llamada al modelo por disco, nunca combinando discos […] anonimización en el dominio […] progreso y cancelación."

## Contexto

El informe HTML de la pantalla de Informes (US-050) es hoy, por disco, **solo una tabla de
alertas** con la clave de regla en crudo (`smart.error_log`, `events.disk_error`). No refleja nada
de la salud actual, los contadores SMART, la temperatura, la actividad ni los eventos de Windows,
aunque la aplicación lleve horas o días recopilando esa información. Un informe generado de un
equipo con cuatro discos y datos reales «no dice prácticamente nada».

Esta funcionalidad rehace el **formato HTML**. El CSV y el JSON siguen siendo el volcado completo
sin cambios (`docs/open-questions.md` J.30).

Además, cuando la ayuda con IA (principio XVI) esté configurada, el informe puede incluir —de forma
**opcional y explícita**— un párrafo por disco que explique en lenguaje llano sus alertas, eventos
y estado SMART. La constitución 1.11.0 amplía el principio XVI para permitir este segundo uso.

## Clarifications

### Session 2026-09-10

- Q: ¿El resumen con IA se genera para todos los discos incluidos o solo para los que tienen incidencias en el intervalo? → A: Para **todos** los discos incluidos (también los sin incidencias).
- Q: ¿Qué alertas aparecen en la sección de un disco: solo las que lo tienen como objeto, o también las de eventos de Windows sin disco asociado? → A: **Solo** las que tienen ese disco como objeto (`target_device_id`); las alertas de eventos sin objeto de disco no aparecen en el informe.
- Q: Si falla la llamada de IA de un disco a mitad de la generación, ¿continúa con el resto o se detiene? → A: **Continúa**; el disco que falla lleva su nota «resumen no disponible» y los demás conservan el suyo. Sin reintento automático (principio XVI).
- Q: Con la última lectura SMART/temperatura antigua (disco que dejó de responder), ¿el informe muestra el último valor conocido o «No disponible»? → A: El **último valor conocido con nota de antigüedad** («última lectura: hace X»); «No disponible» solo si nunca hubo lectura. Coherente con el estado «dato obsoleto» del detalle de disco.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Informe legible por disco, sin IA (Priority: P1)

Una persona que administra un equipo con varios discos exporta el informe HTML de las últimas 24
horas (o del intervalo que elija) para archivarlo, imprimirlo o enviárselo a otra persona. Abre el
fichero —también en un equipo sin conexión— y, por cada disco, ve un retrato claro: qué disco es,
cómo está de salud a día de hoy, qué contadores SMART han cambiado en el periodo, qué alertas y
qué eventos de Windows ha tenido, y cómo han evolucionado su temperatura y su actividad.

**Why this priority**: Es el valor central de la petición y no depende de la IA ni de la enmienda
constitucional. Un informe que por fin sirve para algo. Es un MVP entregable por sí solo.

**Independent Test**: Con datos recopilados de varios discos en un intervalo, exportar el HTML y
comprobar que cada sección de disco contiene identidad, salud actual, contadores SMART con su
variación, alertas con texto legible, eventos y las dos mini-gráficas; y que el fichero se abre
correctamente sin conexión.

**Acceptance Scenarios**:

1. **Given** varios discos monitorizados con historial en el intervalo, **When** se exporta el
   informe HTML, **Then** cada disco tiene una sección con: identidad (modelo/alias, tipo, bus,
   y número de serie solo si se pidió), estado de salud actual, temperatura/desgaste/horas
   actuales, volúmenes con capacidad y espacio libre, contadores SMART relevantes con valor final
   y delta del intervalo, tabla de alertas del intervalo **con frase humana**, lista de eventos de
   Windows del intervalo, y mini-gráficas de temperatura y actividad.
2. **Given** un disco sin ninguna alerta ni evento en el intervalo, **When** se exporta,
   **Then** su sección lo dice explícitamente («Sin alertas en el periodo», «Sin eventos en el
   periodo») y el resto de datos (salud, SMART, gráficas) se muestra igual.
3. **Given** un disco que **nunca** tuvo SMART (USB sin paso de comandos), **When** se exporta,
   **Then** los campos de firmware/temperatura muestran «No disponible» (nunca cero), se muestra la
   razón, y la capacidad, los eventos y la gráfica de actividad **sí** aparecen.
4. **Given** un disco que **tenía** SMART y dejó de responder, **When** se exporta, **Then** se
   muestra su **última lectura conocida con la nota «última lectura: hace X»**, no «No disponible».
5. **Given** el informe exportado, **When** se abre en un navegador **sin conexión de red**,
   **Then** se ve idéntico: sin fuentes remotas, sin imágenes externas, las mini-gráficas son SVG
   embebido.
6. **Given** el intervalo elegido no tiene muestras agregadas de una métrica (equipo recién
   instalado), **When** se exporta, **Then** la mini-gráfica correspondiente muestra su estado
   vacío («Sin muestras en el periodo»), no una línea a cero.
7. **Given** una alerta cuyo objeto es un disco, **When** aparece en el informe, **Then** su texto
   es una frase comprensible (p. ej. «Entradas nuevas en el registro de errores del disco»), no la
   clave `smart.error_log`, y conserva severidad, estado, primera y última ocurrencia y recuento.

---

### User Story 2 - Resumen en lenguaje llano por disco, con IA (Priority: P2)

Una persona con la ayuda con IA ya configurada quiere que el informe, además de los datos, le
**explique** qué significan las alertas, los eventos y el estado SMART de cada disco. Activa la
casilla «incluir resumen con IA», pulsa exportar, revisa en una vista previa el texto exacto y
anonimizado que va a salir del equipo por cada disco, confirma una sola vez, y al terminar el
informe lleva un párrafo de orientación por disco, marcado como generado por IA.

**Why this priority**: Aporta mucho valor pero depende de que la IA esté configurada, de la
enmienda al principio XVI (constitución 1.11.0) y de infraestructura de red. No es el MVP.

**Independent Test**: Con la ayuda con IA configurada y un disco con alertas/eventos en el
intervalo, activar la casilla, exportar, comprobar que aparece la vista previa con el texto
anonimizado del disco, confirmar, y comprobar que el HTML resultante incluye el párrafo de
resumen del disco marcado como IA y que la petición fue una sola por disco.

**Acceptance Scenarios**:

1. **Given** la ayuda con IA **no** configurada, **When** se abre la pantalla de Informes,
   **Then** la casilla «incluir resumen con IA» no se ofrece (o se ofrece deshabilitada con su
   motivo); el informe HTML se exporta como en la Historia 1.
2. **Given** la ayuda con IA configurada y la casilla **apagada** (estado de fábrica), **When** se
   exporta el HTML, **Then** no hay ninguna llamada al modelo y el informe es el de la Historia 1.
3. **Given** la casilla activada, **When** se pulsa exportar, **Then** antes de cualquier llamada
   de red se muestra una **vista previa** con, por cada disco incluido, el texto exacto y ya
   anonimizado que se enviará, más una explicación de a dónde va y para qué; y una **única**
   confirmación autoriza la generación de todo el informe.
4. **Given** la vista previa confirmada, **When** se genera el informe, **Then** el backend hace
   **una llamada por cada disco incluido** (nunca una petición con datos de varios discos), y cada
   llamada contiene solo: alertas del intervalo de ese disco + mensaje y campos de datos de los
   sucesos de Windows que las originaron + sus contadores SMART + el resumen numérico (mínimo,
   media, máximo, pico) de temperatura y actividad de ese intervalo.
5. **Given** un disco incluido **sin** alertas ni eventos en el intervalo y con un SMART sin
   cambios, **When** se genera el informe con resumen IA, **Then** también se hace su llamada y su
   párrafo dice, en lenguaje llano, que no hubo incidencias y cómo está el disco.
6. **Given** la respuesta del modelo para un disco, **When** se incrusta en el informe, **Then**
   se renderiza como texto/markdown seguro (nunca HTML crudo), en la sección de ese disco, con una
   marca visible de que es orientación generada por IA y su procedencia (modelo usado).
7. **Given** que durante la generación falla la red, se agota la cuota o el proveedor responde con
   error para uno o varios discos, **When** termina la exportación, **Then** el informe HTML **se
   genera igualmente** con todas las secciones de datos y, en los discos afectados, una nota de
   que el resumen con IA no pudo generarse; la exportación **no** falla entera.
8. **Given** el toggle «incluir identificadores» del informe activado, **When** se genera el
   resumen con IA, **Then** el texto enviado al modelo **sigue anonimizado** (números de serie,
   WWN, nombre de equipo, usuario, rutas de perfil, etiquetas de volumen, SID, rutas internas de
   dispositivo → marcadores); ese toggle solo afecta a las partes **no-IA** del HTML.
9. **Given** que la anonimización automática no puede garantizar limpio algún fragmento de texto
   libre residual, **When** se muestra la vista previa, **Then** ese fragmento se señala para que
   la persona lo envíe, lo quite o cancele (mismo criterio que la ayuda con IA existente y su modo
   opt-in «enviar sin revisar»).

---

### User Story 3 - Exportar con IA sin bloquear ni dejar en el limbo (Priority: P3)

Cuando el informe con resumen IA se está generando (varias llamadas de red seguidas, que pueden
tardar decenas de segundos con varios discos), la persona ve el progreso y puede cancelar sin que
la aplicación se quede colgada ni deje un fichero a medias.

**Why this priority**: Es pulido de la Historia 2. Sin ella la función es usable pero incómoda con
muchos discos.

**Independent Test**: Con la ayuda con IA configurada y varios discos, lanzar la exportación con
resumen y comprobar que aparece un indicador de progreso, que se puede cancelar a mitad, y que al
cancelar no queda un fichero de informe incompleto.

**Acceptance Scenarios**:

1. **Given** una exportación con resumen IA en curso, **When** la persona mira la pantalla,
   **Then** ve un indicador de progreso (qué disco se está resumiendo / cuántos van) y la interfaz
   sigue respondiendo.
2. **Given** una exportación con resumen IA en curso, **When** la persona cancela, **Then** la
   generación se detiene, no se escribe (o se descarta) el fichero de informe, y no queda ninguna
   petición de red en marcha.
3. **Given** una exportación **sin** resumen IA (Historia 1), **When** se pulsa exportar,
   **Then** el fichero se genera de forma prácticamente inmediata, como hasta ahora (sin cambio de
   percepción de velocidad).

---

### Edge Cases

- **Sin discos monitorizados**: el informe se genera con su cabecera y una nota de que no hay
  discos incluidos; no es un error.
- **Disco incluido sin ningún dato en el intervalo** (recién añadido): su sección muestra
  identidad y salud actual, y «Sin datos en el periodo» en el resto.
- **Muchos discos (20+)** con la casilla IA activada: la vista previa lista los 20; la generación
  hace 20 llamadas de una en una, con progreso; el aviso de que puede tardar es visible antes de
  confirmar.
- **Contador SMART sin línea base** al inicio del intervalo (no había muestra antes): el delta se
  muestra como «—» / «sin referencia», nunca como el valor absoluto disfrazado de variación.
- **Volcado de eventos muy grande** para un disco en el intervalo: la sección de eventos se acota
  a un número razonable con nota de «y N más», para que el informe no sea ilegible ni enorme.
- **Cambio de hora del sistema** dentro del intervalo: las marcas de tiempo del informe van en
  hora local y se muestran tal cual se registraron.
- **Re-exportar** el mismo informe con IA: se vuelven a hacer las llamadas (el resumen es siempre
  del momento de exportar); no hay caché.
- **Respuesta del modelo con markdown que intenta inyectar HTML/scripts**: se neutraliza; jamás se
  ejecuta ni se renderiza como HTML.
- **Anonimización deja un marcador donde iba un dato de contexto legítimo** (marca/modelo): esos
  datos —que no identifican a una persona— se conservan; solo se sustituyen los de la lista del
  principio XVI.

## Requirements *(mandatory)*

### Functional Requirements — Informe HTML (Historia 1)

- **FR-001**: El informe HTML DEBE tener, por cada disco incluido, una sección con su **identidad**:
  alias o modelo, tipo de dispositivo, bus, firmware, y número de serie **solo** si la exportación
  se pidió con identificadores.
- **FR-002**: Cada sección de disco DEBE mostrar el **estado de salud y las magnitudes actuales**
  («a fecha de generación», independientes del intervalo): estado global, temperatura actual,
  desgaste, horas de encendido; y por cada volumen del disco, su capacidad y su espacio libre. Si
  la última lectura de una magnitud es **antigua** (el disco dejó de responder), se muestra el
  **último valor conocido con una nota de antigüedad** («última lectura: hace X»); «No disponible»
  se reserva para cuando nunca hubo lectura (coherente con el estado «dato obsoleto» del detalle de
  disco).
- **FR-003**: Cada sección DEBE listar los **contadores SMART relevantes** del disco (el mismo
  conjunto que muestra el detalle de disco en la aplicación) con su **valor al final del
  intervalo** y su **variación (delta) dentro del intervalo**; sin línea base, el delta se marca
  como «sin referencia».
- **FR-004**: Cada sección DEBE listar las **alertas cuyo objeto es ese disco** (`target_device_id`)
  **y que tuvieron alguna ocurrencia dentro del intervalo**, con: descripción **en lenguaje
  legible** (no la clave de regla), severidad, estado, primera y última ocurrencia y número de
  veces. Las alertas nacidas de sucesos de Windows que **no** quedaron asociadas a ningún disco
  (p. ej. `events.filesystem_error` mientras `system_events.volume_id` no se pueble) **no** aparecen
  en el informe.
- **FR-005**: Cada sección DEBE listar los **eventos de Windows del intervalo asociados a ese
  disco**: fecha, proveedor, identificador, nivel y mensaje (el mensaje llega en el idioma del
  sistema y se marca como texto original del sistema). La lista se acota a un máximo razonable con
  indicación de cuántos se omiten.
- **FR-006**: Cada sección DEBE incluir **dos mini-gráficas embebidas** —temperatura y actividad—
  del intervalo del informe, como **SVG en el propio documento** (sin recursos remotos), con los
  huecos de datos dibujados como huecos y no como cero, y legibles también impresas en escala de
  grises.
- **FR-007**: El informe HTML DEBE ser **autónomo**: CSS embebido, sin fuentes ni imágenes
  remotas, tema claro forzado, hoja de impresión propia; DEBE abrirse igual en un equipo sin
  conexión.
- **FR-008**: El informe DEBE conservar su cabecera actual (versión de esquema, fecha de
  generación, intervalo) y DEBE seguir respetando la selección de discos (`deviceIds`), el
  intervalo y el toggle de identificadores.
- **FR-009**: Un dato ausente DEBE mostrarse como «No disponible»; nunca como cero, guion ni
  cadena vacía. Un disco no compatible con SMART **no** es un fallo: se presenta en gris, sin
  alarma.
- **FR-010**: El CSV y el JSON del informe **NO cambian**: siguen siendo el volcado completo por
  `(dispositivo, métrica, marca de tiempo)`.

### Functional Requirements — Resumen con IA (Historia 2)

- **FR-011**: La pantalla de Informes DEBE ofrecer una casilla **«incluir resumen con IA»**,
  **apagada de fábrica**, visible solo si la ayuda con IA está configurada (si no, ausente o
  deshabilitada con su motivo).
- **FR-012**: Con la casilla apagada, la exportación HTML **NO** DEBE originar ninguna llamada de
  red ni ninguna resolución de nombres.
- **FR-013**: Con la casilla activada, al exportar en HTML el sistema DEBE mostrar, **antes de
  cualquier llamada de red**, una **vista previa** con el texto exacto y **ya anonimizado** que se
  enviará **por cada disco incluido**, más una explicación de a dónde va y para qué. Una **única**
  confirmación autoriza la generación completa del informe.
- **FR-014**: Tras la confirmación, el sistema DEBE hacer **una llamada al modelo por cada disco
  incluido en el informe** —también los que no tienen ninguna alerta ni evento en el intervalo, en
  cuyo caso el resumen dirá que no hubo incidencias—; **nunca** una petición que contenga datos de
  más de un disco.
- **FR-014a**: Antes de la confirmación (FR-013), cuando el número de discos incluidos hace que la
  generación pueda tardar de forma perceptible, la vista previa DEBE **advertirlo** (número de
  llamadas y que cada una tarda unos segundos), para que la persona decida con esa información
  —especialmente si usa la clave de demostración compartida, sujeta a límites del proveedor.
- **FR-015**: El contenido enviado por disco DEBE limitarse a: alertas del intervalo de ese disco,
  el mensaje y los campos de datos de los sucesos de Windows que originaron esas alertas, sus
  contadores SMART, y el resumen numérico (mínimo, media, máximo, pico) de temperatura y actividad
  del intervalo. **Nunca** el inventario completo, la configuración, el historial de otras
  métricas ni datos de otras pantallas o de otros discos. El bloque de metadatos de sistema de los
  sucesos (nombre de equipo, principal de seguridad, identificadores de proceso) **no** se envía.
- **FR-016**: La **anonimización** DEBE aplicarse en el dominio (no en la interfaz) antes de que
  el texto salga del proceso, con las mismas reglas del principio XVI (números de serie, WWN,
  nombre de equipo, usuario, rutas de perfil, etiquetas de volumen, SID, rutas internas de
  dispositivo → marcadores, con sustitución consistente dentro de una petición). Se aplica
  **siempre**, con independencia del toggle «incluir identificadores», que solo afecta a las
  partes no-IA del HTML.
- **FR-017**: Si la anonimización automática no garantiza limpio algún fragmento de texto libre
  residual, la vista previa DEBE señalarlo para que la persona lo envíe, lo quite o cancele
  (coherente con el modo opt-in «enviar sin revisar» ya existente).
- **FR-018**: La respuesta del modelo DEBE incrustarse como **texto o markdown seguro** (jamás
  HTML crudo, jamás se ejecuta) en la sección del disco, con marca visible de «orientación
  generada por IA» y su procedencia (modelo usado). No alimenta ninguna decisión de la aplicación.
- **FR-019**: Si falla la red, se agota la cuota, se alcanza el límite de peticiones, hay tiempo de
  espera o el proveedor da error para el disco que se está resumiendo, la generación **continúa con
  el resto de discos**: ese disco lleva una **nota** («resumen con IA no disponible») y los demás
  conservan el suyo. **Sin reintento automático** (principio XVI). El informe HTML se genera
  **igualmente** con todas las secciones de datos; la exportación **no** falla entera.
- **FR-020**: El resumen con IA DEBE regenerarse en cada exportación (sin caché): siempre refleja
  el estado del momento de exportar.

### Functional Requirements — Progreso y cancelación (Historia 3)

- **FR-021**: Mientras se genera un informe **con** resumen IA, el sistema DEBE mostrar un
  indicador de progreso (disco actual / total) y la interfaz DEBE seguir respondiendo.
- **FR-022**: La persona DEBE poder **cancelar** la generación en curso; al cancelar, no se
  escribe (o se descarta) el fichero de informe y no queda ninguna petición de red activa.
- **FR-023**: Una exportación **sin** resumen IA DEBE seguir siendo prácticamente inmediata (sin
  cambio perceptible de velocidad respecto a hoy).

### Key Entities

- **Sección de disco del informe**: por cada disco incluido, agrupa su identidad, su salud actual,
  sus contadores SMART con delta, sus alertas del intervalo (con texto legible), sus eventos del
  intervalo, sus dos mini-gráficas y —si procede— su resumen con IA.
- **Frase de alerta legible**: la descripción comprensible de una regla de alerta, hoy inexistente
  en el informe (que muestra la clave cruda). Su origen (interfaz, backend o tabla compartida) se
  decide en el plan.
- **Payload de resumen por disco**: el conjunto acotado y anonimizado —alertas + contexto de
  sucesos + SMART + resumen numérico de temperatura y actividad— de **un** disco, que se muestra
  en la vista previa y se envía al modelo.
- **Vista previa de envío**: la lista, por disco, del texto exacto anonimizado que saldrá del
  equipo, con lo que la anonimización no pudo garantizar limpio señalado; una confirmación cubre
  todo el informe.
- **Resumen con IA de un disco**: el texto de orientación devuelto por el modelo para ese disco,
  con su procedencia, o la nota de que no pudo generarse.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: En un informe de un equipo con varios discos y datos reales del intervalo, cada
  sección de disco contiene al menos: identidad, estado de salud actual, contadores SMART con
  variación, alertas (o «sin alertas»), eventos (o «sin eventos») y las dos mini-gráficas. Hoy
  contiene solo la tabla de alertas.
- **SC-002**: Ninguna alerta del informe aparece como clave de regla cruda; el 100 % se muestra
  con una frase legible.
- **SC-003**: El informe exportado se abre y se ve completo en un navegador **sin conexión**, sin
  ningún recurso externo (comprobable: cero peticiones de red al abrirlo).
- **SC-004**: Con la casilla «incluir resumen con IA» **apagada**, exportar un informe HTML no
  genera **ninguna** petición de red (comprobable con un monitor de red).
- **SC-005**: Con la casilla activada, la persona ve el texto exacto que se va a enviar de **cada**
  disco antes de que salga nada del equipo, y una sola confirmación cubre el informe entero.
- **SC-006**: Cada llamada al modelo contiene datos de **un solo** disco; nunca de dos o más.
- **SC-007**: Si el proveedor de IA está caído, el informe HTML se genera igualmente con todas las
  secciones de datos y una nota por disco; la exportación no falla.
- **SC-008**: Una exportación **sin** resumen IA sigue completándose en el mismo orden de tiempo
  que hoy (percepción de «inmediato»).
- **SC-009**: Una exportación **con** resumen IA se puede cancelar a mitad y no deja un fichero de
  informe incompleto.

## Assumptions

- **Discos incluidos**: por defecto, los discos monitorizados (mismo criterio que el CSV/JSON,
  `deviceIds: null`), o los que la persona seleccione. Los discos excluidos por la persona no
  entran, igual que hoy.
- **Contadores SMART «relevantes»**: el mismo conjunto que ya muestra el detalle de disco en la
  aplicación; no se define aquí una lista nueva.
- **Delta del intervalo**: valor de la última muestra dentro del intervalo menos el valor de la
  última muestra en o antes del inicio del intervalo. Sin muestra base: «sin referencia».
- **Mini-gráficas**: una línea de temperatura y una de actividad sobre el intervalo pedido, con la
  misma semántica de huecos que las gráficas de la aplicación; su resolución sigue la
  correspondencia intervalo→resolución ya existente (`open-questions.md` E.1) y el informe lo
  declara.
- **Magnitudes «actuales»**: se toman del último estado conocido del disco al generar el informe,
  y se etiquetan como tal para no confundirlas con datos del intervalo.
- **Mecanismo de las frases legibles de alerta**: el spec exige texto legible; **cómo** se resuelve
  el que hoy el informe lo genere una capa sin diccionarios de traducción (ADR-030) se decide en
  el plan (interfaz genera el HTML / la interfaz pasa un mapa de textos / diccionario mínimo
  duplicado). El dueño delegó esta decisión en quien implemente.
- **Resumen numérico de actividad**: usa la serie `activity_percent` ya persistida (ADR-050/056).
  Si esa serie está vacía en el intervalo, el resumen de actividad se marca «sin datos» y ese
  hecho viaja al modelo como tal.
- **Disco «sin incidencias»**: para el modelo el payload incluye igualmente sus contadores SMART y
  el resumen numérico, aunque no haya alertas ni eventos; el modelo dispone de contexto para decir
  algo útil, no solo «no hay nada».
- **Un solo informe por exportación**: el flujo de vista previa + confirmación + progreso es para
  esa exportación; no hay cola de informes.
- **Idioma del informe**: el de la aplicación (como hoy). El resumen con IA se pide en ese idioma.
- **Dependencias**: se reutiliza toda la infraestructura de la ayuda con IA existente (cliente de
  red de proveedor único, clave en el almacén de credenciales, anonimización en el dominio, flujo
  de vista previa y degradación de las specs 005/006). No se añaden dependencias.
- **Alcance del resumen con IA**: se genera para **todos** los discos incluidos en el informe,
  también los que no tienen incidencias en el intervalo (decisión del dueño). La vista previa
  advierte del número de llamadas y del tiempo cuando hay muchos discos.
- **Constitución**: esta funcionalidad requiere la enmienda al principio XVI (1.11.0), **ya
  aprobada y aplicada**. Requiere **ADR-057** para el flujo del resumen en el informe.
