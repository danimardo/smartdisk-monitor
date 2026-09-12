# 04 · Descarga, requisitos y preguntas frecuentes

## Bloque de descarga (CTA final)

### Titular
Descárgalo. Es gratis, es tuyo y no llama a casa.

### Cuerpo
Instalador para Windows x64, sin cuenta ni suscripción. Todo el código es abierto y auditable en
GitHub.

### Botón
Descargar SmartDisk Monitor → https://github.com/danimardo/smartdisk-monitor/releases

### Línea de apoyo bajo el botón
Windows 10, 11 o Server · se ejecuta con privilegios de administrador · licencia MIT

## Requisitos del sistema

- Windows 10 (versión 1809 o superior), Windows 11, o Windows Server 2016/2019/2022/2025 con
  Experiencia de escritorio.
- Arquitectura x64.
- **WebView2 Runtime**: si falta, el propio instalador lo resuelve sin necesitar conexión a
  Internet.
- Se ejecuta siempre con privilegios de administrador: es lo que necesita para leer los datos
  SMART de los discos físicos.

## Avisos esperados durante la instalación

### "Windows protegió tu PC"
El instalador no va firmado todavía, así que Windows SmartScreen puede avisar de "editor no
reconocido" la primera vez. No es un fallo: en "Más información" → "Ejecutar de todas formas" se
continúa con normalidad.

### Si un disco SATA aparece como "sin datos SMART"
El instalador añade `smartctl.exe` a las aplicaciones permitidas del Control de acceso a carpetas
de Windows Defender —sin esa excepción, Windows bloquea el comando de bajo nivel que SMART
necesita—. Si tienes activada la Protección contra alteraciones de Defender, esa excepción puede
fallar en silencio incluso viniendo del instalador; la propia pantalla del disco afectado ofrece un
botón para reintentarlo.

## Preguntas frecuentes

### ¿Por qué pide permisos de administrador?
Porque leer los datos SMART/NVMe de un disco físico en Windows exige ese nivel de acceso. No hay
forma de evitarlo sin perder esa información.

### ¿Es de verdad gratis?
Sí, sin letra pequeña: sin versión de pago, sin límite de discos ni de tiempo, sin anuncios.

### ¿Qué pasa si mi disco no soporta SMART?
La aplicación lo indica con un estado neutro (gris), no con una alerta falsa. Es habitual en discos
detrás de un controlador RAID, por USB o en máquinas virtuales.

### ¿La aplicación manda algo a Internet?
No, salvo que actives tú mismo la ayuda con IA configurando una clave de OpenRouter. Sin esa clave,
no hay ninguna conexión saliente.

### ¿Qué es la clave de demostración de la IA?
Una clave compartida que trae el propio instalador, limitada a modelos gratuitos, para probar la
explicación con IA con un solo clic y sin crear cuenta. Para uso habitual se recomienda una clave
propia, gratuita también, en openrouter.ai.

### ¿Funciona en segundo plano sin haber iniciado sesión?
No: es una aplicación de escritorio que monitoriza mientras hay una sesión iniciada y sigue en
ejecución. No es un servicio de Windows.

### ¿Puedo desinstalarla sin perder el historial?
Sí: desinstalar quita el ejecutable y los accesos directos, pero conserva la base de datos y los
registros. Para borrarlo todo, Ajustes → «Borrar todos los datos», o borrar la carpeta de datos a
mano tras desinstalar.

### ¿Dónde reporto un problema o pido una funcionalidad?
En el repositorio de GitHub: https://github.com/danimardo/smartdisk-monitor

## Pie de página

SmartDisk Monitor — Daniel Diez Mardomingo — código propio bajo licencia MIT — Windows x64
