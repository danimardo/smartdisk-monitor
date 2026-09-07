# SmartDisk Monitor — documentación consolidada

Reúne **todo el material normativo** del proyecto en un solo documento: especificación funcional,
arquitectura, modelo de datos, reglas de alerta, contrato entre interfaz y backend, decisiones
técnicas, reglas de interfaz y los ficheros que actúan como contrato literal (tokens, tipos,
formateadores e i18n).

Se genera con `python tools/build-historias.py` a partir de los ficheros del repositorio, que siguen
siendo la referencia editable. **No lo edites a mano**: los cambios se perderían en la siguiente
regeneración.

## Cómo leerlo

| Si buscas… | Ve a |
|---|---|
| Qué hace el producto y qué queda fuera | Especificación del producto |
| Qué hay que construir, con criterios de aceptación | Historias de usuario |
| Cómo se estructura por dentro | Arquitectura, Modelo de datos |
| Cuándo salta una alerta y cuándo se resuelve | Reglas de alerta |
| Qué comandos y eventos existen entre UI y backend | Contrato UI ↔ backend |
| Por qué se decidió algo | Registro de decisiones |
| Qué se midió y qué sigue sin decidirse | Cuestiones abiertas y mediciones |
| Cómo se escribe una pantalla | Sistema de diseño: reglas de interfaz (VINCULANTES) |
| Dónde vive cada pieza del sistema de diseño | Sistema de diseño, §0 «Dónde vive cada cosa» |
| Qué hay que probar, a qué nivel y cuándo | Estrategia integral de testing |

## Precedencia

Si dos documentos se contradicen, mandan en este orden:

1. **Reglas de alerta** sobre el resumen de alertas de la especificación.
2. **Contrato UI ↔ backend** sobre cualquier descripción informal de comandos.
3. **Sistema de diseño** (`docs/ui-design.md`) sobre cualquier criterio visual escrito en otro sitio.
4. **Cuestiones abiertas** sobre todo lo demás para lo que registre una decisión: recoge las
   correcciones posteriores, varias de ellas nacidas de medir sobre un Windows real.

Las entradas marcadas `PROPUESTO` en Cuestiones abiertas son valores por defecto adoptados para no
bloquear el trabajo, no decisiones cerradas. Las marcadas `ABIERTO` bloquean la historia que las
cita. Ninguna implementación debería tener que asumir nada que no esté ahí; si aparece algo, se
añade en vez de resolverlo en el código.


## Índice

1. [Presentación del proyecto](#1-presentación-del-proyecto) · `README.md`
2. [Especificación del producto](#2-especificación-del-producto) · `docs/product-specification.md`
3. [Historias de usuario](#3-historias-de-usuario) · `docs/user-stories.md`
4. [Roadmap y backlog](#4-roadmap-y-backlog) · `docs/roadmap.md`
5. [Arquitectura](#5-arquitectura) · `docs/architecture.md`
6. [Modelo de datos](#6-modelo-de-datos) · `docs/data-model.md`
7. [Reglas de alerta](#7-reglas-de-alerta) · `docs/alert-rules.md`
8. [Contrato UI ↔ backend](#8-contrato-ui-backend) · `docs/ui-contract.md`
9. [Convenciones de ingeniería](#9-convenciones-de-ingeniería) · `docs/engineering-conventions.md`
10. [Estrategia integral de testing](#10-estrategia-integral-de-testing) · `docs/testing-strategy.md`
11. [Registro de decisiones técnicas](#11-registro-de-decisiones-técnicas) · `docs/decisions.md`
12. [Cuestiones abiertas y mediciones](#12-cuestiones-abiertas-y-mediciones) · `docs/open-questions.md`
13. [Sistema de diseño: reglas de interfaz (VINCULANTES)](#13-sistema-de-diseño-reglas-de-interfaz-vinculantes) · `docs/ui-design.md`
14. [Bocetos del sistema de diseño](#14-bocetos-del-sistema-de-diseño) · `design/README.md`
15. [Sistema de diseño: principios](#15-sistema-de-diseño-principios) · `src/design-system/README.md`
16. [tokens.css — fuente única de verdad visual](#16-tokenscss-fuente-única-de-verdad-visual) · `src/design-system/tokens.css`
17. [tokens.json — los mismos tokens, para herramientas](#17-tokensjson-los-mismos-tokens-para-herramientas) · `src/design-system/tokens.json`
18. [tailwind.config.cjs — mapeo de tokens](#18-tailwindconfigcjs-mapeo-de-tokens) · `tailwind.config.cjs`
19. [components/index.ts — el catálogo cerrado](#19-componentsindexts-el-catálogo-cerrado) · `src/lib/components/index.ts`
20. [design/types.ts — vocabulario de la UI](#20-designtypests-vocabulario-de-la-ui) · `src/lib/design/types.ts`
21. [design/health.ts — estado → color, umbrales](#21-designhealthts-estado-color-umbrales) · `src/lib/design/health.ts`
22. [design/format.ts — formato de presentación](#22-designformatts-formato-de-presentación) · `src/lib/design/format.ts`
23. [design/theme.svelte.ts — tema](#23-designthemesveltets-tema) · `src/lib/design/theme.svelte.ts`
24. [design/accent.ts — acento de Windows](#24-designaccentts-acento-de-windows) · `src/lib/design/accent.ts`
25. [i18n/index.ts — idioma, formato y plurales](#25-i18nindexts-idioma-formato-y-plurales) · `src/lib/i18n/index.ts`
26. [i18n/es.json](#26-i18nesjson) · `src/lib/i18n/es.json`
27. [i18n/en.json](#27-i18nenjson) · `src/lib/i18n/en.json`
28. [Tipografía empotrada](#28-tipografía-empotrada) · `src/design-system/fonts/README.md`
29. [smartctl redistribuido](#29-smartctl-redistribuido) · `third-party/smartmontools/README.md`
30. [Licencia del código propio](#30-licencia-del-código-propio) · `LICENSE`
31. [Avisos de terceros](#31-avisos-de-terceros) · `THIRD_PARTY_NOTICES.md`


---

# 1. Presentación del proyecto

Fichero de origen: `README.md`

**SmartDisk Monitor** vigila la salud de los discos de un equipo Windows: temperatura, desgaste,
errores, actividad, capacidad y los eventos del sistema relacionados con almacenamiento, todo en
una sola pantalla y con historial. Es una aplicación local y de un solo usuario — no hay servidor
central, cuentas ni telemetría — pensada para quien administra su propio equipo y quiere enterarse
de un disco que empieza a fallar antes de perder datos, no después.

Responde a las preguntas que uno se hace cuando algo empieza a ir mal:

- ¿Qué discos y volúmenes hay en este equipo, y cómo están ahora mismo?
- ¿Cómo ha evolucionado la temperatura o el desgaste en las últimas horas o días?
- ¿Ha pasado algo raro, cuándo, y con qué frecuencia se repite?
- ¿Este disco aguanta bien la carga, o se degrada?
- ¿Qué le enseño a quien me ayude a diagnosticarlo, sin tener que compartir todo el disco?

### Funcionalidades

- **Inventario automático** de discos físicos, particiones y volúmenes, con selección de cuáles
  monitorizar.
- **Salud SMART/NVMe** vía `smartctl` y las fuentes nativas de Windows: temperatura, desgaste,
  errores, horas de encendido y demás contadores del fabricante, con degradación elegante (gris, no
  alerta) cuando el hardware no expone SMART — RAID, USB, máquinas virtuales.
- **Panel general y detalle por disco**, con gráficas de la serie temporal y zonas de aviso/crítico
  sobre el propio gráfico: se ve de un vistazo si un valor es bueno o peligroso, no solo la cifra.
- **Actividad, velocidad y latencia** en tiempo real mediante los contadores de rendimiento de
  Windows.
- **Alertas locales**, agrupadas y con histéresis para no repetir aviso por cada muestra, visibles
  desde la aplicación y desde el icono de la bandeja del sistema.
- **Captura y correlación de eventos** relevantes del registro de eventos de Windows con el disco al
  que corresponden.
- **Historial local en SQLite**, con retención configurable y sin ningún dato saliendo del equipo.
- **Pruebas manuales bajo demanda**: benchmark de lectura/escritura, `chkdsk /scan` y autotest SMART
  corto, cuando el dispositivo lo soporte.
- **Exportación e informes**: CSV, JSON y HTML imprimible, más un paquete ZIP de diagnóstico
  anonimizado por defecto — para compartir un incidente sin compartir números de serie.
- **Español e inglés**, tema claro/oscuro/automático según el sistema.

Fuera de alcance a propósito (por ahora): servicio en segundo plano sin sesión iniciada, consola
remota, cuentas o roles, alertas por correo/mensajería, actualizaciones automáticas y telemetría.
El detalle completo está en [`docs/product-specification.md`](docs/product-specification.md).

### Descargar

Instalador para Windows x64 en la [página de Releases](https://github.com/danimardo/smartdisk-monitor/releases).

Requiere Windows 10 (1809+), Windows 11 o Windows Server 2016+ con Experiencia de escritorio, y el
**WebView2 Runtime** (el instalador lo resuelve sin conexión si falta). Se ejecuta siempre con
privilegios de administrador: es lo que necesita para leer SMART de los discos físicos.

El instalador **no va firmado** — Windows SmartScreen mostrará "Windows protegió tu PC"/"editor no
reconocido" la primera vez que se ejecute. Es un aviso esperado, no un fallo: para continuar, "Más
información" → "Ejecutar de todas formas". Conseguir un certificado de firma de código es trabajo
pendiente, ya recogido en [`docs/roadmap.md`](docs/roadmap.md).

### Desarrollo

```sh
pnpm install          # requiere Node 20+ y pnpm
pnpm app:dev          # levanta Vite y la aplicación Tauri
```

`pnpm app:dev` muestra el diálogo de UAC en cada arranque: la aplicación se ejecuta elevada por
necesidad de acceso a los dispositivos (ADR-004). Las pruebas no lo necesitan, salvo la suite de
aplicación real.

| Comando | Qué hace |
|---|---|
| `pnpm app:dev` | Aplicación completa en modo desarrollo |
| `pnpm app:build` | Instalador NSIS con WebView2 sin conexión |
| `pnpm check` | Tipos y accesibilidad del frontend |
| `pnpm test` | Lógica del frontend, en Node |
| `pnpm test:component` | Componentes en un Chromium real |
| `pnpm test:e2e` | Interfaz completa con Playwright y el IPC simulado |
| `pnpm test:a11y` | Accesibilidad con axe, seis pantallas por dos temas |
| `pnpm verify` | Recursos redistribuidos, tokens del diseño e i18n |
| `pnpm docs:build` | Regenera `historias.md` |
| `cargo test` / `cargo clippy` | Backend, desde `src-tauri/` |

`pnpm verify` es el que impide que el sistema de diseño se erosione: comprueba los hashes de la
tipografía y de `smartctl`, que ningún componente lleve colores o radios literales, y que los dos
diccionarios estén sincronizados.

Antes de escribir código conviene leer [`docs/open-questions.md`](docs/open-questions.md): recoge
todo lo que la especificación dejaba a interpretación y el valor que se ha adoptado en cada caso.
Las entradas marcadas `PROPUESTO` son valores por defecto pendientes de revisión; las marcadas
`ABIERTO` bloquean la historia que las cita.

[`historias.md`](historias.md) reúne todo el material normativo en un solo documento, pensado para
entregárselo entero a una herramienta de generación o a quien se incorpore al proyecto. Se genera
con `python tools/build-historias.py` y no se edita a mano.

El sistema de diseño es vinculante para la implementación de la interfaz: las reglas están en
[`docs/ui-design.md`](docs/ui-design.md), los tokens en `src/design-system/tokens.css`, el catálogo
de componentes en `src/lib/components/` y los bocetos aprobados en [`design/`](design/). El §0 de
`ui-design.md` es el mapa completo.

### Documentación

- [Especificación del producto](docs/product-specification.md)
- [Arquitectura](docs/architecture.md)
- [Modelo de datos](docs/data-model.md)
- [Historias de usuario](docs/user-stories.md)
- [Reglas de alerta](docs/alert-rules.md)
- [Contrato UI ↔ backend](docs/ui-contract.md)
- [Convenciones de ingeniería](docs/engineering-conventions.md)
- [Estrategia de testing](docs/testing-strategy.md)
- [Cuestiones abiertas](docs/open-questions.md)
- [Backlog y versiones](docs/roadmap.md)
- [Decisiones técnicas](docs/decisions.md)
- [Fallos conocidos y silencios](docs/known-issues.md)
- [Constitución del proyecto](.specify/memory/constitution.md)
- [Instrucciones para agentes de IA](AGENTS.md)
- [Todo lo anterior en un solo documento](historias.md)
- [Sistema de diseño: reglas vinculantes de interfaz](docs/ui-design.md)
- [Bocetos aprobados](design/README.md)

### Identidad del proyecto

- Producto: SmartDisk Monitor
- Autor: Daniel Diez Mardomingo
- Repositorio: https://github.com/danimardo/smartdisk-monitor
- Licencia del código propio: MIT
- Plataforma: Windows x64

### Instalación y desinstalación

El instalador (`pnpm app:build`, NSIS) resuelve WebView2 sin conexión (ADR-020) y registra la
aplicación para todos los usuarios. La desinstalación **conserva** el historial y la configuración
(ADR-012): quita el ejecutable, los accesos directos y el registro de Windows, pero no toca
`%ProgramData%\SmartDisk Monitor\`, donde vive todo lo demás —

- `smartdisk.sqlite`: inventario, métricas, alertas, eventos y ejecuciones de pruebas.
- `logs\`: registro de actividad (`smartdisk.log.<fecha>`).

Esto es deliberado: una desinstalación accidental o para reinstalar una versión más reciente no
debe borrar meses de historial. La carpeta la crea el instalador con una ACL propia (ADR-026,
`Get-Acl`/`icacls` — lectura abierta, escritura restringida a administradores), así que solo una
cuenta con privilegios de administrador puede borrarla a mano.

**Para limpiar todo sin desinstalar** —vía normal, con confirmación explícita y sin tocar el
sistema de archivos a mano—: Ajustes → «Borrar todos los datos» (US-073). Exige escribir el nombre
de la aplicación como frase de confirmación y deja la aplicación como recién instalada.

**Para limpiar todo después de haber desinstalado ya** —o antes de reinstalar en un equipo que no
va a volver a usarse—: borrar a mano, desde una sesión con privilegios de administrador,
`%ProgramData%\SmartDisk Monitor\` completa. No queda ningún otro rastro: la aplicación no escribe
en el registro de Windows más allá de lo que el propio instalador NSIS gestiona, ni en `%AppData%`
ni en el perfil del usuario.


---

# 2. Especificación del producto

Fichero de origen: `docs/product-specification.md`

### 1. Propósito

SmartDisk Monitor es una aplicación de escritorio local que permite conocer la salud, temperatura, actividad, rendimiento, capacidad y eventos relevantes de los discos instalados en un equipo Windows. Está orientada a uso personal por un administrador y funciona únicamente mientras exista una sesión iniciada y la aplicación permanezca ejecutándose.

El producto debe ayudar a responder:

- ¿Qué discos físicos y volúmenes existen en el equipo?
- ¿Cuál es su estado actual y cómo ha evolucionado?
- ¿Ha ocurrido alguna anomalía, cuándo sucedió y cuántas veces se repitió?
- ¿Se degrada un disco bajo carga o por temperatura?
- ¿Qué información se puede exportar para analizar o compartir un incidente?

### 2. Alcance de la versión 1.0

#### Incluido

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

#### Fuera del alcance inicial

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

### 3. Experiencia principal

#### Primera ejecución

1. Windows solicita elevación UAC.
2. La aplicación muestra un asistente inicial.
3. Se detectan los discos y se explica qué datos están disponibles.
4. Todos los discos compatibles quedan seleccionados inicialmente.
5. El usuario puede excluir discos y asignar alias.
6. Se presentan los umbrales y frecuencias predeterminados.
7. Comienza la monitorización y se abre el panel general.

#### Panel general

Debe mostrar de un vistazo:

- estado global del equipo;
- número de discos correctos, con advertencias, críticos y desconocidos;
- alertas activas y sucesos recientes;
- tarjetas de discos con temperatura, salud, actividad y capacidad;
- acceso a gráficas, eventos, pruebas, informes y configuración.

#### Bandeja del sistema

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

### 4. Frecuencias predeterminadas

- Temperatura, actividad, capacidad y latencia: cada 30 segundos.
- SMART completo: cada 5 minutos.
- Eventos de Windows: cada 30 segundos, usando un marcador persistente para no duplicarlos.
- Detección de discos añadidos o retirados: cada minuto.
- Botón para forzar una actualización completa.
- En batería se reduce la frecuencia de métricas no críticas, pero no se suspenden alertas graves.

Todas las frecuencias son configurables dentro de estos límites, que valida el backend:

| Trabajo | Por defecto | Mínimo | Máximo |
|---|---|---|---|
| Temperatura, actividad, capacidad, latencia | 30 s | 10 s | 5 min |
| SMART completo | 5 min | 1 min | 60 min |
| Eventos de Windows | 30 s | 15 s | 5 min |
| Detección de altas y bajas | 60 s | 30 s | 10 min |

En batería se multiplica por cuatro el intervalo de las métricas rápidas y el de detección de altas
y bajas. SMART completo y eventos de Windows no se alteran: son las fuentes de las alertas graves.
Al volver a la red eléctrica se restauran de inmediato y se fuerza un ciclo completo.

**Pausar** detiene la recopilación y la evaluación de reglas, y con ellas las notificaciones. No
sobrevive a un reinicio: arrancar la aplicación siempre reanuda, porque una pausa olvidada es un
monitor que no vigila y no lo dice. Mientras está pausada, la barra de herramientas lo anuncia de
forma permanente.

### 5. Alertas

#### Estados

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
cooldown y textos de cada regla— vive en [`alert-rules.md`](docs/alert-rules.md). Lo que sigue es el
resumen; si discrepan, manda aquel documento.

#### Reglas iniciales

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

#### Notificaciones

- Centro de alertas dentro de la aplicación.
- Notificación nativa de Windows cuando la aplicación está minimizada. Se puede desactivar por
  completo (`notifications.enabled`, activada de fábrica): la alerta sigue en la lista, solo deja de
  aparecer la ventana emergente. Distinto de pausar, que además detiene la recopilación.
- Sin canales externos.
- Sonido desactivado inicialmente.
- Silencio temporal de 15 minutos, 1 hora, 8 horas o indefinido hasta reactivación manual.

### 6. Pruebas manuales

#### Prueba de lectura y escritura

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

#### CHKDSK

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

#### Autotest SMART corto

- Solo manual y únicamente si el dispositivo declara compatibilidad.
- Muestra duración estimada, progreso y resultado cuando estén disponibles.
- Puede cancelarse si el dispositivo lo admite.
- Advierte de la posible degradación temporal del rendimiento.
- No se permite simultáneamente con el benchmark de la aplicación.
- No se trata la falta de compatibilidad como una anomalía.

### 7. Historial y retención

- Muestras detalladas: 7 días.
- Agregados de 5 minutos: 30 días como mínimo.
- Resúmenes horarios: un año.

Los 30 días de historial que promete US-022 son de **agregados**, no de detalle. Toda gráfica
declara la resolución que está mostrando: un máximo promediado no es un pico, y confundirlos al
investigar un incidente térmico llevaría a conclusiones falsas. La correspondencia entre intervalo
pedido y resolución servida está en [`open-questions.md`](docs/open-questions.md) §E.1.
- Alertas y eventos importantes: conservación indefinida hasta eliminación manual.
- Logs técnicos: 30 días y máximo aproximado de 100 MB mediante rotación.
- Retención configurable.
- El historial se identifica por número de serie y permanece separado cuando se sustituye un disco.
- Los datos permanecen en el equipo después de desinstalar.
- La aplicación permite borrarlos mediante una acción explícita con confirmación.

### 8. Internacionalización y apariencia

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
- [`ui-design.md`](docs/ui-design.md) es vinculante para cualquier implementación de interfaz; su §0 dice dónde vive cada pieza.
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

### 9. Informes y diagnóstico

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

### 10. Instalación y distribución

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

### 11. Privacidad y seguridad

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

### 12. Criterios globales de calidad

- Una avería de un colector no debe bloquear la interfaz.
- Los errores deben presentarse con lenguaje comprensible y conservar detalle técnico.
- Reiniciar la aplicación no debe duplicar eventos ya importados.
- Un disco no compatible debe aparecer como desconocido/no disponible, no como averiado.
- Todas las operaciones que generen carga o escriban datos requieren confirmación explícita.
- La aplicación debe seguir respondiendo durante recopilaciones, exportaciones y pruebas.
- Toda pantalla debe cumplir la definición de terminado de [`ui-design.md`](docs/ui-design.md) §8 en temas claro y oscuro y en el tamaño mínimo de ventana.
- Todo texto visible debe proceder del sistema i18n; no se admiten literales de interfaz fuera de los
  diccionarios español e inglés, incluidos `aria-label`, títulos y textos alternativos.
- La interfaz debe seguir siendo usable con veinte discos y con miles de eventos.
- Las decisiones que esta especificación deja abiertas se registran, con su valor adoptado, en
  [`open-questions.md`](docs/open-questions.md). Ningún implementador debería tener que asumir nada que
  no esté allí; si encuentra algo, se añade en vez de resolverlo en el código.


---

# 3. Historias de usuario

Fichero de origen: `docs/user-stories.md`

Prioridades: P0 imprescindible para 1.0; P1 importante para 1.0; P2 posterior si el calendario lo exige.

### Épica A. Inicio y configuración

#### US-001 — Ejecución elevada (P0)

Como usuario quiero que la aplicación solicite privilegios de administrador al iniciarse para poder consultar todos los dispositivos compatibles.

Criterios de aceptación:

- El ejecutable solicita elevación mediante UAC.
- Si la elevación se rechaza, la aplicación no continúa en un estado parcialmente funcional.
- Se muestra una explicación comprensible cuando Windows impide la elevación.

#### US-002 — Asistente inicial (P0) · **cubierta** (rediseño v3, `specs/002-rediseno-v3/` US8)

Como usuario quiero configurar la aplicación mediante un asistente para empezar a monitorizar sin conocer SMART.

Criterios de aceptación:

- Aparece cuando `settings.onboarding.completedAt` es nulo y no hay configuración previa; una
  instalación que ya venía configurada se marca como completada sin mostrarlo (FR-043).
- Cuatro pasos, uno por pantalla (Bienvenida · Discos · Alertas · Listo), sin riel ni barra de
  herramientas, con «Omitir y usar los valores de fábrica» visible en todos.
- Enumera los discos detectados y selecciona inicialmente todos los compatibles.
- Permite excluir discos y asignar alias.
- Explica los estados no compatible y desconocido; el disco USB sin SMART se explica como «no es una
  avería», nunca en rojo.
- El paso 3 elige un perfil de alerta (Prudente / Equilibrado / Solo lo grave) y ofrece la
  notificación de Windows y el autoarranque.
- «Omitir» aplica el perfil Equilibrado, graba la marca y va al panel.
- Las elecciones se conservan tras reiniciar; se relanza desde Ajustes → «Repetir la configuración
  inicial» sin borrar datos.

#### US-003 — Preferencias de idioma y tema (P1)

Como usuario quiero que idioma y tema sigan inicialmente el sistema y poder cambiarlos.

Criterios de aceptación:

- Español del sistema selecciona español; inglés selecciona inglés; los demás seleccionan inglés.
- El tema automático responde al tema de Windows.
- Se puede elegir español/inglés y claro/oscuro/sistema.
- El cambio se aplica sin reinstalar.

#### US-004 — Aplicar el sistema de diseño aprobado (P0)

Como usuario quiero una interfaz coherente, accesible y reconocible para interpretar el estado del almacenamiento sin ambigüedad.

Criterios de aceptación:

- La interfaz cumple [`ui-design.md`](docs/ui-design.md) y utiliza el catálogo cerrado de `src/lib/components/`.
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

### Épica B. Inventario y salud

#### US-010 — Descubrir almacenamiento (P0)

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

#### US-011 — Seleccionar discos (P0)

Como usuario quiero decidir qué discos monitorizar para excluir dispositivos irrelevantes.

Criterios de aceptación:

- Puede activarse o desactivarse cada disco.
- Los discos desactivados siguen visibles en una sección separada.
- Se pueden asignar y eliminar alias.
- La selección no se pierde si cambia una letra de unidad.

#### US-012 — Ver salud actual (P0)

Como usuario quiero ver un resumen de salud por disco para reconocer problemas rápidamente.

Criterios de aceptación:

- Se muestra estado, temperatura, desgaste y errores disponibles.
- Cada valor puede indicar fuente y fecha de última lectura.
- Los valores ausentes muestran “No disponible”, nunca cero inventado.
- Se diferencia correcto, advertencia, crítico, desconocido y no compatible.
- La presentación usa `StatusPill`, `StatusDot`, `MetricCard` y demás componentes aprobados según corresponda.

#### US-013 — Actualizar manualmente (P1)

Como usuario quiero forzar una lectura para comprobar un cambio inmediatamente.

Criterios de aceptación:

- Existe “Actualizar ahora”.
- No inicia dos recopilaciones iguales concurrentemente.
- La UI muestra progreso o actividad sin bloquearse.
- Los fallos parciales identifican la fuente afectada.

#### US-014 — Altas y bajas de discos en caliente (P0)

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

### Épica C. Historial y eventos

#### US-020 — Consultar gráficas (P0)

Como usuario quiero consultar gráficas históricas para detectar tendencias de temperatura, desgaste, actividad y errores.

Criterios de aceptación:

- Se pueden elegir disco, métrica e intervalo.
- Existen intervalos de 24 horas, 7 días, 30 días y personalizado.
- La zona horaria presentada es la local.
- Las discontinuidades se muestran como ausencia de datos, no como cero.

#### US-021 — Consultar eventos (P0)

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

#### US-022 — Conservar y compactar historial (P1)

Como usuario quiero disponer de al menos 30 días de historial sin crecimiento ilimitado de datos detallados.

Criterios de aceptación:

- Se cumplen las retenciones definidas en la especificación: 7 días de muestras detalladas, 30 días
  de agregados de 5 minutos y un año de resúmenes horarios.
- Los 30 días prometidos son de **agregados**, no de detalle, y la interfaz lo dice: toda gráfica
  declara la resolución que está mostrando, porque un máximo promediado no es un pico.
- La compactación conserva mínimos, máximos, promedios e incrementos.
- El usuario puede modificar la retención.
- Las operaciones de retención no bloquean perceptiblemente la UI.

### Épica D. Alertas

#### US-030 — Recibir alertas agrupadas (P0)

Como usuario quiero que sucesos repetidos formen una sola alerta para evitar ruido.

Criterios de aceptación:

- Eventos equivalentes incrementan un contador y actualizan la última ocurrencia.
- El detalle conserva todas las fechas y horas.
- Un cambio de severidad actualiza y vuelve a notificar el grupo.
- Recuperarse y recaer crea un nuevo ciclo identificable.

#### US-031 — Gestionar alertas (P0)

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

#### US-032 — Estado en systray (P0)

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

#### US-033 — Alertas de capacidad poco intrusivas (P1)

Como usuario quiero avisos de poco espacio sin recibir la misma notificación continuamente.

Criterios de aceptación:

- Solo se notifica al cruzar un umbral, cambiar de nivel o recaer tras recuperarse.
- La alerta muestra espacio y umbral actuales.
- Puede desactivarse por volumen.

### Épica E. Diagnóstico y pruebas

#### US-040 — Benchmark seguro (P0)

Como usuario quiero probar lectura y escritura mediante un archivo temporal para observar rendimiento y temperatura sin sobrescribir mis archivos.

Criterios de aceptación:

- Requiere confirmación y muestra impacto esperado.
- Nunca usa espacio sin asignar ni escribe directamente sobre el dispositivo.
- Crea un archivo nuevo con nombre no colisionable en el volumen elegido.
- Respeta la reserva de espacio y verifica el patrón leído.
- Muestra progreso, rendimiento, latencia y temperatura.
- Puede cancelarse y se detiene en el umbral térmico crítico.
- Intenta eliminar el archivo siempre y señala claramente cualquier residuo.

#### US-041 — Escanear sistema de archivos (P1)

Como usuario quiero ejecutar `chkdsk /scan` para comprobar un volumen sin programar una reparación fuera de línea.

Criterios de aceptación:

- Solo ofrece la operación en volúmenes compatibles.
- Muestra advertencia y comando antes de confirmar.
- Captura salida, resultado, inicio y fin.
- No ofrece opciones reparadoras en 1.0.

#### US-042 — Autotest SMART corto (P1)

Como usuario quiero solicitar el autotest corto del firmware para detectar fallos internos que un benchmark no revela.

Criterios de aceptación:

- Solo se habilita si el dispositivo declara soporte.
- Requiere confirmación y advierte del posible impacto de rendimiento.
- Muestra progreso y duración estimada cuando existan.
- Permite cancelar si el dispositivo lo admite.
- Conserva resultado e historial del autotest.
- No puede coincidir con el benchmark sobre el mismo disco.

#### US-043 — Ver datos avanzados (P1)

Como usuario avanzado quiero inspeccionar el JSON original de smartctl para investigar atributos no normalizados.

Criterios de aceptación:

- Existe una sección “Detalles avanzados”.
- Puede copiarse el JSON.
- Se indica fecha, versión de smartctl y comando lógico utilizado.
- La vista no expone capacidad de ejecutar comandos arbitrarios.

### Épica F. Exportación y soporte

#### US-050 — Exportar informes (P1)

Como usuario quiero exportar información para conservarla o compartirla.

Criterios de aceptación:

- Exporta CSV, JSON y HTML.
- Permite 24 horas, 7 días, 30 días y rango personalizado.
- El HTML es legible e imprimible.
- La exportación informa claramente de campos omitidos o no disponibles.

#### US-051 — Crear diagnóstico anonimizado (P1)

Como usuario quiero crear un ZIP de diagnóstico sin revelar identificadores personales por defecto.

Criterios de aceptación:

- Anonimiza series, equipo, usuarios y rutas personales inicialmente.
- La sustitución es consistente dentro del paquete.
- El usuario puede incluir identificadores expresamente.
- Antes de guardar se muestra un resumen del contenido.
- El paquete incluye el registro de actividad, sujeto a la misma anonimización que el resto del
  contenido (FR-029c).

### Épica G. Instalación y mantenimiento

#### US-060 — Instalar y desinstalar (P0)

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

#### US-061 — Ver información de la aplicación (P1)

Como usuario quiero consultar versión, autor y licencias.

Criterios de aceptación:

- El botón `?` abre “Acerca de”.
- Nombre y versión se obtienen dinámicamente.
- Muestra autor, MIT, licencias de terceros y repositorio.
- Permite copiar información diagnóstica no sensible.

### Épica H. Configuración y mantenimiento

Esta épica cubre la pantalla de Ajustes, que hasta ahora aparecía en la navegación, en el paquete de
diseño y en el roadmap sin una sola historia que la definiera.

#### US-070 — Ajustar frecuencias y umbrales (P1)

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

#### US-071 — Ajustar la retención y saber cuánto ocupa (P1)

Como usuario quiero saber cuánto espacio consume el historial y decidir cuánto se conserva para que
la aplicación no crezca sin control en mi disco.

Criterios de aceptación:

- Se muestra el tamaño actual de la base de datos, de los logs y de las capturas en bruto.
- Se pueden cambiar los tres periodos de retención dentro de límites seguros.
- Reducir una retención advierte de cuántos datos se van a eliminar **antes** de aplicarla.
- Alertas, ocurrencias críticas, eventos vinculados y ejecuciones de pruebas nunca se borran por
  retención, y la interfaz lo dice.
- La compactación se ejecuta sin bloquear la interfaz.
- Existe un interruptor de **modo detallado** de registro para reproducir un fallo con más
  información, y una acción que abre la carpeta donde reside el registro de actividad. No hay
  visor de registro dentro de la aplicación (spec 001-monitor-discos-windows, FR-029a/b).

#### US-072 — Configurar el arranque, el cierre y la apariencia (P1)

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

#### US-073 — Borrar todos los datos (P1)

Como usuario quiero poder eliminar todo el historial que la aplicación ha guardado sobre mis discos
para dejar el equipo limpio sin buscar carpetas a mano.

Criterios de aceptación:

- La acción vive en Ajustes, separada del resto y claramente marcada como irreversible.
- Exige escribir una frase de confirmación, no solo pulsar un botón.
- Enumera qué se va a borrar y qué se va a conservar antes de hacerlo.
- Al terminar, la aplicación queda como recién instalada y vuelve a mostrar el asistente inicial.
- La documentación explica cómo hacer lo mismo a mano después de desinstalar.

#### US-074 — Recuperarse de un cierre inesperado (P0)

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


---

# 4. Roadmap y backlog

Fichero de origen: `docs/roadmap.md`

**Estado real, 2026-09-05**: las ocho historias de usuario de
[`specs/001-monitor-discos-windows/`](docs/../specs/001-monitor-discos-windows/tasks.md) están
implementadas — inventario, panel e historial, alertas y systray, ajustes, pruebas y diagnóstico,
informes y exportación, instalación/reconocimiento/retirada. Lo que sigue en este documento sin
tachar es lo que de verdad queda abierto, no una plantilla sin actualizar. El seguimiento tarea a
tarea vive en `tasks.md`; este documento es el resumen por versión.

### Criterio de priorización

- P0: necesario para considerar utilizable la versión 1.0.
- P1: debe entrar en 1.0 salvo riesgo técnico demostrado.
- P2: candidato a versiones posteriores.

### Fase 0 — Validación técnica

Objetivo: reducir riesgos antes de construir la interfaz completa.

- Crear esqueleto Tauri 2 + SvelteKit (`adapter-static`, SSR off) + TypeScript + Tailwind (ADR-014).
- Añadir la tipografía Instrument Sans a `design-system/fonts/` con su licencia (ADR-018) y hacer que
  la compilación falle si falta.
- Integrar `src/design-system/tokens.css`, la configuración Tailwind, los módulos de diseño, i18n y el catálogo Svelte entregado.
- Validar los componentes entregados con Svelte 5 y el toolchain definitivo antes de modificarlos.
- Montar un shell navegable con `AppShell`, `Sidebar` y `Toolbar` siguiendo el boceto v2 aprobado.
- ~~Verificar temas claro/oscuro, acento de Windows, fallback sin translucidez y movimiento
  reducido~~ **hecho**: cubierto por `e2e/ui/smoke.spec.ts` y `e2e/ui/a11y.spec.ts` en los dos temas.
- ~~Probar elevación UAC y empaquetado x64~~ **hecho** para desarrollo (`pnpm app:dev` eleva);
  queda el empaquetado real, véase T115 más abajo.
- ~~Verificar la instalación de WebView2 en un Windows Server 2019 limpio y sin salida a
  Internet~~ **resuelto con documentación oficial**, no medido en máquina real: véase
  `open-questions.md` §M.
- **Comprobar la entrega de notificaciones toast desde un proceso elevado** con AUMID registrado
  sigue `ABIERTO` — solo se puede medir con un instalador real empaquetado e instalado
  (`open-questions.md` I.2). El plan B ya está decidido (ventana propia con `Toast`) pero no tiene
  sentido construirlo hasta que la medición falle.
- ~~Medir la codificación de la salida de `chkdsk`~~ **hecho**: es CP1252, no CP850; la detección
  está implementada y probada con volcados reales como fixtures — véase `open-questions.md` §Q.
- ~~Validar `accessibleAccent()` contra los acentos de Windows~~ **hecho**: barrido completo de
  262.144 colores del espacio sRGB, 0 % de fallos AA tras el tratamiento — véase `open-questions.md`
  §O, reverificado en esta sesión (T112).
- ~~Probar bloqueo de instancia única y ACL de la carpeta de `ProgramData`~~ **hecho**: la instancia
  única va con el plugin oficial (ADR-025) y la ACL de `ProgramData` se corrige en el instalador
  (ADR-026) — véase `open-questions.md` §R.
- **Validar `smartctl --scan-open --json` en RAID por hardware y USB** sigue `ABIERTO`: esta sesión
  solo tuvo acceso a NVMe real. Véase `open-questions.md` I.5.
- ~~Interpretar correctamente los bits del código de salida de smartctl~~ **hecho**: parser con
  100 % de cobertura de líneas en `collectors::smartctl_parser` (T108).
- ~~Contrastar en Windows Server la lista de eventos de `alert-rules.md` §3~~ **hecho** contra
  manifiestos y 180 días de registro real — véase `open-questions.md` §P. Queda pendiente el
  contraste visual en servidor, que exige una máquina Server real.
- ~~Probar lectura de eventos y marcadores (bookmark de canal, no RecordId)~~ **hecho**:
  `collectors::event_log` implementado y probado contra fixtures reales.
- ~~Probar contadores de rendimiento y mapeo disco-volumen~~ **hecho**:
  `collectors::perf_counters` y `collectors::windows_storage` implementados.
- ~~Validar SQLite WAL en `ProgramData`~~ **hecho**: `persistence::migrations` y `persistence::db`,
  con pruebas de migración desde una versión publicada anterior.
- ~~Prototipar systray y cierre hacia bandeja~~ **hecho**: `platform::bandeja`, pausa y reanudación
  de la monitorización (Historia 7).
- ~~Documentar cumplimiento de redistribución de smartmontools~~ **hecho**: versión y licencia
  verificadas, binario y fuente en el repositorio — véase `open-questions.md` §N.
- ~~Medir la interfaz con veinte discos y cinco mil eventos~~ **hecho**: ninguna tarea de 50 ms o
  más (`e2e/ui/rendimiento.spec.ts`, `open-questions.md` I.7/J.26). **La escala tipográfica al 125 %
  y 150 %** está medida (`open-questions.md` §L) pero la verificación pantalla por pantalla al
  125 %/150 %/200 % con la ventana mínima 1024 × 560 sigue pendiente (T111, exige revisión visual
  manual).

Riesgos abiertos y su criterio de cierre: [`open-questions.md`](docs/open-questions.md) §I. En síntesis,
lo que de verdad sigue abierto de toda la Fase 0: **I.2** (toast elevado), **I.5** (RAID/USB
reales), el contraste en Windows Server, **T111** (escalado visual) y **T115** (guion completo sobre
un instalador real) — los cinco exigen o hardware que esta sesión no tiene, o un instalador
empaquetado, o revisión visual humana.

Salida: informe de viabilidad y fixtures anonimizados.

Las versiones 0.1 a 0.9 siguientes están **implementadas** (Historias 1-8 de
`specs/001-monitor-discos-windows/tasks.md`); se conservan como agrupación temática del alcance, no
como lista pendiente.

### Versión 0.1 — Inventario y recopilación

- Inventario físico y lógico.
- Selección y alias de discos.
- Colector SMART normalizado y vista JSON.
- Contadores de Windows y capacidad.
- Persistencia SQLite y migraciones.
- Planificador y estados de fuente.
- Altas y bajas de discos en caliente (US-014).
- Tests unitarios de parsers.

### Versión 0.2 — Panel e historial

- Panel general, con comportamiento definido para muchos discos.
- Detalle de disco y volumen.
- Gráficas e intervalos.
- Registro de eventos con deduplicación.
- Español e inglés.
- Tema claro, oscuro y de sistema.
- Asistente inicial.
- Revisión de diseño previa para Informes, Ajustes y asistente inicial, todavía no cubiertos por los bocetos aprobados.

### Versión 0.3 — Alertas y systray

- Motor de reglas según [`alert-rules.md`](docs/alert-rules.md), con histéresis y deduplicación.
- Agrupación y ciclo de vida de alertas, incluido el ciclo de recaída.
- Notificaciones de Windows.
- Estados y menú de systray.
- Pausa y reanudación.
- Preferencia de cierre recordable.
- Diseño y revisión de los estados de systray antes de cerrar su implementación.

### Versión 0.4 — Pruebas e informes

- Benchmark de archivo temporal.
- CHKDSK `/scan`.
- Autotest SMART corto.
- CSV, JSON y HTML.
- ZIP diagnóstico anonimizado.

### Versión 0.9 — Endurecimiento

- Retención, agregación y copias pre-migración (US-071).
- Recuperación tras cierre inesperado (US-074).
- Pantalla de Ajustes completa (US-070 a US-073).
- Límites, timeouts y cancelación.
- Pruebas en Windows cliente y servidor.
- Accesibilidad y revisión de traducciones.
- Logs rotatorios.
- Documentación operativa y de privacidad.
- Auditoría de dependencias y licencias.

### Versión 1.0

- ~~Instalador para todos los usuarios~~ **hecho**: NSIS, WebView2 sin conexión (ADR-020).
- ~~Desinstalación que conserva datos~~ **hecho**: véase «Instalación y desinstalación» en
  [`README.md`](docs/../README.md).
- Aviso documentado de SmartScreen por falta de firma — pendiente de una compilación firmada real.
- ~~Licencia MIT, terceros y atribuciones~~ **hecho**: `THIRD_PARTY_NOTICES.md`.
- ~~Nombre y versión dinámicos~~ **hecho**: `get_app_info` usa `app.package_info()` (T104).
- Release manual en GitHub — pendiente de que el usuario decida publicar.
- **Validación completa contra la definición de terminado de [`ui-design.md`](docs/ui-design.md) §8**
  sigue abierta: exige recorrer cada pantalla al 125 %/150 %/200 % y en la ventana mínima
  1024 × 560 (T111) y ejecutar el guion completo de `quickstart.md` sobre una compilación
  empaquetada, no en desarrollo (T115). Ambas requieren interacción del usuario (revisión visual y
  una instalación elevada real) y quedan deliberadamente para cuando esté disponible.

### Después de 1.0 (P2)

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

### Definición de terminado

Una historia se considera terminada cuando:

- cumple todos sus criterios de aceptación;
- tiene pruebas automatizadas proporcionales al riesgo;
- los errores y estados vacíos están diseñados;
- el texto existe en español e inglés;
- respeta accesibilidad básica de teclado y contraste;
- no introduce permisos Tauri genéricos innecesarios;
- actualiza documentación y avisos de terceros cuando corresponde;
- ha sido verificada en una compilación empaquetada, no solo en desarrollo;
- no ha dejado ninguna decisión implícita en el código: lo que hubo que decidir está en
  `open-questions.md` o en el documento normativo que corresponda.


---

# 5. Arquitectura

Fichero de origen: `docs/architecture.md`

### 1. Resumen

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

### 2. Pila tecnológica

- Tauri 2 como shell de escritorio.
- Rust para acceso privilegiado, procesos, recopilación, reglas y persistencia.
- SvelteKit 2 con `adapter-static` y SSR desactivado, sobre Svelte 5 con runes + TypeScript (ADR-014).
- Tailwind CSS mapeado exclusivamente sobre los tokens del sistema de diseño.
- SQLite, accedido desde Rust.
- `smartctl` como binario auxiliar independiente.
- APIs de Windows, WMI/CIM, Performance Counters y Windows Event Log según disponibilidad.
- Formatos de intercambio internos versionados y serializados con Serde.

No se invocarán comandos privilegiados directamente desde JavaScript. El frontend solo podrá llamar a comandos Tauri enumerados y con argumentos validados.

La comunicación es **bidireccional y por empuje**: la interfaz invoca comandos para pedir estado o
ejecutar acciones, y el backend emite eventos tipados cuando hay datos nuevos. La interfaz no
sondea con temporizadores (ADR-015). El contrato completo —comandos, eventos, DTO y forma de los
errores— está en [`ui-contract.md`](docs/ui-contract.md), y los tipos se generan desde Rust para que
ambos lados no puedan divergir en silencio.

### 2.1. Sistema de diseño e interfaz

El sistema de diseño forma parte de la arquitectura del producto, no es una referencia opcional. Vive dentro del árbol de la aplicación, en una sola copia:

- [`ui-design.md`](docs/ui-design.md): reglas vinculantes de implementación y definición de terminado. Su §0 es el mapa de rutas.
- `src/design-system/tokens.css`: fuente única de verdad visual, importada una sola vez al arrancar.
- `src/design-system/tokens.json`: representación de los mismos tokens para herramientas.
- `src/design-system/fonts/`: tipografía empotrada; la aplicación no descarga tipografías.
- `tailwind.config.cjs`: mapeo permitido de tokens a utilidades.
- `src/lib/components/`: catálogo cerrado de componentes Svelte, importado del barrel `$lib/components`.
- `src/lib/design/`: tipos de presentación, formato, salud, tema y acento de Windows.
- `src/lib/i18n/`: diccionarios y selección de idioma.
- `design/SmartDisk Monitor v2.dc.html`: referencia visual aprobada para las cuatro pantallas diseñadas.

**No puede existir una segunda copia de estos recursos.** El paquete original del diseñador traía la suya y las dos divergieron en silencio hasta que el consolidado publicó tokens caducos; la copia se eliminó y `pnpm verify:tokens` impide que reaparezca (ADR-029).

La UI se monta con `AppShell`, `Sidebar` y `Toolbar`; ninguna pantalla crea su propio chrome. Los componentes nuevos solo se admiten cuando el patrón aparece en al menos tres pantallas y no puede componerse con el catálogo existente.

La comunicación UI-backend usa DTO tipados coherentes con `src/lib/design/types.ts`. Los valores opcionales permanecen como `null`, las series conservan huecos explícitos y toda métrica lleva procedencia y antigüedad cuando estén disponibles.

### 3. Componentes

#### Inventario

- Descubre discos, interfaces, controladoras, particiones y volúmenes.
- Reconcilia distintas identidades de Windows y smartctl.
- Usa número de serie como identidad preferente y una huella estable como alternativa.
- Detecta altas, retiradas, sustituciones y cambios de firmware.
- Registra capacidades no disponibles y su causa conocida.

#### Colector SMART

- Ejecuta `smartctl --scan-open --json` para descubrir dispositivos accesibles y, cuando un
  dispositivo conocido no aparece, reintenta con tipos explícitos (`-d sat`, `-d nvme`,
  `-d sntjmicron`, `-d csmi`) antes de declararlo no compatible: muchos puentes USB y controladoras
  RAID solo responden así.
- Lanza el proceso con `CREATE_NO_WINDOW` (`0x08000000`) para que no parpadee una ventana de consola.
- Pasa `drivedb.h` de la carpeta de la aplicación: sin ella, los atributos específicos de cada
  fabricante se presentan como desconocidos.
- **Distingue la falta de privilegios de la incompatibilidad.** Sin elevación el escaneo funciona,
  pero la lectura de un dispositivo falla con `exit_status: 1` y el mensaje
  `Unable to detect device type`, que parece describir un disco incompatible y no lo es
  (comprobado sobre el binario redistribuido). Confundir ambos casos marcaría un equipo entero
  como no compatible.
- Ejecuta consultas JSON individuales con tiempo máximo y cancelación.
- Conserva los datos normalizados y, selectivamente, la captura bruta.
- Interpreta el código de salida por bits; no depende solo del código cero/no cero.
- Limita ejecutable y argumentos mediante configuración cerrada de Tauri/Rust.

#### Colector Windows

- Obtiene salud complementaria y topología mediante las APIs disponibles.
- Lee capacidad y estado del sistema de archivos por volumen.
- Lee actividad, rendimiento y latencias de los contadores `PhysicalDisk`: `% Idle Time` (del que se
  deriva la actividad, acotada a 0–100), `Disk Read Bytes/sec`, `Disk Write Bytes/sec`,
  `Avg. Disk sec/Read` y `Avg. Disk sec/Write`. No se usa `% Disk Time`, que supera el 100 % con
  varias operaciones simultáneas y no es un porcentaje real.
- La instancia del contador (`"0 C: D:"`) se asocia al dispositivo por su número de disco físico,
  no por la letra de unidad, que puede cambiar.
- Mantiene cada fuente separada para poder indicar procedencia y confianza.

#### Registro de eventos

- Consulta proveedores y eventos de almacenamiento configurados.
- Persiste un marcador por canal para evitar duplicados.
- Normaliza proveedor, identificador, nivel, fecha, dispositivo inferido y mensaje.
- La asociación a un disco puede ser exacta, inferida o desconocida; nunca se presenta una inferencia como certeza.

#### Planificador

- Ejecuta trabajos periódicos independientes.
- Evita ejecuciones concurrentes del mismo trabajo.
- Aplica tiempo máximo, cancelación y backoff ante fallos.
- Reduce trabajos no críticos en batería.
- Suspende recopilaciones incompatibles durante benchmarks o autotests.

#### Motor de alertas

- Evalúa valores absolutos, incrementos, persistencia y frecuencia.
- Deduplica por una clave estable de regla, disco/volumen y contexto.
- Conserva ocurrencias individuales dentro de un grupo.
- Implementa transición activa, reconocida, resuelta y archivada.
- Aplica cooldown a las notificaciones, especialmente para capacidad.

#### Pruebas

- Benchmark basado en archivo temporal seguro y verificable.
- CHKDSK `/scan` con captura de salida. La salida se guarda **en bytes**, y la decodificación para
  presentarla se hace con la detección de `open-questions.md` §Q: las herramientas de Windows no
  coinciden entre sí en la página de códigos, así que codificar una constante produce texto corrupto
  en la mitad de los casos. La codificación deducida se registra junto a la ejecución.
- Autotest SMART corto con detección de soporte, seguimiento y cancelación.
- Exclusión mutua por disco entre pruebas y determinadas recopilaciones.

#### Informes

- Consulta intervalos sin bloquear escrituras de recopilación.
- Exporta datos normalizados y metadatos de procedencia.
- Anonimiza identificadores mediante sustitución estable dentro de cada paquete.

### 4. Concurrencia y ciclo de vida

- La UI y los colectores no comparten operaciones bloqueantes.
- Los trabajos se ejecutan en tareas asíncronas o pools apropiados.
- SQLite usa WAL, transacciones breves y migraciones versionadas.
- Hay **una sola conexión a la base de datos**, protegida por un mutex de Rust: todo acceso queda
  serializado por ese candado, no por SQLite. Por eso **ningún colector retiene el candado mientras
  hace E/S externa** (subprocesos `smartctl`, muestreo PDH con su pausa entre lecturas, FFI del
  registro de eventos): el ciclo de recopilación se estructura en tres fases —candado breve para
  planificar, E/S sin candado, candado único para persistir— para que una consulta de la interfaz
  no espere nunca detrás de un `smartctl.exe` de decenas de segundos (ADR-042).
- Al cerrar hacia la bandeja, todos los trabajos continúan.
- Al salir, se solicita cancelación, se termina cualquier auxiliar propio y se vacían las escrituras pendientes.
- Ante cierre inesperado, el siguiente inicio reconcilia pruebas incompletas y archivos temporales
  huérfanos: toda prueba que quedó en `running` o `cancelling` pasa a `interrupted` y los archivos
  huérfanos se listan con su ruta para que el usuario decida (US-074).
- Solo puede haber una instancia en ejecución; abrir una segunda restaura la ventana de la primera.
  Lo resuelve `tauri-plugin-single-instance`, registrado **el primero** de los plugins porque se
  ejecutan en orden de registro (ADR-025). Su devolución de llamada corre en el proceso que ya
  estaba vivo y llama a `platform::ventana::restaurar_ventana_principal()`, el mismo camino que usa
  el arranque normal: desminimizar, mostrar y enfocar, en ese orden.

### 5. Rutas previstas

```text
C:\Program Files\SmartDisk Monitor\
  SmartDiskMonitor.exe
  bin\
    smartctl.exe                 smartmontools 7.5, x64
    drivedb.h                    base de datos de unidades; sin ella los atributos
                                 específicos de cada fabricante quedan sin interpretar
  licenses\
    smartmontools\
      COPYING.txt                GPL v2
      AUTHORS.txt
      smartmontools-7.5.tar.gz   fuente correspondiente (GPLv2 §3a)
    instrument-sans\
      OFL.txt

C:\ProgramData\SmartDisk Monitor\
  smartdisk.db
  logs\
  raw\
  diagnostics\
  backups\
```

La ruta de datos se obtiene de Windows y no se codifica como literal en la lógica de negocio.

### 6. Seguridad

- Manifiesto Windows `requireAdministrator`.
- Política de capacidades Tauri con mínimo privilegio dentro del proceso elevado.
- Sin shell genérica expuesta al frontend.
- Lista permitida de operaciones y argumentos de smartctl.
- Validación canónica de rutas antes de crear o eliminar archivos de prueba.
- Límites de tamaño y reserva de espacio antes de escribir.
- SQL parametrizado y migraciones verificadas.
- Contenido procedente de eventos o dispositivos renderizado como texto, nunca como HTML sin sanear.
- Sin endpoints de red ni telemetría.

### 7. Tolerancia a fallos

- Cada fuente devuelve estado `ok`, `partial`, `unsupported`, `timeout` o `error`.
- Un fallo parcial no degrada automáticamente la salud del disco.
- Se conserva la última lectura válida y se muestra su antigüedad.
- Los procesos auxiliares tienen timeout y salida limitada.
- Las capturas brutas con errores se conservan con rotación para diagnóstico.
- Los cambios del reloj no duplican eventos gracias a identificadores/marcadores del canal.

### 8. Compatibilidad extensible

Las fuentes implementarán interfaces internas para permitir en el futuro:

- ejecutores de fabricantes RAID;
- Linux y macOS;
- servicio de Windows;
- almacenamiento central;
- nuevas fuentes de alertas.

Estas extensiones no forman parte de la versión 1.0.

### 9. Pruebas técnicas

- Unitarias para parsers, normalización, reglas y retención.
- Datos fixture anonimizados de ATA, NVMe, USB, RAID y VM.
- Integración con un `smartctl` falso para salidas, timeouts y errores.
- Integración SQLite y migraciones desde cada versión soportada.
- Pruebas de rutas seguras y limpieza de benchmark.
- Pruebas manuales en hardware real sin exigir una marca determinada.
- Pruebas de interfaz para flujos críticos y accesibilidad.
- Pruebas visuales en temas claro y oscuro, con acento de Windows y color de respaldo.
- Pruebas de ventana mínima, scroll, ausencia de recortes y fallback sin `backdrop-filter`.
- Comprobación automatizada o revisión que impida colores, radios, sombras y tamaños tipográficos literales en componentes.
- Verificación de foco, teclado, contraste AA, etiquetas accesibles y `prefers-reduced-motion`.


---

# 6. Modelo de datos

Fichero de origen: `docs/data-model.md`

### 1. Principios

- Separar identidad física, topología mutable y observaciones temporales.
- Conservar procedencia y calidad de cada métrica.
- Guardar contadores absolutos para calcular incrementos de forma fiable.
- No asumir que una letra de unidad identifica un disco.
- No marcar como fallo la ausencia de una métrica.

### 2. Entidades principales

#### `devices`

- `id`: UUID interno.
- `fingerprint`: identidad calculada estable. Se compone como
  `sha256(model | capacity_bytes | bus_type | wwn_o_pnp_device_id)`. **El firmware queda fuera a
  propósito**: si formara parte de la huella, actualizarlo partiría el historial del disco en dos
  entidades, y la arquitectura pide justo lo contrario, detectar el cambio sobre la misma entidad.
- `identity_confidence`: `serial` o `fingerprint`. La interfaz marca como identidad inferida los
  discos sin número de serie.
- `serial_number`: nullable.
- `model`, `manufacturer`, `firmware`.
- `device_type`: NVMe, SATA SSD, HDD, USB, virtual, RAID logical, unknown.
- `bus_type` y `smartctl_path`.
- `capacity_bytes`.
- `alias`.
- `monitoring_enabled`.
- `first_seen_at`, `last_seen_at`, `removed_at`.
- `capabilities_json`.

#### `volumes`

- `id`, `volume_guid`, `label`, `filesystem`.
- `drive_letters_json`.
- `capacity_bytes`, `free_bytes`.
- `device_mapping_confidence`.
- `first_seen_at`, `last_seen_at`.

#### `device_volume_links`

- Relación entre discos físicos y volúmenes.
- Permite múltiples discos por volumen y múltiples volúmenes por disco.
- Incluye procedencia y confianza de la asociación.

#### `metric_samples`

- `device_id` o `volume_id`.
- Exactamente uno de `device_id` / `volume_id` es no nulo, garantizado por una restricción `CHECK`.
- `metric_key`, `value_real`, `value_integer`, `unit`.
- `sampled_at_utc`.
- `source`: smartctl, Windows Storage, performance counter, filesystem.
- `quality`: exact, inferred, vendor_specific, stale.
- `resolution`: raw, five_minutes, hourly.

#### `smart_snapshots`

- `device_id`, `captured_at_utc`.
- Campos normalizados de salud.
- `smartctl_version`, `exit_status` y estado de consulta.
- Ruta o contenido comprimido de JSON bruto cuando proceda.

#### `system_events`

- Identidad estable del canal/registro.
- `occurred_at_utc`, `provider`, `event_id`, `level`.
- `message`, `raw_xml` opcional.
- Disco o volumen asociado y confianza de asociación.
- Hash de deduplicación.

#### `alert_occurrences`

- `alert_group_id`, `cycle`, `occurred_at_utc`, `value_real`, `context_json`.
- `triggering_event_id`: el `system_events.id` del evento de Windows que provocó esta ocurrencia.
  Lo rellenan las reglas `events.*` (spec `003-puente-eventos-alertas`); `null` para SMART y
  capacidad. El detalle de una alerta lo usa para enlazar al suceso en la pantalla de eventos.

#### `alert_groups`

- `id`, `deduplication_key`, `rule_key`.
- Objeto afectado.
- Severidad y estado (`active`, `acknowledged`, `resolved`, `archived`).
- `muted_until`: fecha UTC, `null` o el valor especial de silencio indefinido. **No es un estado**:
  es ortogonal y convive con cualquiera de ellos.
- `cycle`: número de episodio. Un grupo resuelto que recae lo incrementa en vez de crear un grupo
  nuevo, para no perder el contador histórico.
- Primera y última ocurrencia.
- Contador.
- Fechas de reconocimiento, resolución y archivo.
- Último valor y contexto.

#### `alert_occurrences`

- `alert_group_id`, `cycle`, fecha, valor, evento y contexto de la ocurrencia.

#### `test_runs`

- Tipo: benchmark, chkdsk_scan o smart_short.
- Disco/volumen objetivo.
- Estado: pending, running, cancelling, completed, failed, cancelled, interrupted.
- Inicio, fin, progreso y resultado.
- Parámetros y resumen de métricas.
- Ruta temporal solo mientras sea necesaria.

#### `settings`

- Claves tipadas y versionadas.
- Preferencias globales, de disco y de volumen.
- Idioma, tema, frecuencias, retención, cierre y umbrales.
- `storage.free_space_warn_bytes` / `storage.free_space_halt_bytes`: umbrales de espacio libre del
  volumen donde reside el historial. Al cruzar el de aviso se notifica; al cruzar el de parada se
  detiene la escritura de historial sin afectar a la monitorización ni a las alertas en vivo. Valor
  por defecto 1 GB / 256 MB, no medido (`open-questions.md` J.13).
- `logging.verbose`: booleano, modo detallado de registro de actividad (US-071).
- `notifications.enabled`: booleano, **fábrica `true`** (ADR-037). `false` oculta el toast nativo sin
  necesidad de pausar la recopilación. La alerta sigue existiendo y contando para el color de salud.
- `lifecycle.start_with_system`: booleano, **fábrica `false`** (ADR-038). Al activarlo, el backend
  registra una tarea programada elevada (`schtasks /SC ONLOGON /RL HIGHEST`); al desactivarlo o al
  hacer `reset_settings("all")`, la borra. Fuente de verdad = esta clave, no el estado real de la
  tarea.
- **`window.width` / `window.height` / `window.x` / `window.y` / `window.maximized`** (ADR-040):
  geometría de la ventana principal de la última sesión, en **píxeles lógicos**. Enteros y un
  booleano. Las escribe **solo el backend** (al cerrar y al salir), nunca el frontend ni
  `set_setting`. Ausentes ⇒ se usa `tauri.conf.json` (primer arranque: 1695 × 988 centrada). Si la
  posición guardada queda fuera de todo monitor actual, se ignora y la ventana abre centrada. Con
  `window.maximized` activo no se tocan tamaño ni posición: se conservan los previos a maximizar.
  `reset_settings` (ámbito «resto» o «all») las borra.
- **`alerts.profile`** (`cautious` | `balanced` | `quiet` | `custom`) y los umbrales que un perfil
  escribe (ADR-036, `cambios/08b-perfiles-de-alerta.md`). Fábrica: `balanced`. Umbrales nuevos frente
  a v2, con su rango de edición y su valor de fábrica (perfil Equilibrado):

  | Clave | Rango | Fábrica |
  |---|---|---|
  | `alerts.temp_configured_warn_c` | 40–95 | **60** (baja de 70) |
  | `alerts.temp_configured_crit_c` | warn–100 | **70** (baja de 80) |
  | `alerts.wear_warn_percent` | 50–99 | 80 |
  | `alerts.wear_crit_percent` | warn–100 | 90 |
  | `alerts.media_errors_warn_per24h` | 1–1000 | 1 — es el incremento del contador que basta para avisar, **no** una ventana de 24 h (clarify Q1) |
  | `alerts.media_errors_crit_per24h` | warn–1000 | 5 |
  | `alerts.driver_retry_warn_per24h` | 1–1000 | 5 — **aún sin consumidor** (necesita el colector de eventos, Historia 4) |
  | `alerts.driver_retry_crit_per24h` | warn–1000 | 12 |

  Perfiles: Prudente 55/65 · 70/85 · … · Equilibrado (= fábrica) · Solo lo grave 70/80 · 90/95 · …
  Tabla completa en `specs/002-rediseno-v3/data-model.md` y en `cambios/08b`.
- **`settings.onboarding.completed_at`**: fecha ISO-8601 UTC o nula. Nula ⇒ el guardián de
  `+layout.ts` redirige al asistente inicial al arrancar, **salvo** que ya haya configuración previa
  observable (tema ≠ `system`, idioma forzado, perfil de alerta ≠ `balanced`, algún alias o alguna
  exclusión), en cuyo caso la graba y sigue sin mostrarlo (FR-043, sin migración). No la restaura
  `reset_settings`.
- **`volume_free_bytes`** (`metric_samples`, `MetricTarget::Volume`): muestra periódica del espacio
  libre de cada volumen monitorizado, persistida en el ciclo de descubrimiento (ADR-036). Antes la
  capacidad solo vivía como instantánea en `volumes.free_bytes`; ahora también como serie, para que
  `capacity.low`/`capacity.critical` tengan histéresis. La retención la compacta igual que el resto.
- `VolumeSummary` gana `is_system_volume` (booleano): `true` para el volumen donde vive Windows. Lo
  calcula el backend al leer (`GetSystemWindowsDirectoryW`), **sin columna nueva ni migración**.

#### `event_cursors`

- Canal/proveedor y **bookmark** del registro de eventos, no un `RecordId` suelto: al limpiar un
  canal los identificadores se reinician, y un cursor numérico se quedaría por delante de los
  eventos nuevos y dejaría de importarlos sin dar ningún error.
- Evita duplicar eventos entre sesiones. La identidad de un evento es `(canal, RecordId)`, no su
  fecha, de modo que un cambio del reloj del sistema tampoco produce duplicados.

#### `schema_migrations`

- Versión, fecha y checksum de cada migración aplicada.

#### `metric_aggregates`

- `device_id` o `volume_id`, misma restricción de exactamente uno que `metric_samples`.
- `metric_key`, `bucket_start_utc`, `bucket_end_utc`, `resolution` (`five_minutes` o `hourly`).
- `value_min`, `value_max`, `value_avg`, `value_first`, `value_last`, `sample_count`, `unit`.
- El incremento de un contador acumulativo dentro del bucket es `value_last - value_first`; no
  lleva columna propia (`open-questions.md` J.14).

### 3. Métricas normalizadas iniciales

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
- `activity_percent` — porcentaje de tiempo con al menos una operación en curso, derivado de
  `PhysicalDisk\% Idle Time` y acotado a 0–100. **No** se usa `% Disk Time` directamente, que en
  discos con varias operaciones simultáneas supera el 100 % y no es un porcentaje real.
- `volume_free_bytes`
- `volume_free_percent`
- `smart_query_ok` — 1.0 si `smartctl` pudo leer el disco ese ciclo, 0.0 si la consulta falló o
  devolvió algo irreconocible. **No** es una medida del disco: es el resultado del intento de
  consulta, y es la serie sobre la que se evalúa la regla `smart.unreadable` (`alert-rules.md`).
- `vendor_temp_limit_celsius` — límite operativo de temperatura que declara el fabricante
  (`temperature.op_limit_max` de `smartctl`, tabla SCT). Solo lo traen algunos discos SATA; ausente
  en la mayoría de NVMe. Es el umbral de `temp.above_vendor_limit` y de la línea de referencia de la
  gráfica de temperatura. **No** hay equivalente para el crítico del fabricante: `smartctl` no lo
  expone de forma fiable en el JSON (`open-questions.md` J.16).

Los campos no disponibles se omiten; no se almacenan como cero.

### 4. Retención

- Un trabajo diario compacta muestras antiguas dentro de una transacción.
- La agregación conserva mínimo, máximo, promedio, primera y última lectura, además de incrementos
  de contadores, en `metric_aggregates` (§2).
- **Tres periodos, uno por resolución** (US-071, `open-questions.md` J.14): pasado
  `retention.raw_days` (7 por defecto) las muestras `raw` se compactan a `five_minutes`; pasado
  `retention.five_minutes_days` (90) se compactan a `hourly`; pasado `retention.hourly_days` (730)
  se purgan. Valores de partida, no medidos.
- Antes de una migración se crea una copia consistente de SQLite.
- Se conservan las tres copias de migración más recientes.
- Alertas, ocurrencias críticas, eventos vinculados y ejecuciones de pruebas no se borran automáticamente.

### 5. Tiempo y unidades

- Persistencia en UTC.
- Presentación en hora local del sistema.
- Capacidades almacenadas en bytes, caudales en bytes por segundo y temperaturas en Celsius. La
  conversión a unidades legibles ocurre solo en la capa de presentación, en base 1024 con las
  etiquetas KB/MB/GB que usa el Explorador de Windows.
- La interfaz puede presentar unidades legibles sin alterar el dato original.


---

# 7. Reglas de alerta

Fichero de origen: `docs/alert-rules.md`

Especificación normativa del motor de alertas. Sustituye a la prosa de
`product-specification.md` §5, que queda como resumen. Si esta tabla y aquel texto discrepan, manda
esta tabla.

Referencias: `docs/data-model.md` (`alert_groups`, `alert_occurrences`), `docs/open-questions.md`
§B, `src/lib/design/health.ts`.

---

### 1. Vocabulario

**Grupo** (`alert_groups`): una condición sobre un objeto concreto. Es lo que el usuario ve y
gestiona. **Ocurrencia** (`alert_occurrences`): cada vez que la condición se ha vuelto a cumplir.
El grupo guarda el contador y las fechas extremas; las ocurrencias guardan la cronología completa.

**Clave de deduplicación.** Determina si una evaluación cae en un grupo existente o crea uno nuevo:

```
deduplication_key = rule_key | target_type:target_id | context
```

`context` es el discriminante propio de cada regla (el identificador del evento de Windows, el
sensor de temperatura, el nombre del contador). Dos evaluaciones con la misma clave **siempre** caen
en el mismo grupo, aunque hayan pasado meses.

**Ciclo.** Un grupo resuelto que vuelve a cumplirse no crea un grupo nuevo: se reactiva e incrementa
`cycle`. El contador histórico se conserva; la cronología separa los episodios visualmente.

**Estados y transiciones.**

| Desde | Evento | Hasta |
|---|---|---|
| — | primera evaluación positiva | `active` |
| `active` | el usuario reconoce | `acknowledged` |
| `active` / `acknowledged` | la condición deja de cumplirse con margen durante N ciclos | `resolved` |
| `acknowledged` | **sube** la severidad | `active` (y vuelve a notificar) |
| `acknowledged` | baja la severidad | `acknowledged` |
| `resolved` | vuelve a cumplirse | `active`, `cycle + 1` |
| cualquiera | el usuario archiva | `archived` |
| `archived` | vuelve a cumplirse | `active`, `cycle + 1` |

El **silencio** (`muted_until`) no es un estado: es ortogonal. Suprime la notificación, nunca el
color ni la presencia en la lista. Valores: `null`, una fecha UTC, o `"infinite"`.

**Qué cuenta para el color.** Los estados `active` y `acknowledged`. Ni `resolved` ni `archived`.
El silencio nunca afecta al color. Una sola implementación: `deviceState()`.

---

### 2. Tabla de reglas

`N ciclos` se refiere siempre a ciclos consecutivos del recopilador de esa fuente, no a ocurrencias
dentro de una ventana. Con las frecuencias por defecto, 3 ciclos son 90 s en el recopilador rápido
y 15 min en SMART.

Las reglas basadas en el registro de eventos llevan además una **ventana de correlación** de 60 s:
un mismo hecho físico produce varios eventos distintos a la vez, y sin ella un solo disco
desconectado generaría cuatro alertas. Véase §3.5.

| `rule_key` | Fuente | Activación | Severidad | Resolución (histéresis) | Cooldown de notificación | Contexto de dedup |
|---|---|---|---|---|---|---|
| `smart.health.failed` | smartctl | `health_passed = false` | crítico inmediato | `health_passed = true` durante 3 ciclos | ninguno: siempre notifica | — |
| `nvme.critical_warning` | smartctl | `critical_warning ≠ 0` | crítico inmediato | `= 0` durante 3 ciclos | ninguno | bit activo |
| `smart.media_errors` | smartctl | el **incremento** de `media_errors_total` entre dos lecturas alcanza `settings.alerts.media_errors_warn_per24h` (aviso) / `_crit_per24h` (crítico). El sufijo `Per24h` es histórico: **no** es una ventana de 24 h (ADR-036) | advertencia; **crítico** en el umbral crítico | no aumenta durante 24 h | 1 h | — |
| `smart.error_log` | smartctl | `error_log_entries_total` aumenta | advertencia; **crítico** si aumenta en 3 ciclos seguidos | no aumenta durante 24 h | 1 h | — |
| `smart.spare_below_threshold` | smartctl | `available_spare_percent < available_spare_threshold_percent` | crítico | por encima del umbral + 2 puntos durante 3 ciclos | 6 h | — |
| `smart.wear_high` | smartctl | `percentage_used ≥ settings.alerts.wear_warn_percent` (fábrica 80) | advertencia; **crítico** en `≥ wear_crit_percent` (fábrica 90) | no se resuelve sola: el desgaste no baja. Se archiva a mano | 7 días | — |
| `temp.above_vendor_limit` | smartctl | `temperature_celsius > vendorTempLimitC` durante 3 ciclos | advertencia | ≤ límite − 3 °C durante 3 ciclos | 30 min | id. de sensor |
| `temp.above_vendor_critical` | smartctl | `temperature_celsius ≥ vendorTempCriticalC` | crítico inmediato | ≤ crítico − 5 °C durante 3 ciclos | 15 min | id. de sensor |
| `temp.above_configured_warn` | smartctl | sin límite del fabricante: `> settings.alerts.temp_configured_warn_c` (fábrica 60 °C — ADR-036) durante 3 ciclos | advertencia | ≤ (umbral − 3 °C) durante 3 ciclos | 30 min | id. de sensor |
| `temp.above_configured_crit` | smartctl | sin límite del fabricante: `≥ settings.alerts.temp_configured_crit_c` (fábrica 70 °C) | crítico inmediato | ≤ (umbral − 5 °C) durante 3 ciclos | 15 min | id. de sensor |
| `capacity.low` | sistema de archivos | `estado_capacidad()` da `warn` sobre la serie `volume_free_bytes` (ADR-036: umbrales de `settings.alerts.capacity_*`) | advertencia | vuelve a `ok` **y** se mantiene 3 ciclos | solo al cambiar de nivel | `volume_guid` |
| `capacity.critical` | sistema de archivos | `estado_capacidad()` da `crit` | crítico | sube a `warn` u `ok` y se mantiene 3 ciclos | solo al cambiar de nivel | `volume_guid` |
| `device.removed_unexpected` | inventario + `disk` 157 | desaparece sin solicitud de expulsión previa | crítico; **advertencia** si `bus_type = USB` | reaparece el mismo `fingerprint` | ninguno | — |
| `events.disk_error` | registro de eventos | `disk` 7, `NvmeDisk` 500, `StorageSpaces-Driver` 202/203/209 | crítico | 24 h sin repetición | 1 h | `provider:event_id` |
| `events.filesystem_error` | registro de eventos | `Ntfs` 55 o 131 | crítico | 24 h sin repetición | 1 h | `provider:event_id` |
| `events.filesystem_repaired` | registro de eventos | `Ntfs` 130: se reparó solo | advertencia | 7 días sin repetición | 24 h | `volume_guid` |
| `events.filesystem_repair_storm` | registro de eventos | `Ntfs` 132: Windows deja de informar de tantas reparaciones | crítico | 7 días sin repetición | 6 h | `volume_guid` |
| `events.controller_reset` | registro de eventos | `disk` 11, `stornvme`/`storahci` 129 | advertencia; **crítico** con ≥ 3 en 1 h sobre el mismo disco fijo | 24 h sin repetición | 1 h | `provider:event_id` |
| `events.paging_error` | registro de eventos | `disk` 51, **solo en discos no extraíbles** y con ≥ 10 en 1 h | advertencia; nunca crítico | 24 h sin repetición | 6 h | `device_id` |
| `events.io_retry` | registro de eventos | `disk` 153 con ≥ 5 en 1 h | advertencia | 24 h sin repetición | 6 h | `device_id` |
| `events.delayed_write` | registro de eventos | `Ntfs` 50 o `Microsoft-Windows-Ntfs` 140 | advertencia; **crítico** si el volumen no es extraíble | 24 h sin repetición | 1 h | `volume_guid` |
| `events.disk_predictive` | registro de eventos | `disk` 52: el disco puede fallar pronto | advertencia | 7 días sin repetición | 24 h | — |
| `events.storage_space_degraded` | registro de eventos | `StorageSpaces-Driver` 300–311 | crítico | 24 h sin repetición | 1 h | id. del disco virtual |
| `inventory.duplicate_id` | registro de eventos | `disk` 158: dos discos comparten identificadores | advertencia | el aviso deja de repetirse 7 días | una sola vez por par | par de discos |
| `smart.unreadable` | smartctl | consulta fallida (timeout o error) 3 ciclos seguidos, en un disco que **sí** soportaba SMART | advertencia | una lectura correcta | 6 h | — |
| `collector.stalled` | planificador | un recopilador no completa un ciclo en 3 intervalos esperados | advertencia | un ciclo completo | 1 h | nombre del recopilador |

#### Lo que explícitamente NO genera alerta

- Un dispositivo que **declara** no soportar SMART (`unsupported`): es normalidad. Aparece en gris.
- Un contador que falta: `unavailable` no es `0` y no se evalúa.
- Un evento informativo de Windows: se muestra en la cronología, no crea grupo. En particular
  `Microsoft-Windows-Ntfs` 98, que dice que el volumen **está bien** y aparece cientos de veces.
- Los eventos de `Volsnap` (instantáneas VSS) y `volmgr` 161 (volcado de memoria): no hablan de la
  salud del disco.
- Un `disk` 51 aislado, o cualquier evento sobre un medio extraíble que se acaba de desconectar:
  véase §3.3 y §3.5.
- La primera lectura de un contador acumulativo: sin lectura previa no hay incremento que medir.

#### Cooldown de notificación

El cooldown suprime la **notificación**, no la ocurrencia: la cronología del grupo lo registra todo.
Las reglas de capacidad son las más restrictivas por exigencia de spec §5: solo notifican al cruzar
un umbral, al cambiar de nivel, o al recaer tras haberse recuperado. Nunca una notificación por
muestra.

---

### 3. Proveedores y eventos de Windows vigilados

Lista verificada el 2026-09-04 contra los manifiestos de proveedor de un Windows 11 real y contra
180 días de su registro `System` (2.038 eventos de almacenamiento sobre 32.620 totales). Sustituye a
la lista tentativa anterior, que **clasificaba mal varios eventos** (véase §3.4). Se guarda en
`settings` para poder ampliarla sin recompilar.

#### 3.1 Proveedores: dos familias que se consultan distinto

| Familia | Proveedores | Cómo se enumeran |
|---|---|---|
| **Clásicos** (sin manifiesto) | `disk`, `Ntfs`, `volmgr`, `volsnap`, `storahci`, `stornvme`, `partmgr`, `iaStor*` | No declaran eventos: `Get-WinEvent -ListProvider` devuelve cero. Sus mensajes viven en el binario del controlador. Hay que filtrar por proveedor e id, no descubrirlos |
| **Con manifiesto** | `Microsoft-Windows-Ntfs`, `Microsoft-Windows-NvmeDisk`, `Microsoft-Windows-StorageSpaces-Driver`, `Microsoft-Windows-StorPort`, `Microsoft-Windows-ReFS` | Declaran sus eventos con nivel y plantilla, y se pueden enumerar |

**`Microsoft-Windows-Disk` no sirve para alertas.** Declara 22 eventos y todos son informativos: son
trazas de E/S ("Distribuyendo una solicitud de lectura"), no diagnósticos. Los errores de disco
vienen del proveedor clásico `disk`.

#### 3.2 Eventos vigilados

Los marcados **(obs.)** se han observado realmente, con su frecuencia en 180 días de un equipo sano.

| Proveedor | Id | Nivel | Qué significa | Regla | Severidad |
|---|---|---|---|---|---|
| `disk` | 7 | Error | Bloque defectuoso | `events.disk_error` | crítico |
| `disk` | 11 | Error | Error de controladora en el dispositivo **(obs., 52)** | `events.controller_reset` | advertencia; crítico con ≥3 en 1 h en el mismo disco fijo |
| `disk` | 51 | Advertencia | Error durante una operación de paginación **(obs., 839)** | `events.paging_error` | **advertencia, nunca crítico**; véase §3.3 |
| `disk` | 52 | Advertencia | El disco puede fallar pronto | `events.disk_predictive` | advertencia |
| `disk` | 153 | Advertencia | Reintento de E/S en un bloque **(obs., 52)** | `events.io_retry` | advertencia con ≥5 en 1 h |
| `disk` | 157 | Advertencia | **El disco se ha extraído de forma imprevista (obs., 63)** | `device.removed_unexpected` | crítico, o advertencia si el bus es USB |
| `disk` | 158 | Advertencia | Dos discos comparten identificadores **(obs., 63)** | `inventory.duplicate_id` | advertencia, una sola vez por par |
| `Ntfs` | 50 | Advertencia | Fallo de escritura demorada; datos perdidos **(obs., 367)** | `events.delayed_write` | advertencia; crítico si el volumen no es extraíble |
| `Ntfs` | 55 | Error | Daño en la estructura del sistema de archivos | `events.filesystem_error` | crítico |
| `Ntfs` | 130 | Advertencia | La estructura se reparó sola | `events.filesystem_repaired` | advertencia |
| `Ntfs` | 131 | Error | **La estructura no se puede corregir; hay que ejecutar chkdsk** | `events.filesystem_error` | crítico |
| `Ntfs` | 132 | Advertencia | Demasiadas reparaciones seguidas; Windows deja de informar | `events.filesystem_repair_storm` | crítico |
| `Microsoft-Windows-Ntfs` | 140 | Advertencia | No se pudo vaciar el registro de transacción **(obs., 173)** | `events.delayed_write` | advertencia; **crítico si el volumen no es extraíble** (como `Ntfs` 50 — manda §2, spec 003) |
| `Microsoft-Windows-NvmeDisk` | 500 | Error | Comando NVM completado con error | `events.disk_error` | crítico |
| `Microsoft-Windows-NvmeDisk` | 501 | Advertencia | Caché de escritura habilitada en el dispositivo | — | **informativo, no genera alerta** |
| `stornvme` / `storahci` | 129 | Advertencia | Restablecimiento del dispositivo **(obs., 4)** | `events.controller_reset` | advertencia; crítico con ≥3 en 1 h |
| `Microsoft-Windows-StorageSpaces-Driver` | 202, 203, 209 | Error | Disco físico con metadatos inválidos o error de E/S | `events.disk_error` | crítico |
| `Microsoft-Windows-StorageSpaces-Driver` | 300–311 | Error | Disco virtual degradado, sin configuración o desconectado | `events.storage_space_degraded` | crítico |
| `Microsoft-Windows-StorageSpaces-Driver` | 103 | Error | El grupo superó el umbral de capacidad | `capacity.critical` | crítico |

#### 3.3 Lo que NO genera alerta, y por qué importa

| Proveedor | Id | Por qué se ignora |
|---|---|---|
| `Microsoft-Windows-Ntfs` | 98 | Es de nivel **Información** y dice literalmente que el volumen es correcto y no se requiere ninguna acción. Observado 319 veces |
| `Microsoft-Windows-Disk` | 1, 201–221 | Trazas informativas de E/S |
| `Volsnap` | 25, 33, 36 | Gestión de instantáneas VSS: es política de espacio, no salud del disco. Observados 102 en total |
| `volmgr` | 161 | Fallo al crear el archivo de volcado tras un cuelgue. No dice nada del estado del disco |
| `Microsoft-Windows-NvmeDisk` | 501 | La caché de escritura habilitada es la configuración normal de fábrica |

**`disk` 51 merece párrafo propio.** Apareció **839 veces en 180 días en un equipo sano**: es con
diferencia el evento de almacenamiento más frecuente de Windows, y es notoriamente benigno — se
dispara al desconectar un medio extraíble, al despertar un disco, o ante cualquier reintento de
paginación que el sistema resuelve solo. Tratarlo como crítico, que es lo que decía la lista
tentativa anterior, habría producido **839 alertas críticas falsas en un equipo sin ningún
problema**, y con ello habría inutilizado el producto entero. Es advertencia, con umbral de
frecuencia, y solo cuenta si se repite sobre el mismo disco no extraíble.

#### 3.4 Errores de la lista anterior, corregidos

La lista tentativa que traía la especificación tenía fallos que solo se ven con datos reales:

| Antes | Realidad |
|---|---|
| `Ntfs` 98 → "metadatos inconsistentes", **crítico** | Es `Microsoft-Windows-Ntfs` 98, de nivel **Información**, y significa que el volumen está bien. Habrían sido 319 falsos críticos |
| `disk` 51 y 52 → **crítico** | 51 es ruido de fondo masivo (839 casos). Advertencia con umbral de frecuencia |
| `Ntfs` 130 → "marcado para comprobación" | 130 es "se **reparó** la estructura". El que indica daño irreparable es el **131**, que faltaba |
| `volmgr` 46, 49 | No aparecen en el sistema. El que sí aparece, 161, no habla de salud del disco |
| — | Faltaban `disk` **157** (extracción imprevista, que es justo el evento que necesita la regla `device.removed_unexpected`), `disk` 158, `Ntfs` 50, `Microsoft-Windows-Ntfs` 140 y el proveedor `Microsoft-Windows-NvmeDisk` entero |

#### 3.5 Los eventos llegan en ráfagas correlacionadas

El dato más útil de la observación: **un solo hecho físico produce varios tipos de evento a la vez.**
Al desconectar en caliente un disco externo se generaron, del mismo dispositivo y en segundos:

```text
disk 157             El disco 1 se ha extraído de forma imprevista
disk 51              Error durante una operación de paginación
Ntfs 50              Error en la escritura demorada, sobre el MFT del volumen
Ms-Windows-Ntfs 140  No se pudo vaciar el registro de transacción
```

Deduplicando solo por `provider:event_id`, ese único suceso crearía **cuatro grupos de alerta**
distintos. Por eso el motor aplica, además de la deduplicación de §1, una **ventana de correlación**:

- Los eventos del mismo dispositivo dentro de una ventana de **60 segundos** se tratan como un
  suceso.
- Si entre ellos hay un `disk` 157 (extracción imprevista), **ese es la causa** y los demás pasan a
  ser ocurrencias suyas en lugar de grupos propios: son su consecuencia, no cuatro problemas.
- La cronología del grupo conserva todos los eventos con su hora, para poder reconstruir qué pasó.
- **Sin un `disk` 157 en la ventana, no se colapsa nada**: dos errores graves distintos a la vez
  (p. ej. `Ntfs` 55 + `disk` 7) crean dos grupos, porque son dos problemas reales (spec 003, Q3).

**Implementación** (spec `003-puente-eventos-alertas`, `docs/open-questions.md` J.49): la ventana es
tiempo de reloj, no ciclos del recopilador. Al evaluar un evento nuevo, `alerts::eventos` consulta
`system_events` los eventos del mismo disco en los 60 s anteriores (ya persistidos), así una ráfaga
partida entre dos ciclos de 30 s se correlaciona igual. Si el `disk` 157 llega en un ciclo posterior
a un síntoma ya agrupado, ese grupo derivado se **resuelve** (era consecuencia) y el 157 crea su
`device.removed_unexpected`.

#### 3.6 Asociación evento → disco

Puede ser `exact` (el evento nombra el dispositivo y hay correspondencia inequívoca), `inferred` (se
deduce por volumen o por controladora) o `unknown`. Una inferencia **nunca** se presenta como
certeza: `EventRow` la etiqueta.

Ojo con los identificadores que aparecen en los mensajes: conviven formas como `\Device\Harddisk1\DR18`,
`\DR39`, "disco 1", `\Device\HarddiskVolume23` y nombres PDO como `\Device\0003d2a5`. **No son
intercambiables**, y el número que sigue a `DR` no es el número de disco físico. La resolución se
hace contra el inventario, nunca por coincidencia textual.

#### 3.7 Deduplicación entre sesiones

Se persiste un *bookmark* del canal, no un `RecordId` suelto: al limpiar un canal los identificadores
se reinician, y un cursor numérico se quedaría por delante de los eventos nuevos y dejaría de
importarlos sin dar ningún error. La identidad de un evento es `(canal, RecordId)`, no su fecha, de
modo que un cambio del reloj del sistema tampoco produce duplicados.

#### 3.8 Alcance de esta verificación

Los datos vienen de **un** equipo Windows 11 x64 en español, con discos internos NVMe y SATA y uso de
discos externos USB. Sirve para saber qué es ruido de fondo y qué identificadores existen de verdad,
que era lo que faltaba. **No** cubre servidores, RAID por hardware ni Storage Spaces en producción:
esos eventos están tomados de los manifiestos, no observados, y siguen pendientes de contraste en la
Fase 0. Un equipo sano no produce eventos de fallo real, así que la ausencia de `Ntfs` 55 o `disk` 7
en la muestra es una buena noticia, no una señal de que no existan.

### 4. Textos

Cada regla que el motor puede emitir necesita en `es.json` y `en.json`:

```
alert.rule.<rule_key>.title      Titular corto, sin jerga.
alert.rule.<rule_key>.summary    Una frase que explique qué significa y por qué importa.
```

`AlertCard` y el detalle (`src/routes/alerts/+page.svelte`) resuelven el título y el resumen desde
`ruleKey` (ADR-030). La rejilla de hechos del detalle usa `labelKey` que **manda el backend**
(`alert.fact.*`), no una clave por regla. Un `.action` por regla queda pendiente para cuando haya
acciones concretas que ofrecer.

Norma de redacción: el titular dice **qué pasa**, no qué contador se ha movido. "El disco reserva
menos bloques de repuesto de los que su fabricante considera seguros" es un titular; "available
spare por debajo del threshold" es una clave técnica, y las claves técnicas solo aparecen en el
detalle, nunca en la lista (`AGENTS.md` §3, nota de `AlertCard`).

---

### 5. Pruebas exigidas

Toda regla de la tabla §2 necesita, como mínimo:

1. Un test de activación con datos fixture reales (ATA, NVMe, USB, RAID, VM).
2. Un test de **no** activación en el caso "dato ausente": nunca se alerta por falta de dato.
3. Un test de histéresis: oscilar alrededor del umbral no debe producir más de un grupo.
4. Un test de deduplicación: N evaluaciones equivalentes producen 1 grupo y N ocurrencias.
5. Un test de ciclo: resolver y recaer incrementa `cycle` y conserva el contador histórico.


---

# 8. Contrato UI ↔ backend

Fichero de origen: `docs/ui-contract.md`

Superficie completa entre la interfaz Svelte y el backend Rust. Es normativo: la UI **solo** puede
invocar lo que aparece aquí, y el backend no puede cambiar una firma sin actualizar este documento.

Los tipos se escriben en TypeScript porque es el lado que los consume, pero **la fuente es Rust**:
se generan con `ts-rs` y CI falla si el `.ts` generado no coincide con el del repositorio
(`open-questions.md` G.3). Este archivo documenta la intención; el `.ts` generado documenta la forma.

Convenciones:

- Toda fecha es **ISO 8601 en UTC**. La presentación en hora local es responsabilidad de la UI.
- Todo tamaño es **bytes**; toda temperatura, **grados Celsius**; todo caudal, **bytes por segundo**.
- Un dato que no se ha podido obtener es `null`. **Nunca `0`, nunca `-1`, nunca cadena vacía.**
- Un array vacío significa "ninguno"; `null` en su lugar significa "no se ha podido saber".

---

### 1. Errores

Todo comando que falle rechaza con un `AppError`. No hay excepciones: un `invoke` nunca devuelve
una cadena suelta ni un error de serialización sin envolver.

```ts
interface AppError {
  code: string;                                   // "smartctl.timeout", "db.locked", "test.busy"
  messageKey: string;                             // clave i18n de la frase humana
  messageVars?: Record<string, string | number>;
  detail?: string | null;                         // stderr, código de salida… literal, sin traducir
  source?: MetricSource | null;                   // qué fuente falló, si aplica
  retryable: boolean;                             // ¿tiene sentido repetir la misma acción?
}
```

La UI muestra siempre `t(messageKey, messageVars)` y guarda `detail` dentro de un `<details>`
copiable. Nunca enseña `detail` solo, y nunca oculta `detail` del todo.

**Un fallo de fuente no es un fallo de aplicación.** Si un recopilador cae, se degrada su tarjeta
con el error y el resto de la interfaz sigue funcionando (`AGENTS.md` §5).

#### Códigos previstos

| `code` | Cuándo | `retryable` |
|---|---|---|
| `smartctl.not_found` | falta el binario auxiliar | no |
| `smartctl.timeout` | la consulta excedió el tiempo máximo | sí |
| `smartctl.exit_status` | código de salida con bits de error | sí |
| `smartctl.unsupported` | el dispositivo no expone SMART | no |
| `device.not_found` | el `device_id` ya no existe | no |
| `volume.not_found` | el `volume_id` ya no existe | no |
| `test.busy` | ya hay una prueba en ese disco (mismo disco físico subyacente, no solo el mismo id) | no |
| `test.unsupported` | el dispositivo no admite esa prueba | no |
| `test.insufficient_space` | no cabe el archivo con la reserva | no |
| `test.io_failed` | fallo de E/S al preparar o ejecutar la prueba (crear la carpeta, lanzar el proceso auxiliar…) | sí |
| `db.locked` | SQLite ocupado más allá del tiempo de espera | sí |
| `db.migration_failed` | migración fallida; se ha restaurado la copia previa | no |
| `path.invalid` | ruta fuera de las carpetas permitidas | no |
| `export.write_failed` | no se pudo escribir el destino | sí |
| `settings.out_of_range` | valor fuera de los límites de `open-questions.md` D.1 | no |
| `db.query_failed` | fallo de SQLite que no es un bloqueo (`db.locked`, más arriba, es el que sí lo es) | no |
| `windows_storage.failed` | falló la consulta de inventario vía PowerShell | sí |
| `app.log_reload_failed` | no se pudo aplicar en caliente el nuevo nivel de registro | sí |
| `app.open_folder_failed` | no se pudo abrir el explorador de archivos en la carpeta de registro | sí |

---

### 2. Tipos compartidos

Los que ya viven en `src/lib/design/types.ts` no se repiten aquí: `HealthState`,
`Severity`, `AlertStatus`, `TestStatus`, `MetricSource`, `MetricQuality`, `UnknownReason`,
`Provenance`, `DiskSummary`, `VolumeSummary`, `AlertGroup`.

```ts
type Resolution = "raw" | "five_minutes" | "hourly";
type MappingConfidence = "exact" | "inferred" | "unknown";
type IdentityConfidence = "serial" | "fingerprint";

/** Estado de una fuente de datos. Arquitectura §7. */
type SourceStatus = "ok" | "partial" | "unsupported" | "timeout" | "error";

interface SourceHealth {
  source: MetricSource;
  status: SourceStatus;
  lastSuccessAt: string | null;
  lastAttemptAt: string | null;
  error?: AppError | null;
}
```

---

### 3. Comandos

#### 3.1 Apariencia y ajustes

```ts
invoke<AppearanceSettings>("get_appearance_settings")
interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "es" | "en" | null;   // null = seguir al sistema
  systemLocale: string;           // BCP-47 de Windows, p. ej. "es-ES". NO usar navigator.language
  useSystemAccent: boolean;       // valor de fábrica: false (v3, ADR-035) — la app estrena paleta propia
}

invoke<WindowsAccent>("get_system_accent_color")   // error si el usuario lo tiene desactivado
interface WindowsAccent {
  hex: string;        // #RRGGBB. OJO: el registro lo guarda en ABGR, no en RGB (open-questions.md O.7)
  palette?: string[]; // los 7 tonos de AccentPalette, del más claro al más oscuro
}

invoke<Settings>("get_settings")
invoke<void>("set_setting", { key: string, value: unknown })   // valida rango; AppError si no cabe
invoke<Settings>("reset_settings", { scope: "all" | "alerts" | "schedule" | "retention" })

interface Settings {
  schedule: {
    metricsFastSeconds: number;   // 30 s de fábrica, 10-300 (D.1)
    smartFullSeconds: number;     // 300 s de fábrica, 60-3600
    eventsSeconds: number;        // 30 s de fábrica, 15-300
    discoverySeconds: number;     // 60 s de fábrica, 30-600
  };
  alerts: {
    profile: "cautious" | "balanced" | "quiet" | "custom";  // "balanced" de fábrica (ADR-036). Elegir un perfil concreto reescribe los 12 umbrales; editar un umbral a mano ⇒ "custom"
    tempConfiguredWarnC: number;  // 60 de fábrica (ADR-036), 40-95 — solo sin límite del fabricante
    tempConfiguredCritC: number;  // 70 de fábrica, entre tempConfiguredWarnC y 100
    wearWarnPercent: number;      // 80 de fábrica, 50-99
    wearCritPercent: number;      // 90 de fábrica, entre wearWarnPercent y 100
    capacityWarnPercent: number;  // 10 de fábrica, 1-50 (C.1/ADR-019)
    capacityCritPercent: number;  // 5 de fábrica, 1-50, menor que capacityWarnPercent
    capacityAbsoluteFloorMinCapacityBytes: number;  // 256 GiB de fábrica: a partir de aquí también cuenta el suelo absoluto
    capacityAbsoluteFloorWarnBytes: number;         // 20 GiB de fábrica
    capacityAbsoluteFloorCritBytes: number;         // 10 GiB de fábrica, menor que el de aviso
    mediaErrorsWarnPer24h: number;  // 1 de fábrica, 1-1000. Nombre histórico: es el incremento del contador que basta para avisar, no una ventana de 24 h (ADR-036)
    mediaErrorsCritPer24h: number;  // 5 de fábrica, entre el de aviso y 1000
    driverRetryWarnPer24h: number;  // 5 de fábrica, 1-1000. Aún sin consumidor: la regla events.* necesita el colector de eventos
    driverRetryCritPer24h: number;  // 12 de fábrica, entre el de aviso y 1000
  };
  onboarding: {
    // ISO-8601 UTC o null. null ⇒ el guardián de `+layout.ts` redirige a `/onboarding` al arrancar,
    // salvo que ya exista configuración previa (FR-043), en cuyo caso lo graba solo. Se escribe con
    // `set_setting("settings.onboarding.completed_at", <fecha>|null)` (clave en la lista blanca desde
    // PR 5). «Repetir la configuración inicial» de Ajustes lo pone a null.
    completedAt: string | null;
  };
  retention: {
    rawDays: number;              // 7 de fábrica, 1-30 (J.14)
    fiveMinutesDays: number;      // 90 de fábrica, 7-365
    hourlyDays: number;           // 730 de fábrica, 90-1825
    freeSpaceWarnBytes: number;   // 1 GiB de fábrica (J.13); sin límites de edición propios
    freeSpaceHaltBytes: number;   // 256 MiB de fábrica
  };
  lifecycle: {
    closeAction: "minimize" | "exit";   // "minimize" de fábrica
    closeActionRemembered: boolean;
    startWithSystem: boolean;     // false de fábrica; al activarlo se registra una tarea programada
                                  //   elevada (`schtasks`, ADR-038). Clave: `lifecycle.start_with_system`
  };
  notifications: {
    soundEnabled: boolean;        // false de fábrica (US-072)
    enabled: boolean;             // true de fábrica; false oculta el toast sin pausar (ADR-037).
                                  //   Clave: `notifications.enabled`
  };
  logging: {
    verbose: boolean;             // ver `set_log_level`, §3.9: no se cambia con `set_setting`
  };
}
```

`Settings` es un objeto tipado, no un diccionario libre. Sus límites están en `open-questions.md`
D.1/C.1/J.13/J.14/J.32 y los valida el backend: la UI puede confiar en que un valor guardado es un
valor legal. La apariencia (`theme`/`language`/`useSystemAccent`) no vive en `Settings`: sigue
teniendo su propio `get_appearance_settings()`; se persiste con el mismo `set_setting(key, value)`
genérico, con las claves `settings.appearance.theme`, `settings.appearance.language` y
`settings.appearance.use_system_accent`. `reset_settings` con `scope: "all"` también restaura
`lifecycle`/`notifications`/`logging`, que no tienen su propio ámbito de reinicio (y al borrar
`lifecycle.start_with_system` también quita la tarea programada de autoarranque); nunca toca la
apariencia ni `settings.onboarding.completedAt`.

#### 3.2 Inventario

```ts
invoke<DeviceListResponse>("get_devices")
interface DeviceListResponse {
  devices: DiskSummary[];
  excluded: DiskSummary[];        // desactivados por el usuario; US-011 exige mostrarlos aparte
  sources: SourceHealth[];        // estado de cada recopilador
  paused: boolean;
  pausedSince: string | null;
  historyWriteHalted: boolean;    // FR-020a/b: volumen del historial bajo el umbral de parada
}

invoke<DeviceDetail>("get_device_detail", { deviceId: string })
interface DeviceDetail extends DiskSummary {
  fingerprint: string;
  identityConfidence: IdentityConfidence;
  serialNumber: string | null;
  firmware: string | null;
  busType: string | null;
  capabilities: DeviceCapability[];
  counters: SmartCounter[];       // contadores normalizados con su delta
  smartRaw: SmartRawInfo | null;  // metadatos; el JSON completo se pide aparte
  firstSeenAt: string;
  lastSeenAt: string;
  removedAt: string | null;
}

interface DeviceCapability {
  key: "smart" | "nvme_log" | "self_test_short" | "chkdsk_scan" | "temperature";
  available: boolean;
  reasonKey: string | null;       // por qué no, en lenguaje humano
}

interface SmartCounter {
  metricKey: string;
  value: number | null;
  unit: string | null;
  /** Incremento respecto a la lectura anterior; null si no hay lectura previa. */
  delta: number | null;
  /** Si el incremento es en sí mismo una mala señal. Decide si `DataRow` colorea el delta. */
  deltaIsMeaningful: boolean;
  provenance: Provenance;
}

invoke<void>("set_device_monitoring", { deviceId: string, enabled: boolean })
invoke<void>("set_device_alias", { deviceId: string, alias: string | null })
invoke<void>("refresh_now", { scope: "all" | "device", deviceId?: string })
```

`refresh_now` es idempotente: si ya hay una recopilación igual en curso, devuelve sin encolar otra
(US-013). No es un error; la respuesta lo indica en el evento `metrics:updated` correspondiente.

#### 3.3 Series temporales

```ts
invoke<MetricSeries>("get_metric_series", {
  deviceId?: string,
  volumeId?: string,
  metricKey: string,
  fromUtc: string,
  toUtc: string
})

interface MetricSeries {
  metricKey: string;
  unit: string;
  /** Resolución realmente servida, que puede no ser la ideal: la UI la muestra al usuario. */
  resolution: Resolution;
  /** true si se ha submuestreado para respetar el tope de 1.500 puntos. */
  downsampled: boolean;
  /** El intervalo pedido, devuelto tal cual: el eje lo cubre entero aunque falten datos. */
  fromUtc: string;
  toUtc: string;
  /** Cadencia esperada entre puntos; alimenta la detección de huecos del gráfico. */
  expectedIntervalMs: number;
  points: { t: number; v: number | null }[];
  /** Umbrales que la gráfica debe dibujar como línea discontinua. */
  vendorLimit: number | null;
  vendorCritical: number | null;
}
```

La tabla intervalo → resolución está en `open-questions.md` E.1. **Los huecos se devuelven como
huecos**: el backend no interpola ni rellena con ceros, y un tramo sin datos simplemente no trae
puntos.

#### 3.4 Alertas

```ts
invoke<AlertGroup[]>("get_alert_groups", { status?: AlertStatus[], deviceId?: string })
invoke<AlertDetail>("get_alert_detail", { alertGroupId: string })

interface AlertDetail extends AlertGroup {
  facts: { labelKey: string; value: string | null }[];
  occurrences: {
    occurredAt: string;
    cycle: number;
    value: number | null;
    eventId: string | null;
    context: string | null;
  }[];
  relatedEvents: SystemEvent[];
}

invoke<void>("acknowledge_alert", { alertGroupId: string })
invoke<void>("mute_alert", { alertGroupId: string, minutes: 15 | 60 | 480 | null })  // null = indefinido
invoke<void>("unmute_alert", { alertGroupId: string })
invoke<void>("archive_alert", { alertGroupId: string })
```

Reconocer **no** cambia el color de nada: el color lo decide `deviceState()` sobre las alertas
`active` y `acknowledged` (`alert-rules.md` §1).

#### 3.5 Eventos

```ts
invoke<SystemEventPage>("get_system_events", {
  deviceId?: string, volumeId?: string,
  levels?: ("error" | "warning" | "info")[],
  providers?: string[],
  fromUtc?: string, toUtc?: string,
  cursor?: string, limit?: number        // paginación: la lista se virtualiza
})

interface SystemEvent {
  id: string;
  occurredAt: string;
  provider: string;
  eventId: number;
  level: "error" | "warning" | "info";
  message: string;                       // en el idioma de Windows, no en el de la app
  deviceId: string | null;
  volumeId: string | null;
  mappingConfidence: MappingConfidence;
  hasRawXml: boolean;
}

interface SystemEventPage { events: SystemEvent[]; nextCursor: string | null; total: number | null }

invoke<string>("get_event_raw_xml", { eventId: string })
```

El `message` llega en el idioma de Windows y se muestra tal cual, marcado como texto original del
sistema (`open-questions.md` J.4). **Se renderiza como texto, jamás como HTML.**

La ruta acepta `?focus=<system_events.id>` (spec `003-puente-eventos-alertas`): al llegar desde el
enlace «Ver el suceso» del detalle de una alerta de evento, la pantalla resalta y abre ese suceso.
Un id que no esté en la página cargada no es un error: la pantalla se comporta como sin parámetro.

#### 3.6 Pruebas

```ts
invoke<string>("start_benchmark", {          // devuelve testRunId
  volumeId: string,
  sizeBytes: number,
  blockSizeBytes: number,
  mode: "sequential" | "random",
  passes: number
})
invoke<string>("run_chkdsk_scan", { volumeId: string })
invoke<string>("run_smart_short_test", { deviceId: string })
invoke<void>("cancel_test", { testRunId: string })

invoke<TestRun[]>("get_test_runs", { deviceId?: string, limit?: number })
interface TestRun {
  id: string;
  type: "benchmark" | "chkdsk_scan" | "smart_short";
  deviceId: string | null;
  volumeId: string | null;
  status: TestStatus;
  startedAt: string;
  finishedAt: string | null;
  progressPercent: number | null;
  /** Comando literal ejecutado, para mostrarlo en el ConfirmDialog y en el historial. */
  command: string | null;
  parameters: Record<string, unknown>;
  result: TestResult | null;
  output: string | null;                 // salida capturada; texto, nunca HTML
  /** Página de códigos deducida para `output` ("cp1252", "cp850", "utf-8"…). Los bytes originales
   *  se conservan aparte: la decodificación es de presentación (open-questions.md §Q). */
  outputEncoding: string | null;
  /** Si quedó un archivo temporal sin borrar, su ruta, para que el usuario pueda limpiarla. */
  orphanPath: string | null;
}

interface TestResult {
  passed: boolean | null;
  readBytesPerSecond: number | null;
  writeBytesPerSecond: number | null;
  readLatencyMs: number | null;
  writeLatencyMs: number | null;
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "space" | "error" | null;
}
```

Parámetros por defecto del benchmark en `product-specification.md` §6. La UI nunca construye el
comando: lo recibe ya formado en `command` solo para mostrarlo.

#### 3.7 Informes y diagnóstico

```ts
invoke<string>("export_report", {          // devuelve la ruta escrita
  format: "csv" | "json" | "html",
  fromUtc: string, toUtc: string,
  deviceIds: string[] | null,              // null = todos los monitorizados
  includeSerials: boolean,
  destinationPath: string
})

invoke<DiagnosticPreview>("preview_diagnostic_zip", { includeIdentifiers: boolean })
interface DiagnosticPreview {
  entries: { path: string; sizeBytes: number; descriptionKey: string }[];
  totalBytes: number;
  redactedFields: string[];                // qué se va a anonimizar, para enseñarlo antes de guardar
}
invoke<string>("create_diagnostic_zip", { includeIdentifiers: boolean, destinationPath: string })
```

US-051 exige mostrar un resumen del contenido antes de guardar: para eso está
`preview_diagnostic_zip`, que no escribe nada.

#### 3.8 Ciclo de vida

```ts
invoke<void>("pause_monitoring")
invoke<void>("resume_monitoring")
invoke<AppInfo>("get_app_info")            // nombre, versión y autor desde el manifiesto (ADR-011)
invoke<void>("delete_all_data", { confirmationPhrase: string })   // US-073
```

`delete_all_data` exige que el usuario escriba una frase de confirmación, no solo que pulse un
botón: es irreversible y borra el historial completo.

#### 3.9 Registro de actividad

```ts
invoke<LogLevel>("get_log_level")
invoke<void>("set_log_level", { verbose: boolean })   // US-071, FR-029a
invoke<void>("open_log_folder")                       // US-071, FR-029b
```

`open_log_folder` abre **una sola ruta conocida** —la carpeta de registro resuelta por
`platform::paths`—, sin recibirla como argumento desde la interfaz: un parámetro de ruta abriría
una segunda vía de acceso al sistema de ficheros, que es justo lo que el principio IX prohíbe.

`log_from_ui` no es de este bloque: es el envoltorio interno que usa `$lib` para escribir en la
única API de registro (constitución §XV); no lo invoca ninguna pantalla directamente.

---

### 4. Eventos emitidos por el backend

La UI **no hace sondeo**. El backend empuja (ADR-015). Cada carga útil lleva `emittedAt` para poder
descartar mensajes fuera de orden.

| Evento | Carga útil | Cuándo |
|---|---|---|
| `metrics:updated` | `{ emittedAt, devices: DiskSummary[], sources: SourceHealth[], historyWriteHalted: boolean }` | al cerrar cada ciclo de recopilación |
| `alerts:changed` | `{ emittedAt, changed: AlertGroup[], removed: string[] }` | alta, cambio de severidad o de estado, resolución |
| `inventory:changed` | `{ emittedAt, added: DiskSummary[], removed: string[], updated: DiskSummary[] }` | alta o retirada de disco o volumen |
| `test:progress` | `{ emittedAt, testRun: TestRun }` | mientras una prueba avanza |
| `source:degraded` | `{ emittedAt, source: SourceHealth }` | una fuente pasa a `timeout` o `error` |
| `system:accent-changed` | `{ hex }` | el usuario cambia el acento de Windows |
| `system:theme-changed` | `{ dark: boolean }` | el usuario cambia el tema de Windows |
| `monitoring:paused` / `monitoring:resumed` | `{ emittedAt, since }` | pausa desde la bandeja o desde la UI |

Los eventos son **incrementales pero autosuficientes**: cada uno trae el objeto completo que ha
cambiado, no un parche. La UI puede reemplazar por identificador sin reconciliar.

Al montar, la UI pide el estado completo con los comandos `get_*` y a partir de ahí solo escucha.
Tras una reconexión o un error de deserialización, vuelve a pedir el estado completo en lugar de
intentar recomponerlo.

---

### 5. Permisos Tauri

La política de capacidades es de mínimo privilegio dentro de un proceso ya elevado (ADR-004):

- **Sin** `shell:allow-execute` genérico. Los procesos auxiliares se lanzan desde Rust con
  ejecutable y argumentos de una lista cerrada.
- **Sin** `fs` genérico. Las rutas se calculan en Rust y se validan canónicamente antes de crear o
  borrar nada.
- El `dialog` de selección de destino es la **única** vía por la que una ruta elegida por el usuario
  entra en el backend, y aun así se valida.
- Cualquier permiso nuevo requiere una entrada en `docs/decisions.md`. La definición de terminado de
  una pantalla incluye "sin permisos Tauri nuevos".


---

# 9. Convenciones de ingeniería

Fichero de origen: `docs/engineering-conventions.md`

Lo que un programador necesitaría asumir si no estuviera escrito. Todo lo de aquí es normativo pero
revisable: si una convención estorba, se cambia en este documento y se aplica en todo el proyecto,
no se hace una excepción local.

---

### 1. Pila y versiones

| Pieza | Versión | Nota |
|---|---|---|
| Tauri | 2.x | ADR-001 |
| Rust | edición 2021, MSRV 1.77 | se fija en `rust-toolchain.toml` para que CI y desarrollo coincidan |
| Node | 20 LTS | se fija en `.nvmrc` |
| Gestor de paquetes | pnpm | `packageManager` en `package.json`; el lockfile se versiona |
| SvelteKit | 2.x con `adapter-static` | ADR-014, SSR desactivado |
| Svelte | 5 con runes | `ui-design.md` §1 |
| TypeScript | 5.x, `strict: true` | sin `any` implícito, sin `@ts-ignore` sin justificar |
| Tailwind | 3.x | solo utilidades mapeadas desde tokens |
| SQLite | vía `rusqlite` con `bundled` | evita depender de la DLL del sistema |
| Vitest | 5.x | dos configuraciones: Node y navegador (ADR-027) |
| Playwright | 1.x | solo Chromium: es el motor del WebView2 (ADR-028) |
| `@axe-core/playwright` | 4.x | accesibilidad automática, ambos temas |

Windows mínimo soportado: **Windows 10 1809 (build 17763)** y **Windows Server 2016**, x64. Edge y
WebView2 llegan en realidad hasta Windows 10 1709, pero por debajo de 1809 las APIs de
almacenamiento dejan de comportarse de forma homogénea y no se van a probar ahí. Microsoft mantiene
actualizaciones de WebView2 en Windows 10 22H2 al menos hasta octubre de 2028.

WebView2 se empaqueta con el instalador sin conexión (ADR-020):

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": { "type": "offlineInstaller" }
    }
  }
}
```

**No lo cambies a `fixedRuntime` por ahorrar tamaño**: congelaría Chromium sin parches de seguridad
en un producto que no tiene actualizador automático. El razonamiento completo está en el ADR-020.

---

### 2. Estructura del proyecto

```
smartdisk-monitor/
  src/                          Frontend SvelteKit
    routes/                     Una carpeta por pantalla
      +layout.svelte            Arranque: tokens.css, tema, idioma, acento, suscripción a eventos
      +page.svelte              Panel general
      disks/[id]/               Detalle de disco
      alerts/                   Alertas
      events/                   Eventos
      tests/                    Pruebas y diagnóstico
      reports/                  Informes
      settings/                 Ajustes
      onboarding/               Asistente inicial
    lib/
      components/               Catálogo cerrado (del paquete de diseño)
      design/                   types, format, health, theme, accent
      i18n/                     Diccionarios
      api/                      Envoltorios tipados de invoke y listen. NINGUNA pantalla llama a
                                invoke directamente: siempre a través de aquí
      stores/                   Estado de aplicación en runes
    design-system/              tokens.css, tokens.json, fonts/ (del paquete de diseño)

  src-tauri/
    src/
      main.rs
      commands/                 Un módulo por área; cada comando es una función fina que valida y
                                delega en domain/
      domain/                   Reglas de negocio. Sin dependencias de Tauri: es lo que se testea
      collectors/               smartctl, windows_storage, perf_counters, event_log
      alerts/                   Motor de reglas (docs/alert-rules.md)
      tests/                    Benchmark, chkdsk, autotest
      persistence/              Repositorios, migraciones, retención
      reporting/                Exportaciones y ZIP de diagnóstico
      platform/                 Envolturas de API de Windows, aisladas para poder simularlas
    migrations/                 SQL numerado, nunca editado una vez publicado
    capabilities/               Política Tauri de mínimo privilegio
    windows/app.manifest        requireAdministrator (ADR-004) + PerMonitorV2

  docs/                         Documentación normativa
  design/                       Bocetos navegables (.dc.html). Referencia visual, no código

  AGENTS.md                     Instrucciones para agentes de IA: fuente canónica
  CLAUDE.md                     Importa AGENTS.md y añade lo específico de Claude Code
  GEMINI.md                     Importa AGENTS.md y añade lo específico de Gemini
  CODEX.md                      Puntero a AGENTS.md, que Codex ya lee de forma nativa
  .claude/rules/                Reglas por ámbito; se cargan al tocar sus `paths:`
  .claude/skills/               Procedimientos; se cargan al activarse
  .claude/settings.json         Permisos y hooks (enforcement)
  .claude/hooks/                Scripts de los hooks
```

#### Instrucciones para agentes de IA

`AGENTS.md` es el núcleo y se carga en cada sesión: **objetivo, menos de 200 líneas**. No se
escribe ahí nada que un agente pueda deducir leyendo el repositorio, porque cuanto más ruido, menos
adherencia a lo que importa.

| Si algo… | Va a |
|---|---|
| Hace falta en cualquier tarea | `AGENTS.md` |
| Solo al tocar cierto código | `.claude/rules/<tema>.md` con `paths:` |
| Es un procedimiento de varios pasos | `.claude/skills/<nombre>/SKILL.md` |
| Es estado o historia del producto | `docs/` o `specs/` |
| Debe cumplirse siempre y de forma determinista | Linter, prueba, verificador o CI |

Los `@imports` **no ahorran contexto**: se expanden al arrancar. Lo que ahorra es `paths:` en las
reglas y la carga diferida de las skills, de las que en el arranque solo entran nombre y
descripción.

**Regla de dependencias.** `domain/` no conoce Tauri, ni Windows, ni SQLite: recibe datos y devuelve
decisiones. Es lo que permite probar el motor de alertas con fixtures y sin hardware.

**Ninguna pantalla llama a `invoke` directamente.** Todo pasa por `src/lib/api/`, que es donde viven
los tipos generados, el manejo de `AppError` y la suscripción a eventos. Así, cuando una firma
cambia, rompe en un sitio y no en once.

---

### 3. Calidad

| Herramienta | Qué exige |
|---|---|
| `rustfmt` | formato; CI falla si hay diferencias |
| `clippy` | `-D warnings`; nada de `unwrap()` fuera de tests |
| `eslint` + `svelte-check` | sin errores de tipo ni de accesibilidad |
| `prettier` | formato del frontend |
| `cargo test` / `vitest` | pruebas unitarias |
| `ts-rs` | los DTO generados deben coincidir con los versionados (puerta pendiente: se activa con el primer DTO real) |
| `@vitest/coverage-v8` | umbrales del frontend, en `vitest.config.ts` |
| `cargo-llvm-cov` | umbrales del backend, en CI |

#### Comprobaciones propias del proyecto

Estas no las da ninguna herramienta estándar; hay que escribirlas, y son las que impiden que el
sistema de diseño se erosione:

1. **Sin valores visuales literales.** Un lint que falle si un `.svelte` contiene un color
   hexadecimal, un `rgb(`, un `border-radius` en px, un `box-shadow` literal o un `font-size` en px
   fuera de `tokens.css`.
2. **Sin `backdrop-filter` a mano.** Solo puede aparecer en `tokens.css`.
3. **Sin literales de interfaz.** Todo texto visible, incluidos `aria-label`, `title` y `alt`, debe
   venir de `t()` o `tp()`. Un lint que detecte cadenas literales en marcado.
4. **Paridad de diccionarios.** `es.json` y `en.json` deben tener exactamente el mismo juego de
   claves. Ya se comprueba trivialmente y evita textos que solo existen en un idioma.
5. **La tipografía existe y es la que dice ser.** La compilación falla si falta cualquiera de los dos
   `.woff2` de `design-system/fonts/` o si su SHA-256 no coincide con el registrado en
   `THIRD_PARTY_NOTICES.md`: sin ellos la aplicación se ve distinta de los bocetos aprobados y nadie
   se entera. El empaquetado debe copiarlos junto con `OFL.txt`.
6. **Sin permisos Tauri nuevos.** Un diff sobre `capabilities/` que exija revisión explícita.
7. **Decodificación de procesos auxiliares.** Test con volcados reales de `chkdsk` (CP1252) y de
   `fsutil` (CP850) que verifique que la detección de `open-questions.md` §Q elige bien en ambos.
   Los volcados se guardan como fixtures: es la única forma de que una regresión aquí se note.

---

### 4. Pruebas

| Nivel | Qué cubre | Dónde |
|---|---|---|
| Unitarias Rust | parsers de smartctl, normalización, reglas de alerta, retención, rutas seguras | `domain/`, `alerts/` |
| Fixtures | salidas reales anonimizadas de ATA, NVMe, USB, RAID y VM | `src-tauri/tests/fixtures/` |
| Integración | `smartctl` simulado: salidas válidas, timeouts, códigos de salida con bits, JSON corrupto | `src-tauri/tests/` |
| Migraciones | migrar desde cada versión publicada hasta la actual, con copia previa | `persistence/` |
| Unitarias TS | `format`, `health`, `accent`, `i18n` | `pnpm test`, jsdom |
| Componentes | estados vacío, cargando, no compatible, error y dato obsoleto de cada componente, más lo que solo se ve en un navegador real: contraste sobre material, respaldo sin `backdrop-filter`, foco visible | `pnpm test:component`, Chromium real. Sufijo `*.browser.test.ts` |
| Interfaz | arranque, chrome, navegación, tema, tipografía, errores de consola | `pnpm test:e2e`, Playwright con IPC propio |
| Accesibilidad | foco, teclado, contraste AA, `prefers-reduced-motion` | automatizado donde se pueda, lista de comprobación donde no |
| Visuales | ambos temas, acento del sistema y de respaldo, sin `backdrop-filter`, 1024 × 560 y 1280 × 720, escalado 125/150/200 % | capturas comparadas; `tools/scale-check.html` como banco de pruebas |
| Manuales | hardware real, sin exigir una marca concreta | documentadas en el informe de Fase 0 |

**Los datos fixture nunca contienen números de serie ni nombres de equipo reales.** Se anonimizan al
capturarlos, no al usarlos.

---

### 5. Git y entrega

- Rama principal protegida; el trabajo va en ramas por historia (`us-030-alertas-agrupadas`).
- Commits en imperativo, en español, referenciando la historia.
- Un *pull request* no se fusiona sin la definición de terminado de la historia y, si toca interfaz,
  la de `ui-design.md` §8.
- Versionado semántico. Nombre y versión salen del manifiesto (ADR-011): no se escriben a mano en
  ningún otro sitio.
- Las publicaciones son manuales en GitHub, sin actualizador automático (ADR-007).

---

### 6. Datos y rutas en desarrollo

En desarrollo, la base de datos **no** va a `C:\ProgramData\SmartDisk Monitor\`: va a una carpeta
local ignorada por git, para no mezclar datos reales con pruebas ni exigir elevación en cada
ejecución de test. La ruta se resuelve siempre por función, nunca por literal, y el modo se decide
por variable de entorno.

Ejecutar la aplicación completa **sí** requiere elevación, también en desarrollo: es la única forma
de que lo que se prueba sea lo que se entrega. En la práctica, `pnpm app:dev` muestra el diálogo de
UAC en cada arranque.

Las suites de prueba **no** necesitan elevación, con una excepción: la de aplicación real
(`test:e2e:app`), que arranca el ejecutable y por tanto debe lanzarse desde un terminal de
administrador para que el hijo herede la elevación y UAC no bloquee la automatización
(`docs/testing-strategy.md` §11).


---

# 10. Estrategia integral de testing

Fichero de origen: `docs/testing-strategy.md`

Documento normativo. Desarrolla el principio VIII de la constitución (testeabilidad y cobertura
mínima) y no puede relajarlo.

**Este proyecto no es una aplicación web.** Es una aplicación de escritorio Tauri con un frontend
SvelteKit compilado a estático. Buena parte del repertorio habitual de testing de SvelteKit —SSR,
hidratación, Server Actions, endpoints, cookies, sesiones— **no existe aquí y no se puede probar**.
Lo primero que hace este documento es decir qué no aplica y por qué, para que nadie monte
infraestructura para algo que no está.

---

### 1. Clasificación del proyecto y runtime real

| | |
|---|---|
| Tipo | Aplicación de escritorio de un solo usuario, sin red |
| Shell | Tauri 2, proceso Rust elevado (`requireAdministrator`) |
| Frontend | SvelteKit con `adapter-static`, `ssr = false`, `prerender = true` |
| Motor de render | WebView2 (Chromium), embebido en el proceso |
| Backend | Rust en el mismo proceso; se comunica por IPC de Tauri |
| Persistencia | SQLite local en `%ProgramData%` |
| Plataforma | Windows x64 exclusivamente |

**Frontera principal a probar: el IPC.** No hay HTTP, no hay sesión, no hay usuario remoto. Todo lo
que cruza entre TypeScript y Rust pasa por `invoke` y `listen`, y es ahí donde se concentra el
riesgo de contrato.

#### Reparto del código

| Ámbito | Dónde | Cómo se prueba |
|---|---|---|
| Solo navegador | `src/lib/components/`, `src/routes/` | Componentes en navegador real, E2E de interfaz |
| Solo servidor | `src-tauri/src/` | `cargo test`, con fixtures anonimizados |
| Compartido de facto | `src/lib/api/schemas.ts` ↔ tipos de Rust | Contrato: mismo caso de prueba a ambos lados |
| Presentación | `src/lib/design/`, `src/lib/i18n/` | Unit tests en Node |
| Estado | `src/lib/stores/` | Unit tests en Node |

---

### 2. Lo que NO aplica, y por qué

No se monta infraestructura para nada de esto. Está escrito para que nadie lo proponga otra vez.

| Técnica habitual | Por qué no aplica |
|---|---|
| Testing de SSR | `ssr = false`. No hay render en servidor que comparar |
| Testing de hidratación | Sin SSR no hay hidratación: el HTML llega vacío y lo pinta el cliente |
| Server Actions | No hay servidor. Las mutaciones son comandos Tauri |
| Endpoints `+server` | Cero endpoints. Cero HTTP |
| Hooks (`hooks.server`, `hooks.client`) | No existen ni pueden existir |
| Cookies y sesiones | No hay sesión: un solo usuario local ya autenticado por Windows |
| Autenticación y autorización | La autorización es UAC, del sistema operativo. No hay roles ni permisos de aplicación |
| Testing de APIs externas | Cero red en funcionamiento normal (ADR-007) |
| Cross-browser | **Se distribuye un solo motor**: el WebView2 que instala el propio instalador. No hay Firefox ni WebKit que soportar. Probar en ellos mediría un entorno que ningún usuario tendrá |
| Prerendering dinámico | Las rutas se prerrenderizan al compilar salvo `/disks/[id]`; no hay contenido de servidor que validar |
| Testing de despliegue | La distribución es un instalador manual, no un servidor |

**Consecuencia práctica:** la matriz de navegadores tiene un solo elemento. Eso ahorra la parte más
cara de una estrategia E2E convencional y permite invertir ese presupuesto en regresión visual y
en el dominio.

---

### 3. Inventario de lo que ya existe

Verificado antes de escribir esta estrategia. **No se instala nada que ya esté resuelto.**

| Capa | Herramienta | Estado |
|---|---|---|
| Tipos y accesibilidad | `svelte-check` | En uso, cero errores y cero avisos exigidos (§XIII) |
| Análisis estático TS | `eslint` con `typescript-eslint` | En uso, sin errores. Centrado en promesas sin gestionar |
| Formato | `prettier` | En uso |
| Análisis estático Rust | `clippy -D warnings`, `rustfmt` | En uso |
| Unit TS | `vitest` 5 + `@vitest/coverage-v8` | En uso: 165 pruebas, umbral 70 % |
| Unit Rust | `cargo test` | En uso: 11 pruebas |
| Componentes | Vitest Browser Mode + `vitest-browser-svelte` | En uso: 8 pruebas (ADR-027) |
| Validación de contrato | `zod` | En uso en la frontera IPC (§XI) |
| Verificadores propios | 5 scripts en `scripts/` | En uso: recursos, tokens, i18n, fronteras |
| E2E de interfaz | `@playwright/test` + IPC propio | En uso: 17 pruebas (ADR-028) |
| Accesibilidad | `@axe-core/playwright` | En uso: 6 pantallas × 2 temas |
| E2E de aplicación real | `tauri-driver` | **No existe**: necesita el ejecutable empaquetado (US-060) |
| Visual | — | **No existe**: sin pantallas definitivas no hay línea base que fijar |
| Mutation | — | **No existe** |

#### Líneas base medidas (T-TEST-001)

Medidas el 2026-09-04, tras montar la infraestructura. Es el presupuesto que hay que preservar: si
una suite se sale de aquí, se investiga antes de subir el plazo.

| Suite | Comando | Ficheros | Pruebas | Duración |
|---|---|---|---|---|
| Lógica, Node | `pnpm test` | 11 | 165 | ~16 s |
| Componentes, Chromium | `pnpm test:component` | 2 | 8 | ~2 s |
| Interfaz, Playwright | `pnpm test:e2e` | 2 | 17 | ~38 s |
| Solo el smoke | `pnpm test:e2e:smoke` | 1 | 5 | ~33 s |
| Accesibilidad | `pnpm test:a11y` | 1 | 12 | ~37 s |
| Rust | `cargo test` | — | 11 | <1 s |

Las de Playwright incluyen compilar el frontend y arrancar la vista previa, que es la mayor parte
del tiempo. Ejecutar más pruebas contra un servidor ya arrancado sale casi gratis.

#### Deuda saldada, y lo que destapó

1. ~~`@testing-library/svelte` instalado sin un solo test~~ **retirado** (ADR-027). Su núcleo vuelve
   por vía transitiva dentro de `vitest-browser-svelte`, que es el envoltorio oficial: el Browser
   Mode no sustituye a Testing Library, la reempaqueta.
2. ~~Ningún test de componente ni E2E~~ **montados**. Encontraron **dos defectos reales** en su
   primera ejecución, ninguno de los cuales era visible en jsdom ni para `svelte-check`:
   - **El anillo de foco no aparecía en ningún control.** `:focus-visible` (0,1,0) empataba con las
     utilidades de Tailwind y perdía por orden de generación. Como la regla hace `outline: none`,
     los controles quedaban **sin ningún indicador de foco**: WCAG 2.4.7 incumplido en todo el
     catálogo. Arreglado subiendo la especificidad a `:focus-visible:focus-visible`.
   - **`StatusDot` emitía `role="img"` con nombre accesible vacío.** Un lector de pantalla anunciaba
     «imagen» y nada más, en cada disco de la barra lateral. Arreglado: decorativo cuando hay
     etiqueta visible, con nombre traducido cuando no la hay. De paso, `HealthDonut` tenía su
     `aria-label` en español a pelo, sin pasar por el diccionario.

---

### 4. Pirámide de niveles

#### Nivel 0 — Comprobaciones estáticas

No son tests y no sustituyen a ninguno: demuestran que el código es coherente, no que hace lo
correcto. Comandos reales:

```sh
pnpm check              # svelte-kit sync + svelte-check
pnpm lint               # prettier --check + eslint
pnpm verify             # recursos, tokens, i18n, fronteras
pnpm build              # compilación del frontend
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Se ejecutan **antes** que las pruebas (§XIII): un error de tipos invalida todo lo que venga después.

#### Nivel 1 — Unit (Node)

Lógica aislable, sin DOM ni IPC. Es la capa más barata y donde debe estar la mayoría.

#### Nivel 2 — Componentes (navegador real)

Componentes Svelte en Chromium mediante Vitest Browser Mode. **No jsdom**: el sistema de diseño
depende de `backdrop-filter`, `color-mix()`, variables CSS resueltas y `prefers-reduced-motion`, y
jsdom no implementa ninguna de esas cosas — un test de contraste o de material en jsdom no probaría
nada, solo daría una falsa sensación de cobertura.

#### Nivel 3 — Integración

Colaboración real entre piezas: esquema Zod contra respuesta serializada por Rust, store contra
eventos, retención contra SQLite en fichero temporal.

#### Nivel 4 — Aceptación

Criterios de aceptación de las historias convertidos en comportamiento verificable, al nivel más
barato que lo demuestre.

#### Nivel 5 — E2E, en dos planos

| Plano | Herramienta | Qué demuestra | Coste |
|---|---|---|---|
| **Interfaz** | Playwright contra `preview`, con `mockIPC` | Navegación, estados, formularios, responsive, accesibilidad, visual | Segundos |
| **Aplicación real** | WebdriverIO + `tauri-driver` contra el `.exe` | Que el binario arranca, pinta y habla con Rust | Minutos |

El segundo plano existe porque **el primero no puede demostrar que la aplicación real funcione**: un
fallo de empaquetado, de política de capacidades o de elevación no aparecería en ningún navegador.

#### Nivel 6 — Transversales

Responsive, accesibilidad, regresión visual. Cross-browser no aplica (§2).

#### Fuera de la pirámide — Mutation testing

No es una capa superior: mide si los tests que ya existen detectan errores. Se ejecuta de forma
programada, nunca en el bucle de desarrollo.

---

### 5. Lotes funcionales y checkpoints

La unidad de trabajo es el **lote funcional coherente**, no el fichero ni la función. Entre tres y
ocho tareas como orientación, sin convertirlo en regla.

**Lote más pequeño** cuando haya: escritura en disco, motor de alertas, migraciones, rutas del
benchmark, elevación de privilegios, concurrencia entre recopiladores.

**Lote mayor** cuando sea: componente presentacional, adaptador fino, refactorización interna,
traducción de textos.

#### Plan previo de cada lote

Antes de implementar se documenta en `tasks.md`: identificador, historia, comportamientos
observables, estados, errores, casos límite, riesgos, niveles de test previstos, datos y dobles
necesarios, suite de cierre y criterios del checkpoint.

No es obligatorio crear los ficheros de test por adelantado.

#### Cuándo se escribe el test

**Antes o inmediatamente después** —sin excepción— cuando se trate de:

- parsers de `smartctl` y de logs NVMe;
- motor de alertas: activación, histéresis, deduplicación, ciclo de recaída;
- retención, agregación y migraciones;
- validación de rutas del benchmark;
- cualquier defecto reproducido: la prueba que lo captura se escribe antes de arreglarlo.

Son las áreas donde el fallo es silencioso: no revienta, produce un dato equivocado que alguien se
cree. En interfaz y cableado, la prueba acompaña al código pero no tiene que precederlo.

#### Deuda de test

Solo puede existir **dentro del lote activo**. No se acumulan lotes sin pruebas, no se cierra una
historia con pruebas obligatorias pendientes, y una pantalla no está terminada porque funcione al
mirarla.

---

### 6. Unit testing

Framework: `vitest` (ya en uso). Ejecuta en Node.

Cubre: validadores, transformaciones, parsers, formateadores, cálculos, reglas de negocio, mapeadores,
serialización, límites, máquinas de estado, normalización, paginación y fechas.

**Reglas duras derivadas del principio I:**

- Todo formateador tiene prueba de **dato ausente**: `null`, `undefined` y `NaN` producen
  «No disponible», nunca cero.
- Todo cálculo tiene prueba de **cero real**, para demostrar que `0` no se confunde con ausente.
- Toda regla con umbral tiene prueba **en el umbral, justo por encima y justo por debajo**.

**Prohibido en un unit test:** red, hora real, aleatoriedad sin semilla, esperas reales, dependencia
del orden de ejecución, acceso a `%ProgramData%`.

---

### 7. Component testing

**Vitest Browser Mode con proveedor Playwright, en Chromium** (ADR-027). `@testing-library/svelte`
retirado: no se mantienen dos soluciones equivalentes.

El sufijo es **`*.browser.test.ts`**, no `*.svelte.test.ts` como decía la primera versión de este
documento: ese ya estaba tomado por las pruebas de los módulos `.svelte.ts` con runas
(`theme.svelte.test.ts`), que corren en Node. El discriminante real es el entorno.

Se prueba un componente de forma aislada cuando sea reutilizable, tenga varios estados, contenga
comportamiento, o su contrato accesible importe.

#### Qué se comprueba en cada componente del catálogo

Además de lo obvio, la definición de terminado de `AGENTS.md` §8 exige estados que hay que probar:

| Estado | Por qué |
|---|---|
| Vacío | Distinto de «error»: no tener discos no es un fallo |
| Cargando | No debe mostrar ceros mientras llega el dato |
| No compatible | **En gris, nunca en rojo**: un USB sin SMART no está averiado |
| Error de fuente | Degrada su tarjeta, no la aplicación |
| Dato obsoleto | Se marca como tal, no se muestra como fresco |

#### Lo que solo se puede probar en navegador real

Y que justifica el modo navegador frente a jsdom:

- **Contraste efectivo** con `getComputedStyle`, sobre el material compuesto y en ambos temas.
- **Que el material cae a `--sdm-solid`** cuando no hay `backdrop-filter`.
- **Que el tema oscuro resuelve todas las variables**: una variable definida solo dentro de un
  bloque de media queda sin valor y el fallo es invisible en jsdom.
- **Que las animaciones desaparecen** con `prefers-reduced-motion`.
- **Que el foco es visible** y que ningún componente anula `:focus-visible`.
- **Que el texto no se recorta** a 1024 × 560, el mínimo técnico.

No se reproduce por interfaz la lógica ya cubierta por unit tests: `capacityState()` se prueba una
vez en Node, no otra vez pintando una barra.

---

### 8. Integration testing

Frontera IPC, que es la única real:

| Qué | Cómo | Riesgo que cubre |
|---|---|---|
| Esquema Zod ↔ serialización de Rust | Un caso de prueba compartido: Rust serializa, TypeScript valida | Que el contrato divergiera en silencio |
| Store ↔ eventos | Emitir una secuencia de eventos y comprobar el estado resultante | Duplicados, orden, reemplazo por identificador |
| Colector ↔ `smartctl` falso | Ejecutable de prueba que devuelve fixtures | Timeouts, códigos de salida con bits, JSON corrupto |
| Repositorios ↔ SQLite | Fichero temporal por prueba, nunca `%ProgramData%` | Migraciones, restricciones, transacciones |
| Detección de codificación | Volcados reales de `chkdsk` (CP1252) y `fsutil` (CP850) | Que una regresión devuelva texto corrupto |

**Nunca se conecta una prueba a la base de datos real ni a un disco real en modo escritura.**

---

### 9. Acceptance testing y trazabilidad

Cada criterio de aceptación se demuestra **al nivel más barato que lo demuestre**. La tabla siguiente
es la trazabilidad inicial; se amplía al implementar cada historia.

| Historia | Criterio | Nivel | Suite | Bloquea |
|---|---|---|---|---|
| US-001 | La elevación se rechaza y la app no queda a medias | E2E app real | `app` | Release |
| US-004 | Cero literales de color; ambos temas correctos | Estático + componente | `verify` + `component` | Lote |
| US-004 | Contraste AA con acento heredado | Unit + componente | `unit` + `component` | Lote |
| US-004 | 1024 × 560 sin recortes | Componente + visual | `component` + `visual` | Historia |
| US-010 | Un RAID o USB sin SMART aparece no compatible, **no averiado** | Unit + componente | `unit` + `component` | Lote |
| US-011 | La selección no se pierde al cambiar una letra de unidad | Integración | `integration` | Historia |
| US-012 | Los valores ausentes muestran «No disponible», nunca cero | Unit | `unit` | Lote |
| US-013 | No inicia dos recopilaciones iguales | Integración | `integration` | Historia |
| US-020 | Las discontinuidades se muestran como ausencia, no como cero | Componente | `component` | Lote |
| US-021 | Reiniciar no duplica eventos | Integración | `integration` | Historia |
| US-030 | Eventos equivalentes incrementan contador, no crean grupos | Unit (dominio) | `unit-rust` | Lote |
| US-031 | Reconocer **no** devuelve el disco a verde | Unit | `unit` | Lote |
| US-032 | Un crítico vigente manda sobre la pausa | Unit | `unit` | Lote |
| US-033 | Solo se notifica al cruzar umbral o recaer | Unit (dominio) | `unit-rust` | Lote |
| US-040 | Nunca sobrescribe un archivo existente | Unit (rutas) | `unit-rust` | **Release** |
| US-040 | Se detiene en el umbral térmico | Integración | `integration` | Release |
| US-050 | La exportación informa de campos omitidos | Integración | `integration` | Historia |
| US-051 | Anonimiza series, equipo y rutas | Unit + integración | `unit` | **Release** |
| US-060 | Desinstalar conserva `ProgramData` | Manual documentado | — | Release |
| US-074 | Una prueba interrumpida no queda «en curso» | Integración | `integration` | Historia |

**No se convierte cada frase en un test independiente.** Los comportamientos relacionados se agrupan.

Formato Given/When/Then admitido en la redacción, **sin introducir Cucumber**: añadiría una capa de
traducción que aquí no compensa.

---

### 10. E2E: plano de interfaz (Playwright + `mockIPC`)

Playwright contra `pnpm preview`, con el IPC de Tauri simulado mediante `@tauri-apps/api/mocks`
(`mockIPC`, `mockWindows`, `clearMocks`), que ya está disponible en la versión instalada.

#### Qué cubre

Navegación entre secciones, estados de pantalla, formularios, mensajes de error, responsive,
accesibilidad y regresión visual. Todo lo que dependa de la interfaz y no del binario real.

#### Smoke mínimo

- La aplicación arranca y no queda en blanco.
- El chrome aparece: barra lateral, barra de herramientas, región de contenido.
- Se navega entre las seis secciones y cada una pinta algo.
- El tema se aplica: `data-theme` presente y el material resuelto.
- La tipografía empotrada carga (no ha caído al respaldo del sistema).
- No hay excepciones de JavaScript sin controlar ni errores de página.

#### Errores de consola

Se vigilan `console.error`, excepciones de página y promesas rechazadas. **No todo mensaje es un
fallo**: se distingue error de aplicación de aviso conocido. La lista de excepciones permitidas debe
ser mínima y estar documentada junto a la prueba, nunca dispersa.

#### Entorno

Desarrollo local puede usar `pnpm dev` por rapidez. **CI y release usan `pnpm preview`** sobre el
build real: es donde aparecen los problemas que el servidor de desarrollo esconde.

---

### 11. E2E: plano de aplicación real (`tauri-driver`)

WebdriverIO con `tauri-driver` contra el ejecutable empaquetado.

#### Efecto colateral sobre `cargo test`

Al aplicar el manifiesto, `cargo test` empezó a fallar con `ERROR_SXS_CANT_GEN_ACTCTX`. La causa:
Cargo compila también un arnés de pruebas para el binario, ese ejecutable hereda el
`requireAdministrator` y no puede arrancarse desde un terminal sin elevar.

Se resuelve declarando `test = false` en el target `[[bin]]`: `main.rs` es una línea que delega en
la biblioteca, no tiene tests y no los va a tener. Todo lo testeable vive en la biblioteca, que sí
se prueba y no lleva manifiesto.

Queda escrito porque es de esas cosas que se olvidan y cuestan media hora la segunda vez.

#### Requiere terminal elevado

La aplicación pide privilegios de administrador (ADR-004, manifiesto en `src-tauri/windows/`). Un
programa **no puede pulsar el botón de UAC**, así que el driver tiene que ir ya elevado para que el
hijo herede la elevación y el diálogo no aparezca.

Consecuencia práctica: **esta suite, y solo esta, se ejecuta desde un terminal de administrador**.
Las demás no lo necesitan. En los agentes Windows de GitHub el usuario ya es administrador, así que
allí funciona sin nada especial.

Se descartó compilar una variante sin elevación para las pruebas: probaría un binario distinto del
que se entrega, y precisamente en la parte que gobierna el acceso a los discos.

#### El driver se sincroniza solo

`msedgedriver` debe coincidir con la versión del WebView2 instalado, y ese runtime **se actualiza
solo cada pocas semanas** (es el precio de haber elegido Evergreen en el ADR-020). Sin nada que lo
gestione, la suite fallaría periódicamente con un error de protocolo que no tiene relación con el
código.

`scripts/ensure-webdriver.mjs` lo resuelve antes de cada ejecución: lee la versión del registro
—igual que ya hace `verify-assets` con los hashes—, descarga el driver que corresponde y lo cachea.
Si no existe un driver para esa versión exacta, **avisa con un mensaje claro** en vez de dejar que
la suite falle treinta segundos después con un error críptico.

#### Qué demuestra, y solo esto

- El binario arranca y abre su ventana.
- La interfaz pinta dentro del WebView2 real.
- El IPC responde: un comando devuelve datos de verdad.
- La ventana respeta su tamaño mínimo.
- El registro escribe en `%ProgramData%`.
- La aplicación cierra limpiamente y suelta sus recursos.

Es una suite **deliberadamente pequeña**. No se replican aquí flujos que el plano de interfaz ya
cubre más rápido y con mejor diagnóstico.

#### Frecuencia

Rama principal y antes de release. **Nunca en cada pull request**: cuesta minutos y requiere una
compilación completa.

---

### 12. Contrato de testabilidad y selectores

Orden de preferencia, en componentes y en E2E:

1. Rol accesible (`getByRole`)
2. Nombre accesible / etiqueta (`getByLabel`)
3. Texto visible estable (`getByText`)
4. Estado semántico (`aria-current`, `aria-disabled`)
5. `data-testid` **solo si lo anterior no basta**

**Prohibido**: clases CSS, clases de Tailwind, posiciones, `nth()`, estructura interna del DOM,
identificadores generados.

Esto no es una preferencia de estilo: si un elemento no se puede localizar por su rol o su nombre
accesible, es que **no es accesible**, y eso incumple el principio VII antes que ninguna prueba.

Identificadores estables admitidos, ligados al dominio:

```text
app-root  main-layout  primary-action  disk-card
empty-state  error-state  loading-state  stale-badge
```

---

### 13. Esperas y sincronización

Se usa la sincronización automática de Playwright: locators, `expect` con reintento, esperas por
estado observable.

**Prohibido**: `waitForTimeout`, `sleep`, esperas arbitrarias, bucles manuales. Subir un tiempo de
espera para tapar una condición de carrera no la arregla, la esconde.

Si una prueba necesita cada vez más tiempo, es un defecto: se investiga la causa.

---

### 14. Datos, dobles y aislamiento

Cada prueba debe poder ejecutarse sola, en cualquier orden, repetidamente y en paralelo.

| Dependencia | Doble | Por qué |
|---|---|---|
| `smartctl` | Ejecutable falso que devuelve fixtures | Probar timeouts, códigos con bits y JSON corrupto sin hardware |
| Registro de eventos de Windows | Volcados XML reales anonimizados | Reproducir ráfagas correlacionadas (`alert-rules.md` §3.5) |
| SQLite | Fichero temporal por prueba | Nunca `%ProgramData%` |
| Reloj | Reloj inyectado / *fake timers* | Histéresis, retención y expiraciones |
| Acento de Windows | Valor fijo | El acento del equipo de CI no puede decidir si una prueba pasa |
| IPC de Tauri | `mockIPC` | Estados de error y casos límite imposibles de provocar de verdad |

**Los fixtures se anonimizan al capturarlos, nunca al usarlos.** Ninguno contiene números de serie
ni nombres de equipo reales (principio IX).

Se declara explícitamente en cada suite si usa IPC simulado, backend real o base de datos temporal.
No se mezclan estrategias dentro de un mismo fichero.

---

### 15. Responsive

Tres viewports, ni uno más, tomados de la medición de `open-questions.md` §L:

| Viewport | Por qué ese |
|---|---|
| 1024 × 560 | Mínimo técnico. Barra lateral colapsada |
| 1280 × 720 | Objetivo de diseño |
| 1920 × 1032 | 1920 × 1080 al 100 %, el caso más común |

Se comprueba: navegación utilizable, acciones alcanzables, sin recortes silenciosos, sin scroll
horizontal en el cuerpo, y que la rejilla reorganiza en vez de comprimir.

No se prueban decenas de resoluciones arbitrarias: no hay más breakpoints funcionales que esos.

---

### 16. Accesibilidad

Automatizable, y se automatiza: `axe-core` sobre cada pantalla, en ambos temas.

Cubre roles, nombres accesibles, asociación de etiquetas, estados de los controles y contraste.

**Comprobaciones propias que `axe` no hace** y que este proyecto sí exige:

- El contraste se mide sobre el **material compuesto**, no sobre un fondo plano.
- El acento heredado se verifica con un acento claro deliberado (`#ffb900`).
- El foco no queda tapado por el chrome translúcido (WCAG 2.4.11).
- Toda gráfica tiene lectura textual equivalente cerca, no solo `aria-label`.

Una prueba automática **no es una auditoría**. Antes de la 1.0 se hace una pasada manual con lector
de pantalla; queda como tarea, no como comprobación automatizable.

---

### 17. Regresión visual

Doce capturas, solo en Chromium. Comparar entre motores daría falsos positivos por renderizado de
fuentes sin cubrir ningún riesgo real: solo se distribuye WebView2.

| Captura | Temas |
|---|---|
| Panel general con cuatro discos | claro, oscuro |
| Detalle de disco | claro, oscuro |
| Alertas con grupo seleccionado | claro, oscuro |
| Pruebas y diagnóstico | claro, oscuro |
| Estado vacío | claro |
| Estado de error de fuente | claro |
| Disco no compatible (gris, no rojo) | claro |
| Dato obsoleto | claro |

**Se controla antes de capturar**: datos fijos, reloj congelado, locale, viewport, tipografía local,
`prefers-reduced-motion` activo y animaciones desactivadas. Se enmascaran únicamente las regiones
legítimamente dinámicas, como la marca de antigüedad.

Una captura **no sustituye una prueba funcional** y no se actualiza porque falle: se actualiza cuando
el cambio es intencionado, se ha revisado y va en el mismo commit que lo provoca.

#### La línea base se genera en CI

La misma pantalla no se dibuja igual en dos equipos: el suavizado de fuentes depende de la
configuración de ClearType, y el desenfoque del material translúcido lo calcula la tarjeta gráfica
—que los agentes de CI no tienen, así que Chromium lo hace por software—. Justo el material, que es
lo que más interesa vigilar, es lo que peor se reproduce entre máquinas.

Por eso **la comparación que decide es la de CI**:

| Dónde | Qué hace |
|---|---|
| Local | Genera las capturas para poder mirarlas. **No compara** |
| CI | Compara contra la línea base versionada en el repositorio |

Aprobar un cambio visual intencionado: se sube, CI falla y adjunta la captura nueva como artefacto,
se revisa, y la línea base actualizada va **en el mismo commit** que el cambio que la provoca.

Se descartó generar la base en local con tolerancia amplia: la holgura necesaria para absorber la
diferencia de desenfoque dejaría pasar regresiones reales, y una prueba así da tranquilidad sin dar
cobertura.

Frecuencia: rama principal. Es donde una regresión de tokens se detecta antes de llegar a release.

---

### 18. Property-based testing

Se aplica donde hay propiedades generales que valen para todo el dominio de entrada, no como
sustituto de casos concretos.

| Candidato | Propiedad |
|---|---|
| `accessibleAccent` | Para **todo** color, el resultado alcanza 4.5:1 |
| `accentOnSurface` | Para todo color y ambos temas, alcanza 4.5:1 |
| `formatBytes` | Monótona: más bytes nunca produce una unidad menor |
| `capacityState` | Monótona: menos espacio libre nunca mejora el estado |
| Deduplicación de alertas | N evaluaciones equivalentes producen 1 grupo y N ocurrencias |
| Detección de codificación | Todo volcado válido decodifica sin excepción |

Semillas reproducibles y registro del caso mínimo que falle. **No se aplica a componentes visuales.**

Nota: `accessibleAccent` ya tiene hoy un barrido exhaustivo escrito a mano; sustituirlo por
property-based es opcional y de bajo valor añadido.

---

### 19. Mutation testing

Mide si los tests detectan errores, no si el código funciona. **Fuera del bucle de desarrollo.**

| Ámbito | Herramienta | Qué se muta |
|---|---|---|
| TypeScript | Stryker | `design/health.ts`, `design/format.ts`, `design/accent.ts` |
| Rust | `cargo-mutants` | `domain/`, `alerts/`, parsers de `smartctl` |

**Se excluye**: componentes visuales, CSS, configuración, envoltorios finos, registro, código
generado, y todo `src/routes/`.

Prioridad en el motor de alertas: si una regla tiene un test que pasa con la lógica invertida, el
producto **miente sobre la salud de un disco**, que es el fallo más caro que puede tener.

Ejecución: programada semanal y antes de release. **Nunca en un pull request**, nunca sobre suites
inestables, nunca durante una refactorización.

#### El informe tiene que ser accionable, no exhaustivo

Un motor de alertas de tamaño medio produce del orden de 400 a 600 mutantes, y cada uno obliga a
recompilar: entre dos y tres horas de ejecución. El tiempo no es el problema —se ejecuta de
madrugada y nadie espera—; el problema es que un informe con sesenta supervivientes **no lo revisa
nadie**, y a la tercera semana deja de abrirse. Una herramienta que nadie mira no cubre ningún
riesgo: solo da la sensación de que sí.

Por eso se versiona una línea base en `docs/mutation-baseline.md` con los supervivientes conocidos y
la razón por la que se aceptan: mutación equivalente, comportamiento no observable, código muerto.
El informe semanal destaca únicamente **los supervivientes nuevos**.

| | |
|---|---|
| Informe útil | «3 supervivientes nuevos» — se revisan los tres |
| Informe inútil | «60 supervivientes» — no se revisa ninguno |

**Un superviviente nuevo sin justificar bloquea la release.** Uno ya justificado en la línea base, no.

Se empieza con un piloto sobre `health.ts` para fijar la primera línea base (mutantes, muertos,
supervivientes, sin cobertura, timeouts, duración). **No se fija el 100 % como objetivo**: un
superviviente puede ser una mutación equivalente, no un hueco.

---

### 20. Cobertura

Umbrales del principio VIII, ya configurados en `vitest.config.ts` y en CI:

| Ámbito | Mínimo |
|---|---|
| `domain/`, `alerts/` (Rust) | 90 % |
| Resto de `src-tauri/` | 80 % |
| `src/lib/` sin componentes | 70 % |
| Componentes y rutas | Sin umbral: se exige **cobertura de estados** |

Atención especial a las ramas de validación, error, límites y reglas.

**La cobertura es una señal, no un objetivo.** No se escriben pruebas sin valor para subirla, y una
cobertura alta no sustituye a las pruebas de aceptación, al E2E ni al mutation testing.

---

### 21. Determinismo

Prohibido depender de `Date.now()`, `new Date()`, `Math.random()`, temporizadores reales o del
entorno del equipo cuando afecte al resultado.

Se prueban explícitamente: cambio de día, expiraciones, ventanas de histéresis y cambios de hora,
que en este producto **importan de verdad** — la retención, el cooldown de notificaciones y la
correlación de eventos dependen de ello.

---

### 22. Suites escalonadas

#### Nivel 0 — Edición

Comprobación de tipos del fichero tocado. Una prueba concreta si se está diagnosticando. Nada más.

#### Nivel 1 — Ultrarrápida (segundos)

```sh
pnpm test -- <fichero>
cargo test <modulo>
```

#### Nivel 2 — Lote (cierre de checkpoint, < 1 min)

```sh
pnpm check && pnpm lint && pnpm verify
pnpm test
cargo test
```

Más los tests de componente del lote si toca interfaz. **Es la suite del checkpoint.**

#### Nivel 3 — Historia / Pull Request (minutos)

Todo lo anterior más: componentes completos, integración, aceptación, E2E de interfaz, accesibilidad
y cobertura con umbrales.

#### Nivel 4 — Rama principal

Añade: regresión visual, E2E de aplicación real, cobertura de Rust.

#### Nivel 5 — Programada y release

Añade: mutation testing, property-based extendido, verificación del instalador y la lista manual de
accesibilidad.

**Nunca se ejecuta el nivel 4 o 5 tras un cambio pequeño.**

---

### 23. Selección de pruebas afectadas

Se ejecuta primero lo más cercano al cambio. Se **amplía a la suite completa** cuando se toca:

`tokens.css` · `tailwind.config.cjs` · `AppShell`, `Sidebar` o `Toolbar` · `src/lib/api/` ·
`src/lib/design/health.ts` · `schemas.ts` · el motor de alertas · migraciones ·
`svelte.config.js`, `vite.config.ts` o `tauri.conf.json` · cualquier dependencia principal.

Son los puntos donde un cambio local tiene efecto global.

---

### 24. Estructura y comandos

Estructura propuesta, respetando la que ya existe:

Dos configuraciones de Vitest, no una: los entornos son distintos y los tiempos también.
`pnpm test` debe seguir siendo la suite rápida que se teclea mientras se programa.

| Configuración | Entorno | Qué incluye | Duración medida |
|---|---|---|---|
| `vitest.config.ts` | jsdom | `src/**/*.test.ts`, excluidas las de navegador | ~16 s |
| `vitest.browser.config.ts` | Chromium | `src/**/*.browser.test.ts` | ~2 s |

Stryker apunta a la de Node y por tanto nunca abre un navegador, que era el riesgo real: mutar
código exige lanzar la suite cientos de veces.

```text
src/**/*.test.ts             unit, junto al código                        EXISTE
src/**/*.browser.test.ts     componentes, en navegador                    EXISTE
src/**/*.svelte.test.ts      módulos `.svelte.ts` con runas, en Node      EXISTE
tests/helpers/               ayudantes compartidos (reloj congelado)      EXISTE
tests/setup-browser.ts       preparación del Browser Mode                 EXISTE
tests/integration/           frontera IPC, SQLite, colectores             pendiente
tests/fixtures/              volcados anonimizados de smartctl y chkdsk   pendiente
e2e/ui/smoke.spec.ts         smoke de interfaz                            EXISTE
e2e/ui/a11y.spec.ts          accesibilidad con axe                        EXISTE
e2e/ui/ipc-falso.ts          doble del IPC de Tauri                       EXISTE
e2e/ui/fixtures/             respuestas validadas contra los esquemas     EXISTE
e2e/ui/visual.spec.ts        regresión visual                             pendiente
e2e/app/                     WebdriverIO + tauri-driver                   pendiente
src-tauri/src/**             tests en módulo `#[cfg(test)]`               EXISTE
src-tauri/tests/             integración de Rust                          pendiente
```

**Comandos que existen de verdad.** Los que no están aquí, no existen: no se prometen scripts que no
se hayan ejecutado.

| Comando | Qué corre |
|---|---|
| `pnpm test` · `pnpm test:unit` | Lógica en Node. Es la suite que se teclea mientras se programa |
| `pnpm test:watch` | La misma, en observación |
| `pnpm test:coverage` | La misma, con umbrales del principio VIII |
| `pnpm test:component` | Componentes en Chromium real |
| `pnpm test:component:watch` | La misma, en observación |
| `pnpm test:e2e` | Todo el plano de interfaz |
| `pnpm test:e2e:smoke` | Solo lo etiquetado `@smoke` |
| `pnpm test:e2e:ui` | Modo interactivo de Playwright |
| `pnpm test:a11y` | Solo lo etiquetado `@a11y` |
| `cargo test` | Rust, desde `src-tauri/` |

Pendientes de existir, cuando exista lo que prueban: `test:integration`, `test:e2e:visual`,
`test:e2e:app`, `test:mutation`, `test:lote`.

---

### 25. CI

Ampliación del workflow actual, que ya tiene trece puertas:

| Disparador | Añade |
|---|---|
| Pull request | Componentes, integración, aceptación, E2E de interfaz, accesibilidad |
| Rama principal | Regresión visual, E2E de aplicación real, cobertura de Rust |
| Programado semanal | Mutation testing, property-based extendido |
| Release | Verificación del instalador y del artefacto que se distribuye |

Artefactos de diagnóstico: captura y traza **de los fallos**, en CI. Vídeo solo donde aporte. No se
guardan artefactos pesados de pruebas correctas.

---

### 26. Pruebas inestables

Una prueba inestable es un **defecto**, no una molestia. Se investiga: estado compartido, condición
de carrera, datos compartidos, orden, temporizadores, animaciones.

No se arregla subiendo reintentos ni tiempos de espera. Un reintento vale como mitigación temporal
**documentada**, nunca como solución. Ninguna prueba crítica se ignora de forma permanente sin
justificación en `docs/known-issues.md`.

---

### 27. Integración con Spec Kit

**Especificación** (`spec.md`): cada historia aporta reglas verificables, criterios de aceptación,
camino correcto, errores, límites, estados y riesgos.

**Plan** (`plan.md`): decide niveles de test, dobles, datos, suites, checkpoints y presupuesto.

**Tareas** (`tasks.md`): agrupadas por lote. **No** se genera una tarea por test ni la secuencia
«implementar función → crear test → ejecutar todo». El testing va dentro del checkpoint del lote.

---

### 28. Definition of Checkpoint

Un lote cierra cuando:

- la implementación prevista está completa;
- `pnpm check`, `pnpm lint` y `pnpm verify` pasan;
- las pruebas obligatorias del lote existen y pasan;
- la aceptación relacionada pasa;
- los fixtures y dobles necesarios existen;
- no queda deuda de test del lote.

**No hace falta** ejecutar en cada checkpoint: E2E completo, visual, aplicación real, mutation
testing ni suites no afectadas.

### 29. Definition of Done

Una historia termina cuando, además de sus lotes:

- los criterios de aceptación son trazables a pruebas concretas;
- errores, límites y estados relevantes están cubiertos;
- los estados vacío, cargando, no compatible, error y obsoleto están probados;
- la suite de historia pasa, incluido el E2E de interfaz cuando aplique;
- los tres viewports están verificados;
- la accesibilidad automática pasa en ambos temas;
- las capturas visuales coinciden o se actualizaron intencionadamente;
- no hay esperas arbitrarias ni pruebas ignoradas sin justificar;
- la cobertura no baja sin razón escrita;
- los comandos están documentados y las pruebas integradas en CI.

**Mutation testing solo es obligatorio** si la historia toca el dominio crítico y el plan lo pide.

---

### 30. Tareas para `tasks.md`

Se adaptan; no se crean todas de golpe.

Estado a 2026-09-04. **Hecho** son tareas ejecutadas y verificadas; el resto indica qué las
desbloquea, porque ninguna se puede hacer hoy sin eso.

```text
HECHO
T-TEST-001  Comandos reales documentados y línea base medida de cada suite     §3
T-UNIT-003  Reloj congelable y zona horaria fija de la suite                   §21
T-COMP-001  Vitest Browser Mode configurado; testing-library retirado          ADR-027
T-PLAY-001  Playwright contra preview, con IPC propio                          ADR-028
T-PLAY-003  Smoke de interfaz: arranque, chrome, navegación, tema, fuente
T-PLAY-005  Errores de consola vigilados, con lista de excepciones junto a la prueba
T-A11Y-001  axe-core en las seis pantallas y los dos temas

PENDIENTE — esperan a que exista el producto que prueban
T-COMP-002  Los cinco estados obligatorios del catálogo        con cada componente
T-COMP-003  Contraste y material con getComputedStyle          con cada componente
T-UNIT-001  Fixtures de smartctl (ATA, NVMe, USB, RAID, VM)    con el colector, US-010
T-UNIT-002  Fixtures de eventos y volcados de chkdsk/fsutil    con el lector, US-021
T-INT-001   smartctl falso: timeouts, bits del código, JSON corrupto
T-INT-002   SQLite temporal: migraciones, restricciones, retención
T-INT-003   Contrato Zod ↔ serde con caso compartido           con el primer DTO real
T-ACC-001   Trazabilidad criterio → prueba de las historias P0
T-PLAY-006  E2E de los flujos críticos
T-A11Y-002  Contraste sobre material y foco no tapado          con las pantallas reales
T-QUAL-002  Detección de pruebas inestables                    cuando haya suite que oscile

PENDIENTE — necesitan la aplicación empaquetada
T-PLAY-002  msedgedriver y tauri-driver                        US-060
T-PLAY-004  Smoke de aplicación real                           US-060
T-VIS-001   Doce capturas controladas en Chromium              pantallas definitivas
T-QUAL-001  Cobertura de Rust en CI                            con dominio en Rust
T-MUT-001   Piloto de Stryker sobre health.ts y línea base
T-MUT-002   cargo-mutants sobre el dominio                     cuando exista el dominio

DESCARTADA
T-TEST-002  Selección de pruebas afectadas por tipo de cambio. La suite completa tarda
            menos de un minuto y medio: un selector costaría más de mantener de lo que
            ahorra, y se equivocaría en silencio. Se reconsidera si pasa de cinco minutos.
```

---

### 31. Preguntas abiertas

Las cinco que planteaba la primera versión de este documento están **resueltas** (2026-09-04):

| # | Era | Resolución |
|---|---|---|
| 1 | ¿Qué versión de `msedgedriver` hace falta? | Ninguna fija: `scripts/ensure-webdriver.mjs` la deduce del registro y la descarga (§11) |
| 2 | ¿Puede `tauri-driver` con una app elevada? | Sí, si el driver va elevado. Esa suite se ejecuta desde terminal de administrador (§11) |
| 3 | ¿Stryker convive con Browser Mode? | No hace falta: dos configuraciones separadas, Stryker usa la de Node (§24) |
| 4 | ¿CI dibuja igual que un equipo local? | No. La línea base se genera y compara en CI; en local solo se miran (§17) |
| 5 | ¿Cuánto tarda `cargo-mutants`? | Horas, y da igual: lo que se acota es el informe, no el tiempo (§19) |

De paso, la número 2 destapó que **la elevación del ADR-004 no estaba implementada**. Se corrigió con
un manifiesto propio en `src-tauri/windows/app.manifest`, que además declara consciencia de DPI por
monitor, necesaria para los escalados de 125 %, 150 % y 200 % que exige `AGENTS.md` §4.

#### Lo que sigue sin respuesta

| # | Pregunta | Cuándo se sabrá |
|---|---|---|
| ~~1~~ | ~~¿Cuánto tarda de verdad la suite de componentes en navegador?~~ **Respondida**: ~2 s con 8 pruebas, y el 70 % es el arranque del navegador, que se paga una vez. Cabe de sobra en el ciclo de trabajo. Se revisa si pasa de 30 s |
| 2 | ¿Las doce capturas visuales son estables entre ejecuciones del mismo agente de CI? | Al implementar `T-VIS-001` |

Ninguna bloquea. Se miden cuando toque.


---

# 11. Registro de decisiones técnicas

Fichero de origen: `docs/decisions.md`

### ADR-001 — Tauri 2 para escritorio

Estado: aceptada.

Se utilizará Tauri 2 por su integración nativa, tamaño razonable, backend Rust, systray y capacidad de empaquetar un ejecutable auxiliar. La primera plataforma es Windows x64.

### ADR-002 — Svelte en lugar de React

Estado: aceptada.

La interfaz usará Svelte + TypeScript. React no se incluirá. Utilizar simultáneamente ambos frameworks duplicaría responsabilidades sin una necesidad funcional.

Tailwind CSS proporcionará utilidades de estilo sobre los tokens aprobados. Se utilizará Svelte 5 con runes y no se usarán bibliotecas externas de componentes.

### ADR-003 — Aplicación local sin servicio

Estado: aceptada.

No existirá agente, servidor central ni servicio de Windows. La recopilación ocurre solo con sesión iniciada y aplicación activa, aunque la ventana esté en la bandeja.

### ADR-004 — Elevación de todo el proceso

Estado: aceptada.

El ejecutable solicitará `requireAdministrator`. Simplifica el acceso a dispositivos y eventos, aceptando que aparezca UAC en cada inicio y que toda la UI viva dentro de un proceso elevado.

Mitigación: capacidades Tauri mínimas, sin shell genérica desde JavaScript y comandos privilegiados cerrados en Rust.

**Implementación** (2026-09-04): manifiesto propio en `src-tauri/windows/app.manifest`, aplicado
desde `build.rs`. Sin él, Tauri genera uno por defecto con `asInvoker` y la aplicación **no pediría
elevación**, que es como estuvo hasta que lo destapó el diseño de la estrategia de pruebas.

El manifiesto declara además `PerMonitorV2`: sin esa marca, Windows escala la ventana por su cuenta
y el resultado se ve borroso al 125 %, 150 % y 200 %, justo los escalados que exige verificar
`AGENTS.md` §4.

Consecuencia para el desarrollo: `pnpm app:dev` muestra UAC en cada arranque, y la suite E2E de
aplicación real necesita un terminal elevado (`docs/testing-strategy.md` §11).

### ADR-005 — smartctl como auxiliar independiente

Estado: aceptada.

`smartctl` será la fuente principal de SMART/NVMe y se ejecutará como binario independiente con salida JSON. Se incluirá en el instalador manteniendo avisos, licencia y obligaciones aplicables de smartmontools. El código propio conserva licencia MIT.

La API nativa de Windows será complementaria y permitirá contrastar topología, volúmenes, actividad y datos de fiabilidad disponibles.

### ADR-006 — SQLite local

Estado: aceptada.

SQLite almacenará configuración, inventario, muestras, eventos, alertas y pruebas en `ProgramData`. Se utilizarán WAL, migraciones versionadas, transacciones breves y copias previas a migraciones.

### ADR-007 — Sin red ni telemetría

Estado: aceptada.

El funcionamiento normal no necesita red. No se recopila ni transmite telemetría. Las actualizaciones son totalmente manuales.

### ADR-008 — Identidad por dispositivo físico

Estado: aceptada.

La clave de presentación no será la letra de unidad ni el número de disco mutable. Se usará el número de serie y, cuando falte, una huella calculada con atributos estables, conservando el grado de confianza.

### ADR-009 — Benchmarks basados en archivos

Estado: aceptada.

Las pruebas de escritura no acceden a bloques sin formato. Utilizan un archivo temporal nuevo, limitado y verificable dentro de un volumen montado. Se reservan espacio y umbral térmico, se permite cancelación y se intenta una limpieza segura.

### ADR-010 — Alertas por estado y cambio

Estado: aceptada.

El motor combina límites absolutos con incrementos de contadores y persistencia temporal. Las alertas se agrupan para reducir ruido, pero cada ocurrencia conserva su hora.

### ADR-011 — Versionado como fuente única

Estado: aceptada.

Nombre y versión proceden de los manifiestos del proyecto durante compilación y ejecución. La UI, “Acerca de”, informes e instalador no mantendrán copias manuales independientes.

### ADR-012 — Datos persistentes tras desinstalar

Estado: aceptada.

El desinstalador conserva SQLite, configuración, historial y logs en `ProgramData`. La aplicación ofrece una acción separada y confirmada para eliminarlos, y la documentación explica la limpieza manual.

### ADR-013 — Sistema de diseño v2 vinculante

Estado: aceptada; **enmendada por ADR-034** (2026-09-06): el sistema de diseño evoluciona a v3
(paleta propia «Ciruela», dos escalones tipográficos de «display», riel de navegación). El fondo de
ADR-013 no cambia: `tokens.css` sigue siendo la fuente única de verdad, el catálogo sigue cerrado,
el material de tres capas y los radios concéntricos no se tocan. Lo que cambia es la paleta y que la
herencia del acento de Windows pasa a opción apagada de fábrica (ADR-035).

La interfaz utilizará el paquete de diseño entregado, versión v2 de material translúcido, con su norma vinculante y `tokens.css` como fuente única de verdad visual.

> **Rutas actualizadas por ADR-029.** Cuando se escribió esta decisión, la norma era `Design-system/AGENTS.md` y el paquete vivía sin integrar. Hoy la norma es `docs/ui-design.md` y el sistema de diseño vive en `src/`. El fondo de la decisión no cambia.

Se adoptan el catálogo cerrado de componentes Svelte, los tipos y formateadores entregados, los diccionarios español/inglés, la herencia del acento de Windows y los bocetos v2 aprobados. No se introducirán valores visuales literales, niveles adicionales de material ni bibliotecas de componentes sin una decisión nueva.

Las pantallas todavía no diseñadas —Informes, Ajustes, asistente inicial, Acerca de y systray— se compondrán inicialmente con el catálogo existente y requerirán revisión visual antes de considerarse terminadas.

### ADR-014 — SvelteKit con `adapter-static` y SSR desactivado

Estado: aceptada.

El frontend se monta sobre SvelteKit compilado a estático, sin renderizado en servidor. Es la vía
que Tauri documenta oficialmente y la que el paquete de diseño ya asumía (`$lib` de serie). Aporta
enrutado por ficheros para las siete pantallas sin añadir dependencias de terceros, que es lo que
haría falta con Vite + Svelte puro y que `AGENTS.md` §1 prohíbe.

Consecuencias: hay que desactivar SSR explícitamente y prerrenderizar, porque no existe servidor.
Ninguna carga de datos ocurre en `load`: todo pasa por comandos Tauri desde el cliente.

### ADR-015 — El backend empuja, la interfaz no sondea

Estado: aceptada.

La comunicación de actualizaciones es por eventos Tauri tipados (`metrics:updated`,
`alerts:changed`, `inventory:changed`, `test:progress`, …), no por sondeo periódico desde la
interfaz. El planificador ya sabe cuándo hay datos nuevos; hacer que la UI pregunte cada pocos
segundos duplicaría el reloj, gastaría batería y garantizaría que lo mostrado va siempre un poco
por detrás.

Cada evento trae el objeto completo que ha cambiado, no un parche: la UI reemplaza por
identificador sin reconciliar. Al montar y tras cualquier error de deserialización, la interfaz pide
el estado completo con los comandos `get_*`.

Contrato completo en `docs/ui-contract.md`.

### ADR-016 — Reconocer una alerta no apaga su color

Estado: aceptada.

El estado de salud que se pinta en un disco es la peor severidad de sus alertas `active` **o**
`acknowledged`. Solo `resolved` y `archived` dejan de contar, y el silencio no afecta nunca al color.

La alternativa —que reconocer devolviera el disco a verde— permitiría limpiar el panel, pero a
cambio el color dejaría de ser una señal fiable del estado del hardware, que es justamente para lo
que existe. Se asume que un disco con un problema crónico se quede en rojo: se mitiga con el
distintivo de reconocida y con el orden de la lista, no apagando la señal.

Implementado en `deviceState()` y `alertCountsTowardHealth()`, un único sitio para toda la interfaz.

### ADR-017 — El acento heredado se corrige antes de aplicarse

Estado: aceptada; **matizada por ADR-035** (2026-09-06): la herencia del acento de Windows deja de
ser el comportamiento de fábrica y pasa a un interruptor de Ajustes apagado por defecto. La parte
técnica de ADR-017 se conserva entera: cuando el interruptor está encendido, `accessibleAccent()` y
`accentOnSurface()` siguen corrigiendo el color del usuario para no romper el contraste.

El acento de Windows se hereda, pero no a ciegas: `accessibleAccent()` elige texto blanco o negro
según cuál contraste mejor y, si aun así no se alcanza 4.5:1, oscurece o aclara el acento hasta
lograrlo.

Sin esto, un usuario con acento amarillo, lima o cian claro dejaría el botón primario por debajo de
AA y la aplicación incumpliría su propia norma de accesibilidad. Se prefiere alterar mínimamente el
color elegido por el usuario antes que entregar texto ilegible, y se prefiere eso a renunciar a la
herencia del acento, que es parte de la identidad visual de v2.

### ADR-018 — Tipografía empotrada, no descargada

Estado: aceptada.

Instrument Sans se distribuye como fichero variable dentro de la aplicación y se declara con
`@font-face` local. La versión original de `tokens.css` la importaba de Google Fonts, lo que
contradecía la promesa de cero comunicaciones de red (spec §11, ADR-007) y habría dejado la
aplicación con otra tipografía en cualquier equipo sin salida a Internet, que es precisamente el
tipo de equipo donde se instala un monitor de discos.

Obligación asociada: incluir la SIL Open Font License 1.1 y registrar la fuente en
`THIRD_PARTY_NOTICES.md`. La compilación falla si el fichero no está.

### ADR-019 — El suelo absoluto de capacidad solo se aplica a volúmenes grandes

Estado: aceptada.

Los umbrales de espacio libre son siempre porcentuales (10 % advertencia, 5 % crítico) y, además,
absolutos (20 GB / 10 GB) **solo en volúmenes de 256 GB o más**.

La regla original —el mayor entre porcentaje y valor absoluto, sin condición— marcaba como crítico
un volumen de 64 GB con 15 GB libres, que es casi una cuarta parte del disco. En equipos con
particiones de sistema pequeñas eso sería ruido permanente, y una alerta que siempre está encendida
deja de leerse.

El corte de 256 GB es configurable, porque es el único número de la regla que no se deduce de nada.

### ADR-020 — WebView2 se distribuye con el instalador sin conexión

Estado: aceptada.

`tauri.conf.json` usará `webviewInstallMode: { "type": "offlineInstaller" }`.

#### El problema

WebView2 es un requisito de ejecución que no siempre está presente. Solo Windows 11 incluye el
runtime como parte del sistema; en Windows 10 lo tiene la gran mayoría de equipos porque Microsoft
lo desplegó por Windows Update a partir de diciembre de 2022, y **en Windows Server no viene
preinstalado en ninguna versión**. Como Windows Server 2016 a 2025 están dentro del alcance, la
aplicación no arrancaría en un servidor recién instalado.

Los sistemas soportados sí son los que declara la especificación: Microsoft Edge —y con él
WebView2— soporta Windows Server 2016, 2019, 2022 y 2025 en LTSC, y Windows 10 desde 1709. La
matriz de compatibilidad no cambia; lo que cambia es el instalador.

#### Las opciones

| Modo | Tamaño añadido | ¿Internet al instalar? | ¿El runtime se parchea solo? |
|---|---|---|---|
| `downloadBootstrapper` (predeterminado) | 0 MB | **sí** | sí |
| `embedBootstrapper` | ~1,8 MB | **sí** | sí |
| `offlineInstaller` | ~127 MB | no | **sí** |
| `fixedRuntime` | ~180 MB | no | **no** |
| `skip` | 0 MB | no | la aplicación no arranca |

#### Por qué el instalador sin conexión

Los dos modos de bootstrapper quedan descartados porque exigen conexión durante la instalación, y
un servidor de almacenamiento aislado de Internet es justo uno de los escenarios para los que se
escribe este producto.

Entre las dos opciones sin conexión, la diferencia decisiva no es el tamaño sino **quién parchea el
runtime**. `fixedRuntime` congela una versión concreta de Chromium dentro de la aplicación: dejaría
de recibir parches de seguridad hasta que nosotros publicásemos una versión nueva, y como no hay
actualizador automático (ADR-007) eso significa "hasta que el usuario se entere y descargue a mano".
La aplicación renderiza texto procedente de dispositivos y del registro de eventos de Windows, así
que arrastrar un Chromium sin parchear no es aceptable. `fixedRuntime` añade además tres problemas
prácticos: no funciona desde una ruta de red o UNC, exige conceder permisos con `icacls` a los
contenedores de aplicación en Windows 10 desde la versión 120, y en disco ocupa más de 250 MB.

`offlineInstaller` instala el runtime *Evergreen*: la instalación funciona sin conexión y, a partir
de ahí, Microsoft lo mantiene actualizado por su cuenta sin que nosotros publiquemos nada.

#### Consecuencias

- El instalador pasa de unos 10 MB a unos 140 MB. Es un coste asumible en una distribución manual
  por GitHub, y se documenta en la página de descarga.
- El instalador comprueba antes si el runtime ya está presente y solo lo instala si falta, así que
  en un Windows 11 o en un Windows 10 al día no se instala nada.
- Como la aplicación se instala elevada, el runtime queda instalado *por equipo*, que es lo que
  corresponde a una instalación para todos los usuarios.
- Si el runtime faltara igualmente en tiempo de ejecución, la aplicación debe detectarlo y decirlo
  con una frase comprensible, no fallar con una ventana en blanco.

### ADR-021 — smartmontools 7.5, con el fuente dentro del instalador

Estado: aceptada.

Concreta el ADR-005, que dejaba sin fijar la versión y el mecanismo de cumplimiento de la licencia.

#### Versión

Se redistribuye **smartmontools 7.5** (publicada el 12 de mayo de 2025, compilación r5714), tomando
el `smartctl.exe` de 64 bits del paquete oficial de Windows. El paquete se llama `win32-setup` por
razones históricas pero contiene las dos arquitecturas; se usa la de `bin/`, verificada como PE
AMD64. Las sumas MD5 se comprueban contra las que publica el propio proyecto.

Se empaqueta también **`drivedb.h`**, la base de datos de unidades. No es opcional: sin ella,
`smartctl` no sabe interpretar los atributos específicos de cada fabricante y los presenta como
desconocidos, que es justo la información que hace útil a un monitor de discos. Queda congelada con
la versión: el script oficial que la actualiza descarga de Internet y no se distribuye, porque
contradiría la promesa de cero comunicaciones de red.

No se empaquetan `smartd` ni sus utilidades: la aplicación ya tiene su propio planificador, y un
segundo vigilante competiría por el acceso a los dispositivos.

#### Licencia

`smartctl` es `GPL-2.0-or-later`. **El código propio sigue siendo MIT**: se invoca como proceso
independiente, por línea de órdenes y JSON, sin enlazarlo ni incorporar su código, así que no hay
obra derivada.

La obligación que sí aplica es la de la sección 3 de la GPLv2: quien recibe el binario tiene derecho
al código fuente correspondiente. Se cumple por la vía **3(a)**, acompañar el binario del fuente:
`smartmontools-7.5.tar.gz` viaja dentro del instalador, en `licenses\smartmontools\`.

Se descarta la vía 3(b), la oferta escrita válida tres años, porque obliga a mantener el fuente
disponible y a atender solicitudes durante ese plazo. Un fichero de 1 MB dentro de un instalador de
140 MB cuesta menos y no caduca. La versión del tarball debe coincidir siempre con la del binario,
o el requisito deja de cumplirse.

#### Consecuencia operativa comprobada

`--scan-open` funciona sin elevación, pero leer datos de un dispositivo sin privilegios falla con
`exit_status: 1` y el mensaje `Unable to detect device type`. Ese mensaje **no** significa que el
dispositivo sea incompatible: tomarlo al pie de la letra marcaría un equipo entero como "no
compatible" por un problema de privilegios. El colector debe distinguir los dos casos.

### ADR-022 — Zod para validar toda frontera de datos

Estado: aceptada.

#### El problema

`invoke<DeviceListResponse>(...)` **no valida nada**. Es una aserción de tipo sobre un dato que
viene de otro proceso: TypeScript se limita a creerse lo que se le dice. Si el backend cambia un
campo, renombra otro o devuelve `null` donde antes había un número, la interfaz lo acepta y muestra
`undefined` como si fuera un dato bueno.

Eso choca de frente con el principio I: *nunca se inventa un valor*. Un `undefined` renderizado es
exactamente un valor inventado, con el agravante de que nadie se entera.

#### Por qué aquí importa más que en una aplicación web

Este producto no tiene red, así que la tentación es concluir que no hay datos no confiables. Los
hay, y son peores que los de una API propia:

- la **salida de `smartctl`**, un binario de terceros cuya versión puede cambiar bajo los pies;
- el **registro de eventos de Windows**, escrito por controladores de fabricantes distintos;
- **filas de SQLite** que escribió la propia aplicación hace seis meses con otro esquema.

Ninguno de los tres avisa cuando cambia de forma.

#### La decisión

**Zod** como biblioteca única de validación y análisis en TypeScript, con los tipos inferidos de los
esquemas (`z.infer<>`) y no declarados a mano en paralelo. Dos declaraciones de la misma forma
acaban divergiendo; una sola no puede.

Toda respuesta de comando y toda carga útil de evento pasa por su esquema antes de tocar el estado.
Un fallo produce un `AppError` con código `ipc.schema_mismatch` y, en el detalle técnico, la ruta
del campo y lo que se esperaba.

En Rust la simetría no necesita dependencia nueva: tipos serde explícitos en cada frontera externa,
y `serde_json::Value` solo como paso intermedio para conservar una captura en bruto, nunca para
alimentar una decisión.

#### Alternativas descartadas

- **Esperar a `ts-rs`.** Genera los tipos desde Rust y evita que front y back diverjan *al compilar*,
  pero no comprueba nada *en ejecución*: no cubre datos de SQLite escritos por una versión anterior,
  que es justo el caso que más preocupa. Ambas cosas son complementarias, no alternativas.
- **Validar solo los eventos.** Los eventos llegan de forma asíncrona y parecen más arriesgados,
  pero una respuesta de `invoke` malformada entra igual al estado. La frontera es la misma.
- **Valibot, ArkType.** Más ligeras, pero Zod es el estándar de hecho del ecosistema y su coste
  —unos pocos KB en una aplicación de escritorio de 140 MB— es irrelevante aquí.

#### Coste asumido

Un esquema por cada tipo del contrato, que hay que mantener al día. Se mitiga con la puerta de CI
que exige prueba de rechazo por esquema: si alguien añade un comando sin esquema, el test de
superficie del contrato lo detecta.

### ADR-023 — Sin variables de entorno en la aplicación

Estado: aceptada.

La aplicación se compila con `adapter-static` y `ssr = false`: **no hay servidor**. De ahí se derivan
consecuencias técnicas que conviene dejar escritas antes de que alguien las descubra a base de
depurar:

- `$env/dynamic/private` y `$env/static/private` **no existen** sin un runtime de servidor. No hay
  ficheros `.server.ts` en este proyecto y no puede haberlos.
- `$env/dynamic/public` **no funciona con prerenderizado**. La vía sería `$env/static/public`, que
  requiere enmienda de la constitución.
- `process.env` no existe en el navegador.

**Decisión:** `.env` se usa únicamente para variables de construcción leídas por Node durante el
build (`vite.config.ts`, `svelte.config.js`, `vitest.config.ts`); hoy son las `TAURI_*` que inyecta
la propia herramienta. Ese fichero nunca se empaqueta. La configuración del producto vive en la
tabla `settings` de SQLite (ADR-006), que es la única fuente.

**Sobre secretos:** la aplicación no tiene cuentas, claves ni servicios externos (ADR-007). Y
conviene dejarlo escrito para el futuro: *un `.env` empaquetado en un instalador de escritorio no es
secreto*, cualquiera puede abrir el instalador y leerlo. Si algún día hiciera falta guardar una
credencial, la vía es el almacén de credenciales de Windows (DPAPI), nunca un fichero de texto junto
al ejecutable.

### ADR-024 — `tracing` en Rust y envoltorio propio en TypeScript

Estado: aceptada.

#### Qué se descarta y por qué

La propuesta inicial era Pino en el servidor y `loglevel` en el cliente, con el nivel controlado por
`LOG_LEVEL` y `PUBLIC_LOG_LEVEL`. Nada de eso encaja aquí:

- **Pino es una biblioteca de Node.** El «servidor» de esta aplicación es Rust; no hay proceso Node
  en ejecución, solo durante la compilación.
- **Las variables de entorno están prohibidas** por el principio XII, y no hay servidor donde
  vivirían. El nivel va a `settings`, que es donde vive la configuración (principio V).

#### Lo que se adopta

**Rust: `tracing`.** Es el estándar de hecho del ecosistema y aporta algo que aquí importa de
verdad: los *spans* permiten seguir una recopilación completa o un benchmark de veinte minutos con
su contexto, sin repetir el identificador en cada llamada. Con `log` + `env_logger` habría que
arrastrarlo a mano por todas partes.

Se acompaña de `tracing-appender` para la rotación diaria y de `time` para el formato de hora.

**TypeScript: envoltorio propio** en `src/lib/logger.ts`, unas cuarenta líneas. No se añade
`loglevel`: son unos 40 KB para lo que aquí resuelven cuarenta líneas, y el envío selectivo por IPC
—que es el requisito real del frontend— habría que escribirlo igual, porque ninguna biblioteca
genérica sabe hablar con Tauri.

#### Consecuencias

- Una dependencia nueva en Rust y ninguna en TypeScript.
- El frontend envía a Rust solo `warn` y `error`; `debug` e `info` se quedan en la consola del
  WebView. Enviar todo saturaría el canal y cambiaría el rendimiento que se intenta diagnosticar.
- El nivel se resuelve con la precedencia `--log-level` > `settings` > `info`. El argumento de línea
  de órdenes es la única forma de diagnosticar un fallo **anterior** a poder leer la configuración.
- **La hora se escribe en local con desplazamiento explícito**, no en UTC ni en una zona fija. Esta
  aplicación correlaciona sus métricas con el Visor de eventos de Windows, que muestra hora local:
  un log en otra zona obligaría a convertir mentalmente cada vez que se coteja un pico de
  temperatura con un evento de disco. El desplazamiento evita la ambigüedad cuando el fichero viaja
  por correo.


### ADR-025 — Instancia única con el plugin oficial de Tauri

Estado: aceptada.

#### El problema

La especificación no pide solo impedir una segunda instancia: pide que **abrir una segunda restaure
la ventana de la primera** (`docs/product-specification.md` §11, `docs/architecture.md` §4). Son dos
requisitos distintos, y el segundo obliga a comunicar los dos procesos.

#### La decisión

`tauri-plugin-single-instance` 2.4, del propio equipo de Tauri, registrado **el primero** de todos
los plugins: se ejecutan en el orden en que se añaden al `Builder`, y este tiene que decidir si el
proceso sigue vivo antes de que nada más se inicialice.

Su devolución de llamada corre en el proceso que ya estaba en marcha y recibe los argumentos y el
directorio de trabajo del segundo, que termina solo. Ahí se llama a
`platform::ventana::restaurar_ventana_principal()`, que desminimiza, muestra y enfoca **en ese
orden**: una ventana minimizada sigue contando como visible, y `set_focus()` sobre una ventana
oculta no hace nada.

#### Alternativas descartadas

- **Mutex con nombre (`CreateMutexW`) sin dependencias.** Quince líneas y cero superficie añadida,
  pero solo resuelve la mitad: la segunda instancia muere en silencio y el usuario, que no ve
  aparecer nada, concluye que la aplicación no arranca. Restaurar la primera ventana exigiría
  escribir igualmente el canal entre procesos, que es exactamente lo que aporta el plugin.
- **Fichero de bloqueo en `%ProgramData%`.** Sobrevive a un cierre inesperado y deja la aplicación
  inarrancable hasta que alguien lo borra a mano. Un mutex del núcleo desaparece con el proceso.

#### Consecuencias

- Una dependencia más en un binario privilegiado. Se acepta porque es oficial, está en el mismo
  espacio de versiones que Tauri y su alternativa exigiría escribir el mismo mecanismo peor.
- El nivel de registro del segundo proceso **no se aplica**: el suscriptor de `tracing` ya está
  instalado con el nivel del primero. Los argumentos del segundo se registran en el log, que es lo
  útil para diagnosticar; cambiar el nivel en caliente requeriría un `reload::Handle` y no compensa.
- La restauración de ventana queda en un único sitio, compartida con el arranque normal.

### ADR-026 — ACL explícita y toma de propiedad de la carpeta de `ProgramData`

Estado: aceptada.

#### El problema

La aplicación guarda en `%ProgramData%\SmartDisk Monitor\` la base SQLite, los logs y los informes.
La suposición de partida era que `ProgramData` ya restringe la escritura a administradores. **Es
falsa**, y se ha medido en un Windows 11 real (`docs/open-questions.md` §R):

```text
C:\ProgramData  BUILTIN\Usuarios:(CI)(WD,AD,WEA,WA)
                CREATOR OWNER:(OI)(CI)(IO)(F)
```

`(CI)` propaga a toda subcarpeta. Un usuario **sin privilegios** puede crear
`C:\ProgramData\SmartDisk Monitor\` antes de que se instale nada y, por `CREATOR OWNER`, queda con
Control total sobre ella. Se comprobó ejecutándolo desde una sesión no elevada.

#### La decisión

El instalador, y solo el instalador, crea la carpeta y le aplica:

```text
icacls "%ProgramData%\SmartDisk Monitor" /setowner *S-1-5-32-544 /t /c
icacls "%ProgramData%\SmartDisk Monitor" /inheritance:r ^
  /grant:r *S-1-5-18:(OI)(CI)F ^
  /grant:r *S-1-5-32-544:(OI)(CI)F ^
  /grant:r *S-1-5-32-545:(OI)(CI)RX
```

Tres detalles que no son opcionales:

1. **`/setowner` primero.** Restablecer la ACL no basta: el propietario conserva `WRITE_DAC`
   implícito y vuelve a concederse Control total en silencio. Medido: tras endurecer la carpeta, el
   usuario que la había creado recuperó la escritura con un solo `icacls /grant`.
2. **SID numéricos, no nombres.** En esta máquina el grupo se llama `Administradores`; en un Windows
   en inglés, `Administrators`. Un instalador que use nombres falla en la mitad del planeta.
   `S-1-5-18` es `SYSTEM`, `S-1-5-32-544` administradores, `S-1-5-32-545` usuarios.
3. **`/inheritance:r` y sin `CREATOR OWNER`.** Sin cortar la herencia, los permisos de `ProgramData`
   siguen aplicándose por debajo de los explícitos.

`platform::paths::log_dir()` deja de crear la raíz en compilación de publicación: si falta, la
instalación está rota y debe notarse, no repararse creando una carpeta con la ACL heredada débil.

#### Alternativas descartadas

- **Comprobar y reparar la ACL al arrancar.** La aplicación va elevada y podría hacerlo, pero exige
  el crate `windows` con `Win32_Security` y código `unsafe` para leer descriptores de seguridad,
  para cubrir un hueco que el instalador ya cierra por completo: después de instalar, nadie sin
  privilegios puede cambiar esos permisos. Se descarta por coste frente a beneficio.
- **Usar `%LocalAppData%` por usuario.** Evitaría el problema, pero rompe el requisito de que el
  historial sea del equipo y no de la cuenta que abrió la aplicación.

#### Consecuencias

- El instalador gana un paso obligatorio y verificable, que forma parte de los criterios de US-060.
- Un usuario sin privilegios puede seguir **leyendo** la carpeta. Es deliberado: la interfaz muestra
  informes y el ZIP de diagnóstico se genera ahí. Los datos ya se anonimizan por defecto y ningún
  número de serie ni ruta de perfil entra en un log.
- Desinstalar conserva `ProgramData` (US-060), así que la ACL endurecida sobrevive a la
  desinstalación y una reinstalación se la vuelve a encontrar. `/setowner` la deja consistente
  igualmente.

### ADR-027 — Vitest 5 con Browser Mode, en dos configuraciones separadas

Estado: aceptada.

#### El problema

El sistema de diseño es vinculante y su incumplimiento es un defecto de producto, no estético. Aun
así había **25 componentes y cero pruebas de componente**, y `@testing-library/svelte` instalado sin
un solo uso. Faltaba el nivel entero.

El proyecto estaba en Vitest 2.1.9, donde el Browser Mode es experimental y su configuración
(`browser.name`) fue **sustituida** en la 3 por `browser.instances` con proveedores en paquetes
aparte. Montarlo sobre la 2.1.9 era escribir una configuración ya retirada.

#### La decisión

**Vitest 5.0.0**, con `@vitest/browser` + `@vitest/browser-playwright` y `vitest-browser-svelte`.
Se retira `@testing-library/svelte`. Momento elegido a propósito: 165 pruebas de lógica pura y
ningún producto encima es lo más barato que va a estar nunca.

**Dos configuraciones, no una** (`docs/testing-strategy.md` §24):

| Configuración | Entorno | Incluye | Medido |
|---|---|---|---|
| `vitest.config.ts` | jsdom | `src/**/*.test.ts` menos las de navegador | 165 pruebas, ~16 s |
| `vitest.browser.config.ts` | Chromium | `src/**/*.browser.test.ts` | 8 pruebas, ~2 s |

`pnpm test` sigue siendo la suite rápida que se teclea mientras se programa, y Stryker apunta solo a
la de Node: mutar código lanza la suite cientos de veces y abrir un navegador en cada una la haría
inviable.

**El sufijo es `*.browser.test.ts`, no `*.svelte.test.ts`** como proponía la primera versión de la
estrategia. Ese sufijo ya estaba tomado por las pruebas de los módulos `.svelte.ts` con runas
—`theme.svelte.test.ts`, `app.svelte.test.ts`—, que corren en Node. El discriminante real es el
entorno de ejecución, no el tipo de fichero.

La zona horaria de la suite se fija en `Europe/Madrid`, no en UTC: es la que tiene cambio de hora, y
ahí es donde aparecen los fallos de retención, de enfriamiento y de correlación con el Visor de
eventos, que muestra hora local.

#### Alternativas descartadas

- **Quedarse en Vitest 2.1.9** fijando `@vitest/browser@2.1.9` y `vitest-browser-svelte@1.1.0`.
  Funciona hoy, pero se escribiría con la API ya retirada, para reescribirla al actualizar y ya con
  producto encima. Y el Browser Mode de la 2 lo marcaba experimental su propio equipo.
- **Vitest 3.2.7** como salto intermedio. Ecosistema más asentado, pero deja dos mayores de deuda.
- **jsdom para los componentes.** Es lo que hay que evitar: el contraste sobre material compuesto,
  el respaldo a `--sdm-solid`, la resolución de variables en tema oscuro y la visibilidad del foco
  **no son observables en jsdom**. Se comprobó en la práctica: los dos defectos que destapó esta
  infraestructura habrían pasado en verde con jsdom.
- **Un proyecto único con `projects`.** Mezclaría los tiempos y obligaría a Stryker a filtrar.

#### Consecuencias

- Cuatro dependencias de desarrollo nuevas y una retirada. Ninguna entra en el binario.
- `tsconfig.json` redefine `include`: al hacerlo **sustituye** al de SvelteKit y sus rutas pasan a
  ser relativas a la raíz. Se repite su contenido y se añade `e2e/`. Sin eso, ESLint analiza las
  pruebas sin tipos y las reglas que los necesitan quedan mudas justo donde más falta hacen.
- `vitest-browser-svelte` 3.1.0 se publicó el mismo día en que se instaló y salta la política de
  antigüedad mínima de `pnpm`. Se comprobó a mano contra la 3.0.0: el `dist/` es **byte a byte
  idéntico** y solo cambia el rango de pares. La excepción queda razonada en `pnpm-workspace.yaml`.

### ADR-028 — Plano de interfaz con Playwright y un IPC propio

Estado: aceptada.

#### El problema

Playwright no puede conducir una ventana de Tauri, pero sí puede conducir la interfaz: es la misma
aplicación sobre el mismo motor, Chromium. Lo que falta es el backend.

#### La decisión

Playwright contra `pnpm preview` —el build, no `vite dev`: es donde aparecen los problemas que el
servidor de desarrollo esconde— con un doble de IPC instalado por `addInitScript`.

**No se usa `mockIPC()` de `@tauri-apps/api/mocks`**: esa función se ejecuta en el proceso de Node y
aquí el doble tiene que existir **dentro de la página**, antes del primer `invoke`. `e2e/ui/ipc-falso.ts`
hace lo mismo que ella —poblar `window.__TAURI_INTERNALS__`— pero en un script de inicialización, y
además registra las llamadas para poder afirmar sobre ellas.

Los fixtures se validan contra **los mismos esquemas Zod** que usa la aplicación. No es ceremonia:
en la primera ejecución rechazaron tres valores inventados (`certain`, `no_smart_support`, `wmi`)
que no existen en el contrato. Sin esa validación, las pruebas habrían pasado en verde probando una
forma de datos que no existe.

Solo Chromium. El WebView2 de la aplicación es Chromium; probar en Firefox o WebKit mediría un motor
que ningún usuario va a ejecutar.

#### Consecuencias

- 17 pruebas de interfaz, ~38 s incluyendo compilación y arranque del servidor de vista previa.
- La accesibilidad se comprueba con `axe` en las seis pantallas **por ambos temas**, doce
  combinaciones. Encontró un incumplimiento real en la primera ejecución.
- El plano de aplicación real (`tauri-driver`) sigue pendiente y es otra cosa: exige el ejecutable
  empaquetado y terminal elevada. Entra con US-060.

### ADR-029 — Una sola copia del sistema de diseño, y su norma en `docs/`

Estado: aceptada. Reemplaza las rutas de ADR-013, cuyo fondo sigue vigente.

#### El problema

El paquete del diseñador llegó como `Design-system/` y se integró en el árbol de la aplicación
(`src/design-system/` y `src/lib/`). La copia original se conservó "como referencia". El resultado
fueron **dos sistemas de diseño vivos a la vez**, y divergieron:

| Fichero | Divergencia |
|---|---|
| `tokens.css` | el arreglo de foco `:focus-visible:focus-visible` (WCAG 2.4.7) solo llegó a la copia de `src/` |
| `design/accent.ts` | 23 líneas distintas |
| `design/format.ts` | 14 líneas distintas |
| `design/health.ts` | 6 líneas distintas |
| `design/types.ts`, `i18n/es.json`, `i18n/en.json` | 3 líneas distintas cada uno |

Lo grave no es la divergencia, sino **quién la leía**: `tools/build-historias.py` generaba el
consolidado desde la copia del paquete. `historias.md` —el documento que se entrega entero a un
agente de IA o a quien se incorpora al proyecto— estuvo publicando los tokens sin el arreglo de
foco. Un agente que se fiara del consolidado habría reintroducido un fallo de accesibilidad ya
resuelto, y todas las puertas de calidad habrían pasado en verde, porque ninguna miraba ahí.

La prueba de que el coste era real y ya se estaba pagando: existía `tools/_nav.py`, un script cuyo
único cometido era aplicar cada cambio **dos veces**, una en cada copia. Se ha eliminado con esta
decisión.

Había además un problema de nombres. La norma de interfaz se llamaba `Design-system/AGENTS.md` y
convivía con el `AGENTS.md` de la raíz, que es la guía general para agentes de IA. Dos ficheros con
el mismo nombre y significados distintos: `AGENTS.md` de la raíz tenía que dedicar un párrafo a
avisar de la confusión.

#### La decisión

**Una sola copia, dentro de `src/`.** La carpeta `Design-system/` desaparece:

| Qué era | Dónde está ahora |
|---|---|
| `Design-system/AGENTS.md` | `docs/ui-design.md` |
| `Design-system/HANDOFF.md` | absorbido en los apéndices A–C de `docs/ui-design.md` |
| `Design-system/design-system/**` | ya estaba en `src/design-system/`; la copia se borra |
| `Design-system/src/**` | ya estaba en `src/lib/`; la copia se borra |
| `Design-system/tailwind.config.cjs` | ya estaba en la raíz; la copia se borra |
| Los tres `.dc.html`, `support.js` | `design/`, con su propio `README.md` |

La norma pasa a llamarse **`docs/ui-design.md`**: elimina la colisión de nombres, la coloca junto al
resto de documentos normativos y hace pareja con `docs/ui-contract.md` —aquel dice qué puede pedir
la interfaz, este cómo se pinta lo que recibe—. Gana además un **§0 «Dónde vive cada cosa»**, que es
lo que antes no existía en ningún sitio: un agente tenía que deducir las rutas.

Los bocetos se conservan porque no viven en ningún otro sitio; el resto no, porque duplicar para
"conservar la referencia" es precisamente lo que causó el problema. **El paquete original íntegro
sigue en el historial de git**, que es donde va lo que se conserva por trazabilidad.

**Y se hace determinista.** `pnpm verify:tokens` gana una comprobación que falla la integración si
aparece un segundo `tokens.css`, un segundo `tokens.json` o un segundo barrel de componentes fuera
de `src/`. Un principio que solo vive en un documento dura hasta el primer día de prisa.

#### Consecuencias

- `historias.md` se genera desde las rutas vivas y suma dos fuentes que faltaban: `tokens.json` y
  `src/lib/components/index.ts`, que es el catálogo real y ejecutable. Pasa de 29 a 31 ficheros.
- Los ficheros de agentes (`AGENTS.md`, `CLAUDE.md`, y los nuevos `GEMINI.md` y `CODEX.md`) llevan
  el mapa de rutas y las órdenes duras de interfaz. Ninguno duplica la norma: apuntan a ella.
- `AGENTS.md` recupera el párrafo que gastaba en avisar de la colisión de nombres.
- Ningún valor de token, umbral ni regla visual cambia. Es reorganización, no rediseño.

### ADR-030 — El backend no manda texto de alerta; solo `ruleKey`, y el frontend lo resuelve por i18n

Estado: aceptada.

#### El problema

Al conectar el motor de alertas (Historia 2, spec `001-monitor-discos-windows`) con los comandos
reales, el contrato existente de `AlertGroup` (`src/lib/design/types.ts`) exigía `title` y
`summary` como **cadenas ya resueltas**, y `AlertCard.svelte` las pintaba directamente. El
componente además tenía literales en español escritos a mano (`"Informativa"`, `"reconocida"`…),
en violación de la regla de cero-literales-de-interfaz.

Rellenar `title`/`summary` desde Rust exigía generar texto en español directamente en el backend
—el propio principio VI lo prohíbe— o cambiar el contrato para mandar una clave que el frontend
resuelva. Es un cambio de contrato, y por tanto exige esta decisión antes de programarlo
(`AGENTS.md`, límites duros).

#### La decisión

El backend deja de mandar `title` y `summary`. `get_alert_groups` y `get_alert_detail` mandan
`ruleKey` (ya lo hacían) y nada más de texto libre; `AlertCard.svelte` resuelve el título y el
resumen con `t(\`alert.rule.${ruleKey}.title\`)` / `.summary`, una clave por cada `rule_key` del
catálogo implementado (`docs/alert-rules.md` §2, subconjunto de `open-questions.md` J.16). El
estado (`activa`/`reconocida`/…) también deja de estar hardcodeado: nuevas claves
`alert.status.*`.

`target` se conserva como campo del backend: no es texto de interfaz traducible, es el alias o
modelo del disco (dato del usuario, no una frase de la aplicación), igual que `disk.alias ??
disk.model` en el resto de la interfaz.

De paso se corrige un desajuste de cable independiente: `AlertSeverity` serializaba
`"warning"`/`"critical"`, pero el esquema Zod de `severity` (compartido con el vocabulario general
de `Severity`) espera `"warn"`/`"crit"`. Se corrige la serialización de cable de `AlertSeverity`
sin tocar el almacenamiento en SQLite (`alert_groups.severity` sigue guardando `warning`/`critical`,
que es lo que exige el `CHECK` de la migración): son dos representaciones distintas del mismo dato,
como ya ocurre con `DeviceType`.

#### Consecuencias

- `src/lib/design/types.ts`: `AlertGroup` pierde `title` y `summary`.
- `src/lib/api/schemas.ts`: el esquema `alertGroup` pierde esos dos campos.
- `AlertCard.svelte` deja de tener literales de interfaz; sus dos diccionarios ganan las claves
  `alert.rule.<rule_key>.title/summary` (una por regla implementada) y `alert.status.*`.
- Ningún componente que ya estuviera consumiendo `title`/`summary` queda roto: el único consumidor
  era este mismo componente, corregido en el mismo cambio.

### ADR-031 — `tauri-plugin-dialog` para elegir el destino de una exportación

Estado: aceptada.

#### El problema

`export_report`, `preview_diagnostic_zip` y `create_diagnostic_zip` (`docs/ui-contract.md` §3.7)
necesitan una ruta de destino que hoy nadie puede producir: `capabilities/default.json` solo
declara `core:default`, sin ningún permiso de acceso al sistema de archivos ni de diálogo. Aceptar
`destinationPath` como una cadena libre construida por la interfaz sería justo la segunda vía de
acceso al sistema de ficheros que el principio IX prohíbe — el mismo motivo por el que
`open_log_folder` no recibe una ruta como argumento.

#### La decisión

`tauri-plugin-dialog` 2.7 (equipo de Tauri), con un único permiso concedido:
`dialog:allow-save` (no el conjunto `dialog:default`, que además habilita `allow-open` y
`allow-message`, innecesarios aquí — mínimo privilegio real, no solo declarado). El usuario elige
carpeta y nombre con el selector nativo de Windows desde `$lib/api`; el backend solo escribe en la
ruta que ese diálogo devuelve, nunca en una construida por la interfaz.

#### Alternativas descartadas

- **Carpeta fija sin diálogo** (como la carpeta controlada del benchmark, T077/J.27): evita la
  dependencia y el permiso nuevos, pero un informe o un ZIP de diagnóstico está pensado para
  salir del equipo — adjuntarlo a un correo, subirlo a un ticket de soporte—, y forzarlo siempre a
  la misma carpeta interna contradice ese uso. Se ofreció como alternativa real (`AskUserQuestion`)
  y el usuario prefirió el diálogo nativo.

#### Consecuencias

- Dependencia nueva en un binario privilegiado (`tauri-plugin-dialog` + su equivalente JS
  `@tauri-apps/plugin-dialog`) y permiso nuevo en `capabilities/default.json`. Se acepta por ser
  oficial del equipo de Tauri, la UX esperada de cualquier «Guardar como» de la plataforma, y por
  decisión explícita del usuario.
- `$lib/api` gana el envoltorio de `save()` del plugin; ninguna pantalla lo llama directamente
  (mismo criterio que el resto de la frontera IPC).

### ADR-032 — El crate `zip` para el paquete de diagnóstico

Estado: aceptada.

#### El problema

T090 necesita empaquetar varios ficheros (ajustes, eventos exportados, capturas SMART brutas,
registro de actividad) en un único ZIP de diagnóstico (US-051). No había ninguna dependencia de
compresión en `Cargo.toml`.

#### La decisión

`zip` 2.4 (`zip-rs/zip2`, mantenido, muy usado), con `default-features = false` y la sola
característica `deflate` (arrastra a su vez `deflate-flate2` y `deflate-zopfli`: la variante solo
`flate2` no compila sin depender también de una de las dos, así que se acepta `zopfli` en el árbol
en vez de pelear con la selección de características). Sin `aes-crypto`, `bzip2`, `lzma`, `zstd`,
`xz` ni `chrono`: el ZIP de diagnóstico no necesita cifrado ni otros algoritmos de compresión, y
cada uno de esos suma dependencias transitivas propias.

#### Alternativas descartadas

- **Escritor ZIP propio, sin comprimir** (mismo criterio que el LCG de T079 frente a `rand`):
  descartado porque el formato ZIP tiene más superficie de la que parece a primera vista —cabeceras
  local y central, CRC32, el registro de fin de directorio central— y un error ahí no falla alto:
  produce un ZIP que algunos lectores abren mal y otros no, que es peor que no tener la función. El
  LCG de T079 era mucho más simple (un generador congruencial lineal, no un formato de contenedor
  con implicaciones de compatibilidad). Se ofreció como alternativa real (`AskUserQuestion`) y el
  usuario prefirió el crate.

#### Consecuencias

- Dependencia nueva en un binario privilegiado, con `zopfli` como dependencia transitiva
  (algoritmo de compresión, sin superficie de seguridad relevante: no toca red ni entrada externa
  sin confiar, solo comprime bytes ya generados por la propia aplicación).
- El ZIP de diagnóstico admite compresión real (no solo `stored`), lo que mantiene manejable el
  tamaño del registro de actividad incluido por FR-029c.

### ADR-033 — El planificador en segundo plano es un hilo bloqueante que sondea cada 1 s, sin `tokio`

Estado: aceptada.

#### El problema

T020/T021/T022 necesitaban un bucle real que ejecutara la recopilación (SMART, contadores de
rendimiento, eventos de Windows, altas/bajas de inventario) sin que la interfaz tuviera que pedirlo:
hasta esta historia solo existía `refresh_now`, un comando manual. Cada uno de los cuatro trabajos
tiene su propia cadencia configurable (`collectors::planificador`), que además se reduce en batería
para los que no alimentan alertas graves (FR-030), y debe reaccionar a la pausa manual, a un cambio
de ajuste de frecuencia y al cierre real de la aplicación sin quedarse colgado.

#### La decisión

Un único hilo bloqueante (`tauri::async_runtime::spawn_blocking`, lanzado en `.setup()`), con un
bucle que **sondea cada 1 segundo** (`open-questions.md` J.34) en vez de dormir el intervalo
completo del próximo trabajo: en cada sondeo comprueba la señal de parada, si está pausado, y para
cada uno de los cuatro trabajos si ya toca ejecutarse (`collectors::planificador::trabajos_debidos`,
función pura, probada con tiempo inyectado). Cada trabajo debido se ejecuta de forma independiente
—el fallo de uno no bloquea a los demás (`open-questions.md` J.39)— y el post-proceso común
(notificaciones, `alerts:changed`, `metrics:updated`, `inventory:changed`, `source:degraded`, icono
de bandeja) se comparte con `refresh_now` a través de `post_procesar_ciclo`. Todos los colectores son
E/S síncrona (procesos, SQLite, FFI de Windows), nunca futuros, así que no hace falta ningún runtime
asíncrono nuevo: `tauri::async_runtime::spawn_blocking` ya viene con Tauri, cero dependencias nuevas.
El apagado limpio pasa por `RunEvent::Exit`/`ExitRequested` (`lib.rs`, vía `.build().run(|_,event|
...)` en vez de `.run(...)` directo), que marca un `Arc<AtomicBool>` en `AppState` comprobado en
cada sondeo — un único punto de parada, sea cual sea la vía de salida real.

#### Alternativas descartadas

- **Un temporizador (`tokio::time::interval` o similar) por trabajo**: exigiría añadir `tokio` como
  dependencia directa (hoy solo llega transitivamente vía Tauri, y no para Windows) y sincronizar
  cuatro relojes independientes contra pausa, batería y ajustes que cambian en caliente —más
  complejidad para el mismo resultado que un sondeo de 1 s ya da con cuatro comparaciones de
  `Instant`.
- **Interceptar cada `app.exit(0)` por separado** para señalizar la parada: descartado porque hay
  al menos dos puntos de salida (cierre real de ventana, "Salir" de la bandeja) y cualquier futuro
  tercero se olvidaría con facilidad; `RunEvent::Exit`/`ExitRequested` es el único punto que Tauri
  garantiza que se dispara siempre, venga de donde venga la salida.

#### Consecuencias

- La aplicación monitoriza de verdad sin intervención manual (T022 queda satisfecho además "gratis":
  `AppState.paused` nunca se persiste, así que todo arranque empieza activo).
- `SourceHealth` pasa de estar sin tipar (`status: String`) a un enum `SourceStatus` real
  (`ok`/`partial`/`unsupported`/`timeout`/`error`), aunque esta historia solo produce `ok`/`timeout`/
  `error` (`open-questions.md` J.37); `partial`/`unsupported` quedan para cuando alguien los pida.
- `refresh_smart` se separó en `refresh_smart` (solo SMART) y `refresh_metricas_rendimiento` (solo
  PDH): antes estaban acopladas porque nada las llamaba con cadencias distintas
  (`open-questions.md` J.38).
- `metrics:updated.historyWriteHalted` refleja un cálculo real de espacio libre, pero **no** detiene
  todavía ninguna escritura (`open-questions.md` J.40): queda como seguimiento explícito, no como
  olvido.

### ADR-034 — Sistema de diseño v3: «escena de datos» con paleta propia «Ciruela»

Estado: aceptada. Fecha: 2026-09-06. Enmienda ADR-013. Spec: `specs/002-rediseno-v3/`.

#### El problema

Las capturas de la interfaz v2 sobre un Windows real (`design/entregable-rediseno/salida/capturas/`)
mostraron tres defectos de presentación que no son de implementación:

1. El panel general parece a medio cargar: con dos discos la rejilla ocupa ~230 px y deja ~570 px de
   lienzo vacío.
2. Cada magnitud es texto plano del mismo tamaño y color; nada dice si «41 °C» está bien, y un dato
   ausente pesa más que un dato presente.
3. El acento azul heredado de Windows (`#0067c0`) es correcto pero indistinguible de cualquier
   utilidad del sistema; en tema oscuro el conjunto queda gris plano.

El diseñador entregó una propuesta (`design/propuesta-redisenov2/`) que los resuelve sin reescribir
el sistema.

#### La decisión

El sistema de diseño evoluciona a **v3**. `docs/ui-design.md` pasa a describir v3. Cambios:

- **Paleta «Ciruela»**: neutros malva, acento morado de tinta (`#7a3f9d` claro / `#c79aec` oscuro) y
  crítico desplazado al bermellón (`#b03434` / `#ef8080`) para no confundirse con el acento. Todos
  los ratios de contraste están medidos sobre el material compuesto en `docs/open-questions.md`.
- **`--sdm-on-accent` deja de ser blanco en tema oscuro**: el acento oscuro es claro y el texto
  blanco encima daba 2,27:1. Pasa a tinta (`#20132a`, 7,80:1). Todo texto sobre el acento usa
  `--sdm-on-accent`, nunca `text-white`.
- **Familia de «display»** (`--sdm-font-display`) y dos escalones nuevos (58 px, 76 px) para cifras y
  titulares (nunca texto corrido), mediante la clase `.sdm-display`. **No se empaqueta una segunda
  familia tipográfica**: `--sdm-font-display` resuelve a la familia sans ya empotrada. Se descartó Bricolage
  Grotesque para no ampliar la superficie de un binario privilegiado que se distribuye a terceros
  (constitución §III); si se revisa, `.sdm-display` y su `@font-face` son el único punto de cambio.
- **Tres componentes nuevos** en el catálogo, cada uno con su justificación contra `ui-design.md` §3:
  `Icon` (juego propio de 15 iconos de línea que heredan `currentColor`), `Sparkline` (trazo sin
  ejes) y `HeroPanel` (dato dominante del panel). Ningún componente se elimina.
- **La `Sidebar` pasa a un riel de 74 px** solo con iconos, devolviendo 176 px de ancho al contenido
  (crítico a 1024 px). La lista de discos sale de la barra (ya está en la rejilla del panel).

#### Qué NO cambia

El material de tres capas y sus desenfoques (28 / 24 / 44), la escala de radios concéntricos
(18 → 13 → 9 → cápsula), el movimiento (220 ms, `cubic-bezier(.32,.72,0,1)`, `active:scale-[0.98]`,
`prefers-reduced-motion`), el catálogo cerrado, `tokens.css` como fuente única de verdad visual, y
**todas** las reglas de producto. No se añade ningún comando Tauri ni ningún permiso: el rediseño es
de presentación, salvo los cambios de frontera acotados que la spec 002 documenta (una preferencia
de apariencia cuyo campo ya existía, la marca del asistente inicial y los umbrales de perfil de
alerta).

#### Alternativas descartadas

- **Mantener el azul de Windows y ofrecer Ciruela como tema alternativo**: duplica el mantenimiento
  de dos identidades y deja sin resolver el problema original (la aplicación no se reconoce) para la
  mayoría de usuarios, que no cambian de tema.
- **Empaquetar Bricolage Grotesque**: aporta carácter a las cifras grandes pero obliga a gestionar
  un binario y una licencia OFL más en un instalador privilegiado. El coste no compensa; se deja la
  puerta abierta con un único punto de cambio.

#### Consecuencias

- `docs/ui-design.md` §0, §2, §2.bis y §3 se reescriben para v3. §4 (composición), §6
  (accesibilidad) y §8 (definición de terminado) se conservan literalmente y siguen siendo el
  criterio de aceptación visual de cada pantalla.
- El catálogo pasa de N a N+3 componentes.
- La entrega se hace en nueve PR ordenados por dependencia (`specs/002-rediseno-v3/plan.md`).

### ADR-035 — La herencia del acento de Windows pasa a opción apagada de fábrica

Estado: aceptada. Fecha: 2026-09-06. Matiza ADR-017. Spec: `specs/002-rediseno-v3/`.

#### El problema

ADR-013 adoptó «la herencia del acento de Windows» como parte de la identidad visual de v2, y
ADR-017 construyó toda la corrección de contraste (`accessibleAccent()`, `accentOnSurface()`, el
barrido de los 262.144 acentos posibles de `open-questions.md` §O) sobre esa premisa. Pero heredar
el acento del sistema es justo lo que hace que la aplicación no se distinga de cualquier utilidad de
Windows (ADR-034, problema 3).

#### La decisión

Con la paleta Ciruela, el acento propio es el comportamiento **de fábrica**. Heredar el acento de
Windows pasa a un interruptor en Ajustes → Apariencia, **apagado por defecto**
(`settings.appearance.useSystemAccent`, cuyo campo ya existía en el backend; solo cambia su valor de
fábrica de `true` a `false`, implantado en PR 8 de `specs/002-rediseno-v3/`). Al encenderlo,
`applySystemAccent()` sobrescribe los **tres roles de acento** —`--sdm-accent` (fondo),
`--sdm-accent-fg` (texto), `--sdm-on-accent` (texto sobre el fondo)— más sus dos derivados
(`--sdm-accent-hi`, `--sdm-accent-soft`): cinco propiedades CSS en total. Al apagarlo,
`clearSystemAccent()` las restaura todas.

**La parte técnica de ADR-017 se conserva entera**: cuando el interruptor está encendido, el acento
del usuario sigue pasando por `accessibleAccent()` / `accentOnSurface()` para no bajar de AA en
ningún tema. El acento sigue sin comunicar salud y sigue siendo acción/selección: no se toca ningún
principio de la constitución §VI, solo se precisa que la herencia es opcional. La viñeta del acento
de §VI se ajustó en consecuencia (constitución 1.7.0, 2026-09-06).

#### Alternativas descartadas

- **Quitar la herencia por completo**: se pierde valor para el usuario que prefiere integrarse con
  su sistema, y se tira una inversión de trabajo (ADR-017, `open-questions.md` §O) que ya está hecha
  y probada.
- **Dejar la herencia encendida de fábrica y Ciruela como respaldo**: no resuelve el problema para
  la mayoría, que no toca los ajustes de apariencia.

#### Consecuencias

- El texto de la preferencia (`settings.appearance.useSystemAccent.label` / `.hint`) se reescribe:
  hoy asume el comportamiento contrario.
- La definición de terminado de `ui-design.md` §8 sigue exigiendo verificar cada pantalla con un
  acento del sistema claro y en los dos temas: el camino de la herencia no se abandona, se hace
  opcional.

### ADR-036 — El motor de alertas se parametriza por perfil

Estado: aceptada. Fecha: 2026-09-06. Spec: `specs/002-rediseno-v3/` (US10). Amplía `alert-rules.md` §2.

#### El problema

El rediseño v3 añade un paso al asistente inicial y una sección a Ajustes para elegir «cuánto avisa»
la aplicación con un perfil (Prudente / Equilibrado / Solo lo grave). Para que esa elección no sea
decorativa —y la constitución §I exige que la interfaz no mienta sobre qué está activo— el motor de
alertas tiene que **consumir de verdad** los umbrales.

Hasta ahora no lo hacía: `alerts::motor` es puro y sus umbrales estaban **escritos a mano**
(`90/100` para desgaste, `70/80` para temperatura). `settings.alerts.temp_configured_warn_c` se
guardaba y `get_settings` lo devolvía, pero **ninguna regla lo leía** — el mismo hueco que
`open-questions.md` J.32 describía para las claves de capacidad.

#### La decisión

1. **`settings.alerts` gana siete claves**: `profile` (`cautious`|`balanced`|`quiet`|`custom`),
   `wear_warn_percent`, `wear_crit_percent`, `media_errors_warn_per24h`, `media_errors_crit_per24h`,
   `driver_retry_warn_per24h`, `driver_retry_crit_per24h`. Cada una con su rango, su validación
   (`crit` más severo que `warn`) y su prueba de rechazo (constitución §XI).
2. **El motor las lee**. `alerts::motor` sigue puro: recibe los umbrales como parámetros;
   `commands::refresh_smart` los resuelve de `settings` una vez por ciclo y se los pasa. Reglas
   afectadas:
   - `smart.wear_high` → `wear_warn_percent` / `wear_crit_percent`.
   - `temp.above_configured_warn/crit` → `temp_configured_warn_c` / `_crit_c`. La histéresis de
     resolución conserva su margen (aviso − 3 °C, crítico − 5 °C).
   - `smart.media_errors` → la activación pasa de «el contador aumentó» a «el **incremento** entre
     dos lecturas alcanza `media_errors_warn/crit_per24h`». El sufijo `Per24h` es histórico: **no**
     es una ventana de 24 h (spec 002, clarify Q1). `smart.error_log` no se parametriza.
   - **`capacity.low` / `capacity.critical`**: este ADR las **implementa en el motor** (antes solo
     figuraban en `alert-rules.md`, sin código). Requiere persistir `volume_free_bytes` como muestra
     periódica de cada volumen — antes la capacidad solo vivía como instantánea en
     `volumes.free_bytes`. `domain::capacidad::estado_capacidad` es el espejo Rust de
     `capacityState()` de `src/lib/design/health.ts`.
3. **El umbral térmico de fábrica baja a 60/70 °C** (era 70/80), que es el valor del perfil
   Equilibrado. Decisión de producto adoptada (spec 002, clarify Q2): 60 °C sigue siendo temperatura
   alta para un SSD de consumo y mantener dos números («fábrica» vs «Equilibrado») confundiría.
4. **Elegir un perfil escribe sus doce umbrales de golpe** y guarda el identificador. **Editar a
   mano cualquiera de esos umbrales** pone `profile = "custom"`; la única forma de volver a un perfil
   concreto es elegirlo. La interfaz muestra «Personalizado (a partir de \<perfil anterior\>)».

#### Lo que queda fuera

- **`driver_retry_warn/crit_per24h` se guardan pero ninguna regla los consume todavía**: las reglas
  `events.controller_reset` / `events.io_retry` necesitan el colector de eventos completo (Historia
  4). El perfil escribe los doce valores igualmente, así el día que exista esa regla ya tiene su
  umbral — el mismo patrón con el que las claves de capacidad y `logging.verbose` vivieron guardadas
  sin consumidor (J.32, FR-029a). Registrado en `docs/open-questions.md`.
- No se añade ningún comando Tauri ni ningún permiso: todo pasa por el `set_setting` genérico.

#### Alternativas descartadas

- **Guardar los perfiles pero no cablear el motor** (opción del planteamiento inicial): el paso 3 del
  asistente y la sección de Ajustes serían decoración. El usuario eligió el alcance completo.
- **Implementar una ventana de conteo real «por 24 h»** para errores de medios y reintentos: mucho
  más código en el motor (mecánica de conteo nueva + sus cinco pruebas) para un matiz que la
  reinterpretación sobre las reglas existentes ya cubre (clarify Q1).

#### Consecuencias

- `alert-rules.md` §2: la tabla pasa a decir «valor configurado (`settings.alerts.*`)» donde antes
  ponía `> 70 °C` / `≥ 90` / «aumenta»; `capacity.low`/`critical` dejan de estar pendientes.
- `metric_samples` gana un tipo de muestra: `volume_free_bytes` con `MetricTarget::Volume`. La
  retención lo compacta igual que el resto (misma columna `volume_id` del esquema).
- `VolumeSummary` gana `is_system_volume` (necesario para `selectHeroDisk()`, spec 002 clarify Q3),
  calculado al leer con `GetSystemWindowsDirectoryW`, sin migración de esquema.

### ADR-037 — Apagar las notificaciones es un ajuste propio, no solo pausar

Estado: aceptada. Fecha: 2026-09-06. Spec: `specs/002-rediseno-v3/` (US8, paso 3 del asistente).

#### El problema

El paso 3 del asistente inicial (`cambios/08-onboarding.md`) ofrece un `Switch` «Avisarme con una
notificación de Windows». Hasta v3 el toast nativo estaba **siempre activo** cuando la ventana estaba
minimizada (`product-specification.md` §Notificaciones); lo único que se podía apagar era el
**sonido** (`notifications.sound_enabled`). Sonido ≠ presencia: alguien puede querer el aviso sin el
«ding», y también puede querer ningún aviso emergente sin tener que **pausar toda la recopilación**
(que es lo que hoy silencia las notificaciones, y de paso deja de vigilar los discos).

#### La decisión

Clave nueva `notifications.enabled` (booleano, **fábrica: `true`**). La consume
`alerts::notificaciones::procesar_una`: si está en `false`, no se muestra el toast, con independencia
de la transición de la alerta. La alerta **sigue existiendo** en la lista y sigue contando para el
color de salud — apagar el aviso emergente no apaga la vigilancia (constitución §I). Se persiste con
el `set_setting` genérico; no añade comando ni permiso. La decisión de enviar se factoriza a una
función pura `debe_enviar(...)` con sus pruebas (el resto de `procesar_una` necesita un proceso Tauri
real, `research.md` R1).

#### Alternativas descartadas

- **Reutilizar `sound_enabled`**: cambia la semántica de una clave existente y confunde («sin sonido»
  no es «sin aviso»).
- **Depender de pausar**: pausar es una acción temporal y global; no es una preferencia de «no quiero
  ventanas emergentes».

### ADR-038 — El autoarranque es una tarea programada, no una entrada `Run`

Estado: aceptada. Fecha: 2026-09-06. Spec: `specs/002-rediseno-v3/` (US8, paso 3 del asistente).

#### El problema

El paso 3 ofrece «Arrancar SmartDisk con el sistema». La aplicación corre bajo
`requireAdministrator` (manifiesto, UAC al abrir). Una entrada en
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run` la lanzaría con el **token sin elevar** en cada
inicio de sesión: Windows mostraría un diálogo de UAC en cada login, o el arranque fallaría en
silencio. El plugin `tauri-plugin-autostart` usa exactamente esa clave `Run` (y además sería una
dependencia nueva).

#### La decisión

Clave nueva `lifecycle.start_with_system` (booleano, **fábrica: `false`**). Al activarla,
`platform::autoarranque::aplicar(true)` registra una **tarea programada** —
`schtasks.exe /Create /TN "SmartDisk Monitor - Autostart" /SC ONLOGON /RL HIGHEST`— que el
Programador de tareas eleva **sin diálogo**. Al desactivarla, `/Delete`. `reset_settings` con
`scope: "all"` también borra la tarea. Solo se mira el **código de salida** de `schtasks`: la
codificación de su salida de texto no es fiable entre configuraciones de Windows
(`.claude/rules/backend-rust.md`), así que no se parsea `stdout`. El comando real no se ejecuta en
`cargo test` (crearía una tarea en el equipo del desarrollador): se prueba el formato de la línea de
comando y se verifica a mano, igual que J.28 y `platform/sistema.rs`.

#### Alternativas descartadas

- **Clave `Run` de HKCU** (y `tauri-plugin-autostart`, que la usa): UAC en cada login por la
  elevación; y el plugin es dependencia nueva sin justificación (límite duro de `AGENTS.md`).
- **Carpeta «Inicio» del menú**: mismo problema de elevación que `Run`.

### ADR-039 — Ilustraciones del asistente inicial: SVG propio con tokens, no recurso empaquetado

Estado: aceptada. Fecha: 2026-09-06. Extiende `docs/ui-design.md` §3 (catálogo). Origen: revisión
de las capturas del asistente sobre un Windows real.

#### El problema

Los cuatro pasos del asistente inicial (US-002) tenían huecos donde el diseño pedía una figura y
solo había un cuadrado de `Icon` diminuto —cuando llegaba a verse: el sprite se montaba solo en
`AppShell`, que esta ruta no usa (corregido aparte: `IconSprite` en `+layout.svelte`)—. Un icono de
18 px no llena una pantalla de bienvenida a ancho completo. El asistente es la primera impresión del
producto y quedaba pobre.

#### La decisión

Un componente nuevo, **`OnboardingArt`**, con **cuatro escenas** (`welcome`, `disks`, `alerts`,
`done`), una por paso. Son **SVG en línea escritos a mano**, planas, estilo «Corporate Memphis /
Alegría» adaptado a la «escena de datos» de v3: formas geométricas rotundas, **sin figuras
humanas** (el motivo es siempre el hardware y su vigilancia — y así no hace falta un tono de piel,
que no es un token).

Reglas que cumple, como cualquier pieza del catálogo:

- **Solo `currentColor` y `var(--sdm-*)`.** Ni un color literal; lo verifica `pnpm verify:tokens`.
  Por eso funciona en tema claro y oscuro **sin una sola condicional**. Las coordenadas y los
  `stroke-width` del dibujo son geometría, no valores de tema (misma consideración que el sprite de
  `IconSprite`).
- **Decorativa**: sale `aria-hidden`, sin nombre accesible. El texto de cada paso ya lo dice todo;
  un `role="img"` sin contenido informativo sería peor (`ui-design.md` §6).
- La paleta del dibujo es sobre todo **acento + neutro**; el verde `ok` solo aparece donde refuerza
  el mensaje real del producto (el latido de «constantes vitales» del paso 1, el sello de
  conformidad del paso 4). El acento **no** codifica salud aquí: es identidad visual.
- Exportada en el barrel; `width` como única prop (el alto sale de la proporción 8:5).

#### Alternativas descartadas

- **Empaquetar PNG/SVG generados con una herramienta de ilustración**: un binario más en un
  instalador privilegiado, con su hash que mantener (constitución §III y §IX), y un recurso que no
  reacciona al tema — habría que entregar dos juegos (claro/oscuro) y conmutarlos. El SVG con
  tokens se adapta solo.
- **Seguir con cuadros de `Icon`**: no es una ilustración, es un pictograma; no llena la pantalla
  ni da carácter a la primera impresión.
- **Ilustración con personajes al estilo Corporate Memphis puro**: obliga a decidir tonos de piel
  sin un token que los represente, y desentona con una aplicación de sistema. Se conserva el
  lenguaje de formas, no las figuras.

#### Consecuencias

- El catálogo suma `OnboardingArt`. Su uso está acotado al asistente; no es un patrón general de
  «mete una ilustración donde quieras».
- `docs/ui-design.md` §3 lo recoge y §7.6 (asistente) menciona la escena por paso.
- Si en el futuro otra pantalla quiere una ilustración, se decide entonces con el criterio de
  `ui-design.md` §3, no por analogía con esta.

### ADR-040 — La geometría de la ventana se recuerda en `settings`, no con un plugin

Estado: aceptada. Fecha: 2026-09-06.

#### El problema

La ventana principal nacía siempre con el tamaño fijo de `tauri.conf.json` (1360 × 880, centrada).
Se quiere que la primera vez abra a **1695 × 988** y que, a partir de ahí, recuerde entre sesiones
el **tamaño, la posición y si estaba maximizada**.

#### La decisión

El estado de la ventana se persiste en la tabla **`settings`** de SQLite, en cinco claves
internas (`window.width`, `window.height`, `window.x`, `window.y`, `window.maximized`), en
**píxeles lógicos** (independientes del escalado de Windows, igual que `tauri.conf.json`).

- **El frontend no participa.** No hay comando nuevo ni evento nuevo. Todo ocurre en Rust:
  `platform::ventana::aplicar_geometria_guardada` se llama en `.setup()` **antes** de mostrar la
  ventana (que nace `visible: false`, así que no hay salto), y `persistir_geometria` se llama al
  cerrar (`CloseRequested`) y al salir (`RunEvent::ExitRequested`, que cubre «Salir» de la bandeja
  y el apagado).
- **Ausente ⇒ valor de fábrica.** Sin filas `window.*` manda `tauri.conf.json`: el primer
  arranque abre a 1695 × 988 centrada. `reset_settings` (ámbito «resto» o «all») borra las claves.
- **Maximizada**: se guarda solo `window.maximized = true` y no se tocan tamaño/posición, para que
  al restaurar y quitar la maximización la ventana vuelva al tamaño que el usuario había elegido.
- **Posición fuera de pantalla**: `aplicar_geometria_guardada` comprueba con `geometria_visible`
  (función pura, con pruebas) que la barra de título cae dentro de algún monitor actual con margen;
  si el monitor donde estaba se ha desconectado, se ignora la posición y la ventana abre centrada.
- **Cuándo se guarda**: al pulsar la X (aunque esa X minimice a la bandeja) y en `ExitRequested`.
  Una muerte dura del proceso (Administrador de tareas) pierde el último movimiento; es aceptable
  (constitución §II.5, simplicidad antes que generalidad) y evita un temporizador de *debounce*
  sobre `Resized`/`Moved`.

#### Alternativas descartadas

- **`tauri-plugin-window-state`** (el estándar de Tauri para esto): guarda su estado en un fichero
  JSON propio, fuera de SQLite — choca de frente con el principio **V** (INNEGOCIABLE): «`localStorage`
  está prohibido para estado del producto… Todo se almacena en SQLite… Ningún otro almacén de datos
  estructurados». Además, añadirlo sería **dependencia nueva** (enmienda de la constitución, §III) y
  traería **permisos de Tauri nuevos** (su ADR). La vía `settings` no necesita nada de eso.
- **`localStorage` + redimensionar al arrancar desde el frontend**: mismo choque con el principio V,
  y produce un salto visible (la ventana ya está pintada cuando se redimensiona), y el
  almacenamiento del WebView es frágil ante limpiezas.
- **Debounce sobre `Resized`/`Moved`**: más robusto ante una muerte dura, pero necesita un
  temporizador y escribe en SQLite durante el arrastre. No compensa para el caso que se da.

#### Consecuencias

- `docs/data-model.md` §2 documenta las cinco claves `window.*`.
- `docs/open-questions.md` J.8 y `docs/ui-design.md` §4.0 pasan la predeterminada a 1695 × 988; el
  **objetivo de diseño** (1280 × 720) y el **mínimo técnico** (1024 × 560) no cambian.
- Sin contrato nuevo, sin DTO `ts-rs`, sin esquema Zod, sin permiso de Tauri, sin dependencia.

### ADR-041 — `DiskSummary` lleva la autoevaluación SMART (`smartHealthPassed`)

Estado: aceptada. Fecha: 2026-09-06.

#### El problema

El `HeroPanel` del panel general muestra cuatro «hechos» del disco protagonista. El boceto aprobado
(`design/propuesta-rediseno`, `smartdisk-v3.html`) pone como primero **«Salud del firmware ·
Correcta»**; la implementación mostraba en su lugar **«Ocupación · N %»** (porcentaje ocupado del
volumen principal). La autoevaluación SMART (`smart_status.passed`) sí se recopila —se persiste como
métrica `health_passed` (1.0/0.0) y se expone en `DeviceDetail.counters`—, pero **no viaja en
`DiskSummary`**, que es el único DTO que reciben el `HeroPanel` y la `DiskCard`. La clave i18n
`disk.firmwareHealth` ya existía en los dos diccionarios, dejada preparada.

#### La decisión

Añadir a `DiskSummary` el campo `smart_health_passed: Option<bool>` (`smartHealthPassed` en el
wire): `Some(true)` autoevaluación superada, `Some(false)` fallida, `None` sin dato o disco sin
SMART. Lo rellena `enrich_with_smart_data` leyendo la última muestra de `health_passed`, igual que
ya lee temperatura, desgaste y horas. El `HeroPanel` sustituye el hecho «Ocupación» por «Salud del
firmware» (`icon: shield`, en rojo solo si `false`); el orden de hechos pasa a ser el del boceto:
firmware, desgaste, actividad, horas.

#### Alternativas descartadas

- **Mantener «Ocupación» y aceptar la desviación del boceto.** Coste cero (nada de backend). Se
  descarta porque el boceto es la referencia vinculante (`ADR-034`, `docs/ui-design.md` §0), la
  ocupación de un volumen ya la comunica la barra de capacidad de cada `DiskCard`, y el trabajo del
  Hero es «¿tengo un problema?» —donde «el disco ha fallado su propia autoevaluación» encaja y «el
  disco está lleno al 93 %» ya tiene su alerta y su barra—.
- **Derivarlo en el frontend de `disk.state`.** No sirve: `state` refleja alertas y frescura, no el
  resultado del autotest. Un disco puede tener `state = ok` y `health_passed = false` en el mismo
  ciclo en que se está creando la alerta `smart.health.failed`.
- **Reusar `provenance` o un campo existente.** Opaco y frágil; un booleano nuevo es más honesto.

#### Consecuencias

- Un campo anulable más en un DTO que `ts-rs` ya refleja: regenera `generated/DiskSummary.ts` y
  `generated/DeviceDetail.ts` al compilar. `DeviceDetail` lo hereda por `#[serde(flatten)]` —
  inofensivo, ya tenía el mismo dato en `counters`.
- Esquema Zod (`schemas.ts`) y su prueba de rechazo; interfaz en `src/lib/design/types.ts`;
  `docs/ui-contract.md` §3.2.
- Sin comando nuevo, sin permiso de Tauri, sin dependencia.

### ADR-042 — Los colectores no retienen el mutex de la conexión durante la E/S externa

Estado: aceptada. Fecha: 2026-09-07.

#### El problema

`AppState` tiene una única `Mutex<Connection>`. El bucle de recopilación en segundo plano
(`iniciar_planificador` → `ejecutar_ciclo`) tomaba ese candado y, **con el candado en la mano**,
lanzaba la E/S externa lenta de cada colector:

- `refresh_smart`: por cada disco, una cascada de hasta 5 modos de `smartctl.exe` × 15 s de límite
  = **hasta 75 s por disco**.
- `refresh_metricas_rendimiento`: `perf_counters::leer` **duerme 1 s** entre las dos muestras PDH
  que exige calcular una tasa → ≥ N s con N discos.
- `refresh_events`: la lectura del registro de eventos por FFI (`wevtapi.dll`), segundos con
  backlog grande.

Cualquier comando de consulta de la interfaz (`get_system_events`, `get_alert_groups`,
`get_devices`, `set_setting`…) hace `conn.lock()` y se quedaba esperando todo ese tiempo. El
usuario lo vivía como **congelación de varios segundos al cambiar de sección** en el sidebar: la
navegación de SvelteKit espera al `load` de la ruta, el `load` espera al comando, y el comando
espera al candado. `docs/architecture.md` §4 ya decía «la UI y los colectores no comparten
operaciones bloqueantes» y la constitución §V exige «transacciones breves»: el código lo incumplía.

#### La decisión

Reestructurar `refresh_smart`, `refresh_metricas_rendimiento` y `refresh_events` en **tres fases**:

1. **Planificar** — candado breve: leer los dispositivos y la configuración necesarios y construir
   un plan de trabajo.
2. **Recopilar** — **sin candado**: toda la E/S externa (subprocesos, PDH, FFI).
3. **Persistir** — candado único: escribir muestras, registrar fallos y evaluar alertas.

Los orquestadores reciben `&Mutex<…>` (no una guarda) y toman y sueltan el candado ellos mismos;
`conn` y `source_health` nunca se anidan. Una guarda `AppState.recoleccion_smart: Mutex<()>`
conserva la semántica de «el refresco manual espera al ciclo en curso» sin retener `conn`.
`refresh_inventory` ya cumplía (su E/S por PowerShell corre antes del candado) y no se toca.

#### Alternativas descartadas

- **Una segunda `Connection` de solo lectura en `AppState`.** La interfaz leería por su conexión
  mientras el colector escribe por la suya. Se descarta: introduce `SQLITE_BUSY` real entre las dos
  conexiones (que hoy no existe con una sola), obliga a manejar reintentos, y **no arregla el lado
  escritor** —`set_setting`, `acknowledge_alert` y demás comandos que escriben seguirían detrás del
  candado del colector—. Queda como posible mejora futura independiente para las lecturas pesadas
  de informes.
- **Bajar el límite de la cascada de `smartctl`.** Reduce el síntoma, no la causa: con dos discos
  lentos se vuelve a notar, y perder modos de sondeo deja discos sin leer.

#### Consecuencias

- El bucle de fondo y un `refresh_now` manual pueden **solapar sus subprocesos `smartctl` en la
  fase 2**. Sin corrupción —filas nuevas, borrado lógico de `devices`, evaluación de alertas
  serializada en la fase 3— y la guarda `recoleccion_smart` lo evita del todo.
- Hay un desfase entre el `ahora` de la fase 1 y la escritura de la fase 3; ya ocurría antes y la
  fase 3 dura milisegundos.
- La asimetría de contabilidad de `smartctl` (fallo de consulta suma intento y fallo; JSON
  inválido suma solo intento; fallo de persistencia no toca contadores) se traslada intacta y
  ahora está cubierta por pruebas de `smart_planificar` y `smart_persistir`.
- Sin comando nuevo, sin permiso de Tauri, sin dependencia. `.claude/rules/backend-rust.md` recoge
  la trampa.


---

# 12. Cuestiones abiertas y mediciones

Fichero de origen: `docs/open-questions.md`

Registro de todo lo que la especificación dejaba a interpretación, con el valor que se ha adoptado.
Nació de la revisión cruzada de `docs/` contra el paquete de diseño previa a la implementación.

**Cómo leerlo.** Cada entrada tiene un estado:

| Estado | Significa |
|---|---|
| `DECIDIDO` | Resuelto por el responsable del producto. Es normativo: implementar tal cual. |
| `PROPUESTO` | Valor por defecto adoptado para no bloquear el trabajo. Vale hasta que se revise; cambiarlo es barato ahora y caro después de programarlo. |
| `ABIERTO` | Requiere una decisión o una medición que todavía no se ha hecho. Bloquea la historia que lo cita. |

Cuando una entrada se cierra, se traslada su contenido al documento normativo que corresponda
(`product-specification.md`, `alert-rules.md`, `ui-contract.md`, `AGENTS.md`) y aquí queda solo el
resumen y el enlace. Este archivo no es una fuente de verdad paralela: es la sala de espera.

---

### A. Contradicciones internas — corregidas

Ya aplicadas en el repositorio. Se listan porque cambian ficheros que estaban aprobados.

| # | Qué pasaba | Cómo se ha resuelto | Estado |
|---|---|---|---|
| A.1 | `tokens.css` descargaba Instrument Sans de Google Fonts, contra spec §11 y ADR-007 | `@font-face` local sobre dos subconjuntos (`latin` y `latin-ext`) ya presentes en `design-system/fonts/`, con su `OFL.txt`. Cerrado | `DECIDIDO` |
| A.2 | `AGENTS.md` exigía 32 px de objetivo interactivo; los tokens daban 29–30 px | La norma pasa a 30 px y `--sdm-control-sm` sube de 29 a 30. Queda por encima de los 24 px de WCAG 2.2 AA | `PROPUESTO` |
| A.3 | `AGENTS.md` mandaba `gap: space-4`; el token y el boceto usan 18 px (`space-5`) | 18 px entre tarjetas, 16 px dentro de una tarjeta, 8 px entre controles de una fila | `DECIDIDO` |
| A.4 | `HANDOFF.md` decía 26 componentes; hay 25, y 8 no tenían notas de uso | Recuento corregido, tabla del catálogo completada y 4 componentes nuevos autorizados | `DECIDIDO` |
| A.5 | `TimeSeriesChart` repartía el eje X por índice de muestra y tenía ancho fijo de 780 px | Reescrito: escala temporal real, dominio `from`/`to`, ancho fluido, cursor con teclado, textos en i18n | `DECIDIDO` |
| A.6 | `format.ts` formateaba con `navigator.language` mientras los textos seguían al idioma elegido | Todo formatea con `i18n.formatLocale`, que sigue al idioma de la app conservando la variante regional si comparte idioma | `DECIDIDO` |
| A.7 | Nadie actualizaba `<html lang>` al cambiar de idioma | `i18n.init()` e `i18n.set()` lo escriben | `DECIDIDO` |
| A.8 | `applySystemAccent()` inyectaba cualquier color sin comprobar contraste | `accessibleAccent()` elige texto blanco/negro y oscurece el acento hasta AA si hace falta | `DECIDIDO` |
| A.9 | `TimeSeriesChart` tenía literales en español en el marcado | Todo pasa por i18n, incluido el `aria-label` | `DECIDIDO` |

---

### B. Alertas y estado de salud

#### B.1 · Qué color muestra un disco con una alerta reconocida · `DECIDIDO`

El estado de un disco es la peor severidad de sus alertas en estado `active` **o** `acknowledged`.
Reconocer la saca de la lista de pendientes y le añade un distintivo; no cambia el color. Solo
`resolved` y `archived` dejan de contar. Un disco sin alertas pero sin datos frescos es `unknown`,
no `ok`.

*Por qué:* el color es la señal de salud del hardware; si reconocer lo apagara, dejaría de ser
fiable. Implementado en `deviceState()` y `alertCountsTowardHealth()` (`src/lib/design/health.ts`).

*Consecuencia asumida:* un disco con un problema crónico se queda en rojo. Se mitiga con el
distintivo de "reconocida" y con el orden de la lista, no apagando el color.

*Corrección 2026-09-06:* la decisión estaba solo a medias. `enrich_with_smart_data` (Rust) siempre
pasa `None` como severidad a `device_state`, así que `DiskSummary.state` únicamente refleja la
frescura de SMART (`ok` / `unknown`), nunca `warn` / `crit`; y ninguna pantalla fundía `app.alerts`
con `app.devices`. Resultado observado: con tres alertas `active` el panel decía «Todo en orden».
Se cierra con `estadoConAlertas(disk, alerts)` (`src/lib/design/health.ts`), que el panel general y
el chrome aplican antes de leer `disk.state`. **Cuentan las alertas dirigidas al dispositivo
(`…|device:<id>`) y a cualquier volumen suyo (`…|volume:<id>`)**: un volumen lleno es un problema
del disco que lo contiene, no una categoría aparte. `deviceState()` se conserva como la definición
canónica de la regla y la prueba de `health.test.ts` que la fija.

#### B.2 · El silencio no es un estado · `DECIDIDO`

`mutedUntil` es ortogonal a `AlertStatus`: una alerta puede estar activa y silenciada a la vez. El
silencio suprime **la notificación**, nunca el color ni la presencia en la lista. Sobrevive a un
reinicio (se persiste como fecha absoluta UTC en `alert_groups`) y se aplica al grupo, no a la regla
entera. Un silencio indefinido se guarda como `"infinite"`.

#### B.3 · Histéresis de resolución · `PROPUESTO`

La especificación definía cuándo se activa cada alerta, nunca cuándo se resuelve. Regla general:
una alerta se resuelve automáticamente cuando la condición deja de cumplirse **con margen** durante
**tres ciclos consecutivos** de su recopilador. El margen por tipo de regla está en
`docs/alert-rules.md`, columna *Resolución*.

*Por qué:* sin margen, un disco oscilando en 69,5–70,5 °C generaría un ciclo activa→resuelta→activa
por muestra, que es exactamente el ruido que la agrupación pretende evitar.

#### B.4 · Alertas informativas · `DECIDIDO`

`Severity` mantiene `info`, pero **ninguna regla de la v1.0 la produce**. Se conserva en el tipo
porque los eventos de Windows de nivel informativo se muestran en la cronología de un grupo. Un
`info` nunca crea un grupo de alerta por sí solo ni afecta al estado de un disco.

#### B.5 · Qué cuenta para el estado global y el color de la bandeja · `DECIDIDO`

Implementado en `trayState()`:

1. Si hay alguna alerta crítica vigente → **rojo**, aunque la monitorización esté pausada. La
   condición sigue siendo cierta aunque hayamos dejado de mirar; la pausa se comunica con el texto
   del menú y un aviso en la `Toolbar`, no apagando la señal.
2. Si no, pausa, fallo general del recopilador o cero discos monitorizados → **gris**.
3. Si no, alguna advertencia → **ámbar**.
4. Si no → **verde**.

Un `unknown` no impide el verde por sí solo, pero sí cuando su causa es `unreadable` o
`collector-error`: eso es una degradación real y aporta una advertencia (`unknownContributesWarning`).
Un dispositivo que declara no soportar SMART (`unsupported`) es normalidad y no ensucia nada.

*Corrección 2026-09-06:* también estaba a medias.
- `enrich_with_smart_data` marcaba `not-yet-sampled` («aún no medido») cuando en realidad había
  habido lecturas y dejaron de llegar. Ahora, si hay al menos una muestra histórica y la última no
  es fresca, el motivo es `unreadable` («dejó de responder»). El caso `not-yet-sampled` queda solo
  para un disco que nunca ha devuelto nada.
- `unknownContributesWarning()` era **código muerto**. **Dónde se aplica** (decisión del usuario,
  tras verlo): un disco `unknown` se presenta siempre como **«Sin datos SMART» en gris** —en la
  tarjeta, el Hero y la fila del reparto—, sea cual sea el motivo. Que un `unknown` por
  `unreadable`/`collector-error` **cuente para «N necesitan atención»** y el color de la bandeja lo
  decide `estadoParaRecuento()` (solo lo usa el chrome), no `estadoConAlertas()` (que lo usan las
  tarjetas y se queda en `unknown`). Con la monitorización en pausa tampoco cuenta. Así el reparto
  es una partición limpia (Correcto + Advertencia + Crítico + Sin datos SMART = total) y el usuario
  ve el mismo texto y color en todas partes; la urgencia del disco que dejó de responder vive en el
  recuento de arriba, en la notificación y en el grupo de alerta `smart.unreadable`.
- `selectHeroDisk()` gana un criterio intermedio: sin alerta de dispositivo, protagoniza el disco
  con problema —`crit`, luego `warn`, luego un `unknown` que cuenta como degradación— antes que el
  de sistema, para que el Hero no muestre «Todo en orden» habiendo un disco en apuros. El
  `HeroPanel` estrena un texto genérico («Este disco necesita atención…») para el caso sin alerta.

*Grupo de alerta `smart.unreadable` — hecho (2026-09-06):* cada ciclo de SMART escribe la métrica
`smart_query_ok` (1.0 leído / 0.0 falló), incluidos los ciclos que fallan
(`commands::registrar_ciclo_smart_fallido`, que ya no hace solo `continue`). `alerts::evaluar_unreadable`
la evalúa con el motor de siempre: 3 ceros seguidos → advertencia, una lectura correcta la resuelve,
cooldown 6 h. La compuerta «un disco que **sí** respondía» la da `repo_metricas::hubo_lectura_smart_correcta`
(un `smart_snapshots` con `query_status` de éxito): un disco que nunca dio datos es «no compatible»,
no «ilegible», y no dispara la regla. Cinco pruebas en `alerts/mod.rs` (`alert-rules.md` §5). El
disco ilegible ahora **sí** aparece en la pantalla de Alertas con su cronología y notifica.

#### B.6 · Cambio de severidad de un grupo ya reconocido · `PROPUESTO`

Si un grupo `acknowledged` sube de severidad (advertencia → crítico), vuelve a `active` y se
notifica de nuevo. Si baja, conserva `acknowledged`. El reconocimiento vale para lo que el usuario
vio, no para algo peor que aún no ha visto.

#### B.7 · Recaída tras resolución · `DECIDIDO`

Un grupo resuelto que vuelve a cumplirse **no** crea un grupo nuevo: reactiva el existente e
incrementa `cycle`. La cronología separa visualmente los episodios por ciclo. Así el contador
histórico ("esto ha pasado 14 veces en tres meses") no se pierde, que es lo que pedía US-030.

#### B.8 · Retirada de un disco USB · `PROPUESTO`

"Disco retirado inesperadamente: crítico inmediato" no puede aplicarse tal cual a USB: expulsar
correctamente un pendrive monitorizado generaría un crítico falso. Reglas:

- Se escucha `WM_DEVICECHANGE`; una retirada precedida de una solicitud de expulsión limpia
  (`DBT_DEVICEQUERYREMOVE` concedida) **no** genera alerta, solo un evento de inventario.
- Una retirada sin aviso previo en un dispositivo con `bus_type = USB` genera **advertencia**.
- En cualquier otro bus, genera **crítico**, como decía la especificación.

#### B.9 · Ventanas de conteo · `DECIDIDO`

Donde la especificación decía "tras tres muestras" o "tras tres intentos", se entiende **tres
ciclos consecutivos del recopilador correspondiente**, no tres dentro de una ventana. Con la
frecuencia por defecto: 90 s para temperatura, 15 min para SMART. Recogido en `alert-rules.md`.

#### B.10 · Un disco sin SMART fresco no enseña su última lectura como si fuera de ahora · `DECIDIDO`

`enrich_with_smart_data` conserva `temperatureC` / `percentageUsed` / `powerOnHours` con la última
muestra persistida aunque ya no sea fresca (solo `state` y `unknownReason` se condicionan a la
frescura). En la `DiskCard`, si `unknownReason` no es `null` —bus sin SMART, disco que dejó de
responder, o primera lectura aún no llegada— las tres magnitudes se muestran como «—», nunca el
valor viejo ni un contador de rendimiento en vivo presentado como lectura SMART (boceto
`01-panel-general.md` §4, constitución §I). La marca de dato obsoleto con la hora de la última
lectura válida es trabajo aparte (afecta al `HeroPanel`, ver §K).

---

### C. Capacidad

#### C.1 · Suelo absoluto solo en volúmenes grandes · `DECIDIDO`

La regla original ("el mayor entre 10 % y 20 GB") marcaba como crítico un volumen de 64 GB con
15 GB libres, que es el 23 %. Regla adoptada:

- Siempre por porcentaje: <10 % advertencia, <5 % crítico.
- Además, **solo en volúmenes de 256 GB o más**, por valor absoluto: <20 GB advertencia, <10 GB
  crítico.
- Gana el criterio más severo de los dos.

El corte de 256 GB es configurable
(`alerts.capacity.absoluteFloorMinCapacityBytes`). Implementado en `capacityState()`.

#### C.2 · Desactivación por volumen · `DECIDIDO`

US-033 permite desactivar las alertas de capacidad por volumen. Es una preferencia por
`volume_guid`, no por letra de unidad, y sobrevive a un cambio de letra.

---

### D. Frecuencias, batería y pausa

#### D.1 · Límites de las frecuencias configurables · `PROPUESTO`

La especificación decía "configurables dentro de límites seguros" sin definirlos. La pantalla de
Ajustes no se puede diseñar sin ellos:

| Trabajo | Por defecto | Mínimo | Máximo |
|---|---|---|---|
| Temperatura, actividad, capacidad, latencia | 30 s | 10 s | 5 min |
| SMART completo | 5 min | 1 min | 60 min |
| Eventos de Windows | 30 s | 15 s | 5 min |
| Detección de altas y bajas | 60 s | 30 s | 10 min |

Por debajo del mínimo el coste de CPU y de despertar el disco deja de compensar; por encima del
máximo la aplicación deja de merecer el nombre de monitor.

#### D.2 · Comportamiento en batería · `PROPUESTO`

Con el equipo a batería se multiplica por **4** el intervalo de temperatura/actividad/capacidad y de
detección de altas y bajas. SMART completo y eventos de Windows **no se alteran**: son las fuentes de
las alertas graves, y spec §4 exige no suspenderlas. Al volver a red se restauran de inmediato y se
fuerza un ciclo completo.

#### D.3 · Qué hace exactamente "Pausar" · `PROPUESTO`

Pausa la recopilación y la evaluación de reglas; por tanto también las notificaciones. **No** se
persiste entre reinicios: arrancar la aplicación siempre reanuda. Mientras está pausada:

- la `Toolbar` muestra un aviso permanente con el tiempo transcurrido;
- las alertas ya existentes conservan su estado y su color;
- el icono de la bandeja sigue la regla B.5.

*Por qué no se persiste:* una pausa olvidada es un monitor que no monitoriza y no lo dice. El coste
de reanudar sin querer es mucho menor que el de no vigilar durante semanas.

---

### E. Historial y gráficas

#### E.1 · Tabla intervalo → resolución · `PROPUESTO`

| Intervalo pedido | Resolución servida | Origen |
|---|---|---|
| ≤ 24 h y dentro de los últimos 7 días | muestras crudas | `metric_samples` con `resolution = raw` |
| ≤ 7 días | agregados de 5 min | `resolution = five_minutes` |
| ≤ 90 días | agregados de 5 min si existen, si no horarios | mixto |
| > 90 días o personalizado antiguo | resúmenes horarios | `resolution = hourly` |

Reglas asociadas:

- La gráfica **declara siempre** la resolución que está mostrando (`resolutionLabel`): un máximo
  promediado no es un pico, y confundirlos al investigar un incidente térmico sería grave.
- Si se pide un rango anterior a la instalación o ya compactado, el tramo sin datos se dibuja como
  hueco, nunca se recorta el eje ni se interpola.
- Tope de **1.500 puntos** por serie; por encima, el backend submuestrea conservando mínimo y máximo
  de cada cubo, y lo indica en la respuesta.
- **Un salto es un hueco a partir de 2,5× la cadencia** (`MULTIPLO_HUECO` en `domain::series.rs`,
  `FACTOR_HUECO` en `src/lib/design/series.ts`). Empezó en **1,5×** (`completar_serie`, T060), pero
  usando la aplicación de verdad el usuario vio que la gráfica del detalle de disco salía como
  **puntos sueltos**: 1,5× marca como hueco cada ciclo de recopilación puntualmente perdido —normal
  en un equipo que se suspende o va cargado—, y cada racha de una muestra se dibujaba como un punto
  solitario. 2,5× ≈ dos ciclos: un salto suelto no parte la línea, una parada de minutos u horas
  sigue quedando como banda gris. `domain::retencion`/`domain::salud` mantienen su 1,5× (cubos,
  frescura: otra decisión).
- **El trazo es una curva suave, no una polilínea recta**: spline cúbica de Hermite **monótona**
  (`rutaSuave` en `series.ts`), por tramo continuo, que no rebasa el mínimo/máximo de cada segmento
  —así no aparenta cruzar un umbral ni inventa un pico—. Es «la onda del boceto» sin falsear el
  dato. `Sparkline` y `TimeSeriesChart` la comparten.

#### E.2 · Interacción de la gráfica · `DECIDIDO`

Cursor de lectura con ratón (el punto más cercano en tiempo) y con teclado (flechas, `Inicio`,
`Fin`, `Esc`), que muestra hora y valor en el pie. Sin zoom ni selección por arrastre en la v1.0:
el `SegmentedControl` de intervalo cubre la necesidad y evita un patrón nuevo. Implementado.

#### E.3 · Retención mínima frente a US-022 · `DECIDIDO`

US-022 promete "al menos 30 días de historial": se cumple con los agregados de 5 minutos, no con las
muestras crudas (7 días). La historia se reformula para decirlo explícitamente y no dar a entender
que habrá 30 días de detalle.

#### E.4 · La sparkline del panel general se ajusta al último tramo continuo · `DECIDIDO` (2026-09-06)

El `HeroPanel` y las miniaturas de la `DiskCard` piden 24 h, pero **dibujan solo el último tramo
sin cortes** (`ultimoTramoVisible()` en `src/lib/design/series.ts`), no las 24 h enteras. Motivo:
con la app parada a ratos —se cierra, se reinicia el equipo, se acaba de instalar— el histórico
tiene huecos de horas que `Sparkline` pinta como rayas sueltas (regla «un hueco es un hueco», que
no cambia). Enseñar el tramo en curso devuelve la onda del boceto y su ancho se adapta a lo que
hay: un minuto de datos → ventana de un minuto (**sin mínimo de zoom**, decisión del usuario).
El trazo curvo (spline monótona, E.1) refuerza ese «devuelve la onda»; el corte por tramo del
`ultimoTramoVisible` (4× P25) es independiente del umbral de hueco de E.1 (2,5×) y no cambia.

- El corte se hace donde una separación supera **4×** el **percentil 25** de las separaciones
  reales (no la mediana: con pocas muestras y un parón, media serie *es* el parón). Un ciclo
  perdido no abre tramo nuevo; un parón de horas sí.
- El pie del Hero muestra la ventana real («Ventana: 8 min» / «Ventana: 24 h», `formatSpanShort`).
- **Recién abierta la app** hay una o dos muestras y no da para una onda: por debajo de **4 puntos
  o minuto y medio** de ventana el Hero no dibuja la rayita casi plana —parecía un fallo—, pone
  «Recopilando datos…» y se rellena solo en unos minutos.
- **No afecta** al detalle de disco (`/disks/[id]`): ahí el `SegmentedControl` de intervalo y los
  ejes son la interfaz, y la ventana la elige el usuario.
- Los factores (4×, p25, umbral de «recopilando») son de afinado; si un histórico real se ve mal,
  se ajustan aquí.

---

### F. Identidad de dispositivo

#### F.1 · Composición de la huella · `PROPUESTO`

`fingerprint = sha256(model | capacity_bytes | bus_type | wwn_o_pnp_device_id)`.

**El firmware queda fuera a propósito.** La arquitectura pide detectar cambios de firmware; si
formara parte de la huella, actualizar el firmware partiría el historial del disco en dos
dispositivos distintos. Un cambio de firmware se registra como evento de inventario sobre la misma
entidad.

#### F.2 · Dispositivos sin número de serie · `PROPUESTO`

Se monitorizan igual, con `serial_number = null` y la huella de F.1 como identidad, dejando
`identity_confidence = "fingerprint"`. La UI marca esos discos como "identidad inferida" en el
detalle. Si además cambia el `PNPDeviceID` (un USB movido de puerto), se tratará como dispositivo
nuevo: es una limitación conocida y documentada, no un fallo.

#### F.3 · Sustitución de disco · `DECIDIDO`

El historial se conserva ligado a la entidad antigua, marcada con `removed_at`, y el disco nuevo
arranca su propia entidad. Nunca se fusionan historiales, ni siquiera con la misma capacidad y
modelo.

---

### G. Contrato UI ↔ backend

#### G.1 · Empuje, no sondeo · `DECIDIDO` (ADR-015)

El backend emite eventos Tauri tipados; la UI no usa `setInterval` para pedir datos. Lista completa
en `docs/ui-contract.md`.

#### G.2 · Forma del error · `DECIDIDO`

Todo comando que falle devuelve un `AppError { code, messageKey, messageVars, detail, source,
retryable }`. `messageKey` da la frase humana, `detail` el texto técnico literal que se muestra
dentro de un `<details>` y se puede copiar. Definido en `src/lib/design/types.ts`.

#### G.3 · Generación de los tipos · `PROPUESTO`

Los DTO se generan desde Rust con `ts-rs` y se comprueban en CI: si un tipo de Rust cambia y el
`.ts` generado no coincide con el del repositorio, la compilación falla. Evita que
`docs/ui-contract.md` envejezca en silencio, que es el destino habitual de este tipo de documento.

---

### H. Decisiones de ingeniería

#### H.1 · SvelteKit con `adapter-static` y SSR desactivado · `DECIDIDO` (ADR-014)

Es la vía que Tauri documenta oficialmente y la que el paquete de diseño ya asumía (`$lib`). Aporta
enrutado por ficheros para las siete pantallas sin añadir dependencias de terceros, cosa que un
router externo sí haría y que `AGENTS.md` §1 prohíbe.

#### H.2 · Resto de convenciones · `PROPUESTO`

Versiones, gestor de paquetes, estructura de carpetas, linters y CI en
`docs/engineering-conventions.md`.

---

### I. Riesgos técnicos a validar en Fase 0

De los siete, cuatro están cerrados. Los tres que siguen abiertos **no son medibles hoy**: uno
necesita el instalador, otro hardware que no hay y el tercero una lista virtualizada que aún no
existe. Cada uno queda anclado a la historia que lo desbloquea, en lugar de a una lista aparte que
nadie mira.

| # | Riesgo | Qué hay que comprobar | Si sale mal | Estado |
|---|---|---|---|---|
| I.1 | WebView2 no viene preinstalado en Windows Server | ~~Pendiente~~ **Resuelto**: instalador sin conexión del runtime Evergreen (ADR-020). La matriz de sistemas no cambia; el instalador pasa a ~140 MB. Véase §M | — | `DECIDIDO` |
| I.2 | Notificaciones toast desde un proceso elevado | Si Windows las entrega con la app bajo `requireAdministrator` y AUMID registrado | Plan B: ventana propia con el componente `Toast`, anclada sobre la bandeja | `ABIERTO` — se mide al empaquetar: **US-060** |
| I.3 | Codificación de la salida de `chkdsk` | ~~Pendiente~~ **Resuelto**: no es CP850 sino CP1252, y las herramientas de Windows no coinciden entre sí. Detección validada. Véase §Q | — | `DECIDIDO` |
| I.4 | Acento del sistema con contraste bajo | ~~Pendiente~~ **Resuelto**: barrido del espacio sRGB completo. `accessibleAccent()` era correcto, pero faltaba el acento como texto. Véase §O | — | `DECIDIDO` |
| I.5 | `smartctl` tras controladoras RAID y puentes USB | Qué cascada de `-d` (`sat`, `nvme`, `sntjmicron`, `csmi`) merece la pena antes de declarar "no compatible" | Se documenta la limitación por modelo de puente | **Parcial**: el formato de ruta (`/dev/pdN`, no `\\.\PhysicalDriveN`) ya está medido y corregido contra 2 SATA + 2 NVMe reales, véase J.42. La cascada de modos para puentes USB/RAID exóticos sigue `ABIERTO` — necesita ese hardware concreto: **US-010** |
| I.6 | Instancia única y ACL de `ProgramData` | ~~Pendiente~~ **Resuelto**: eran dos problemas. La instancia única exige comunicar procesos, no solo detectarlos (ADR-025). Y `ProgramData` **no** restringe la escritura a administradores: un usuario sin privilegios se apropia de la carpeta pre-creándola (ADR-026). Véase §R | — | `DECIDIDO` |
| I.7 | Rendimiento de la interfaz con 20 discos y 5.000 eventos | ~~Pendiente~~ **Resuelto**: medido con Playwright + `PerformanceObserver` de tareas largas, aislando el coste fijo de la primera navegación del coste real de la interacción. Cero tareas ≥50 ms en ambos escenarios (desplazar 5.000 eventos, recibir 20 discos en caliente). Véase J.26 | — | `DECIDIDO` |

---

### J. Cuestiones menores resueltas por defecto

Todas `PROPUESTO`. Se agrupan porque ninguna merece una sección propia, pero todas eran una
asunción del programador.

| # | Cuestión | Valor adoptado |
|---|---|---|
| J.1 | Base de las unidades de tamaño | Base 1024 con etiquetas KB/MB/GB, como el Explorador de Windows. Documentado en `format.ts` para que nadie lo "corrija" |
| J.2 | Unidad de caudal en el contrato | `bytes/s` en el dato; la conversión a MB/s vive solo en `formatThroughput()` |
| J.3 | Restricción de `metric_samples` | Exactamente uno de `device_id` / `volume_id` no nulo, garantizado por `CHECK` |
| J.4 | Idioma de eventos y de `chkdsk` | Vienen en el idioma de Windows. Se muestran tal cual, marcados como "texto original del sistema" |
| J.5 | Informe HTML exportado | Autónomo: CSS embebido, sin fuentes ni recursos remotos, tema claro forzado y hoja de impresión propia |
| J.6 | Versionado de exportaciones | Campo `schemaVersion` en JSON, ZIP y cabecera de CSV |
| J.7 | Cursor del registro de eventos | *Bookmark* del Event Log, no `RecordId` suelto: al limpiar un canal los identificadores se reinician y se perderían eventos en silencio |
| J.8 | Tamaño de ventana | Mínimo técnico 1024 × 560, objetivo de diseño 1280 × 720, **predeterminada 1695 × 988** (solo el primer arranque; luego manda la geometría guardada, §W y ADR-040). Medido en §L.2 |
| J.9 | Acerca de | Diálogo modal sobre la pantalla actual, no sección de la `Sidebar` |
| J.10 | Eventos en la navegación | Sección propia en la `Sidebar`, con filtro preaplicado al entrar desde el detalle de un disco |
| J.11 | Plurales en i18n | Función `tp()` con `Intl.PluralRules`; claves `<clave>.one` / `<clave>.other` |
| J.12 | Persistencia de tema e idioma | `theme.set()` e `i18n.set()` devuelven la clave a guardar, pero **no** persisten: el llamante debe invocar `set_setting`. Es fácil de olvidar; conviene un envoltorio que lo haga |
| J.16 | Qué reglas de `alert-rules.md` §2 entran en el primer motor de alertas | Solo las que evalúan datos de `smartctl` ya persistidos (§3, sin colector de eventos/capacidad/estado de recopilador): `smart.health.failed`, `nvme.critical_warning`, `smart.media_errors`, `smart.error_log`, `smart.spare_below_threshold`, `smart.wear_high`, `temp.above_configured_warn/crit` (8 reglas; ampliada desde la lista original al conectar el motor con datos reales — T051 — porque `error_log_entries_total` ya lo produce el parser y `motor::evaluar_error_log` ya estaba probado, sin motivo real para dejarlo fuera). Quedan explícitamente fuera —no implementadas a medias, no simuladas— las que dependen de: registro de eventos (`events.*`, `device.removed_unexpected`, `inventory.duplicate_id`: Historia 4), capacidad de volumen (`capacity.*`: Historia 3), límite del fabricante (`temp.above_vendor_limit/critical`: requiere parsear umbrales de atributo SMART, no implementado), fallo de consulta (`smart.unreadable`) y estado del recopilador (`collector.stalled`): ambos necesitaban el seguimiento de estado por fuente que T020/T021 aportaban. **Actualizado tras T020/T021**: ese seguimiento ya existe (`SourceHealth`/`source:degraded`, `open-questions.md` J.37), pero todavía no hay ninguna regla de `alert-rules.md` que lo consuma para producir `smart.unreadable`/`collector.stalled` — sigue siendo trabajo de una historia de alertas futura, ya no de recopilación. Provisional hasta que exista esa regla (spec 001-monitor-discos-windows, T046). **Actualizado (rediseño v3)**: ya están implementadas `smart.unreadable` (métrica `smart_query_ok`), `capacity.low/critical` (serie `volume_free_bytes`, ADR-036), `temp.above_vendor_limit` (métrica `vendor_temp_limit_celsius` desde `temperature.op_limit_max`; sin límite del fabricante un disco sigue con `temp.above_configured_warn`) y `collector.stalled` (`motor::colector_estancado` + `alerts::evaluar_collector_stalled` sobre `SourceHealth`, evaluada en `post_procesar_ciclo`; una fuente que nunca tuvo éxito no dispara). Siguen fuera: `temp.above_vendor_critical` (`smartctl` no expone un crítico del fabricante fiable en el JSON). **Caso raro asumido**: si al desplegar hay un `temp.above_configured_warn` activo en un disco que justo empieza a reportar `op_limit_max`, ese grupo deja de evaluarse y se queda activo hasta archivarse a mano — el estado «reporta límite del fabricante» de un disco no cambia en la práctica. **Actualizado (spec `003-puente-eventos-alertas`)**: implementadas las 10 reglas `events.*`, `device.removed_unexpected` (por `disk` 157 correlacionado **o** por baja de inventario de un disco no USB, J.47) e `inventory.duplicate_id`, con la ventana de correlación de ráfaga de 60 s (J.49). El motor de eventos vive en `src-tauri/src/alerts/{reglas_eventos,eventos,correlacion_rafaga}.rs`; lo conecta `commands::refresh_events`. **Ya no queda ninguna regla de `alert-rules.md` §2 sin implementar salvo `temp.above_vendor_critical`.** Límite conocido: `inventory.duplicate_id` deduplica por el par de discos solo si los dos números se leen del mensaje del evento; si no, cae a un grupo único (`provider:event_id`). Las reglas de objetivo de volumen (`events.filesystem_error`, etc.) crean alertas **sin objeto** mientras `system_events.volume_id` siga sin poblarse (la correlación por volumen es trabajo futuro del colector) |
| J.17 | Cómo se resuelven `smart.media_errors` y `smart.error_log`, que según `alert-rules.md` resuelven "sin aumento durante 24 h" | **No implementado.** Esa resolución es temporal (tiempo transcurrido sin incremento), no de N ciclos consecutivos sobre el valor como el resto de la histéresis de `motor.rs`, y requeriría persistir cuándo fue el último incremento por grupo — no existe ese seguimiento. Ambas reglas quedan **activas hasta archivarse a mano** una vez creadas, igual que `smart.wear_high` (que sí documenta ese comportamiento como definitivo; estas dos no deberían quedarse así para siempre). Pendiente de una vía real: bien un campo temporal nuevo en `alert_groups`, bien un barrido periódico que compare `last_occurrence_at_utc` contra la ventana de 24 h (spec 001-monitor-discos-windows, T051). **Actualizado (spec `003`)**: la spec 003 construyó justo ese barrido —`alerts::eventos::resolver_grupos_de_eventos_vencidos`, que resuelve un grupo cuando `ahora - last_occurrence_at_utc` supera la ventana de la regla, corriendo cada ciclo en `post_procesar_ciclo`— pero **acotado a las reglas de eventos** (`es_regla_de_eventos`). Cerrar J.17 es ahora barato: darles a `smart.media_errors`/`smart.error_log` una ventana de resolución de 24 h y sumarlas al barrido. Se deja **fuera del alcance de la spec 003** a propósito (no es una regla de eventos) pero deja de necesitar diseño nuevo: es cablear el barrido existente |
| J.18 | Dónde se conecta la evaluación del motor con los datos reales | En `alerts::evaluar_smart(conn, device_id, ahora_utc)`, llamado desde `commands::refresh_smart` justo tras `persist_smart_reading` para cada dispositivo — un fallo al evaluar alertas se registra y no interrumpe el resto del ciclo (mismo criterio SC-008 que ya aplicaba a la propia lectura SMART). Sin esta llamada el motor nunca produce ningún `alert_group` en la aplicación real, por probado que esté en aislamiento; se descubrió al construir la bandeja del sistema (T052), cuando no había ninguna alerta real que mostrarle (spec 001-monitor-discos-windows, T051) |
| J.19 | Cuándo se recalcula el color del icono de la bandeja, y qué hace el botón de cierre | El color (`domain::salud::tray_state`, espejo exacto de `trayState()` en `health.ts`) se recalcula en los puntos de sincronización existentes: arranque, `refresh_now`, las seis acciones sobre alertas y pausar/reanudar. **Actualizado tras T020/T021/T022**: ya hay un ciclo real (el planificador en segundo plano sondea cada 1 s y ejecuta cada trabajo según su propia cadencia, `open-questions.md` J.34), así que el icono se recalcula también al cerrar cada ciclo de recopilación, no solo en los puntos de sincronización manuales de antes. El icono en sí se genera en memoria (RGBA) con los mismos `--sdm-{ok,warn,crit,unknown}` de tema claro, sin fichero `.ico` nuevo — `docs/decisions.md` (línea 102) ya señala systray como pantalla sin revisión visual. **Actualizado en T099**: el botón de cierre (`X`) ya lee `lifecycle.close_action` de `settings` en cada cierre (no solo al arrancar, porque Ajustes puede cambiarlo mientras la aplicación sigue abierta) y minimiza o sale de verdad según lo que diga; sin ninguna clave guardada, sigue minimizando — el lado seguro ya razonado aquí. Lo que **no** se construyó, porque ninguna tarea de la Historia 7 lo pedía explícitamente: un diálogo emergente la primera vez que se cierra, preguntando "¿minimizar o salir?" con una casilla de "recordar". La especificación (§3) lo sugiere ("pregunta... y permite recordar la decisión"), pero `platform::ventana.rs` es un fichero de backend, no de interfaz, y añadir ese diálogo habría exigido un evento nuevo (`docs/ui-contract.md` §4 no tiene ninguno para esto) y un componente nuevo fuera del catálogo cerrado — la vía elegida en su lugar es que la propia pantalla de Ajustes (T100) exponga `lifecycle.close_action` como una preferencia normal, sin ceremonia de primer cierre: cumple igual "se puede elegir... y cambiar la decisión" (US-072) sin inventar un patrón de interfaz nuevo a mitad de una historia sobre el backend de ajustes (spec 001-monitor-discos-windows, T052/T099) |
| J.20 | Con qué se implementó el toast nativo (T053), y cómo se decide cuándo notificar | **`tauri-plugin-notification` 2.0.0** (oficial del equipo de Tauri, mismo criterio que `tauri-plugin-single-instance`; usa WinRT en Windows). Se llama solo desde Rust (`NotificationExt`), nunca desde el webview, así que no necesita permiso de capacidades. Ajustado `rust-version` de `src-tauri/Cargo.toml` de `1.77` a `1.77.2` porque es el mínimo que declara el propio plugin. La decisión de notificar vive en `alerts::notificaciones` (no en `agrupacion`, que lo deja explícito en su cabecera): un episodio nuevo, una recaída o un escalado **siempre** notifican; una ocurrencia repetida respeta el cooldown por regla de `alert-rules.md` §2 (de "ninguno" en `smart.health.failed` a "7 días" en `smart.wear_high`); un grupo silenciado (`muted_until`) nunca notifica, silencio y color son cosas distintas (`ciclo.rs`). El cooldown se guarda en `AppState.notified_at` (id de grupo → instante), **en memoria, sin persistir** — igual que `paused`: perderlo al reiniciar puede como mucho volver a notificar algo ya visto, nunca dejar de notificar algo nuevo. **R1 sigue sin medirse**: si el toast llega de verdad bajo `requireAdministrator` con el identificador de aplicación registrado solo puede comprobarse al empaquetar (`research.md` R1); la alternativa ya decidida (ventana propia con `Toast`) no se ha construido, porque no tiene sentido hasta que R1 se mida y falle (spec 001-monitor-discos-windows, T053) |
| J.21 | Por qué la cronología de un grupo recién creado aparecía vacía | Bug real, no una regla nueva: `repo_alertas::create_group` solo escribía en `alert_groups`, nunca en `alert_occurrences`; `reopen_as_new_cycle` (recaída) tampoco. La primera ocurrencia de cada episodio —y la primera del ciclo nuevo tras una recaída— no tenían fila propia. Corregido: ambas funciones insertan ahora su fila en la misma transacción, y `reopen_as_new_cycle` gana un parámetro `value_real` para poder escribirla. `get_alert_detail_impl` pasó de fabricar una única ocurrencia sintética a partir del grupo a consultar `repo_alertas::list_occurrences` de verdad. Encontrado al construir la pantalla de alertas (T054), al intentar mostrar una cronología que no tenía nada real que mostrar (spec 001-monitor-discos-windows, T054) |
| J.22 | Cómo se implementó el colector de capacidad de volumen (T059), y por qué `DiskSummary.volumes` estaba siempre vacío | **Bug preexistente encontrado, no de esta tarea**: `VolumeSummary.drive_letters`/`DiskSummary.volumes` estaban declarados en el DTO pero nada los rellenaba nunca — `enrich_with_smart_data` fijaba `volumes: vec![]` a secas. Corregido con `build_volume_summaries()`, que lee `repo_inventario::volumes_for_device` + `get_volume` sin condicionarlo a que el disco tenga SMART (un disco sin SMART puede tener volúmenes). El colector en sí (`collectors::capacidad`) hace un único `Get-Partition \| Get-Volume` por PowerShell —mismo patrón que `windows_storage.rs`— porque `Get-Partition` ya sabe el `DiskNumber`, evitando correlacionar dos consultas por letra de unidad (frágil: la letra puede faltar). El enlace disco↔volumen usa esa misma numeración efímera de Windows, capturada en la misma pasada de `reconciliar_inventario` en que ya se conoce para los discos: confianza `Exact` si coincide con un dispositivo reconciliado, `Unknown` si no. Un volumen sin `UniqueId` se omite en vez de usar la letra como clave, que es justo lo inestable (spec 001-monitor-discos-windows, T059) |
| J.23 | Cómo se implementó el colector de contadores de rendimiento (T058), y un hallazgo medido sobre Windows real | Enlace FFI directo a `pdh.dll` (mismo criterio que `platform::locale.rs` con `kernel32`: sin añadir el crate `windows` completo por cinco funciones estables). **Medido en un Windows real en español**: los nombres de objeto y contador de PDH están **localizados** (`PhysicalDisk` = "Disco físico", `% Idle Time` = "% de tiempo inactivo"); `PdhAddCounterW`/`PdhExpandWildCardPathW` con una ruta en inglés fallan con `PDH_CSTATUS_NO_OBJECT` fuera de un Windows en inglés — se comprobó primero con `Get-Counter` (falla con el nombre inglés, funciona con el español) y confirmó el diagnóstico. Solución: **`PdhAddEnglishCounterW`**, que traduce el nombre **y** resuelve el comodín de instancia (`\PhysicalDisk(0 *)\...`) en la misma llamada, sin paso de expansión aparte — probado end-to-end contra dos discos físicos reales de esta máquina, con valores de actividad y latencia coherentes con su carga real en el momento de la medición. Una tasa (bytes/s, sec/operación) exige dos muestras separadas en el tiempo: se recoge dos veces con 1 s de espera entre medias, una sola vez para las cinco fuentes (no cinco esperas), y se cierra la consulta —autónoma, no persistente entre ciclos. **Actualizado tras T020**: el planificador en segundo plano ya existe y llama a esta función una vez por ciclo de `METRICAS_RAPIDAS` debido, pero sigue abriendo y cerrando su propia consulta PDH en cada llamada en vez de mantenerla abierta entre ciclos reales — eso sigue siendo una optimización pendiente, no relacionada con si el bucle existe (spec 001-monitor-discos-windows, T058) |
| J.24 | Qué hash calcula `system_events.dedup_hash` (T067) | No especificado en ningún documento más allá de "hash de deduplicación" (`data-model.md` §2). La identidad real de un evento ya es `UNIQUE(channel, record_id)`, así que este campo no decide duplicados por sí solo. Se calcula como `sha256(provider \| event_id \| occurred_at_utc \| message)`: una huella de contenido pensada para el trabajo futuro de correlación por ventana temporal de `alert-rules.md` §3.5 (un mismo suceso físico produce varios eventos correlacionados en 60 s), no usada todavía por ningún módulo de esta sesión. Provisional hasta que la correlación por ventana (§3.5) se implemente y decida si necesita este campo o algo distinto |
| J.25 | Confianza de la correlación evento→disco por número de disco (T069) según de dónde salga el número | `docs/alert-rules.md` §3.6 exige resolver contra el inventario, nunca por coincidencia textual pura, pero no distingue confianza entre las formas de identificador que "conviven" en un mismo mensaje. Decisión: **`exact`** cuando el número de disco sale de una ruta de dispositivo estructurada (`\Device\HarddiskN\...`, generada por el propio sistema en el XML crudo del evento) y coincide con un disco del inventario; **`inferred`** cuando sale del texto humano ya formateado ("disco N"/"disk N"), porque ese texto está traducido y depende de la plantilla de mensaje del proveedor, una capa menos directa que la ruta de dispositivo. `\Device\HarddiskVolumeNN` y los nombres PDO (`\Device\0003d2a5`) quedan sin resolver (`unknown`): el colector de capacidad (T059) no captura ese identificador por volumen todavía, y añadirlo es trabajo del propio colector, no de la correlación. El número que sigue a `DR` en `\Device\HarddiskN\DRxx` **nunca** se confunde con el número de disco (`alert-rules.md` §3.6, advertencia explícita) (spec 001-monitor-discos-windows, T069) |
| J.26 | Medición de R3 (T074): 20 discos y 5.000 eventos frente al umbral de 50 ms de SC-007/SC-009 | **Medido con el plano de interfaz** (Playwright + IPC propio + `PerformanceObserver` de "long tasks", que solo informa de tareas ≥50 ms). Hallazgo real durante la medición: la **primera navegación** de la prueba produce 70-120 ms de tarea larga **incluso con 0 o 2 discos** — coste fijo de evaluar el paquete en un Chromium recién arrancado, no relacionado con la cantidad de datos. Confundir ese coste con el de renderizar 20 discos habría hecho fallar la prueba por una razón ajena a SC-007 (que habla de seguir respondiendo *durante* el trabajo, no del arranque en sí). Corregido separando ambos: cada prueba dejar pasar la carga inicial y **luego** reinicia el observador, midiendo solo la interacción real — desplazar los 5.000 eventos con `VirtualList`, o recibir 20 discos en caliente vía un `metrics:updated` simulado (`ipc-falso.ts` ganó `emitirEvento()` para poder disparar ese evento desde la prueba). Ambos escenarios pasan limpios, cero tareas largas. De camino se virtualizó también el panel general (T064 ya había virtualizado la lista de eventos): la rejilla `DiskCard` pasó de pintar todas las tarjetas de una vez a virtualizarse **por fila** con el mismo `VirtualList` genérico, agrupando tantas tarjetas por fila como columnas quepan en el ancho disponible — la primera medición (antes de aislar el coste fijo de navegación) señaló la rejilla sin virtualizar como sospechosa, y aunque el diagnóstico final mostró que el problema real estaba en la metodología de medición y no en la rejilla, la virtualización quedó aplicada por ser una mejora real y ya verificada, no se revirtió (spec 001-monitor-discos-windows, T074). **Reemplazado en parte el 2026-09-06 (§U):** el panel v3 retira la `VirtualList` de la rejilla de discos —su nuevo encuadre (héroe + pie) exige una sola región de scroll— y cubre SC-006 con la variante compacta de `DiskCard` (sin sparkline a partir de 12 discos, `ui-design.md` §7); la misma prueba de rendimiento sigue verde. La virtualización de la **lista de eventos** (T064) se mantiene |
| J.27 | Nombre de la "carpeta controlada" del benchmark (T077), no especificado en ningún documento | `<raíz del volumen>\SmartDisk Monitor Benchmark\`: en la raíz del volumen que se está probando, no en `%ProgramData%` —tiene que vivir en el mismo volumen para medir su E/S real, no la del disco del sistema—, con el mismo nombre visible que ya usa la carpeta de datos (`platform::paths::data_dir()`). El nombre de archivo dentro de esa carpeta lleva un sufijo aleatorio (`benchmark-<aleatorio>.tmp`); "nunca se sobrescribe un archivo existente" (product-specification.md §6) se comprueba activamente antes de crear el archivo, no se asume por la aleatoriedad del nombre (spec 001-monitor-discos-windows, T077) |
| J.28 | Forma exacta del JSON de estado del autotest SMART corto (T081), **sin verificar contra hardware real** | A diferencia de todo lo demás de esta sesión (SMART, PDH, wevtapi, chkdsk, benchmark: todo probado contra el sistema real de esta máquina), este dato concreto **no se ha verificado**: un autotest corto real tarda minutos en el disco y el usuario pidió expresamente no ejecutarlo. `tests::autotest::parse_estado_json` asume la forma documentada de `ata_smart_data.self_test.status.{value,string,passed}` y `.polling_minutes.short` que expone `smartctl -a -j`, construida a partir de conocimiento general de su formato JSON, no de una captura propia. Antes de dar el autotest por terminado hay que lanzar uno real (cuando el usuario lo autorice) y comparar el JSON verdadero con lo que este parser espera — el mismo trato que ya se dio a `smartctl_parser.rs` con sus fixtures reales (spec 001-monitor-discos-windows, T081) |
| J.15 | Cómo distinguir "sin compatibilidad SMART" de "aún sin leer" en `get_device_detail` | Ausencia de `smartctl_path` (T025: `Get-PhysicalDisk.DeviceId` no numérico, típico de volúmenes RAID lógicos) se trata como `unsupported`; presencia de `smartctl_path` sin ninguna muestra `metric_samples.source = smartctl` se trata como `not-yet-sampled`. Deliberadamente **no** se interpreta el `exit_status` de `smartctl` como señal de soporte: sus bits documentan fallos de sintaxis/apertura/hallazgos SMART, no "este bus no expone SMART", y esa lectura no se ha podido verificar contra hardware real (`open-questions.md` I.5). Provisional hasta medir (spec 001-monitor-discos-windows, T038) |
| J.13 | Umbrales de espacio libre para detener la escritura de historial | 1 GB para el aviso y 256 MB para la parada, sobre el volumen donde reside el historial (`storage.free_space_warn_bytes` / `storage.free_space_halt_bytes`, spec 001-monitor-discos-windows). Valores de partida razonables para Windows, **no medidos**; confirmar al implementar la retención (T001, T017-T018) |
| J.14 | Cómo se representa la agregación de `metric_samples` | `docs/data-model.md` §4 exige conservar mínimo, máximo, promedio, primera y última lectura, pero el esquema solo tenía una columna de valor por fila. Se añade la tabla `metric_aggregates` (migración 0002) con `value_min/max/avg/first/last`, `bucket_start_utc`/`bucket_end_utc` y `resolution`. Los "tres periodos de retención" de US-071 son las tres resoluciones ya definidas (`raw`, `five_minutes`, `hourly`): `retention.raw_days` (7), `retention.five_minutes_days` (90), `retention.hourly_days` (730); pasado el tercero se purga. `value_last - value_first` da el incremento del bucket para contadores acumulativos, sin columna aparte. Valores por defecto, **no medidos** (spec 001-monitor-discos-windows, T015) |
| J.29 | Cómo conectar los cinco comandos de pruebas (T083): identificadores, exclusión mutua, umbral térmico y columnas sin sitio propio en `test_runs` | **Identificador de `test_run` y sufijo aleatorio del archivo del benchmark** (J.27): `format!("{:x}", OffsetDateTime::now_utc().unix_timestamp_nanos())` — nanosegundos UTC en hexadecimal, sin añadir una dependencia de aleatoriedad (mismo criterio que el LCG de T079); la unicidad real la sigue dando `rutas::confirmar_no_sobrescribe`, no la improbabilidad de colisión. **Exclusión mutua** (`test.busy`, ya previsto en `ui-contract.md` §1: "ya hay una prueba en ese disco"): se aplica por disco físico subyacente vía `device_volume_links`, no solo por el id exacto recibido — antes de arrancar cualquier prueba se comprueba que ni el objetivo ni ningún otro volumen/dispositivo del mismo disco tenga ya un `test_run` en `pending`/`running`/`cancelling`. Esto cubre a la vez la regla genérica del contrato y la regla explícita de `product-specification.md` §6 ("el autotest no se permite simultáneamente con el benchmark de la aplicación"), sin tabla de exclusión aparte. **Umbral térmico "configurado"** de `tests::guardia::limite_critico_efectivo` cuando el fabricante no lo declara (hoy siempre: `vendor_temp_critical_c` no está implementado, J.15/J.16): se reutiliza el mismo valor que ya usa el motor de alertas para `temp.above_configured_crit`, **80 °C** (`alert-rules.md`, `alerts::motor::evaluar_temperatura_configurada_crit`) — mismo concepto normativo, no un valor nuevo. **Columnas sin sitio propio**: `test_runs` (migración 0001) no tiene columna para `command`, `output` ni `outputEncoding` (`ui-contract.md` §3.6); se guardan dentro de `parameters_json` (el comando, fijado al crear la fila) y `result_summary_json` (salida y codificación, solo se conocen al terminar) en vez de abrir una migración nueva. `orphanPath` reutiliza la columna `temp_path` ya existente: mientras la prueba corre, o si el archivo no se pudo borrar al terminar, queda con la ruta; se limpia a `NULL` en cuanto el borrado tiene éxito. `volume.not_found` se añade a la tabla de códigos de `ui-contract.md` §1 en paralelo a `device.not_found`, que hasta ahora solo cubría `device_id`. **Límite conocido, no simulado**: `RazonParada::Space` (T079) solo es alcanzable como rechazo previo (`test.insufficient_space`) antes de crear la fila — `tests::benchmark::ejecutar` no comprueba espacio libre durante la ejecución (T079 solo implementó cancelación y guardia térmica), así que un agotamiento de espacio a mitad de prueba no se detecta hoy (spec 001-monitor-discos-windows, T083) |
| J.30 | Contenido exacto de la exportación tabular/estructurada (T087): ningún documento fija las columnas o campos | **CSV y JSON son el volcado completo**, una fila/objeto por `(dispositivo, metric_key, marca de tiempo)`: `schemaVersion, deviceId, deviceLabel, metricKey, unit, resolution, timestampUtc, value`. Qué métricas incluir no es una lista fija: `repo_metricas::distinct_metric_keys` devuelve las que de verdad tengan dato del dispositivo en el rango (crudo o agregado), para no inventar columnas vacías ni olvidar una real. **Resolución por rango**, igual que ya hace `get_metric_series_impl` para las gráficas (crudo ≤24 h y dentro de los últimos 7 días; `five_minutes` ≤7 días; `hourly` con reserva a `five_minutes` ≤90 días; `hourly` más allá) — implementada de nuevo en `reporting/export.rs`, sin tocar la función existente de `commands/mod.rs`, para no arriesgar una regresión en la gráfica por una necesidad distinta (el `value` de una fila agregada es `value_avg` con reserva a `value_last`, igual que ya hace `leer_agregados_dispositivo`). **HTML es un resumen legible, no el mismo volcado**: identidad y salud actual del dispositivo más las alertas que se dispararon en el rango — la especificación solo exige que sea "legible e imprimible" (`product-specification.md` §9), no que reproduzca miles de filas; quien necesite el detalle completo tiene el CSV o el JSON. **Dato ausente**: `null` en JSON, cadena literal `"N/A"` en CSV — nunca vacío ni cero, mismo criterio que el resto de la aplicación. `deviceIds: null` en el comando significa todos los dispositivos monitorizados (no los excluidos). **Sin evento de progreso nuevo**: `docs/ui-contract.md` §4 no define uno para exportar, y el escenario de aceptación ("la aplicación sigue respondiendo y muestra progreso", Historia 6 §spec) queda cubierto por la propia naturaleza asíncrona del `invoke` (la interfaz no se bloquea) más un indicador indeterminado local mientras se espera la respuesta — no hace falta inventar `export:progress` para una operación que no tiene fases intermedias que reportar. **Las alertas del resumen HTML muestran `ruleKey` tal cual** (p. ej. `smart.wear_high`), no una frase humana: ADR-030 decidió que el backend nunca manda texto de alerta, solo la clave, y este HTML lo genera el propio backend sin acceso a los diccionarios de `$lib/i18n` — duplicar ahí una traducción sería una segunda copia sin mantener, exactamente lo que ADR-030 quiso evitar (spec 001-monitor-discos-windows, T087/T088) |
| J.31 | Contenido y disposición del ZIP de diagnóstico (T090): ningún documento fija los ficheros que lleva dentro | **`smart_snapshots.raw_json_path`/`fields_json` están sin usar**: ningún colector escribe hoy el JSON crudo de `smartctl` a disco ni a la base (`insert_smart_snapshot` siempre los llama con `None`, T036). El ZIP no puede leer un archivo que no existe, así que **vuelve a consultar `smartctl` en el momento de generarlo** (`collectors::smartctl::query_device_json`, ya verificado contra hardware real esta sesión) para cada dispositivo con `smartctl_path` — un diagnóstico fresco, no uno reconstruido de una captura que nunca se guardó. **Disposición dentro del ZIP**: `manifest.json` (schemaVersion, generatedAtUtc, versión de la app, `anonymized`, `redactedFields`), `settings.json` (`repo_varios::list_settings`, todas las claves), `events.json` (`repo_varios::list_events` sin filtro, límite alto en vez de paginado: es un volcado, no una pantalla), `smart/<deviceId>.json` (o `smart/<deviceId>.error.txt` si la consulta falla — un fallo de un disco no debe tirar el paquete entero), `logs/<nombre-de-fichero>` (todo lo que haya en `platform::paths::log_dir()`, tal cual lo escribe `tracing_appender::rolling::daily`, FR-029c). **Cada entrada de texto pasa por el mismo `Anonimizador`** antes de escribirse — de ahí que la sustitución sea consistente en todo el paquete (US-051): el mismo número de serie se convierte en el mismo `<SERIE-N>` tanto en `smart/*.json` como en `logs/*` si apareciera ahí. `includeIdentifiers: true` construye un `Anonimizador::sin_anonimizar()`: nada se sustituye, y `redactedFields` viaja vacío en el manifiesto (spec 001-monitor-discos-windows, T090) |
| J.32 | Forma completa de `Settings` (T095/T096): `ui-contract.md` §3.1 nombra `get_settings`/`set_setting`/`reset_settings` pero nunca escribe la interfaz — ningún documento reúne en un solo sitio todos los campos configurables que ya estaban dispersos (D.1, C.1, J.13, J.14) | Cuatro grupos, alineados con los cuatro valores de `reset_settings({scope})`: **`schedule`** (`metricsFastSeconds`/`smartFullSeconds`/`eventsSeconds`/`discoverySeconds`) reutiliza tal cual los límites ya codificados en `collectors::planificador::{METRICAS_RAPIDAS,SMART_COMPLETO,EVENTOS_WINDOWS,ALTAS_Y_BAJAS}` (D.1) — ese módulo ya decía en su propio comentario "esto lo hace `domain::ajustes`, no este módulo", así que no son límites nuevos, son los que ya existían sin consumidor. **`alerts`**: `tempConfiguredWarnC`/`tempConfiguredCritC` (por defecto 70/80, los mismos literales que hoy tiene hardcodeados `alerts::motor` para `temp.above_configured_warn/crit`; límites nuevos, no medidos: 40-95 °C para el aviso, el crítico entre el aviso y 100 °C) y los cinco campos de capacidad ya decididos en C.1/ADR-019 (`capacityWarnPercent` 10, `capacityCritPercent` 5, `capacityAbsoluteFloorMinCapacityBytes` 256 GiB, `capacityAbsoluteFloorWarnBytes` 20 GiB, `capacityAbsoluteFloorCritBytes` 10 GiB — los mismos valores que ya usa `capacityState()` en `src/lib/design/health.ts`, hoy con el suelo fijo en una constante en vez de leído de `settings`). **`retention`**: los tres periodos de J.14 (7/90/730 días, límites nuevos y razonables: crudo 1-30, cinco minutos 7-365, horario 90-1825) más `storage.free_space_warn_bytes`/`halt_bytes` de J.13 (1 GiB/256 MiB, sin límites de UI porque US-071 solo pide poder cambiar los tres periodos, no estos dos bytes). **Fuera de estos tres grupos** (solo se restauran con `scope: "all"`): `lifecycle.closeAction` (`"minimize"` por defecto, `docs/open-questions.md` J.19 seguía abierta y este valor la cierra: minimizar es "el lado seguro" ya razonado en `lib.rs`) y `closeActionRemembered`, `notifications.soundEnabled` (`false` de fábrica, US-072), `logging.verbose` (ya nombrada en `data-model.md`). **Apariencia no vive en `Settings`**: `theme`/`language`/`useSystemAccent` siguen teniendo su propio `get_appearance_settings()` ya construido; se persisten con el mismo `set_setting(key, value)` genérico (`theme.svelte.ts`/`i18n.svelte.ts` ya devuelven `{key: "settings.appearance.theme"/"settings.appearance.language", value}` a la espera de un consumidor, que es exactamente lo que T097 les da). **Límite conocido, no ampliado por esta historia**: ni `domain::espacio` (guardia de espacio del historial) ni `capacityState()` ni ninguna regla `capacity.low/critical` en el motor de alertas leen hoy estos valores de `settings` en un ciclo real — no existe todavía el bucle de recopilación en producción que los invoque (ninguna tarea de esta historia lo pide); esta historia deja el valor correctamente guardado y validado, listo para cuando ese consumidor exista, igual que ya pasaba con `logging.verbose` antes de FR-029a (spec 001-monitor-discos-windows, T095/T096) |
| J.33 | Comprobación de "cero peticiones salientes" (T107, SC-014, `quickstart.md` eslabón 10) — el guion exige "instalar en un equipo sin conexión" y "verificar con un monitor de red", que requiere un instalador real construido, instalado y en ejecución bajo un monitor de paquetes: no ejecutado esta sesión | **Auditoría estática, no medición en vivo** — mismo trato honesto que J.28 (autotest SMART): `src-tauri/Cargo.toml` no declara ningún cliente HTTP (`reqwest`/`hyper`/`ureq`) entre sus dependencias directas; `cargo tree --target x86_64-pc-windows-msvc -e normal` confirma que **ni siquiera aparecen como transitivas** en el árbol real de Windows — sí figuran en `Cargo.lock` (`reqwest`, `hyper`, `tokio`), pero por una dependencia opcional de `tauri` que el propio manifiesto de `tauri` acota a `cfg(target_os = "android", ...apple...)`: nunca se compilan para Windows. Búsqueda en todo `src-tauri/src`: cero usos de `std::net`, `TcpStream`, `UdpSocket` o una URL `http(s)://` real (las únicas coincidencias son un espacio de nombres XML dentro de una fixture de evento de Windows capturada, y las propias aserciones de test que comprueban que el HTML exportado *no* contiene ninguna). En el frontend: cero `fetch`/`XMLHttpRequest`/`WebSocket`/`EventSource` en todo `src/`; `package.json` solo depende de `@tauri-apps/api`, `@tauri-apps/plugin-dialog` y `zod`, ninguno de red. La CSP de `tauri.conf.json` cierra en profundidad: `connect-src 'self' ipc: http://ipc.localhost`, sin ningún origen remoto permitido aunque algo lo intentara. **Pendiente de la medición real** que el guion pide: construir el instalador (`pnpm app:build`), instalarlo en una máquina sin red y confirmar con un monitor de paquetes (Wireshark o similar) que no sale ni un byte — requiere una acción invasiva (instalación elevada de un binario real en el sistema) que esta sesión no ha ejecutado sin autorización explícita (spec 001-monitor-discos-windows, T107) |
| J.34 | Cadencia de sondeo del bucle en segundo plano (T020): ningún documento fija con qué frecuencia el hilo comprueba si algún trabajo ya toca | **1 segundo**, no el intervalo real de cada trabajo. Sondear con un período fijo corto (en vez de dormir el intervalo completo de la próxima tarea) es lo que permite que pausar, cambiar una frecuencia en Ajustes o cerrar la aplicación reaccionen con un retardo máximo de 1 s, en vez de hasta 1 hora (el máximo configurable de SMART completo). El coste de sondear cada segundo con cuatro comparaciones de `Instant` es insignificante frente al beneficio de reactividad; no hay medición que lo respalde porque no hay nada que medir — es un valor de diseño, no un dato empírico (spec 001-monitor-discos-windows, T020) |
| J.35 | Qué dispara `inventory:changed` y qué lleva su campo `updated` (T021): el esquema Zod ya tiene `added`/`removed`/`updated`, pero `ui-contract.md` §4 solo documenta el disparador como "alta o retirada de disco o volumen" | **`updated` viaja siempre `[]`**. El contrato tal cual está escrito no pide detectar cambios de campo en un disco que sigue presente (alias, modelo, capacidades) como disparador de este evento — inventar un diff campo a campo sin que ninguna historia lo pida sería anticipar un requisito que no existe. Si en el futuro se necesita, es una decisión de una historia con su propio criterio de qué cuenta como "cambio relevante" (spec 001-monitor-discos-windows, T021) |
| J.36 | Qué hacer cuando `GetSystemPowerStatus` o `GetDiskFreeSpaceExW` fallan (T020): ninguna de las dos syscalls está garantizada a tener éxito, y ningún documento dice qué asumir si fallan | **Batería**: `ACLineStatus` fuera de `{0, 1}` (incluido `255`, "desconocido", y cualquier error de la llamada) se trata como **red eléctrica** — es el lado que menos reduce la frecuencia de recopilación y menos sorprende si en realidad el equipo funciona con batería (peor caso: se recopila un poco más de lo estrictamente necesario, nunca menos de lo que hace falta para una alerta grave). **Espacio libre**: si `GetDiskFreeSpaceExW` falla, se trata como `EstadoEspacio::Normal` (no se detiene la escritura de historial) y se registra un `tracing::warn!`: un dato desconocido no equivale a "disco lleno", mismo principio que "no compatible ≠ averiado" aplicado aquí a un fallo de sistema en vez de a un disco (spec 001-monitor-discos-windows, T020) |
| J.37 | Cómo se agrega `SourceHealth` por ciclo (T020/T021): `SourceStatus` tiene cinco valores (`ok`/`partial`/`unsupported`/`timeout`/`error`) pero ningún documento dice cuándo usar cada uno, ni con qué granularidad se mide (¿por disco?, ¿por colector?) | El propio struct `SourceHealth` es por **tipo de colector** (`source: MetricSource`, cuatro variantes), no por disco. Se agrega **por ciclo**: cada llamada a un colector externo (`smartctl::query_device_json`, `perf_counters::leer`, `windows_storage::list_physical_disks`/`list_volumes`) cuenta como un intento; al cerrar el ciclo se compara el total de intentos contra los que fallaron. **Alcance de esta implementación**: solo se distinguen `ok` (todos los intentos de ese colector tuvieron éxito) de `timeout`/`error` (al menos uno falló; `timeout` si el `AppError` resultante es `retryable`, `error` si no) — **`partial` y `unsupported` no se sintetizan todavía**: `partial` exigiría decidir un umbral de qué proporción de fallos ya cuenta como degradación parcial frente a total, que nadie ha pedido; `unsupported` exigiría saber que un colector no tiene ningún dispositivo elegible, que no es lo mismo que haber fallado. Si un colector no se invoca en absoluto durante un ciclo (cero dispositivos elegibles), su entrada en `source_health` se deja tal cual estaba, nunca se inventa un valor. `Filesystem` (la cuarta variante de `MetricSource`) no tiene todavía ningún colector que la produzca (no hay muestras `filesystem` en `persistence::repo_metricas`); se queda sin entrada hasta que exista. `source:degraded` solo se emite en el **flanco** de subida a `timeout`/`error` desde cualquier otro estado, nunca en cada ciclo que siga degradado (spec 001-monitor-discos-windows, T020/T021) |
| J.38 | `refresh_smart` mezclaba SMART y contadores de rendimiento en un único bucle por disco (T058); el bucle en segundo plano necesita dos cadencias independientes (`SMART_COMPLETO` y `METRICAS_RAPIDAS`) | **Se separa en dos funciones**: `refresh_smart` (solo SMART, evalúa alertas) y `refresh_metricas_rendimiento` (solo PDH, sin alertas). Antes de esta historia estaban acopladas porque nada las llamaba con cadencias distintas — `refresh_now` las invocaba juntas una sola vez—; el bucle en segundo plano sí necesita invocarlas por separado (30 s frente a 5 min por defecto), así que mantenerlas juntas habría hecho que el SMART completo se ejecutara cada 30 s en vez de cada 5 min, vaciando de sentido `SMART_COMPLETO`. `refresh_now` pasa a llamar a las dos, una tras otra, para conservar exactamente su comportamiento manual de antes ("actualizar ahora" sigue refrescando ambas cosas de golpe) (spec 001-monitor-discos-windows, T020) |
| J.39 | Cómo comparten lógica `refresh_now` (comando manual) y el bucle en segundo plano, sin que el fallo de un trabajo bloquee a los demás en el bucle | **Orquestación separada, post-proceso compartido**. `refresh_now` conserva su semántica de siempre (una operación falla y se aborta esa llamada entera, tal como ya esperan sus pruebas y el guion manual "actualizar ahora"). El bucle en segundo plano (`ejecutar_ciclo`) trata cada trabajo debido de forma independiente: si `AltasYBajas` falla, `EventosWindows`/`SmartCompleto`/`MetricasRapidas` igualmente debidos en el mismo sondeo se siguen ejecutando — son dominios de fallo distintos (un fallo de enumeración de discos no tiene por qué impedir una lectura SMART ya en curso de otro disco), y un bucle autónomo que se bloquea entero por un fallo ajeno sería peor que uno que registra el fallo y sigue. Ambas vías comparten la función `post_procesar_ciclo` para el post-proceso común (notificaciones, `alerts:changed`, `metrics:updated`, `inventory:changed`, `source:degraded`, icono de bandeja): es la parte que sí debe comportarse igual venga de donde venga, y compartirla es lo que evita que diverjan en silencio (spec 001-monitor-discos-windows, T020/T021) |
| J.40 | `metrics:updated.historyWriteHalted` necesita un valor real de la guardia de espacio (T018, ya implementada mas nunca conectada); ¿se aprovecha también para *detener* la escritura, o solo para *informarla*? | **Solo para informarla, esta sesión**. Se calcula de verdad (`platform::energia::espacio_libre_bytes` sobre `%ProgramData%` + `domain::espacio::UmbralesEspacio` con los umbrales de `settings`), así que el campo no miente. **No se ha conectado a ningún punto de escritura** (`persist_smart_reading`, `persist_perf_reading`, `refresh_events`): `evaluar_smart` decide activación/histéresis releyendo `metric_samples` recién persistidas, así que saltarse la escritura sin más dejaría a las alertas sin la lectura que necesitan evaluar — contradiciendo FR-020a ("la vigilancia y las alertas en vivo no dejan de funcionar"). Distinguir qué parte de la escritura debe seguir (la que alimenta alertas) de cuál debe detenerse (el historial de tendencias a largo plazo) exige mirar con cuidado `repo_metricas`/`evaluar_smart`, y no es prudente improvisarlo dentro de esta historia ya grande. Queda como tarea explícita de seguimiento, no como olvido (spec 001-monitor-discos-windows, T020) |
| J.41 | T111 (escalado 125/150/200 %) encontró que `ui-design.md` §4.0.bis (Sidebar a 56 px por debajo de 1180 px) nunca se implementó, y causaba recortes reales de texto ("Panel gene...", "No" en vez de "No disponible") — pero la norma pide "iconos" y ni el boceto aprobado ni el catálogo de componentes definen ninguno para las seis secciones | **Marcador circular con la inicial de cada sección** (P/A/E/P/I/A), como paso intermedio autorizado explícitamente por el usuario tras plantear la disyuntiva (mismo criterio que bloqueó el asistente inicial, T041, por el motivo contrario). No es iconografía nueva: reutiliza la tipografía y el `rounded-pill`/`bg-glass-3` ya existentes, con el nombre completo como `aria-label`/`title` del enlace — el texto oculto no deja de ser accesible. El pie de pausar/reanudar se oculta por completo bajo 1180 px: la misma acción sigue disponible desde el menú de la bandeja (`platform::bandeja`), así que no hay pérdida funcional, solo de acceso redundante. Verificado con capturas en las siete pantallas reales a 1024×560/1280×720/819×448/683×373/512×280 (`e2e/ui/escalado.spec.ts`); sustituir el marcador por iconos reales sigue abierto para cuando haya una revisión de diseño (spec 001-monitor-discos-windows, T111) |
| J.42 | Con el planificador ya arrancando de verdad contra hardware real (T020), `smartctl` fallaba con "Unable to detect device type" en los cuatro modos de la cascada, en los cuatro discos físicos de esta máquina (2 SATA, 2 NVMe) — medido por primera vez, `open-questions.md` I.5 seguía sin datos de hardware real | **El formato de ruta era el equivocado, no un problema de compatibilidad de disco ni de elevación**. `windows_storage::list_physical_disks` construye `smartctl_device_path` como `\\.\PhysicalDriveN` (la ruta nativa de Windows para `CreateFileW`), pero el `smartctl.exe` redistribuido (compilación MinGW, `x86_64-w64-mingw32-w11-b26200`) espera su propia convención POSIX: `smartctl -h` lo documenta explícitamente (`smartctl -a /dev/pd3` → "Prints all information for disk on PhysicalDrive 3"). Verificado a mano contra los cuatro discos reales: `\\.\PhysicalDriveN` falla siempre ("Unable to detect device type" en autodetección, "Invalid argument" en cada modo de `-d` explícito, para SATA **y** NVMe por igual, elevado o no); `/dev/pdN` funciona a la primera en los cuatro, con `model_name` y atributos SMART reales. Corregido cambiando la construcción de la ruta en `windows_storage.rs` y su análisis inverso en `disk_number_from_smartctl_path` (`commands/mod.rs`, usado para los contadores de rendimiento PDH, que sí siguen tomando el número de disco de Windows, no la ruta de `smartctl`). Cierra la parte de I.5 que bloqueaba cualquier lectura SMART en absoluto; la cascada de modos para puentes USB/RAID exóticos sigue abierta tal como I.5 ya la planteaba, pero ahora al menos parte de una ruta que sí abre el dispositivo (spec 001-monitor-discos-windows, T058/US-010) |
| J.43 | `docs/ui-design.md` ya exige "Movimiento: duration-base (220 ms) con ease-sdm... en... cambio de pantalla", pero ningún cambio de sección lo tenía — usuario lo notó al usar la aplicación de verdad por primera vez: "todo aparece de golpe". La norma dice cuánto dura y con qué curva, no qué efecto visual usar | **Entrada con desvanecimiento y una leve subida** (`opacity 0→1`, `translateY(4px)→0`), sin animación de salida — mismo criterio que ya usan `ConfirmDialog`/`Toast` (una sola animación de entrada vía `@keyframes` + `var(--sdm-duration-base)`/`var(--sdm-ease)`, nunca JS). Se dispara con `{#key}` en `AppShell.svelte`, con la clave siendo `page.url.pathname` (pasada desde `+layout.svelte`): cambia de sección → remonta el contenido → repite la animación; cambia solo un parámetro dentro de la misma pantalla (un filtro, una página del listado) → no remonta, no hay parpadeo innecesario. Una animación cruzada (fundido simultáneo de la pantalla saliente y la entrante) se descartó por ser más compleja sin que la norma la pida, y por arriesgar un parpadeo si ambas pantallas comparten elementos con el mismo punto de foco. `prefers-reduced-motion` la anula igual que a `sdm-dialog`/`sdm-toast`, sin código adicional: es la misma regla global de `tokens.css` que ya vigila `animation-duration` (spec 001-monitor-discos-windows) |
| J.44 | J.19 ya dejaba a propósito sin construir el diálogo "¿minimizar o salir? [ ] recordar" de la primera vez que se cierra la ventana; usando la aplicación real, el usuario preguntó si el minimizado silencioso (sin aviso alguno) estaba bien — no lo estaba: la ventana desaparece sin ninguna señal de que sigue vigilando | **Aviso nativo, una sola vez por arranque del proceso**, no el diálogo con casilla de recordar que J.19 seguía dejando pendiente (eso sigue exigiendo un componente y un evento nuevos, fuera de alcance de un cambio pequeño). Al minimizar por primera vez en la sesión, se muestra una notificación de Windows ("SmartDisk Monitor sigue activo... clic para reabrir, o Salir para cerrarla del todo"), reusando el mismo `tauri-plugin-notification` que ya usan las alertas — nada nuevo que aprobar. Un `AtomicBool` en `AppState` (`aviso_bandeja_mostrado`, en memoria, no persistido: cada arranque nuevo vuelve a avisar una vez) evita repetirlo en cada minimizado posterior de la misma sesión, que sería ruido. El "Salir" del menú de la bandeja sigue sin pedir confirmación (una acción explícita de menú no la necesita) (spec 001-monitor-discos-windows, US-072/T099) |
| J.45 | El usuario pidió que la aplicación "se comporte más como una aplicación nativa de Windows": nada de selección de texto libre ni del cursor de I en cualquier etiqueta, como en una página web — pero acotó él mismo el alcance: "el texto que tenga sentido seleccionar y copiar lo vamos a dejar disponible". Qué cuenta como "tiene sentido copiar" no estaba escrito en ningún sitio | **`user-select: none` global en `body`** (`tokens.css`), reactivado solo en el contenido que ya llevaba una marca semántica de "es un valor o un dato técnico": `.sdm-num` (cifras de métrica y contador), `.font-mono`/`code`/`pre` (comando literal de `ConfirmDialog`, salida de `CodeOutput`, detalle técnico de `EmptyState`, XML crudo de un evento) y los campos de formulario (`input`/`textarea`/`contenteditable`, que gestionan su propia selección nativa). No se tocó `cursor`: basta con `user-select: none` para que el navegador deje de mostrar el cursor de texto sobre lo no seleccionable (solo lo muestra sobre contenido seleccionable), sin arriesgar el cursor de mano de enlaces y botones. Se añadió además una clase de escape explícita, `.sdm-selectable`, para marcar caso a caso contenido identificador que no encaja en las categorías anteriores — usada en el modelo/alias del disco (`DiskCard.svelte`, cabecera de `disks/[id]/+page.svelte`), pensado para buscar el modelo exacto o compararlo con la documentación del fabricante. El número de serie no se muestra todavía en ninguna pantalla (`DeviceDetail.serialNumber` existe en el contrato pero no se renderiza); cuando se añada, debe llevar `.sdm-selectable` o una de las clases ya cubiertas |
| J.46 | El usuario notó que el gráfico de temperatura en tiempo real del detalle de disco no indica si un valor (p. ej. 52 °C frente a 100 °C) es bueno o peligroso, y eligió explícitamente la opción más completa entre las dos planteadas: zonas de fondo coloreadas, no solo una línea de umbral | **Dos zonas de fondo** en `TimeSeriesChart.svelte` (`warnThreshold`/`critThreshold`, sustituyendo el `threshold`/`thresholdLabel` que existía pero nunca se conectó desde ninguna pantalla): crítica desde el techo del gráfico hasta el umbral crítico, aviso desde ahí hasta el umbral de aviso, con `--sdm-warn-soft`/`--sdm-crit-soft` (ya usados en otras insignias, nunca un color nuevo) y sin superponerse entre sí. Los umbrales se calculan una sola vez por pantalla (`temperatureThresholds()` en `design/health.ts`, nueva) con la misma precedencia que `alert-rules.md` documenta para `temp.above_vendor_limit`/`_critical`/`temp.above_configured_warn`/`_crit` — el límite del fabricante manda si `smartctl` lo declaró (hoy nunca lo declara, I.5 sigue abierta), si no el configurado en Ajustes — y se reutilizan tal cual para colorear también la cifra grande de `MetricCard` (antes sin color: mismo defecto que el usuario señaló, pero en el número, no solo en el gráfico), así las dos lecturas del mismo dato en la misma pantalla no pueden discrepar entre sí. Ver K.7: el motor de alertas real todavía no lee el umbral configurado, solo el literal 70/80 °C — discrepancia ya registrada, no corregida aquí |
| J.47 | **DECIDIDO** e implementado (spec `003-puente-eventos-alertas`, research.md D2). La tabla de `alert-rules.md` §2 da como fuente de `device.removed_unexpected` «inventario + `disk` 157», sin decir cómo se distingue una expulsión limpia de una imprevista cuando el disparador es la desaparición del inventario. Windows **no** deja un rastro fiable de expulsión ordenada (no hay un id de evento equivalente al 157 para el caso bueno) | La regla se activa por **cualquiera** de: (a) un evento `disk` 157 correlacionado con un disco del inventario; (b) un disco monitorizado **no USB** que desaparece del inventario —un disco fijo no desaparece en operación normal—. Un disco USB que desaparece **sin** `disk` 157 se trata como retirada esperada y no alerta. Severidad crítica salvo `bus_type == "USB"` → advertencia. Resolución: reaparece el mismo `fingerprint`. La ventana de correlación de 60 s (J.49) evita el doble grupo cuando ambas vías se disparan por el mismo suceso |
| J.48 | **DECIDIDO** e implementado (spec `003`, research.md D3). Dónde vive la tabla `(proveedor, id) → regla` que implementa `alert-rules.md` §3.2: `settings` (mutable sin recompilar, como `PROVEEDORES_VIGILADOS`) o código | **En código**, `src-tauri/src/alerts/reglas_eventos.rs`, con una prueba de completitud y fidelidad contra `alert-rules.md` §3.2/§3.3. La lista de *proveedores* vigilados (filtro de ingesta) se queda en `event_log.rs` y sí puede ir a `settings`; la *semántica* de una regla (severidad, resolución, contexto de dedup) es dominio normativo y no la edita el usuario |
| J.49 | **DECIDIDO** e implementado (spec `003`, research.md D4). La ventana de correlación de ráfaga de `alert-rules.md` §3.5 es tiempo de reloj (60 s), pero el colector de eventos va a lotes cada 30 s: una ráfaga puede quedar partida entre dos ciclos | La correlación **no** se hace solo sobre el lote del ciclo: al evaluar cada evento nuevo se consulta `system_events` los eventos del mismo disco en los 60 s anteriores, ya persistidos. Si el `disk` 157 llegó en un ciclo anterior y ya creó su grupo, el derivado del ciclo actual se registra como ocurrencia suya. Si el `disk` 157 llega **después** que un derivado ya agrupado, esos grupos derivados se resuelven con nota de «absorbido por la extracción imprevista» y sus eventos se re-registran bajo `device.removed_unexpected` (caso poco frecuente, con prueba propia) |
| J.50 | **DECIDIDO** e implementado (spec `003`, research.md D5). Cómo llega al detalle de una alerta de evento el suceso que la disparó | `alert_occurrences.triggering_event_id` (columna que ya existe en el esquema, hoy nunca escrita) se empieza a rellenar. El DTO de ocurrencia gana `triggeringEventId: number \| null`; el detalle de alerta muestra, para las filas que lo tengan, un enlace `<a href="/events?focus=<id>">` a la pantalla de sucesos, que ya renderiza el XML crudo y la etiqueta de certeza. No se duplica el contenido del evento dentro del detalle de alerta |
| J.51 | **DECIDIDO** e implementado (spec `003`, research.md D6). Alcance de la primera activación del puente de eventos: ¿evalúa los eventos ya ingeridos con anterioridad? (clarify Q1 → «solo hacia delante») | **Sin marcador de corte nuevo.** El puente evalúa solo los eventos que `repo_varios::insert_event_if_new` devuelve como nuevos (`Ok(true)`) en ese ciclo. Los eventos ya presentes en `system_events` al desplegar nunca se re-leen (el bookmark del canal está por delante) y, si el bookmark se invalidara y el canal se releyera entero, `insert_event_if_new` devuelve `Ok(false)` para los conocidos → no se evalúan. El «punto de corte» lo da el bookmark existente (J.7) más la unicidad `(channel, record_id)` |
| J.52 | **DECIDIDO** e implementado (spec `004-navegacion-sin-congelacion`, ADR-042). Usando la aplicación real, el usuario reportó que casi siempre que cambiaba de sección en el sidebar la interfaz se congelaba varios segundos y parecía colgada. Causa: el bucle de recopilación retenía el mutex de `AppState.conn` mientras lanzaba `smartctl.exe` (cascada de hasta 75 s/disco), dormía entre muestras PDH y leía el registro de eventos; cualquier `load` de ruta que hiciera `conn.lock()` esperaba todo ese tiempo. J.39 solo cubría el reparto de dominios de fallo, no la retención del candado | `refresh_smart`, `refresh_metricas_rendimiento` y `refresh_events` pasan a **tres fases**: candado breve para planificar → E/S externa **sin candado** → candado único para persistir. Guarda `AppState.recoleccion_smart: Mutex<()>` para que el refresco manual siga esperando al ciclo en curso sin retener `conn`. `refresh_inventory` ya cumplía y no se toca. Como red de seguridad —no como sustituto de que la interfaz responda al instante— `AppShell` pinta una barra de progreso fina arriba mientras `navigating` sea no nulo, con retardo de 150 ms para no parpadear. Descartada una 2ª conexión de solo lectura: abre `SQLITE_BUSY` real y no arregla el lado escritor (`set_setting`, `acknowledge_alert`) |
| J.53 | **DECIDIDO** e implementado. Usando la aplicación real, el usuario notó que el icono de la bandeja era un cuadrado de color liso (indistinguible a 16 px de otras aplicaciones) y que el texto emergente decía solo «Todo en orden», sin nombrar a qué aplicación pertenece. `decisions.md` (línea 106) ya marcaba el systray como pantalla sin revisión visual; esto es un primer paso, no el rediseño fino | El icono pasa a un **tile redondeado del color de estado B.5 + un glifo que también cambia con el estado**: cilindro de datos lleno (todo en orden), con «!» (advertencia), con «×» (crítico), hueco (sin datos / sin discos / fallo de recopilador), dos barras (en pausa). Así el color no es el único portador de significado (constitución §VII) y se distingue a 16 px. Se sigue generando en memoria (búfer RGBA supermuestreado 4×, dibujo procedural, sin biblioteca ni fichero `.ico`). El texto emergente pasa a `«SmartDisk Monitor — <resumen>»` (clave `tray.tooltip`), aplicado en `instalar()` y `actualizar()`. Ayuda de QA: `cargo test volcar_iconos_bmp -- --ignored` vuelca los cinco iconos a `src-tauri/target/bandeja/`. El rediseño visual completo del systray (y del resto de pantallas del Apéndice C de `ui-design.md`) sigue abierto |
| J.54 | **DECIDIDO** e implementado. Usando la aplicación real, el usuario notó que el icono de arriba del riel (logo de marca) y el primero de la navegación («Panel general») usan el mismo icono (`diskStack`) y **llevan los dos a `/`** — redundante—, y que el logo tiene un fallo de hover (la regla global `a:hover { color: --sdm-accent-fg }` teñía de violeta el icono blanco sobre el degradado de acento, que quedaba como un cuadrado). El boceto `design/.../Sidebar.md` sí dibujaba un logo aparte | **Se quita el logo.** En un riel de solo iconos no aporta: «Panel general» ya va a `/` con el icono del disco, y la identidad de la app está en la barra de título y en «Acerca de». Con ello desaparece también el fallo de hover. `docs/ui-design.md` no menciona el logo (solo «solo iconos con `title`+`aria-label`»), así que no hay que tocarlo; el boceto de `design/` no es normativo (AGENTS.md §«Fuentes de verdad»). La clave i18n `app.name` se conserva (la usan la bandeja y «Acerca de») |

---

### K. Pendiente de decisión

Cerradas desde la última revisión:

- **K.1** (tipografía empotrada), 2026-09-04 — los dos `.woff2` de Instrument Sans v4 y su `OFL.txt`
  están en `src/design-system/fonts/`, declarados en `tokens.css` con `unicode-range` y
  registrados con sus hashes en `THIRD_PARTY_NOTICES.md`.
- **K.4** (escala tipográfica y escalado de Windows), 2026-09-04 — medido; véase §L.
- **I.1** (WebView2 en Windows Server), 2026-09-04 — resuelto con documentación oficial; véase §M.
- **K.2 y K.3** (versión de smartctl y cumplimiento de la GPLv2), 2026-09-04 — binario y fuente ya
  en el repositorio, verificados; véase §N.
- **I.4** (contraste del acento heredado), 2026-09-04 — medido sobre 262.144 colores; véase §O.
- **K.5** (eventos de Windows), 2026-09-04 — lista verificada contra manifiestos y 180 días de
  registro real; véase §P. Queda pendiente el contraste en servidor.
- **I.3** (codificación de los procesos auxiliares), 2026-09-04 — medido; la suposición de la
  especificación era incorrecta. Véase §Q.
- **K.6** (cobertura de «resto de `src-tauri/src/`»), 2026-09-06 — el usuario eligió la vía (a):
  enmendar el principio VIII (constitución 1.6.0) en vez de aceptar el 80 % como excepción
  permanente. Medido de nuevo en esta fecha con `cargo llvm-cov`, ya con todo lo añadido desde
  T108 (planificador, ajustes, informes): **69,80 %** — el déficit había bajado de 72,50 % porque
  `commands/mod.rs` creció mucho más rápido que sus pruebas. Se descartó separar los envoltorios
  `#[tauri::command]` a un fichero excluido de la medición al comprobar, comando a comando, que
  la premisa no se cumplía para unos 20 de los 35: tienen lógica real escrita directamente en el
  comando, sin ningún `_impl` que la recoja (`pause_monitoring`/`resume_monitoring`,
  `acknowledge_alert`/`mute_alert`/`unmute_alert`/`archive_alert`, `refresh_now`,
  `run_chkdsk_scan`/`run_smart_short_test`/`cancel_test`/`start_benchmark`…). Excluirlos tal cual
  habría escondido lógica sin probar detrás de la excepción, no solo pegamento no instanciable.
  El mínimo de esa fila baja a **69 %** (con margen sobre el 69,80 % medido) en vez de mantener
  una excepción de dos casos que no encajaba con el código real; extraer esos ~20 comandos a sus
  propias funciones `_impl` con prueba —lo que sí permitiría separar y excluir el envoltorio de
  verdad— queda como mejora futura, no bloqueante, y subiría el mínimo de nuevo cuando se haga.


| # | Cuestión | Por qué no se ha decidido |
|---|---|---|
| K.7 | Implementando J.46 (zonas de aviso/crítico en el gráfico de temperatura) se encontró que `alerts::motor::evaluar_temperatura_configurada_warn`/`_crit` llevan los umbrales **literales** (`70.0`/`80.0`/`67.0`/`75.0`) en vez de recibir `AlertSettings.temp_configured_warn_c`/`_crit_c` — los mismos campos que `settings.alerts` ya persiste y que la pantalla de Ajustes ya deja editar (`settings.alerts.tempWarn`/`tempCrit`). Cambiar el ajuste en la interfaz no tiene ningún efecto sobre qué alertas se disparan de verdad | **No se ha tocado el motor de alertas en esta tarea**: era un cambio de comportamiento de alertas ya en producción, fuera del alcance autorizado (una mejora de UX en el gráfico de temperatura), y la constitución exige tratar un cambio de comportamiento observable como historia propia, no colarlo dentro de otra. El frontend (`disks/[id]/+page.svelte`, `src/lib/design/health.ts::temperatureThresholds`) sí lee `settings.alerts.tempConfiguredWarnC/CritC` para las zonas del gráfico y el color de la cifra grande, que es el comportamiento **documentado y pretendido** (`alert-rules.md`, columna "configured"): así, en cuanto se corrija el motor, backend y frontend coincidirán sin tocar la interfaz de nuevo. Mientras tanto, un usuario que cambie el umbral en Ajustes verá el gráfico reflejar su cambio, pero las alertas reales seguirán disparándose a 70/80 °C — una discrepancia real que corregir es tarea aparte (pasar `configuradas: (f64, f64)` a `evaluar_temperatura_configurada_warn`/`_crit`, leído de `AlertSettings` en el punto de la llamada, con sus pruebas de umbral actualizadas) |

---

### L. Escala, densidad y escalado de Windows — medido

Cerrada el 2026-09-04. Banco de pruebas: `tools/scale-check.html`, que reproduce la composición
normativa con los tokens reales. Medido en un navegador Chromium con la tipografía ya empotrada.

#### L.1 · La escala tipográfica está bien. La sospecha era infundada

La preocupación era que 12,5 px de cuerpo y 11 px de píldora fueran demasiado pequeños. La medición
dice lo contrario:

| | Altura de x por em |
|---|---|
| Instrument Sans | 0,5175 |
| Segoe UI | 0,5000 |

Instrument Sans se ve un **3,5 % más grande** que Segoe UI al mismo `font-size`. Por tanto:

| Token | px | Equivale ópticamente a Segoe UI |
|---|---|---|
| `text-2xs` (píldoras) | 11 | 11,4 px |
| `text-xs` (metadatos) | 12 | 12,4 px |
| `text-sm` (cuerpo denso) | 12,5 | **12,9 px** |
| `text-base` (título de tarjeta) | 13,5 | 14,0 px |
| `text-lg` (barra de herramientas) | 14,5 | 15,0 px |

La convención de Windows para el texto de interfaz es Segoe UI 9 pt, o sea 12 px. El cuerpo denso de
SmartDisk equivale a 12,9 px: está **por encima** del estándar del sistema, no por debajo. **No se
toca la escala.** Y no se vuelve a tocar sin repetir esta medición.

Además, el escalado de Windows no encoge el texto: multiplica por igual el tamaño físico de todo. Al
125 % o al 150 %, el texto se ve más grande, no más pequeño. La preocupación estaba mal planteada.

#### L.2 · Lo que sí falla: el espacio en píxeles CSS

Lo que el escalado sí reduce es el espacio disponible. Área máxima de ventana por configuración,
descontando barra de tareas (48) y barra de título (32):

| Pantalla | Escalado | Ventana máxima | ¿Cabía el mínimo de 1120 × 720? |
|---|---|---|---|
| 1366 × 768 | 100 % | 1366 × 720 | sí, al límite |
| 1366 × 768 | 125 % | **1092 × 566** | **no** |
| 1600 × 900 | 100 % | 1600 × 852 | sí |
| 1920 × 1080 | 100 % | 1920 × 1032 | sí |
| 1920 × 1080 | 125 % | 1536 × 816 | sí |
| 1920 × 1080 | 150 % | **1280 × 672** | **no** |
| 2560 × 1440 | 150 % | 1706 × 912 | sí |
| 3840 × 2160 | 200 % | 1920 × 1032 | sí |

Dos de ocho configuraciones habituales no admitían la ventana mínima declarada. La de 1920 × 1080 al
150 % es especialmente común en portátiles de 13 y 14 pulgadas.

#### L.3 · Lo que falla siempre: "No disponible" no cabe en una `MetricCard`

El hallazgo más grave, y no tiene nada que ver con el escalado. Anchos medidos a 27 px
(`--sdm-text-metric`), que es como `MetricCard` componía **todos** los valores:

| Valor | Ancho | Tarjeta necesaria para 4 en fila |
|---|---|---|
| `12 %` | 51 px | 380 px |
| `47 °C` | 65 px | 436 px |
| `684 GB` | 93 px | **548 px** |
| `No disponible` | 174 px | **872 px** |
| `Sin datos SMART` | 218 px | 1048 px |

Con la rejilla anterior (`minmax(420px, 1fr)`), la celda útil de una métrica era de 61 a 86 px. O
sea: **`684 GB` ya se recortaba, y `No disponible` se recortaba en todas las resoluciones sin
excepción** — justo en el caso más frecuente de la aplicación, que es un disco USB, RAID o virtual
sin SMART. La auditoría encontraba entre 2 y 10 elementos recortados en cada configuración.

#### L.4 · Las cuatro correcciones, verificadas

| # | Cambio | Dónde |
|---|---|---|
| 1 | Un valor no numérico se compone como **texto** (`text-base`, peso 500, gris tenue), no como cifra | `MetricCard.svelte` |
| 2 | La fila de métricas pasa de flex a `grid` con `repeat(auto-fit, minmax(104px, 1fr))`: se reorganiza en vez de comprimirse | `AGENTS.md` §4.6 |
| 3 | La rejilla del panel sube de `minmax(420px)` a `minmax(460px)` | `AGENTS.md` §4.0.bis |
| 4 | Ventana mínima de 1120 × 720 a **1024 × 560**, con la barra lateral colapsada a iconos por debajo de 1180 px | `AGENTS.md` §4.0 |

Tras aplicarlas, la auditoría da **cero recortes en las ocho configuraciones**, y las ocho admiten la
ventana mínima. Verificado también visualmente en el caso más apretado (1092 × 566).

De paso, `MetricCard` usaba `sdm-material` con radio de tarjeta dentro de otra tarjeta, lo que
incumplía la prohibición de apilar materiales de `AGENTS.md` §2.bis. Corregido a `bg-glass-3` +
`rounded-inner`.

#### L.5 · Queda un detalle de afinado

Con `auto-fit`, una tarjeta estrecha reparte las cuatro métricas en 3 + 1 en lugar de 2 × 2. No
recorta nada y se ve correcto, pero 2 × 2 sería más regular. Es una decisión de composición para
quien diseñe la `DiskCard` definitiva, no un defecto.

---

### M. WebView2 en Windows Server — resuelto

Cerrada el 2026-09-04 contra la documentación oficial de Microsoft, sin necesidad de ensayo en
hardware. Decisión completa en el ADR-020.

#### M.1 · La matriz de sistemas de la especificación es correcta

Microsoft Edge —y con él WebView2, que sigue exactamente su soporte— cubre:

| | Soportado |
|---|---|
| Windows Server 2016, 2019, 2022, 2025 (LTSC) | sí |
| Windows 10 desde SAC 1709, y todas las LTSC desde 2015 | sí |
| Windows 11 | sí |

No hay que recortar nada de lo prometido. Microsoft además mantiene actualizaciones de WebView2 en
Windows 10 22H2 al menos hasta octubre de 2028, con lo que la plataforma no caduca antes que el
producto.

Dos requisitos que no estaban escritos y ahora sí:

- **CPU con SSE3**, que Edge exige desde su versión 128.
- **Experiencia de escritorio** en Windows Server: una aplicación gráfica no es utilizable sobre
  Server Core. No es un problema de WebView2, es de sentido común, pero conviene decirlo porque
  "Windows Server 2016–2025" a secas se puede leer como que incluye Core.

#### M.2 · Lo que sí era un problema real

**El runtime no viene preinstalado en Windows Server, en ninguna versión.** Solo Windows 11 lo
incluye como parte del sistema; en Windows 10 lo tiene la gran mayoría de equipos porque Microsoft
lo desplegó por Windows Update desde diciembre de 2022. En un servidor recién instalado, la
aplicación sencillamente no arrancaría.

#### M.3 · Qué se descartó y por qué

| Modo | Añade | ¿Internet al instalar? | ¿Se parchea solo? | Veredicto |
|---|---|---|---|---|
| `downloadBootstrapper` | 0 MB | sí | sí | **No**: un servidor aislado es el escenario, no la excepción |
| `embedBootstrapper` | ~1,8 MB | sí | sí | **No**: mismo problema |
| `offlineInstaller` | ~127 MB | no | sí | **Elegido** |
| `fixedRuntime` | ~180 MB | no | **no** | **No**: véase abajo |
| `skip` | 0 MB | no | — | **No**: la aplicación no arrancaría |

`fixedRuntime` parecía la opción evidente para un producto sin conexión, y es la que había apuntado
la revisión inicial. Es la peor: congela una versión de Chromium dentro de la aplicación, que
dejaría de recibir parches de seguridad hasta que publicásemos una versión nueva —y sin actualizador
automático (ADR-007), eso es "hasta que el usuario se entere". La aplicación renderiza texto que
viene de dispositivos y del registro de eventos, así que un motor sin parchear no es aceptable.
Además no funciona desde rutas de red o UNC, exige conceder permisos con `icacls` a los contenedores
de aplicación en Windows 10 desde la versión 120, y ocupa más de 250 MB en disco.

`offlineInstaller` instala el runtime **Evergreen**: la instalación funciona sin conexión y a partir
de ahí lo mantiene Microsoft. Es la única opción que cumple las dos condiciones a la vez.

#### M.4 · Lo que queda por hacer

- Declarar `webviewInstallMode: { "type": "offlineInstaller" }` en `tauri.conf.json`.
- Comprobar en tiempo de ejecución que el runtime está presente y, si no, mostrar una frase
  comprensible en lugar de una ventana en blanco. La detección oficial es la clave del registro
  `pv` en `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`,
  con valor mayor que `0.0.0.0`.
- Indicar el tamaño del instalador (~140 MB) en la página de descarga.
- Verificar en Fase 0 que la instalación silenciosa funciona en un Windows Server 2019 limpio y sin
  salida a Internet. Es lo único que sigue requiriendo una máquina de verdad.

---

### N. smartctl: versión y licencia — resuelto

Cerradas K.2 y K.3 el 2026-09-04. Decisión completa en el ADR-021; detalle operativo en
`third-party/smartmontools/README.md`.

#### N.1 · Versión elegida

**smartmontools 7.5**, publicada el 12 de mayo de 2025 (compilación r5714). Es la última estable.
El binario ya está en `third-party/smartmontools/`, con sus sumas MD5 verificadas contra las que
publica el propio proyecto y contra el `checksums64.txt` que viaja dentro del paquete oficial.

Un detalle que conviene saber: el paquete oficial de Windows se llama `win32-setup` por razones
históricas, pero **contiene las dos arquitecturas**. El de `bin/` es x64 —verificado leyendo la
cabecera PE, máquina `0x8664`— y el de `bin32/` es x86. No hay ZIP portable: hay que extraer el
instalador.

#### N.2 · Hacía falta un fichero que no estaba en ninguna parte de la documentación

`drivedb.h` (268 KB) es la base de datos de unidades de smartmontools. **Sin ella, `smartctl` no
sabe interpretar los atributos específicos de cada fabricante** y los presenta como desconocidos,
que es justo la información que hace útil a un monitor de discos. No aparecía mencionada en ninguno
de los documentos del proyecto. Se empaqueta junto al binario.

Queda congelada con la versión: el script oficial que la actualiza (`update-smart-drivedb.ps1`)
descarga de Internet, así que no se distribuye. Los modelos de disco muy recientes podrían no ser
reconocidos hasta que se actualice la versión de smartmontools; es una limitación conocida, no un
fallo.

#### N.3 · Qué se deja fuera

`smartd` y sus utilidades de notificación. La aplicación ya tiene su propio planificador, y un
segundo vigilante competiría por el acceso a los dispositivos. También los binarios de 32 bits.

#### N.4 · La GPLv2, resuelta por la vía 3(a)

`smartctl` es `GPL-2.0-or-later`. **El código propio sigue siendo MIT**: se invoca como proceso
independiente, por línea de órdenes y JSON, sin enlazarlo ni incorporar su código, así que no hay
obra derivada.

La obligación real es la de la sección 3: quien recibe el binario tiene derecho al fuente
correspondiente. Se cumple acompañando el binario del código —vía 3(a)—, metiendo
`smartmontools-7.5.tar.gz` (1,1 MB) dentro del instalador, en `licenses\smartmontools\`.

Se descarta la vía 3(b), la oferta escrita válida tres años, porque obliga a mantener el fuente
disponible y atender solicitudes durante ese plazo. Un fichero de 1 MB dentro de un instalador de
140 MB cuesta menos y no caduca. **La versión del tarball debe coincidir siempre con la del
binario**, o el requisito deja de cumplirse.

#### N.5 · Dos comprobaciones hechas sobre el binario real

Se ejecutó el binario redistribuido en este equipo:

- `smartctl --scan-open --json` funciona **sin privilegios de administrador** y enumera los
  dispositivos con su tipo. Devolvió `exit_status: 0` y detectó dispositivos ATA y NVMe.
- Leer datos de un dispositivo **sin elevación falla**, lo que confirma la premisa del ADR-004. Pero
  falla de una forma engañosa: `exit_status: 1` y el mensaje
  `"/Device/HarddiskN/Partition0: Unable to detect device type"`.

Esto último importa más de lo que parece. Ese mensaje **no** significa que el disco sea
incompatible, y tomarlo al pie de la letra marcaría un equipo entero como "no compatible" cuando el
problema es de privilegios — rompiendo la regla de "no compatible ≠ averiado" por el lado
contrario, dando por normal lo que es un fallo de configuración. El colector debe distinguir los dos
casos y, ante ese mensaje, comprobar primero si el proceso está elevado.

---

### O. Contraste del acento heredado — medido

Cerrada el 2026-09-04. Herramienta: `tools/accent-check.py`, que barre el espacio sRGB completo
(262.144 colores, paso 4) contra las superficies efectivas de ambos temas.

#### O.1 · La pregunta estaba mal planteada

La duda original hablaba de "los 48 acentos de Windows". **No existe tal lista cerrada**: Windows
ofrece una cuadrícula de sugerencias, pero el usuario puede elegir cualquier color con un selector
completo. La validación por muestreo no servía; había que barrer el espacio entero.

#### O.2 · Lo que ya funcionaba

`accessibleAccent()`, el ajuste del acento **como fondo** del botón primario:

| | |
|---|---|
| Colores por debajo de AA tras el ajuste | **0 de 262.144** |
| Colores que necesitaron retoque | 9.477 (3,6 %) |
| Mayor desviación aplicada | 26/255 en un canal, imperceptible |

Validado sin cambios.

#### O.3 · Lo que no se había mirado, y fallaba

El acento tiene **un segundo uso con el requisito opuesto**: pintar texto e iconos *sobre* el
material — enlaces, la etiqueta de la pestaña seleccionada, la serie principal de la gráfica. Ahí el
contraste se mide contra la superficie, que es casi blanca en tema claro (`#f9f9fb` efectivo) y casi
negra en oscuro (`#212128`).

Sin tratar:

| | Ilegibles como texto |
|---|---|
| Tema claro | **170.562 de 262.144 (65,1 %)** |
| Tema oscuro | **130.071 de 262.144 (49,6 %)** |

Y lo más grave: **falla también el azul `#0078d4` que Windows trae de fábrica** — 4,31:1 en tema
claro y 3,53:1 en oscuro, ambos por debajo de AA. Es decir, `a { color: var(--sdm-accent) }`
producía enlaces que incumplían la norma del propio sistema de diseño **en la configuración más
común que existe**, sin que hiciera falta un acento raro.

#### O.4 · El sistema de diseño ya tenía la respuesta, y el código la destruía

Los respaldos de `tokens.css` estaban bien elegidos, con un tono distinto por tema:

| Tema | Respaldo | Contraste sobre su material |
|---|---|---|
| Claro | `#0067c0` | 5,40:1 ✔ |
| Oscuro | `#3d95ea` | 5,09:1 ✔ |

El defecto estaba en `applySystemAccent()`, que **sobrescribía los dos con el mismo color plano del
sistema**, borrando justamente la distinción que hacía que funcionaran.

#### O.5 · Windows ya deriva los tonos que hacen falta

El registro expone en `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent\AccentPalette`
siete tonos derivados del acento. Leídos en este equipo, con el azul de fábrica:

| Índice | Color | Sobre material claro | Sobre material oscuro |
|---|---|---|---|
| 1 | `#4CC2FF` | 1,91:1 | **7,97:1 ✔** |
| 3 (base) | `#0078D4` | 4,31:1 | 3,53:1 |
| 4 | `#0067C0` | **5,40:1 ✔** | 2,82:1 |

El tono 4 es **exactamente** el respaldo que el diseñador ya había puesto en `tokens.css` para tema
claro. Windows hace este mismo trabajo para su propia interfaz, así que usar su paleta integra la
aplicación con el sistema en lugar de inventarse un color.

#### O.6 · La corrección, verificada

1. Token nuevo **`--sdm-accent-fg`** en ambos temas, mapeado a Tailwind como `text-accent-fg`, con
   los respaldos ya verificados. `a { }` pasa a usarlo.
2. **`accentOnSurface()`** en `accent.ts`: busca primero en la paleta de Windows el tono más cercano
   al acento base que alcance AA sobre la superficie del tema; si ninguno llega, deriva uno.
3. La superficie efectiva se calcula **desde los tokens vivos**, componiendo `--sdm-glass` sobre
   `--sdm-bg`, no desde constantes: si `tokens.css` cambia, la comprobación no miente en silencio.
4. `theme.svelte.ts` llama a `refreshAccentForTheme()` al cambiar de tema, porque la superficie
   cambia y el tono derivado deja de valer.

Resultado del mismo barrido tras la corrección: **0 de 262.144 por debajo de AA**, en los dos temas
y en los dos usos.

#### O.7 · Un detalle para quien escriba el backend

`HKCU\Software\Microsoft\Windows\DWM\AccentColor` guarda el color en **ABGR**, no en RGB.
Leerlo como RGB devuelve el color invertido: el azul de fábrica (`0xFFD47800`) saldría naranja
`#D47800` en vez de azul `#0078D4`. El comando `get_system_accent_color` debe devolver además la
`AccentPalette` cuando esté disponible, porque es lo que alimenta el punto 2.

---

### P. Eventos de Windows — verificados

Cerrada K.5 el 2026-09-04. La lista normativa vive en `alert-rules.md` §3; aquí queda el método y
lo que cambió.

#### P.1 · Método

Dos fuentes, ninguna de ellas un blog:

1. **Los manifiestos de proveedor** del propio Windows (`Get-WinEvent -ListProvider`), que declaran
   cada evento con su identificador, su nivel y su plantilla de mensaje.
2. **180 días de registro `System`** de un equipo real: 2.038 eventos de almacenamiento sobre 32.620
   totales, agrupados por proveedor, identificador y nivel.

La segunda fuente es la que no se puede sustituir por documentación: dice **qué es ruido de fondo**,
que resulta ser la pregunta importante.

#### P.2 · El hallazgo que habría hundido el producto

`disk` 51 —"Error detectado en el dispositivo durante una operación de paginación"— apareció
**839 veces en 180 días en un equipo sano**. Es, con diferencia, el evento de almacenamiento más
frecuente de Windows, y es benigno: se dispara al desconectar un medio extraíble, al despertar un
disco o ante cualquier reintento que el sistema resuelve solo.

La lista tentativa de la especificación lo clasificaba como **crítico**. Habría producido 839
alertas críticas falsas en un equipo sin ningún problema. Un monitor que grita todos los días deja
de leerse, así que eso no habría sido un defecto menor: habría inutilizado el producto.

Ahora es advertencia, solo sobre discos no extraíbles y con umbral de ≥ 10 en una hora.

#### P.3 · Tres errores más de clasificación

| Antes | Realidad |
|---|---|
| `Ntfs` 98 → "metadatos inconsistentes", crítico | Es `Microsoft-Windows-Ntfs` 98, de nivel **Información**, y significa que el volumen **está bien**. Observado 319 veces: otros 319 falsos críticos |
| `Ntfs` 130 → "marcado para comprobación" | 130 es "se **reparó** la estructura", una advertencia leve. El que indica daño irreparable es el **131**, que no estaba en la lista |
| `volmgr` 46 y 49 | No existen en el sistema. El `volmgr` que sí aparece (161) es un fallo al crear el volcado de memoria y no dice nada del disco |

#### P.4 · Lo que faltaba

- **`disk` 157, "El disco se ha extraído de forma imprevista"**. Es exactamente el evento que
  necesita la regla `device.removed_unexpected`, y no estaba. Observado 63 veces.
- `disk` 158, dos discos con identificadores duplicados. Confirma que la colisión de identidad de
  §F.2 no es teórica.
- `Ntfs` 50 (fallo de escritura demorada, 367 veces) y `Microsoft-Windows-Ntfs` 140 (no se pudo
  vaciar el registro de transacción, 173 veces).
- **El proveedor `Microsoft-Windows-NvmeDisk` entero**, con su evento 500 (comando NVM con error) y
  501 (caché de escritura habilitada, que es informativo).
- `Microsoft-Windows-StorageSpaces-Driver`, con eventos muy concretos para disco virtual degradado.

#### P.5 · Un cambio de diseño en el motor, no solo de lista

El dato más útil no fue ningún identificador suelto, sino el patrón: **un solo hecho físico produce
una ráfaga de eventos distintos**. Al desconectar en caliente un disco externo, el mismo dispositivo
generó en segundos un `disk` 157, un `disk` 51, un `Ntfs` 50 y un `Microsoft-Windows-Ntfs` 140.

Con la deduplicación por `provider:event_id` que decía la especificación, ese único suceso habría
creado **cuatro grupos de alerta**. El motor necesita, además, una **ventana de correlación de
60 segundos por dispositivo**, con una regla de causa: si en la ráfaga hay un `disk` 157, ese es el
suceso y los demás son ocurrencias suyas. Recogido en `alert-rules.md` §3.5.

#### P.6 · Qué sigue pendiente

La muestra es de **un** equipo Windows 11 en español, con NVMe, SATA y discos externos USB. No cubre
servidores, RAID por hardware ni Storage Spaces en producción: esos eventos están tomados de los
manifiestos, no observados. Sigue en la Fase 0 contrastarlos en un servidor.

Y una obviedad que conviene decir: un equipo sano no produce eventos de fallo real, así que la
ausencia de `Ntfs` 55 o `disk` 7 en la muestra es una buena noticia, no una señal de que no existan.

---

### Q. Codificación de la salida de los procesos auxiliares — medido

Cerrada I.3 el 2026-09-04. Herramienta y heurística de referencia: `tools/console-encoding.py`.

#### Q.1 · La suposición de partida era falsa

La especificación decía que la salida de `chkdsk` llegaría en la página OEM de la consola, CP850 en
un Windows en español. **No es así.** Capturando los bytes crudos de un proceso sin consola
(`CreateNoWindow`) con la salida por tubería, que es exactamente como lo lanzará Rust:

```
chkdsk  ->  E1 E9 ED F1 F3 FA
            como CP1252: áéíñóú      ✔
            como CP850 : ßÚÝ±¾·      ✘
```

`chkdsk` emite **CP1252, la página ANSI**, no la OEM.

#### Q.2 · Y el problema real es peor: Windows no es consistente consigo mismo

En el mismo equipo, el mismo día, con el mismo tipo de tubería:

| Herramienta | Bytes de las vocales acentuadas | Página |
|---|---|---|
| `chkdsk` | `E1 E9 ED F1 F3 FA` | **CP1252** (ANSI) |
| `chkntfs` | `E1 E9 ED F1 F3` | **CP1252** (ANSI) |
| `fsutil` | `A0 A1 A2` | **CP850** (OEM) |
| `vssadmin` | `A0 A1 A2 A3 A4` | **CP850** (OEM) |

No hay una regla del sistema que seguir: depende de cómo se escribió cada herramienta. Cualquier
constante que se codifique acertará con unas y producirá basura con otras.

#### Q.3 · La solución evidente no funciona

Fijar la página de códigos antes de invocar, con `chcp 850` o `chcp 65001`, **no cambia nada** si la
salida está redirigida. Comprobado: los volcados de `chkdsk` con la página heredada, con 850 y con
65001 salieron **byte a byte idénticos**, mismo MD5 los tres. La página de consola gobierna lo que
se pinta en una consola, no lo que se escribe en una tubería.

#### Q.4 · Detección, validada

`tools/console-encoding.py` implementa la heurística de referencia, en cascada:

1. **Un BOM manda.** Es una declaración explícita, no una conjetura.
2. **Si todo es ASCII**, cualquier página vale y no se adivina nada.
3. **Si decodifica como UTF-8 estricto, es UTF-8.** Esto cubre los equipos con el modo
   *Beta: usar Unicode UTF-8* activado, donde la ANSI del sistema pasa a ser 65001.
4. **Si no**, se puntúan las páginas de un byte: suman las letras que un texto real produce
   (`áéíóúüñ¿¡°…`) y restan, con peso triple, los símbolos que delatan una página equivocada
   (griego, dibujo de cajas, matemáticas). Gana la de mayor puntuación.
5. **Empate o puntuación nula**: la ANSI del sistema. Nunca se falla ni se pierde salida.

Resultado sobre las cuatro herramientas, con márgenes que no dejan lugar a duda:

| Volcado | Elegida | Puntuaciones |
|---|---|---|
| `chkdsk` | **cp1252** | cp1252 = 31, cp850 = −87, cp437 = −61 |
| `chkntfs` | **cp1252** | cp1252 = 15, cp850 = −44, cp437 = −36 |
| `fsutil` | **cp850** | cp850 = 25, cp1252 = 3 |
| `vssadmin` | **cp850** | cp850 = 13, cp1252 = −2 |

Cuatro de cuatro. Nota de afinado: en un Windows en inglés la OEM es CP437 y no CP850, así que en
caso de empate conviene preferir la OEM que declare el sistema (`GetOEMCP()`) en lugar de una
constante.

#### Q.5 · Reglas que se derivan

- **La salida se guarda en bytes, siempre.** La decodificación es solo para presentar. Así un fallo
  de detección no destruye información, y el ZIP de diagnóstico lleva el original.
- **Se registra la codificación deducida** junto a la ejecución de la prueba, para que un informe
  raro se pueda diagnosticar sin repetir el escaneo.
- **Nunca se falla por un byte no decodificable**: se sustituye por U+FFFD y se sigue. Un carácter
  raro en un mensaje no puede tumbar la captura de un `chkdsk` de veinte minutos.
- La misma detección vale para cualquier proceso auxiliar futuro. `smartctl` no la necesita porque
  emite JSON en inglés, pero conviene aplicarla igual: sale gratis y evita una sorpresa.
- Hay que **guardar volcados reales como fixtures** de test, uno de CP1252 y otro de CP850. Es la
  única forma de que una regresión en esto se note antes de llegar al usuario.

---

### R. Instancia única y ACL de `ProgramData` — medido

Cierra I.6 el 2026-09-04. Decisiones resultantes: ADR-025 (instancia única) y ADR-026 (ACL).

#### R.1 · Eran dos preguntas, no una

I.6 juntaba dos cosas sin relación técnica. Separadas:

- **Instancia única.** El requisito real no es «bloquear la segunda», sino «abrir una segunda
  restaura la ventana de la primera». Eso obliga a comunicar dos procesos, no solo a detectarse.
  Resuelto con `tauri-plugin-single-instance` (ADR-025).
- **ACL de la carpeta de datos.** Aquí estaba el hallazgo.

#### R.2 · La suposición de partida era falsa

La especificación daba por hecho que `%ProgramData%` restringe la escritura a administradores.
Medido con `icacls` en Windows 11 Pro 26200, en español:

```text
C:\ProgramData  NT AUTHORITY\SYSTEM:(OI)(CI)(F)
                BUILTIN\Administradores:(OI)(CI)(F)
                CREATOR OWNER:(OI)(CI)(IO)(F)
                BUILTIN\Usuarios:(OI)(CI)(RX)
                BUILTIN\Usuarios:(CI)(WD,AD,WEA,WA)
```

La última línea concede a **cualquier usuario** crear ficheros (`WD`) y carpetas (`AD`), y `(CI)`
lo propaga a toda subcarpeta. `CREATOR OWNER` remata: quien cree algo ahí queda con Control total
sobre ello.

#### R.3 · Verificado: un usuario sin privilegios se apropia de la carpeta

Desde una sesión **no elevada** (`net session` → acceso denegado):

```text
mkdir C:\ProgramData\_smartdisk_acl_probe        ->  creada, sin UAC
icacls C:\ProgramData\_smartdisk_acl_probe
   ...
   RYZEN\danimardo:(I)(F)        <- Control total heredado de CREATOR OWNER
```

Es un ataque de **pre-creación**: basta con adelantarse al instalador. A partir de ahí el atacante
controla dónde va a vivir la base SQLite del historial.

Nota: el usuario **no** puede modificar ficheros que cree un administrador. `WD,AD` van sin `(OI)`,
así que aplican a la carpeta —crear— y no se heredan a los ficheros, que reciben solo `(OI)(RX)`.
Tampoco puede borrarlos: `DC` no está concedido. El riesgo es plantar ficheros y controlar la raíz,
no manipular los existentes.

#### R.4 · El endurecimiento funciona, y el orden importa

Aplicado sobre la carpeta de sondeo:

```text
icacls <carpeta> /inheritance:r
  /grant:r *S-1-5-18:(OI)(CI)F        SYSTEM
  /grant:r *S-1-5-32-544:(OI)(CI)F    administradores
  /grant:r *S-1-5-32-545:(OI)(CI)RX   usuarios, solo lectura
```

Resultado inmediato, desde la misma sesión no elevada:

```text
touch <carpeta>\intruso.txt   ->  Permission denied   ✔
mkdir <carpeta>\sub           ->  Permission denied   ✔
```

**Pero no basta.** El propietario conserva `WRITE_DAC` implícito:

```text
icacls <carpeta> /grant "danimardo:(OI)(CI)F"   ->  correcto
touch <carpeta>\intruso.txt                     ->  escribe   ✘
Owner: RYZEN\danimardo
```

De ahí que ADR-026 exija **`/setowner *S-1-5-32-544` antes** de fijar la ACL. Restablecer permisos
sin cambiar el propietario deja el agujero abierto y da falsa sensación de estar cerrado.

#### R.5 · SID numéricos, no nombres de grupo

En esta máquina el grupo es `Administradores`; en un Windows en inglés, `Administrators`; en
francés, `Administrateurs`. Un instalador escrito con nombres falla fuera de su idioma. Se usan
siempre `*S-1-5-18`, `*S-1-5-32-544` y `*S-1-5-32-545`.

#### R.6 · Reglas que se derivan

- **La raíz de datos la crea el instalador, con su ACL explícita.** `platform::paths::log_dir()`
  ya no la crea en compilación de publicación: crear la raíz ad hoc reproduce la ACL heredada
  débil, que es justo lo que se quiere evitar. Si falta, la instalación está rota y debe notarse.
- **`/setowner` antes que `/grant`**, siempre, y con `/t /c` para arrastrar lo que hubiera dentro.
- **La comprobación de la ACL entra en los criterios de US-060**, no en una lista aparte.
- El usuario sin privilegios conserva **lectura**, deliberadamente: la interfaz muestra informes y
  el ZIP de diagnóstico se genera ahí, y su contenido ya está anonimizado.
- **Pendiente de verificación manual**: que lanzar una segunda instancia restaure la ventana de la
  primera. El código está cableado y compila, pero comprobarlo exige arrancar la aplicación
  elevada y aceptar el UAC, cosa que ninguna prueba automática de este proyecto puede hacer.
  Entra como comprobación de humo de US-060.

---

### S. Paleta v3 «Ciruela» — ratios de contraste medidos

Cerrada el 2026-09-06 al implantar el rediseño v3 (ADR-034, `specs/002-rediseno-v3/`, US1).
Herramienta: `scripts`/`tools/accent-check.py` reutilizado para componer cada color sobre el
material real (`--sdm-glass` sobre la media del degradado del lienzo; y `--sdm-glass-3` encima, para
la columna «bloque interno»; la columna «píldora» mide el color contra su propio `-soft` compuesto
sobre el material, que es el caso de `StatusPill`). Mínimo exigido: 4,5:1 (constitución §VII, WCAG
1.4.3), medido **como texto de píldora**, que es el uso más exigente.

#### S.1 · Resultado

Todos los tokens de texto y de salud de la paleta Ciruela cumplen AA en los dos temas, sobre
material y sobre bloque interno. Los valores medidos coinciden con la tabla que entregó el diseñador
en `design/propuesta-redisenov2/cambios/00-tokens.md` dentro de ±0,05.

| | material (claro / oscuro) | bloque interno (claro / oscuro) | píldora (claro / oscuro) |
|---|---|---|---|
| `--sdm-text` | 16,40 / 13,81 | 14,62 / 11,58 | — |
| `--sdm-text-dim` | 6,15 / 6,06 | 5,48 / 5,08 | — |
| `--sdm-text-faint` | 5,42 / 5,72 | **4,83** / **4,80** | — |
| `--sdm-accent-fg` | 6,52 / 6,86 | 5,81 / 5,75 | 5,46 / 4,82 |
| `--sdm-ok` | 5,38 / 7,62 | 4,80 / 6,39 | **4,68** / 5,56 |
| `--sdm-warn` | 6,13 / 8,11 | 5,46 / 6,80 | 5,31 / 5,78 |
| `--sdm-crit` (bermellón) | 5,82 / 5,98 | 5,19 / 5,01 | 4,86 / **4,57** |
| `--sdm-unknown` | 6,04 / 6,16 | 5,38 / 5,17 | 5,19 / 4,80 |

Texto blanco sobre el acento sólido en claro: 6,94:1 (`--sdm-on-accent` sigue siendo `#ffffff`).
`--sdm-on-accent` en oscuro pasa a tinta `#20132a`: 7,80:1 sobre el acento (en blanco daba 2,27:1).

#### S.2 · Los tres valores más justos, verificados

- `--sdm-text-faint` sobre bloque interno: 4,83 (claro) / 4,80 (oscuro). El diseñador ya los había
  subido respecto a su primera propuesta (`#988ea0` daba 4,18 en oscuro); estos son los definitivos.
- `--sdm-ok` como texto de píldora en claro: 4,68. Sin margen para aclararlo.
- `--sdm-crit` como texto de píldora en oscuro: 4,57. El bermellón `#ef8080` está calibrado al
  límite: no lo aclares.

#### S.3 · Reglas que se derivan

- **Ninguno de estos tokens se aclara.** Si un texto queda justo sobre el material, se sube la
  opacidad de la capa, nunca se rebaja el color (misma regla que §O y que `ui-design.md` §6).
- **No pongas texto directamente sobre `bg-glass-3`** salvo que sea uno de los tokens de esta tabla:
  la columna «bloque interno» es el suelo.
- El interruptor «usar el acento de Windows» (ADR-035) mantiene intacta la corrección de §O:
  `accessibleAccent()` / `accentOnSurface()` siguen barriendo el acento del usuario.

---

### T. Perfiles de alerta — decisiones adoptadas (ADR-036)

Cerrada el 2026-09-06 al implantar US10 del rediseño v3 (`specs/002-rediseno-v3/`).

#### T.1 · El motor pasa a leer los umbrales de `settings`

Hasta v3, `alerts::motor` llevaba los umbrales **escritos a mano** (`90/100` desgaste, `70/80`
temperatura) y `settings.alerts.*` se guardaba sin que ninguna regla lo leyera — el mismo hueco que
J.32 describía para las claves de capacidad. Con ADR-036 el motor recibe los umbrales como parámetro
(`ConfigUmbrales`), que `commands::refresh_smart` resuelve de `settings` una vez por ciclo. Reglas
parametrizadas: `smart.wear_high`, `temp.above_configured_warn/crit`, `smart.media_errors` y —nuevas
en el motor— `capacity.low`/`capacity.critical`.

#### T.2 · El umbral térmico de fábrica baja a 60/70 °C

Era 70/80. El perfil «Equilibrado» de `cambios/08b-perfiles-de-alerta.md` lo fija en 60/70, y ese
pasa a ser también el valor de fábrica (spec 002, clarify Q2). 60 °C sigue siendo temperatura alta
para un SSD de consumo, y mantener dos números distintos («fábrica» vs «Equilibrado») confundiría.
Actualizado `alert-rules.md` §2 y las pruebas de `alerts::motor` y `commands::set_setting`.

#### T.3 · `media_errors_*` no es una ventana de 24 h

El nombre `mediaErrorsWarnPer24h` viene de la propuesta del diseñador, pero la semántica adoptada
(clarify Q1) es **el incremento de `media_errors_total` entre dos lecturas consecutivas** que basta
para avisar. No se construye una mecánica de conteo por ventana de 24 h: reinterpretar sobre la regla
existente cubre el caso. La interfaz no muestra «/24 h».

#### T.4 · `driver_retry_*` se guarda pero **aún no lo consume ninguna regla**

Un perfil escribe los doce umbrales, `driver_retry_warn/crit_per24h` incluidos, para que el juego
esté completo. Pero las reglas `events.controller_reset` / `events.io_retry` que los consumirían
**no existen en el motor**: necesitan el colector de eventos de Windows completo (Historia 4). Es el
mismo patrón con el que las claves de capacidad y `logging.verbose` vivieron guardadas sin consumidor
hasta que su regla se implementó (J.32, FR-029a). Cuando exista esa regla, el umbral ya está.

#### T.5 · Editar un umbral a mano rompe el perfil

`set_setting` sobre cualquier `alerts.*` (salvo `alerts.profile`) pone `alerts.profile = "custom"`.
La única forma de volver a un perfil concreto es elegirlo, y entonces se reescriben sus doce valores.
La interfaz muestra «Personalizado (a partir de \<perfil anterior\>)» derivando el «anterior» del
último `profile` no-`custom` conocido en memoria, no de un segundo campo persistido.

### U. Panel general v3 — virtualización sustituida por la variante compacta

Cerrada el 2026-09-06 al implantar US5 del rediseño v3 (`specs/002-rediseno-v3/`, PR 6).

#### U.1 · Por qué desaparece la `VirtualList` de la rejilla de discos

El panel v2 envolvía la rejilla de `DiskCard` en una `VirtualList` que virtualizaba **por filas**
(J.26): con 20 discos, pintar la rejilla entera producía una tarea de ~100 ms, por encima del umbral
de 50 ms de SC-006/SC-007.

El panel v3 (`cambios/01-panel-general.md`) cambia el encuadre: ahora hay un `HeroPanel` de 246 px
arriba y una fila inferior (sucesos + reparto de estados) abajo, y **el diseñador especifica que la
región entera hace scroll** («con más discos la región hace scroll, nada se recorta»). Anidar una
`VirtualList` de altura fija solo para la rejilla, entre un héroe y un pie que también deben
desplazarse con ella, va contra ese encuadre y contra la constitución §XIV (una sola región de
scroll natural).

En su lugar se aplica lo que ya prescribía `ui-design.md` §7: **a partir de 12 discos monitorizados
la `DiskCard` pierde la sparkline de cabecera** (`conSparklines = devices.length <= 12`). El coste de
render que J.26 midió venía casi todo del SVG por tarjeta; sin él, 20 tarjetas se pintan holgadas.

#### U.2 · Medido, no estimado

`e2e/ui/rendimiento.spec.ts` («recibir 20 discos en caliente … no produce ninguna tarea de 50 ms o
más») se conserva sin cambios y **pasa** contra el panel v3 con rejilla plana: `[]` tareas largas.
SC-006 se mantiene por medición, que era el objeto de J.26 — no por la técnica concreta.

#### U.3 · Series de temperatura: carga perezosa por disco visible

El panel solo trae el inventario en su `load` (constitución §XIV). Tras el primer render, un
`$effect` pide `getMetricSeries("temperature_celsius", 24 h)` para el disco del héroe y —si
`conSparklines`— para el resto; el store (`app.temperatureSeries`) cachea por disco para no repetir
la petición. Un fallo por disco degrada solo esa sparkline (no se pinta) y no tumba el panel.

### V. Asistente inicial — decisiones adoptadas (US8, ADR-037/038)

Cerrada el 2026-09-06 al implantar US8 del rediseño v3 (`specs/002-rediseno-v3/`).

#### V.1 · El guardián de `+layout.ts` detecta «ya configurado» con lo observable

FR-043 pide no mostrar el asistente a quien actualiza desde una versión sin él. El frontend **no
puede** saber si se guardó *cualquier* clave suelta de `settings` sin una señal nueva del backend
(`get_settings` devuelve valores resueltos, no dice cuáles son de fábrica y cuáles guardados). Se
comprueba lo que sí es observable: `theme != "system"`, `language != null`,
`settings.alerts.profile != "balanced"`, algún `device.alias`, o `excluded.length > 0`. Cubre todos
los casos realistas de actualización (quien ya usaba la app renombró un disco, cambió el tema o tocó
las alertas). **Limitación aceptada**: un usuario de v3 desde cero que solo cambió, p. ej., un día de
retención y cerró la app antes de acabar el asistente lo volverá a ver — que es justo lo que FR-043
dice que debe pasar («interrumpida antes de guardar nada vuelve a mostrar el asistente»). No se
añade backend por este caso.

#### V.2 · El guardián nunca atrapa: cualquier fallo cae a «seguir normal»

Si `get_settings` / `get_devices` / `get_appearance_settings` fallan, el `load` del layout devuelve
`{}` sin redirigir. Un fallo de arranque real ya lo explica `+layout.svelte`; lo que no puede pasar
es un bucle de redirección a `/onboarding` cuando el backend no responde.

#### V.3 · `notifications.enabled` y el autoarranque — verificación

- `notifications.enabled`: la decisión de enviar el toast se factoriza a `debe_enviar(...)` (pura,
  con pruebas). El resto de `alerts::notificaciones::procesar_una` sigue sin prueba automática
  porque necesita un proceso Tauri real (`research.md` R1) — mismo trato que ya tenía.
- `lifecycle.start_with_system`: `platform::autoarranque::aplicar()` lanza `schtasks.exe` y **no se
  ejecuta en `cargo test`** (crearía una tarea en el equipo del desarrollador). Se prueba el formato
  de la línea de comando (`linea_de_comando`) y el nombre estable de la tarea; el registro/borrado
  real en el Programador de tareas se verifica a mano —mismo criterio que J.28 (autotest SMART) y
  `platform/sistema.rs`—. **Pendiente**: activar el interruptor en un Windows real, comprobar en el
  Programador que existe «SmartDisk Monitor - Autostart» con «Ejecutar con los privilegios más
  altos» y disparador «al iniciar sesión», reiniciar y confirmar que la app abre elevada sin UAC.

### W. Geometría de la ventana entre sesiones — decisión adoptada (ADR-040)

Cerrada el 2026-09-06.

#### W.1 · Por qué en `settings` y no con `tauri-plugin-window-state`

El plugin estándar guarda su estado en un fichero JSON propio, fuera de SQLite: choca con el
principio **V** (INNEGOCIABLE, «ningún otro almacén de datos estructurados»). Además sería
dependencia nueva (enmienda de la constitución) y permisos de Tauri nuevos. La vía elegida —cinco
claves `window.*` en la tabla `settings`, escritas solo por el backend— no necesita ninguna de esas
tres cosas. El detalle completo, en ADR-040.

#### W.2 · La predeterminada 1695 × 988 y las pantallas pequeñas

Es lo que pidió el usuario para el **primer** arranque. No cabe entera en configuraciones con
mucho escalado (1920 × 1080 al 150 % deja ~1280 × 720): en ese primer arranque Windows/Tauri acota
la ventana al área de trabajo, y a partir de ahí manda la geometría que el usuario dejó, que por
definición cabía. El mínimo técnico (1024 × 560, §L) protege el caso extremo. El objetivo de
diseño (1280 × 720) no cambia: se sigue componiendo y revisando contra él.

#### W.3 · Qué se verifica a mano

`geometria_visible` (la comprobación de «¿queda dentro de algún monitor?») es pura y tiene pruebas.
`aplicar_geometria_guardada` / `persistir_geometria` **no** se prueban en `cargo test`: necesitan
un proceso Tauri con ventana real, mismo criterio que `platform::autoarranque` (V.3) y
`platform/sistema.rs`. **Pendiente** (recorrido en `specs/002-rediseno-v3/regresion-visual.md` o
al empaquetar): primer arranque a 1695 × 988; redimensionar/mover/cerrar y reabrir en la misma
geometría; maximizar/cerrar/reabrir maximizada; mover a un segundo monitor, cerrarlo y reabrir sin
que la ventana quede fuera de pantalla; «Restaurar valores de fábrica» vuelve a 1695 × 988; salir
desde la bandeja también guarda.


---

# 13. Sistema de diseño: reglas de interfaz (VINCULANTES)

Fichero de origen: `docs/ui-design.md`

Este documento es **vinculante** para cualquier agente (humano o IA) que escriba interfaz en este
repositorio. Describe cómo construir pantallas con el sistema de diseño aprobado: **v3, «escena de
datos»** sobre el material translúcido de v2 (paleta propia «Ciruela», tipografía de «display» para
cifras grandes, riel de navegación; ADR-034). Si algo no está aquí, no lo inventes: pregunta o propón
una extensión del sistema.

> **v2 → v3 (ADR-034/ADR-035).** No es una reescritura: el material de tres capas, los radios
> concéntricos, el movimiento, el catálogo cerrado y todas las reglas de producto siguen intactos.
> Cambian la paleta (acento morado de tinta, crítico bermellón), `--sdm-on-accent` (tinta en oscuro,
> ya no blanco), se añaden `.sdm-display` y tres componentes (`Icon`, `Sparkline`, `HeroPanel`), y la
> `Sidebar` pasa a un riel de 74 px. La herencia del acento de Windows deja de ser el comportamiento
> de fábrica y pasa a un interruptor apagado por defecto.

Es el par visual de `docs/ui-contract.md`: aquel dice **qué** puede pedirle la interfaz al backend,
este dice **cómo** se pinta lo que recibe. Referencias funcionales: `docs/product-specification.md`,
`docs/user-stories.md`, `docs/data-model.md`.

---

### 0. Dónde vive cada cosa

Esta tabla es el punto de entrada. Un agente que empieza una pantalla no debería tener que buscar
ninguna de estas rutas.

| Qué | Dónde |
|---|---|
| **Reglas vinculantes de interfaz** | `docs/ui-design.md` (este fichero) |
| **Tokens: fuente única de verdad visual** | `src/design-system/tokens.css` |
| Los mismos tokens, legibles por herramientas | `src/design-system/tokens.json` |
| Mapeo de tokens a utilidades Tailwind | `tailwind.config.cjs` (raíz del proyecto) |
| **Catálogo de componentes** | `src/lib/components/` — se importa del barrel `$lib/components` |
| Tipos, formato, salud, iconos, tema y acento | `src/lib/design/` (incluye `icons.ts`) |
| Diccionarios de idioma | `src/lib/i18n/es.json` y `src/lib/i18n/en.json` |
| Tipografía empotrada | `src/design-system/fonts/` |
| Juego de iconos de línea (sprite, 15 símbolos) | `src/lib/components/IconSprite.svelte`, montado una vez en `src/routes/+layout.svelte` (fuera de `AppShell`, para que resuelva también en `/onboarding`); se usa vía `<Icon name="…" />` |
| **Boceto aprobado v3** (4 pantallas, ambos temas) | `design/propuesta-redisenov2/mockups/smartdisk-v3.html` |
| Hoja de contacto de los iconos | `design/propuesta-redisenov2/mockups/icons-hoja-de-contacto.html` |
| Fichas de cambio del rediseño v3 | `design/propuesta-redisenov2/cambios/` · spec: `specs/002-rediseno-v3/` |
| Boceto v2 (referencia histórica) | `design/SmartDisk Monitor v2.dc.html` |
| Comandos y eventos que la UI puede llamar | `docs/ui-contract.md` |
| Verificadores que fallan la integración | `pnpm verify:tokens`, `pnpm verify:i18n` |

**Una sola copia.** El sistema de diseño vive en `src/` y en ningún otro sitio. `design/` contiene
únicamente los bocetos, que son referencia visual y no código reutilizable. Una segunda copia de
`tokens.css` o del catálogo diverge en silencio, y el consolidado acaba publicando valores caducos:
lo comprueba `pnpm verify:tokens` (ADR-029).

Las notas de arranque de la aplicación y las advertencias para quien programa están en el
**apéndice** al final de este documento.

---

### 1. Pila y convenciones de código

- **Tauri 2 + Rust** (backend privilegiado) · **Svelte 5 con runes** + **TypeScript** · **Tailwind CSS**.
- Nada de React. Nada de librerías de componentes de terceros: los componentes de `src/lib/components/`
  son el catálogo completo.
- El frontend **solo** llama comandos Tauri enumerados y tipados (`invoke("get_devices")`, …).
  Nunca construyas comandos, rutas o argumentos de `smartctl` desde la UI.
- Todo texto de interfaz pasa por i18n (`$lib/i18n`, claves `es` + `en`). Ningún literal suelto en JSX/markup.
- Runes: `let { … } = $props()`, `$state`, `$derived`, `$effect`. No uses `export let` ni stores para estado local.

### 2. Regla cero: los tokens

```
src/design-system/tokens.css     ← fuente única de verdad (importar una sola vez en el arranque)
src/design-system/tokens.json    ← misma información, legible por herramientas
tailwind.config.cjs              ← mapeo de tokens a utilidades
```

- **Prohibido** escribir un color, radio, sombra o tamaño de fuente literal en un componente.
  Usa la utilidad Tailwind (`bg-surface`, `text-fg-dim`, `rounded-xl`, `shadow-card`) o `var(--sdm-*)`.
- El tema se conmuta con `document.documentElement.dataset.theme = "light" | "dark"`; lo gestiona
  `$lib/design/theme.svelte.ts`. **Todo componente debe verse correcto en ambos temas sin condicionales.**
- Preferencia de tema y de idioma se persisten en la tabla `settings`, no en `localStorage`.
- **El acento es propio: morado «Ciruela»** (`#7a3f9d` claro / `#c79aec` oscuro), definido en
  `tokens.css`. Heredar el acento de Windows es un **interruptor de Ajustes → Apariencia, apagado de
  fábrica** (`settings.appearance.useSystemAccent`, ADR-035): al encenderlo, `applySystemAccent()`
  (`$lib/design/accent.ts`) sobrescribe los tres tokens de acento, siempre corregidos a AA por
  `accessibleAccent()`/`accentOnSurface()`; al apagarlo, `clearSystemAccent()` restaura el morado.
  Nunca codifiques un color de acento a mano. El acento **no** comunica salud.
- **`--sdm-on-accent` no es blanco en tema oscuro.** El acento oscuro es claro y el texto blanco
  encima daba 2,27:1. Todo texto o icono sobre el acento —o sobre un color de estado— usa
  `text-fg-onAccent` (`var(--sdm-on-accent)`), **nunca `text-white`**. Excepciones (son brillos, no
  tinta): el filo interior de `ProgressBar` en modo `display` y el punto del `Switch` activo.
- **Hay dos tokens de acento y no son intercambiables:**

  | Token | Uso | Contra qué se mide su contraste |
  |---|---|---|
  | `--sdm-accent` | **fondo**: botón primario, relleno de selección | contra `--sdm-on-accent` |
  | `--sdm-accent-fg` | **texto e iconos** sobre el material: enlaces, etiqueta seleccionada, serie principal de la gráfica | contra la superficie del tema |

  Un color no puede cumplir las dos cosas: el azul `#0078d4` que Windows trae de fábrica da 4,31:1
  como texto sobre el material claro, **por debajo de AA**. Barriendo el espacio sRGB completo, el
  65 % de los acentos posibles son ilegibles como texto en tema claro y el 50 % en oscuro
  (`tools/accent-check.py`, `docs/open-questions.md` §O).

  Por eso: **texto de acento ⇒ `text-accent-fg`. Fondo de acento ⇒ `bg-accent`.** Nunca al revés,
  y nunca `--sdm-accent` para pintar texto.

### 2.bis Material translúcido (lo propio de v2)

- Tres capas y nada más: `.sdm-material-chrome` (barra lateral y barra de herramientas),
  `.sdm-material` (tarjetas) y `.sdm-material-overlay` (diálogos y menús). **No escribas
  `backdrop-filter` a mano** ni inventes nuevos niveles de desenfoque.
- **No apiles materiales**: una tarjeta nunca contiene otra tarjeta. Los bloques internos usan
  `bg-glass-3` + `rounded-inner`.
- Toda superficie de material lleva su filo de 1 px (`shadow-edge`, es decir
  `inset 0 1px 0 var(--sdm-highlight)`) y su hairline (`border-hairline`). Sin ellos el cristal se ve plano.
- **Radios concéntricos**: exterior 18 → interior 13 → navegación 9 → controles en cápsula (999).
  El radio interior se calcula como exterior − padding; no mezcles radios al azar.
- **Contraste primero.** La translucidez es sutil por decisión: si un texto queda por debajo de AA sobre
  el material, sube la opacidad de la capa, nunca bajes el contraste del texto.
- El lienzo lleva un degradado muy tenue (`--sdm-bg` → `--sdm-bg-2`): es lo que da vida al desenfoque.
  No lo sustituyas por un color plano ni por un degradado de color saturado.
- Peso tipográfico máximo **600**. La jerarquía la aporta el material y el tamaño, no la grasa.
- **Tipografía de «display» (v3).** Para **cifras y titulares**, nunca para texto corrido, se usa la
  clase `.sdm-display` (`--sdm-font-display` + peso 600 + `tabular-nums` + `--sdm-tracking-display`).
  Sus usos: la cifra del héroe (`--sdm-text-hero`, 76 px), el progreso de una prueba
  (`--sdm-text-display`, 58 px), las cifras de `MetricCard`/`DiskCard`, el alias de la cabecera del
  detalle y el título de la barra de herramientas. `--sdm-font-display` hoy resuelve a la familia
  sans ya empotrada: no se empaqueta una segunda familia (ADR-034).
- **El riel de navegación (v3).** La `Sidebar` es un riel de `--sdm-rail-width` (74 px) solo con
  iconos; cada botón lleva `title` **y** `aria-label`. No lleva texto de sección ni lista de discos.
- **Crítico bermellón.** `--sdm-crit` se desplazó al bermellón (`#b03434` / `#ef8080`) para no
  confundirse con el acento morado. Sigue siendo el único rojo, y `unknown` sigue sin ser nunca rojo.
- La escala tipográfica **no se toca sin volver a medir**. Parece pequeña sobre el papel y no lo es:
  Instrument Sans tiene una altura de x de 0,5175 em frente a los 0,50 de Segoe UI, así que el cuerpo
  denso de 12,5 px equivale ópticamente a Segoe UI 12,9 px, por encima de los 12 px (9 pt) que
  Windows usa para el texto de interfaz. Medido, no estimado (`docs/open-questions.md` K.4).
- Movimiento: `duration-base` (220 ms) con `ease-sdm` (`cubic-bezier(.32,.72,0,1)`) en selección,
  cambio de pantalla y aparición de diálogos; `active:scale-[0.98]` en los botones. Nada decorativo,
  y todo anulado por `prefers-reduced-motion`.
- Sin soporte de `backdrop-filter` el material cae a `--sdm-solid` automáticamente: no añadas ramas propias.

#### Paleta semántica (no decorativa)

| Token | Significado | Uso |
|---|---|---|
| `ok` verde | Correcto | estado dentro de umbrales |
| `warn` ámbar | Advertencia | umbral cruzado, degradación no bloqueante |
| `crit` rojo | Crítico | requiere atención inmediata |
| `unknown` gris | Desconocido / no compatible / sin datos | **jamás rojo** |
| `accent` violeta | Acción primaria, selección, serie de datos principal | no comunica salud |

### 3. Catálogo de componentes

Importa siempre desde el barrel: `import { Card, DiskCard } from "$lib/components";`

| Componente | Para qué | Notas de uso obligatorias |
|---|---|---|
| `Card` | contenedor de toda información | radio xl + `shadow-card`; no anides sombras; ranura `leading` opcional (cuadrado de icono a la izquierda del título, v3); prop `border` (`hairline` por defecto, `crit` para una zona destructiva — solo el filo, el fondo no se tiñe) |
| `Button` | acciones | **una sola** `variant="primary"` por pantalla; `disabledReason` siempre que esté deshabilitado; `primary` escribe `text-fg-onAccent`, nunca `text-white` |
| `Icon` (v3) | símbolo de línea que hereda `currentColor` | uno de los 15 del sprite; `label` **obligatorio** si es el único portador de significado, si no `aria-hidden`; mapas semánticos en `$lib/design/icons.ts` |
| `Sparkline` (v3) | trazo de serie sin ejes ni etiqueta | un **`path` curvo** (spline monótona, `rutaSuave`) por tramo continuo, **nunca interpola** un hueco; `vector-effect="non-scaling-stroke"`; es contexto, no lectura |
| `HeroPanel` (v3) | dato dominante del panel con su serie de fondo | componente de pantalla (como `DiskCard`); la elección del disco protagonista vive en `selectHeroDisk()`, no en el componente; velo de legibilidad entre la curva y el texto |
| `OnboardingArt` (v3) | ilustración plana decorativa del asistente inicial | cuatro escenas (`welcome` / `disks` / `alerts` / `done`); solo `currentColor` y `var(--sdm-*)`, correcta en ambos temas sin condicionales; `aria-hidden` siempre (ADR-039); **solo se usa en `/onboarding`** |
| `StatusPill` / `StatusDot` | estado de salud | requieren `label`; el color nunca es el único portador de significado; `StatusPill` admite ranura de icono (`icon="auto"` ⇒ `healthIcon[state]`) |
| `MetricCard` | cifra destacada + procedencia | icono obligatorio + `sparkline` opcional; cifra con `.sdm-display` (peso 600, **no** 800); `value={null}` ⇒ "No disponible" **compuesto como texto en `text-lg`, no como cifra**. Bloque interno (`bg-glass-3` + `rounded-inner`), nunca material sobre material |
| `DataRow` | contador SMART etiqueta/valor/delta | color en el delta solo si significa algo |
| `CapacityBar` | ocupación de volumen | el color lo decide `capacityBarTone()` —imita al Explorador de Windows: rojo cuando queda poco espacio (≥ 91 % ocupado), ámbar ≥ 85 %—, no el llamante ni la severidad de la alerta `capacity.*` |
| `ProgressBar` | operación en curso | siempre con leyenda y tiempo restante; prop `emphasis` (`inline` por defecto, `display` para la prueba en curso) |
| `Sidebar` | navegación principal (riel de 74 px, v3) | material de chrome; solo iconos con `title`+`aria-label`; selección con material elevado e icono en acento, **nunca** barra de color lateral; navega con `<a href>`; sin lista de discos ni texto de estado global |
| `Toolbar` | barra de herramientas unificada | `title`/`subtitle` **de la ruta**; píldora de estado global con icono (única fuente); acción primaria; sin botón «?» (Acerca de va al riel) ni ranura de controles contextuales |
| `SegmentedControl` | intervalos 24 h / 7 d / 30 d / personalizado | |
| `DiskCard` | tarjeta de disco del panel | recibe `href`; cabecera de 52 px que hereda el color del estado con `sparkline` de temperatura de fondo (`temperatureSeries` opcional); dato ausente como «—» discreto, no «No disponible» a 23 px |
| `HealthDonut` | reparto de estados del equipo | acompañar de leyenda numérica. **En v3 sale del panel general** (lo sustituye el bloque «Reparto de estados», que con 2–4 discos se lee mejor); se conserva en el catálogo |
| `AlertCard` | grupo de alertas en lista | píldora de severidad con icono (`severityIcon[severity]`: `info→shield`, `warn→alert`, `crit→bolt`); contador `×N` en `.sdm-num`; claves técnicas solo en el detalle |
| `EventRow` | evento de Windows | nivel como **cuadrado de 26 px con icono** (`eventLevelIcon`) en el color del token, `aria-label` con el nombre del nivel — el color nunca viaja solo; altura de fila **fija en 42 px** (la `VirtualList` no recalcula); etiqueta "asociación inferida" a `text-2xs` sobre `bg-unknown-soft` cuando `mappingConfidence !== "exact"` |
| `TimeSeriesChart` | gráficas históricas | trazo curvo por tramo (comparte `rutaSuave`/`tramos` con `Sparkline`); huecos como huecos; umbral del fabricante discontinuo |
| `ConfirmDialog` | confirmación previa | declarar acción, destino, impacto y comando literal |
| `EmptyState` | vacío / no compatible / error de fuente | distingue los tres casos |
| `AppShell` | raíz de la aplicación | se monta una sola vez; contiene el lienzo con degradado y la región de scroll |
| `Toast` | aviso efímero no bloqueante | solo para confirmar acciones del usuario; **nunca** para alertas de salud, que van al centro de alertas |
| `Switch` | preferencia booleana de efecto inmediato | etiqueta a la izquierda, control a la derecha; nunca dentro de un formulario con botón Guardar |
| `Select` | elección entre 4 o más opciones excluyentes | por debajo de 4 opciones usa `RadioGroup` o `SegmentedControl` |
| `RadioGroup` | 2–4 opciones excluyentes con explicación | cada opción admite descripción; obligatorio para tema e idioma |
| `TextField` | entrada de texto o número | `suffix` para la unidad; validar en `onblur`, nunca en cada pulsación |
| `CodeOutput` | salida literal de un proceso auxiliar | monoespaciada, `white-space: pre`, scroll propio; **renderiza texto, jamás HTML**; botón de copiar obligatorio |

#### Autorizados y pendientes de construir

Estos cuatro patrones son necesarios para pantallas ya especificadas y **no requieren una decisión
nueva**: la regla de las ≥3 pantallas no les aplica. Siguen todos los requisitos de un componente
del catálogo (solo tokens, ambos temas, `null` admitido, etiqueta accesible, export en el barrel).

| Componente | Lo exige | Por qué no se puede componer |
|---|---|---|
| `DateRangePicker` | US-020, US-050 (intervalo "personalizado") | no hay ningún control de fecha en el catálogo |
| `FilterBar` | US-021 (filtrar eventos por disco, volumen, nivel y proveedor) | requiere selección múltiple, que `Select` no ofrece |
| `VirtualList` | US-021 (un servidor genera miles de eventos) | renderizar 5.000 `EventRow` bloquea la interfaz |
| `Tooltip` | `Button.disabledReason`, procedencia de métricas | hoy la norma exige el dato pero no hay dónde mostrarlo |

#### Cuándo crear un componente nuevo

Solo si (a) el patrón aparece en ≥3 pantallas y (b) no se puede expresar componiendo el catálogo.
Excepción ya autorizada: los cuatro componentes de la tabla "Autorizados y pendientes de construir"
no requieren nueva decisión, solo revisión visual antes de darlos por terminados.
Un componente nuevo debe: consumir solo tokens, funcionar en ambos temas, aceptar `null` en todo dato
opcional, tener etiqueta accesible y exportarse en `src/lib/components/index.ts`.

**Componentes añadidos en v3** (ADR-034): `Icon`, `Sparkline` y `HeroPanel`. Cada uno cumple el
criterio (a)+(b) y su justificación completa está en `design/propuesta-redisenov2/cambios/componentes/`
y en `specs/002-rediseno-v3/`. Ninguno del catálogo se elimina.

### 4. Reglas de composición de pantalla

0. **Medidas de ventana.** Hay tres números y no significan lo mismo:

   | | Valor | Para qué |
   |---|---|---|
   | Mínimo técnico | **1024 × 560** | `minWidth`/`minHeight` de `tauri.conf.json`. Nada puede romperse aquí |
   | Objetivo de diseño | **1280 × 720** | El tamaño contra el que se compone y se revisa |
   | Predeterminado | **1695 × 988** | Solo el **primer** arranque (`tauri.conf.json`). Después manda la geometría que el usuario dejó, que se recuerda en `settings` (ADR-040). Windows la acota si no cabe en la pantalla |

   El mínimo técnico no es un capricho: el escalado de Windows **no encoge el texto, encoge el
   espacio disponible en píxeles CSS**. Un portátil de 1920 × 1080 al 150 % deja una ventana máxima
   de 1280 × 672, y un 1366 × 768 al 125 % deja 1092 × 566. Con un mínimo de 720 de alto, en esas dos
   configuraciones la ventana no cabría en la pantalla. Medido, no estimado (`docs/open-questions.md` K.4).

   Debe seguir siendo correcta al 125 %, 150 % y 200 % de escalado.

0.bis **Degradación por ancho.** Dos umbrales, y solo dos:

   - Por debajo de **1180 px** la `Sidebar` se reduce a iconos (56 px), conservando el estado de cada
     disco en su punto de color. A 250 px fijos se comería la cuarta parte de una ventana de 1024.
   - La rejilla del panel es `repeat(auto-fill, minmax(460px, 1fr))`. **460, no 420**: por debajo de
     460 px una `DiskCard` no puede mostrar cuatro métricas sin recortar la más ancha ("684 GB" mide
     94 px a 27 px de cuerpo, y la celda se queda en 61 px).
1. Estructura: `Sidebar` fija a la izquierda (250 px) → `Toolbar` fija arriba de la columna derecha →
   región de contenido con `overflow: auto` y `padding: var(--sdm-space-6)`.
   **La ventana nunca recorta contenido en silencio**: si no cabe, la región hace scroll.
   El contenido pasa por debajo del chrome translúcido: no le pongas fondo opaco.
2. Rejilla: `display: grid` / `flex`. **Gap canónico entre tarjetas: `var(--sdm-space-5)` (18 px)**, que es
   el del boceto aprobado; `var(--sdm-space-4)` (16 px) para bloques dentro de una tarjeta y
   `var(--sdm-space-2)` (8 px) para elementos de una misma fila de controles.
   Nunca márgenes sueltos entre hermanos.
3. Jerarquía por pantalla: un título (`text-xl font-black`), una acción primaria, el resto secundario.
4. Densidad: aire generoso. Si una pantalla necesita más densidad, es señal de que sobra información.
5. Máximo dos niveles dentro de una tarjeta (`glass` → `glass-3`); `glass-2` queda para controles y bloques del chrome.
6. **La fila de métricas de una tarjeta es una rejilla, no un flex.**
   `grid-template-columns: repeat(auto-fit, minmax(104px, 1fr))`: con tarjetas anchas las cuatro
   métricas quedan en línea y con tarjetas estrechas se reorganizan solas, en vez de comprimirse
   hasta recortar. Una fila flex con `flex: 1 1 0` recorta en silencio, que es justo lo que prohíbe
   la regla 1.
6. Tamaño mínimo de objetivo interactivo: **30 px de alto (`--sdm-tap-min`)**, que es la altura de
   `--sdm-control-md`. Ningún control baja de ahí: `--sdm-control-sm` (30 px) es el suelo, no una excepción.
   Queda por encima de los 24 px que exige WCAG 2.2 AA (2.5.8) y por debajo de los 44 px táctiles,
   decisión consciente para una aplicación de escritorio con ratón. Texto mínimo 11 px, y solo dentro de píldoras.
7. Cifras: clase `.sdm-num` (`tabular-nums`) para que las columnas no bailen al actualizarse.
8. Todos los controles son cápsulas (`rounded-pill`). No hay botones rectangulares en v2.

### 5. Reglas de producto que la UI debe respetar

Estas no son estéticas: vienen de la especificación y su incumplimiento es un bug.

- **Nunca inventes ceros.** Dato ausente ⇒ `formatBytes/formatTemperature/...` devuelven "No disponible".
- **No compatible ≠ averiado.** Un disco USB, RAID o virtual sin SMART se presenta en gris como
  "Sin datos SMART", nunca en rojo y nunca como alerta activa.
- **Procedencia visible.** Cada métrica puede mostrar fuente (`smartctl`, `filesystem`, contador de rendimiento)
  y antigüedad de la última lectura válida. Si el dato es `stale`, dilo.
- **Inferencia etiquetada.** Una asociación evento→disco inferida se marca como inferida.
- **Alertas agrupadas.** Se muestra un grupo con contador y cronología de ocurrencias, no una fila por muestra.
  Estados: activa · reconocida · resuelta automáticamente · archivada. Silencio: 15 min · 1 h · 8 h · indefinido.
- **Confirmación antes de escribir o cargar.** Benchmark, chkdsk y autotest abren `ConfirmDialog` con impacto
  explícito (rendimiento, temperatura, escrituras del SSD) y comando literal cuando exista.
- **La UI no se bloquea.** Recopilaciones, exportaciones y pruebas van en el backend; la pantalla muestra
  progreso o actividad. Un colector caído degrada su tarjeta, no la aplicación.
- **Errores comprensibles + detalle técnico conservado**: frase humana visible, detalle en `<details>`.
- **Hora local en presentación, UTC en persistencia.** Unidades: bytes y °C en el dato; formato legible en la vista.
- **Contenido de eventos y dispositivos se renderiza como texto**, nunca como HTML.
- **Reconocer una alerta no cambia el color.** El estado de un disco es la peor severidad de sus
  alertas `active` **o** `acknowledged`; solo `resolved` y `archived` dejan de contar. Reconocer la
  saca de la lista de pendientes y le pone un distintivo, nada más: el color no puede mentir sobre el
  estado del hardware. El silencio es ortogonal y solo afecta a la notificación, jamás al color.
  Todo esto vive en `deviceState()` y `alertCountsTowardHealth()`; ninguna pantalla lo recalcula.
- **Un disco sin datos frescos es `unknown`, no `ok`.** No saber que algo está bien no es saber que
  está bien. `deviceState()` lo resuelve con su parámetro `hasFreshData`.
- **El acento heredado se corrige antes de aplicarse.** `accessibleAccent()` elige texto blanco o
  negro sobre el acento del usuario y lo oscurece si aún así no llega a AA, y `accentOnSurface()`
  deriva el tono legible como texto en el tema activo, prefiriendo la paleta que Windows ya expone.
  Nunca escribas `--sdm-on-accent` ni `--sdm-accent-fg` a mano, ni supongas que el texto sobre el
  acento es blanco. Verificado sobre los 262.144 colores del barrido: cero por debajo de AA.
- **El eje X de una gráfica es tiempo, no índice de muestra.** Las series no son equiespaciadas.
  `TimeSeriesChart` recibe `from`/`to` (el intervalo pedido, no el que cubren los datos) y dibuja
  como hueco todo salto mayor que 1,5× la cadencia, incluidos los extremos.
- **Toda gráfica declara su resolución.** Si se están viendo promedios de 5 minutos u horarios en
  vez de muestras crudas, se dice en el pie (`resolutionLabel`): un máximo promediado no es un pico.

### 6. Accesibilidad

- Contraste mínimo AA (4.5:1) para texto en ambos temas. Los tokens de texto y de salud están verificados
  contra el material de su tema **como texto de píldora a 11 px**: no los aclares, no bajes opacidades sobre
  texto y no pongas texto directamente sobre `glass-3`. Si necesitas más translucidez en una capa, sube la
  opacidad del material, nunca rebajes el color del texto.
- Foco visible en todo elemento interactivo (`:focus-visible` global en `tokens.css`; no lo anules).
  La regla se escribe **con la pseudoclase repetida**, `:focus-visible:focus-visible`, y eso no es
  un descuido: con una sola (0,1,0) empata en especificidad con cualquier utilidad de Tailwind
  —`shadow-edge`, `shadow-[...]`— y pierde por orden, porque las utilidades se generan después de
  `tokens.css`. Se midió: antes de arreglarlo, **ningún** botón mostraba anillo de foco, ni siquiera
  la variante `ghost`, y como la regla hace `outline: none`, los controles quedaban sin ningún
  indicador. Si añades otra regla de estado que compita con una utilidad, súbele la especificidad
  igual. Lo vigila `src/lib/components/Button.browser.test.ts`.
- **Un `role="img"` sin nombre accesible es peor que no ponerlo.** `StatusDot` emitía
  `aria-label=""` cuando no recibía etiqueta y un lector de pantalla anunciaba «imagen» y nada más.
  Regla: si hay etiqueta visible al lado, el gráfico es decorativo y va con `aria-hidden="true"`;
  si no la hay, lleva su propio nombre traducido. Lo detectó `axe` en `e2e/ui/a11y.spec.ts`.
- Navegación completa por teclado: pestañas con `role="tablist"`, diálogos con `role="dialog" aria-modal` y foco atrapado.
- Toda gráfica y todo anillo llevan `role="img"` con `aria-label` que resume el dato, y una lectura textual equivalente cerca.
- `prefers-reduced-motion` respetado globalmente; no añadas animaciones decorativas.

### 7. Pantallas y su composición aprobada

1. **Panel general** (v3) — `HeroPanel` con el disco que necesita atención (`selectHeroDisk()`) y su
   serie de temperatura de fondo; rejilla `repeat(auto-fill, minmax(272px, 1fr))` de `DiskCard`; fila
   inferior `1fr 300px` con «Sucesos del sistema» (`EventRow`) y «Reparto de estados» (composición de
   pantalla, sustituye a `HealthDonut` en el panel — `HealthDonut` sigue en el catálogo). La región
   entera hace scroll; **sin `VirtualList`** en la rejilla (véase `open-questions.md` §U). La lista de
   discos ya **no** vive en la `Sidebar` (riel de solo iconos, v3).
2. **Detalle de disco** (v3) — cabecera de identidad (`Card` de una fila: cuadrado de `Icon` con el
   color del estado, alias `.sdm-display`, `StatusPill` con icono, línea de identidad, botón «Probar
   disco»); fila de 4 `MetricCard` con icono y sparkline de 24 h; rejilla `1.6fr 1fr` con
   `TimeSeriesChart` de temperatura y panel de contadores con `DataRow`. El `SegmentedControl` de
   intervalo va **junto a la gráfica**, ya no en la `Toolbar`.
3. **Alertas** — lista de `AlertCard` (columna fija ~470 px) + detalle: severidad, titular, explicación humana,
   rejilla de hechos (los dos primeros — valor y umbral — en `text-metric` con `.sdm-display`), acciones
   (Reconocer / Silenciar / Archivar) y cronología de ocurrencias.
4. **Pruebas y diagnóstico** (v3) — **si hay una prueba en curso**, su bloque va arriba y a ancho
   completo: cabecera con píldora «Prueba en curso» + tipo de prueba `.sdm-display` + cifra de progreso
   a `text-display` (58 px, a `text-metric` por debajo de 1100 px) + botón Cancelar; `ProgressBar
   emphasis="display"`; rejilla de métricas en cuadros `bg-glass-3`; aviso de parada automática en
   `bg-warn-soft` con `Icon` (nunca un badge `text-white`). Debajo, las tres tarjetas de prueba (cada
   una con su cuadrado de `Icon`, `testIcon`), y el historial con columna de icono de estado. Sin
   prueba en curso, el bloque no se muestra y las tarjetas suben.
5. **Ajustes** — secciones apiladas, cada una en su `Card`; controles internos sobre `bg-glass-3`
   (no material sobre material). «Borrar todos los datos» separada al final con `border="crit"` y
   ~32 px extra de separación; el fondo no se tiñe.
6. **Asistente inicial** (`/onboarding`, US-002, v3) — **sin `AppShell`**: `+layout.svelte` omite el
   riel y la barra de herramientas en esta ruta. Cabecera propia de 56 px (logo, indicador de paso,
   «Omitir y usar los valores de fábrica» siempre visible), cuerpo `max-w-[1000px]` centrado, pie de
   navegación `sticky bottom-0` con `.sdm-material-chrome`. Cuatro pasos, uno por pantalla, cada uno
   con su escena de `OnboardingArt` (ADR-039): `welcome` y `done` centradas sobre el título;
   `disks` y `alerts` compactas junto al encabezado, ocultas por debajo de 720 px de cuerpo. El
   guardián de redirección vive en `+layout.ts` (`open-questions.md` §V).
7. **Informes**: hereda tokens; sin composición nueva.

#### Comportamiento con muchos discos

El boceto v2 está dibujado con cuatro discos, pero Windows Server entra en el alcance y un equipo
puede tener veinte o más. Reglas obligatorias, no opcionales:

- La lista de discos de la `Sidebar` tiene su propio `overflow-y: auto`; la navegación principal y el
  estado global **nunca** hacen scroll con ella.
- El panel general usa rejilla `repeat(auto-fill, minmax(272px, 1fr))` (v3; la `DiskCard` v3 encaja
  tres magnitudes y la barra de capacidad en 272 px).
- A partir de **12 discos monitorizados**, `DiskCard` usa su variante compacta: **sin la sparkline de
  temperatura de cabecera** (`conSparklines = devices.length <= 12`). Es también lo que mantiene
  SC-006 sin virtualizar la rejilla (`open-questions.md` §U); medido en `e2e/ui/rendimiento.spec.ts`.
- El «Reparto de estados» y el recuento cuentan solo los discos monitorizados. Los excluidos por el
  usuario no aparecen; se listan aparte, como exige US-011.

### 8. Definición de terminado para una pantalla

- [ ] Solo tokens; cero literales de color/tamaño; cero `backdrop-filter` escrito a mano.
- [ ] Correcta en tema claro y oscuro, con el acento del sistema y con el azul de respaldo.
- [ ] Correcta en la ventana mínima de la app sin recortes, y legible con el material sobre contenido denso.
- [ ] Textos en `es` y `en`.
- [ ] Estados diseñados: cargando, vacío, no compatible, error de fuente, dato obsoleto.
- [ ] Datos ausentes como "No disponible"; ningún cero inventado.
- [ ] Teclado y foco verificados; `aria-label` en gráficas e iconos.
- [ ] Acciones con carga o escritura confirmadas con impacto explícito.
- [ ] Sin permisos Tauri nuevos ni comandos genéricos.
- [ ] Verificada a 1024 × 560 (mínimo técnico) y a 1280 × 720 (objetivo de diseño), con la barra
      lateral colapsada y sin un solo recorte silencioso.
- [ ] Verificada con un valor ausente en cada métrica: "No disponible" cabe y se distingue de una cifra.
- [ ] Verificada con un acento del sistema claro (por ejemplo el amarillo `#ffb900`) **y en los dos
      temas**: el texto sobre el acento y el texto de acento sobre el material cumplen AA, y el
      cambio de tema recalcula `--sdm-accent-fg`.
- [ ] Verificada con 20 discos y con 5.000 eventos, sin bloqueo perceptible al desplazarse.
- [ ] Ni un solo literal de interfaz fuera de `es.json` / `en.json`, incluidos `aria-label` y títulos.

---

### Apéndice A. Arranque de la aplicación

La base es **SvelteKit con `adapter-static` y SSR desactivado** (ADR-014). `$lib` ya apunta a
`src/lib`: no toques el alias.

`tokens.css` es la **única** importación de CSS global y va en `src/routes/+layout.svelte`, antes
del primer render:

```ts
import "../design-system/tokens.css";
import { invoke } from "@tauri-apps/api/core";
import { theme } from "$lib/design/theme.svelte";
import { applySystemAccent } from "$lib/design/accent";
import { i18n } from "$lib/i18n";

const s = await invoke<AppearanceSettings>("get_appearance_settings");
theme.init(s.theme);
i18n.init(s.language, s.systemLocale); // el locale viene del backend, no de navigator
await applySystemAccent();
```

`i18n.init()` fija además `<html lang>`, e `i18n.formatLocale` es el locale que usan **todas** las
funciones de `format.ts`: los números siguen al idioma de la aplicación, no al de Windows.

La aplicación se monta con `AppShell` + `Sidebar` + `Toolbar`; ninguna pantalla monta su propio
chrome. El estado inicial de cada pantalla llega por `load` en `+page.ts`, no por `onMount`; las
actualizaciones vienen después por eventos. **No hagas sondeo con `setInterval`**: el backend empuja
(ADR-015). Los comandos y eventos disponibles son los de `docs/ui-contract.md`, que es el normativo:
la UI solo llama comandos enumerados.

### Apéndice B. Notas para quien programe

Errores que se cometen aunque las reglas de arriba estén leídas:

- Ningún literal de color, radio, sombra o tamaño en un componente: utilidad Tailwind o `var(--sdm-*)`.
- Nunca escribas `backdrop-filter` a mano: usa `.sdm-material`, `.sdm-material-chrome`, `.sdm-material-overlay`.
- Los tokens de texto y de salud están verificados a 4.5:1 sobre el material de su tema. No los aclares.
- Un dato ausente es "No disponible"; un dispositivo sin SMART es gris, nunca rojo ni alerta activa.
- Toda acción que escriba datos o genere carga pasa por `ConfirmDialog` con impacto y comando literal.
- Contenido procedente de eventos o dispositivos se renderiza como texto, jamás como HTML.
- Reconocer una alerta **no** devuelve el disco a verde: el color lo decide `deviceState()`, que
  cuenta las alertas `active` y `acknowledged`. El silencio nunca toca el color.
- El acento del sistema pasa por `accessibleAccent()` antes de aplicarse; no supongas texto blanco
  sobre el acento, usa `--sdm-on-accent`.
- `TimeSeriesChart` necesita `from`/`to` además de los puntos: el eje es tiempo real, y el intervalo
  pedido debe verse entero aunque falten datos.
- **La navegación se hace con enlaces, no con callbacks.** `DiskCard` recibe `href` y `Sidebar`
  recibe secciones con su `href`: un `onclick` con `goto()` rompe el ctrl+clic, el menú contextual y
  el anuncio como enlace de un lector de pantalla.
- **Un enlace que envuelve un bloque entero** (tarjeta de disco, fila de suceso del panel) lleva la
  clase `sdm-block-link`: no se subraya al pasar el ratón ni muestra el cursor de mano. Es una zona
  pulsable, no texto; debe comportarse como una lista nativa de Windows, no como una página web. El
  subrayado en `:hover` y el `cursor: pointer` se reservan para los enlaces **de texto en línea**
  («Ver todos», «Ver el suceso»).
- **Realce de hover de una zona pulsable de bloque** (tarjeta de disco, fila de suceso, **grupo de
  alerta**): clase `sdm-hover-bloque` de `tokens.css`. Pinta una **pátina del violeta de acento a
  media intensidad** (`--sdm-accent-soft` al 50 %) mediante una capa `::after` — no un aclarado, y
  no un `hover:bg-*`: el `::after` hace falta porque el material de `Card` taparía cualquier fondo
  del propio elemento y `overflow-hidden` recortaría su sombra. Hereda el radio del elemento (quien
  la use fija su `border-radius`). Es la única señal de que la zona es pulsable, igual que las
  listas del Explorador o de Configuración de Windows. Un elemento **seleccionado** no la lleva: ya
  lo marca su borde de acento.
- **Todo control pulsable tiene estado de hover visible.** Los botones cápsula (`Button`) lo traen
  por variante; los controles de formulario que no lo tenían (`Select`, `Switch`) ganan
  `hover:border-fg-faint` (+ `hover:bg-glass` en `Select`); los selectores de segmento
  (`SegmentedControl`, `FilterBar`) realzan el fondo del segmento inactivo con `hover:bg-glass-2`
  además del texto. El riel (`Sidebar`) y las opciones de `RadioGroup` ya realzaban con `bg-glass-3`
  / `bg-glass`.
- **Indicador de navegación.** `AppShell` pinta una barra fina (2 px) pegada al borde superior de la
  ventana mientras `navigating` (de `$app/state`) sea no nulo: `role="progressbar"`, color
  `bg-accent`, con un `animation-delay` de ~150 ms para que una navegación instantánea no la haga
  parpadear (bajo `prefers-reduced-motion` el retardo sigue vigente; solo se anula el avance). Es la
  red de seguridad para cuando un `load` tarda —no sustituye a que la interfaz responda al instante,
  que es lo normal tras ADR-042—.
- **`ConfirmDialog` con `dismissible`** muestra una cruz de cerrar en la esquina. Se usa solo en
  diálogos **informativos** (Acerca de), donde cerrar y «cancelar» son lo mismo; una confirmación
  real de escritura/carga no la lleva — se decide con sus botones.

### Apéndice C. Pantallas pendientes de diseño

Todas tienen ya criterios de aceptación en `docs/user-stories.md` (épica H y US-070 a US-074); lo
que falta es la composición visual, no la definición funcional.

- **Informes** (US-050): selector de intervalo, resumen de contenido y destino de exportación.
- **Ajustes**: apariencia, frecuencias, umbrales, retención, comportamiento al cerrar, borrado de datos.
- **Asistente inicial** (US-002): detección, exclusión de discos y alias.
- **Acerca de** (US-061).

**Icono de la bandeja del sistema** — primer paso visual hecho (`platform/bandeja.rs`,
`open-questions.md` J.53); el rediseño fino sigue pendiente. Se genera en memoria, sin fichero
`.ico`: un **tile redondeado del color de estado** (los cuatro de la regla B.5: verde, ámbar, rojo,
gris) con un **glifo que también cambia con el estado** —cilindro de datos lleno (todo en orden),
con «!» (advertencia), con «×» (crítico), hueco (sin datos / sin discos / fallo de recopilador),
dos barras (en pausa)—: a 16 px el color y la forma van juntos (§VII). El texto emergente es
`«SmartDisk Monitor — <resumen>»` (`tray.tooltip`), nunca el resumen a secas: entre muchos iconos
de bandeja tiene que decir de quién es.

Casi todo se compone con el catálogo actual (`Switch`, `Select`, `TextField`, `RadioGroup`,
`SegmentedControl`, `ConfirmDialog`, `EmptyState`, `CodeOutput`). Las excepciones ya están
autorizadas y no requieren decisión nueva: `DateRangePicker`, `FilterBar`, `VirtualList` y `Tooltip`
(§3, "Autorizados y pendientes de construir").


---

# 14. Bocetos del sistema de diseño

Fichero de origen: `design/README.md`

Referencia **visual**, no código. Aquí solo hay lienzos navegables que enseñan cómo debe verse la
aplicación; lo que se implementa vive en `src/`.

| Fichero | Qué es |
|---|---|
| `SmartDisk Monitor v2.dc.html` | **APROBADO.** Cuatro pantallas (panel general, detalle de disco, alertas, pruebas), en tema claro y oscuro, con el diálogo de confirmación incluido. Es la referencia contra la que se revisa una pantalla |
| `Sistema de diseno SmartDisk.dc.html` | Guía visual de tokens y componentes. Sigue en estilo v1: **pendiente de refresco a v2**. Ante una diferencia con el boceto aprobado, manda el aprobado |
| `Bocetos SmartDisk Monitor.dc.html` | Exploración inicial de las direcciones 1a y 1b. Referencia histórica: no se implementa nada de aquí |

`support.js` es el runtime que los tres necesitan para funcionar, y `.thumbnail` la miniatura de
previsualización. Ninguno de los dos se edita a mano.

### Cómo se abren

Doble clic en el `.html`, o desde la terminal:

```powershell
Invoke-Item ".\design\SmartDisk Monitor v2.dc.html"
```

Los tres cargan `support.js` desde la misma carpeta, así que **no funcionan si se copian sueltos** a
otro sitio.

### Lo que estos ficheros NO son

- **No son la fuente de verdad visual.** Esa es `src/design-system/tokens.css`. Si un boceto y un
  token discrepan, gana el token, y la discrepancia se anota en `docs/open-questions.md`.
- **No contienen código reutilizable.** Su marcado es de la herramienta de diseño; el catálogo real
  es `src/lib/components/`.
- **No son normativos por sí solos.** Las reglas están escritas en `docs/ui-design.md`, que es lo
  vinculante. El boceto muestra el resultado; el documento dice por qué y con qué límites.

### Procedencia

El diseñador entregó estos bocetos dentro del paquete `Design-system/`, que además traía una copia
del sistema de diseño (tokens, componentes, i18n). Esa copia se integró en `src/` y **se eliminó de
aquí** porque las dos versiones habían empezado a divergir en silencio: el motivo completo y la
divergencia medida están en ADR-029 (`docs/decisions.md`). El paquete original íntegro sigue
disponible en el historial de git, anterior a ese cambio.


---

# 15. Sistema de diseño: principios

Fichero de origen: `src/design-system/README.md`

Versión aprobada: **v2, material translúcido** (evolución de la dirección 1b). Una herramienta de
administración que se lee de un vistazo: capas de cristal sutil que dejan intuir el contenido detrás,
profundidad en lugar de líneas divisorias, color reservado para el significado y cifras grandes donde importa.

El acento lo hereda del color de acento de Windows; la app solo aporta el respaldo.

### Principios

1. **El color es información.** Verde/ámbar/rojo/gris solo significan salud. El violeta de acento significa
   acción o selección, nunca estado.
2. **La ausencia de dato es un dato.** "No disponible" es una respuesta legítima y frecuente; un cero inventado
   es un error de producto.
3. **Calma sobre densidad.** Aire, pocas líneas divisorias, jerarquía por material y tamaño antes que por bordes o grasa tipográfica (peso máximo 600).
4. **Cada número lleva su procedencia.** Fuente y antigüedad acompañan a la métrica.
5. **Nada ocurre sin avisar.** Cualquier operación que escriba o caliente el disco se explica antes.

### Anatomía

- **Tipografía**: Instrument Sans. 600 como peso máximo, 500 para etiquetas, 400 para prosa.
  Escala: 11 · 12 · 12,5 · 13,5 · 14,5 · 20 · 21 · 27 px. Cifras siempre con `.sdm-num`.
- **Espaciado**: escala de 4. `18 px` entre tarjetas, `20 px` de margen de pantalla.
- **Radios concéntricos**: 18 (ventana y tarjeta) → 13 (bloque interno) → 9 (navegación) → cápsula (controles).
  Interior = exterior − padding.
- **Material**: tres capas — `.sdm-material-chrome` (barra lateral y barra de herramientas, desenfoque 28),
  `.sdm-material` (tarjetas, 24), `.sdm-material-overlay` (diálogos, 44). Cada una con hairline y filo
  superior de 1 px. Sin soporte de `backdrop-filter`, caen a `--sdm-solid`.
- **Elevación**: `--sdm-shadow` para tarjetas, `--sdm-shadow-lift` para diálogos y ventana. Nada más.
- **Movimiento**: 220 ms con `cubic-bezier(.32,.72,0,1)`; los botones se hunden un 2 % al pulsar.
- **Capas de color**: `bg`→`bg-2` (lienzo con degradado tenue) → `glass` (material) → `glass-2` (controles)
  → `glass-3` (pistas y bloques internos).

### Archivos

| Archivo | Contenido |
|---|---|
| `tokens.css` | variables `--sdm-*`, temas claro/oscuro, base y `:focus-visible` |
| `tokens.json` | los mismos valores en formato legible por herramientas |
| `../tailwind.config.cjs` | mapeo token → utilidad |
| `../src/lib/design/` | tipos, formateo, mapa de salud, control de tema y acento del sistema |
| `../src/lib/components/` | catálogo Svelte |
| `../AGENTS.md` | reglas de uso obligatorias para agentes |

### Arranque

```ts
// src/main.ts
import "../design-system/tokens.css";
import { theme } from "$lib/design/theme.svelte";
import { invoke } from "@tauri-apps/api/core";

import { applySystemAccent } from "$lib/design/accent";

const pref = await invoke<"light" | "dark" | "system">("get_theme_preference");
theme.init(pref);
await applySystemAccent(); // hereda el color de acento de Windows; si falla, se mantiene el respaldo
```


---

# 16. tokens.css — fuente única de verdad visual

Fichero de origen: `src/design-system/tokens.css`

```css
/* SmartDisk Monitor — tokens de diseño v2 "material translúcido"
   Fuente única de verdad. Nunca escribas un valor literal en un componente: usa var(--sdm-*)
   o la utilidad Tailwind equivalente. Todo token de color existe en tema claro y oscuro. */

/* Tipografía empotrada: la aplicación NO hace peticiones de red (spec §11, ADR-007).
   Los ficheros viven en ./fonts/; véase fonts/README.md para su procedencia y su licencia
   (SIL Open Font License 1.1, recogida en THIRD_PARTY_NOTICES.md).

   Dos subconjuntos, igual que los sirve el proyecto original: `latin` cubre español e inglés y es
   el que se carga casi siempre; `latin-ext` solo se descarga si aparece un carácter de su rango,
   cosa que puede pasar con el texto original de un evento de Windows. Al ser ficheros locales el
   coste de tener los dos es nulo, pero el `unicode-range` evita decodificar el que no hace falta.

   `font-weight: 400 600` declara menos rango del que el fichero admite (llega a 700) a propósito:
   así, si alguien pide 700, el navegador lo limita a 600 en vez de sintetizar una negrita falsa.
   El tope de 600 es norma de v2 (AGENTS.md §2.bis). */
@font-face {
  font-family: "Instrument Sans";
  src: url("./fonts/InstrumentSans-latin.woff2") format("woff2");
  font-weight: 400 600;
  font-style: normal;
  font-stretch: 100%;
  font-display: swap;
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304,
    U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}
@font-face {
  font-family: "Instrument Sans";
  src: url("./fonts/InstrumentSans-latin-ext.woff2") format("woff2");
  font-weight: 400 600;
  font-style: normal;
  font-stretch: 100%;
  font-display: swap;
  unicode-range: U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304, U+0308,
    U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB, U+20AD-20C0, U+2113,
    U+2C60-2C7F, U+A720-A7FF;
}

:root {
  /* ---- Tipografía ---- */
  --sdm-font-sans: "Instrument Sans", system-ui, "Segoe UI Variable", "Segoe UI", sans-serif;
  --sdm-font-mono: ui-monospace, "Cascadia Mono", Consolas, monospace;
  /* Familia de "display" para cifras y titulares grandes (v3, ADR-034). Hoy resuelve a la familia
     sans ya empotrada: se descartó empaquetar una segunda familia (Bricolage Grotesque) para no
     ampliar la superficie de un binario privilegiado. `.sdm-display` es el único punto de cambio si
     esa decisión se revisa. */
  --sdm-font-display: var(--sdm-font-sans);

  --sdm-text-2xs: 0.6875rem;    /* 11px  etiquetas de píldora y unidades */
  --sdm-text-xs: 0.75rem;       /* 12px  metadatos */
  --sdm-text-sm: 0.78125rem;    /* 12.5px cuerpo denso */
  --sdm-text-base: 0.84375rem;  /* 13.5px título de tarjeta */
  --sdm-text-lg: 0.90625rem;    /* 14.5px título de barra de herramientas */
  --sdm-text-xl: 1.25rem;       /* 20px  título de pantalla */
  --sdm-text-2xl: 1.3125rem;    /* 21px  titular de alerta */
  --sdm-text-metric: 1.6875rem; /* 27px  cifra grande */
  --sdm-text-display: 3.625rem; /* 58px  cifra de progreso de prueba (solo con .sdm-display) */
  --sdm-text-hero: 4.75rem;     /* 76px  temperatura del héroe del panel (solo con .sdm-display) */

  /* En v2/v3 el peso máximo es 600: el material aporta la jerarquía, no la grasa tipográfica. */
  --sdm-weight-regular: 400;
  --sdm-weight-medium: 500;
  --sdm-weight-semibold: 600;

  --sdm-tracking-tight: -0.02em;
  --sdm-tracking-metric: -0.03em;
  --sdm-tracking-display: -0.045em;  /* acompaña a .sdm-display en cifras a partir de 22px */

  /* ---- Espaciado (escala de 4; v2 respira más) ---- */
  --sdm-space-1: 4px;
  --sdm-space-2: 8px;
  --sdm-space-3: 12px;
  --sdm-space-4: 16px;
  --sdm-space-5: 18px;   /* gap canónico entre tarjetas */
  --sdm-space-6: 20px;   /* padding de pantalla */
  --sdm-space-8: 32px;

  /* ---- Dimensiones de composición fijas (v3) ---- */
  --sdm-rail-width: 74px;    /* ancho del riel de la Sidebar (sustituye a los 250px de v2) */
  --sdm-hero-height: 246px;  /* alto del HeroPanel del panel general */

  /* ---- Radios concéntricos ----
     Regla: el radio interior = radio exterior − padding. 18 → 13 → 9 → cápsula. */
  --sdm-radius-window: 18px;
  --sdm-radius-card: 18px;
  --sdm-radius-inner: 13px;
  --sdm-radius-nav: 9px;
  --sdm-radius-pill: 999px;   /* todos los controles son cápsulas */

  /* ---- Alturas de control ---- */
  --sdm-control-sm: 30px;   /* suelo absoluto: nunca por debajo de --sdm-tap-min */
  --sdm-control-md: 30px;   /* barra de herramientas */
  --sdm-control-lg: 35px;
  --sdm-tap-min: 30px;

  /* ---- Material ---- */
  --sdm-blur-chrome: 28px;   /* barra lateral y barra de herramientas */
  --sdm-blur-card: 24px;
  --sdm-blur-overlay: 44px;  /* diálogos */
  --sdm-saturate: 180%;

  /* ---- Iconografía de línea (v3) ----
     El juego de iconos hereda currentColor y no fija ningún color. Solo fija estos dos valores. */
  --sdm-icon-stroke: 1.7;   /* stroke-width de todos los iconos de línea */
  --sdm-icon-size: 24px;    /* viewBox de referencia; se renderizan a 12-28px */

  /* ---- Movimiento: elástico y breve ---- */
  --sdm-duration-fast: 140ms;
  --sdm-duration-base: 220ms;
  --sdm-duration-overlay: 320ms;
  --sdm-ease: cubic-bezier(0.32, 0.72, 0, 1);
}

/* =======================  TEMA CLARO  ======================= */
:root,
[data-theme="light"] {
  color-scheme: light;

  /* v3 — paleta "Ciruela" (ADR-034). Neutros malva y acento morado de tinta. Ratios de contraste
     medidos sobre el material compuesto en docs/open-questions.md; mínimo exigido 4,5:1. */
  --sdm-bg: #efeaf1;
  --sdm-bg-2: #e6dfe9;         /* extremo del degradado del lienzo */

  /* Capas de material. `glass` lleva backdrop-filter; `solid` es el respaldo sin soporte. */
  --sdm-glass: rgba(255, 255, 255, 0.72);
  --sdm-glass-2: rgba(255, 255, 255, 0.5);
  --sdm-glass-3: rgba(124, 114, 128, 0.1);   /* pistas de barra, bloques internos */
  --sdm-solid: #fdfcfe;

  --sdm-hairline: rgba(30, 23, 35, 0.09);
  --sdm-highlight: rgba(255, 255, 255, 0.9); /* brillo superior de 1px */
  --sdm-scrim: rgba(14, 10, 16, 0.36);       /* fondo de diálogo */

  --sdm-text: #1e1723;
  --sdm-text-dim: #635a6b;
  --sdm-text-faint: #6c6274;
  --sdm-on-accent: #ffffff;

  /* Acento: se sobreescribe en runtime con el color de acento de Windows (véase accent.ts).
     Estos valores son el respaldo, el azul predeterminado de Windows 11.

     Hay DOS tokens de acento porque hay dos usos con requisitos opuestos:
       --sdm-accent     va de FONDO (botón primario). Su contraste se mide contra --sdm-on-accent.
       --sdm-accent-fg  va de TEXTO sobre el material (enlaces, selección, serie de la gráfica).
                        Su contraste se mide contra la superficie, que es casi blanca en claro y
                        casi negra en oscuro, así que NO puede ser el mismo color en ambos temas.
     Un solo color no sirve para las dos cosas: el azul #0078d4 de Windows da 4,31:1 como texto
     sobre el material claro, por debajo de AA. Medido en tools/accent-check.py. */
  /* Acento morado de tinta (v3). #7a3f9d da 6,53:1 sobre el material claro, así que sirve de fondo
     (--sdm-accent) y de texto (--sdm-accent-fg). Con el interruptor "usar el acento de Windows"
     encendido, accent.ts sobrescribe estos tres tokens en runtime. */
  --sdm-accent: #7a3f9d;
  --sdm-accent-hi: #8b4bb0;    /* extremo claro del degradado vertical del botón */
  --sdm-accent-soft: rgba(122, 63, 157, 0.12);
  --sdm-accent-fg: #7a3f9d;    /* 6,53:1 sobre el material claro */

  /* Salud suavizada: menos saturación para convivir con el material, manteniendo 4.5:1 como texto
     de píldora sobre el material claro. Verificado con AA; no los aclares. El crítico se desplaza al
     bermellón para no confundirse con el acento morado. */
  --sdm-ok: #2f7256;
  --sdm-ok-soft: rgba(67, 144, 111, 0.13);
  --sdm-warn: #7d5619;
  --sdm-warn-soft: rgba(183, 129, 58, 0.14);
  --sdm-crit: #b03434;
  --sdm-crit-soft: rgba(176, 52, 52, 0.12);
  --sdm-unknown: #635c69;
  --sdm-unknown-soft: rgba(99, 92, 105, 0.11);

  --sdm-shadow: 0 1px 1px rgba(24, 18, 28, 0.05), 0 8px 22px -14px rgba(24, 18, 28, 0.28);
  --sdm-shadow-lift: 0 2px 4px rgba(24, 18, 28, 0.06), 0 24px 60px -22px rgba(24, 18, 28, 0.42);
  --sdm-focus-ring: 0 0 0 3px color-mix(in srgb, var(--sdm-accent) 40%, transparent);
}

/* =======================  TEMA OSCURO  ======================= */
[data-theme="dark"] {
  color-scheme: dark;

  /* v3 — paleta "Ciruela" en oscuro. Atención: --sdm-on-accent deja de ser blanco (daba 2,27:1
     sobre el acento claro); pasa a tinta. Todo texto sobre el acento usa --sdm-on-accent, nunca
     text-white (ADR-034, docs/00-tokens §2). */
  --sdm-bg: #130f16;
  --sdm-bg-2: #19141d;

  --sdm-glass: rgba(48, 42, 52, 0.66);
  --sdm-glass-2: rgba(64, 56, 68, 0.42);
  --sdm-glass-3: rgba(255, 255, 255, 0.06);
  --sdm-solid: #1c1620;

  --sdm-hairline: rgba(255, 255, 255, 0.09);
  --sdm-highlight: rgba(255, 255, 255, 0.13);
  --sdm-scrim: rgba(0, 0, 0, 0.52);

  --sdm-text: #f4f0f6;
  --sdm-text-dim: #a89eb0;
  --sdm-text-faint: #a29aa8;
  --sdm-on-accent: #20132a;    /* 7,80:1 sobre el acento oscuro */

  --sdm-accent: #c79aec;
  --sdm-accent-hi: #d4aef5;
  --sdm-accent-soft: rgba(199, 154, 236, 0.18);
  --sdm-accent-fg: #c79aec;    /* 6,89:1 sobre el material oscuro */

  --sdm-ok: #6cc79c;
  --sdm-ok-soft: rgba(108, 199, 156, 0.16);
  --sdm-warn: #e0b473;
  --sdm-warn-soft: rgba(224, 180, 115, 0.16);
  --sdm-crit: #ef8080;
  --sdm-crit-soft: rgba(239, 128, 128, 0.16);
  --sdm-unknown: #a8a0b0;
  --sdm-unknown-soft: rgba(168, 160, 176, 0.14);

  --sdm-shadow: 0 1px 1px rgba(0, 0, 0, 0.3), 0 10px 26px -16px rgba(0, 0, 0, 0.7);
  --sdm-shadow-lift: 0 2px 6px rgba(0, 0, 0, 0.4), 0 28px 70px -24px rgba(0, 0, 0, 0.85);
  --sdm-focus-ring: 0 0 0 3px color-mix(in srgb, var(--sdm-accent) 50%, transparent);
}

/* =======================  BASE  ======================= */
html,
body {
  margin: 0;
  height: 100%;
}

body {
  /* El lienzo lleva un degradado muy tenue: es lo que da vida al desenfoque de las capas. */
  background: linear-gradient(160deg, var(--sdm-bg) 0%, var(--sdm-bg-2) 100%);
  color: var(--sdm-text);
  font-family: var(--sdm-font-sans);
  font-size: var(--sdm-text-sm);
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
  overflow: hidden;
}

/* Texto de acento: SIEMPRE --sdm-accent-fg, nunca --sdm-accent. */
a {
  color: var(--sdm-accent-fg);
  text-decoration: none;
}
a:hover {
  color: var(--sdm-accent-fg);
  text-decoration: underline;
}

/* Enlaces que envuelven un bloque entero (tarjeta de disco, fila de suceso del panel): no son
   texto, son una zona pulsable. No se subrayan al pasar el ratón ni muestran el cursor de mano —
   se comportan como una lista nativa de Windows (Explorador, Configuración), no como una página
   web. La doble pseudoclase iguala la especificidad de `a:hover` (0,2,x) y gana por orden. */
a.sdm-block-link,
a.sdm-block-link:hover {
  color: inherit;
  text-decoration: none;
  cursor: default;
}

/* Realce al pasar el ratón para una zona pulsable de bloque (tarjeta de disco, fila de suceso,
   grupo de alerta): una pátina del violeta de acento a media intensidad (`--sdm-accent-soft` al
   50 %), no un aclarado. Va en `::after` para poder graduar la opacidad y para superponerse al
   material que a veces tapa el fondo del propio elemento (`Card` dentro de la tarjeta de disco).
   Hereda el radio del elemento: quien la use fija su `border-radius`. `prefers-reduced-motion` ya
   anula la transición globalmente (más abajo). */
.sdm-hover-bloque {
  position: relative;
}
.sdm-hover-bloque::after {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: var(--sdm-accent-soft);
  opacity: 0;
  transition: opacity var(--sdm-duration-fast) var(--sdm-ease);
  pointer-events: none;
}
.sdm-hover-bloque:hover::after {
  opacity: 0.5;
}

/* La doble pseudoclase NO es un descuido: sube la especificidad a (0,2,0) a propósito.
   Con `:focus-visible` a secas (0,1,0) empata con cualquier utilidad de Tailwind —`shadow-edge`,
   `shadow-[...]`— y pierde por orden de aparición, porque las utilidades se generan después de
   este fichero. Resultado medido antes del arreglo: el anillo de foco no aparecía en NINGUNA
   variante de Button, ni siquiera en `ghost`. Como aquí se hace `outline: none`, eso dejaba a los
   controles sin ningún indicador de foco: WCAG 2.4.7 incumplido en todo el catálogo.
   Lo vigila `src/lib/components/Button.browser.test.ts`. */
:focus-visible:focus-visible {
  outline: none;
  box-shadow: var(--sdm-focus-ring);
  border-radius: var(--sdm-radius-pill);
}

::selection {
  background: var(--sdm-accent-soft);
  color: var(--sdm-text);
}

/* ---- Utilidades de material ----
   Úsalas en vez de repetir backdrop-filter a mano. */
.sdm-material {
  background: var(--sdm-glass);
  backdrop-filter: blur(var(--sdm-blur-card)) saturate(var(--sdm-saturate));
  -webkit-backdrop-filter: blur(var(--sdm-blur-card)) saturate(var(--sdm-saturate));
  border: 1px solid var(--sdm-hairline);
  box-shadow: var(--sdm-shadow), inset 0 1px 0 var(--sdm-highlight);
}

.sdm-material-chrome {
  background: var(--sdm-glass);
  backdrop-filter: blur(var(--sdm-blur-chrome)) saturate(var(--sdm-saturate));
  -webkit-backdrop-filter: blur(var(--sdm-blur-chrome)) saturate(var(--sdm-saturate));
  box-shadow: inset 0 1px 0 var(--sdm-highlight);
}

.sdm-material-overlay {
  background: var(--sdm-glass);
  backdrop-filter: blur(var(--sdm-blur-overlay)) saturate(190%);
  -webkit-backdrop-filter: blur(var(--sdm-blur-overlay)) saturate(190%);
  border: 1px solid var(--sdm-hairline);
  box-shadow: var(--sdm-shadow-lift), inset 0 1px 0 var(--sdm-highlight);
}

/* Respaldo cuando el motor no soporta backdrop-filter: material opaco, mismo layout. */
@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px))) {
  .sdm-material,
  .sdm-material-chrome,
  .sdm-material-overlay {
    background: var(--sdm-solid);
  }
}

/* Cifras alineadas: obligatorio en toda métrica y toda columna numérica. */
.sdm-num {
  font-variant-numeric: tabular-nums;
  letter-spacing: var(--sdm-tracking-tight);
}

/* Cifras y titulares de "display" (v3): nunca para texto corrido.
   Da jerarquía sin tocar el peso (sigue topado en 600) ni inventar una negrita 700. */
.sdm-display {
  font-family: var(--sdm-font-display);
  font-weight: var(--sdm-weight-semibold);
  font-variant-numeric: tabular-nums;
  letter-spacing: var(--sdm-tracking-display);
  line-height: 1;
}

/* ---- Comportamiento de aplicación nativa (docs/open-questions.md J.45) ----
   Tauri renderiza con un motor web, pero la aplicación debe sentirse como un programa de
   escritorio: sin selección de texto arrastrando el ratón por toda la pantalla. Desactivar
   `user-select` basta para que tampoco aparezca el cursor de texto sobre una etiqueta o un botón
   (el navegador solo lo muestra sobre contenido seleccionable), sin tocar `cursor` y sin arriesgar
   el cursor de mano de enlaces y botones. Se reactiva solo donde tiene sentido copiar: cifras
   (`.sdm-num`), contenido técnico o monoespaciado (`.font-mono`, `code`, `pre`), lo marcado
   explícitamente (`.sdm-selectable`) y los campos de formulario, que gestionan su propia
   selección. */
body {
  user-select: none;
  -webkit-user-select: none;
}

.sdm-num,
.font-mono,
code,
pre,
.sdm-selectable,
input,
textarea,
[contenteditable="true"] {
  user-select: text;
  -webkit-user-select: text;
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```


---

# 17. tokens.json — los mismos tokens, para herramientas

Fichero de origen: `src/design-system/tokens.json`

```json
{
  "$meta": {
    "name": "SmartDisk Monitor Design System",
    "version": "3.0.0",
    "direction": "v3 — «escena de datos» con paleta Ciruela (evolución de v2, material translúcido)",
    "themes": [
      "light",
      "dark",
      "system"
    ],
    "notes": [
      "Los colores viven por tema; en código se consumen SIEMPRE como var(--sdm-*).",
      "La fuente de verdad es src/design-system/tokens.css. Este JSON la espeja.",
      "El acento Ciruela es propio (ADR-034). Con el interruptor 'usar el acento de Windows' encendido, src/lib/design/accent.ts sobrescribe accent / accentHi / accentSoft en runtime (ADR-035).",
      "--sdm-on-accent NO es blanco en oscuro: es tinta (#20132a). Todo texto sobre el acento usa on-accent, nunca text-white.",
      "Todos los tokens de texto y de salud cumplen 4.5:1 sobre el material de su tema; no los aclares.",
      "Peso tipográfico máximo 600. La familia 'display' solo se usa con .sdm-display, a partir de 22px."
    ]
  },
  "typography": {
    "fontSans": "\"Instrument Sans\", system-ui, \"Segoe UI Variable\", sans-serif",
    "fontMono": "ui-monospace, \"Cascadia Mono\", Consolas, monospace",
    "fontDisplay": "var(--sdm-font-sans)",
    "scale": {
      "2xs": 11,
      "xs": 12,
      "sm": 12.5,
      "base": 13.5,
      "lg": 14.5,
      "xl": 20,
      "2xl": 21,
      "metric": 27,
      "display": 58,
      "hero": 76
    },
    "weights": {
      "regular": 400,
      "medium": 500,
      "semibold": 600
    },
    "tracking": {
      "tight": "-0.02em",
      "metric": "-0.03em",
      "display": "-0.045em"
    }
  },
  "space": {
    "1": 4,
    "2": 8,
    "3": 12,
    "4": 16,
    "5": 18,
    "6": 20,
    "8": 32
  },
  "layout": {
    "railWidth": 74,
    "heroHeight": 246
  },
  "radius": {
    "window": 18,
    "card": 18,
    "inner": 13,
    "nav": 9,
    "pill": 999,
    "rule": "interior = exterior − padding"
  },
  "control": {
    "sm": 30,
    "md": 30,
    "lg": 35,
    "tapMin": 30
  },
  "material": {
    "blurChrome": 28,
    "blurCard": 24,
    "blurOverlay": 44,
    "saturate": "180%"
  },
  "icon": {
    "stroke": 1.7,
    "size": 24
  },
  "motion": {
    "fast": "140ms",
    "base": "220ms",
    "overlay": "320ms",
    "ease": "cubic-bezier(0.32,0.72,0,1)"
  },
  "color": {
    "light": {
      "bg": "#efeaf1",
      "bg2": "#e6dfe9",
      "glass": "rgba(255,255,255,0.72)",
      "glass2": "rgba(255,255,255,0.5)",
      "glass3": "rgba(124,114,128,0.1)",
      "solid": "#fdfcfe",
      "hairline": "rgba(30,23,35,0.09)",
      "highlight": "rgba(255,255,255,0.9)",
      "scrim": "rgba(14,10,16,0.36)",
      "text": "#1e1723",
      "textDim": "#635a6b",
      "textFaint": "#6c6274",
      "onAccent": "#ffffff",
      "accent": "#7a3f9d",
      "accentHi": "#8b4bb0",
      "accentSoft": "rgba(122,63,157,0.12)",
      "accentFg": "#7a3f9d",
      "ok": "#2f7256",
      "warn": "#7d5619",
      "crit": "#b03434",
      "unknown": "#635c69"
    },
    "dark": {
      "bg": "#130f16",
      "bg2": "#19141d",
      "glass": "rgba(48,42,52,0.66)",
      "glass2": "rgba(64,56,68,0.42)",
      "glass3": "rgba(255,255,255,0.06)",
      "solid": "#1c1620",
      "hairline": "rgba(255,255,255,0.09)",
      "highlight": "rgba(255,255,255,0.13)",
      "scrim": "rgba(0,0,0,0.52)",
      "text": "#f4f0f6",
      "textDim": "#a89eb0",
      "textFaint": "#a29aa8",
      "onAccent": "#20132a",
      "accent": "#c79aec",
      "accentHi": "#d4aef5",
      "accentSoft": "rgba(199,154,236,0.18)",
      "accentFg": "#c79aec",
      "ok": "#6cc79c",
      "warn": "#e0b473",
      "crit": "#ef8080",
      "unknown": "#a8a0b0"
    }
  },
  "semantics": {
    "ok": "Correcto — dentro de umbrales.",
    "warn": "Advertencia — umbral cruzado o degradación no bloqueante.",
    "crit": "Crítico — atención inmediata. Bermellón, no el morado del acento.",
    "unknown": "Desconocido / no compatible / sin datos. NUNCA rojo.",
    "accent": "Acción primaria, selección y serie principal. No transmite salud."
  }
}
```


---

# 18. tailwind.config.cjs — mapeo de tokens

Fichero de origen: `tailwind.config.cjs`

```js
/** Tailwind mapeado 1:1 sobre design-system/tokens.css (v2 material translúcido).
 *  Si una utilidad no existe aquí, el valor no está en el sistema: no inventes clases arbitrarias.
 *  Las capas de material se aplican con las clases .sdm-material / -chrome / -overlay de tokens.css.
 *  @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,svelte,ts,js}"],
  theme: {
    extend: {
      colors: {
        bg: { DEFAULT: "var(--sdm-bg)", 2: "var(--sdm-bg-2)" },
        glass: { DEFAULT: "var(--sdm-glass)", 2: "var(--sdm-glass-2)", 3: "var(--sdm-glass-3)" },
        solid: "var(--sdm-solid)",
        hairline: "var(--sdm-hairline)",
        scrim: "var(--sdm-scrim)",
        fg: {
          DEFAULT: "var(--sdm-text)",
          dim: "var(--sdm-text-dim)",
          faint: "var(--sdm-text-faint)",
          onAccent: "var(--sdm-on-accent)"
        },
        accent: {
          DEFAULT: "var(--sdm-accent)",
          hi: "var(--sdm-accent-hi)",
          soft: "var(--sdm-accent-soft)",
          fg: "var(--sdm-accent-fg)"
        },
        ok: { DEFAULT: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)" },
        warn: { DEFAULT: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)" },
        crit: { DEFAULT: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)" },
        unknown: { DEFAULT: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)" }
      },
      fontFamily: {
        sans: "var(--sdm-font-sans)",
        mono: "var(--sdm-font-mono)",
        display: "var(--sdm-font-display)"
      },
      fontSize: {
        "2xs": ["var(--sdm-text-2xs)", { lineHeight: "1.35" }],
        xs: ["var(--sdm-text-xs)", { lineHeight: "1.45" }],
        sm: ["var(--sdm-text-sm)", { lineHeight: "1.5" }],
        base: ["var(--sdm-text-base)", { lineHeight: "1.4" }],
        lg: ["var(--sdm-text-lg)", { lineHeight: "1.3" }],
        xl: ["var(--sdm-text-xl)", { lineHeight: "1.2" }],
        "2xl": ["var(--sdm-text-2xl)", { lineHeight: "1.2" }],
        metric: ["var(--sdm-text-metric)", { lineHeight: "1.05" }],
        display: ["var(--sdm-text-display)", { lineHeight: "1" }],
        hero: ["var(--sdm-text-hero)", { lineHeight: "1" }]
      },
      fontWeight: { regular: "400", medium: "500", semibold: "600" },
      spacing: {
        1: "var(--sdm-space-1)",
        2: "var(--sdm-space-2)",
        3: "var(--sdm-space-3)",
        4: "var(--sdm-space-4)",
        5: "var(--sdm-space-5)",
        6: "var(--sdm-space-6)",
        8: "var(--sdm-space-8)"
      },
      borderRadius: {
        window: "var(--sdm-radius-window)",
        card: "var(--sdm-radius-card)",
        inner: "var(--sdm-radius-inner)",
        nav: "var(--sdm-radius-nav)",
        pill: "var(--sdm-radius-pill)"
      },
      height: {
        "control-sm": "var(--sdm-control-sm)",
        "control-md": "var(--sdm-control-md)",
        "control-lg": "var(--sdm-control-lg)",
        hero: "var(--sdm-hero-height)"
      },
      width: {
        rail: "var(--sdm-rail-width)"
      },
      boxShadow: {
        card: "var(--sdm-shadow)",
        lift: "var(--sdm-shadow-lift)",
        edge: "inset 0 1px 0 var(--sdm-highlight)",
        focus: "var(--sdm-focus-ring)"
      },
      backdropBlur: {
        chrome: "var(--sdm-blur-chrome)",
        card: "var(--sdm-blur-card)",
        overlay: "var(--sdm-blur-overlay)"
      },
      transitionTimingFunction: { sdm: "var(--sdm-ease)" },
      transitionDuration: {
        fast: "var(--sdm-duration-fast)",
        base: "var(--sdm-duration-base)",
        overlay: "var(--sdm-duration-overlay)"
      }
    }
  },
  plugins: []
};
```


---

# 19. components/index.ts — el catálogo cerrado

Fichero de origen: `src/lib/components/index.ts`

```ts
export { default as AppShell } from "./AppShell.svelte";
export { default as Sidebar } from "./Sidebar.svelte";
export { default as Toolbar } from "./Toolbar.svelte";

export { default as Button } from "./Button.svelte";
export { default as Card } from "./Card.svelte";
export { default as SegmentedControl } from "./SegmentedControl.svelte";
export { default as Switch } from "./Switch.svelte";
export { default as Select } from "./Select.svelte";
export { default as TextField } from "./TextField.svelte";
export { default as RadioGroup } from "./RadioGroup.svelte";
export { default as DateRangePicker } from "./DateRangePicker.svelte";
export { default as FilterBar } from "./FilterBar.svelte";
export { default as VirtualList } from "./VirtualList.svelte";

export { default as Icon } from "./Icon.svelte";
export { default as IconSprite } from "./IconSprite.svelte";
export { default as OnboardingArt } from "./OnboardingArt.svelte";
export { default as StatusPill } from "./StatusPill.svelte";
export { default as StatusDot } from "./StatusDot.svelte";
export { default as MetricCard } from "./MetricCard.svelte";
export { default as DataRow } from "./DataRow.svelte";
export { default as CapacityBar } from "./CapacityBar.svelte";
export { default as ProgressBar } from "./ProgressBar.svelte";
export { default as HealthDonut } from "./HealthDonut.svelte";
export { default as Sparkline } from "./Sparkline.svelte";
export { default as TimeSeriesChart } from "./TimeSeriesChart.svelte";

export { default as DiskCard } from "./DiskCard.svelte";
export { default as HeroPanel } from "./HeroPanel.svelte";
export { default as AlertCard } from "./AlertCard.svelte";
export { default as EventRow } from "./EventRow.svelte";

export { default as ConfirmDialog } from "./ConfirmDialog.svelte";
export { default as Toast } from "./Toast.svelte";
export { default as EmptyState } from "./EmptyState.svelte";
export { default as CodeOutput } from "./CodeOutput.svelte";
```


---

# 20. design/types.ts — vocabulario de la UI

Fichero de origen: `src/lib/design/types.ts`

```ts
/** Vocabulario compartido de la UI. Refleja el modelo de datos (docs/data-model.md). */

/** Estado de salud presentable. `unknown` cubre "no compatible" y "sin datos": nunca se pinta en rojo. */
export type HealthState = "ok" | "warn" | "crit" | "unknown";

/** Severidad de una alerta (alert_groups.severity). */
export type Severity = "info" | "warn" | "crit";

/** Ciclo de vida de un grupo de alertas (spec §5). El silencio **no** es un estado: es ortogonal
 *  y vive en `mutedUntil`. Una alerta puede estar activa y silenciada a la vez. */
export type AlertStatus = "active" | "acknowledged" | "resolved" | "archived";

/** Por qué un dato es desconocido. Distingue lo normal de lo averiado: un USB que no expone SMART
 *  es `unsupported` y no ensucia el estado global; un disco que debería responder y no responde es
 *  `unreadable` y sí cuenta como advertencia (véase `unknownContributesWarning`). */
export type UnknownReason = "unsupported" | "unreadable" | "collector-error" | "not-yet-sampled" | "paused";

/** Estado de una ejecución de prueba (test_runs.status). */
export type TestStatus =
  "pending" | "running" | "cancelling" | "completed" | "failed" | "cancelled" | "interrupted";

/** Procedencia y confianza de una métrica (metric_samples.source/quality). */
export type MetricSource = "smartctl" | "windows-storage" | "perf-counter" | "filesystem";
export type MetricQuality = "exact" | "inferred" | "vendor_specific" | "stale";

export type ThemePreference = "light" | "dark" | "system";

export interface Provenance {
  source: MetricSource;
  quality: MetricQuality;
  /** ISO UTC; la UI la presenta en hora local. */
  readAt?: string;
}

/** Error devuelto por cualquier comando Tauri. `AGENTS.md` §5 exige frase humana visible y detalle
 *  técnico conservado: `messageKey` es la clave i18n de la frase, `detail` el texto técnico crudo
 *  que se muestra dentro de un `<details>` y se copia al portapapeles. Nunca se enseña `detail` solo. */
export interface AppError {
  /** Identificador estable, apto para ramificar en la UI: "smartctl.timeout", "db.locked", … */
  code: string;
  /** Clave i18n de la explicación en lenguaje humano. */
  messageKey: string;
  /** Interpolaciones de `messageKey` (nombre de disco, ruta, número de intentos…). */
  messageVars?: Record<string, string | number>;
  /** Texto técnico literal: stderr, código de salida, mensaje de SQLite. Nunca traducido. */
  detail?: string | null;
  /** Qué fuente falló, cuando aplique: permite degradar una tarjeta y no la aplicación entera. */
  source?: MetricSource | null;
  /** true si repetir la misma acción tiene sentido (timeout, bloqueo temporal). */
  retryable: boolean;
}

export interface DiskSummary {
  id: string;
  alias?: string | null;
  model: string;
  deviceType: string;
  state: HealthState;
  /** null = no disponible. Nunca 0 inventado. */
  temperatureC: number | null;
  percentageUsed: number | null;
  activityPercent: number | null;
  powerOnHours: number | null;
  vendorTempLimitC?: number | null;
  /** Umbral crítico del fabricante, si lo declara; por debajo de él manda el configurado en ajustes. */
  vendorTempCriticalC?: number | null;
  /** Autoevaluación SMART global (`smart_status.passed`): `true` superada, `false` fallida, `null`
   *  sin dato o disco sin SMART. La consume el primer hecho del `HeroPanel` (ADR-041). */
  smartHealthPassed?: boolean | null;
  /** Presente solo cuando `state === "unknown"`: explica por qué y decide si cuenta como advertencia. */
  unknownReason?: UnknownReason | null;
  /** Última lectura válida de cualquier fuente. Alimenta la marca de dato obsoleto. */
  lastReadAt?: string | null;
  /** Fuente y calidad del bloque principal de métricas. */
  provenance?: Provenance | null;
  volumes: VolumeSummary[];
}

export interface VolumeSummary {
  id: string;
  label: string;
  driveLetters: string[];
  capacityBytes: number | null;
  freeBytes: number | null;
  mappingConfidence: "exact" | "inferred" | "unknown";
  /** `chkdsk /scan` solo existe en NTFS: lo decide el backend, no se repite el criterio aquí. */
  chkdskAvailable: boolean;
  /** `true` para el volumen donde vive Windows (v3, ADR-036). Lo calcula el backend; la interfaz no
   *  lo infiere. Lo consume `selectHeroDisk()`. */
  isSystemVolume: boolean;
}

export interface AlertGroup {
  id: string;
  /** El backend no manda texto de interfaz (ADR-030): el título y el resumen se resuelven en el
   *  componente con `t(\`alert.rule.${ruleKey}.title\`)` / `.summary`, una clave por regla. */
  ruleKey: string;
  deduplicationKey: string;
  severity: Severity;
  status: AlertStatus;
  count: number;
  firstOccurredAt: string;
  lastOccurredAt: string;
  target: string;
  /** Silencio de la notificación. `null` = no silenciada; una fecha ISO UTC = silenciada hasta
   *  ese momento; `"infinite"` = hasta reactivación manual. **Nunca afecta al color** (ADR-016).
   *
   *  El tipo es `string | null` y no `string | "infinite" | null` porque el literal quedaría
   *  absorbido por `string` sin aportar nada; el valor especial se documenta aquí y lo valida el
   *  esquema Zod, que sí puede distinguirlos. */
  mutedUntil?: string | null;
  /** Ciclo de recaída: se incrementa cada vez que el grupo se resuelve y vuelve a activarse,
   *  para que la cronología distinga episodios (US-030). */
  cycle?: number;
}
```


---

# 21. design/health.ts — estado → color, umbrales

Fichero de origen: `src/lib/design/health.ts`

```ts
import type { HealthState, Severity, AlertStatus, UnknownReason } from "./types";

/** Único mapa autorizado de estado → token de color. Ningún componente decide colores por su cuenta. */
export const healthToken: Record<HealthState, { fg: string; soft: string; labelKey: string }> = {
  ok: { fg: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)", labelKey: "health.ok" },
  warn: { fg: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)", labelKey: "health.warn" },
  crit: { fg: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)", labelKey: "health.crit" },
  unknown: { fg: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)", labelKey: "health.unknown" }
};

export const severityToHealth: Record<Severity, HealthState> = {
  info: "unknown",
  warn: "warn",
  crit: "crit"
};

/** Estados de alerta que siguen pesando sobre el color de salud.
 *  Decisión de producto: **reconocer no cambia el color**. Reconocer saca la alerta de la lista de
 *  pendientes y le pone un distintivo, pero la condición sigue siendo real y el color no debe mentir
 *  sobre el estado del hardware. Solo `resolved` y `archived` dejan de contar.
 *  El silencio es ortogonal al estado: silencia la notificación, nunca el color. */
const COUNTS_TOWARD_HEALTH: readonly AlertStatus[] = ["active", "acknowledged"];

export function alertCountsTowardHealth(status: AlertStatus): boolean {
  return COUNTS_TOWARD_HEALTH.includes(status);
}

/** Salud presentable de un disco: la peor severidad de sus alertas no resueltas.
 *  Un disco sin alertas y sin datos SMART legibles es `unknown`, nunca `ok`: no sabemos que esté bien. */
export function deviceState(
  alerts: readonly { severity: Severity; status: AlertStatus }[],
  hasFreshData: boolean
): HealthState {
  const live = alerts.filter((a) => alertCountsTowardHealth(a.status));
  if (live.some((a) => a.severity === "crit")) return "crit";
  if (live.some((a) => a.severity === "warn")) return "warn";
  return hasFreshData ? "ok" : "unknown";
}

/** Estado **presentable** de un disco en las tarjetas, el Hero y el reparto: su `state` de SMART
 *  (frescura), elevado a la peor severidad de sus alertas `active`/`acknowledged`.
 *
 *  Existe porque `B.1` (`docs/open-questions.md`) está a medio conectar: el backend nunca funde las
 *  alertas en `DiskSummary.state` (`enrich_with_smart_data` siempre pasa `None` a `device_state`),
 *  así que se hace aquí —que es donde `B.1` dijo que vivía—, una sola vez, y todo lo que lee
 *  `disk.state` en el panel pasa antes por esta función.
 *
 *  Cuentan tanto las alertas dirigidas al **dispositivo** (`…|device:<id>`) como a un **volumen
 *  suyo** (`…|volume:<id>`): un volumen lleno es un problema del disco que lo contiene, no una
 *  categoría aparte.
 *
 *  **Un `unknown` se queda `unknown`**, sea cual sea el motivo: un disco sin datos SMART se
 *  presenta como «sin datos SMART» (gris), no como advertencia, aunque haya dejado de responder.
 *  Que eso cuente para «N necesitan atención» y el color de la bandeja lo decide aparte
 *  `estadoParaRecuento`; el color de la tarjeta no. */
export function estadoConAlertas(
  disk: {
    id: string;
    state: HealthState;
    volumes?: readonly { id: string }[];
  },
  alerts: readonly { severity: Severity; status: AlertStatus; deduplicationKey: string }[]
): HealthState {
  const suyas = alerts.filter(
    (a) =>
      alertCountsTowardHealth(a.status) &&
      (a.deduplicationKey.includes(`device:${disk.id}`) ||
        (disk.volumes ?? []).some((v) => a.deduplicationKey.includes(`volume:${v.id}`)))
  );
  if (suyas.some((a) => a.severity === "crit")) return "crit";
  if (suyas.some((a) => a.severity === "warn")) return "warn";
  return disk.state;
}

/** Estado de un disco **solo para el recuento global** (píldora de la `Toolbar`, pie del riel,
 *  icono de la bandeja): como `estadoConAlertas`, pero además un `unknown` por `unreadable` o
 *  `collector-error` cuenta como advertencia (`unknownContributesWarning`, §B.5) —una fuente que
 *  debería funcionar y no funciona es una degradación real—, salvo con la monitorización en pausa,
 *  donde el estado de pausa manda. **No** se usa para pintar tarjetas: ahí un `unknown` es gris. */
export function estadoParaRecuento(
  disk: {
    id: string;
    state: HealthState;
    unknownReason?: UnknownReason | null;
    volumes?: readonly { id: string }[];
  },
  alerts: readonly { severity: Severity; status: AlertStatus; deduplicationKey: string }[],
  opts: { paused?: boolean } = {}
): HealthState {
  const s = estadoConAlertas(disk, alerts);
  if (
    s === "unknown" &&
    !opts.paused &&
    disk.unknownReason != null &&
    unknownContributesWarning(disk.unknownReason)
  ) {
    return "warn";
  }
  return s;
}

/** Severidad máxima de una lista. `unknown` no gana nunca a un estado conocido:
 *  se usa solo cuando no hay ningún estado conocido que mostrar. */
export function worstState(states: readonly HealthState[]): HealthState {
  if (states.includes("crit")) return "crit";
  if (states.includes("warn")) return "warn";
  if (states.includes("ok")) return "ok";
  return "unknown";
}

/** Estado global de la aplicación, calculado **una sola vez** y presentado en dos sitios: la píldora
 *  de la `Toolbar` (con texto) y el pie del riel de la `Sidebar` (solo icono). Al ser la misma
 *  función, no pueden contradecirse (`docs/ui-design.md` §7, `09-chrome-y-estados.md`).
 *
 *  `kind` distingue los casos que la interfaz rotula distinto; `state` es el token de color; `count`
 *  es cuántos discos monitorizados necesitan atención. La `Sidebar`/`Toolbar` traducen `kind` a texto
 *  con `t()` — aquí no hay literales de interfaz. */
export type GlobalStatusKind = "loading" | "paused" | "noDevices" | "ok" | "attention";

export function globalStatus(input: {
  /** false mientras el inventario no ha llegado: NO es lo mismo que "no hay discos". */
  loaded: boolean;
  paused: boolean;
  /** Estados de los discos **monitorizados** (los excluidos no cuentan). */
  monitoredStates: readonly HealthState[];
}): { kind: GlobalStatusKind; state: HealthState; count: number } {
  if (!input.loaded) return { kind: "loading", state: "unknown", count: 0 };
  if (input.paused) return { kind: "paused", state: "unknown", count: 0 };
  if (input.monitoredStates.length === 0) return { kind: "noDevices", state: "unknown", count: 0 };
  const count = input.monitoredStates.filter((s) => s === "warn" || s === "crit").length;
  if (count === 0) return { kind: "ok", state: "ok", count: 0 };
  return { kind: "attention", state: worstState(input.monitoredStates), count };
}

/** El disco que protagoniza el `HeroPanel` del panel general (v3, `HeroPanel.md`). **La pantalla
 *  elige, no el componente**, y este es el criterio:
 *
 *   1. el disco con la alerta que cuenta para la salud (`active`/`acknowledged`) de mayor severidad;
 *      empate → la de ocurrencia más reciente;
 *   2. si no hay ninguna, el disco que no está sano —crit, luego warn, luego un `unknown` que
 *      cuenta como degradación (`unreadable`/`collector-error`)—, para que el protagonista nunca
 *      sea un disco sano habiendo uno con problema, aunque venga de un volumen lleno o de un SMART
 *      que dejó de responder y no de una alerta de dispositivo; empate → orden de inventario;
 *   3. si todos van bien, el disco cuyo volumen sea el de sistema (`isSystemVolume`);
 *   4. si no se sabe, el primero del inventario;
 *   5. **un disco sin SMART (`unknown` por `unsupported`) nunca protagoniza**, salvo que sea el único.
 *
 *  `state` debe venir ya con las alertas fundidas (`estadoConAlertas`). Devuelve `null` solo si no
 *  hay ningún disco. */
export function selectHeroDisk<
  D extends {
    id: string;
    state: HealthState;
    unknownReason?: UnknownReason | null;
    volumes?: readonly { isSystemVolume?: boolean }[];
  }
>(
  disks: readonly D[],
  alerts: readonly { severity: Severity; status: AlertStatus; deduplicationKey: string; lastOccurredAt: string }[]
): D | null {
  if (disks.length === 0) return null;

  const sinSmart = (d: D) => d.state === "unknown" && (d.unknownReason ?? "unsupported") === "unsupported";
  const elegibles = disks.some((d) => !sinSmart(d)) ? disks.filter((d) => !sinSmart(d)) : disks;

  const sev: Record<Severity, number> = { crit: 3, warn: 2, info: 1 };
  const puntuados: { disk: D; sev: number; when: string }[] = [];
  for (const d of elegibles) {
    const suyas = alerts.filter(
      (a) => alertCountsTowardHealth(a.status) && a.deduplicationKey.includes(`device:${d.id}`)
    );
    if (!suyas.length) continue;
    let mejorSev = 0;
    let mejorWhen = "";
    for (const a of suyas) {
      if (sev[a.severity] > mejorSev || (sev[a.severity] === mejorSev && a.lastOccurredAt > mejorWhen)) {
        mejorSev = sev[a.severity];
        mejorWhen = a.lastOccurredAt;
      }
    }
    puntuados.push({ disk: d, sev: mejorSev, when: mejorWhen });
  }

  if (puntuados.length) {
    puntuados.sort((a, b) => b.sev - a.sev || b.when.localeCompare(a.when));
    return puntuados[0].disk;
  }

  // Sin alerta de dispositivo, pero el `state` ya trae fundidas las alertas de volumen: si algún
  // disco no está sano —o dejó de responder a SMART—, protagoniza él, no el de sistema.
  const conProblema =
    elegibles.find((d) => d.state === "crit") ??
    elegibles.find((d) => d.state === "warn") ??
    elegibles.find(
      (d) =>
        d.state === "unknown" &&
        d.unknownReason != null &&
        unknownContributesWarning(d.unknownReason)
    );
  if (conProblema) return conProblema;

  return elegibles.find((d) => d.volumes?.some((v) => v.isSystemVolume)) ?? elegibles[0];
}

/** Un `unknown` que se debe a una fuente que **debería** funcionar es una degradación real y se
 *  presenta como advertencia; un `unknown` declarado por el propio dispositivo (un USB que no expone
 *  SMART) es normalidad y no ensucia el estado global. Regla derivada de spec §5
 *  ("SMART ilegible persistentemente: advertencia; 'no compatible' no genera alerta"). */
export function unknownContributesWarning(reason: UnknownReason): boolean {
  return reason === "unreadable" || reason === "collector-error";
}

/** Color del icono de la bandeja del sistema (spec §3).
 *  Prioridad: un crítico vigente manda sobre la pausa — la condición sigue siendo cierta aunque
 *  hayamos dejado de mirar; la pausa se comunica con el texto del menú, no apagando la señal. */
export function trayState(input: {
  paused: boolean;
  collectorFailure: boolean;
  monitoredStates: readonly HealthState[];
}): HealthState {
  if (input.monitoredStates.includes("crit")) return "crit";
  if (input.paused || input.collectorFailure || input.monitoredStates.length === 0) return "unknown";
  if (input.monitoredStates.includes("warn")) return "warn";
  return worstState(input.monitoredStates);
}

/** Umbrales efectivos de temperatura para un disco (`docs/alert-rules.md`,
 *  `temp.above_vendor_limit`/`_critical` y `temp.above_configured_warn`/`_crit`): el límite del
 *  fabricante manda si existe; a falta de él, el configurado en `settings.alerts` es el respaldo. */
export function temperatureThresholds(
  vendorLimitC: number | null | undefined,
  vendorCriticalC: number | null | undefined,
  configuredWarnC: number,
  configuredCritC: number
): { warn: number; crit: number } {
  return { warn: vendorLimitC ?? configuredWarnC, crit: vendorCriticalC ?? configuredCritC };
}

/** Estado de salud de una lectura frente a un par de umbrales aviso/crítico. Mismo criterio de
 *  operadores que `alert-rules.md`: el aviso es estrictamente por encima, el crítico llega igual. */
export function classifyAgainstThresholds(
  value: number | null,
  warn: number,
  crit: number
): HealthState {
  if (value === null) return "unknown";
  if (value >= crit) return "crit";
  if (value > warn) return "warn";
  return "ok";
}

/** Por debajo de esta capacidad, el suelo absoluto de espacio libre no se aplica: en un volumen
 *  pequeño, 20 GB libres pueden ser un tercio del disco y marcarlo en rojo sería ruido puro.
 *  Configurable en `settings` (`alerts.capacity.absoluteFloorMinCapacityBytes`). */
export const CAPACITY_ABSOLUTE_FLOOR_MIN_BYTES = 256 * 1024 ** 3;

/** Nivel de capacidad (spec §5). Reglas:
 *  - siempre por porcentaje: <10 % advertencia, <5 % crítico;
 *  - además, en volúmenes de ≥256 GB, por valor absoluto: <20 GB advertencia, <10 GB crítico.
 *  Gana el más severo de los dos criterios. */
export function capacityState(
  freeBytes: number | null,
  capacityBytes: number | null,
  absoluteFloorMinBytes = CAPACITY_ABSOLUTE_FLOOR_MIN_BYTES
): HealthState {
  if (freeBytes === null || capacityBytes === null || capacityBytes <= 0) return "unknown";
  const pct = (freeBytes / capacityBytes) * 100;
  const GB = 1024 ** 3;
  const applyAbsolute = capacityBytes >= absoluteFloorMinBytes;

  if (pct < 5 || (applyAbsolute && freeBytes < 10 * GB)) return "crit";
  if (pct < 10 || (applyAbsolute && freeBytes < 20 * GB)) return "warn";
  return "ok";
}

/** Color de la **barra** de ocupación de un volumen (no de la alerta). Imita al Explorador de
 *  Windows: rojo cuando queda poco espacio. Umbrales del boceto (`smartdisk-v3.html`,
 *  `bar = u => u >= 91 ? crit : u >= 85 ? warn : ok`): ≥ 91 % ocupado → rojo, ≥ 85 % → ámbar, por
 *  debajo → verde. La regla de alerta `capacity.*` es otra cosa y la decide `capacityState()`. */
export function capacityBarTone(usedPercent: number | null): HealthState {
  if (usedPercent === null) return "unknown";
  if (usedPercent >= 91) return "crit";
  if (usedPercent >= 85) return "warn";
  return "ok";
}
```


---

# 22. design/format.ts — formato de presentación

Fichero de origen: `src/lib/design/format.ts`

```ts
/** Formateo de presentación. Regla de oro: un valor ausente se muestra como "No disponible",
 *  nunca como 0, "-" ni cadena vacía (spec §12 y modelo de datos §1).
 *
 *  Locale: **todas** las funciones formatean con `i18n.formatLocale`, que sigue al idioma elegido
 *  en la aplicación, no al de Windows. Pasar un locale explícito es una excepción reservada a las
 *  exportaciones (un informe puede pedirse en un idioma distinto al de la interfaz).
 *
 *  Unidades: se usa base 1024 con las etiquetas KB/MB/GB, que es la convención del Explorador de
 *  Windows y por tanto la que el usuario podrá contrastar. Es deliberado: no "corregir" a KiB/MiB
 *  ni a base 1000. El dato persistido son siempre bytes (modelo de datos §5).
 */

import { i18n, t } from "$lib/i18n";

export const NOT_AVAILABLE = () => t("common.notAvailable"); // es: "No disponible" / en: "Not available"

const isMissing = (v: unknown): v is null | undefined => v === null || v === undefined || Number.isNaN(v);

/** Base binaria con etiquetas decimales, igual que el Explorador de Windows. */
const KIB = 1024;

/** Bytes → unidad legible, base 1024, locale de la aplicación. El dato original nunca se altera. */
export function formatBytes(bytes: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(bytes)) return NOT_AVAILABLE();
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let value = bytes;
  let i = 0;
  while (value >= KIB && i < units.length - 1) {
    value /= KIB;
    i++;
  }
  const decimals = value < 10 && i > 2 ? 2 : value < 100 && i > 1 ? 1 : 0;
  return `${value.toLocaleString(locale, { minimumFractionDigits: decimals, maximumFractionDigits: decimals })} ${units[i]}`;
}

export function formatTemperature(celsius: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(celsius)) return NOT_AVAILABLE();
  return `${celsius.toLocaleString(locale, { maximumFractionDigits: 0 })} °C`;
}

export function formatPercent(value: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(value)) return NOT_AVAILABLE();
  return `${value.toLocaleString(locale, { maximumFractionDigits: 0 })} %`;
}

export function formatHours(hours: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(hours)) return NOT_AVAILABLE();
  return `${hours.toLocaleString(locale)} h`;
}

/** Duración compacta y localizada ("45 s", "6 min", "3 h"). La usa el pie del gráfico del panel
 *  general para decir cuánto abarca la ventana visible, que en v3 se adapta a los datos que hay
 *  (`ultimoTramoVisible`). Como el panel nunca pide más de 24 h, no hay tramo de días. Unidad y
 *  plural los resuelve `Intl`, no un diccionario. */
export function formatSpanShort(
  milliseconds: number | null | undefined,
  locale = i18n.formatLocale
): string {
  if (isMissing(milliseconds) || milliseconds < 0) return NOT_AVAILABLE();
  const s = milliseconds / 1000;
  const [value, unit] =
    s < 90
      ? [Math.round(s), "second" as const]
      : s < 90 * 60
        ? [Math.round(s / 60), "minute" as const]
        : [Math.round(s / 3600), "hour" as const];
  return new Intl.NumberFormat(locale, { style: "unit", unit, unitDisplay: "short" }).format(value);
}

/** Caudal en **bytes por segundo**, que es la unidad que persiste el backend
 *  (`read_bytes_per_second` / `write_bytes_per_second`). La escala es la misma base 1024 que
 *  `formatBytes`, de modo que "180 MB/s" son 180 × 1024² B/s.
 *  Nunca pases MB/s ya convertidos: la conversión vive aquí y en un solo sitio. */
export function formatThroughput(
  bytesPerSecond: number | null | undefined,
  locale = i18n.formatLocale
): string {
  if (isMissing(bytesPerSecond)) return NOT_AVAILABLE();
  return `${formatBytes(bytesPerSecond, locale)}/s`;
}

/** Latencia en milisegundos. Por debajo de 10 ms se muestra un decimal: la diferencia entre
 *  0,2 ms (NVMe) y 4 ms (HDD) es justo la que interesa leer. */
export function formatLatency(milliseconds: number | null | undefined, locale = i18n.formatLocale): string {
  if (isMissing(milliseconds)) return NOT_AVAILABLE();
  const decimals = milliseconds < 10 ? 1 : 0;
  return `${milliseconds.toLocaleString(locale, { minimumFractionDigits: decimals, maximumFractionDigits: decimals })} ms`;
}

/** UTC persistido → hora local del sistema. */
export function formatDateTime(isoUtc: string | null | undefined, locale = i18n.formatLocale): string {
  if (!isoUtc) return NOT_AVAILABLE();
  return new Date(isoUtc).toLocaleString(locale, { dateStyle: "short", timeStyle: "medium" });
}

export function formatTime(isoUtc: string | null | undefined, locale = i18n.formatLocale): string {
  if (!isoUtc) return NOT_AVAILABLE();
  return new Date(isoUtc).toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
}

/** Antigüedad de una lectura ("hace 2 min"). Alimenta la marca de dato obsoleto exigida por
 *  `AGENTS.md` §5. Devuelve null si no hay fecha: el llamante decide si omitir la marca. */
export function formatAge(
  isoUtc: string | null | undefined,
  now = Date.now(),
  locale = i18n.formatLocale
): string | null {
  if (!isoUtc) return null;
  const seconds = Math.round((now - new Date(isoUtc).getTime()) / 1000);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto", style: "narrow" });
  if (seconds < 60) return rtf.format(-seconds, "second");
  if (seconds < 3600) return rtf.format(-Math.round(seconds / 60), "minute");
  if (seconds < 86400) return rtf.format(-Math.round(seconds / 3600), "hour");
  return rtf.format(-Math.round(seconds / 86400), "day");
}

/** Número de serie enmascarado para capturas y exportaciones anonimizadas. */
export function maskSerial(serial: string | null | undefined): string {
  if (!serial) return NOT_AVAILABLE();
  if (serial.length <= 6) return "••••";
  return `${serial.slice(0, 4)}••••${serial.slice(-2)}`;
}

/** Porcentaje ocupado de un volumen; null si falta cualquiera de los dos datos. */
export function usedPercent(capacityBytes: number | null, freeBytes: number | null): number | null {
  if (isMissing(capacityBytes) || isMissing(freeBytes) || capacityBytes === 0) return null;
  return Math.min(100, Math.max(0, ((capacityBytes - freeBytes) / capacityBytes) * 100));
}
```


---

# 23. design/theme.svelte.ts — tema

Fichero de origen: `src/lib/design/theme.svelte.ts`

```ts
/** Preferencia de tema: claro / oscuro / sistema (spec §8, US-003).
 *  El valor efectivo se escribe en <html data-theme="light|dark">; los tokens hacen el resto.
 *  Persistir la preferencia en `settings` vía comando Tauri, no en localStorage. */

import type { ThemePreference } from "./types";
import { refreshAccentForTheme } from "./accent";

const STORAGE_HINT = "settings.appearance.theme"; // clave tipada en la tabla settings

let preference = $state<ThemePreference>("system");
let systemDark = $state(false);

export const theme = {
  get preference() {
    return preference;
  },
  get resolved(): "light" | "dark" {
    return preference === "system" ? (systemDark ? "dark" : "light") : preference;
  },
  /** Llamar una vez al arrancar la app, con el valor leído de settings. */
  init(initial: ThemePreference) {
    preference = initial;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    systemDark = mq.matches;
    mq.addEventListener("change", (e) => (systemDark = e.matches));
    apply();
  },
  set(next: ThemePreference) {
    preference = next;
    apply();
    return { key: STORAGE_HINT, value: next }; // el llamante lo persiste con invoke("set_setting", …)
  }
};

function apply() {
  document.documentElement.dataset.theme = theme.resolved;
  // `--sdm-accent-fg` depende de la superficie, y la superficie cambia con el tema: si no se
  // recalcula aquí, un acento heredado que era legible en claro puede quedar por debajo de AA al
  // pasar a oscuro (open-questions.md §O).
  refreshAccentForTheme();
}

$effect.root(() => {
  $effect(() => {
    if (typeof document !== "undefined") apply();
  });
});
```


---

# 24. design/accent.ts — acento de Windows

Fichero de origen: `src/lib/design/accent.ts`

```ts
/** El acento de la app hereda el color de acento de Windows (decisión de diseño v2).
 *  El backend Rust lo lee del registro y lo expone como comando Tauri; aquí solo se traduce a
 *  tokens. Si el usuario lo tiene desactivado o falla la lectura, se conservan los respaldos de
 *  `tokens.css`, que ya están verificados contra AA en cada tema.
 *
 *  **El acento tiene dos usos con requisitos opuestos, y por eso hay dos tokens.**
 *
 *    `--sdm-accent`     va de FONDO (botón primario). Se mide contra `--sdm-on-accent`.
 *    `--sdm-accent-fg`  va de TEXTO sobre el material (enlaces, selección, serie de la gráfica).
 *                       Se mide contra la superficie, casi blanca en claro y casi negra en oscuro.
 *
 *  Un solo color no puede servir para las dos cosas. Barriendo el espacio sRGB completo
 *  (262.144 colores, `tools/accent-check.py`): el 65 % de los acentos posibles son ilegibles como
 *  texto sobre el material claro y el 50 % sobre el oscuro — **incluido el azul #0078d4 que Windows
 *  trae de fábrica**, que da 4,31:1 en tema claro, por debajo de AA. Aplicar el acento del sistema
 *  a los dos tokens a la vez, como se hacía antes, rompía el contraste en la configuración más
 *  común que existe.
 *
 *  Por eso `--sdm-accent-fg` se deriva por tema y se recalcula cuando el tema cambia.
 */

import { getSystemAccentColor } from "$lib/api";
import type { WindowsAccentShape as WindowsAccent } from "$lib/api/schemas";

/* El acento llega validado por su esquema Zod (constitución §XI): el `hex` es siempre `#RRGGBB`
   —el backend hace la conversión desde el ABGR del registro— y la paleta, si viene, son los tonos
   que Windows ya usa en su propia interfaz. Aquí solo se traduce a tokens. */

/** Contraste mínimo para texto normal (WCAG AA). */
const AA = 4.5;

type RGB = [number, number, number];

function toRgb(hex: string): RGB {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function toHex([r, g, b]: RGB): string {
  return (
    "#" +
    [r, g, b]
      .map((c) =>
        Math.round(Math.min(255, Math.max(0, c)))
          .toString(16)
          .padStart(2, "0")
      )
      .join("")
  );
}

/** Luminancia relativa WCAG 2.x. */
function luminance([r, g, b]: RGB): number {
  const lin = [r, g, b].map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2];
}

function contrast(a: RGB, b: RGB): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const WHITE: RGB = [255, 255, 255];
const BLACK: RGB = [17, 17, 20];

/** Mezcla lineal hacia blanco (`amount` > 0) o hacia negro (`amount` < 0).
 *
 *  **Redondea a enteros**, y no es un detalle: el color que se devuelve al navegador es de 8 bits
 *  por canal, así que medir el contraste sobre los valores en coma flotante y emitir después los
 *  redondeados puede entregar un color por debajo de AA. Ocurría con `#00cc6a`, que salía a
 *  4,497:1 tras redondear. Se redondea aquí para que lo medido y lo emitido sean el mismo color. */
function shift(rgb: RGB, amount: number): RGB {
  const target = amount > 0 ? 255 : 0;
  const k = Math.abs(amount);
  return rgb.map((c) => Math.round(c + (target - c) * k)) as RGB;
}

function lighten(hex: string, amount = 0.14): string {
  return toHex(shift(toRgb(hex), amount));
}

function rgba(hex: string, alpha: number): string {
  const [r, g, b] = toRgb(hex);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

const isHex = (v: unknown): v is string => typeof v === "string" && /^#[0-9a-fA-F]{6}$/.test(v);

/**
 * Acento utilizable **como fondo**, con el color de texto que le corresponde.
 * Busca el ajuste más pequeño que alcance AA, para no desvirtuar el color elegido por el usuario.
 * Verificado sobre el espacio sRGB completo: ningún color queda por debajo de AA, y el retoque
 * máximo es de 26/255 en un canal, imperceptible.
 */
export function accessibleAccent(hex: string): { accent: string; onAccent: string; adjusted: boolean } {
  const original = toRgb(hex);
  const prefersBlackText = contrast(original, BLACK) > contrast(original, WHITE);
  const text = prefersBlackText ? BLACK : WHITE;

  if (contrast(original, text) >= AA) {
    return { accent: hex, onAccent: toHex(text), adjusted: false };
  }

  const direction = prefersBlackText ? 1 : -1;
  for (let k = 0.05; k <= 0.9; k += 0.05) {
    const candidate = shift(original, direction * k);
    if (contrast(candidate, text) >= AA) {
      return { accent: toHex(candidate), onAccent: toHex(text), adjusted: true };
    }
  }
  return { accent: toHex(shift(original, direction * 0.9)), onAccent: toHex(text), adjusted: true };
}

/**
 * Acento utilizable **como texto** sobre una superficie dada.
 * Si Windows ofrece su paleta de tonos, se busca ahí primero: son los tonos que el usuario ya ve en
 * el resto del sistema, así que la aplicación se integra en lugar de inventarse un color propio.
 * Si ninguno llega a AA, se deriva oscureciendo o aclarando el acento base.
 */
export function accentOnSurface(hex: string, surface: RGB, palette?: string[]): string {
  const base = toRgb(hex);
  if (contrast(base, surface) >= AA) return hex;

  const surfaceIsLight = luminance(surface) > 0.5;

  const candidates = (palette ?? []).filter(isHex);
  // En tema claro interesan los tonos oscuros de la paleta, y al revés. Se ordenan por cercanía
  // al acento base para elegir el mínimo cambio que cumpla.
  const ordered = candidates
    .map((h) => ({ hex: h, rgb: toRgb(h) }))
    .filter((c) => (surfaceIsLight ? luminance(c.rgb) < luminance(base) : luminance(c.rgb) > luminance(base)))
    .sort(
      (a, b) => Math.abs(luminance(a.rgb) - luminance(base)) - Math.abs(luminance(b.rgb) - luminance(base))
    );

  for (const c of ordered) {
    if (contrast(c.rgb, surface) >= AA) return c.hex;
  }

  const direction = surfaceIsLight ? -1 : 1;
  for (let k = 0.05; k <= 0.95; k += 0.05) {
    const candidate = shift(base, direction * k);
    if (contrast(candidate, surface) >= AA) return toHex(candidate);
  }
  return toHex(shift(base, direction * 0.95));
}

/**
 * Color efectivo de la superficie sobre la que se lee el texto: el material translúcido compuesto
 * sobre el lienzo. Se calcula desde los tokens vivos del tema activo, no desde constantes, para que
 * un cambio en `tokens.css` no deje esta comprobación mintiendo en silencio.
 */
function effectiveSurface(): RGB {
  const cs = getComputedStyle(document.documentElement);
  const bg = parseColor(cs.getPropertyValue("--sdm-bg").trim()) ?? [255, 255, 255];
  const glass = parseColor(cs.getPropertyValue("--sdm-glass").trim());
  if (!glass) return bg;
  const alpha = parseAlpha(cs.getPropertyValue("--sdm-glass").trim());
  return glass.map((c, i) => Math.round(c * alpha + bg[i] * (1 - alpha))) as RGB;
}

function parseColor(value: string): RGB | null {
  // `isHex` es una guarda de tipo: sin el `else`, TypeScript estrecha `value` a `never` después.
  if (/^#[0-9a-fA-F]{6}$/.test(value)) return toRgb(value);
  const m = value.match(/rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)/i);
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

function parseAlpha(value: string): number {
  const m = value.match(/rgba\(\s*[\d.]+[\s,]+[\d.]+[\s,]+[\d.]+[\s,/]+([\d.]+)\s*\)/i);
  return m ? Number(m[1]) : 1;
}

/** Último acento aplicado, para poder recalcular `--sdm-accent-fg` al cambiar de tema. */
let current: WindowsAccent | null = null;

/** Llamar al arrancar y cuando el backend emita `system:accent-changed`. */
export async function applySystemAccent(): Promise<void> {
  let accent: WindowsAccent;
  try {
    accent = await getSystemAccentColor();
  } catch {
    return; // sin acento del sistema: se mantienen los respaldos de tokens.css
  }
  if (!accent || !isHex(accent.hex)) return;

  current = accent;
  paint();
}

/** Reaplica el acento al tema actual. Debe llamarse tras cada cambio de tema: `--sdm-accent-fg`
 *  depende de la superficie, y la superficie cambia con el tema. */
export function refreshAccentForTheme(): void {
  if (current) paint();
}

function paint(): void {
  if (!current) return;
  const { accent: safe, onAccent } = accessibleAccent(current.hex);
  const fg = accentOnSurface(current.hex, effectiveSurface(), current.palette);

  const root = document.documentElement.style;
  root.setProperty("--sdm-accent", safe);
  root.setProperty("--sdm-accent-hi", lighten(safe));
  root.setProperty("--sdm-accent-soft", rgba(safe, 0.14));
  root.setProperty("--sdm-on-accent", onAccent);
  root.setProperty("--sdm-accent-fg", fg);
}

/** Quita la sobreescritura y vuelve a los respaldos del sistema de diseño. */
export function clearSystemAccent(): void {
  current = null;
  for (const p of [
    "--sdm-accent",
    "--sdm-accent-hi",
    "--sdm-accent-soft",
    "--sdm-on-accent",
    "--sdm-accent-fg"
  ]) {
    document.documentElement.style.removeProperty(p);
  }
}
```


---

# 25. i18n/index.ts — idioma, formato y plurales

Fichero de origen: `src/lib/i18n/index.ts`

```ts
/** Barrel del módulo de i18n.
 *
 *  La implementación vive en `i18n.svelte.ts` porque usa runes (`$state`), y Svelte 5 solo las
 *  compila en ficheros `.svelte.ts`. Este barrel existe para que el resto del código pueda seguir
 *  importando de `$lib/i18n` sin conocer ese detalle.
 */
export { i18n, t, tp, type Locale } from "./i18n.svelte";
```


---

# 26. i18n/es.json

Fichero de origen: `src/lib/i18n/es.json`

```json
{
  "app.name": "SmartDisk Monitor",
  "common.notAvailable": "No disponible",
  "common.unsupported": "No compatible",
  "common.noData": "Sin datos",
  "common.sourceError": "Fuente con error",
  "common.refresh": "Actualizar",
  "common.cancel": "Cancelar",
  "common.close": "Cerrar",
  "common.continue": "Continuar",
  "common.technicalDetail": "Detalle técnico",
  "common.viewAll": "Ver todos",
  "common.updatedAgo": "hace {value}",
  "common.moment": "un momento",
  "health.ok": "Correcto",
  "health.warn": "Advertencia",
  "health.crit": "Crítico",
  "health.unknown": "Sin datos SMART",
  "global.allGood": "Todo en orden",
  "global.paused": "En pausa",
  "global.loading": "Comprobando…",
  "nav.alertsUnread": "Alertas, {count} sin revisar",
  "nav.dashboard": "Panel general",
  "nav.alerts": "Alertas",
  "nav.tests": "Pruebas y diagnóstico",
  "nav.reports": "Informes",
  "nav.settings": "Ajustes",
  "nav.monitoredDisks": "Discos monitorizados",
  "nav.pause": "Pausar recopilación",
  "nav.resume": "Reanudar recopilación",
  "disk.temperature": "Temperatura",
  "disk.temperatureShort": "Temp.",
  "disk.wear": "Desgaste",
  "disk.wearShort": "Desg.",
  "disk.activity": "Actividad",
  "disk.activityShort": "Act.",
  "disk.powerOnHours": "Horas encendido",
  "disk.firmwareHealth": "Salud del firmware",
  "disk.firmwareHealthOk": "Correcta",
  "disk.firmwareHealthFail": "Revisar",
  "disk.vendorLimit": "límite del fabricante {value}",
  "disk.freeSpace": "{value} libres",
  "disk.advancedDetails": "Detalles avanzados",
  "disk.testThisDisk": "Probar disco",
  "alerts.severity.info": "Informativa",
  "alerts.severity.warn": "Advertencia",
  "alerts.severity.crit": "Crítica",
  "alerts.status.active": "activa",
  "alerts.status.acknowledged": "reconocida",
  "alerts.status.resolved": "resuelta",
  "alerts.status.archived": "archivada",
  "alerts.acknowledge": "Reconocer",
  "alerts.archive": "Archivar",
  "alerts.mute": "Silenciar {duration}",
  "alerts.occurrences": "Cronología de las {count} ocurrencias",
  "events.inferredMapping": "asociación inferida",
  "events.level.error": "Error",
  "events.level.warning": "Aviso",
  "events.level.info": "Info",
  "range.24h": "24 h",
  "range.7d": "7 d",
  "range.30d": "30 d",
  "range.custom": "Personalizado",
  "tests.benchmark": "Lectura y escritura",
  "tests.chkdsk": "Escaneo del sistema de archivos",
  "tests.smartShort": "Autotest SMART corto",
  "tests.running": "En curso",
  "tests.available": "Disponible",
  "tests.unsupported": "No soportado",
  "tests.remaining": "{percent} % · quedan {seconds} s",
  "tests.stopWarning": "La prueba se detiene sola si el disco llega a {tempLimit} o si el espacio libre baja de la reserva de {reserve}.",
  "settings.language": "Idioma",
  "settings.theme": "Tema",
  "settings.theme.light": "Claro",
  "settings.theme.dark": "Oscuro",
  "settings.theme.system": "Según el sistema",
  "settings.useSystemAccent": "Usar el color de acento de Windows",
  "chart.gapRange": "sin datos {from} – {to}",
  "chart.noSamples": "Sin muestras en el intervalo",
  "chart.emptyLabel": "Gráfica sin datos en el intervalo elegido.",
  "chart.summaryLabel": "Serie de {from} a {to} en {unit}. Mínimo {min}, máximo {max}, último valor {last}. Use las flechas para recorrer los puntos.",
  "chart.resolution.raw": "Muestras cada 30 s",
  "chart.resolution.five_minutes": "Promedios de 5 min",
  "chart.resolution.hourly": "Promedios horarios",
  "chart.tempWarnLabel": "Aviso ≥ {value}",
  "chart.tempCritLabel": "Crítico ≥ {value}",
  "alerts.occurrences.one": "Cronología de 1 ocurrencia",
  "alerts.occurrences.other": "Cronología de las {count} ocurrencias",
  "alerts.mutedUntil": "Silenciada hasta {value}",
  "alerts.mutedIndefinitely": "Silenciada hasta reactivación manual",
  "alerts.acknowledgedNote": "Reconocida: sigue activa, el color no cambia.",
  "common.loading": "Cargando…",
  "common.notImplemented": "Esta pantalla todavía no está implementada.",
  "nav.events": "Eventos",
  "startup.failed": "No se pudo iniciar la supervisión",
  "error.deviceNotFound": "Este disco ya no existe en el inventario.",
  "error.storageCollectorFailed": "No se pudo leer el inventario de almacenamiento de Windows.",
  "error.smartctlQueryFailed": "No se pudo consultar smartctl.",
  "error.perfCountersFailed": "No se pudieron leer los contadores de rendimiento.",
  "error.dbLocked": "La base de datos está ocupada; se puede reintentar.",
  "error.dbQueryFailed": "No se pudo completar la operación con el historial guardado.",
  "error.unexpected": "Ha ocurrido un error inesperado al hablar con el servicio de supervisión.",
  "dashboard.noDevices": "No hay discos monitorizados",
  "dashboard.noDevicesHint": "Comprueba que la aplicación se está ejecutando con privilegios de administrador.",
  "dashboard.noDevicesCta": "Buscar dispositivos otra vez",
  "dashboard.deviceCount.one": "1 disco monitorizado",
  "dashboard.deviceCount.other": "{count} discos monitorizados",
  "dashboard.hero.allGood": "Todo en orden",
  "dashboard.hero.allGoodBody": "Ningún disco necesita atención ahora mismo.",
  "dashboard.hero.attentionBody": "Este disco necesita atención. Ábrelo para ver el detalle.",
  "dashboard.hero.openDisk": "Abrir el disco",
  "dashboard.hero.viewAlert": "Ver la alerta",
  "dashboard.hero.lastValid": "último dato válido a las {time}",
  "dashboard.hero.noSeries": "Sin muestras recientes",
  "dashboard.hero.window": "Ventana: {span}",
  "dashboard.hero.collecting": "Recopilando datos…",
  "dashboard.spread.title": "Reparto de estados",
  "dashboard.events.title": "Sucesos del sistema",
  "dashboard.events.empty": "Sin sucesos recientes",
  "disk.noSmartExplain": "El bus de este disco no reenvía los comandos SMART. No es una avería: se vigila su capacidad y los sucesos de Windows, pero no la temperatura ni el desgaste.",
  "disk.noSmartUnreadable": "Este disco ha dejado de responder a las consultas SMART. Se sigue vigilando su capacidad y los sucesos de Windows; la última lectura conocida puede estar anticuada.",
  "disk.noSmartPending": "Todavía no ha llegado la primera lectura SMART de este disco.",
  "disk.capacity": "Ocupación",
  "disk.notFound": "Disco no encontrado",
  "onboarding.title": "Configuración inicial",
  "onboarding.step.welcome": "Bienvenida",
  "onboarding.step.disks": "Discos",
  "onboarding.step.alerts": "Alertas",
  "onboarding.step.done": "Listo",
  "onboarding.stepIndicator": "Paso {n} de {total}",
  "onboarding.progress": "Progreso del asistente",
  "onboarding.skip": "Omitir y usar los valores de fábrica",
  "onboarding.welcome.title": "Te damos la bienvenida a SmartDisk Monitor",
  "onboarding.welcome.body": "SmartDisk vigila la salud de tus discos en segundo plano y te avisa antes de que un problema sea grave. No necesitas saber nada de SMART.",
  "onboarding.welcome.guarantee": "SmartDisk solo lee. No modifica, no repara y no borra nada de tus discos.",
  "onboarding.welcome.cta": "Buscar mis discos",
  "onboarding.welcome.read": "Lee los datos SMART y la salud de cada disco",
  "onboarding.welcome.warn": "Avisa antes de que un problema sea grave",
  "onboarding.welcome.test": "Ejecuta pruebas de disco solo cuando se lo pides",
  "onboarding.disks.title": "Hemos encontrado {count} discos en este equipo",
  "onboarding.disks.body": "Puedes dejar fuera los que no te interesen y ponerles un nombre reconocible. Todo esto se cambia después en Ajustes, y ningún disco se modifica: SmartDisk solo lee.",
  "onboarding.disks.aliasLabel": "Ponle un nombre",
  "onboarding.disks.usbNote": "El disco externo por USB no expone datos SMART: su puente no reenvía esos comandos. Eso no es una avería. Si lo dejas marcado, vigilaremos su capacidad y los sucesos de Windows que lo mencionen, pero no verás temperatura ni desgaste.",
  "onboarding.disks.cta": "Continuar con las alertas",
  "onboarding.disks.empty": "No se ha detectado ningún disco en este equipo",
  "onboarding.disks.rescan": "Volver a buscar",
  "onboarding.disks.continueAnyway": "Continuar igualmente",
  "onboarding.disks.error": "No se ha podido detectar el hardware de disco",
  "onboarding.disks.include": "Vigilar {name}",
  "onboarding.selectedCount": "{selected} de {total} discos seleccionados",
  "onboarding.alerts.title": "¿Cuánto quieres que te avise?",
  "onboarding.alerts.showThresholds": "Ver los umbrales exactos de este perfil",
  "onboarding.alerts.notifyWindows": "Avisarme con una notificación de Windows",
  "onboarding.alerts.notifyWindowsHint": "Aparece cuando la ventana está minimizada.",
  "onboarding.alerts.startWithSystem": "Arrancar SmartDisk con el sistema",
  "onboarding.alerts.startWithSystemHint": "Crea una tarea programada que abre SmartDisk al iniciar sesión.",
  "onboarding.done.title": "Todo listo",
  "onboarding.done.watching": "Vigilando {count} discos con el perfil «{profile}».",
  "onboarding.done.notifyOn": "Te avisaremos con una notificación de Windows.",
  "onboarding.done.notifyOff": "Las notificaciones de Windows están desactivadas.",
  "onboarding.done.firstScan": "Primera lectura en marcha…",
  "onboarding.done.cta": "Ir al panel",
  "onboarding.done.footnote": "Todo esto se cambia en Ajustes.",
  "settings.onboarding.repeat": "Repetir la configuración inicial",
  "settings.onboarding.repeatHint": "Reabre el asistente con los valores actuales. No borra discos, alias ni umbrales.",
  "common.back": "Atrás",
  "common.retry": "Reintentar",
  "disk.noSmartData": "Sin datos SMART",
  "disk.open": "Abrir {name}",
  "global.needsAttention.one": "1 disco necesita atención",
  "global.needsAttention.other": "{count} discos necesitan atención",
  "global.noDevices": "Sin discos monitorizados",
  "error.schemaMismatch": "Los datos recibidos del servicio de supervisión no tienen la forma esperada. Puede que la aplicación y su servicio no coincidan de versión.",
  "nav.monitoring": "Supervisión",
  "disk.noVolumes": "Sin volúmenes montados",
  "error.screenFailed": "No se pudo mostrar esta pantalla",
  "donut.label": "Reparto de estados de los discos monitorizados",
  "alert.status.active": "activa",
  "alert.status.acknowledged": "reconocida",
  "alert.status.resolved": "resuelta",
  "alert.status.archived": "archivada",
  "alert.rule.smart.health.failed.title": "Autoevaluación SMART fallida",
  "alert.rule.smart.health.failed.summary": "El disco ha fallado su propia autoevaluación de salud.",
  "alert.rule.nvme.critical_warning.title": "Aviso crítico del propio disco NVMe",
  "alert.rule.nvme.critical_warning.summary": "El disco ha activado uno o más indicadores de aviso crítico.",
  "alert.rule.smart.media_errors.title": "Nuevos errores de medio",
  "alert.rule.smart.media_errors.summary": "El contador de errores de medio ha aumentado respecto a la lectura anterior.",
  "alert.rule.smart.spare_below_threshold.title": "Reserva de repuesto agotándose",
  "alert.rule.smart.spare_below_threshold.summary": "La reserva de bloques de repuesto ha caído por debajo de su umbral.",
  "alert.rule.smart.wear_high.title": "Desgaste elevado",
  "alert.rule.smart.wear_high.summary": "El disco ha superado el 90 % de su vida útil estimada.",
  "alert.rule.temp.above_configured_warn.title": "Temperatura por encima de lo esperado",
  "alert.rule.temp.above_configured_warn.summary": "La temperatura lleva varios ciclos por encima de 70 °C.",
  "alert.rule.temp.above_configured_crit.title": "Temperatura crítica",
  "alert.rule.temp.above_configured_crit.summary": "La temperatura ha alcanzado 80 °C o más.",
  "alert.rule.temp.above_vendor_limit.title": "Temperatura por encima del límite del fabricante",
  "alert.rule.temp.above_vendor_limit.summary": "La temperatura lleva varios ciclos por encima del límite que marca el fabricante del disco.",
  "alert.rule.capacity.low.title": "Poco espacio libre",
  "alert.rule.capacity.low.summary": "Al volumen le queda poco espacio libre.",
  "alert.rule.capacity.critical.title": "Espacio libre crítico",
  "alert.rule.capacity.critical.summary": "El volumen está a punto de quedarse sin espacio.",
  "alert.rule.smart.unreadable.title": "El disco no responde a las consultas SMART",
  "alert.rule.smart.unreadable.summary": "Varios ciclos seguidos sin poder leer los datos SMART de un disco que sí los daba. Se sigue vigilando su capacidad y los sucesos de Windows.",
  "alert.rule.collector.stalled.title": "Un recopilador lleva rato sin completar un ciclo",
  "alert.rule.collector.stalled.summary": "Una de las fuentes de datos no completa una lectura desde hace varios intervalos. Los datos que muestra pueden estar desactualizados.",
  "alert.rule.events.disk_error.title": "Windows registró un error de disco",
  "alert.rule.events.disk_error.summary": "El sistema operativo anotó un fallo de lectura o escritura en este disco. Conviene revisarlo cuanto antes.",
  "alert.rule.events.filesystem_error.title": "Daño en el sistema de archivos",
  "alert.rule.events.filesystem_error.summary": "Windows detectó una estructura dañada en un volumen de este disco. Puede hacer falta ejecutar una comprobación de disco.",
  "alert.rule.events.filesystem_repaired.title": "El sistema de archivos se reparó solo",
  "alert.rule.events.filesystem_repaired.summary": "Windows corrigió automáticamente una inconsistencia en un volumen. No suele requerir acción, pero conviene vigilarlo.",
  "alert.rule.events.filesystem_repair_storm.title": "Demasiadas reparaciones del sistema de archivos",
  "alert.rule.events.filesystem_repair_storm.summary": "Windows dejó de informar de las reparaciones de un volumen porque se repiten demasiado. Es señal de un problema persistente.",
  "alert.rule.events.controller_reset.title": "La controladora del disco se reinició",
  "alert.rule.events.controller_reset.summary": "Windows tuvo que restablecer el controlador de almacenamiento de este disco. Si se repite, apunta a un problema de hardware o de cable.",
  "alert.rule.events.paging_error.title": "Errores al usar el disco como memoria",
  "alert.rule.events.paging_error.summary": "El archivo de paginación de este disco acumula errores de acceso. Puede degradar el rendimiento del equipo.",
  "alert.rule.events.io_retry.title": "Reintentos de acceso al disco",
  "alert.rule.events.io_retry.summary": "Windows tuvo que reintentar varias operaciones sobre este disco. Suele preceder a un fallo de sectores.",
  "alert.rule.events.delayed_write.title": "Datos perdidos al escribir en el disco",
  "alert.rule.events.delayed_write.summary": "Windows no pudo terminar de escribir datos en un volumen de este disco. Puede haber corrupción o pérdida de información.",
  "alert.rule.events.disk_predictive.title": "El disco avisa de un posible fallo",
  "alert.rule.events.disk_predictive.summary": "El propio disco informó a Windows de que podría fallar pronto. Conviene tener una copia de seguridad al día.",
  "alert.rule.events.storage_space_degraded.title": "Un espacio de almacenamiento está degradado",
  "alert.rule.events.storage_space_degraded.summary": "Windows informó de que un disco virtual de Espacios de almacenamiento perdió redundancia o quedó desconectado.",
  "alert.rule.device.removed_unexpected.title": "Un disco se desconectó sin avisar",
  "alert.rule.device.removed_unexpected.summary": "Este disco desapareció del sistema sin una expulsión previa. Si no fuiste tú, revisa el cable y la alimentación.",
  "alert.rule.inventory.duplicate_id.title": "Dos discos comparten identificadores",
  "alert.rule.inventory.duplicate_id.summary": "Windows detectó dos discos con los mismos identificadores. Mientras dure, los datos que se muestran de ellos pueden confundirse.",
  "alert.fact.ruleKey": "Regla",
  "alert.fact.lastValue": "Último valor",
  "tray.open": "Abrir SmartDisk Monitor",
  "tray.exit": "Salir",
  "tray.tooltip": "SmartDisk Monitor — {summary}",
  "tray.minimizedTitle": "SmartDisk Monitor sigue activo",
  "tray.minimizedBody": "Se minimizó a la bandeja del sistema y sigue vigilando tus discos. Haz clic en el icono para volver a abrirla, o elige «Salir» para cerrarla del todo.",
  "alert.rule.smart.error_log.title": "Errores registrados en el disco",
  "alert.rule.smart.error_log.summary": "El registro de errores del disco ha aumentado.",
  "alerts.filter.active": "Activas",
  "alerts.filter.resolved": "Resueltas",
  "alerts.filter.archived": "Archivadas",
  "alerts.filter.all": "Todas",
  "alerts.empty.title": "Sin alertas",
  "alerts.empty.body": "No hay alertas que coincidan con este filtro.",
  "alerts.detail.empty": "Selecciona una alerta de la lista para ver su detalle.",
  "alerts.actions.acknowledge": "Reconocer",
  "alerts.actions.mute": "Silenciar",
  "alerts.actions.unmute": "Reanudar notificaciones",
  "alerts.actions.archive": "Archivar",
  "alerts.mute.duration": "Duración del silencio",
  "alerts.mute.15": "15 minutos",
  "alerts.mute.60": "1 hora",
  "alerts.mute.480": "8 horas",
  "alerts.mute.indefinite": "Indefinido",
  "alerts.mute.untilDate": "Silenciada hasta {value}",
  "alerts.mute.untilIndefinite": "Silenciada indefinidamente",
  "alerts.timeline.title": "Cronología",
  "alerts.timeline.entry": "Ciclo {cycle} — {when}",
  "alerts.timeline.viewEvent": "Ver el suceso",
  "alerts.archive.confirmTitle": "¿Archivar esta alerta?",
  "alerts.archive.confirmBody": "Se retira de la vista principal. El historial y la cronología se conservan.",
  "alerts.archive.confirmImpact": "No se puede deshacer desde la interfaz: quedará fuera de las pestañas Activas y Resueltas.",
  "dateRange.from": "Desde",
  "dateRange.to": "Hasta",
  "disk.counters": "Contadores",
  "smart.counter.health_passed": "Autoevaluación superada",
  "smart.counter.critical_warning": "Aviso crítico (bits)",
  "smart.counter.media_errors_total": "Errores de medio",
  "smart.counter.error_log_entries_total": "Entradas en el registro de errores",
  "smart.counter.available_spare_percent": "Reserva de repuesto disponible",
  "smart.counter.available_spare_threshold_percent": "Umbral de reserva de repuesto",
  "smart.counter.power_cycles": "Ciclos de encendido",
  "smart.counter.unsafe_shutdowns": "Apagados no seguros",
  "smart.counter.read_bytes_per_second": "Lectura por segundo",
  "smart.counter.write_bytes_per_second": "Escritura por segundo",
  "smart.counter.read_latency_ms": "Latencia de lectura",
  "smart.counter.write_latency_ms": "Latencia de escritura",
  "events.empty.title": "Sin eventos",
  "events.empty.body": "No hay eventos que coincidan con este filtro.",
  "events.filter.level": "Nivel",
  "events.filter.provider": "Proveedor",
  "events.detail.title": "Detalle del evento",
  "events.detail.rawXml": "XML original",
  "events.detail.systemText": "Texto original del sistema",
  "events.detail.empty": "Selecciona un evento de la lista para ver su detalle.",
  "events.loadMore": "Cargar más",
  "tests.picker.device": "Disco",
  "tests.picker.volume": "Volumen",
  "tests.cta.configure": "Configurar y ejecutar",
  "tests.chip.available": "Disponible",
  "tests.chip.running": "En curso",
  "tests.chip.unsupported": "No compatible",
  "tests.chkdskUnsupportedReason": "Este volumen no es NTFS: chkdsk /scan solo existe para ese sistema de archivos.",
  "tests.cards.benchmark.title": "Lectura y escritura",
  "tests.cards.benchmark.desc": "Crea un archivo temporal nuevo, escribe, sincroniza, lee y verifica el patrón. Nunca sobrescribe archivos existentes.",
  "tests.cards.chkdsk.title": "Escaneo del sistema de archivos",
  "tests.cards.chkdsk.desc": "Ejecuta chkdsk /scan en línea sobre un volumen NTFS compatible y conserva la salida completa. Sin opciones de reparación.",
  "tests.cards.autotest.title": "Autotest SMART corto",
  "tests.cards.autotest.desc": "Solicita al firmware su autotest corto. Solo se ofrece si el dispositivo declara compatibilidad; no puede coincidir con el benchmark.",
  "tests.confirm.benchmark.title": "Probar lectura y escritura",
  "tests.confirm.benchmark.body": "Se creará un archivo temporal de 1 GiB en el volumen elegido, en bloques de 1 MiB y acceso secuencial. Se elimina automáticamente al terminar o cancelar.",
  "tests.confirm.benchmark.impact": "El rendimiento del equipo, la temperatura del disco y sus escrituras pueden verse afectados mientras dure la prueba.",
  "tests.confirm.benchmark.confirmLabel": "Iniciar prueba",
  "tests.confirm.chkdsk.title": "Ejecutar chkdsk /scan en {letter}:",
  "tests.confirm.chkdsk.body": "Se comprobará el sistema de archivos en línea. No se programa ninguna reparación fuera de línea y no se modifica ningún archivo.",
  "tests.confirm.chkdsk.impact": "El análisis puede tardar varios minutos y aumentar temporalmente la actividad del disco.",
  "tests.confirm.chkdsk.confirmLabel": "Ejecutar análisis",
  "tests.confirm.autotest.title": "Ejecutar autotest SMART corto",
  "tests.confirm.autotest.body": "El firmware ejecutará su autotest corto, que puede degradar temporalmente el rendimiento. No puede solicitarse a la vez que un benchmark sobre el mismo disco.",
  "tests.confirm.autotest.impact": "El resultado puede tardar varios minutos en estar disponible.",
  "tests.confirm.autotest.confirmLabel": "Iniciar autotest",
  "tests.active.title": "Prueba en curso",
  "tests.active.cancel": "Cancelar",
  "tests.active.progress": "{percent} %",
  "tests.active.indeterminate": "En curso",
  "tests.active.warning": "La prueba se detiene sola si el disco alcanza el límite térmico crítico o si el espacio libre baja de la reserva de seguridad. Mientras dure, el rendimiento del equipo puede bajar.",
  "tests.metrics.write": "Escritura",
  "tests.metrics.read": "Lectura",
  "tests.metrics.latency": "Latencia media",
  "tests.metrics.temperature": "Temperatura",
  "tests.history.title": "Historial de pruebas",
  "tests.history.empty": "Todavía no se ha ejecutado ninguna prueba.",
  "tests.history.inProgress": "{percent} % completado",
  "tests.history.inProgressIndeterminate": "En curso",
  "tests.type.benchmark": "Lectura/escritura",
  "tests.type.chkdsk": "chkdsk /scan",
  "tests.type.autotest": "Autotest corto",
  "tests.status.pending": "Pendiente",
  "tests.status.running": "En curso",
  "tests.status.cancelling": "Cancelando",
  "tests.status.completed": "Completada",
  "tests.status.failed": "Fallida",
  "tests.status.cancelled": "Cancelada",
  "tests.status.interrupted": "Interrumpida",
  "tests.stoppedReason.completed": "Sin incidencias",
  "tests.stoppedReason.cancelled": "Cancelada por el usuario",
  "tests.stoppedReason.thermal": "Detenida en el límite térmico",
  "tests.stoppedReason.space": "Detenida por falta de espacio",
  "tests.stoppedReason.error": "Detenida por un error",
  "error.volumeNotFound": "Este volumen ya no existe en el inventario.",
  "error.testBusy": "Ya hay una prueba en curso sobre ese disco.",
  "error.testUnsupported": "El dispositivo no admite esta prueba.",
  "error.testInsufficientSpace": "No queda espacio suficiente tras la reserva de seguridad.",
  "error.testIoFailed": "No se pudo preparar o ejecutar la prueba.",
  "error.pathInvalid": "La ruta no es válida para esta operación.",
  "error.exportWriteFailed": "No se pudo escribir el destino elegido.",
  "reports.range.title": "Intervalo",
  "reports.devices.title": "Discos incluidos",
  "reports.includeSerials.label": "Incluir números de serie",
  "reports.includeSerials.hint": "Los números de serie identifican el disco físico; se omiten por defecto.",
  "reports.format.csv": "CSV",
  "reports.format.csvDesc": "Volcado tabular completo: una fila por disco, métrica y marca de tiempo.",
  "reports.format.json": "JSON",
  "reports.format.jsonDesc": "El mismo volcado que el CSV, estructurado por disco y con sus muestras anidadas.",
  "reports.format.html": "HTML",
  "reports.format.htmlDesc": "Resumen legible e imprimible: identidad de cada disco y sus alertas en el intervalo.",
  "reports.cta.export": "Exportar",
  "reports.export.savedAt": "Guardado en {path}",
  "reports.diagnostic.title": "Paquete de diagnóstico",
  "reports.diagnostic.desc": "Configuración, eventos, capturas SMART y registro de actividad en un único ZIP, anonimizado por defecto.",
  "reports.diagnostic.includeIdentifiers.label": "Incluir identificadores reales",
  "reports.diagnostic.includeIdentifiers.hint": "Por defecto se sustituyen números de serie, nombre de equipo y usuario.",
  "reports.diagnostic.includeIdentifiers.warning": "El paquete incluirá números de serie, nombre de equipo y usuario reales.",
  "reports.diagnostic.zipFilter": "Archivo ZIP",
  "reports.diagnostic.cta.preview": "Ver contenido",
  "reports.diagnostic.cta.save": "Guardar paquete",
  "reports.diagnostic.preview.total": "Tamaño total: {size}",
  "reports.diagnostic.preview.redacted": "Se ha sustituido:",
  "diagnostic.entry.manifest": "Manifiesto del paquete",
  "diagnostic.entry.settings": "Ajustes",
  "diagnostic.entry.events": "Eventos del sistema",
  "diagnostic.entry.smart": "Captura SMART",
  "diagnostic.entry.logs": "Registro de actividad",
  "diagnostic.entry.other": "Otro contenido",
  "diagnostic.redacted.serialNumber": "números de serie",
  "diagnostic.redacted.computerName": "nombre de equipo",
  "diagnostic.redacted.userPaths": "usuario y rutas personales",
  "settings.appearance.title": "Apariencia",
  "settings.appearance.theme.light": "Claro",
  "settings.appearance.theme.dark": "Oscuro",
  "settings.appearance.theme.system": "Sistema",
  "settings.appearance.language.es": "Español",
  "settings.appearance.language.en": "Inglés",
  "settings.appearance.useSystemAccent.label": "Usar el color de acento de Windows",
  "settings.appearance.useSystemAccent.hint": "Sustituye el morado de la aplicación por el color que tengas configurado en Windows.",
  "settings.notifications.sound.label": "Sonido en las notificaciones",
  "settings.notifications.sound.hint": "Desactivado de fábrica.",
  "settings.cta.restoreDefaults": "Restaurar valores de fábrica",
  "settings.schedule.title": "Frecuencias",
  "settings.schedule.rangeHint": "Entre {min} y {max} s",
  "settings.schedule.metricsFast": "Temperatura, actividad, capacidad y latencia",
  "settings.schedule.smartFull": "SMART completo",
  "settings.schedule.events": "Eventos de Windows",
  "settings.schedule.discovery": "Detección de altas y bajas",
  "settings.unit.seconds": "s",
  "settings.unit.days": "días",
  "settings.alerts.title": "Umbrales de alerta",
  "settings.alerts.tempPrecedence": "Solo se aplican a discos sin límite de temperatura declarado por el fabricante; si el fabricante lo declara, ese valor manda.",
  "settings.alerts.tempWarn": "Temperatura de aviso",
  "settings.alerts.tempCrit": "Temperatura crítica",
  "settings.alerts.capacityWarnPercent": "Capacidad libre de aviso",
  "settings.alerts.capacityCritPercent": "Capacidad libre crítica",
  "settings.alerts.profile.title": "Perfil",
  "settings.alerts.profile.hint": "Elige cuánto quieres que te avise. Ajusta los doce umbrales de golpe; puedes afinar cualquiera abajo.",
  "settings.alerts.profile.cautious": "Prudente",
  "settings.alerts.profile.cautiousHint": "Avisa antes. Más avisos.",
  "settings.alerts.profile.balanced": "Equilibrado",
  "settings.alerts.profile.balancedHint": "Recomendado.",
  "settings.alerts.profile.quiet": "Solo lo grave",
  "settings.alerts.profile.quietHint": "Solo condiciones críticas. Menos avisos.",
  "settings.alerts.profile.custom": "Personalizado (a partir de {base})",
  "settings.alerts.wearWarn": "Desgaste de aviso",
  "settings.alerts.wearCrit": "Desgaste crítico",
  "settings.alerts.mediaErrorsWarn": "Errores de medios de aviso",
  "settings.alerts.mediaErrorsCrit": "Errores de medios críticos",
  "settings.alerts.mediaErrorsHint": "Incremento del contador entre dos lecturas que basta para avisar.",
  "settings.alerts.driverRetryWarn": "Reintentos del controlador de aviso",
  "settings.alerts.driverRetryCrit": "Reintentos del controlador críticos",
  "settings.alerts.driverRetryHint": "Pendiente: aún no hay ninguna regla que vigile los reintentos del controlador.",
  "settings.retention.title": "Retención e historial",
  "settings.retention.neverPurged": "Las alertas, sus ocurrencias críticas, los eventos vinculados y las ejecuciones de pruebas nunca se borran por retención.",
  "settings.retention.raw": "Muestras crudas",
  "settings.retention.fiveMinutes": "Agregados de 5 minutos",
  "settings.retention.hourly": "Agregados horarios",
  "settings.retention.freeSpaceGuard": "La escritura de historial avisa por debajo de {warn} y se detiene por debajo de {halt}, sin afectar a la monitorización ni a las alertas en vivo.",
  "settings.logging.title": "Registro de actividad",
  "settings.collection.label": "Recopilación de datos activa",
  "settings.collection.hint": "Desactívala para pausar la lectura de SMART, contadores y eventos. Una actualización manual sigue disponible.",
  "settings.logging.verbose.label": "Modo detallado",
  "settings.logging.verbose.hint": "Actívalo para reproducir un fallo con más información.",
  "settings.logging.cta.openFolder": "Abrir carpeta del registro",
  "settings.lifecycle.title": "Al cerrar la ventana",
  "settings.lifecycle.closeAction.label": "Qué hace el botón de cerrar",
  "settings.lifecycle.closeAction.minimize": "Minimizar a la bandeja",
  "settings.lifecycle.closeAction.minimizeHint": "La aplicación sigue monitorizando en segundo plano.",
  "settings.lifecycle.closeAction.exit": "Salir de la aplicación",
  "settings.lifecycle.closeAction.exitHint": "Deja de monitorizar hasta que se vuelva a abrir.",
  "settings.dangerZone.title": "Borrar todos los datos",
  "settings.dangerZone.desc": "Elimina todo el historial guardado: inventario, métricas, alertas, eventos y ejecuciones de pruebas. La aplicación queda como recién instalada.",
  "settings.dangerZone.confirmPhraseLabel": "Escribe «{phrase}» para confirmar",
  "settings.dangerZone.cta": "Borrar todos los datos",
  "settings.dangerZone.confirmTitle": "¿Borrar todos los datos?",
  "settings.dangerZone.confirmBody": "Se eliminará todo el historial guardado por la aplicación: inventario de discos, métricas, alertas, eventos vinculados, ejecuciones de pruebas y preferencias. Esta acción no se puede deshacer.",
  "settings.dangerZone.confirmImpact": "Al terminar, la aplicación queda como recién instalada y vuelve a mostrar el asistente inicial.",
  "settings.dangerZone.confirmLabel": "Borrar todos los datos",
  "settings.dangerZone.done": "Todos los datos se han eliminado.",
  "nav.about": "Acerca de",
  "about.title": "Acerca de",
  "about.body": "De {author}. Licencia MIT. Incluye smartmontools (GPLv2) y la tipografía Instrument Sans (SIL OFL 1.1); consulta los avisos de terceros completos en el instalador. Repositorio: https://github.com/danimardo/smartdisk-monitor",
  "about.cta.copy": "Copiar información"
}
```


---

# 27. i18n/en.json

Fichero de origen: `src/lib/i18n/en.json`

```json
{
  "app.name": "SmartDisk Monitor",
  "common.notAvailable": "Not available",
  "common.unsupported": "Not supported",
  "common.noData": "No data",
  "common.sourceError": "Source error",
  "common.refresh": "Refresh",
  "common.cancel": "Cancel",
  "common.close": "Close",
  "common.continue": "Continue",
  "common.technicalDetail": "Technical detail",
  "common.viewAll": "View all",
  "common.updatedAgo": "{value} ago",
  "common.moment": "a moment",
  "health.ok": "Healthy",
  "health.warn": "Warning",
  "health.crit": "Critical",
  "health.unknown": "No SMART data",
  "global.allGood": "All good",
  "global.paused": "Paused",
  "global.loading": "Checking…",
  "nav.alertsUnread": "Alerts, {count} unread",
  "nav.dashboard": "Overview",
  "nav.alerts": "Alerts",
  "nav.tests": "Tests & diagnostics",
  "nav.reports": "Reports",
  "nav.settings": "Settings",
  "nav.monitoredDisks": "Monitored disks",
  "nav.pause": "Pause collection",
  "nav.resume": "Resume collection",
  "disk.temperature": "Temperature",
  "disk.temperatureShort": "Temp.",
  "disk.wear": "Wear",
  "disk.wearShort": "Wear",
  "disk.activity": "Activity",
  "disk.activityShort": "Act.",
  "disk.powerOnHours": "Power-on hours",
  "disk.firmwareHealth": "Firmware health",
  "disk.firmwareHealthOk": "Passed",
  "disk.firmwareHealthFail": "Check",
  "disk.vendorLimit": "vendor limit {value}",
  "disk.freeSpace": "{value} free",
  "disk.advancedDetails": "Advanced details",
  "disk.testThisDisk": "Test this disk",
  "alerts.severity.info": "Informational",
  "alerts.severity.warn": "Warning",
  "alerts.severity.crit": "Critical",
  "alerts.status.active": "active",
  "alerts.status.acknowledged": "acknowledged",
  "alerts.status.resolved": "resolved",
  "alerts.status.archived": "archived",
  "alerts.acknowledge": "Acknowledge",
  "alerts.archive": "Archive",
  "alerts.mute": "Mute {duration}",
  "alerts.occurrences": "Timeline of {count} occurrences",
  "events.inferredMapping": "inferred mapping",
  "events.level.error": "Error",
  "events.level.warning": "Warning",
  "events.level.info": "Info",
  "range.24h": "24 h",
  "range.7d": "7 d",
  "range.30d": "30 d",
  "range.custom": "Custom",
  "tests.benchmark": "Read & write test",
  "tests.chkdsk": "File system scan",
  "tests.smartShort": "SMART short self-test",
  "tests.running": "Running",
  "tests.available": "Available",
  "tests.unsupported": "Not supported",
  "tests.remaining": "{percent} % · {seconds} s left",
  "tests.stopWarning": "The test stops by itself if the disk reaches {tempLimit} or free space drops below the {reserve} reserve.",
  "settings.language": "Language",
  "settings.theme": "Theme",
  "settings.theme.light": "Light",
  "settings.theme.dark": "Dark",
  "settings.theme.system": "Follow system",
  "settings.useSystemAccent": "Use the Windows accent colour",
  "chart.gapRange": "no data {from} – {to}",
  "chart.noSamples": "No samples in the range",
  "chart.emptyLabel": "No data in the selected range.",
  "chart.summaryLabel": "Series from {from} to {to} in {unit}. Minimum {min}, maximum {max}, latest {last}. Use the arrow keys to step through the points.",
  "chart.resolution.raw": "Samples every 30 s",
  "chart.resolution.five_minutes": "5-minute averages",
  "chart.resolution.hourly": "Hourly averages",
  "chart.tempWarnLabel": "Warning ≥ {value}",
  "chart.tempCritLabel": "Critical ≥ {value}",
  "alerts.occurrences.one": "Timeline of 1 occurrence",
  "alerts.occurrences.other": "Timeline of {count} occurrences",
  "alerts.mutedUntil": "Muted until {value}",
  "alerts.mutedIndefinitely": "Muted until manually re-enabled",
  "alerts.acknowledgedNote": "Acknowledged: still active, the colour does not change.",
  "common.loading": "Loading…",
  "common.notImplemented": "This screen is not implemented yet.",
  "nav.events": "Events",
  "startup.failed": "Monitoring could not start",
  "error.deviceNotFound": "This disk no longer exists in the inventory.",
  "error.storageCollectorFailed": "The Windows storage inventory could not be read.",
  "error.smartctlQueryFailed": "smartctl could not be queried.",
  "error.perfCountersFailed": "The performance counters could not be read.",
  "error.dbLocked": "The database is busy; retrying may work.",
  "error.dbQueryFailed": "The operation against the saved history could not be completed.",
  "error.unexpected": "An unexpected error occurred while talking to the monitoring service.",
  "dashboard.noDevices": "No monitored disks",
  "dashboard.noDevicesHint": "Check that the application is running with administrator privileges.",
  "dashboard.noDevicesCta": "Look for devices again",
  "dashboard.deviceCount.one": "1 monitored disk",
  "dashboard.deviceCount.other": "{count} monitored disks",
  "dashboard.hero.allGood": "All good",
  "dashboard.hero.allGoodBody": "No disk needs attention right now.",
  "dashboard.hero.attentionBody": "This disk needs attention. Open it for details.",
  "dashboard.hero.openDisk": "Open disk",
  "dashboard.hero.viewAlert": "View alert",
  "dashboard.hero.lastValid": "last valid reading at {time}",
  "dashboard.hero.noSeries": "No recent samples",
  "dashboard.hero.window": "Window: {span}",
  "dashboard.hero.collecting": "Collecting data…",
  "dashboard.spread.title": "Status spread",
  "dashboard.events.title": "System events",
  "dashboard.events.empty": "No recent events",
  "disk.noSmartExplain": "This disk's bus does not forward SMART commands. That is not a fault: its capacity and Windows events are still watched, but not temperature or wear.",
  "disk.noSmartUnreadable": "This disk has stopped responding to SMART queries. Its capacity and Windows events are still watched; the last known reading may be stale.",
  "disk.noSmartPending": "The first SMART reading for this disk has not arrived yet.",
  "disk.capacity": "Usage",
  "disk.notFound": "Disk not found",
  "onboarding.title": "Initial setup",
  "onboarding.step.welcome": "Welcome",
  "onboarding.step.disks": "Disks",
  "onboarding.step.alerts": "Alerts",
  "onboarding.step.done": "Done",
  "onboarding.stepIndicator": "Step {n} of {total}",
  "onboarding.progress": "Wizard progress",
  "onboarding.skip": "Skip and use factory defaults",
  "onboarding.welcome.title": "Welcome to SmartDisk Monitor",
  "onboarding.welcome.body": "SmartDisk watches your disks' health in the background and warns you before a problem becomes serious. You don't need to know anything about SMART.",
  "onboarding.welcome.guarantee": "SmartDisk only reads. It does not modify, repair or delete anything on your disks.",
  "onboarding.welcome.cta": "Find my disks",
  "onboarding.welcome.read": "Reads each disk's SMART data and health",
  "onboarding.welcome.warn": "Warns you before a problem becomes serious",
  "onboarding.welcome.test": "Runs disk tests only when you ask",
  "onboarding.disks.title": "We found {count} disks on this PC",
  "onboarding.disks.body": "You can leave out the ones you don't care about and give them a recognisable name. All of this can be changed later in Settings, and no disk is modified: SmartDisk only reads.",
  "onboarding.disks.aliasLabel": "Give it a name",
  "onboarding.disks.usbNote": "The external USB disk does not expose SMART data: its bridge does not forward those commands. That is not a fault. If you leave it checked we will watch its capacity and the Windows events that mention it, but you will not see temperature or wear.",
  "onboarding.disks.cta": "Continue to alerts",
  "onboarding.disks.empty": "No disks detected on this PC",
  "onboarding.disks.rescan": "Search again",
  "onboarding.disks.continueAnyway": "Continue anyway",
  "onboarding.disks.error": "Could not detect the disk hardware",
  "onboarding.disks.include": "Watch {name}",
  "onboarding.selectedCount": "{selected} of {total} disks selected",
  "onboarding.alerts.title": "How much should it warn you?",
  "onboarding.alerts.showThresholds": "Show this profile's exact thresholds",
  "onboarding.alerts.notifyWindows": "Notify me with a Windows notification",
  "onboarding.alerts.notifyWindowsHint": "Shows when the window is minimised.",
  "onboarding.alerts.startWithSystem": "Start SmartDisk with the system",
  "onboarding.alerts.startWithSystemHint": "Creates a scheduled task that opens SmartDisk at sign-in.",
  "onboarding.done.title": "You're all set",
  "onboarding.done.watching": "Watching {count} disks with the “{profile}” profile.",
  "onboarding.done.notifyOn": "We'll notify you with a Windows notification.",
  "onboarding.done.notifyOff": "Windows notifications are turned off.",
  "onboarding.done.firstScan": "First reading under way…",
  "onboarding.done.cta": "Go to the dashboard",
  "onboarding.done.footnote": "All of this can be changed in Settings.",
  "settings.onboarding.repeat": "Repeat the initial setup",
  "settings.onboarding.repeatHint": "Reopens the wizard with your current values. It does not delete disks, aliases or thresholds.",
  "common.back": "Back",
  "common.retry": "Retry",
  "disk.noSmartData": "No SMART data",
  "disk.open": "Open {name}",
  "global.needsAttention.one": "1 disk needs attention",
  "global.needsAttention.other": "{count} disks need attention",
  "global.noDevices": "No monitored disks",
  "error.schemaMismatch": "The data received from the monitoring service does not have the expected shape. The application and its service may be on different versions.",
  "nav.monitoring": "Monitoring",
  "disk.noVolumes": "No mounted volumes",
  "error.screenFailed": "This screen could not be shown",
  "donut.label": "Breakdown of monitored disk states",
  "alert.status.active": "active",
  "alert.status.acknowledged": "acknowledged",
  "alert.status.resolved": "resolved",
  "alert.status.archived": "archived",
  "alert.rule.smart.health.failed.title": "Failed SMART self-assessment",
  "alert.rule.smart.health.failed.summary": "The disk has failed its own health self-assessment.",
  "alert.rule.nvme.critical_warning.title": "Critical warning from the NVMe disk itself",
  "alert.rule.nvme.critical_warning.summary": "The disk has raised one or more critical warning flags.",
  "alert.rule.smart.media_errors.title": "New media errors",
  "alert.rule.smart.media_errors.summary": "The media error count has increased since the previous reading.",
  "alert.rule.smart.spare_below_threshold.title": "Spare capacity running low",
  "alert.rule.smart.spare_below_threshold.summary": "The spare block reserve has dropped below its threshold.",
  "alert.rule.smart.wear_high.title": "High wear",
  "alert.rule.smart.wear_high.summary": "The disk has exceeded 90% of its estimated usable life.",
  "alert.rule.temp.above_configured_warn.title": "Temperature above expected",
  "alert.rule.temp.above_configured_warn.summary": "Temperature has stayed above 70 °C for several cycles.",
  "alert.rule.temp.above_configured_crit.title": "Critical temperature",
  "alert.rule.temp.above_configured_crit.summary": "Temperature has reached 80 °C or higher.",
  "alert.rule.temp.above_vendor_limit.title": "Above the vendor temperature limit",
  "alert.rule.temp.above_vendor_limit.summary": "The temperature has stayed above the disk vendor's limit for several cycles.",
  "alert.rule.capacity.low.title": "Low free space",
  "alert.rule.capacity.low.summary": "This volume is running low on free space.",
  "alert.rule.capacity.critical.title": "Critical free space",
  "alert.rule.capacity.critical.summary": "This volume is about to run out of space.",
  "alert.rule.smart.unreadable.title": "The disk is not answering SMART queries",
  "alert.rule.smart.unreadable.summary": "Several cycles in a row without being able to read SMART data from a disk that used to provide it. Its capacity and Windows events are still watched.",
  "alert.rule.collector.stalled.title": "A collector hasn't completed a cycle in a while",
  "alert.rule.collector.stalled.summary": "One of the data sources hasn't completed a read for several intervals. The data it shows may be out of date.",
  "alert.rule.events.disk_error.title": "Windows logged a disk error",
  "alert.rule.events.disk_error.summary": "The operating system recorded a read or write failure on this disk. It should be checked soon.",
  "alert.rule.events.filesystem_error.title": "File system damage",
  "alert.rule.events.filesystem_error.summary": "Windows detected a damaged structure on a volume of this disk. A disk check may be needed.",
  "alert.rule.events.filesystem_repaired.title": "The file system repaired itself",
  "alert.rule.events.filesystem_repaired.summary": "Windows automatically fixed an inconsistency on a volume. It usually needs no action, but keep an eye on it.",
  "alert.rule.events.filesystem_repair_storm.title": "Too many file system repairs",
  "alert.rule.events.filesystem_repair_storm.summary": "Windows stopped reporting repairs on a volume because they happen too often. This points to a persistent problem.",
  "alert.rule.events.controller_reset.title": "The disk controller was reset",
  "alert.rule.events.controller_reset.summary": "Windows had to reset the storage controller for this disk. If it repeats, it suggests a hardware or cable problem.",
  "alert.rule.events.paging_error.title": "Errors using the disk as memory",
  "alert.rule.events.paging_error.summary": "The paging file on this disk is accumulating access errors. It can slow the computer down.",
  "alert.rule.events.io_retry.title": "Disk access retries",
  "alert.rule.events.io_retry.summary": "Windows had to retry several operations on this disk. This often precedes a sector failure.",
  "alert.rule.events.delayed_write.title": "Data lost while writing to the disk",
  "alert.rule.events.delayed_write.summary": "Windows could not finish writing data to a volume on this disk. There may be corruption or data loss.",
  "alert.rule.events.disk_predictive.title": "The disk warns of a possible failure",
  "alert.rule.events.disk_predictive.summary": "The disk itself told Windows it might fail soon. Make sure you have a current backup.",
  "alert.rule.events.storage_space_degraded.title": "A storage space is degraded",
  "alert.rule.events.storage_space_degraded.summary": "Windows reported that a Storage Spaces virtual disk lost redundancy or went offline.",
  "alert.rule.device.removed_unexpected.title": "A disk was disconnected without warning",
  "alert.rule.device.removed_unexpected.summary": "This disk disappeared from the system without being ejected first. If it wasn't you, check the cable and power.",
  "alert.rule.inventory.duplicate_id.title": "Two disks share identifiers",
  "alert.rule.inventory.duplicate_id.summary": "Windows detected two disks with the same identifiers. While this lasts, the data shown for them may be mixed up.",
  "alert.fact.ruleKey": "Rule",
  "alert.fact.lastValue": "Last value",
  "tray.open": "Open SmartDisk Monitor",
  "tray.exit": "Exit",
  "tray.tooltip": "SmartDisk Monitor — {summary}",
  "tray.minimizedTitle": "SmartDisk Monitor is still running",
  "tray.minimizedBody": "It minimized to the system tray and keeps watching your disks. Click the icon to reopen it, or choose \"Exit\" to close it completely.",
  "alert.rule.smart.error_log.title": "Errors logged on the disk",
  "alert.rule.smart.error_log.summary": "The disk's error log count has increased.",
  "alerts.filter.active": "Active",
  "alerts.filter.resolved": "Resolved",
  "alerts.filter.archived": "Archived",
  "alerts.filter.all": "All",
  "alerts.empty.title": "No alerts",
  "alerts.empty.body": "No alerts match this filter.",
  "alerts.detail.empty": "Select an alert from the list to see its detail.",
  "alerts.actions.acknowledge": "Acknowledge",
  "alerts.actions.mute": "Mute",
  "alerts.actions.unmute": "Resume notifications",
  "alerts.actions.archive": "Archive",
  "alerts.mute.duration": "Mute duration",
  "alerts.mute.15": "15 minutes",
  "alerts.mute.60": "1 hour",
  "alerts.mute.480": "8 hours",
  "alerts.mute.indefinite": "Indefinite",
  "alerts.mute.untilDate": "Muted until {value}",
  "alerts.mute.untilIndefinite": "Muted indefinitely",
  "alerts.timeline.title": "Timeline",
  "alerts.timeline.entry": "Cycle {cycle} — {when}",
  "alerts.timeline.viewEvent": "View the event",
  "alerts.archive.confirmTitle": "Archive this alert?",
  "alerts.archive.confirmBody": "It's removed from the main view. History and timeline are kept.",
  "alerts.archive.confirmImpact": "Can't be undone from the interface: it will drop out of the Active and Resolved tabs.",
  "dateRange.from": "From",
  "dateRange.to": "To",
  "disk.counters": "Counters",
  "smart.counter.health_passed": "Self-assessment passed",
  "smart.counter.critical_warning": "Critical warning (bits)",
  "smart.counter.media_errors_total": "Media errors",
  "smart.counter.error_log_entries_total": "Error log entries",
  "smart.counter.available_spare_percent": "Available spare",
  "smart.counter.available_spare_threshold_percent": "Available spare threshold",
  "smart.counter.power_cycles": "Power cycles",
  "smart.counter.unsafe_shutdowns": "Unsafe shutdowns",
  "smart.counter.read_bytes_per_second": "Read per second",
  "smart.counter.write_bytes_per_second": "Write per second",
  "smart.counter.read_latency_ms": "Read latency",
  "smart.counter.write_latency_ms": "Write latency",
  "events.empty.title": "No events",
  "events.empty.body": "No events match this filter.",
  "events.filter.level": "Level",
  "events.filter.provider": "Provider",
  "events.detail.title": "Event detail",
  "events.detail.rawXml": "Raw XML",
  "events.detail.systemText": "Original system text",
  "events.detail.empty": "Select an event from the list to see its detail.",
  "events.loadMore": "Load more",
  "tests.picker.device": "Disk",
  "tests.picker.volume": "Volume",
  "tests.cta.configure": "Configure and run",
  "tests.chip.available": "Available",
  "tests.chip.running": "Running",
  "tests.chip.unsupported": "Not supported",
  "tests.chkdskUnsupportedReason": "This volume isn't NTFS: chkdsk /scan only exists for that file system.",
  "tests.cards.benchmark.title": "Read and write",
  "tests.cards.benchmark.desc": "Creates a new temporary file, writes, syncs, reads and verifies the pattern. Never overwrites an existing file.",
  "tests.cards.chkdsk.title": "File system scan",
  "tests.cards.chkdsk.desc": "Runs chkdsk /scan online on a compatible NTFS volume and keeps the full output. No repair options.",
  "tests.cards.autotest.title": "Short SMART self-test",
  "tests.cards.autotest.desc": "Asks the firmware to run its short self-test. Only offered if the device declares support; it can't coincide with the benchmark.",
  "tests.confirm.benchmark.title": "Test read and write",
  "tests.confirm.benchmark.body": "A 1 GiB temporary file will be created on the chosen volume, in 1 MiB blocks with sequential access. It's removed automatically when it finishes or is cancelled.",
  "tests.confirm.benchmark.impact": "The computer's performance, the disk's temperature and its writes may be affected while the test runs.",
  "tests.confirm.benchmark.confirmLabel": "Start test",
  "tests.confirm.chkdsk.title": "Run chkdsk /scan on {letter}:",
  "tests.confirm.chkdsk.body": "The file system will be checked online. No offline repair is scheduled and no file is modified.",
  "tests.confirm.chkdsk.impact": "The scan can take several minutes and temporarily increase disk activity.",
  "tests.confirm.chkdsk.confirmLabel": "Run scan",
  "tests.confirm.autotest.title": "Run short SMART self-test",
  "tests.confirm.autotest.body": "The firmware will run its short self-test, which may temporarily degrade performance. It can't be requested at the same time as a benchmark on the same disk.",
  "tests.confirm.autotest.impact": "The result may take several minutes to become available.",
  "tests.confirm.autotest.confirmLabel": "Start self-test",
  "tests.active.title": "Test in progress",
  "tests.active.cancel": "Cancel",
  "tests.active.progress": "{percent}%",
  "tests.active.indeterminate": "In progress",
  "tests.active.warning": "The test stops on its own if the disk reaches the critical thermal limit or if free space drops below the safety reserve. Computer performance may drop while it runs.",
  "tests.metrics.write": "Write",
  "tests.metrics.read": "Read",
  "tests.metrics.latency": "Average latency",
  "tests.metrics.temperature": "Temperature",
  "tests.history.title": "Test history",
  "tests.history.empty": "No test has run yet.",
  "tests.history.inProgress": "{percent}% complete",
  "tests.history.inProgressIndeterminate": "In progress",
  "tests.type.benchmark": "Read/write",
  "tests.type.chkdsk": "chkdsk /scan",
  "tests.type.autotest": "Short self-test",
  "tests.status.pending": "Pending",
  "tests.status.running": "Running",
  "tests.status.cancelling": "Cancelling",
  "tests.status.completed": "Completed",
  "tests.status.failed": "Failed",
  "tests.status.cancelled": "Cancelled",
  "tests.status.interrupted": "Interrupted",
  "tests.stoppedReason.completed": "No issues",
  "tests.stoppedReason.cancelled": "Cancelled by the user",
  "tests.stoppedReason.thermal": "Stopped at the thermal limit",
  "tests.stoppedReason.space": "Stopped for lack of space",
  "tests.stoppedReason.error": "Stopped due to an error",
  "error.volumeNotFound": "This volume no longer exists in the inventory.",
  "error.testBusy": "There's already a test running on that disk.",
  "error.testUnsupported": "The device doesn't support this test.",
  "error.testInsufficientSpace": "There isn't enough space left after the safety reserve.",
  "error.testIoFailed": "The test couldn't be prepared or run.",
  "error.pathInvalid": "The path isn't valid for this operation.",
  "error.exportWriteFailed": "The chosen destination couldn't be written.",
  "reports.range.title": "Interval",
  "reports.devices.title": "Disks included",
  "reports.includeSerials.label": "Include serial numbers",
  "reports.includeSerials.hint": "Serial numbers identify the physical disk; omitted by default.",
  "reports.format.csv": "CSV",
  "reports.format.csvDesc": "Full tabular dump: one row per disk, metric and timestamp.",
  "reports.format.json": "JSON",
  "reports.format.jsonDesc": "The same dump as the CSV, structured by disk with its samples nested.",
  "reports.format.html": "HTML",
  "reports.format.htmlDesc": "Readable, printable summary: each disk's identity and its alerts in the interval.",
  "reports.cta.export": "Export",
  "reports.export.savedAt": "Saved to {path}",
  "reports.diagnostic.title": "Diagnostic package",
  "reports.diagnostic.desc": "Settings, events, SMART captures and the activity log in a single ZIP, anonymized by default.",
  "reports.diagnostic.includeIdentifiers.label": "Include real identifiers",
  "reports.diagnostic.includeIdentifiers.hint": "By default, serial numbers, computer name and username are replaced.",
  "reports.diagnostic.includeIdentifiers.warning": "The package will include real serial numbers, computer name and username.",
  "reports.diagnostic.zipFilter": "ZIP archive",
  "reports.diagnostic.cta.preview": "View contents",
  "reports.diagnostic.cta.save": "Save package",
  "reports.diagnostic.preview.total": "Total size: {size}",
  "reports.diagnostic.preview.redacted": "Replaced:",
  "diagnostic.entry.manifest": "Package manifest",
  "diagnostic.entry.settings": "Settings",
  "diagnostic.entry.events": "System events",
  "diagnostic.entry.smart": "SMART capture",
  "diagnostic.entry.logs": "Activity log",
  "diagnostic.entry.other": "Other content",
  "diagnostic.redacted.serialNumber": "serial numbers",
  "diagnostic.redacted.computerName": "computer name",
  "diagnostic.redacted.userPaths": "username and personal paths",
  "settings.appearance.title": "Appearance",
  "settings.appearance.theme.light": "Light",
  "settings.appearance.theme.dark": "Dark",
  "settings.appearance.theme.system": "System",
  "settings.appearance.language.es": "Spanish",
  "settings.appearance.language.en": "English",
  "settings.appearance.useSystemAccent.label": "Use the Windows accent colour",
  "settings.appearance.useSystemAccent.hint": "Replaces the app's purple with the colour configured in Windows.",
  "settings.notifications.sound.label": "Notification sound",
  "settings.notifications.sound.hint": "Off by default.",
  "settings.cta.restoreDefaults": "Restore factory values",
  "settings.schedule.title": "Frequencies",
  "settings.schedule.rangeHint": "Between {min} and {max} s",
  "settings.schedule.metricsFast": "Temperature, activity, capacity and latency",
  "settings.schedule.smartFull": "Full SMART",
  "settings.schedule.events": "Windows events",
  "settings.schedule.discovery": "Arrival/removal detection",
  "settings.unit.seconds": "s",
  "settings.unit.days": "days",
  "settings.alerts.title": "Alert thresholds",
  "settings.alerts.tempPrecedence": "Only applies to disks without a temperature limit declared by the manufacturer; when the manufacturer declares one, that value wins.",
  "settings.alerts.tempWarn": "Warning temperature",
  "settings.alerts.tempCrit": "Critical temperature",
  "settings.alerts.capacityWarnPercent": "Free capacity warning",
  "settings.alerts.capacityCritPercent": "Free capacity critical",
  "settings.alerts.profile.title": "Profile",
  "settings.alerts.profile.hint": "Choose how much you want to be warned. Sets all twelve thresholds at once; you can fine-tune any of them below.",
  "settings.alerts.profile.cautious": "Cautious",
  "settings.alerts.profile.cautiousHint": "Warns earlier. More alerts.",
  "settings.alerts.profile.balanced": "Balanced",
  "settings.alerts.profile.balancedHint": "Recommended.",
  "settings.alerts.profile.quiet": "Only serious",
  "settings.alerts.profile.quietHint": "Critical conditions only. Fewer alerts.",
  "settings.alerts.profile.custom": "Custom (based on {base})",
  "settings.alerts.wearWarn": "Wear warning",
  "settings.alerts.wearCrit": "Wear critical",
  "settings.alerts.mediaErrorsWarn": "Media errors warning",
  "settings.alerts.mediaErrorsCrit": "Media errors critical",
  "settings.alerts.mediaErrorsHint": "Counter increase between two reads that is enough to warn.",
  "settings.alerts.driverRetryWarn": "Controller retries warning",
  "settings.alerts.driverRetryCrit": "Controller retries critical",
  "settings.alerts.driverRetryHint": "Pending: no rule watches controller retries yet.",
  "settings.retention.title": "Retention and history",
  "settings.retention.neverPurged": "Alerts, their critical occurrences, linked events and test runs are never deleted by retention.",
  "settings.retention.raw": "Raw samples",
  "settings.retention.fiveMinutes": "5-minute aggregates",
  "settings.retention.hourly": "Hourly aggregates",
  "settings.retention.freeSpaceGuard": "History writing warns below {warn} and stops below {halt}, without affecting monitoring or live alerts.",
  "settings.logging.title": "Activity log",
  "settings.collection.label": "Data collection active",
  "settings.collection.hint": "Turn it off to pause SMART, counter and event reads. A manual refresh stays available.",
  "settings.logging.verbose.label": "Verbose mode",
  "settings.logging.verbose.hint": "Turn it on to reproduce a failure with more detail.",
  "settings.logging.cta.openFolder": "Open log folder",
  "settings.lifecycle.title": "When closing the window",
  "settings.lifecycle.closeAction.label": "What the close button does",
  "settings.lifecycle.closeAction.minimize": "Minimize to tray",
  "settings.lifecycle.closeAction.minimizeHint": "The application keeps monitoring in the background.",
  "settings.lifecycle.closeAction.exit": "Exit the application",
  "settings.lifecycle.closeAction.exitHint": "Stops monitoring until reopened.",
  "settings.dangerZone.title": "Delete all data",
  "settings.dangerZone.desc": "Removes all stored history: inventory, metrics, alerts, events and test runs. The application ends up like a fresh install.",
  "settings.dangerZone.confirmPhraseLabel": "Type \"{phrase}\" to confirm",
  "settings.dangerZone.cta": "Delete all data",
  "settings.dangerZone.confirmTitle": "Delete all data?",
  "settings.dangerZone.confirmBody": "All history stored by the application will be removed: disk inventory, metrics, alerts, linked events, test runs and preferences. This action cannot be undone.",
  "settings.dangerZone.confirmImpact": "Once finished, the application ends up like a fresh install and shows the initial setup wizard again.",
  "settings.dangerZone.confirmLabel": "Delete all data",
  "settings.dangerZone.done": "All data has been deleted.",
  "nav.about": "About",
  "about.title": "About",
  "about.body": "By {author}. MIT license. Includes smartmontools (GPLv2) and the Instrument Sans typeface (SIL OFL 1.1); see the full third-party notices in the installer. Repository: https://github.com/danimardo/smartdisk-monitor",
  "about.cta.copy": "Copy information"
}
```


---

# 28. Tipografía empotrada

Fichero de origen: `src/design-system/fonts/README.md`

`tokens.css` declara la familia Instrument Sans sobre los ficheros de esta carpeta. La aplicación no
descarga tipografías: la especificación (§11) y el ADR-018 prohíben cualquier petición de red
durante el funcionamiento normal, y un equipo sin salida a Internet debe renderizar exactamente
igual que uno conectado.

### Ficheros

| Fichero | Subconjunto | Tamaño | SHA-256 |
|---|---|---|---|
| `InstrumentSans-latin.woff2` | latin | 30.092 B | `2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971` |
| `InstrumentSans-latin-ext.woff2` | latin-ext | 11.144 B | `c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2` |
| `OFL.txt` | — | 4.403 B | licencia, sin modificar |

Son ficheros **variables** en el eje `wght` (400–700), con el eje `wdth` fijado en 100. `tokens.css`
declara solo `400 600` a propósito: si alguien pidiera 700, el navegador lo limita a 600 en lugar de
sintetizar una negrita falsa. El peso máximo de v2 es 600 (`AGENTS.md` §2.bis).

### Por qué dos ficheros

El proyecto original publica la fuente subseteada, igual que la sirve Google Fonts. `latin` cubre
por completo el español y el inglés de la interfaz —vocales acentuadas, `ñ`, `ü`, `¿`, `¡`, `°`, `×`,
`·`, `•`— y es el único que se carga en uso normal. `latin-ext` solo se decodifica si aparece un
carácter de su rango, cosa que puede ocurrir con el texto original de un evento de Windows, que
llega en el idioma del sistema. Al ser ficheros locales, tener los dos no cuesta nada; el
`unicode-range` evita procesar el que no hace falta.

### Procedencia

- Proyecto: Instrument Sans, de Rodrigo Fuenzalida y Jordan Egstad.
- Repositorio: https://github.com/Instrument/instrument-sans
- Ficheros obtenidos de: `https://fonts.gstatic.com/s/instrumentsans/v4/…` (versión **v4** del
  catálogo de Google Fonts, que es quien publica los `.woff2` ya subseteados).
- `OFL.txt` obtenida de: https://github.com/google/fonts/blob/main/ofl/instrumentsans/OFL.txt
- Copyright 2022 The Instrument Sans Project Authors.
- Licencia: SIL Open Font License 1.1.

### Obligaciones

1. `OFL.txt` viaja con la aplicación, sin modificar. **Ya está en esta carpeta.**
2. La fuente está registrada en `THIRD_PARTY_NOTICES.md` con su versión y sus hashes.
3. No se renombra la familia: la OFL solo obliga a cambiar el nombre si se modifica el fichero, y
   aquí se redistribuye tal cual.
4. Si se actualiza la fuente, se actualizan los hashes de esta tabla y los del aviso de terceros.

### Verificación

```sh
sha256sum -c <<'EOF'
2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971 *InstrumentSans-latin.woff2
c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2 *InstrumentSans-latin-ext.woff2
EOF
```

### Comprobación visual

`tools/font-check.html` renderiza la fuente en ambos subconjuntos y los tres pesos. Necesita un
servidor, porque `file://` no carga `.woff2`:

```sh
python -m http.server 8731 --bind 127.0.0.1     # desde la raíz del repositorio
# abrir http://127.0.0.1:8731/tools/font-check.html
```

Verificado el 2026-09-04: los dos `@font-face` cargan (`document.fonts.check` a `true`), los
acentos españoles, `°`, `¿`, `¡`, `«»`, `×` y `·` renderizan con Instrument Sans, el subconjunto
`latin-ext` entra cuando toca, y los pesos 600 y 700 miden exactamente lo mismo: el clamp de
`tokens.css` impide la negrita sintética.

### Por qué falla la compilación sin estos ficheros

La compilación debe fallar si falta cualquiera de los dos `.woff2`: sin ellos la aplicación cae a
`Segoe UI`, la métrica cambia, los bocetos aprobados dejan de ser fieles y nadie se entera
(`docs/engineering-conventions.md` §3).


---

# 29. smartctl redistribuido

Fichero de origen: `third-party/smartmontools/README.md`

Fuente principal de datos SMART y NVMe de SmartDisk Monitor (ADR-005). Se ejecuta como **proceso
independiente**, nunca enlazado, lo que mantiene el código propio bajo licencia MIT
(véase *Licencia*, más abajo).

### Versión

| | |
|---|---|
| Versión | **smartmontools 7.5** |
| Fecha de publicación | 12 de mayo de 2025 (compilación del 30 de abril de 2025, r5714) |
| Origen | `smartmontools-7.5.win32-setup.exe` de las publicaciones oficiales del proyecto |
| Identificación que reporta | `smartctl 7.5 2025-04-30 r5714 [x86_64-w64-mingw32-w11-b26200]` |

El paquete oficial se llama `win32-setup` por motivos históricos, pero **contiene ambas
arquitecturas**: `bin/` es x64 y `bin32/` es x86. Aquí se redistribuye el de `bin/`, verificado como
PE de máquina `0x8664` (AMD64).

### Qué se incluye y por qué

| Fichero | Tamaño | Para qué |
|---|---|---|
| `bin/smartctl.exe` | 1.165.312 B | El programa. Lo único que ejecuta la aplicación |
| `bin/drivedb.h` | 267.943 B | **Base de datos de unidades.** Sin ella, los atributos específicos de cada fabricante se muestran como desconocidos en vez de con su nombre y su interpretación |
| `licenses/COPYING.txt` | | GPL versión 2, íntegra y sin modificar |
| `licenses/AUTHORS.txt`, `NEWS.txt` | | Avisos de autoría que la licencia obliga a conservar |
| `source/smartmontools-7.5.tar.gz` | 1.122.317 B | El código fuente correspondiente. Es lo que satisface la obligación de la GPLv2 |

#### Qué se deja fuera, deliberadamente

- `smartd.exe` y toda su configuración: es el demonio de vigilancia del propio smartmontools. La
  aplicación hace ese trabajo con su propio planificador, y arrancar un segundo vigilante sería
  duplicar la función y competir por el acceso a los dispositivos.
- `update-smart-drivedb.ps1`: actualiza `drivedb.h` **descargándola de Internet**. Incompatible con
  la promesa de cero comunicaciones de red (spec §11, ADR-007). La base de datos se actualiza
  cambiando de versión de smartmontools, no en caliente.
- `smartd_mailer.ps1`, `wtssendmsg.exe`, `runcmda.exe`, `runcmdu.exe`: utilidades de notificación de
  `smartd`. No se usan y ampliarían la superficie de la instalación sin motivo.
- Los binarios de 32 bits de `bin32/`: la plataforma es x64.

#### `smartctl.exe` frente a `smartctl-nc.exe`

El paquete oficial trae también `smartctl-nc.exe`, idéntico pero enlazado como aplicación de
ventanas para que no aparezca una consola al invocarlo. **Se usa `smartctl.exe`**, el canónico: el
parpadeo de consola se evita desde Rust lanzando el proceso con la bandera `CREATE_NO_WINDOW`
(`0x08000000`), que es la solución correcta y no depende de qué binario se empaquete.

### Verificación de integridad

Sumas MD5 comprobadas contra `doc/checksums64.txt` del propio paquete oficial y contra los ficheros
`.md5` publicados junto a la descarga:

| Fichero | MD5 | SHA-256 |
|---|---|---|
| `bin/smartctl.exe` | `c1d1d016a33b09014517636c61d30947` | `b5db94e5082c042be44994b7a4fa8f7b5c8e713b2ab1c9a560d8f7a7995ea27d` |
| `bin/drivedb.h` | | `dd39c6a520d38895da61923fe26fe7c9c5eb3f42325f2e4477a06ed7a61966d0` |
| `source/smartmontools-7.5.tar.gz` | `38c38b0b82db7fc4906cdd50d15a7931` | `690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e` |

La compilación debe verificar estos hashes antes de empaquetar. Un binario privilegiado que se
distribuye a terceros no se copia a ciegas.

### Licencia y obligaciones

`SPDX-License-Identifier: GPL-2.0-or-later`. Copyright (C) 2002-2011 Bruce Allen,
2008-2025 Christian Franke, 2000 Michael Cornwell y otros; véase `licenses/AUTHORS.txt`.

**La licencia propia no se ve afectada.** SmartDisk Monitor invoca `smartctl.exe` como proceso
separado, comunicándose por línea de órdenes y JSON por la salida estándar. No lo enlaza ni
incorpora su código, así que no hay obra derivada y el código propio sigue siendo MIT.

**La obligación que sí aplica** es la de la sección 3 de la GPLv2: quien recibe el binario tiene
derecho a recibir su código fuente correspondiente. Se cumple por la vía 3(a), acompañar el binario
del código: `source/smartmontools-7.5.tar.gz` viaja **dentro del instalador**, en
`licenses\smartmontools\`, de modo que el fuente acompaña siempre al binario, en la misma entrega y
sin depender de nada externo.

Se descartó la vía 3(b), la oferta escrita válida durante tres años, porque obliga a mantener
disponible el fuente y a atender solicitudes durante ese plazo. Un fichero de 1 MB dentro del
instalador cuesta menos y no caduca.

### Al actualizar de versión

1. Descargar el `win32-setup.exe` y el `.tar.gz` de la misma versión, con sus `.md5`.
2. Verificar las sumas publicadas antes de extraer nada.
3. Extraer `bin/smartctl.exe` y `bin/drivedb.h` (los de 64 bits, no los de `bin32/`).
4. Comprobar los hashes contra `doc/checksums64.txt` del paquete.
5. Sustituir también el tarball de `source/`: **debe ser exactamente el de la versión del binario**,
   o el requisito de "fuente correspondiente" deja de cumplirse.
6. Actualizar la tabla de hashes de este documento y la de `THIRD_PARTY_NOTICES.md`.
7. Repasar `NEWS.txt`: un cambio en el formato JSON obligaría a revisar los analizadores.

### Notas de comportamiento verificadas

Comprobado el 2026-09-04 sobre este mismo binario:

- `smartctl --scan-open --json` funciona **sin privilegios de administrador** y enumera los
  dispositivos con su tipo (`ata`, `nvme`).
- Leer datos de un dispositivo **sin elevación falla**, lo que confirma la premisa del ADR-004. Pero
  falla de una forma engañosa: `exit_status: 1` y el mensaje
  `"/Device/HarddiskN/Partition0: Unable to detect device type"`.

  **Ese mensaje no significa que el disco sea incompatible.** Si se tomara al pie de la letra, un
  equipo entero aparecería como "no compatible" cuando el problema es de privilegios. El colector
  debe distinguir los dos casos y, ante ese mensaje, comprobar primero si el proceso está elevado.


---

# 30. Licencia del código propio

Fichero de origen: `LICENSE`

```text
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
```


---

# 31. Avisos de terceros

Fichero de origen: `THIRD_PARTY_NOTICES.md`

This file will be completed from the exact dependency lockfiles and bundled artifacts before the first distributed build.

### smartmontools / smartctl

SmartDisk Monitor is designed to invoke `smartctl` as a separate executable.

- Project: https://www.smartmontools.org/
- Copyright (C) 2002-2011 Bruce Allen; 2008-2025 Christian Franke; 2000 Michael Cornwell; and others
  (see `third-party/smartmontools/licenses/AUTHORS.txt`)
- License: GNU General Public License, version 2 or later (`SPDX-License-Identifier: GPL-2.0-or-later`)
- Version: **smartmontools 7.5**, released 12 May 2025 (build r5714)
- Distribution status: **bundled**

| Bundled file | MD5 | SHA-256 |
|---|---|---|
| `bin/smartctl.exe` (x64) | `c1d1d016a33b09014517636c61d30947` | `b5db94e5082c042be44994b7a4fa8f7b5c8e713b2ab1c9a560d8f7a7995ea27d` |
| `bin/drivedb.h` | | `dd39c6a520d38895da61923fe26fe7c9c5eb3f42325f2e4477a06ed7a61966d0` |
| `source/smartmontools-7.5.tar.gz` | `38c38b0b82db7fc4906cdd50d15a7931` | `690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e` |

MD5 sums verified against `doc/checksums64.txt` inside the official package and against the `.md5`
files published alongside the download.

When a binary is added, this distribution must include the corresponding copyright and license text and satisfy the source-code obligations applicable to that binary. This notice does not replace those materials.

`smartctl` runs as a separate process, communicating over the command line and JSON on standard
output. It is never linked into the application, so no derivative work is created and the project's
own MIT licence is unaffected.

The GPLv2 section 3 source obligation does apply to the redistributed binary, and is satisfied via
**option 3(a)**: the corresponding source archive ships **inside the installer**, at
`licenses\smartmontools\smartmontools-7.5.tar.gz`, so the source always accompanies the binary in
the same delivery. Option 3(b), a written offer valid for three years, was rejected because it
requires keeping the source available and answering requests for that period; a 1 MB file inside a
140 MB installer costs less and does not expire.

The source archive version must always match the binary version, or the "corresponding source"
requirement is no longer met.

### Instrument Sans

The user interface embeds the Instrument Sans variable font. The application performs no network
requests, so the font is shipped as a local file rather than loaded from a font CDN (ADR-018).

- Project: https://github.com/Instrument/instrument-sans
- Authors: Rodrigo Fuenzalida, Jordan Egstad
- Copyright 2022 The Instrument Sans Project Authors
- License: SIL Open Font License 1.1
- Version: v4 of the Google Fonts catalogue, which publishes the subsetted `.woff2` files
- Distribution status: **bundled**

| Bundled file | SHA-256 |
|---|---|
| `src/design-system/fonts/InstrumentSans-latin.woff2` | `2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971` |
| `src/design-system/fonts/InstrumentSans-latin-ext.woff2` | `c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2` |

The unmodified `OFL.txt` ships alongside the font files in the same folder. The font is
redistributed under its original family name and is not modified, so the licence requires no name
change. Both files must also be copied into the installed application folder.

### Application dependencies

Rust and JavaScript dependency notices will be generated and reviewed from the locked dependency graph before release. No dependency is yet vendored at the specification stage.
