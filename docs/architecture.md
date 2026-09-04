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
errores— está en [`ui-contract.md`](ui-contract.md), y los tipos se generan desde Rust para que
ambos lados no puedan divergir en silencio.

## 2.1. Sistema de diseño e interfaz

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

## 3. Componentes

### Inventario

- Descubre discos, interfaces, controladoras, particiones y volúmenes.
- Reconcilia distintas identidades de Windows y smartctl.
- Usa número de serie como identidad preferente y una huella estable como alternativa.
- Detecta altas, retiradas, sustituciones y cambios de firmware.
- Registra capacidades no disponibles y su causa conocida.

### Colector SMART

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

### Colector Windows

- Obtiene salud complementaria y topología mediante las APIs disponibles.
- Lee capacidad y estado del sistema de archivos por volumen.
- Lee actividad, rendimiento y latencias de los contadores `PhysicalDisk`: `% Idle Time` (del que se
  deriva la actividad, acotada a 0–100), `Disk Read Bytes/sec`, `Disk Write Bytes/sec`,
  `Avg. Disk sec/Read` y `Avg. Disk sec/Write`. No se usa `% Disk Time`, que supera el 100 % con
  varias operaciones simultáneas y no es un porcentaje real.
- La instancia del contador (`"0 C: D:"`) se asocia al dispositivo por su número de disco físico,
  no por la letra de unidad, que puede cambiar.
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
- CHKDSK `/scan` con captura de salida. La salida se guarda **en bytes**, y la decodificación para
  presentarla se hace con la detección de `open-questions.md` §Q: las herramientas de Windows no
  coinciden entre sí en la página de códigos, así que codificar una constante produce texto corrupto
  en la mitad de los casos. La codificación deducida se registra junto a la ejecución.
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
- Ante cierre inesperado, el siguiente inicio reconcilia pruebas incompletas y archivos temporales
  huérfanos: toda prueba que quedó en `running` o `cancelling` pasa a `interrupted` y los archivos
  huérfanos se listan con su ruta para que el usuario decida (US-074).
- Solo puede haber una instancia en ejecución; abrir una segunda restaura la ventana de la primera.
  Lo resuelve `tauri-plugin-single-instance`, registrado **el primero** de los plugins porque se
  ejecutan en orden de registro (ADR-025). Su devolución de llamada corre en el proceso que ya
  estaba vivo y llama a `platform::ventana::restaurar_ventana_principal()`, el mismo camino que usa
  el arranque normal: desminimizar, mostrar y enfocar, en ese orden.

## 5. Rutas previstas

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
- Pruebas visuales en temas claro y oscuro, con acento de Windows y color de respaldo.
- Pruebas de ventana mínima, scroll, ausencia de recortes y fallback sin `backdrop-filter`.
- Comprobación automatizada o revisión que impida colores, radios, sombras y tamaños tipográficos literales en componentes.
- Verificación de foco, teclado, contraste AA, etiquetas accesibles y `prefers-reduced-motion`.
