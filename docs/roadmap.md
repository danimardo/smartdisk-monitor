# Roadmap y backlog

## Criterio de priorización

- P0: necesario para considerar utilizable la versión 1.0.
- P1: debe entrar en 1.0 salvo riesgo técnico demostrado.
- P2: candidato a versiones posteriores.

## Fase 0 — Validación técnica

Objetivo: reducir riesgos antes de construir la interfaz completa.

- Crear esqueleto Tauri 2 + SvelteKit (`adapter-static`, SSR off) + TypeScript + Tailwind (ADR-014).
- Añadir la tipografía Instrument Sans a `design-system/fonts/` con su licencia (ADR-018) y hacer que
  la compilación falle si falta.
- Integrar `src/design-system/tokens.css`, la configuración Tailwind, los módulos de diseño, i18n y el catálogo Svelte entregado.
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

Riesgos abiertos y su criterio de cierre: [`open-questions.md`](open-questions.md) §I.

Salida: informe de viabilidad y fixtures anonimizados.

## Versión 0.1 — Inventario y recopilación

- Inventario físico y lógico.
- Selección y alias de discos.
- Colector SMART normalizado y vista JSON.
- Contadores de Windows y capacidad.
- Persistencia SQLite y migraciones.
- Planificador y estados de fuente.
- Altas y bajas de discos en caliente (US-014).
- Tests unitarios de parsers.

## Versión 0.2 — Panel e historial

- Panel general, con comportamiento definido para muchos discos.
- Detalle de disco y volumen.
- Gráficas e intervalos.
- Registro de eventos con deduplicación.
- Español e inglés.
- Tema claro, oscuro y de sistema.
- Asistente inicial.
- Revisión de diseño previa para Informes, Ajustes y asistente inicial, todavía no cubiertos por los bocetos aprobados.

## Versión 0.3 — Alertas y systray

- Motor de reglas según [`alert-rules.md`](alert-rules.md), con histéresis y deduplicación.
- Agrupación y ciclo de vida de alertas, incluido el ciclo de recaída.
- Notificaciones de Windows.
- Estados y menú de systray.
- Pausa y reanudación.
- Preferencia de cierre recordable.
- Diseño y revisión de los estados de systray antes de cerrar su implementación.

## Versión 0.4 — Pruebas e informes

- Benchmark de archivo temporal.
- CHKDSK `/scan`.
- Autotest SMART corto.
- CSV, JSON y HTML.
- ZIP diagnóstico anonimizado.

## Versión 0.9 — Endurecimiento

- Retención, agregación y copias pre-migración (US-071).
- Recuperación tras cierre inesperado (US-074).
- Pantalla de Ajustes completa (US-070 a US-073).
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
- Validación completa contra la definición de terminado de [`ui-design.md`](ui-design.md) §8.

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
- ha sido verificada en una compilación empaquetada, no solo en desarrollo;
- no ha dejado ninguna decisión implícita en el código: lo que hubo que decidir está en
  `open-questions.md` o en el documento normativo que corresponda.
