# Especificación de funcionalidad: SmartDisk Monitor 1.0

**Directorio de la funcionalidad**: `specs/001-monitor-discos-windows`

**Rama**: `main` (no se creó rama: no hay extensión de git registrada en `.specify/extensions.yml`)

**Creada**: 2026-09-04

**Estado**: Borrador

**Entrada**: Documentación consolidada del proyecto (`historias.md`, 31 ficheros normativos):
especificación de producto, historias de usuario, arquitectura, modelo de datos, reglas de alerta,
contrato entre interfaz y backend, convenciones de ingeniería, estrategia de testing, decisiones
técnicas, cuestiones abiertas y el sistema de diseño vinculante.

> **Nota sobre el origen de esta especificación.** No describe una funcionalidad nueva: reformula en
> lenguaje de producto el alcance de la versión 1.0, que ya estaba escrito y decidido. Ante cualquier
> discrepancia manda el documento normativo original, en el orden de autoridad de `AGENTS.md`:
> constitución → `docs/ui-design.md` → `docs/alert-rules.md` y `docs/ui-contract.md` →
> `docs/open-questions.md` → resto de `docs/`. Su utilidad es dar a las fases siguientes
> (`/speckit-plan`, `/speckit-tasks`) unas historias priorizadas e independientemente entregables.

## Aclaraciones

### Sesión 2026-09-04

- P: ¿Qué umbral concreto convierte «sin bloqueo perceptible» en algo que una prueba pueda medir?
  (SC-007, SC-009) → R: La interfaz nunca deja de responder más de 50 ms seguidos, y toda acción del
  usuario produce respuesta visible en menos de 100 ms (modelo RAIL). Cierra la cuestión I.7 de
  `docs/open-questions.md`.
- P: ¿Qué debe hacer la aplicación cuando el volumen donde guarda su propio historial se queda sin
  espacio libre? → R: Avisar al bajar de un umbral de espacio libre y **detener la escritura de
  historial**, siguiendo con la monitorización y las alertas en vivo. Nunca se purgan datos por
  iniciativa propia para hacer sitio.
- P: ¿Debe el administrador poder consultar desde la propia aplicación el registro de actividad
  interno, y activar un modo detallado cuando algo falla? → R: Sí, con alcance acotado: un
  interruptor de modo detallado en Ajustes y un botón que abre la carpeta del registro. Sin visor
  propio dentro de la aplicación. El contenido va además en el paquete de diagnóstico.

## Escenarios de usuario y pruebas *(obligatorio)*

El usuario es **uno solo**: el administrador del equipo. No hay cuentas, roles ni acceso remoto. Las
historias van ordenadas por valor entregado, y cada una es un producto utilizable por sí misma.

### Historia 1 — Saber qué discos hay y cómo están (Prioridad: P1)

El administrador abre la aplicación por primera vez, acepta la elevación de privilegios y ve
inmediatamente qué dispositivos de almacenamiento tiene el equipo, cuáles puede vigilar y en qué
estado se encuentran. Puede excluir los que no le interesan y ponerles un nombre reconocible.

**Por qué esta prioridad**: es el producto mínimo. Sin inventario y estado no hay nada más que
construir, y responde por sí sola a la pregunta que trae al usuario: *¿están bien mis discos?*

**Prueba independiente**: se instala en un equipo con discos de distintos tipos, se abre y se
comprueba que cada dispositivo aparece con su estado y su procedencia, sin haber configurado nada.

**Escenarios de aceptación**:

1. **Dado** un equipo con discos internos y uno externo, **cuando** se abre la aplicación por primera
   vez, **entonces** el asistente inicial los lista todos, indica de cuáles puede leer datos de salud
   y deja todos los compatibles seleccionados.
2. **Dado** un dispositivo del que no se pueden leer datos de salud (conectado por USB, tras una
   controladora RAID o virtual), **cuando** se muestra su ficha, **entonces** aparece en gris como
   «Sin datos de salud», **nunca** en rojo y sin generar ninguna alerta.
3. **Dado** un dato que la fuente no ha entregado, **cuando** se presenta la métrica, **entonces**
   dice «No disponible» y jamás un cero, un guion ni una casilla vacía.
4. **Dado** un disco monitorizado, **cuando** se consulta cualquiera de sus métricas, **entonces**
   puede verse de qué fuente procede y cuándo se leyó por última vez; si el dato está caduco, se
   dice.
5. **Dado** un disco que se conecta con la aplicación abierta, **cuando** pasa un minuto, **entonces**
   aparece en la lista sin reiniciar y sin volver a mostrar el asistente.
6. **Dado** un disco monitorizado que se retira **con** expulsión segura, **cuando** desaparece,
   **entonces** se marca como retirado y se registra un apunte de inventario, sin alerta.
7. **Dado** ese mismo disco retirado **sin** aviso, **cuando** desaparece, **entonces** se genera una
   alerta crítica —o de advertencia si es un dispositivo extraíble—.
8. **Dado** un disco retirado que vuelve a conectarse, **cuando** se reconoce, **entonces** continúa
   su historial anterior; si la identidad solo pudo deducirse por huella, se indica como inferida.

---

### Historia 2 — Enterarse de un problema sin estar mirando (Prioridad: P1)

El administrador no vigila la pantalla. Quiere que la aplicación le avise cuando algo se sale de lo
normal, con un aviso por problema y no uno por medición, y poder reconocerlo, silenciarlo o
archivarlo sin que el sistema le mienta sobre el estado real del hardware.

**Por qué esta prioridad**: un monitor que solo informa cuando lo abres no es un monitor. Es la
diferencia entre enterarse de un fallo de disco antes o después de perder los datos.

**Prueba independiente**: se fuerza una condición de alerta —temperatura o umbral de capacidad— y se
comprueba que aparece un único grupo con su contador de repeticiones, que el icono de la bandeja
cambia de color y que reconocerla no altera el color del disco.

**Escenarios de aceptación**:

1. **Dado** un umbral cruzado repetidamente, **cuando** se consultan las alertas, **entonces** hay
   **un** grupo con contador de ocurrencias y su cronología, no una fila por muestra.
2. **Dado** un grupo de alertas activo, **cuando** el administrador lo reconoce, **entonces** sale de
   la lista de pendientes y queda distinguido, pero **el color del disco no cambia**: seguirá
   reflejando la peor alerta no resuelta.
3. **Dado** un grupo de alertas, **cuando** se silencia por 15 minutos, 1 hora, 8 horas o
   indefinidamente, **entonces** deja de notificar, y **el color sigue sin cambiar**.
4. **Dado** que la condición que originó la alerta desaparece, **cuando** se vuelve a evaluar,
   **entonces** el grupo se marca como resuelto automáticamente y deja de contar para el estado.
5. **Dado** el estado global del equipo, **cuando** se mira el icono de la bandeja, **entonces** es
   verde si todo está correcto, ámbar si hay advertencias, rojo si hay algo crítico activo y gris si
   la monitorización está pausada, sin datos o con fallo de recopilación.
6. **Dado** un disco monitorizado del que no llegan lecturas frescas, **cuando** se calcula su estado,
   **entonces** es **desconocido**, no correcto: no saber que algo está bien no es saber que está bien.
7. **Dado** un aviso de capacidad, **cuando** se muestra, **entonces** no interrumpe el trabajo del
   usuario ni compite en severidad con un fallo de hardware.

---

### Historia 3 — Entender cómo ha evolucionado (Prioridad: P2)

El administrador ve una temperatura alta y necesita saber si es de hoy o lleva un mes subiendo. Puede
consultar la evolución de cada métrica en el intervalo que elija.

**Por qué esta prioridad**: convierte un dato puntual en un diagnóstico. Es lo que distingue «hace
calor» de «este disco se está muriendo». No es imprescindible para el primer valor entregado, pero sí
para actuar con criterio.

**Prueba independiente**: tras dejar la aplicación recopilando unas horas, se abre una gráfica y se
comprueba que el eje temporal cubre el intervalo pedido, que los períodos sin datos aparecen como
huecos y que el pie declara la resolución mostrada.

**Escenarios de aceptación**:

1. **Dado** un intervalo de 24 horas, 7 días, 30 días o personalizado, **cuando** se pide la gráfica,
   **entonces** el eje horizontal cubre **el intervalo pedido completo**, aunque falten datos en los
   extremos.
2. **Dado** un período sin lecturas, **cuando** se dibuja la serie, **entonces** aparece como hueco y
   **nunca** se interpola una línea que sugiera una continuidad que no existe.
3. **Dado** que se muestran promedios en vez de muestras crudas, **cuando** se lee la gráfica,
   **entonces** el pie declara la resolución: un máximo promediado no es un pico.
4. **Dado** un umbral declarado por el fabricante, **cuando** existe para esa métrica, **entonces** se
   dibuja de forma distinguible de los datos medidos.
5. **Dado** el historial acumulado, **cuando** se supera el período de retención configurado,
   **entonces** los datos antiguos se compactan o se eliminan según lo previsto, y el usuario puede
   ver cuánto espacio ocupa.

---

### Historia 4 — Relacionar un problema con lo que registró Windows (Prioridad: P2)

Cuando algo va mal, el administrador quiere ver los eventos que el propio sistema operativo registró
sobre el almacenamiento, y a qué disco corresponde cada uno.

**Por qué esta prioridad**: aporta la evidencia que las métricas no dan —errores de controlador,
reintentos, sectores— y es lo que se adjunta a un parte de incidencia.

**Prueba independiente**: se consultan los eventos de almacenamiento del sistema y se comprueba que
se listan con su nivel y su origen, y que los que no pudieron atribuirse con certeza a un disco
aparecen marcados como asociación inferida.

**Escenarios de aceptación**:

1. **Dado** un evento de almacenamiento registrado por el sistema, **cuando** se muestra, **entonces**
   lleva su marca de tiempo, su nivel y su origen, y se puede filtrar por disco, volumen, nivel y
   proveedor.
2. **Dado** un evento cuya atribución a un disco concreto se ha deducido y no consta con certeza,
   **cuando** se muestra, **entonces** va **etiquetado como asociación inferida**.
3. **Dado** el texto original de un evento, **cuando** se presenta en pantalla, **entonces** se
   muestra como texto plano y nunca se interpreta como contenido con formato.
4. **Dado** que la aplicación se cierra y se vuelve a abrir, **cuando** relee los eventos,
   **entonces** no duplica los ya registrados.
5. **Dado** un equipo con miles de eventos, **cuando** se recorre la lista, **entonces** el
   desplazamiento se mantiene dentro de los umbrales de SC-007.

---

### Historia 5 — Comprobar un disco bajo demanda (Prioridad: P2)

Ante una sospecha, el administrador quiere ejecutar una prueba concreta —rendimiento, integridad del
sistema de archivos o autodiagnóstico del propio dispositivo— sabiendo exactamente qué va a ocurrir
antes de lanzarla.

**Por qué esta prioridad**: cierra el bucle entre observar y actuar. Es también la parte con más
capacidad de hacer daño, y por eso exige confirmación explícita.

**Prueba independiente**: se lanza cada prueba sobre un disco de descarte y se comprueba que antes se
muestra su impacto, que se puede cancelar, que la aplicación no se bloquea mientras corre y que el
resultado queda en el historial.

**Escenarios de aceptación**:

1. **Dado** que se va a lanzar una prueba que escribe datos o genera carga, **cuando** se pulsa
   iniciar, **entonces** se pide confirmación declarando acción, destino, impacto —rendimiento,
   temperatura y escrituras sobre el dispositivo— y la orden literal que se ejecutará, si existe.
2. **Dado** una prueba en curso, **cuando** se observa la pantalla, **entonces** muestra progreso y
   tiempo restante, la aplicación sigue respondiendo y la prueba se puede cancelar.
3. **Dado** que durante una prueba se supera un límite de seguridad de temperatura, **cuando** se
   detecta, **entonces** la prueba se detiene sola y se avisa de por qué.
4. **Dado** un dispositivo que no admite autodiagnóstico, **cuando** se abre la pantalla de pruebas,
   **entonces** la opción aparece deshabilitada **con el motivo escrito**, no simplemente apagada.
5. **Dado** que una prueba termina, **cuando** se consulta el historial, **entonces** consta con su
   resultado, su duración y el disco sobre el que se ejecutó.
6. **Dado** la salida literal de una comprobación del sistema de archivos, **cuando** se muestra,
   **entonces** se presenta como texto tal cual, con posibilidad de copiarla.

---

### Historia 6 — Llevarse la información fuera (Prioridad: P3)

El administrador necesita adjuntar datos a una garantía, a un parte de incidencia o a una consulta
con un tercero, sin entregar de paso información que identifique su equipo.

**Por qué esta prioridad**: es valor real, pero llega después de que exista algo que exportar.

**Prueba independiente**: se exporta un informe en cada formato y un paquete de diagnóstico, y se
comprueba que el contenido corresponde al intervalo pedido y que el paquete va anonimizado salvo que
se pida lo contrario.

**Escenarios de aceptación**:

1. **Dado** un intervalo y un conjunto de discos, **cuando** se exporta, **entonces** se obtiene el
   fichero en el formato elegido —tabular, estructurado o imprimible— con exactamente esos datos.
2. **Dado** que se genera un paquete de diagnóstico, **cuando** se crea, **entonces** va
   **anonimizado por defecto**: sin números de serie, nombres de equipo, rutas de usuario ni nada que
   identifique a una persona.
3. **Dado** que el usuario decide incluir los datos identificativos, **cuando** lo solicita
   explícitamente, **entonces** se le advierte de qué va a contener antes de generarlo.
4. **Dado** que una exportación tarda, **cuando** está en curso, **entonces** la aplicación sigue
   respondiendo y muestra progreso.

---

### Historia 7 — Ajustar el comportamiento a su equipo (Prioridad: P3)

Cada equipo es distinto: un servidor con veinte discos y un portátil con uno no quieren la misma
frecuencia de muestreo ni los mismos umbrales. El administrador puede ajustarlo, y también borrar
todo lo acumulado.

**Por qué esta prioridad**: hay valores por defecto razonables, así que el producto funciona sin
tocar nada. La configuración amplía el número de equipos donde el producto es adecuado.

**Prueba independiente**: se cambia cada preferencia, se cierra y se reabre la aplicación, y se
comprueba que se conservó y que un valor fuera de rango se rechaza con una explicación.

**Escenarios de aceptación**:

1. **Dado** un valor de frecuencia o umbral fuera de los límites permitidos, **cuando** se intenta
   guardar, **entonces** se rechaza con una explicación comprensible y el valor anterior se mantiene.
2. **Dado** un cambio de idioma o de tema, **cuando** se aplica, **entonces** surte efecto
   inmediatamente y sobrevive al reinicio de la aplicación.
3. **Dado** que el equipo pasa a funcionar con batería, **cuando** cambia la alimentación,
   **entonces** se espacian las mediciones no críticas, **pero no** las que alimentan las alertas
   graves; al volver a la red se restablecen y se fuerza una lectura completa.
4. **Dado** que el usuario pausa la monitorización, **cuando** la aplicación se reinicia,
   **entonces** **siempre reanuda**: una pausa olvidada es un monitor que no vigila y no lo dice.
5. **Dado** que se pide borrar todos los datos, **cuando** se confirma explícitamente, **entonces** se
   elimina el historial y se dice qué se ha borrado.
6. **Dado** un fallo que hay que reproducir, **cuando** se activa el modo detallado en Ajustes,
   **entonces** el registro pasa a recoger más información, y una acción permite abrir su carpeta.
7. **Dado** que se cierra la ventana con el aspa, **cuando** ocurre, **entonces** se pregunta si
   minimizar o salir, con opción de recordar la decisión.

---

### Historia 8 — Instalar, reconocer y retirar la aplicación (Prioridad: P3)

El administrador instala la aplicación en un equipo sin conexión, comprueba qué versión tiene y puede
desinstalarla sabiendo qué se queda y qué se va.

**Por qué esta prioridad**: imprescindible para entregar, pero no aporta valor de uso por sí misma.

**Prueba independiente**: se instala en un equipo limpio y sin red, se arranca, se consulta la
información de versión y licencias, y se desinstala comprobando qué queda.

**Escenarios de aceptación**:

1. **Dado** un equipo sin conexión a internet, **cuando** se ejecuta el instalador, **entonces**
   instala todo lo necesario para funcionar sin descargar nada.
2. **Dado** que la aplicación necesita privilegios elevados, **cuando** se inicia, **entonces** los
   solicita y, si no los obtiene, **no arranca a medias**: se explica por qué en vez de quedar en un
   estado parcialmente funcional.
3. **Dado** que ya hay una instancia en ejecución, **cuando** se intenta abrir otra, **entonces** se
   trae al frente la existente en lugar de abrir una segunda.
4. **Dado** que se consulta la información de la aplicación, **cuando** se abre, **entonces** muestra
   versión, licencia propia y avisos de los componentes de terceros redistribuidos.
5. **Dado** que se desinstala, **cuando** termina, **entonces** los datos históricos y la
   configuración **se conservan**, y se explica cómo eliminarlos manualmente.
6. **Dado** un cierre inesperado con datos a medio escribir, **cuando** se vuelve a abrir,
   **entonces** arranca con el historial íntegro y sin quedar en un estado a medias.

---

### Casos límite

- **Un dispositivo del que no se puede leer nada.** Se presenta como no compatible y en gris. Nunca
  se interpreta el silencio de una fuente como un valor correcto ni como una avería.
- **Una fuente de datos que deja de responder.** Degrada su propia tarjeta y lo dice; no bloquea ni
  tumba el resto de la aplicación.
- **El mismo disco visto por dos fuentes con datos distintos.** Se muestra el dato con su procedencia
  para que la discrepancia sea visible, en vez de elegir en silencio.
- **Veinte o más discos.** El panel se reorganiza y prioriza los que no están correctos; la lista
  lateral se desplaza sin arrastrar consigo la navegación ni el estado global.
- **Miles de eventos registrados.** La lista sigue siendo utilizable al desplazarse, dentro del
  umbral de SC-007.
- **Una alerta reconocida sobre un problema que persiste.** El color del disco sigue reflejando el
  problema. Reconocer no repara.
- **El equipo se suspende o hiberna.** Al despertar hay un hueco en las series; se dibuja como hueco.
- **Cambio de firmware de un disco.** No parte el historial: se registra como un apunte sobre el
  mismo dispositivo.
- **Espacio en disco agotado para el propio historial.** Se avisa al cruzar el umbral de aviso y se
  detiene la escritura al llegar al de parada, diciéndolo. La vigilancia y las alertas siguen
  funcionando: se deja de acumular pasado, no de mirar el presente. Nada se borra solo para hacer
  sitio.
- **La ventana en su tamaño mínimo, o con el sistema al 200 % de escala.** Nada se recorta en
  silencio: si no cabe, se desplaza.

## Requisitos *(obligatorio)*

### Requisitos funcionales

**Inventario y estado**

- **FR-001**: El sistema DEBE descubrir automáticamente los discos físicos, particiones y volúmenes
  del equipo, y mantener esa lista al día ante altas y bajas en caliente en menos de un minuto.
- **FR-002**: El sistema DEBE permitir al usuario excluir dispositivos de la monitorización y
  asignarles un alias reconocible.
- **FR-003**: El sistema DEBE distinguir cinco estados —correcto, advertencia, crítico, desconocido y
  no compatible— y NO DEBE representar «no compatible» ni «sin datos» como avería.
- **FR-004**: El sistema DEBE mostrar «No disponible» ante cualquier dato ausente, y NO DEBE sustituir
  un dato que falta por un cero, un guion o una cadena vacía.
- **FR-005**: El sistema DEBE poder indicar, por cada métrica, su procedencia y el momento de la
  última lectura válida, y señalar cuándo el dato está caduco.
- **FR-006**: El sistema DEBE tratar como **desconocido** el estado de un dispositivo monitorizado del
  que no haya lecturas frescas.
- **FR-007**: El sistema DEBE conservar el historial de un dispositivo retirado, y reanudarlo si el
  mismo dispositivo vuelve a conectarse, señalando cuándo la identidad se ha deducido por huella.

**Alertas**

- **FR-008**: El sistema DEBE agrupar las alertas por problema, con contador de ocurrencias y
  cronología, y NO DEBE emitir una alerta por cada medición.
- **FR-009**: El sistema DEBE permitir reconocer, silenciar —15 min, 1 h, 8 h o indefinidamente— y
  archivar un grupo de alertas.
- **FR-010**: El estado de salud de un dispositivo DEBE reflejar la peor alerta **activa o
  reconocida**; reconocer o silenciar NO DEBE alterar ese estado.
- **FR-011**: El sistema DEBE resolver automáticamente un grupo de alertas cuando la condición que lo
  originó deja de cumplirse.
- **FR-012**: El sistema DEBE reflejar el estado global en el área de notificación con cuatro
  estados: correcto, advertencia, crítico y sin vigilancia, y permitir desde ahí abrir la aplicación,
  ver un resumen, pausar o reanudar y salir.
- **FR-013**: Los avisos de capacidad NO DEBEN interrumpir al usuario ni equipararse en severidad a un
  fallo de hardware.

**Historial y eventos**

- **FR-014**: El sistema DEBE registrar la evolución de las métricas y presentarla sobre un eje
  temporal que cubra el intervalo solicitado por completo, aunque falten datos.
- **FR-015**: El sistema DEBE representar los períodos sin lecturas como huecos, y NO DEBE interpolar
  valores inexistentes.
- **FR-016**: Toda representación de una serie DEBE declarar su resolución cuando muestre valores
  agregados en lugar de mediciones directas.
- **FR-017**: El sistema DEBE recoger los eventos de almacenamiento registrados por el sistema
  operativo sin duplicarlos entre ejecuciones, y permitir filtrarlos por dispositivo, volumen, nivel
  y origen.
- **FR-018**: El sistema DEBE etiquetar como **inferida** toda asociación entre un evento y un
  dispositivo que no conste con certeza.
- **FR-019**: El sistema DEBE presentar como texto plano todo contenido procedente de un dispositivo o
  del registro de eventos, y NUNCA interpretarlo como contenido con formato.
- **FR-020**: El sistema DEBE aplicar una política de retención configurable, informar del espacio
  ocupado y permitir el borrado explícito y confirmado de todo el historial.
- **FR-020a**: El sistema DEBE avisar cuando el espacio libre del volumen donde guarda su historial
  baje del umbral de aviso, y **detener la escritura de historial** al alcanzar el umbral de parada,
  continuando con la monitorización, la evaluación de reglas y las alertas en vivo.
- **FR-020b**: Mientras la escritura de historial esté detenida, el sistema DEBE indicarlo de forma
  visible y NO DEBE presentar el historial resultante como si fuera continuo. Un monitor que ha
  dejado de registrar y no lo dice es peor que uno parado.
- **FR-020c**: El sistema NUNCA DEBE eliminar ni compactar datos por iniciativa propia para ganar
  espacio. La retención configurada y el borrado explícito son las únicas vías por las que
  desaparecen datos.

**Pruebas y diagnóstico**

- **FR-021**: El sistema DEBE ofrecer, bajo demanda, una prueba de rendimiento, una comprobación del
  sistema de archivos y un autodiagnóstico corto del dispositivo cuando este lo admita.
- **FR-022**: Toda acción que escriba datos o genere carga DEBE confirmarse previamente declarando
  acción, destino, impacto y la orden literal que se ejecutará cuando exista.
- **FR-023**: Una prueba en curso DEBE mostrar progreso y tiempo restante, poder cancelarse y NO DEBE
  bloquear la interfaz.
- **FR-024**: El sistema DEBE detener automáticamente una prueba que supere un límite de seguridad de
  temperatura, y explicar por qué la detuvo.
- **FR-025**: Todo control deshabilitado DEBE indicar el motivo por el que lo está.
- **FR-026**: El sistema DEBE conservar el historial de pruebas ejecutadas con su resultado, duración
  y dispositivo.

**Exportación**

- **FR-027**: El sistema DEBE exportar informes del intervalo y los dispositivos elegidos en formato
  tabular, estructurado e imprimible.
- **FR-028**: El sistema DEBE generar un paquete de diagnóstico **anonimizado por defecto**, y advertir
  explícitamente del contenido antes de generar uno que incluya datos identificativos.

**Configuración y ciclo de vida**

- **FR-029**: El sistema DEBE permitir configurar frecuencias de muestreo, umbrales, retención,
  idioma, tema y comportamiento al cerrar la ventana, validando todo valor contra sus límites y
  rechazando con explicación los que se salgan.
- **FR-029a**: El sistema DEBE ofrecer un **modo detallado** de registro de actividad, activable y
  desactivable por el usuario, para poder reproducir un fallo con más información.
- **FR-029b**: El sistema DEBE ofrecer una acción que **abra la carpeta** donde reside el registro de
  actividad, y NO DEBE incluir un visor de registro dentro de la aplicación.
- **FR-029c**: El registro de actividad DEBE incluirse en el paquete de diagnóstico, sujeto a la
  misma anonimización que el resto de su contenido (FR-028).
- **FR-030**: El sistema DEBE espaciar las mediciones no críticas al funcionar con batería, SIN
  alterar las que alimentan las alertas graves, y restablecerlas al volver a la red eléctrica.
- **FR-031**: El sistema DEBE reanudar siempre la monitorización al arrancar, aunque se hubiera
  quedado pausada.
- **FR-032**: El sistema DEBE solicitar privilegios elevados al iniciarse y, si no los obtiene, NO
  DEBE continuar en un estado parcialmente funcional. Cuando el sistema operativo impida la
  elevación, DEBE mostrarse una explicación comprensible.
- **FR-033**: El sistema DEBE impedir una segunda instancia simultánea, trayendo al frente la
  existente.
- **FR-034**: El sistema DEBE instalarse sin requerir conexión a internet, y conservar historial y
  configuración al desinstalarse, explicando cómo eliminarlos.
- **FR-035**: El sistema DEBE recuperarse de un cierre inesperado sin pérdida de integridad del
  historial.

**Presentación, idioma y accesibilidad**

- **FR-036**: Toda la interfaz DEBE estar disponible en español e inglés, seleccionando el idioma
  inicial según el del sistema, sin ningún texto visible fuera de los diccionarios.
- **FR-037**: Toda la interfaz DEBE ser correcta en tema claro y oscuro, y seguir el tema del sistema
  cuando así se configure.
- **FR-038**: El color NUNCA DEBE ser el único portador de significado: siempre acompañado de texto o
  icono.
- **FR-039**: La ventana NUNCA DEBE recortar contenido en silencio: si no cabe, la región se desplaza.
- **FR-040**: Toda la aplicación DEBE ser operable por teclado y cumplir el nivel AA de las pautas de
  accesibilidad vigentes, incluido el contraste en ambos temas.
- **FR-041**: Todo fallo mostrado al usuario DEBE llevar una explicación comprensible **y** conservar
  el detalle técnico consultable.
- **FR-042**: El sistema NO DEBE enviar dato alguno fuera del equipo ni requerir conexión para
  funcionar.

### Entidades principales

- **Dispositivo**: un disco físico. Identidad estable, modelo, tipo de conexión, capacidad, si está
  monitorizado, alias, estado de compatibilidad y si sigue presente.
- **Volumen**: unidad lógica con letra o punto de montaje, sistema de archivos, capacidad y espacio
  libre. Se relaciona con uno o varios dispositivos.
- **Muestra de métrica**: un valor medido de una magnitud, con su momento, su unidad y su procedencia.
- **Instantánea de salud**: el conjunto de contadores de estado leídos de un dispositivo en un momento
  dado.
- **Evento del sistema**: un registro del sistema operativo relativo al almacenamiento, con su
  momento, nivel, origen y el grado de certeza de su asociación a un dispositivo.
- **Grupo de alertas**: un problema detectado, con su severidad, su estado —activa, reconocida,
  resuelta, archivada—, su silencio si lo tiene y el recuento de ocurrencias.
- **Ocurrencia de alerta**: cada vez que la condición de un grupo volvió a cumplirse.
- **Ejecución de prueba**: una prueba lanzada por el usuario, con tipo, dispositivo, resultado,
  duración y salida.
- **Preferencia**: un ajuste del usuario, con su valor tipado y sus límites válidos.
- **Marcador de lectura de eventos**: la posición hasta la que ya se leyó el registro del sistema, para
  no duplicar entre ejecuciones.

## Criterios de éxito *(obligatorio)*

### Resultados medibles

- **SC-001**: Desde una instalación limpia, el usuario ve el estado de todos sus discos en **menos de
  3 minutos**, incluyendo la instalación y el asistente inicial.
- **SC-002**: El **100 %** de las métricas ausentes se presenta como «No disponible». Ni un solo cero
  inventado, verificable revisando cualquier dispositivo no compatible.
- **SC-003**: Ningún dispositivo sin datos de salud se presenta jamás como averiado ni genera alerta:
  **cero** falsos positivos por incompatibilidad.
- **SC-004**: Una condición anómala que se repite **N** veces produce **un** aviso agrupado con
  contador N, no N avisos.
- **SC-005**: Un disco conectado o retirado en caliente se refleja en la aplicación en **menos de 60
  segundos**, sin reiniciarla.
- **SC-006**: Reconocer o silenciar una alerta **nunca** cambia el color del disco mientras el
  problema persista: comprobable en el 100 % de los casos.
- **SC-007**: La aplicación sigue respondiendo durante recopilaciones, exportaciones y pruebas: la
  interfaz **nunca deja de responder durante más de 50 ms seguidos**, y **toda acción del usuario
  produce una respuesta visible en menos de 100 ms**.
- **SC-008**: El fallo de una fuente de datos degrada **solo** su propia tarjeta; el resto de la
  aplicación sigue funcionando.
- **SC-009**: Con **20 discos** monitorizados y **5.000 eventos** registrados, la navegación y el
  desplazamiento se mantienen dentro de los umbrales de SC-007.
- **SC-010**: El **100 %** de la interfaz está disponible en español e inglés, sin texto sin traducir,
  incluidos los nombres accesibles de gráficas e iconos.
- **SC-011**: El **100 %** de las pantallas cumple el nivel AA de contraste en ambos temas y es
  operable íntegramente por teclado.
- **SC-012**: Toda acción que escriba datos o genere carga se confirma antes: **cero** acciones de ese
  tipo ejecutables sin confirmación explícita.
- **SC-013**: El paquete de diagnóstico generado por defecto **no contiene** ningún dato que
  identifique al equipo o a la persona.
- **SC-014**: La aplicación funciona **sin ninguna conexión de red**, tanto al instalarse como al
  usarse: cero peticiones salientes.
- **SC-015**: Tras un cierre inesperado, la aplicación arranca con el historial íntegro en el **100 %**
  de los casos.
- **SC-015a**: Con el volumen del historial por debajo del umbral de parada, la aplicación sigue
  detectando y notificando alertas, y declara que no está registrando: **cero** pérdidas silenciosas
  de historial.
- **SC-016**: La aplicación es utilizable sin recortes en el tamaño mínimo de ventana previsto y con el
  sistema escalado al 125 %, 150 % y 200 %.

## Supuestos

Valores adoptados donde la entrada no era explícita, o donde el proyecto ya tenía una decisión escrita
que esta especificación asume sin repetir:

- **Un solo usuario, sin servidor.** No hay cuentas, roles, autenticación ni acceso remoto. El usuario
  es el administrador del equipo, presente físicamente.
- **La aplicación solo vigila mientras está abierta.** No hay servicio en segundo plano ni inicio
  automático con el sistema; ambos están explícitamente fuera del alcance de la versión 1.0.
- **Plataforma**: equipos Windows de escritorio y servidor con interfaz gráfica, arquitectura de 64
  bits. Quedan fuera otros sistemas operativos y otras arquitecturas.
- **Privilegios elevados obligatorios**: sin ellos no puede leerse el estado de salud de los
  dispositivos. Es un requisito, no una preferencia.
- **La pila tecnológica, las dependencias y las decisiones de arquitectura ya están fijadas** en la
  constitución del proyecto y en su registro de decisiones. Esta especificación no las repite ni las
  reabre.
- **Las reglas de alerta —umbrales, histéresis, deduplicación y resolución— son las del documento
  normativo de reglas de alerta**, que prevalece sobre cualquier descripción de este documento.
- **El sistema de diseño es vinculante y ya está aprobado**: catálogo cerrado de componentes, valores
  visuales centralizados y su definición de terminado por pantalla.
- **Frecuencias y umbrales por defecto razonables**, ajustables dentro de límites validados. El
  producto es útil sin tocar la configuración.
- **Formatos de exportación**: uno tabular, uno estructurado y uno imprimible, que cubren analizar,
  automatizar y compartir.
- **Umbrales de espacio libre**: se adoptan **1 GB** para el aviso y **256 MB** para detener la
  escritura, sobre el volumen donde reside el historial. Son valores de partida razonables para
  Windows, no medidos; conviene confirmarlos al implementar la retención y registrarlos en
  `docs/open-questions.md`.
- **La hora se presenta en local y se conserva en tiempo universal**; las magnitudes se guardan en
  unidades base y se formatean al mostrarlas.

### Riesgos conocidos, con alternativa ya prevista

Tres puntos siguen pendientes de medir sobre hardware real. Ninguno bloquea esta especificación,
porque los tres tienen alternativa decidida (registro de cuestiones abiertas, §I):

| Riesgo | Alternativa si no se cumple |
|---|---|
| Que el sistema entregue notificaciones emergentes desde una aplicación elevada | Ventana propia anclada sobre el área de notificación |
| Hasta dónde merece la pena insistir en leer la salud tras controladoras RAID y puentes USB | Documentar la limitación por modelo y declarar «no compatible» |
| Que la interfaz aguante 20 discos y 5.000 eventos dentro del umbral de SC-007 | Reducir la densidad del panel o paginar |
