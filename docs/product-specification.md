# Especificación del producto

## 1. Propósito

SmartDisk Monitor es una aplicación de escritorio local que permite conocer la salud, temperatura, actividad, rendimiento, capacidad y eventos relevantes de los discos instalados en un equipo Windows. Está orientada a uso personal por un administrador y funciona únicamente mientras exista una sesión iniciada y la aplicación permanezca ejecutándose.

El producto debe ayudar a responder:

- ¿Qué discos físicos y volúmenes existen en el equipo?
- ¿Cuál es su estado actual y cómo ha evolucionado?
- ¿Ha ocurrido alguna anomalía, cuándo sucedió y cuántas veces se repitió?
- ¿Se degrada un disco bajo carga o por temperatura?
- ¿Qué información se puede exportar para analizar o compartir un incidente?

## 2. Alcance de la versión 1.0

### Incluido

- Windows 10 (1809, build 17763, o superior), Windows 11 y Windows Server 2016, 2019, 2022 y 2025, x64.
- Requisito de plataforma: **WebView2 Runtime**. Solo Windows 11 lo incluye de serie; en Windows 10
  lo tiene la gran mayoría de equipos desde que Microsoft lo desplegó por Windows Update, y **en
  Windows Server no viene preinstalado en ninguna versión**. El instalador lo resuelve sin necesidad
  de conexión (§10, ADR-020).
- Requisito de CPU: soporte de SSE3, que Microsoft Edge exige desde su versión 128.
- Se asume Windows Server con **Experiencia de escritorio**: una aplicación de interfaz gráfica no
  es utilizable en una instalación Server Core.
- Aplicación local, de usuario único y sin servidor central.
- Ejecución obligatoria con privilegios de administrador.
- Detección automática de discos físicos, particiones y volúmenes.
- Selección de los discos que se desean monitorizar.
- Compatibilidad prioritaria con NVMe, SSD SATA y HDD conectados directamente.
- Degradación elegante cuando SMART no esté disponible en RAID, USB o máquinas virtuales.
- Lectura SMART/NVMe mediante `smartctl` y fuentes nativas de Windows.
- Temperatura, salud, desgaste, errores, horas de funcionamiento y demás contadores disponibles.
- Capacidad libre de los volúmenes asociados.
- Actividad, velocidad y latencia observadas mediante contadores de Windows.
- Captura y correlación de eventos relevantes de Windows.
- Historial local en SQLite, gráficas y panel general.
- Alertas locales, agrupadas y visibles desde la aplicación y el systray.
- Prueba manual de lectura y escritura con un archivo temporal controlado.
- Ejecución manual de `chkdsk /scan`.
- Autotest SMART corto manual cuando el dispositivo lo soporte.
- Exportación CSV, JSON y HTML imprimible.
- Paquete ZIP de diagnóstico anonimizado por defecto.
- Español e inglés, seleccionados inicialmente según el idioma del sistema.
- Tema claro, oscuro o automático según el sistema.
- Instalador manual y desinstalador.
- **Ayuda con IA opcional** (spec `005-explicacion-ia`, ampliada por `006-explicacion-ia-contexto-crudo`
  y ADR-049, principio XVI): si la persona configura una clave de API de OpenRouter, puede pedir que
  se le traduzca a lenguaje llano el detalle técnico de una alerta, del detalle SMART de un disco o
  de un evento del registro de Windows, con posibles pasos a seguir. Apagada de fábrica; sin clave, la aplicación no hace ninguna conexión a
  Internet. La consulta lleva el volcado técnico completo del disco y, en alertas de sucesos de
  Windows, el contenido de ese suceso; todo se anonimiza en capas antes de salir del equipo
  (número de serie, WWN, nombre de equipo y usuario, SID, rutas de dispositivo) y la persona ve el
  texto exacto la primera vez. Un ajuste opcional «enviar sin revisar», apagado de fábrica y con
  aviso de riesgo, omite la pantalla de revisión de fragmentos dudosos.

### Fuera del alcance inicial

- Servicio de Windows y monitorización sin sesión iniciada.
- Inicio automático con Windows.
- Consola central o monitorización remota.
- Cuentas, roles o autenticación.
- Alertas por correo, mensajería o servicios externos.
- Actualizaciones automáticas.
- Telemetría.
- Versiones Linux, macOS, ARM64 o x86 de 32 bits.
- Integraciones específicas con herramientas de fabricantes RAID.
- Autotest SMART extendido.
- Programación automática de autotests o benchmarks.

## 3. Experiencia principal

### Primera ejecución

1. Windows solicita elevación UAC.
2. La aplicación muestra un asistente inicial.
3. Se detectan los discos y se explica qué datos están disponibles.
4. Todos los discos compatibles quedan seleccionados inicialmente.
5. El usuario puede excluir discos y asignar alias.
6. Se presentan los umbrales y frecuencias predeterminados.
7. Comienza la monitorización y se abre el panel general.

### Panel general

Debe mostrar de un vistazo:

- estado global del equipo;
- número de discos correctos, con advertencias, críticos y desconocidos;
- alertas activas y sucesos recientes;
- tarjetas de discos con temperatura, salud, actividad y capacidad;
- acceso a gráficas, eventos, pruebas, informes y configuración.

### Bandeja del sistema

- Verde: todos los elementos monitorizados están correctos.
- Ámbar: existe al menos una advertencia.
- Rojo: existe al menos una alerta crítica activa.
- Gris: monitorización pausada, sin datos o fallo general de recopilación.
- Clic izquierdo: mostrar o restaurar la ventana.
- Clic derecho: abrir, ver un resumen, pausar/reanudar y salir.
- La aplicación no se inicia automáticamente de fábrica. El asistente inicial y Ajustes ofrecen
  «Arrancar SmartDisk con el sistema» (`lifecycle.start_with_system`): al activarlo se registra una
  tarea programada que la abre —ya elevada, sin diálogo de UAC— al iniciar sesión (ADR-038).
- Al cerrar con X, pregunta si debe minimizarse o salir y permite recordar la decisión.

## 4. Frecuencias predeterminadas

- Actividad, capacidad y latencia: cada 30 segundos.
- SMART completo (incluye la temperatura): cada 5 minutos.
- Eventos de Windows: cada 30 segundos, usando un marcador persistente para no duplicarlos.
- Detección de discos añadidos o retirados: cada minuto.
- Botón para forzar una actualización completa.
- En batería se reduce la frecuencia de métricas no críticas, pero no se suspenden alertas graves.

Todas las frecuencias son configurables dentro de estos límites, que valida el backend:

| Trabajo | Por defecto | Mínimo | Máximo |
|---|---|---|---|
| Actividad, capacidad, latencia, caudal | 30 s | 10 s | 5 min |
| SMART completo (incluye la temperatura) | 5 min | 1 min | 60 min |
| Eventos de Windows | 30 s | 15 s | 5 min |
| Detección de altas y bajas | 60 s | 30 s | 10 min |

La temperatura va con «SMART completo», no con las métricas rápidas: solo se obtiene del parseo de
`smartctl` (`docs/open-questions.md` D.6).

En batería se multiplica por cuatro el intervalo de las métricas rápidas y el de detección de altas
y bajas. SMART completo y eventos de Windows no se alteran: son las fuentes de las alertas graves.
Al volver a la red eléctrica se restauran de inmediato y se fuerza un ciclo completo.

**Pausar** detiene la recopilación y la evaluación de reglas, y con ellas las notificaciones. No
sobrevive a un reinicio: arrancar la aplicación siempre reanuda, porque una pausa olvidada es un
monitor que no vigila y no lo dice. Mientras está pausada, la barra de herramientas lo anuncia de
forma permanente.

## 5. Alertas

### Estados

- Activa: la condición continúa.
- Reconocida: el usuario la ha visto, pero la condición continúa.
- Resuelta automáticamente: la condición ha desaparecido.
- Archivada: permanece en el historial, pero no en la lista principal.

El **silencio** no es un estado: es ortogonal y puede convivir con cualquiera de ellos. Suprime la
notificación, nunca el color ni la presencia en la lista, y sobrevive a un reinicio.

**Reconocer no cambia el color.** El estado que se pinta en un disco es la peor severidad de sus
alertas activas o reconocidas; solo las resueltas y archivadas dejan de contar (ADR-016).

Las alertas equivalentes se agrupan. Cada grupo conserva contador, primera ocurrencia, última ocurrencia y detalle cronológico de todas las repeticiones.

La especificación normativa completa —activación, histéresis de resolución, deduplicación,
cooldown y textos de cada regla— vive en [`alert-rules.md`](alert-rules.md). Lo que sigue es el
resumen; si discrepan, manda aquel documento.

### Reglas iniciales

- Estado SMART crítico: crítico inmediato.
- `critical_warning` NVMe distinto de cero: crítico inmediato.
- Incremento de errores de medios o errores no corregibles: crítico inmediato.
- Incremento del registro de errores: advertencia; crítico según tipo y repetición.
- Disco retirado inesperadamente: crítico inmediato.
- Eventos graves de disco o sistema de archivos: crítico.
- Reinicios o reintentos del controlador repetidos: advertencia o crítico según frecuencia.
- Temperatura por encima del límite del fabricante: advertencia tras tres muestras; crítica al alcanzar el umbral crítico.
- Si no existe límite del fabricante: valores generales configurables, inicialmente 70 °C y 80 °C.
- Poco espacio: siempre por porcentaje (advertencia bajo 10 %, crítico bajo 5 %) y, **solo en
  volúmenes de 256 GB o más**, también por valor absoluto (advertencia bajo 20 GB, crítico bajo
  10 GB). Gana el criterio más severo. El corte de 256 GB es configurable: sin él, un volumen de
  64 GB con 15 GB libres —casi una cuarta parte— se marcaría como crítico (ADR-019).
- SMART ilegible persistentemente: advertencia después de tres intentos; “no compatible” no genera alerta.
- Recopilador detenido o bloqueado: advertencia tras superar tres intervalos esperados.

Las alertas de capacidad son poco intrusivas: se genera una alerta agrupada al cruzar el umbral, no una nueva alerta en cada muestra. Solo vuelve a notificarse al cambiar de nivel o después de recuperarse y recaer.

### Notificaciones

- Centro de alertas dentro de la aplicación.
- Notificación nativa de Windows cuando la aplicación está minimizada. Se puede desactivar por
  completo (`notifications.enabled`, activada de fábrica): la alerta sigue en la lista, solo deja de
  aparecer la ventana emergente. Distinto de pausar, que además detiene la recopilación.
- Sin canales externos.
- Sonido desactivado inicialmente.
- Silencio temporal de 15 minutos, 1 hora, 8 horas o indefinido hasta reactivación manual.

## 6. Pruebas manuales

### Prueba de lectura y escritura

- El usuario elige un volumen de un disco seleccionado.
- Se crea un archivo temporal dedicado en una carpeta controlada por la aplicación.
- Nunca se sobrescribe un archivo existente.
- Parámetros predeterminados: **1 GiB** de archivo, bloques de **1 MiB**, acceso **secuencial**,
  **una** pasada de escritura y una de lectura. Configurables entre 256 MiB y 8 GiB.
- La escritura se hace **sin caché del sistema** (`FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH`)
  y se sincroniza antes de medir. Sin esto se estaría midiendo la memoria RAM y las cifras no serían
  comparables entre ejecuciones ni entre discos.
- El tamaño se limita por configuración, espacio libre y una **reserva de seguridad de 2 GiB o el
  5 % del volumen, la mayor de las dos**, que nunca se invade.
- Se escribe, sincroniza, lee y verifica el contenido mediante bloques con patrón comprobable.
- Se muestran rendimiento, latencia, progreso y temperatura.
- Se puede cancelar.
- Se detiene si se alcanza el límite térmico crítico —el del fabricante si lo declara, y si no el
  configurado— o si se llega a la reserva de espacio. La razón de la parada se conserva en el
  historial de la prueba.
- El archivo se elimina al finalizar o cancelar; si no fuera posible, queda claramente identificado para su limpieza posterior.
- Antes de comenzar se advierte del impacto temporal en rendimiento, temperatura y escrituras del SSD.

### CHKDSK

- Ejecución manual de `chkdsk /scan`, **solo en volúmenes NTFS**: es el único sistema de archivos
  que admite ese modificador. En ReFS, exFAT y FAT32 la acción aparece deshabilitada explicando por qué.
- La salida **no** viene en UTF-8, y tampoco se puede asumir una página de códigos fija: medido en
  un Windows 11 en español, `chkdsk` emite **CP1252 (ANSI)** mientras que `fsutil` y `vssadmin`
  emiten **CP850 (OEM)**, en el mismo equipo y por el mismo tipo de tubería. Fijar `chcp` antes de
  invocar no cambia nada cuando la salida está redirigida. Se aplica la detección descrita en
  `open-questions.md` §Q, se guardan siempre los bytes originales y se registra qué codificación
  se dedujo.
- Se muestra el comando y una advertencia antes de ejecutarlo.
- La salida se conserva asociada a la ejecución.
- No se ofrecen inicialmente opciones de reparación fuera de línea.

### Autotest SMART corto

- Solo manual y únicamente si el dispositivo declara compatibilidad.
- Muestra duración estimada, progreso y resultado cuando estén disponibles.
- Puede cancelarse si el dispositivo lo admite.
- Advierte de la posible degradación temporal del rendimiento.
- No se permite simultáneamente con el benchmark de la aplicación.
- No se trata la falta de compatibilidad como una anomalía.

## 7. Historial y retención

- Muestras detalladas: 7 días.
- Agregados de 5 minutos: 30 días como mínimo.
- Resúmenes horarios: un año.

Los 30 días de historial que promete US-022 son de **agregados**, no de detalle. Toda gráfica
declara la resolución que está mostrando: un máximo promediado no es un pico, y confundirlos al
investigar un incidente térmico llevaría a conclusiones falsas. La correspondencia entre intervalo
pedido y resolución servida está en [`open-questions.md`](open-questions.md) §E.1.
- Alertas y eventos importantes: conservación indefinida hasta eliminación manual.
- Logs técnicos: 30 días y máximo aproximado de 100 MB mediante rotación.
- Retención configurable.
- El historial se identifica por número de serie y permanece separado cuando se sustituye un disco.
- Los datos permanecen en el equipo después de desinstalar.
- La aplicación permite borrarlos mediante una acción explícita con confirmación.

## 8. Internacionalización y apariencia

- Idiomas iniciales: español e inglés.
- Se usa el idioma del sistema si está soportado; en caso contrario, inglés.
- El usuario puede cambiar el idioma sin reinstalar.
- Tema inicial según Windows, con selección manual claro/oscuro/sistema.
- Tamaño de ventana: mínimo técnico 1024 × 560, objetivo de diseño 1280 × 720, predeterminado
  1695 × 988 acotado a la pantalla **solo el primer arranque**. A partir de ahí la aplicación
  recuerda entre sesiones el tamaño, la posición y si estaba maximizada (ADR-040); si el monitor
  donde estaba ya no existe, abre centrada. El mínimo es bajo por una razón medida: el escalado de
  Windows no encoge el texto, encoge el espacio en píxeles CSS, y un portátil de 1920 × 1080 al
  150 % solo deja 1280 × 672 de ventana. La interfaz debe ser correcta al 125 %, 150 % y 200 %.
- Por debajo de 1180 px de ancho la barra lateral se reduce a iconos, conservando el punto de estado
  de cada disco.
- El formato de números y fechas sigue al **idioma elegido en la aplicación**, no al de Windows,
  conservando la variante regional del sistema cuando comparten idioma.
- Sistema de diseño aprobado: **SmartDisk Monitor v2, material translúcido**.
- [`ui-design.md`](ui-design.md) es vinculante para cualquier implementación de interfaz; su §0 dice dónde vive cada pieza.
- `src/design-system/tokens.css` es la fuente única de colores, tipografía, espaciado, radios, sombras, materiales y movimiento; no se permiten valores visuales literales en componentes.
- El acento de acciones y selección se hereda de Windows, con el azul de respaldo definido en los tokens. El acento nunca comunica salud.
- Verde, ámbar, rojo y gris se reservan respectivamente para correcto, advertencia, crítico y desconocido/no compatible/sin datos. El color siempre se acompaña de texto o iconografía accesible.
- Se usan exclusivamente tres niveles de material: chrome, tarjetas y overlays; no se apilan tarjetas ni se inventan niveles de desenfoque.
- Tipografía principal Instrument Sans con alternativas del sistema y peso máximo 600.
- Movimiento funcional y breve, respetando `prefers-reduced-motion`.
- Contraste mínimo AA, foco visible y navegación completa por teclado.
- Las preferencias de idioma y tema se guardan en SQLite mediante `settings`, nunca en `localStorage`.
- Panel general, detalle de disco, alertas y pruebas/diagnóstico siguen los bocetos aprobados de `design/SmartDisk Monitor v2.dc.html`.
- Informes, Ajustes, asistente inicial, Acerca de y estados de systray deben componerse con el catálogo existente y someterse a revisión antes de introducir patrones nuevos.

## 9. Informes y diagnóstico

- Intervalos: últimas 24 horas, 7 días, 30 días o fechas personalizadas.
- Exportaciones: CSV, JSON y HTML imprimible.
- El PDF se obtiene inicialmente mediante impresión del HTML.
- El HTML exportado es autónomo: CSS embebido, sin fuentes ni recursos remotos, tema claro forzado y
  hoja de impresión propia. Debe abrirse igual en un equipo sin conexión.
- Toda exportación lleva un campo `schemaVersion`, incluidas las cabeceras de CSV.
- Los mensajes de eventos de Windows y la salida de `chkdsk` llegan en el idioma del sistema, no en
  el de la aplicación. Se muestran tal cual, marcados como texto original del sistema.
- Vista avanzada del JSON bruto de `smartctl`, con copia al portapapeles.
- ZIP de diagnóstico con configuración, eventos, SMART bruto y logs.
- El ZIP oculta por defecto números de serie, nombre del equipo, usuarios y rutas personales.

## 10. Instalación y distribución

- Instalador para Windows x64, para todos los usuarios.
- El instalador resuelve la dependencia de WebView2 con el **instalador sin conexión** del runtime
  Evergreen (`webviewInstallMode: offlineInstaller`): comprueba si está presente y solo lo instala
  si falta. Funciona en equipos sin salida a Internet, y el runtime queda después mantenido al día
  por Microsoft sin que la aplicación publique nada (ADR-020).
- Esto lleva el instalador a unos 140 MB, frente a los ~10 MB del paquete a secas. Se indica en la
  página de descarga para que nadie se sorprenda.
- Instalación y desinstalación manuales.
- Sin certificado de firma inicialmente; se documentará la posible advertencia de SmartScreen.
- Sin comprobación automática de actualizaciones.
- Los datos se conservan tras desinstalar.
- `smartctl` se distribuye como programa auxiliar independiente con sus licencias y avisos:
  **smartmontools 7.5**, binario x64, junto a su base de datos de unidades `drivedb.h` y su código
  fuente completo, que viaja dentro del instalador para cumplir la GPLv2 (ADR-021).
- Se genera `THIRD_PARTY_NOTICES`.
- Nombre y versión se obtienen dinámicamente de los manifiestos de compilación.

## 11. Privacidad y seguridad

- Cero telemetría.
- Cero comunicaciones de red necesarias durante el funcionamiento normal.
- Todo el procesamiento y almacenamiento es local.
- La interfaz web no puede ejecutar comandos arbitrarios.
- Los comandos elevados se implementan en Rust mediante una API cerrada y argumentos validados.
- Los archivos de prueba se crean exclusivamente en rutas calculadas y verificadas.
- Las exportaciones se anonimizan por defecto.
- Solo una instancia de la aplicación puede ejecutarse a la vez; abrir una segunda **restaura y
  enfoca la ventana de la primera** en lugar de morir en silencio (ADR-025).
- La carpeta de datos de `ProgramData` restringe la escritura a administradores **mediante una ACL
  explícita que aplica el instalador**, con toma de propiedad previa (ADR-026). No se hereda: los
  permisos por omisión de `%ProgramData%` permiten a cualquier usuario crear ficheros y carpetas, y
  quedarse con el control de las que crea. La lectura sí queda abierta, deliberadamente.

## 12. Criterios globales de calidad

- Una avería de un colector no debe bloquear la interfaz.
- Los errores deben presentarse con lenguaje comprensible y conservar detalle técnico.
- Reiniciar la aplicación no debe duplicar eventos ya importados.
- Un disco no compatible debe aparecer como desconocido/no disponible, no como averiado.
- Todas las operaciones que generen carga o escriban datos requieren confirmación explícita.
- La aplicación debe seguir respondiendo durante recopilaciones, exportaciones y pruebas.
- Toda pantalla debe cumplir la definición de terminado de [`ui-design.md`](ui-design.md) §8 en temas claro y oscuro y en el tamaño mínimo de ventana.
- Todo texto visible debe proceder del sistema i18n; no se admiten literales de interfaz fuera de los
  diccionarios español e inglés, incluidos `aria-label`, títulos y textos alternativos.
- La interfaz debe seguir siendo usable con veinte discos y con miles de eventos.
- Las decisiones que esta especificación deja abiertas se registran, con su valor adoptado, en
  [`open-questions.md`](open-questions.md). Ningún implementador debería tener que asumir nada que
  no esté allí; si encuentra algo, se añade en vez de resolverlo en el código.
