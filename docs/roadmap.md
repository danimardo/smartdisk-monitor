# Roadmap y backlog

**Estado real, 2026-09-05**: las ocho historias de usuario de
[`specs/001-monitor-discos-windows/`](../specs/001-monitor-discos-windows/tasks.md) están
implementadas — inventario, panel e historial, alertas y systray, ajustes, pruebas y diagnóstico,
informes y exportación, instalación/reconocimiento/retirada. Lo que sigue en este documento sin
tachar es lo que de verdad queda abierto, no una plantilla sin actualizar. El seguimiento tarea a
tarea vive en `tasks.md`; este documento es el resumen por versión.

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

Riesgos abiertos y su criterio de cierre: [`open-questions.md`](open-questions.md) §I. En síntesis,
lo que de verdad sigue abierto de toda la Fase 0: **I.2** (toast elevado), **I.5** (RAID/USB
reales), el contraste en Windows Server, **T111** (escalado visual) y **T115** (guion completo sobre
un instalador real) — los cinco exigen o hardware que esta sesión no tiene, o un instalador
empaquetado, o revisión visual humana.

Salida: informe de viabilidad y fixtures anonimizados.

Las versiones 0.1 a 0.9 siguientes están **implementadas** (Historias 1-8 de
`specs/001-monitor-discos-windows/tasks.md`); se conservan como agrupación temática del alcance, no
como lista pendiente.

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

- ~~Instalador para todos los usuarios~~ **hecho**: NSIS, WebView2 sin conexión (ADR-020).
- ~~Desinstalación que conserva datos~~ **hecho**: véase «Instalación y desinstalación» en
  [`README.md`](../README.md).
- Aviso documentado de SmartScreen por falta de firma — pendiente de una compilación firmada real.
- ~~Licencia MIT, terceros y atribuciones~~ **hecho**: `THIRD_PARTY_NOTICES.md`.
- ~~Nombre y versión dinámicos~~ **hecho**: `get_app_info` usa `app.package_info()` (T104).
- Release manual en GitHub — pendiente de que el usuario decida publicar.
- **Validación completa contra la definición de terminado de [`ui-design.md`](ui-design.md) §8**
  sigue abierta: exige recorrer cada pantalla al 125 %/150 %/200 % y en la ventana mínima
  1024 × 560 (T111) y ejecutar el guion completo de `quickstart.md` sobre una compilación
  empaquetada, no en desarrollo (T115). Ambas requieren interacción del usuario (revisión visual y
  una instalación elevada real) y quedan deliberadamente para cuando esté disponible.

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
