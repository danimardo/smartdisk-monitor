# Historias de usuario

Prioridades: P0 imprescindible para 1.0; P1 importante para 1.0; P2 posterior si el calendario lo exige.

## Épica A. Inicio y configuración

### US-001 — Ejecución elevada (P0)

Como usuario quiero que la aplicación solicite privilegios de administrador al iniciarse para poder consultar todos los dispositivos compatibles.

Criterios de aceptación:

- El ejecutable solicita elevación mediante UAC.
- Si la elevación se rechaza, la aplicación no continúa en un estado parcialmente funcional.
- Se muestra una explicación comprensible cuando Windows impide la elevación.

### US-002 — Asistente inicial (P0)

Como usuario quiero configurar la aplicación mediante un asistente para empezar a monitorizar sin conocer SMART.

Criterios de aceptación:

- Aparece cuando no existe una configuración inicial completa.
- Enumera los discos detectados y selecciona inicialmente todos los compatibles.
- Permite excluir discos y asignar alias.
- Explica los estados no compatible y desconocido.
- Las elecciones se conservan tras reiniciar.

### US-003 — Preferencias de idioma y tema (P1)

Como usuario quiero que idioma y tema sigan inicialmente el sistema y poder cambiarlos.

Criterios de aceptación:

- Español del sistema selecciona español; inglés selecciona inglés; los demás seleccionan inglés.
- El tema automático responde al tema de Windows.
- Se puede elegir español/inglés y claro/oscuro/sistema.
- El cambio se aplica sin reinstalar.

### US-004 — Aplicar el sistema de diseño aprobado (P0)

Como usuario quiero una interfaz coherente, accesible y reconocible para interpretar el estado del almacenamiento sin ambigüedad.

Criterios de aceptación:

- La interfaz cumple `Design-system/AGENTS.md` y utiliza el catálogo entregado en `Design-system/src/lib/components/`.
- Todos los valores visuales proceden de `tokens.css` o de su mapeo Tailwind; no existen colores, radios, sombras o tamaños tipográficos literales en componentes.
- Funciona correctamente en temas claro y oscuro y hereda el acento de Windows, usando el respaldo definido si no está disponible.
- El acento solo indica acción o selección; los estados usan tokens semánticos y nunca dependen exclusivamente del color.
- “No disponible”, “no compatible” y “desconocido” se representan en gris y no generan apariencia de avería.
- Todo texto visible utiliza claves de i18n en español e inglés.
- Toda interacción es accesible por teclado, conserva foco visible y cumple contraste AA.
- Las animaciones respetan `prefers-reduced-motion`.
- La ventana en su mínimo técnico (1024 × 560) no recorta silenciosamente contenido y la región
  correspondiente ofrece scroll; sigue siendo correcta al 125 %, 150 % y 200 % de escalado de Windows.
- Un valor ausente cabe en su hueco: "No disponible" se compone como texto, no como cifra, y se
  distingue de un número a simple vista.
- El acento heredado de Windows se corrige antes de aplicarse: el texto sobre el acento elige blanco
  o negro según el contraste, y el acento se oscurece si aun así no alcanza AA. Un acento amarillo
  o lima no puede dejar ilegible el botón primario.
- La interfaz sigue siendo usable con veinte discos y con miles de eventos: la lista de discos tiene
  su propio scroll, el panel general fluye en vez de mantener una rejilla fija y las listas largas
  se virtualizan.

## Épica B. Inventario y salud

### US-010 — Descubrir almacenamiento (P0)

Como usuario quiero ver discos físicos, particiones y volúmenes para entender la topología del equipo.

Criterios de aceptación:

- Se enumeran NVMe, SSD SATA y HDD accesibles.
- Se muestran modelo, serie, firmware, interfaz, capacidad y volúmenes asociados cuando estén disponibles.
- No se confunde una letra de unidad con la identidad física.
- Un RAID, USB o disco virtual sin SMART aparece como no compatible o parcialmente compatible, no como averiado.
- **Cierra el riesgo I.5**: se fija y documenta la cascada de `-d` que `smartctl` intenta
  (`sat`, `nvme`, `sntjmicron`, `csmi`, …) antes de declarar un dispositivo no compatible, medida
  contra el hardware disponible. Lo que no se pueda medir se documenta como limitación por modelo
  de puente, con su identificador USB, en vez de dejarse como un «no compatible» sin explicación.
  Recordatorio de `.claude/rules/backend-rust.md`: `smartctl` sin elevación devuelve
  `Unable to detect device type`, que **no** significa que el disco sea incompatible.

### US-011 — Seleccionar discos (P0)

Como usuario quiero decidir qué discos monitorizar para excluir dispositivos irrelevantes.

Criterios de aceptación:

- Puede activarse o desactivarse cada disco.
- Los discos desactivados siguen visibles en una sección separada.
- Se pueden asignar y eliminar alias.
- La selección no se pierde si cambia una letra de unidad.

### US-012 — Ver salud actual (P0)

Como usuario quiero ver un resumen de salud por disco para reconocer problemas rápidamente.

Criterios de aceptación:

- Se muestra estado, temperatura, desgaste y errores disponibles.
- Cada valor puede indicar fuente y fecha de última lectura.
- Los valores ausentes muestran “No disponible”, nunca cero inventado.
- Se diferencia correcto, advertencia, crítico, desconocido y no compatible.
- La presentación usa `StatusPill`, `StatusDot`, `MetricCard` y demás componentes aprobados según corresponda.

### US-013 — Actualizar manualmente (P1)

Como usuario quiero forzar una lectura para comprobar un cambio inmediatamente.

Criterios de aceptación:

- Existe “Actualizar ahora”.
- No inicia dos recopilaciones iguales concurrentemente.
- La UI muestra progreso o actividad sin bloquearse.
- Los fallos parciales identifican la fuente afectada.

### US-014 — Altas y bajas de discos en caliente (P0)

Como usuario quiero que la aplicación se entere de que he conectado o retirado un disco para no
tener que reiniciarla ni quedarme mirando datos de un disco que ya no está.

Criterios de aceptación:

- Un disco conectado aparece en menos de un minuto sin reiniciar la aplicación, y el asistente no
  vuelve a aparecer: se añade a la lista con su estado de compatibilidad.
- Un disco monitorizado que desaparece se marca como retirado, conserva su historial y deja de
  contar para el estado global.
- Una retirada **precedida de una expulsión segura** no genera alerta, solo un apunte de inventario.
- Una retirada sin aviso genera alerta crítica, o advertencia si el dispositivo es USB.
- Si vuelve a conectarse el mismo dispositivo, se reconoce como el mismo y continúa su historial;
  si su identidad es solo una huella, la interfaz lo indica como identidad inferida.
- Un cambio de firmware no parte el historial: se registra como un apunte sobre el mismo disco.

## Épica C. Historial y eventos

### US-020 — Consultar gráficas (P0)

Como usuario quiero consultar gráficas históricas para detectar tendencias de temperatura, desgaste, actividad y errores.

Criterios de aceptación:

- Se pueden elegir disco, métrica e intervalo.
- Existen intervalos de 24 horas, 7 días, 30 días y personalizado.
- La zona horaria presentada es la local.
- Las discontinuidades se muestran como ausencia de datos, no como cero.

### US-021 — Consultar eventos (P0)

Como usuario quiero ver eventos de almacenamiento con su hora exacta para investigar incidentes.

Criterios de aceptación:

- Se muestran fecha, proveedor, ID, nivel y mensaje.
- Puede filtrarse por disco, volumen, nivel, proveedor e intervalo.
- La asociación inferida se etiqueta como tal.
- Reiniciar la aplicación no duplica eventos importados.
- **Cierra el riesgo I.7**: la lista se virtualiza y se mide con el peor caso previsto —20 discos y
  5.000 eventos— comprobando que el desplazamiento y el filtrado no producen bloqueo perceptible.
  Si no aguanta, se recorta la densidad del panel o se pagina, y la decisión se registra. Esta es
  la historia que crea la lista virtualizada, así que es aquí donde el riesgo deja de ser teórico.

### US-022 — Conservar y compactar historial (P1)

Como usuario quiero disponer de al menos 30 días de historial sin crecimiento ilimitado de datos detallados.

Criterios de aceptación:

- Se cumplen las retenciones definidas en la especificación: 7 días de muestras detalladas, 30 días
  de agregados de 5 minutos y un año de resúmenes horarios.
- Los 30 días prometidos son de **agregados**, no de detalle, y la interfaz lo dice: toda gráfica
  declara la resolución que está mostrando, porque un máximo promediado no es un pico.
- La compactación conserva mínimos, máximos, promedios e incrementos.
- El usuario puede modificar la retención.
- Las operaciones de retención no bloquean perceptiblemente la UI.

## Épica D. Alertas

### US-030 — Recibir alertas agrupadas (P0)

Como usuario quiero que sucesos repetidos formen una sola alerta para evitar ruido.

Criterios de aceptación:

- Eventos equivalentes incrementan un contador y actualizan la última ocurrencia.
- El detalle conserva todas las fechas y horas.
- Un cambio de severidad actualiza y vuelve a notificar el grupo.
- Recuperarse y recaer crea un nuevo ciclo identificable.

### US-031 — Gestionar alertas (P0)

Como usuario quiero reconocer, silenciar y archivar alertas para controlar cuáles requieren atención.

Criterios de aceptación:

- Una alerta activa puede reconocerse sin considerarla resuelta.
- **Reconocer no devuelve el disco a verde**: el color refleja la peor alerta no resuelta, esté
  reconocida o no. Reconocer la saca de la lista de pendientes y le añade un distintivo.
- Puede silenciarse 15 minutos, 1 hora, 8 horas o indefinidamente. El silencio suprime la
  notificación, nunca el color, y sobrevive a un reinicio.
- La resolución automática exige que la condición deje de cumplirse con margen durante tres ciclos,
  para que oscilar alrededor de un umbral no genere una alerta por muestra.
- La resolución automática conserva el historial.
- Archivar retira la alerta de la vista principal.

### US-032 — Estado en systray (P0)

Como usuario quiero reconocer el estado global desde la bandeja del sistema.

Criterios de aceptación:

- El color refleja la mayor severidad activa, con esta prioridad: una alerta crítica vigente pinta
  el icono de rojo aunque la monitorización esté pausada; solo si no hay críticos manda el gris de
  pausa, sin datos o fallo del recopilador.
- Un disco que declara no soportar SMART no impide el verde; uno que debería responder y no
  responde, sí: aporta advertencia.
- Pausar detiene la recopilación y la evaluación de reglas, y por tanto las notificaciones. La
  pausa no sobrevive a un reinicio y se anuncia de forma permanente en la barra de herramientas.
- Clic izquierdo muestra/restaura la aplicación.
- Clic derecho ofrece abrir, resumen, pausa/reanudación y salida.
- Al cerrar con X se pregunta minimizar o salir, con opción de recordar.

### US-033 — Alertas de capacidad poco intrusivas (P1)

Como usuario quiero avisos de poco espacio sin recibir la misma notificación continuamente.

Criterios de aceptación:

- Solo se notifica al cruzar un umbral, cambiar de nivel o recaer tras recuperarse.
- La alerta muestra espacio y umbral actuales.
- Puede desactivarse por volumen.

## Épica E. Diagnóstico y pruebas

### US-040 — Benchmark seguro (P0)

Como usuario quiero probar lectura y escritura mediante un archivo temporal para observar rendimiento y temperatura sin sobrescribir mis archivos.

Criterios de aceptación:

- Requiere confirmación y muestra impacto esperado.
- Nunca usa espacio sin asignar ni escribe directamente sobre el dispositivo.
- Crea un archivo nuevo con nombre no colisionable en el volumen elegido.
- Respeta la reserva de espacio y verifica el patrón leído.
- Muestra progreso, rendimiento, latencia y temperatura.
- Puede cancelarse y se detiene en el umbral térmico crítico.
- Intenta eliminar el archivo siempre y señala claramente cualquier residuo.

### US-041 — Escanear sistema de archivos (P1)

Como usuario quiero ejecutar `chkdsk /scan` para comprobar un volumen sin programar una reparación fuera de línea.

Criterios de aceptación:

- Solo ofrece la operación en volúmenes compatibles.
- Muestra advertencia y comando antes de confirmar.
- Captura salida, resultado, inicio y fin.
- No ofrece opciones reparadoras en 1.0.

### US-042 — Autotest SMART corto (P1)

Como usuario quiero solicitar el autotest corto del firmware para detectar fallos internos que un benchmark no revela.

Criterios de aceptación:

- Solo se habilita si el dispositivo declara soporte.
- Requiere confirmación y advierte del posible impacto de rendimiento.
- Muestra progreso y duración estimada cuando existan.
- Permite cancelar si el dispositivo lo admite.
- Conserva resultado e historial del autotest.
- No puede coincidir con el benchmark sobre el mismo disco.

### US-043 — Ver datos avanzados (P1)

Como usuario avanzado quiero inspeccionar el JSON original de smartctl para investigar atributos no normalizados.

Criterios de aceptación:

- Existe una sección “Detalles avanzados”.
- Puede copiarse el JSON.
- Se indica fecha, versión de smartctl y comando lógico utilizado.
- La vista no expone capacidad de ejecutar comandos arbitrarios.

## Épica F. Exportación y soporte

### US-050 — Exportar informes (P1)

Como usuario quiero exportar información para conservarla o compartirla.

Criterios de aceptación:

- Exporta CSV, JSON y HTML.
- Permite 24 horas, 7 días, 30 días y rango personalizado.
- El HTML es legible e imprimible.
- La exportación informa claramente de campos omitidos o no disponibles.

### US-051 — Crear diagnóstico anonimizado (P1)

Como usuario quiero crear un ZIP de diagnóstico sin revelar identificadores personales por defecto.

Criterios de aceptación:

- Anonimiza series, equipo, usuarios y rutas personales inicialmente.
- La sustitución es consistente dentro del paquete.
- El usuario puede incluir identificadores expresamente.
- Antes de guardar se muestra un resumen del contenido.

## Épica G. Instalación y mantenimiento

### US-060 — Instalar y desinstalar (P0)

Como usuario quiero un instalador manual para todos los usuarios y un desinstalador limpio.

Criterios de aceptación:

- Instala la aplicación x64 y `smartctl` con sus avisos correspondientes.
- Registra correctamente nombre, versión y autor.
- La versión procede del manifiesto, no de texto duplicado.
- Desinstalar conserva `ProgramData` y el historial.
- Se documenta cómo borrar manualmente los datos.
- **La carpeta de `ProgramData` se crea con la propiedad y la ACL de ADR-026**: `/setowner` a
  administradores **antes** de `/inheritance:r`, y con SID numéricos, no nombres de grupo. Se
  verifica que un usuario sin privilegios no puede escribir en ella ni recuperar el permiso, y que
  pre-crearla antes de instalar no le sirve de nada (`open-questions.md` §R).
- **Cierra el riesgo I.2**: se comprueba en la máquina empaquetada que Windows entrega las
  notificaciones toast con la aplicación bajo `requireAdministrator` y su AUMID registrado. Si no
  las entrega, se activa el plan B —ventana propia con el componente `Toast` anclada sobre la
  bandeja— y se anota en `open-questions.md`. Es lo que sostiene US-030: sin toast, esa historia
  pierde su mecanismo principal.
- **Comprobación de humo de la instancia única** (ADR-025): con la aplicación abierta y
  minimizada, lanzarla de nuevo no crea un segundo proceso y **restaura y enfoca la ventana
  existente**. No es automatizable: exige aceptar el UAC.

### US-061 — Ver información de la aplicación (P1)

Como usuario quiero consultar versión, autor y licencias.

Criterios de aceptación:

- El botón `?` abre “Acerca de”.
- Nombre y versión se obtienen dinámicamente.
- Muestra autor, MIT, licencias de terceros y repositorio.
- Permite copiar información diagnóstica no sensible.

## Épica H. Configuración y mantenimiento

Esta épica cubre la pantalla de Ajustes, que hasta ahora aparecía en la navegación, en el paquete de
diseño y en el roadmap sin una sola historia que la definiera.

### US-070 — Ajustar frecuencias y umbrales (P1)

Como usuario quiero cambiar cada cuánto se mide y a partir de qué valores se avisa para adaptar la
aplicación a mi equipo sin tener que aceptar los valores de fábrica.

Criterios de aceptación:

- Se pueden modificar las cuatro frecuencias (métricas rápidas, SMART completo, eventos, detección
  de altas y bajas) dentro de los límites documentados, y el control impide salirse de ellos en vez
  de aceptar el valor y fallar después.
- Cada ajuste muestra su valor de fábrica y permite volver a él.
- Se pueden cambiar los umbrales de temperatura de respaldo, que solo se aplican a discos sin límite
  declarado por el fabricante; la interfaz explica esa precedencia.
- Se pueden cambiar los umbrales de capacidad y el tamaño a partir del cual se aplica el suelo
  absoluto.
- Un cambio se aplica sin reiniciar y sin perder el ciclo en curso.
- Un valor fuera de rango se rechaza con una explicación comprensible, nunca con un error técnico.

### US-071 — Ajustar la retención y saber cuánto ocupa (P1)

Como usuario quiero saber cuánto espacio consume el historial y decidir cuánto se conserva para que
la aplicación no crezca sin control en mi disco.

Criterios de aceptación:

- Se muestra el tamaño actual de la base de datos, de los logs y de las capturas en bruto.
- Se pueden cambiar los tres periodos de retención dentro de límites seguros.
- Reducir una retención advierte de cuántos datos se van a eliminar **antes** de aplicarla.
- Alertas, ocurrencias críticas, eventos vinculados y ejecuciones de pruebas nunca se borran por
  retención, y la interfaz lo dice.
- La compactación se ejecuta sin bloquear la interfaz.

### US-072 — Configurar el arranque, el cierre y la apariencia (P1)

Como usuario quiero decidir qué pasa al cerrar la ventana y cómo se ve la aplicación para que se
comporte como espero.

Criterios de aceptación:

- Se puede elegir entre minimizar a la bandeja o salir al pulsar la X, y cambiar la decisión que se
  recordó la primera vez.
- Se puede elegir idioma (español/inglés) y tema (claro/oscuro/sistema), y ambos se aplican al
  instante.
- Se puede desactivar la herencia del acento de Windows y volver al azul del sistema de diseño.
- Se puede activar o desactivar el sonido de las notificaciones, desactivado de fábrica.
- Todas las preferencias se guardan en `settings`, nunca en el navegador, y sobreviven al reinicio.

### US-073 — Borrar todos los datos (P1)

Como usuario quiero poder eliminar todo el historial que la aplicación ha guardado sobre mis discos
para dejar el equipo limpio sin buscar carpetas a mano.

Criterios de aceptación:

- La acción vive en Ajustes, separada del resto y claramente marcada como irreversible.
- Exige escribir una frase de confirmación, no solo pulsar un botón.
- Enumera qué se va a borrar y qué se va a conservar antes de hacerlo.
- Al terminar, la aplicación queda como recién instalada y vuelve a mostrar el asistente inicial.
- La documentación explica cómo hacer lo mismo a mano después de desinstalar.

### US-074 — Recuperarse de un cierre inesperado (P0)

Como usuario quiero que un corte de corriente o un cierre forzado no deje la aplicación en un estado
raro ni basura en mis discos.

Criterios de aceptación:

- Al arrancar, toda prueba que quedó en `running` o `cancelling` pasa a `interrupted` y se explica
  en su historial; ninguna queda eternamente "en curso".
- Los archivos temporales de benchmark que quedaron huérfanos se detectan, se listan y se ofrecen
  para eliminar, indicando su ruta exacta.
- Si un archivo huérfano no se puede borrar, se dice claramente en vez de fallar en silencio.
- Los eventos ya importados no se reimportan.
- Si una migración quedó a medias, se restaura la copia previa y se explica lo ocurrido.
- La base de datos se abre en modo WAL y una recuperación normal no requiere intervención.
