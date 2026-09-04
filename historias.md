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
| Cómo se escribe una pantalla | Reglas de interfaz (VINCULANTES) |
| Qué hay que probar, a qué nivel y cuándo | Estrategia integral de testing |

## Precedencia

Si dos documentos se contradicen, mandan en este orden:

1. **Reglas de alerta** sobre el resumen de alertas de la especificación.
2. **Contrato UI ↔ backend** sobre cualquier descripción informal de comandos.
3. **Reglas de interfaz** (`AGENTS.md`) sobre cualquier criterio visual escrito en otro sitio.
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
13. [Reglas de interfaz (VINCULANTES)](#13-reglas-de-interfaz-vinculantes) · `Design-system/AGENTS.md`
14. [Entrega del sistema de diseño](#14-entrega-del-sistema-de-diseño) · `Design-system/HANDOFF.md`
15. [Sistema de diseño: principios](#15-sistema-de-diseño-principios) · `Design-system/design-system/README.md`
16. [tokens.css — fuente única de verdad visual](#16-tokenscss-fuente-única-de-verdad-visual) · `Design-system/design-system/tokens.css`
17. [tailwind.config.cjs — mapeo de tokens](#17-tailwindconfigcjs-mapeo-de-tokens) · `Design-system/tailwind.config.cjs`
18. [design/types.ts — vocabulario de la UI](#18-designtypests-vocabulario-de-la-ui) · `Design-system/src/lib/design/types.ts`
19. [design/health.ts — estado → color, umbrales](#19-designhealthts-estado-color-umbrales) · `Design-system/src/lib/design/health.ts`
20. [design/format.ts — formato de presentación](#20-designformatts-formato-de-presentación) · `Design-system/src/lib/design/format.ts`
21. [design/theme.svelte.ts — tema](#21-designthemesveltets-tema) · `Design-system/src/lib/design/theme.svelte.ts`
22. [design/accent.ts — acento de Windows](#22-designaccentts-acento-de-windows) · `Design-system/src/lib/design/accent.ts`
23. [i18n/index.ts — idioma, formato y plurales](#23-i18nindexts-idioma-formato-y-plurales) · `Design-system/src/lib/i18n/index.ts`
24. [i18n/es.json](#24-i18nesjson) · `Design-system/src/lib/i18n/es.json`
25. [i18n/en.json](#25-i18nenjson) · `Design-system/src/lib/i18n/en.json`
26. [Tipografía empotrada](#26-tipografía-empotrada) · `Design-system/design-system/fonts/README.md`
27. [smartctl redistribuido](#27-smartctl-redistribuido) · `third-party/smartmontools/README.md`
28. [Licencia del código propio](#28-licencia-del-código-propio) · `LICENSE`
29. [Avisos de terceros](#29-avisos-de-terceros) · `THIRD_PARTY_NOTICES.md`


---

# 1. Presentación del proyecto

Fichero de origen: `README.md`

Aplicación local de supervisión de discos para Windows, desarrollada con Tauri 2, Rust, Svelte, TypeScript y Tailwind CSS.

Estado actual: **esqueleto de Fase 0 en marcha**. La especificación está cerrada y el proyecto
compila y arranca; los comandos del backend devuelven `not_implemented` hasta que se implemente cada
recopilador.

### Puesta en marcha

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
| `pnpm test` | Pruebas del frontend |
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

El sistema de diseño aprobado se encuentra en [`Design-system/`](Design-system/) y es vinculante para la implementación de la interfaz.

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
- [Entrega del sistema de diseño](Design-system/HANDOFF.md)
- [Reglas vinculantes de interfaz](Design-system/AGENTS.md)

### Identidad del proyecto

- Producto: SmartDisk Monitor
- Autor: Daniel Diez Mardomingo
- Repositorio previsto: https://github.com/danimardo/smartdisk-monitor
- Licencia del código propio: MIT
- Plataforma inicial: Windows x64


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
- La aplicación no se inicia automáticamente.
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
- Notificación nativa de Windows cuando la aplicación está minimizada.
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
  1360 × 880 acotado a la pantalla. El mínimo es bajo por una razón medida: el escalado de Windows
  no encoge el texto, encoge el espacio en píxeles CSS, y un portátil de 1920 × 1080 al 150 % solo
  deja 1280 × 672 de ventana. La interfaz debe ser correcta al 125 %, 150 % y 200 %.
- Por debajo de 1180 px de ancho la barra lateral se reduce a iconos, conservando el punto de estado
  de cada disco.
- El formato de números y fechas sigue al **idioma elegido en la aplicación**, no al de Windows,
  conservando la variante regional del sistema cuando comparten idioma.
- Sistema de diseño aprobado: **SmartDisk Monitor v2, material translúcido**, entregado en `Design-system/`.
- `Design-system/AGENTS.md` es vinculante para cualquier implementación de interfaz.
- `Design-system/design-system/tokens.css` es la fuente única de colores, tipografía, espaciado, radios, sombras, materiales y movimiento; no se permiten valores visuales literales en componentes.
- El acento de acciones y selección se hereda de Windows, con el azul de respaldo definido en los tokens. El acento nunca comunica salud.
- Verde, ámbar, rojo y gris se reservan respectivamente para correcto, advertencia, crítico y desconocido/no compatible/sin datos. El color siempre se acompaña de texto o iconografía accesible.
- Se usan exclusivamente tres niveles de material: chrome, tarjetas y overlays; no se apilan tarjetas ni se inventan niveles de desenfoque.
- Tipografía principal Instrument Sans con alternativas del sistema y peso máximo 600.
- Movimiento funcional y breve, respetando `prefers-reduced-motion`.
- Contraste mínimo AA, foco visible y navegación completa por teclado.
- Las preferencias de idioma y tema se guardan en SQLite mediante `settings`, nunca en `localStorage`.
- Panel general, detalle de disco, alertas y pruebas/diagnóstico siguen los bocetos aprobados de `Design-system/SmartDisk Monitor v2.dc.html`.
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
- Toda pantalla debe cumplir la definición de terminado de `Design-system/AGENTS.md` en temas claro y oscuro y en el tamaño mínimo de ventana.
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

#### US-002 — Asistente inicial (P0)

Como usuario quiero configurar la aplicación mediante un asistente para empezar a monitorizar sin conocer SMART.

Criterios de aceptación:

- Aparece cuando no existe una configuración inicial completa.
- Enumera los discos detectados y selecciona inicialmente todos los compatibles.
- Permite excluir discos y asignar alias.
- Explica los estados no compatible y desconocido.
- Las elecciones se conservan tras reiniciar.

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

### Criterio de priorización

- P0: necesario para considerar utilizable la versión 1.0.
- P1: debe entrar en 1.0 salvo riesgo técnico demostrado.
- P2: candidato a versiones posteriores.

### Fase 0 — Validación técnica

Objetivo: reducir riesgos antes de construir la interfaz completa.

- Crear esqueleto Tauri 2 + SvelteKit (`adapter-static`, SSR off) + TypeScript + Tailwind (ADR-014).
- Añadir la tipografía Instrument Sans a `design-system/fonts/` con su licencia (ADR-018) y hacer que
  la compilación falle si falta.
- Integrar `Design-system/design-system/tokens.css`, la configuración Tailwind, los módulos de diseño, i18n y el catálogo Svelte entregado.
- Validar los componentes entregados con Svelte 5 y el toolchain definitivo antes de modificarlos.
- Montar un shell navegable con `AppShell`, `Sidebar` y `Toolbar` siguiendo el boceto v2 aprobado.
- Verificar temas claro/oscuro, acento de Windows, fallback sin translucidez y movimiento reducido.
- Probar elevación UAC y empaquetado x64.
- **Verificar la instalación de WebView2 en un Windows Server 2019 limpio y sin salida a Internet**.
  El modo ya está decidido (instalador sin conexión, ADR-020); lo que falta es comprobar que la
  instalación silenciosa funciona en una máquina real.
- **Comprobar la entrega de notificaciones toast desde un proceso elevado** con AUMID registrado. Si
  Windows no las entrega, hay que sustituirlas por una ventana propia con el componente `Toast`.
- ~~Medir la codificación de la salida de `chkdsk`~~ **hecho**: es CP1252, no CP850, y las
  herramientas de Windows no coinciden entre sí. Queda **implementar y probar** la detección de
  `open-questions.md` §Q con volcados reales como fixtures.
- **Validar `accessibleAccent()`** contra los acentos de Windows, empezando por los claros.
- ~~Probar bloqueo de instancia única y ACL de la carpeta de `ProgramData`~~ **hecho**: eran dos
  problemas distintos. La instancia única va con el plugin oficial (ADR-025); queda una
  comprobación de humo manual, que exige UAC, dentro de US-060. Y `%ProgramData%` **no** restringe
  la escritura a administradores: un usuario sin privilegios se apropia de la carpeta
  pre-creándola, y restablecer la ACL sin tomar la propiedad no lo arregla (ADR-026,
  `open-questions.md` §R).
- Validar `smartctl --scan-open --json` en NVMe, SATA y USB disponibles.
- Interpretar correctamente los bits del código de salida de smartctl.
- Contrastar en **Windows Server** la lista de eventos de `alert-rules.md` §3, verificada hasta ahora
  solo en Windows 11 cliente: faltan RAID por hardware y Storage Spaces en producción.
- Probar lectura de eventos y marcadores (bookmark de canal, no RecordId).
- Probar contadores de rendimiento y mapeo disco-volumen.
- Validar SQLite WAL en `ProgramData`.
- Prototipar systray y cierre hacia bandeja.
- Documentar cumplimiento de redistribución de smartmontools: fijar la versión exacta del binario y
  elegir cómo se satisface la obligación de código fuente de la GPLv2.
- Medir la interfaz con veinte discos y cinco mil eventos, y la escala tipográfica al 125 % y 150 %.

Riesgos abiertos y su criterio de cierre: [`open-questions.md`](docs/open-questions.md) §I.

Salida: informe de viabilidad y fixtures anonimizados.

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

- Instalador para todos los usuarios.
- Desinstalación que conserva datos.
- Aviso documentado de SmartScreen por falta de firma.
- Licencia MIT, terceros y atribuciones.
- Nombre y versión dinámicos.
- Release manual en GitHub.
- Validación completa contra la definición de terminado de `Design-system/AGENTS.md`.

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

El paquete ubicado en `Design-system/` forma parte de la arquitectura del producto, no es una referencia opcional:

- `Design-system/AGENTS.md`: reglas vinculantes de implementación y definición de terminado.
- `Design-system/HANDOFF.md`: contrato de integración y comandos Tauri esperados por la UI.
- `Design-system/design-system/tokens.css`: fuente única de verdad visual, importada una sola vez al arrancar.
- `Design-system/design-system/tokens.json`: representación de los mismos tokens para herramientas.
- `Design-system/tailwind.config.cjs`: mapeo permitido de tokens a utilidades.
- `Design-system/src/lib/components/`: catálogo cerrado de componentes Svelte.
- `Design-system/src/lib/design/`: tipos de presentación, formato, salud, tema y acento de Windows.
- `Design-system/src/lib/i18n/`: diccionarios y selección de idioma.
- `Design-system/SmartDisk Monitor v2.dc.html`: referencia visual aprobada para las cuatro pantallas diseñadas.

La integración copiará estos recursos al esqueleto Tauri conservando su estructura lógica. La UI se montará con `AppShell`, `Sidebar` y `Toolbar`; ninguna pantalla creará su propio chrome. Los componentes nuevos solo se admitirán cuando el patrón aparezca en al menos tres pantallas y no pueda componerse con el catálogo existente.

La comunicación UI-backend se definirá con DTO tipados coherentes con `Design-system/src/lib/design/types.ts`. Los valores opcionales permanecerán como `null`, las series conservarán huecos explícitos y toda métrica llevará procedencia y antigüedad cuando estén disponibles.

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

#### `event_cursors`

- Canal/proveedor y **bookmark** del registro de eventos, no un `RecordId` suelto: al limpiar un
  canal los identificadores se reinician, y un cursor numérico se quedaría por delante de los
  eventos nuevos y dejaría de importarlos sin dar ningún error.
- Evita duplicar eventos entre sesiones. La identidad de un evento es `(canal, RecordId)`, no su
  fecha, de modo que un cambio del reloj del sistema tampoco produce duplicados.

#### `schema_migrations`

- Versión, fecha y checksum de cada migración aplicada.

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

Los campos no disponibles se omiten; no se almacenan como cero.

### 4. Retención

- Un trabajo diario compacta muestras antiguas dentro de una transacción.
- La agregación conserva mínimo, máximo, promedio, primera y última lectura, además de incrementos de contadores.
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
§B, `Design-system/src/lib/design/health.ts`.

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
| `smart.media_errors` | smartctl | `media_errors_total` aumenta respecto a la lectura anterior | crítico inmediato | no aumenta durante 24 h | 1 h | — |
| `smart.error_log` | smartctl | `error_log_entries_total` aumenta | advertencia; **crítico** si aumenta en 3 ciclos seguidos | no aumenta durante 24 h | 1 h | — |
| `smart.spare_below_threshold` | smartctl | `available_spare_percent < available_spare_threshold_percent` | crítico | por encima del umbral + 2 puntos durante 3 ciclos | 6 h | — |
| `smart.wear_high` | smartctl | `percentage_used ≥ 90` | advertencia; **crítico** en `≥ 100` | no se resuelve sola: el desgaste no baja. Se archiva a mano | 7 días | — |
| `temp.above_vendor_limit` | smartctl | `temperature_celsius > vendorTempLimitC` durante 3 ciclos | advertencia | ≤ límite − 3 °C durante 3 ciclos | 30 min | id. de sensor |
| `temp.above_vendor_critical` | smartctl | `temperature_celsius ≥ vendorTempCriticalC` | crítico inmediato | ≤ crítico − 5 °C durante 3 ciclos | 15 min | id. de sensor |
| `temp.above_configured_warn` | smartctl | sin límite del fabricante: `> 70 °C` durante 3 ciclos | advertencia | ≤ 67 °C durante 3 ciclos | 30 min | id. de sensor |
| `temp.above_configured_crit` | smartctl | sin límite del fabricante: `≥ 80 °C` | crítico inmediato | ≤ 75 °C durante 3 ciclos | 15 min | id. de sensor |
| `capacity.low` | sistema de archivos | `capacityState()` da `warn` | advertencia | vuelve a `ok` **y** se mantiene 3 ciclos | solo al cambiar de nivel | `volume_guid` |
| `capacity.critical` | sistema de archivos | `capacityState()` da `crit` | crítico | sube a `warn` u `ok` y se mantiene 3 ciclos | solo al cambiar de nivel | `volume_guid` |
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
| `Microsoft-Windows-Ntfs` | 140 | Advertencia | No se pudo vaciar el registro de transacción **(obs., 173)** | `events.delayed_write` | advertencia |
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

Cada regla necesita cuatro claves i18n en `es.json` y `en.json`:

```
alert.<rule_key>.title      Titular corto, sin jerga.
alert.<rule_key>.summary    Una frase que explique qué significa y por qué importa.
alert.<rule_key>.fact.*     Etiquetas de la rejilla de hechos del detalle.
alert.<rule_key>.action     Qué puede hacer el usuario, si hay algo que hacer.
```

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
| `test.busy` | ya hay una prueba en ese disco | no |
| `test.unsupported` | el dispositivo no admite esa prueba | no |
| `test.insufficient_space` | no cabe el archivo con la reserva | no |
| `db.locked` | SQLite ocupado más allá del tiempo de espera | sí |
| `db.migration_failed` | migración fallida; se ha restaurado la copia previa | no |
| `path.invalid` | ruta fuera de las carpetas permitidas | no |
| `export.write_failed` | no se pudo escribir el destino | sí |
| `settings.out_of_range` | valor fuera de los límites de `open-questions.md` D.1 | no |

---

### 2. Tipos compartidos

Los que ya viven en `Design-system/src/lib/design/types.ts` no se repiten aquí: `HealthState`,
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
  useSystemAccent: boolean;
}

invoke<WindowsAccent>("get_system_accent_color")   // error si el usuario lo tiene desactivado
interface WindowsAccent {
  hex: string;        // #RRGGBB. OJO: el registro lo guarda en ABGR, no en RGB (open-questions.md O.7)
  palette?: string[]; // los 7 tonos de AccentPalette, del más claro al más oscuro
}

invoke<Settings>("get_settings")
invoke<void>("set_setting", { key: string, value: unknown })   // valida rango; AppError si no cabe
invoke<Settings>("reset_settings", { scope: "all" | "alerts" | "schedule" | "retention" })
```

`Settings` es un objeto tipado, no un diccionario libre. Sus límites están en `open-questions.md`
D.1 y los valida el backend: la UI puede confiar en que un valor guardado es un valor legal.

#### 3.2 Inventario

```ts
invoke<DeviceListResponse>("get_devices")
interface DeviceListResponse {
  devices: DiskSummary[];
  excluded: DiskSummary[];        // desactivados por el usuario; US-011 exige mostrarlos aparte
  sources: SourceHealth[];        // estado de cada recopilador
  paused: boolean;
  pausedSince: string | null;
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

---

### 4. Eventos emitidos por el backend

La UI **no hace sondeo**. El backend empuja (ADR-015). Cada carga útil lleva `emittedAt` para poder
descartar mensajes fuera de orden.

| Evento | Carga útil | Cuándo |
|---|---|---|
| `metrics:updated` | `{ emittedAt, devices: DiskSummary[], sources: SourceHealth[] }` | al cerrar cada ciclo de recopilación |
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
| Svelte | 5 con runes | `AGENTS.md` §1 |
| TypeScript | 5.x, `strict: true` | sin `any` implícito, sin `@ts-ignore` sin justificar |
| Tailwind | 3.x | solo utilidades mapeadas desde tokens |
| SQLite | vía `rusqlite` con `bundled` | evita depender de la DLL del sistema |

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
  Design-system/                Paquete de entrega original, congelado como referencia

  AGENTS.md                     Instrucciones para agentes de IA: fuente canónica
  CLAUDE.md                     Importa AGENTS.md y añade lo específico de Claude Code
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
| Unitarias TS | `format`, `health`, `accent`, `i18n` | `vitest` |
| Componentes | estados vacío, cargando, no compatible, error y dato obsoleto de cada componente | `vitest` + testing-library |
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
  la de `AGENTS.md` §8.
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
| Unit TS | `vitest` + `@vitest/coverage-v8` | En uso: 160 pruebas, umbral 70 % |
| Unit Rust | `cargo test` | En uso: 11 pruebas |
| Componentes | `@testing-library/svelte` | **Instalado y sin usar: cero tests de componente** |
| Validación de contrato | `zod` | En uso en la frontera IPC (§XI) |
| Verificadores propios | 5 scripts en `scripts/` | En uso: recursos, tokens, i18n, fronteras |
| E2E | — | **No existe** |
| Visual | — | **No existe** |
| Mutation | — | **No existe** |

Duración medida: `vitest` 22 s, `cargo test` 2 s. Ese es el presupuesto que hay que preservar.

#### Deuda identificada

1. **`@testing-library/svelte` instalado sin un solo test.** Se resuelve en el lote L1.
2. **Ningún test de componente ni E2E**, pese a que el sistema de diseño es vinculante y su
   incumplimiento es un defecto de producto, no estético.

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

**Vitest Browser Mode con proveedor Playwright, en Chromium.** Se retira `@testing-library/svelte`
si el modo navegador lo hace redundante; no se mantienen dos soluciones equivalentes.

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

| Configuración | Entorno | Qué incluye | Duración |
|---|---|---|---|
| `vitest.config.ts` | Node | `src/lib/**/*.test.ts` | ~22 s |
| `vitest.browser.config.ts` | Chromium | `src/lib/**/*.svelte.test.ts` | más lenta |

Stryker apunta a la de Node y por tanto nunca abre un navegador, que era el riesgo real: mutar
código exige lanzar la suite cientos de veces.

```text
src/lib/**/*.test.ts        unit, junto al código (ya en uso)
src/lib/**/*.svelte.test.ts componentes, en navegador
tests/integration/          frontera IPC, SQLite, colectores
tests/fixtures/             volcados anonimizados de smartctl, eventos, chkdsk
e2e/ui/                     Playwright + mockIPC
e2e/ui/visual.spec.ts       regresión visual
e2e/ui/a11y.spec.ts         accesibilidad
e2e/app/                    WebdriverIO + tauri-driver
src-tauri/src/**            tests en módulo `#[cfg(test)]` (ya en uso)
src-tauri/tests/            integración de Rust
```

Scripts a añadir, sin romper los existentes:

```text
test:unit          las pruebas de Node actuales
test:component     Vitest Browser Mode
test:integration   frontera IPC y persistencia
test:e2e           Playwright, plano de interfaz
test:e2e:smoke     solo el smoke
test:e2e:ui        modo interactivo de Playwright
test:e2e:visual    regresión visual
test:e2e:app       WebdriverIO contra el ejecutable
test:a11y          accesibilidad
test:mutation      Stryker sobre el dominio
test:lote          nivel 2 completo
```

**Los comandos reales se documentan al crearlos**, no antes: no se prometen scripts que no existan.

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

```text
T-TEST-001  Documentar comandos reales y medir la línea base de cada suite
T-TEST-002  Definir la selección de pruebas afectadas por tipo de cambio

T-UNIT-001  Fixtures anonimizados: smartctl (ATA, NVMe, USB, RAID, VM)
T-UNIT-002  Fixtures de eventos de Windows y volcados de chkdsk/fsutil
T-UNIT-003  Reloj inyectable y control de aleatoriedad

T-COMP-001  Configurar Vitest Browser Mode; retirar testing-library si es redundante
T-COMP-002  Probar los cinco estados obligatorios del catálogo
T-COMP-003  Verificar contraste y material en ambos temas con getComputedStyle

T-INT-001   smartctl falso: timeouts, códigos con bits, JSON corrupto
T-INT-002   SQLite temporal: migraciones, restricciones, retención
T-INT-003   Contrato Zod ↔ serde con caso de prueba compartido

T-ACC-001   Trazabilidad criterio → prueba de las historias P0

T-PLAY-001  Configurar Playwright contra preview con mockIPC
T-PLAY-002  Instalar msedgedriver y configurar tauri-driver
T-PLAY-003  Smoke de interfaz
T-PLAY-004  Smoke de aplicación real
T-PLAY-005  Captura de errores de consola con allowlist documentada
T-PLAY-006  E2E de los flujos críticos con navegador

T-A11Y-001  axe-core en ambos temas
T-A11Y-002  Verificaciones propias: contraste sobre material, foco no tapado
T-VIS-001   Doce capturas controladas en Chromium

T-QUAL-001  Cobertura de Rust en CI
T-QUAL-002  Detección de pruebas inestables

T-MUT-001   Piloto de Stryker sobre health.ts y línea base
T-MUT-002   cargo-mutants sobre el dominio cuando exista
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
| 1 | ¿Cuánto tarda de verdad la suite de componentes en navegador? | Al implementar `T-COMP-001` |
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

Estado: aceptada.

La interfaz utilizará el paquete `Design-system/`, versión v2 de material translúcido. `Design-system/AGENTS.md` constituye la norma vinculante y `tokens.css` la fuente única de verdad visual.

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

Estado: aceptada.

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


---

# 12. Cuestiones abiertas y mediciones

Fichero de origen: `docs/open-questions.md`

Registro de todo lo que la especificación dejaba a interpretación, con el valor que se ha adoptado.
Nació de la revisión cruzada de `docs/` contra `Design-system/` previa a la implementación.

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

#### E.2 · Interacción de la gráfica · `DECIDIDO`

Cursor de lectura con ratón (el punto más cercano en tiempo) y con teclado (flechas, `Inicio`,
`Fin`, `Esc`), que muestra hora y valor en el pie. Sin zoom ni selección por arrastre en la v1.0:
el `SegmentedControl` de intervalo cubre la necesidad y evita un patrón nuevo. Implementado.

#### E.3 · Retención mínima frente a US-022 · `DECIDIDO`

US-022 promete "al menos 30 días de historial": se cumple con los agregados de 5 minutos, no con las
muestras crudas (7 días). La historia se reformula para decirlo explícitamente y no dar a entender
que habrá 30 días de detalle.

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
| I.5 | `smartctl` tras controladoras RAID y puentes USB | Qué cascada de `-d` (`sat`, `nvme`, `sntjmicron`, `csmi`) merece la pena antes de declarar "no compatible" | Se documenta la limitación por modelo de puente | `ABIERTO` — necesita hardware: **US-010** |
| I.6 | Instancia única y ACL de `ProgramData` | ~~Pendiente~~ **Resuelto**: eran dos problemas. La instancia única exige comunicar procesos, no solo detectarlos (ADR-025). Y `ProgramData` **no** restringe la escritura a administradores: un usuario sin privilegios se apropia de la carpeta pre-creándola (ADR-026). Véase §R | — | `DECIDIDO` |
| I.7 | Rendimiento de la interfaz con 20 discos y 5.000 eventos | Que la lista virtualizada y el panel aguantan sin bloqueo perceptible | Se recorta la densidad del panel o se pagina | `ABIERTO` — necesita la lista virtualizada: **US-021** |

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
| J.8 | Tamaño de ventana | Mínimo técnico 1024 × 560, objetivo de diseño 1280 × 720, predeterminada 1360 × 880. Medido en §L.2 |
| J.9 | Acerca de | Diálogo modal sobre la pantalla actual, no sección de la `Sidebar` |
| J.10 | Eventos en la navegación | Sección propia en la `Sidebar`, con filtro preaplicado al entrar desde el detalle de un disco |
| J.11 | Plurales en i18n | Función `tp()` con `Intl.PluralRules`; claves `<clave>.one` / `<clave>.other` |
| J.12 | Persistencia de tema e idioma | `theme.set()` e `i18n.set()` devuelven la clave a guardar, pero **no** persisten: el llamante debe invocar `set_setting`. Es fácil de olvidar; conviene un envoltorio que lo haga |

---

### K. Pendiente de decisión

Cerradas desde la última revisión:

- **K.1** (tipografía empotrada), 2026-09-04 — los dos `.woff2` de Instrument Sans v4 y su `OFL.txt`
  están en `Design-system/design-system/fonts/`, declarados en `tokens.css` con `unicode-range` y
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


| # | Cuestión | Por qué no se ha decidido |
|---|---|---|

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

# 13. Reglas de interfaz (VINCULANTES)

Fichero de origen: `Design-system/AGENTS.md`

Este documento es **vinculante** para cualquier agente (humano o IA) que escriba interfaz en este
repositorio. Describe cómo construir pantallas con el sistema de diseño aprobado: **v2, material
translúcido** (evolución de la dirección 1b). Si algo no está aquí, no lo inventes: pregunta o propón
una extensión del sistema.

Referencias funcionales: `docs/product-specification.md`, `docs/user-stories.md`, `docs/data-model.md`.
Boceto aprobado (v2, cuatro pantallas y ambos temas): `SmartDisk Monitor v2.dc.html`.
Guía visual de tokens y componentes: `Sistema de diseno SmartDisk.dc.html` (pendiente de actualizar a v2).
El boceto original de la dirección 1b queda como referencia histórica en `Bocetos SmartDisk Monitor.dc.html`.

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
design-system/tokens.css     ← fuente única de verdad (importar una sola vez en el arranque)
design-system/tokens.json    ← misma información, legible por herramientas
tailwind.config.cjs          ← mapeo de tokens a utilidades
```

- **Prohibido** escribir un color, radio, sombra o tamaño de fuente literal en un componente.
  Usa la utilidad Tailwind (`bg-surface`, `text-fg-dim`, `rounded-xl`, `shadow-card`) o `var(--sdm-*)`.
- El tema se conmuta con `document.documentElement.dataset.theme = "light" | "dark"`; lo gestiona
  `$lib/design/theme.svelte.ts`. **Todo componente debe verse correcto en ambos temas sin condicionales.**
- Preferencia de tema y de idioma se persisten en la tabla `settings`, no en `localStorage`.
- **El acento lo hereda de Windows.** `applySystemAccent()` (`$lib/design/accent.ts`) sobreescribe
  los tokens de acento al arrancar. Nunca codifiques el azul: el respaldo ya vive en `tokens.css`.
  El acento **no** comunica salud.
- **Hay dos tokens de acento y no son intercambiables:**

  | Token | Uso | Contra qué se mide su contraste |
  |---|---|---|
  | `--sdm-accent` | **fondo**: botón primario, relleno de selección | contra `--sdm-on-accent` |
  | `--sdm-accent-fg` | **texto e iconos** sobre el material: enlaces, etiqueta seleccionada, serie principal de la gráfica | contra la superficie del tema |

  Un color no puede cumplir las dos cosas: el azul `#0078d4` que Windows trae de fábrica da 4,31:1
  como texto sobre el material claro, **por debajo de AA**. Barriendo el espacio sRGB completo, el
  65 % de los acentos posibles son ilegibles como texto en tema claro y el 50 % en oscuro
  (`tools/accent-check.py`, `open-questions.md` §O).

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
- La escala tipográfica **no se toca sin volver a medir**. Parece pequeña sobre el papel y no lo es:
  Instrument Sans tiene una altura de x de 0,5175 em frente a los 0,50 de Segoe UI, así que el cuerpo
  denso de 12,5 px equivale ópticamente a Segoe UI 12,9 px, por encima de los 12 px (9 pt) que
  Windows usa para el texto de interfaz. Medido, no estimado (`open-questions.md` K.4).
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
| `Card` | contenedor de toda información | radio xl + `shadow-card`; no anides sombras |
| `Button` | acciones | **una sola** `variant="primary"` por pantalla; `disabledReason` siempre que esté deshabilitado |
| `StatusPill` / `StatusDot` | estado de salud | requieren `label`; el color nunca es el único portador de significado |
| `MetricCard` | cifra destacada + procedencia | `value={null}` ⇒ "No disponible", **compuesto como texto en `text-base`, no como cifra**: a 27 px no cabe en ninguna celda realista. Es un bloque interno (`bg-glass-3` + `rounded-inner`), nunca material sobre material |
| `DataRow` | contador SMART etiqueta/valor/delta | color en el delta solo si significa algo |
| `CapacityBar` | ocupación de volumen | el color lo decide `capacityState()`, no el llamante |
| `ProgressBar` | operación en curso | siempre con leyenda y tiempo restante |
| `Sidebar` | navegación principal + lista de discos | material de chrome; la selección se marca con material elevado y punto de acento; navega con `<a href>`, **nunca** con callback |
| `Toolbar` | barra de herramientas unificada | título y subtítulo de pantalla, controles contextuales, estado global y acción primaria |
| `SegmentedControl` | intervalos 24 h / 7 d / 30 d / personalizado | |
| `DiskCard` | tarjeta de disco del panel | recibe `href`; sin él se renderiza como bloque no interactivo |
| `HealthDonut` | reparto de estados del equipo | acompañar de leyenda numérica |
| `AlertCard` | grupo de alertas en lista | contador `×N`; claves técnicas solo en el detalle |
| `EventRow` | evento de Windows | etiqueta "asociación inferida" cuando `mappingConfidence !== "exact"` |
| `TimeSeriesChart` | gráficas históricas | huecos como huecos; umbral del fabricante discontinuo |
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

### 4. Reglas de composición de pantalla

0. **Medidas de ventana.** Hay tres números y no significan lo mismo:

   | | Valor | Para qué |
   |---|---|---|
   | Mínimo técnico | **1024 × 560** | `minWidth`/`minHeight` de `tauri.conf.json`. Nada puede romperse aquí |
   | Objetivo de diseño | **1280 × 720** | El tamaño contra el que se compone y se revisa |
   | Predeterminado | **1360 × 880** | Acotado a lo que quepa en la pantalla del usuario |

   El mínimo técnico no es un capricho: el escalado de Windows **no encoge el texto, encoge el
   espacio disponible en píxeles CSS**. Un portátil de 1920 × 1080 al 150 % deja una ventana máxima
   de 1280 × 672, y un 1366 × 768 al 125 % deja 1092 × 566. Con un mínimo de 720 de alto, en esas dos
   configuraciones la ventana no cabría en la pantalla. Medido, no estimado (`open-questions.md` K.4).

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
- Navegación completa por teclado: pestañas con `role="tablist"`, diálogos con `role="dialog" aria-modal` y foco atrapado.
- Toda gráfica y todo anillo llevan `role="img"` con `aria-label` que resume el dato, y una lectura textual equivalente cerca.
- `prefers-reduced-motion` respetado globalmente; no añadas animaciones decorativas.

### 7. Pantallas y su composición aprobada

1. **Panel general** — `HealthDonut` + leyenda + tarjeta de atención a la izquierda; rejilla 2×2 de `DiskCard`;
   tarjeta "Sucesos recientes" con `EventRow` al pie. La lista de discos vive además en la `Sidebar`.
2. **Detalle de disco** — cabecera con alias y estado; el `SegmentedControl` de intervalo va en la `Toolbar`; fila de 4 `MetricCard`;
   `TimeSeriesChart` de temperatura (2/3) + panel de contadores con `DataRow` (1/3) y acciones al pie.
3. **Alertas** — lista de `AlertCard` (columna fija ~470 px) + detalle: severidad, titular, explicación humana,
   rejilla de hechos, acciones (Reconocer / Silenciar / Archivar) y cronología de ocurrencias.
4. **Pruebas y diagnóstico** — tres tarjetas de prueba; tarjeta de ejecución en curso con `ProgressBar` y
   cinco métricas; aviso ámbar de parada automática; historial de `test_runs`.
5. **Informes**, **Ajustes** y **asistente inicial** (US-002) están pendientes de diseño: compón con este mismo
   catálogo y pide revisión antes de introducir patrones nuevos.

#### Comportamiento con muchos discos

El boceto v2 está dibujado con cuatro discos, pero Windows Server entra en el alcance y un equipo
puede tener veinte o más. Reglas obligatorias, no opcionales:

- La lista de discos de la `Sidebar` tiene su propio `overflow-y: auto`; la navegación principal y el
  estado global **nunca** hacen scroll con ella.
- El panel general pasa de rejilla fija 2×2 a `repeat(auto-fill, minmax(460px, 1fr))` (véase §4.0.bis:
  460 es el ancho por debajo del cual las cuatro métricas dejan de caber).
- A partir de **12 discos monitorizados**, `DiskCard` usa su variante compacta (una sola fila de
  métricas, sin gráfica en miniatura) y el panel muestra primero los que no están en `ok`.
- `HealthDonut` cuenta solo los discos monitorizados. Los excluidos por el usuario no aparecen en el
  reparto ni en el recuento; se listan aparte, como exige US-011.

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

# 14. Entrega del sistema de diseño

Fichero de origen: `Design-system/HANDOFF.md`

Contiene el sistema de diseño aprobado (**v2, material translúcido**) y el catálogo de componentes
Svelte listos para montar la aplicación Tauri. No es un proyecto ejecutable: son los archivos de UI
para integrar en el esqueleto Tauri 2 + Svelte + TypeScript + Tailwind.

### Qué hay dentro

```
AGENTS.md                        Reglas VINCULANTES de UI. Leer antes de escribir una sola pantalla.
HANDOFF.md                       Este archivo.
tailwind.config.cjs              Tokens → utilidades Tailwind.

design-system/
  tokens.css                     Fuente única de verdad: variables --sdm-*, temas claro/oscuro,
                                 utilidades de material y base. Importar UNA vez al arrancar.
  fonts/                         Instrument Sans variable, empotrada. La app NO descarga tipografías
                                 (spec §11). Falta el .woff2: véase fonts/README.md.
  tokens.json                    Los mismos valores, legibles por herramientas.
  README.md                      Principios, anatomía y arranque.

src/lib/design/
  types.ts                       Vocabulario de UI alineado con docs/data-model.md.
  format.ts                      Formateo de presentación. Todo dato ausente → "No disponible".
  health.ts                      Único mapa estado→color + umbrales de capacidad de la spec.
  theme.svelte.ts                Preferencia claro/oscuro/sistema → data-theme en <html>.
  accent.ts                      Hereda el color de acento de Windows sobre los tokens de acento.

src/lib/i18n/
  index.ts, es.json, en.json     i18n mínimo. Idioma inicial del sistema (es-* → es, resto → en).

src/lib/components/              Catálogo cerrado (25 componentes). Importar del barrel index.ts.
                                 Otros 4 quedan autorizados y por construir: véase AGENTS.md §3.

*.dc.html                        Bocetos navegables (abrir en el navegador):
  SmartDisk Monitor v2.dc.html      ← APROBADO: 4 pantallas, claro y oscuro, diálogo incluido.
  Sistema de diseno SmartDisk.dc.html  Guía visual de tokens (estilo v1, pendiente de refresco).
  Bocetos SmartDisk Monitor.dc.html    Exploración inicial 1a/1b, referencia histórica.
```

### Integración en 5 pasos

La base es **SvelteKit con `adapter-static` y SSR desactivado** (ADR-014). Rutas de destino exactas
en `docs/engineering-conventions.md`; resumen:

| Origen | Destino en el proyecto |
|---|---|
| `design-system/` (con `fonts/`) | `src/design-system/` |
| `src/lib/` | `src/lib/` |
| `tailwind.config.cjs` | raíz del proyecto |

1. Copia los tres bloques de la tabla. `$lib` ya apunta a `src/lib` en SvelteKit: no toques el alias.
2. Deja `tokens.css` como **única** importación de CSS global, en `src/routes/+layout.svelte`.
3. En el arranque (`+layout.svelte`, antes del primer render):

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

4. Monta la app con `AppShell` + `Sidebar` + `Toolbar`; ninguna pantalla monta su propio chrome.
5. Suscríbete a los eventos de la tabla de abajo. **No hagas sondeo con `setInterval`**: el backend
   empuja (ADR-015).
6. Lee `AGENTS.md` y su checklist de "terminado" antes de cerrar cada pantalla.

### Comandos Tauri que la UI espera

**El contrato exacto y normativo vive en `docs/ui-contract.md`**: firmas, DTO, eventos de
actualización en vivo y forma de los errores. Lo de abajo es solo el índice. La UI **solo** llama
comandos enumerados.

| Comando | Devuelve |
|---|---|
| `get_appearance_settings` | tema e idioma persistidos en `settings` |
| `set_setting` | guarda una clave tipada de `settings` |
| `get_system_accent_color` | `{ hex }` del acento de Windows (o error si está desactivado) |
| `get_devices` / `get_device_detail` | inventario y detalle con procedencia por métrica |
| `get_metric_series` | serie temporal con huecos explícitos (`v: null`), no interpolados |
| `get_alert_groups` / `acknowledge_alert` / `mute_alert` / `archive_alert` | ciclo de vida de alertas |
| `get_system_events` | eventos con `mappingConfidence` |
| `start_benchmark` / `cancel_test` / `run_chkdsk_scan` / `run_smart_short_test` | pruebas manuales |
| `get_test_runs` | historial de `test_runs` |
| `export_report` / `create_diagnostic_zip` | informes y ZIP anonimizado |
| `get_settings` / `set_setting` / `reset_settings` | pantalla de Ajustes (US-070) |
| `pause_monitoring` / `resume_monitoring` | pausa desde la bandeja (US-032) |
| `delete_all_data` | borrado explícito y confirmado del historial (US-073) |

Además el backend **emite eventos**; la UI no hace sondeo (ADR-015):

| Evento | Cuándo |
|---|---|
| `metrics:updated` | cada ciclo de recopilación, con el lote de métricas frescas |
| `alerts:changed` | alta, cambio de severidad, resolución o cambio de estado de un grupo |
| `inventory:changed` | alta o retirada de un disco o volumen |
| `test:progress` | progreso de una prueba en curso |
| `system:accent-changed` / `system:theme-changed` | el usuario cambia la apariencia de Windows |

### Pendiente de diseño (no incluido)

Todas estas pantallas tienen ya criterios de aceptación en `docs/user-stories.md` (épica H y US-070
a US-074); lo que falta es la composición visual, no la definición funcional.

- **Informes** (US-050): selector de intervalo, resumen de contenido y destino de exportación.
- **Ajustes**: apariencia, frecuencias, umbrales, retención, comportamiento al cerrar, borrado de datos.
- **Asistente inicial** (US-002): detección, exclusión de discos y alias.
- **Acerca de** (US-061) y estados de systray.

Casi todo se compone con el catálogo actual (`Switch`, `Select`, `TextField`, `RadioGroup`,
`SegmentedControl`, `ConfirmDialog`, `EmptyState`, `CodeOutput`). Las excepciones ya están
autorizadas y no requieren decisión nueva: `DateRangePicker`, `FilterBar`, `VirtualList` y `Tooltip`
(AGENTS.md §3, "Autorizados y pendientes de construir").

### Notas para quien programe

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
- El estado inicial de cada pantalla llega por `load` en `+page.ts`, no por `onMount`. Las
  actualizaciones vienen después por eventos.


---

# 15. Sistema de diseño: principios

Fichero de origen: `Design-system/design-system/README.md`

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

Fichero de origen: `Design-system/design-system/tokens.css`

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

  --sdm-text-2xs: 0.6875rem;    /* 11px  etiquetas de píldora y unidades */
  --sdm-text-xs: 0.75rem;       /* 12px  metadatos */
  --sdm-text-sm: 0.78125rem;    /* 12.5px cuerpo denso */
  --sdm-text-base: 0.84375rem;  /* 13.5px título de tarjeta */
  --sdm-text-lg: 0.90625rem;    /* 14.5px título de barra de herramientas */
  --sdm-text-xl: 1.25rem;       /* 20px  título de pantalla */
  --sdm-text-2xl: 1.3125rem;    /* 21px  titular de alerta */
  --sdm-text-metric: 1.6875rem; /* 27px  cifra grande */

  /* En v2 el peso máximo es 600: el material aporta la jerarquía, no la grasa tipográfica. */
  --sdm-weight-regular: 400;
  --sdm-weight-medium: 500;
  --sdm-weight-semibold: 600;

  --sdm-tracking-tight: -0.02em;
  --sdm-tracking-metric: -0.03em;

  /* ---- Espaciado (escala de 4; v2 respira más) ---- */
  --sdm-space-1: 4px;
  --sdm-space-2: 8px;
  --sdm-space-3: 12px;
  --sdm-space-4: 16px;
  --sdm-space-5: 18px;   /* gap canónico entre tarjetas */
  --sdm-space-6: 20px;   /* padding de pantalla */
  --sdm-space-8: 32px;

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

  --sdm-bg: #e9ebf0;
  --sdm-bg-2: #dfe2ea;         /* extremo del degradado del lienzo */

  /* Capas de material. `glass` lleva backdrop-filter; `solid` es el respaldo sin soporte. */
  --sdm-glass: rgba(255, 255, 255, 0.72);
  --sdm-glass-2: rgba(255, 255, 255, 0.5);
  --sdm-glass-3: rgba(120, 124, 140, 0.1);   /* pistas de barra, bloques internos */
  --sdm-solid: #fdfdfe;

  --sdm-hairline: rgba(22, 24, 32, 0.09);
  --sdm-highlight: rgba(255, 255, 255, 0.9); /* brillo superior de 1px */
  --sdm-scrim: rgba(10, 10, 14, 0.34);       /* fondo de diálogo */

  --sdm-text: #191b22;
  --sdm-text-dim: #5f6371;
  --sdm-text-faint: #6a6f7b;
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
  --sdm-accent: #0067c0;
  --sdm-accent-hi: #1a7cd4;    /* extremo claro del degradado vertical del botón */
  --sdm-accent-soft: rgba(0, 103, 192, 0.12);
  --sdm-accent-fg: #0067c0;    /* 5,40:1 sobre el material claro */

  /* Salud suavizada: menos saturación para convivir con el material, manteniendo 4.5:1 como texto
     de píldora sobre el material claro. Verificado con AA; no los aclares. */
  --sdm-ok: #2f7256;
  --sdm-ok-soft: rgba(67, 144, 111, 0.13);
  --sdm-warn: #7d5619;
  --sdm-warn-soft: rgba(183, 129, 58, 0.14);
  --sdm-crit: #a83d45;
  --sdm-crit-soft: rgba(194, 90, 96, 0.13);
  --sdm-unknown: #5d616d;
  --sdm-unknown-soft: rgba(138, 141, 153, 0.13);

  --sdm-shadow: 0 1px 1px rgba(20, 22, 30, 0.05), 0 8px 22px -14px rgba(20, 22, 30, 0.28);
  --sdm-shadow-lift: 0 2px 4px rgba(20, 22, 30, 0.06), 0 24px 60px -22px rgba(20, 22, 30, 0.42);
  --sdm-focus-ring: 0 0 0 3px color-mix(in srgb, var(--sdm-accent) 40%, transparent);
}

/* =======================  TEMA OSCURO  ======================= */
[data-theme="dark"] {
  color-scheme: dark;

  --sdm-bg: #101014;
  --sdm-bg-2: #16161c;

  --sdm-glass: rgba(42, 42, 50, 0.66);
  --sdm-glass-2: rgba(58, 58, 68, 0.42);
  --sdm-glass-3: rgba(255, 255, 255, 0.06);
  --sdm-solid: #1b1b21;

  --sdm-hairline: rgba(255, 255, 255, 0.09);
  --sdm-highlight: rgba(255, 255, 255, 0.13);
  --sdm-scrim: rgba(0, 0, 0, 0.5);

  --sdm-text: #f2f2f6;
  --sdm-text-dim: #a2a4b0;
  --sdm-text-faint: #9195a1;
  --sdm-on-accent: #ffffff;

  --sdm-accent: #3d95ea;
  --sdm-accent-hi: #5aa8f2;
  --sdm-accent-soft: rgba(61, 149, 234, 0.18);
  --sdm-accent-fg: #3d95ea;    /* 5,09:1 sobre el material oscuro */

  --sdm-ok: #6cc79c;
  --sdm-ok-soft: rgba(108, 199, 156, 0.16);
  --sdm-warn: #e0b473;
  --sdm-warn-soft: rgba(224, 180, 115, 0.16);
  --sdm-crit: #e88b90;
  --sdm-crit-soft: rgba(232, 139, 144, 0.16);
  --sdm-unknown: #9396a2;
  --sdm-unknown-soft: rgba(147, 150, 162, 0.14);

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

:focus-visible {
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

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```


---

# 17. tailwind.config.cjs — mapeo de tokens

Fichero de origen: `Design-system/tailwind.config.cjs`

```js
/** Tailwind mapeado 1:1 sobre design-system/tokens.css (v2 material translúcido).
 *  Si una utilidad no existe aquí, el valor no está en el sistema: no inventes clases arbitrarias.
 *  Las capas de material se aplican con las clases .sdm-material / -chrome / -overlay de tokens.css.
 *  @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{svelte,ts,js}"],
  theme: {
    extend: {
      colors: {
        bg: { DEFAULT: "var(--sdm-bg)", 2: "var(--sdm-bg-2)" },
        glass: { DEFAULT: "var(--sdm-glass)", 2: "var(--sdm-glass-2)", 3: "var(--sdm-glass-3)" },
        solid: "var(--sdm-solid)",
        hairline: "var(--sdm-hairline)",
        scrim: "var(--sdm-scrim)",
        fg: { DEFAULT: "var(--sdm-text)", dim: "var(--sdm-text-dim)", faint: "var(--sdm-text-faint)", onAccent: "var(--sdm-on-accent)" },
        accent: { DEFAULT: "var(--sdm-accent)", hi: "var(--sdm-accent-hi)", soft: "var(--sdm-accent-soft)", fg: "var(--sdm-accent-fg)" },
        ok: { DEFAULT: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)" },
        warn: { DEFAULT: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)" },
        crit: { DEFAULT: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)" },
        unknown: { DEFAULT: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)" }
      },
      fontFamily: { sans: "var(--sdm-font-sans)", mono: "var(--sdm-font-mono)" },
      fontSize: {
        "2xs": ["var(--sdm-text-2xs)", { lineHeight: "1.35" }],
        xs: ["var(--sdm-text-xs)", { lineHeight: "1.45" }],
        sm: ["var(--sdm-text-sm)", { lineHeight: "1.5" }],
        base: ["var(--sdm-text-base)", { lineHeight: "1.4" }],
        lg: ["var(--sdm-text-lg)", { lineHeight: "1.3" }],
        xl: ["var(--sdm-text-xl)", { lineHeight: "1.2" }],
        "2xl": ["var(--sdm-text-2xl)", { lineHeight: "1.2" }],
        metric: ["var(--sdm-text-metric)", { lineHeight: "1.05" }]
      },
      fontWeight: { regular: "400", medium: "500", semibold: "600" },
      spacing: {
        1: "var(--sdm-space-1)", 2: "var(--sdm-space-2)", 3: "var(--sdm-space-3)",
        4: "var(--sdm-space-4)", 5: "var(--sdm-space-5)", 6: "var(--sdm-space-6)", 8: "var(--sdm-space-8)"
      },
      borderRadius: {
        window: "var(--sdm-radius-window)", card: "var(--sdm-radius-card)",
        inner: "var(--sdm-radius-inner)", nav: "var(--sdm-radius-nav)", pill: "var(--sdm-radius-pill)"
      },
      height: { "control-sm": "var(--sdm-control-sm)", "control-md": "var(--sdm-control-md)", "control-lg": "var(--sdm-control-lg)" },
      boxShadow: {
        card: "var(--sdm-shadow)",
        lift: "var(--sdm-shadow-lift)",
        edge: "inset 0 1px 0 var(--sdm-highlight)",
        focus: "var(--sdm-focus-ring)"
      },
      backdropBlur: { chrome: "var(--sdm-blur-chrome)", card: "var(--sdm-blur-card)", overlay: "var(--sdm-blur-overlay)" },
      transitionTimingFunction: { sdm: "var(--sdm-ease)" },
      transitionDuration: { fast: "var(--sdm-duration-fast)", base: "var(--sdm-duration-base)", overlay: "var(--sdm-duration-overlay)" }
    }
  },
  plugins: []
};
```


---

# 18. design/types.ts — vocabulario de la UI

Fichero de origen: `Design-system/src/lib/design/types.ts`

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
export type TestStatus = "pending" | "running" | "cancelling" | "completed" | "failed" | "cancelled" | "interrupted";

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
}

export interface AlertGroup {
  id: string;
  ruleKey: string;
  deduplicationKey: string;
  severity: Severity;
  status: AlertStatus;
  title: string;
  /** Frase corta en lenguaje humano; el detalle técnico va aparte. */
  summary: string;
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

# 19. design/health.ts — estado → color, umbrales

Fichero de origen: `Design-system/src/lib/design/health.ts`

```ts
import type { HealthState, Severity, AlertStatus, UnknownReason } from "./types";

/** Único mapa autorizado de estado → token de color. Ningún componente decide colores por su cuenta. */
export const healthToken: Record<HealthState, { fg: string; soft: string; labelKey: string }> = {
  ok: { fg: "var(--sdm-ok)", soft: "var(--sdm-ok-soft)", labelKey: "health.ok" },
  warn: { fg: "var(--sdm-warn)", soft: "var(--sdm-warn-soft)", labelKey: "health.warn" },
  crit: { fg: "var(--sdm-crit)", soft: "var(--sdm-crit-soft)", labelKey: "health.crit" },
  unknown: { fg: "var(--sdm-unknown)", soft: "var(--sdm-unknown-soft)", labelKey: "health.unknown" }
};

export const severityToHealth: Record<Severity, HealthState> = { info: "unknown", warn: "warn", crit: "crit" };

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

/** Severidad máxima de una lista. `unknown` no gana nunca a un estado conocido:
 *  se usa solo cuando no hay ningún estado conocido que mostrar. */
export function worstState(states: readonly HealthState[]): HealthState {
  if (states.includes("crit")) return "crit";
  if (states.includes("warn")) return "warn";
  if (states.includes("ok")) return "ok";
  return "unknown";
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
```


---

# 20. design/format.ts — formato de presentación

Fichero de origen: `Design-system/src/lib/design/format.ts`

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

const isMissing = (v: unknown): v is null | undefined =>
  v === null || v === undefined || Number.isNaN(v);

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

/** Caudal en **bytes por segundo**, que es la unidad que persiste el backend
 *  (`read_bytes_per_second` / `write_bytes_per_second`). La escala es la misma base 1024 que
 *  `formatBytes`, de modo que "180 MB/s" son 180 × 1024² B/s.
 *  Nunca pases MB/s ya convertidos: la conversión vive aquí y en un solo sitio. */
export function formatThroughput(bytesPerSecond: number | null | undefined, locale = i18n.formatLocale): string {
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
export function formatAge(isoUtc: string | null | undefined, now = Date.now(), locale = i18n.formatLocale): string | null {
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

# 21. design/theme.svelte.ts — tema

Fichero de origen: `Design-system/src/lib/design/theme.svelte.ts`

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

# 22. design/accent.ts — acento de Windows

Fichero de origen: `Design-system/src/lib/design/accent.ts`

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
  return "#" + [r, g, b].map((c) => Math.round(Math.min(255, Math.max(0, c))).toString(16).padStart(2, "0")).join("");
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
    .sort((a, b) => Math.abs(luminance(a.rgb) - luminance(base)) - Math.abs(luminance(b.rgb) - luminance(base)));

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
  for (const p of ["--sdm-accent", "--sdm-accent-hi", "--sdm-accent-soft", "--sdm-on-accent", "--sdm-accent-fg"]) {
    document.documentElement.style.removeProperty(p);
  }
}
```


---

# 23. i18n/index.ts — idioma, formato y plurales

Fichero de origen: `Design-system/src/lib/i18n/index.ts`

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

# 24. i18n/es.json

Fichero de origen: `Design-system/src/lib/i18n/es.json`

```json
{
  "common.notAvailable": "No disponible",
  "common.unsupported": "No compatible",
  "common.noData": "Sin datos",
  "common.refresh": "Actualizar",
  "common.cancel": "Cancelar",
  "common.close": "Cerrar",
  "common.continue": "Continuar",
  "common.technicalDetail": "Detalle técnico",
  "common.viewAll": "Ver todos",
  "common.updatedAgo": "hace {value}",
  "health.ok": "Correcto",
  "health.warn": "Advertencia",
  "health.crit": "Crítico",
  "health.unknown": "Sin datos SMART",
  "global.allGood": "Todo en orden",
  "global.paused": "Monitorización pausada",
  "nav.dashboard": "Panel general",
  "nav.alerts": "Alertas",
  "nav.tests": "Pruebas y diagnóstico",
  "nav.reports": "Informes",
  "nav.settings": "Ajustes",
  "nav.monitoredDisks": "Discos monitorizados",
  "nav.pause": "Pausar recopilación",
  "nav.resume": "Reanudar recopilación",
  "disk.temperature": "Temperatura",
  "disk.wear": "Desgaste",
  "disk.activity": "Actividad",
  "disk.powerOnHours": "Horas encendido",
  "disk.firmwareHealth": "Salud del firmware",
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
  "chart.gaps.one": "1 tramo sin datos",
  "chart.gaps.other": "{count} tramos sin datos",
  "chart.emptyLabel": "Gráfica sin datos en el intervalo elegido.",
  "chart.summaryLabel": "Serie de {from} a {to} en {unit}. Mínimo {min}, máximo {max}, último valor {last}. Use las flechas para recorrer los puntos.",
  "chart.resolution.raw": "Muestras cada 30 s",
  "chart.resolution.five_minutes": "Promedios de 5 min",
  "chart.resolution.hourly": "Promedios horarios",
  "alerts.occurrences.one": "Cronología de 1 ocurrencia",
  "alerts.occurrences.other": "Cronología de las {count} ocurrencias",
  "alerts.mutedUntil": "Silenciada hasta {value}",
  "alerts.mutedIndefinitely": "Silenciada hasta reactivación manual",
  "alerts.acknowledgedNote": "Reconocida: sigue activa, el color no cambia.",
  "common.loading": "Cargando…",
  "common.notImplemented": "Esta pantalla todavía no está implementada.",
  "nav.events": "Eventos",
  "startup.failed": "No se pudo iniciar la supervisión",
  "error.unexpected": "Ha ocurrido un error inesperado al hablar con el servicio de supervisión.",
  "dashboard.noDevices": "No hay discos monitorizados",
  "dashboard.noDevicesHint": "Comprueba que la aplicación se está ejecutando con privilegios de administrador.",
  "disk.notFound": "Disco no encontrado",
  "onboarding.title": "Configuración inicial",
  "disk.noSmartData": "Sin datos SMART",
  "disk.open": "Abrir {name}",
  "global.needsAttention.one": "1 disco necesita atención",
  "global.needsAttention.other": "{count} discos necesitan atención",
  "global.noDevices": "Sin discos monitorizados",
  "error.schemaMismatch": "Los datos recibidos del servicio de supervisión no tienen la forma esperada. Puede que la aplicación y su servicio no coincidan de versión.",
  "nav.monitoring": "Supervisión",
  "disk.noVolumes": "Sin volúmenes montados",
  "error.screenFailed": "No se pudo mostrar esta pantalla"
}
```


---

# 25. i18n/en.json

Fichero de origen: `Design-system/src/lib/i18n/en.json`

```json
{
  "common.notAvailable": "Not available",
  "common.unsupported": "Not supported",
  "common.noData": "No data",
  "common.refresh": "Refresh",
  "common.cancel": "Cancel",
  "common.close": "Close",
  "common.continue": "Continue",
  "common.technicalDetail": "Technical detail",
  "common.viewAll": "View all",
  "common.updatedAgo": "{value} ago",
  "health.ok": "Healthy",
  "health.warn": "Warning",
  "health.crit": "Critical",
  "health.unknown": "No SMART data",
  "global.allGood": "All good",
  "global.paused": "Monitoring paused",
  "nav.dashboard": "Overview",
  "nav.alerts": "Alerts",
  "nav.tests": "Tests & diagnostics",
  "nav.reports": "Reports",
  "nav.settings": "Settings",
  "nav.monitoredDisks": "Monitored disks",
  "nav.pause": "Pause collection",
  "nav.resume": "Resume collection",
  "disk.temperature": "Temperature",
  "disk.wear": "Wear",
  "disk.activity": "Activity",
  "disk.powerOnHours": "Power-on hours",
  "disk.firmwareHealth": "Firmware health",
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
  "chart.gaps.one": "1 gap with no data",
  "chart.gaps.other": "{count} gaps with no data",
  "chart.emptyLabel": "No data in the selected range.",
  "chart.summaryLabel": "Series from {from} to {to} in {unit}. Minimum {min}, maximum {max}, latest {last}. Use the arrow keys to step through the points.",
  "chart.resolution.raw": "Samples every 30 s",
  "chart.resolution.five_minutes": "5-minute averages",
  "chart.resolution.hourly": "Hourly averages",
  "alerts.occurrences.one": "Timeline of 1 occurrence",
  "alerts.occurrences.other": "Timeline of {count} occurrences",
  "alerts.mutedUntil": "Muted until {value}",
  "alerts.mutedIndefinitely": "Muted until manually re-enabled",
  "alerts.acknowledgedNote": "Acknowledged: still active, the colour does not change.",
  "common.loading": "Loading…",
  "common.notImplemented": "This screen is not implemented yet.",
  "nav.events": "Events",
  "startup.failed": "Monitoring could not start",
  "error.unexpected": "An unexpected error occurred while talking to the monitoring service.",
  "dashboard.noDevices": "No monitored disks",
  "dashboard.noDevicesHint": "Check that the application is running with administrator privileges.",
  "disk.notFound": "Disk not found",
  "onboarding.title": "Initial setup",
  "disk.noSmartData": "No SMART data",
  "disk.open": "Open {name}",
  "global.needsAttention.one": "1 disk needs attention",
  "global.needsAttention.other": "{count} disks need attention",
  "global.noDevices": "No monitored disks",
  "error.schemaMismatch": "The data received from the monitoring service does not have the expected shape. The application and its service may be on different versions.",
  "nav.monitoring": "Monitoring",
  "disk.noVolumes": "No mounted volumes",
  "error.screenFailed": "This screen could not be shown"
}
```


---

# 26. Tipografía empotrada

Fichero de origen: `Design-system/design-system/fonts/README.md`

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

# 27. smartctl redistribuido

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

# 28. Licencia del código propio

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

# 29. Avisos de terceros

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
| `Design-system/design-system/fonts/InstrumentSans-latin.woff2` | `2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971` |
| `Design-system/design-system/fonts/InstrumentSans-latin-ext.woff2` | `c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2` |

The unmodified `OFL.txt` ships alongside the font files in the same folder. The font is
redistributed under its original family name and is not modified, so the licence requires no name
change. Both files must also be copied into the installed application folder.

### Application dependencies

Rust and JavaScript dependency notices will be generated and reviewed from the locked dependency graph before release. No dependency is yet vendored at the specification stage.
