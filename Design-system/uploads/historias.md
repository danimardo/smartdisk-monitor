# Documentación consolidada de SmartDisk Monitor

Este documento reúne el contenido de todos los documentos del proyecto. Los archivos originales siguen siendo la fuente editable por separado.

---

## Archivo de origen: `README.md`

# SmartDisk Monitor

Aplicación local de supervisión de discos para Windows, desarrollada con Tauri 2, Rust, Svelte, TypeScript y Tailwind CSS.

Estado actual: fase de especificación. La implementación comenzará después de aprobar los documentos funcionales y técnicos.

## Documentación

- [Especificación del producto](docs/product-specification.md)
- [Arquitectura](docs/architecture.md)
- [Modelo de datos](docs/data-model.md)
- [Historias de usuario](docs/user-stories.md)
- [Backlog y versiones](docs/roadmap.md)
- [Decisiones técnicas](docs/decisions.md)

## Identidad del proyecto

- Producto: SmartDisk Monitor
- Autor: Daniel Diez Mardomingo
- Repositorio previsto: https://github.com/danimardo/smartdisk-monitor
- Licencia del código propio: MIT
- Plataforma inicial: Windows x64

---

## Archivo de origen: `docs/product-specification.md`

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

- Windows 10, Windows 11 y Windows Server 2016, 2019, 2022 y 2025, x64.
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
- La aplicación no se inicia automáticamente.
- Al cerrar con X, pregunta si debe minimizarse o salir y permite recordar la decisión.

## 4. Frecuencias predeterminadas

- Temperatura, actividad, capacidad y latencia: cada 30 segundos.
- SMART completo: cada 5 minutos.
- Eventos de Windows: cada 30 segundos, usando un marcador persistente para no duplicarlos.
- Detección de discos añadidos o retirados: cada minuto.
- Botón para forzar una actualización completa.
- En batería se reduce la frecuencia de métricas no críticas, pero no se suspenden alertas graves.

Todas las frecuencias serán configurables dentro de límites seguros.

## 5. Alertas

### Estados

- Activa: la condición continúa.
- Reconocida: el usuario la ha visto, pero la condición continúa.
- Resuelta automáticamente: la condición ha desaparecido.
- Archivada: permanece en el historial, pero no en la lista principal.

Las alertas equivalentes se agrupan. Cada grupo conserva contador, primera ocurrencia, última ocurrencia y detalle cronológico de todas las repeticiones.

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
- Poco espacio: advertencia bajo el mayor entre 10 % y 20 GB; crítico bajo el mayor entre 5 % y 10 GB.
- SMART ilegible persistentemente: advertencia después de tres intentos; “no compatible” no genera alerta.
- Recopilador detenido o bloqueado: advertencia tras superar tres intervalos esperados.

Las alertas de capacidad son poco intrusivas: se genera una alerta agrupada al cruzar el umbral, no una nueva alerta en cada muestra. Solo vuelve a notificarse al cambiar de nivel o después de recuperarse y recaer.

### Notificaciones

- Centro de alertas dentro de la aplicación.
- Notificación nativa de Windows cuando la aplicación está minimizada.
- Sin canales externos.
- Sonido desactivado inicialmente.
- Silencio temporal de 15 minutos, 1 hora, 8 horas o indefinido hasta reactivación manual.

## 6. Pruebas manuales

### Prueba de lectura y escritura

- El usuario elige un volumen de un disco seleccionado.
- Se crea un archivo temporal dedicado en una carpeta controlada por la aplicación.
- Nunca se sobrescribe un archivo existente.
- El tamaño se limita por configuración, espacio libre y reserva de seguridad.
- Se escribe, sincroniza, lee y verifica el contenido mediante bloques con patrón comprobable.
- Se muestran rendimiento, latencia, progreso y temperatura.
- Se puede cancelar.
- Se detiene si se alcanza el límite térmico crítico o el espacio de reserva.
- El archivo se elimina al finalizar o cancelar; si no fuera posible, queda claramente identificado para su limpieza posterior.
- Antes de comenzar se advierte del impacto temporal en rendimiento, temperatura y escrituras del SSD.

### CHKDSK

- Ejecución manual de `chkdsk /scan` para volúmenes compatibles.
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
- Identidad visual provisional: disco estilizado y pulso de actividad; colores de salud verde, ámbar, rojo y gris.
- El diseño debe permitir sustituir posteriormente tokens, iconos y componentes con un paquete de diseño profesional.

## 9. Informes y diagnóstico

- Intervalos: últimas 24 horas, 7 días, 30 días o fechas personalizadas.
- Exportaciones: CSV, JSON y HTML imprimible.
- El PDF se obtiene inicialmente mediante impresión del HTML.
- Vista avanzada del JSON bruto de `smartctl`, con copia al portapapeles.
- ZIP de diagnóstico con configuración, eventos, SMART bruto y logs.
- El ZIP oculta por defecto números de serie, nombre del equipo, usuarios y rutas personales.

## 10. Instalación y distribución

- Instalador para Windows x64, para todos los usuarios.
- Instalación y desinstalación manuales.
- Sin certificado de firma inicialmente; se documentará la posible advertencia de SmartScreen.
- Sin comprobación automática de actualizaciones.
- Los datos se conservan tras desinstalar.
- `smartctl` se distribuye como programa auxiliar independiente con sus licencias y avisos.
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

## 12. Criterios globales de calidad

- Una avería de un colector no debe bloquear la interfaz.
- Los errores deben presentarse con lenguaje comprensible y conservar detalle técnico.
- Reiniciar la aplicación no debe duplicar eventos ya importados.
- Un disco no compatible debe aparecer como desconocido/no disponible, no como averiado.
- Todas las operaciones que generen carga o escriban datos requieren confirmación explícita.
- La aplicación debe seguir respondiendo durante recopilaciones, exportaciones y pruebas.

---

## Archivo de origen: `docs/architecture.md`

# Arquitectura

## 1. Resumen

SmartDisk Monitor será una aplicación monolítica local con separación interna entre presentación, dominio, recopiladores y persistencia.

```text
Svelte + TypeScript
        │ comandos y eventos Tauri tipados
Rust / dominio y orquestación
   ├── Inventario de almacenamiento
   ├── Colector smartctl
   ├── Contadores de Windows
   ├── Registro de eventos
   ├── Motor de alertas
   ├── Pruebas manuales
   ├── Informes
   └── Repositorios SQLite
        │
SQLite + logs + capturas JSON
```

La aplicación completa se inicia elevada y vive en el systray mientras está activa. No se instala un servicio.

## 2. Pila tecnológica

- Tauri 2 como shell de escritorio.
- Rust para acceso privilegiado, procesos, recopilación, reglas y persistencia.
- Svelte + TypeScript para la interfaz.
- Tailwind CSS para estilos mediante tokens reemplazables.
- SQLite, accedido desde Rust.
- `smartctl` como binario auxiliar independiente.
- APIs de Windows, WMI/CIM, Performance Counters y Windows Event Log según disponibilidad.
- Formatos de intercambio internos versionados y serializados con Serde.

No se invocarán comandos privilegiados directamente desde JavaScript. El frontend solo podrá llamar a comandos Tauri enumerados y con argumentos validados.

## 3. Componentes

### Inventario

- Descubre discos, interfaces, controladoras, particiones y volúmenes.
- Reconcilia distintas identidades de Windows y smartctl.
- Usa número de serie como identidad preferente y una huella estable como alternativa.
- Detecta altas, retiradas, sustituciones y cambios de firmware.
- Registra capacidades no disponibles y su causa conocida.

### Colector SMART

- Ejecuta `smartctl --scan-open --json` para descubrir dispositivos accesibles.
- Ejecuta consultas JSON individuales con tiempo máximo y cancelación.
- Conserva los datos normalizados y, selectivamente, la captura bruta.
- Interpreta el código de salida por bits; no depende solo del código cero/no cero.
- Limita ejecutable y argumentos mediante configuración cerrada de Tauri/Rust.

### Colector Windows

- Obtiene salud complementaria y topología mediante las APIs disponibles.
- Lee capacidad y estado del sistema de archivos por volumen.
- Lee actividad, rendimiento y latencias acumuladas o instantáneas.
- Mantiene cada fuente separada para poder indicar procedencia y confianza.

### Registro de eventos

- Consulta proveedores y eventos de almacenamiento configurados.
- Persiste un marcador por canal para evitar duplicados.
- Normaliza proveedor, identificador, nivel, fecha, dispositivo inferido y mensaje.
- La asociación a un disco puede ser exacta, inferida o desconocida; nunca se presenta una inferencia como certeza.

### Planificador

- Ejecuta trabajos periódicos independientes.
- Evita ejecuciones concurrentes del mismo trabajo.
- Aplica tiempo máximo, cancelación y backoff ante fallos.
- Reduce trabajos no críticos en batería.
- Suspende recopilaciones incompatibles durante benchmarks o autotests.

### Motor de alertas

- Evalúa valores absolutos, incrementos, persistencia y frecuencia.
- Deduplica por una clave estable de regla, disco/volumen y contexto.
- Conserva ocurrencias individuales dentro de un grupo.
- Implementa transición activa, reconocida, resuelta y archivada.
- Aplica cooldown a las notificaciones, especialmente para capacidad.

### Pruebas

- Benchmark basado en archivo temporal seguro y verificable.
- CHKDSK `/scan` con captura de salida.
- Autotest SMART corto con detección de soporte, seguimiento y cancelación.
- Exclusión mutua por disco entre pruebas y determinadas recopilaciones.

### Informes

- Consulta intervalos sin bloquear escrituras de recopilación.
- Exporta datos normalizados y metadatos de procedencia.
- Anonimiza identificadores mediante sustitución estable dentro de cada paquete.

## 4. Concurrencia y ciclo de vida

- La UI y los colectores no comparten operaciones bloqueantes.
- Los trabajos se ejecutan en tareas asíncronas o pools apropiados.
- SQLite usa WAL, transacciones breves y migraciones versionadas.
- Al cerrar hacia la bandeja, todos los trabajos continúan.
- Al salir, se solicita cancelación, se termina cualquier auxiliar propio y se vacían las escrituras pendientes.
- Ante cierre inesperado, el siguiente inicio reconcilia pruebas incompletas y archivos temporales huérfanos.

## 5. Rutas previstas

```text
C:\Program Files\SmartDisk Monitor\
  SmartDiskMonitor.exe
  bin\smartctl.exe
  licenses\

C:\ProgramData\SmartDisk Monitor\
  smartdisk.db
  logs\
  raw\
  diagnostics\
  backups\
```

La ruta de datos se obtiene de Windows y no se codifica como literal en la lógica de negocio.

## 6. Seguridad

- Manifiesto Windows `requireAdministrator`.
- Política de capacidades Tauri con mínimo privilegio dentro del proceso elevado.
- Sin shell genérica expuesta al frontend.
- Lista permitida de operaciones y argumentos de smartctl.
- Validación canónica de rutas antes de crear o eliminar archivos de prueba.
- Límites de tamaño y reserva de espacio antes de escribir.
- SQL parametrizado y migraciones verificadas.
- Contenido procedente de eventos o dispositivos renderizado como texto, nunca como HTML sin sanear.
- Sin endpoints de red ni telemetría.

## 7. Tolerancia a fallos

- Cada fuente devuelve estado `ok`, `partial`, `unsupported`, `timeout` o `error`.
- Un fallo parcial no degrada automáticamente la salud del disco.
- Se conserva la última lectura válida y se muestra su antigüedad.
- Los procesos auxiliares tienen timeout y salida limitada.
- Las capturas brutas con errores se conservan con rotación para diagnóstico.
- Los cambios del reloj no duplican eventos gracias a identificadores/marcadores del canal.

## 8. Compatibilidad extensible

Las fuentes implementarán interfaces internas para permitir en el futuro:

- ejecutores de fabricantes RAID;
- Linux y macOS;
- servicio de Windows;
- almacenamiento central;
- nuevas fuentes de alertas.

Estas extensiones no forman parte de la versión 1.0.

## 9. Pruebas técnicas

- Unitarias para parsers, normalización, reglas y retención.
- Datos fixture anonimizados de ATA, NVMe, USB, RAID y VM.
- Integración con un `smartctl` falso para salidas, timeouts y errores.
- Integración SQLite y migraciones desde cada versión soportada.
- Pruebas de rutas seguras y limpieza de benchmark.
- Pruebas manuales en hardware real sin exigir una marca determinada.
- Pruebas de interfaz para flujos críticos y accesibilidad.

---

## Archivo de origen: `docs/data-model.md`

# Modelo de datos

## 1. Principios

- Separar identidad física, topología mutable y observaciones temporales.
- Conservar procedencia y calidad de cada métrica.
- Guardar contadores absolutos para calcular incrementos de forma fiable.
- No asumir que una letra de unidad identifica un disco.
- No marcar como fallo la ausencia de una métrica.

## 2. Entidades principales

### `devices`

- `id`: UUID interno.
- `fingerprint`: identidad calculada estable.
- `serial_number`: nullable.
- `model`, `manufacturer`, `firmware`.
- `device_type`: NVMe, SATA SSD, HDD, USB, virtual, RAID logical, unknown.
- `bus_type` y `smartctl_path`.
- `capacity_bytes`.
- `alias`.
- `monitoring_enabled`.
- `first_seen_at`, `last_seen_at`, `removed_at`.
- `capabilities_json`.

### `volumes`

- `id`, `volume_guid`, `label`, `filesystem`.
- `drive_letters_json`.
- `capacity_bytes`, `free_bytes`.
- `device_mapping_confidence`.
- `first_seen_at`, `last_seen_at`.

### `device_volume_links`

- Relación entre discos físicos y volúmenes.
- Permite múltiples discos por volumen y múltiples volúmenes por disco.
- Incluye procedencia y confianza de la asociación.

### `metric_samples`

- `device_id` o `volume_id`.
- `metric_key`, `value_real`, `value_integer`, `unit`.
- `sampled_at_utc`.
- `source`: smartctl, Windows Storage, performance counter, filesystem.
- `quality`: exact, inferred, vendor_specific, stale.
- `resolution`: raw, five_minutes, hourly.

### `smart_snapshots`

- `device_id`, `captured_at_utc`.
- Campos normalizados de salud.
- `smartctl_version`, `exit_status` y estado de consulta.
- Ruta o contenido comprimido de JSON bruto cuando proceda.

### `system_events`

- Identidad estable del canal/registro.
- `occurred_at_utc`, `provider`, `event_id`, `level`.
- `message`, `raw_xml` opcional.
- Disco o volumen asociado y confianza de asociación.
- Hash de deduplicación.

### `alert_groups`

- `id`, `deduplication_key`, `rule_key`.
- Objeto afectado.
- Severidad y estado.
- Primera y última ocurrencia.
- Contador.
- Fechas de reconocimiento, resolución, archivo y silencio.
- Último valor y contexto.

### `alert_occurrences`

- `alert_group_id`, fecha, valor, evento y contexto de la ocurrencia.

### `test_runs`

- Tipo: benchmark, chkdsk_scan o smart_short.
- Disco/volumen objetivo.
- Estado: pending, running, cancelling, completed, failed, cancelled, interrupted.
- Inicio, fin, progreso y resultado.
- Parámetros y resumen de métricas.
- Ruta temporal solo mientras sea necesaria.

### `settings`

- Claves tipadas y versionadas.
- Preferencias globales, de disco y de volumen.
- Idioma, tema, frecuencias, retención, cierre y umbrales.

### `event_cursors`

- Canal/proveedor y último identificador procesado.
- Evita duplicar eventos entre sesiones.

### `schema_migrations`

- Versión, fecha y checksum de cada migración aplicada.

## 3. Métricas normalizadas iniciales

- `temperature_celsius`
- `temperature_sensor_N_celsius`
- `critical_warning`
- `health_passed`
- `percentage_used`
- `available_spare_percent`
- `available_spare_threshold_percent`
- `power_on_hours`
- `power_cycles`
- `unsafe_shutdowns`
- `media_errors_total`
- `error_log_entries_total`
- `data_read_bytes_total`
- `data_written_bytes_total`
- `read_bytes_per_second`
- `write_bytes_per_second`
- `read_latency_ms`
- `write_latency_ms`
- `activity_percent`
- `volume_free_bytes`
- `volume_free_percent`

Los campos no disponibles se omiten; no se almacenan como cero.

## 4. Retención

- Un trabajo diario compacta muestras antiguas dentro de una transacción.
- La agregación conserva mínimo, máximo, promedio, primera y última lectura, además de incrementos de contadores.
- Antes de una migración se crea una copia consistente de SQLite.
- Se conservan las tres copias de migración más recientes.
- Alertas, ocurrencias críticas, eventos vinculados y ejecuciones de pruebas no se borran automáticamente.

## 5. Tiempo y unidades

- Persistencia en UTC.
- Presentación en hora local del sistema.
- Capacidades almacenadas en bytes y temperaturas en Celsius.
- La interfaz puede presentar unidades legibles sin alterar el dato original.

---

## Archivo de origen: `docs/user-stories.md`

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

## Épica B. Inventario y salud

### US-010 — Descubrir almacenamiento (P0)

Como usuario quiero ver discos físicos, particiones y volúmenes para entender la topología del equipo.

Criterios de aceptación:

- Se enumeran NVMe, SSD SATA y HDD accesibles.
- Se muestran modelo, serie, firmware, interfaz, capacidad y volúmenes asociados cuando estén disponibles.
- No se confunde una letra de unidad con la identidad física.
- Un RAID, USB o disco virtual sin SMART aparece como no compatible o parcialmente compatible, no como averiado.

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

### US-013 — Actualizar manualmente (P1)

Como usuario quiero forzar una lectura para comprobar un cambio inmediatamente.

Criterios de aceptación:

- Existe “Actualizar ahora”.
- No inicia dos recopilaciones iguales concurrentemente.
- La UI muestra progreso o actividad sin bloquearse.
- Los fallos parciales identifican la fuente afectada.

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

### US-022 — Conservar y compactar historial (P1)

Como usuario quiero disponer de al menos 30 días de historial sin crecimiento ilimitado de datos detallados.

Criterios de aceptación:

- Se cumplen las retenciones definidas en la especificación.
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
- Puede silenciarse 15 minutos, 1 hora, 8 horas o indefinidamente.
- La resolución automática conserva el historial.
- Archivar retira la alerta de la vista principal.

### US-032 — Estado en systray (P0)

Como usuario quiero reconocer el estado global desde la bandeja del sistema.

Criterios de aceptación:

- El color refleja la mayor severidad activa.
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

### US-061 — Ver información de la aplicación (P1)

Como usuario quiero consultar versión, autor y licencias.

Criterios de aceptación:

- El botón `?` abre “Acerca de”.
- Nombre y versión se obtienen dinámicamente.
- Muestra autor, MIT, licencias de terceros y repositorio.
- Permite copiar información diagnóstica no sensible.

---

## Archivo de origen: `docs/roadmap.md`

# Roadmap y backlog

## Criterio de priorización

- P0: necesario para considerar utilizable la versión 1.0.
- P1: debe entrar en 1.0 salvo riesgo técnico demostrado.
- P2: candidato a versiones posteriores.

## Fase 0 — Validación técnica

Objetivo: reducir riesgos antes de construir la interfaz completa.

- Crear esqueleto Tauri 2 + Svelte + TypeScript + Tailwind.
- Probar elevación UAC y empaquetado x64.
- Validar `smartctl --scan-open --json` en NVMe, SATA y USB disponibles.
- Interpretar correctamente los bits del código de salida de smartctl.
- Probar lectura de eventos y marcadores de Windows.
- Probar contadores de rendimiento y mapeo disco-volumen.
- Validar SQLite WAL en `ProgramData`.
- Prototipar systray y cierre hacia bandeja.
- Documentar cumplimiento de redistribución de smartmontools.

Salida: informe de viabilidad y fixtures anonimizados.

## Versión 0.1 — Inventario y recopilación

- Inventario físico y lógico.
- Selección y alias de discos.
- Colector SMART normalizado y vista JSON.
- Contadores de Windows y capacidad.
- Persistencia SQLite y migraciones.
- Planificador y estados de fuente.
- Tests unitarios de parsers.

## Versión 0.2 — Panel e historial

- Panel general.
- Detalle de disco y volumen.
- Gráficas e intervalos.
- Registro de eventos con deduplicación.
- Español e inglés.
- Tema claro, oscuro y de sistema.
- Asistente inicial.

## Versión 0.3 — Alertas y systray

- Motor de reglas.
- Agrupación y ciclo de vida de alertas.
- Notificaciones de Windows.
- Estados y menú de systray.
- Pausa y reanudación.
- Preferencia de cierre recordable.

## Versión 0.4 — Pruebas e informes

- Benchmark de archivo temporal.
- CHKDSK `/scan`.
- Autotest SMART corto.
- CSV, JSON y HTML.
- ZIP diagnóstico anonimizado.

## Versión 0.9 — Endurecimiento

- Retención, agregación y copias pre-migración.
- Recuperación tras cierre inesperado.
- Límites, timeouts y cancelación.
- Pruebas en Windows cliente y servidor.
- Accesibilidad y revisión de traducciones.
- Logs rotatorios.
- Documentación operativa y de privacidad.
- Auditoría de dependencias y licencias.

## Versión 1.0

- Instalador para todos los usuarios.
- Desinstalación que conserva datos.
- Aviso documentado de SmartScreen por falta de firma.
- Licencia MIT, terceros y atribuciones.
- Nombre y versión dinámicos.
- Release manual en GitHub.

## Después de 1.0 (P2)

- Firma de código.
- Actualizaciones manualmente comprobables o firmadas.
- Autotest SMART extendido.
- Integraciones RAID: storcli/perccli/HPE.
- Servicio opcional sin sesión iniciada.
- Soporte Linux/macOS.
- ARM64.
- Consola central opcional.
- Reglas configurables por dispositivo.
- Exportación PDF nativa.

## Definición de terminado

Una historia se considera terminada cuando:

- cumple todos sus criterios de aceptación;
- tiene pruebas automatizadas proporcionales al riesgo;
- los errores y estados vacíos están diseñados;
- el texto existe en español e inglés;
- respeta accesibilidad básica de teclado y contraste;
- no introduce permisos Tauri genéricos innecesarios;
- actualiza documentación y avisos de terceros cuando corresponde;
- ha sido verificada en una compilación empaquetada, no solo en desarrollo.

---

## Archivo de origen: `docs/decisions.md`

# Registro de decisiones técnicas

## ADR-001 — Tauri 2 para escritorio

Estado: aceptada.

Se utilizará Tauri 2 por su integración nativa, tamaño razonable, backend Rust, systray y capacidad de empaquetar un ejecutable auxiliar. La primera plataforma es Windows x64.

## ADR-002 — Svelte en lugar de React

Estado: aceptada.

La interfaz usará Svelte + TypeScript. React no se incluirá. Utilizar simultáneamente ambos frameworks duplicaría responsabilidades sin una necesidad funcional.

Tailwind CSS proporcionará utilidades de estilo sobre tokens reemplazables. La identidad inicial es provisional y debe poder sustituirse por el futuro lenguaje de diseño.

## ADR-003 — Aplicación local sin servicio

Estado: aceptada.

No existirá agente, servidor central ni servicio de Windows. La recopilación ocurre solo con sesión iniciada y aplicación activa, aunque la ventana esté en la bandeja.

## ADR-004 — Elevación de todo el proceso

Estado: aceptada.

El ejecutable solicitará `requireAdministrator`. Simplifica el acceso a dispositivos y eventos, aceptando que aparezca UAC en cada inicio y que toda la UI viva dentro de un proceso elevado.

Mitigación: capacidades Tauri mínimas, sin shell genérica desde JavaScript y comandos privilegiados cerrados en Rust.

## ADR-005 — smartctl como auxiliar independiente

Estado: aceptada.

`smartctl` será la fuente principal de SMART/NVMe y se ejecutará como binario independiente con salida JSON. Se incluirá en el instalador manteniendo avisos, licencia y obligaciones aplicables de smartmontools. El código propio conserva licencia MIT.

La API nativa de Windows será complementaria y permitirá contrastar topología, volúmenes, actividad y datos de fiabilidad disponibles.

## ADR-006 — SQLite local

Estado: aceptada.

SQLite almacenará configuración, inventario, muestras, eventos, alertas y pruebas en `ProgramData`. Se utilizarán WAL, migraciones versionadas, transacciones breves y copias previas a migraciones.

## ADR-007 — Sin red ni telemetría

Estado: aceptada.

El funcionamiento normal no necesita red. No se recopila ni transmite telemetría. Las actualizaciones son totalmente manuales.

## ADR-008 — Identidad por dispositivo físico

Estado: aceptada.

La clave de presentación no será la letra de unidad ni el número de disco mutable. Se usará el número de serie y, cuando falte, una huella calculada con atributos estables, conservando el grado de confianza.

## ADR-009 — Benchmarks basados en archivos

Estado: aceptada.

Las pruebas de escritura no acceden a bloques sin formato. Utilizan un archivo temporal nuevo, limitado y verificable dentro de un volumen montado. Se reservan espacio y umbral térmico, se permite cancelación y se intenta una limpieza segura.

## ADR-010 — Alertas por estado y cambio

Estado: aceptada.

El motor combina límites absolutos con incrementos de contadores y persistencia temporal. Las alertas se agrupan para reducir ruido, pero cada ocurrencia conserva su hora.

## ADR-011 — Versionado como fuente única

Estado: aceptada.

Nombre y versión proceden de los manifiestos del proyecto durante compilación y ejecución. La UI, “Acerca de”, informes e instalador no mantendrán copias manuales independientes.

## ADR-012 — Datos persistentes tras desinstalar

Estado: aceptada.

El desinstalador conserva SQLite, configuración, historial y logs en `ProgramData`. La aplicación ofrece una acción separada y confirmada para eliminarlos, y la documentación explica la limpieza manual.

---

## Archivo de origen: `LICENSE`

MIT License

Copyright (c) 2026 Daniel Diez Mardomingo

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## Archivo de origen: `THIRD_PARTY_NOTICES.md`

# Third-party notices

This file will be completed from the exact dependency lockfiles and bundled artifacts before the first distributed build.

## smartmontools / smartctl

SmartDisk Monitor is designed to invoke `smartctl` as a separate executable.

- Project: https://www.smartmontools.org/
- License: GNU General Public License, version 2 or later
- Distribution status: planned; exact binary version not yet selected

When a binary is added, this distribution must include the corresponding copyright and license text and satisfy the source-code obligations applicable to that binary. This notice does not replace those materials.

## Application dependencies

Rust and JavaScript dependency notices will be generated and reviewed from the locked dependency graph before release. No dependency is yet vendored at the specification stage.

