# SmartDisk Monitor

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

## Funcionalidades

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

## Descargar

Instalador para Windows x64 en la [página de Releases](https://github.com/danimardo/smartdisk-monitor/releases).

Requiere Windows 10 (1809+), Windows 11 o Windows Server 2016+ con Experiencia de escritorio, y el
**WebView2 Runtime** (el instalador lo resuelve sin conexión si falta). Se ejecuta siempre con
privilegios de administrador: es lo que necesita para leer SMART de los discos físicos.

El instalador **no va firmado** — Windows SmartScreen mostrará "Windows protegió tu PC"/"editor no
reconocido" la primera vez que se ejecute. Es un aviso esperado, no un fallo: para continuar, "Más
información" → "Ejecutar de todas formas". Conseguir un certificado de firma de código es trabajo
pendiente, ya recogido en [`docs/roadmap.md`](docs/roadmap.md).

## Desarrollo

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

## Documentación

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

## Identidad del proyecto

- Producto: SmartDisk Monitor
- Autor: Daniel Diez Mardomingo
- Repositorio: https://github.com/danimardo/smartdisk-monitor
- Licencia del código propio: MIT
- Plataforma: Windows x64

## Instalación y desinstalación

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
