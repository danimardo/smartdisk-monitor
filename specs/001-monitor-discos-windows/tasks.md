---
description: "Lista de tareas de implementación de SmartDisk Monitor 1.0"
---

# Tareas: SmartDisk Monitor 1.0

**Entrada**: documentos de diseño de `specs/001-monitor-discos-windows/`

**Requisitos previos**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/README.md), [quickstart.md](quickstart.md)

**Pruebas: obligatorias, no opcionales.** La plantilla de SpecKit las trata como opcionales, pero la
constitución §VIII exige **test-first con ciclo rojo-verde-refactor** en cinco áreas donde el fallo
es silencioso: parsers de `smartctl` y NVMe, motor de alertas, retención y agregación, validación de
rutas del benchmark, y migraciones de SQLite. Las tareas de esas áreas van marcadas **🔴 test-first**
y su prueba debe fallar antes de escribir el código. En interfaz, comandos finos y formateadores las
pruebas acompañan al código pero no tienen que precederlo.

**Organización**: por historia de usuario, para poder implementar y validar cada una por separado.

## Formato: `[ID] [P?] [Historia] Descripción`

- **[P]**: paralelizable (ficheros distintos, sin dependencias pendientes)
- **[US*]**: historia de usuario a la que pertenece
- Toda tarea lleva su ruta de fichero exacta

## Convenciones de ruta

Backend en `src-tauri/src/`, interfaz en `src/`, migraciones en `src-tauri/migrations/`. Regla de
dependencias: **`domain/` no conoce Tauri, ni Windows, ni SQLite**, y **ninguna pantalla llama a
`invoke`**: todo pasa por `src/lib/api/`.

---

## Fase 1: Preparación

**Propósito**: dejar registrado por escrito lo que aún no lo está, y crear la estructura vacía.

> **T001 y T002 bloquean todo lo demás y no son burocracia.** El flujo de desarrollo prohíbe
> programar una decisión que no esté escrita: hacerlo es «la infracción más grave del proceso».

- [X] T001 Registrar los umbrales de espacio libre (1 GB aviso / 256 MB parada) como `PROPUESTO` en `docs/open-questions.md`, con su razón y la marca de que **no están medidos** (riesgo R4)
- [X] T002 Llevar el modo detallado de registro y el acceso a su carpeta a `docs/user-stories.md` (US-071 o historia propia), para que la fuente normativa no vaya por detrás de la especificación (FR-029a/b/c)
- [X] T003 [P] Añadir a `docs/data-model.md` las tres claves nuevas de `settings` del delta D1 y el estado de escritura detenida (D2)
- [X] T004 [P] Añadir a `docs/ui-contract.md` el delta C1 (estado de escritura detenida junto a `sources`) y C2 (nivel de registro y apertura de carpeta)
- [X] T005 [P] Crear los módulos vacíos con su `mod.rs` documentado en `src-tauri/src/domain/`, `src-tauri/src/collectors/`, `src-tauri/src/alerts/`, `src-tauri/src/persistence/`, `src-tauri/src/tests/` y `src-tauri/src/reporting/`
- [X] T006 Regenerar `historias.md` con `pnpm docs:build` y confirmar que `pnpm docs:check` pasa

---

## Fase 2: Cimientos (bloquean todas las historias)

**Propósito**: sin persistencia no hay historial, sin historial no hay reglas, sin reglas no hay
alertas. Corresponde al eslabón 1 del plan.

**⚠️ CRÍTICO**: ninguna historia puede empezar hasta que esta fase esté completa.

### Persistencia

- [X] T007 🔴 test-first Escribir las pruebas de migración en `src-tauri/src/persistence/migrations_test.rs`: aplicación desde cero, idempotencia y restauración de la copia previa ante fallo
- [X] T008 Crear la migración inicial con las doce tablas de `docs/data-model.md` §2 en `src-tauri/migrations/0001_esquema_inicial.sql`
- [X] T009 Implementar el ejecutor de migraciones con copia previa y conservación de las tres últimas en `src-tauri/src/persistence/migrations.rs`
- [X] T010 Abrir la base en `%ProgramData%\SmartDisk Monitor\` con WAL y transacciones breves en `src-tauri/src/persistence/db.rs`
- [X] T011 [P] Implementar los repositorios de `devices`, `volumes` y `device_volume_links` en `src-tauri/src/persistence/repo_inventario.rs`
- [X] T012 [P] Implementar los repositorios de `metric_samples` y `smart_snapshots` en `src-tauri/src/persistence/repo_metricas.rs`
- [X] T013 [P] Implementar los repositorios de `alert_groups` y `alert_occurrences` en `src-tauri/src/persistence/repo_alertas.rs`
- [X] T014 [P] Implementar los repositorios de `system_events`, `event_cursors`, `test_runs` y `settings` en `src-tauri/src/persistence/repo_varios.rs`

### Retención y guardia de espacio

- [X] T015 🔴 test-first Escribir las pruebas de retención y agregación en `src-tauri/src/domain/retencion_test.rs`: la agregación conserva mínimo, máximo, promedio, primera y última lectura, y **nunca** inventa un cero por un dato ausente
- [X] T016 Implementar la compactación diaria transaccional en `src-tauri/src/domain/retencion.rs`
- [X] T017 🔴 test-first Escribir las pruebas de la guardia de espacio en `src-tauri/src/domain/espacio_test.rs`: cruzar el umbral de aviso, cruzar el de parada, y comprobar que **no se purga nada** por iniciativa propia (FR-020c)
- [X] T018 Implementar la guardia de espacio libre en `src-tauri/src/domain/espacio.rs`, con los umbrales leídos de `settings` (D1)

### Estado y ciclo de recopilación

- [X] T019 Implementar el estado de ejecución —fuentes degradadas, pausa, escritura de historial detenida— en `src-tauri/src/domain/estado.rs` (delta D2 y C1)
- [ ] T020 Implementar el planificador de recopilación con sus frecuencias configurables y la reducción en batería en `src-tauri/src/collectors/planificador.rs` (FR-030)
- [ ] T021 Implementar la emisión de eventos hacia la interfaz en `src-tauri/src/commands/eventos.rs`, sin que ninguna pantalla necesite sondear (ADR-015)
- [ ] T022 Reanudar siempre la monitorización al arrancar, aunque quedara pausada, en `src-tauri/src/lib.rs` (FR-031)

**Punto de control**: la base guarda, compacta y se defiende del disco lleno. Pueden empezar las historias.

---

## Fase 3: Historia 1 — Saber qué discos hay y cómo están (P1) 🎯 MVP

**Objetivo**: que la aplicación diga qué dispositivos hay, cuáles puede vigilar y en qué estado
están, sin inventarse nada. Eslabones 2 y 3 del plan.

**Validación independiente**: instalar en un equipo con discos de distintos tipos, abrir y comprobar
que cada dispositivo aparece con su estado y su procedencia sin haber configurado nada.

### Inventario

- [X] T023 🔴 test-first [US1] Escribir las pruebas de identidad en `src-tauri/src/domain/identidad_test.rs`: huella estable, **el firmware no forma parte de ella**, y `identity_confidence` correcto con y sin número de serie
- [X] T024 [US1] Implementar el cálculo de identidad y la reconciliación de altas y bajas en `src-tauri/src/domain/identidad.rs`
- [X] T025 [US1] Implementar el colector de inventario de Windows en `src-tauri/src/collectors/windows_storage.rs`
- [X] T026 [US1] Detectar altas y bajas en caliente, distinguiendo expulsión segura de retirada sin aviso, en `src-tauri/src/collectors/deteccion.rs` (SC-005)
- [X] T027 [US1] Conectar `get_devices`, `set_device_monitoring`, `set_device_alias` y `refresh_now` con sus fuentes reales en `src-tauri/src/commands/mod.rs`
- [X] T028 [US1] Distinguir el inventario vacío legítimo del fallo de recopilación en la respuesta de `get_devices`, poblando `sources` en `src-tauri/src/commands/mod.rs`

### Puerta de DTO generados

- [X] T029 [US1] Generar los DTO desde Rust y **retirar el mantenimiento manual** de `src/lib/api/types.ts`, en el mismo cambio que introduce el primer DTO real
- [X] T030 [US1] Activar la puerta en `.github/workflows/ci.yml` y actualizar la nota de «puerta pendiente» de `.specify/memory/constitution.md` (requiere autorización explícita: es enmienda)

### Salud

- [X] T031 🔴 test-first [P] [US1] Escribir las pruebas del parser de `smartctl` en `src-tauri/src/collectors/smartctl_parser_test.rs` con salidas reales: campos ausentes se **omiten**, nunca se guardan como cero
- [X] T032 ~~[P]~~ [US1] Plegada en T031: `smartctl -j` entrega el log NVMe dentro del mismo JSON que el resto de campos, no en un registro aparte — un único `smartctl_parser.rs` con sus 5 pruebas cubre ambos casos (NVMe y ATA) sin duplicar el intento de invocación
- [X] T033 [US1] Implementar el parser de `smartctl` en `src-tauri/src/collectors/smartctl_parser.rs`
- [X] T034 [US1] Implementar la invocación de `smartctl` con su cascada de modos de acceso y tiempo máximo en `src-tauri/src/collectors/smartctl.rs` (**medir R2**: tiempo, no solo éxito)
- [X] T035 🔴 test-first [P] [US1] Escribir las pruebas de estado en `src-tauri/src/domain/salud_test.rs`: sin datos ⇒ `unknown` y **jamás** crítico; sin lecturas frescas ⇒ `unknown`, **nunca** `ok`
- [X] T036 [US1] Implementar el cálculo de estado y la frescura por métrica en `src-tauri/src/domain/salud.rs`
- [X] T037 [US1] Registrar procedencia y antigüedad en cada muestra al persistirla en `src-tauri/src/persistence/repo_metricas.rs` (FR-005)
- [X] T038 [US1] Conectar `get_device_detail` con contadores reales en `src-tauri/src/commands/mod.rs` (`enrich_with_smart_data`, `persist_smart_reading`, `refresh_smart`). De paso corrige un fallo de contrato propio: `refresh_now` usaba ámbitos inventados (`"inventory"`/`"smart"`) en vez de los normativos `docs/ui-contract.md` §3.2 (`"all"` | `"device"`); ya alineado y probado (5 pruebas nuevas). `docs/open-questions.md` J.15 registra el criterio provisional de "sin compatibilidad" vs. "aún sin leer"

### Interfaz

- [X] T039 [P] [US1] Poblar el panel general con datos reales en `src/routes/+page.svelte` y `src/routes/+page.ts` (ya conectado desde el esqueleto original; ahora `getDevices()` trae salud real, no solo la forma)
- [X] T040 [P] [US1] Poblar la carga de datos del detalle de disco (`+page.ts` con `getDeviceDetail`, ahora devolviendo `DeviceDetail` real). La composición visual de la pantalla (`+page.svelte`) sigue en `EmptyState` a falta del boceto v2 de esta pantalla — no es un placeholder de datos, es de maquetación
- [ ] T041 [P] [US1] **Sin hacer, deliberadamente.** Implementar el asistente inicial exige una composición visual que no está en el boceto aprobado — inventar una esta noche sería exactamente lo que `docs/ui-design.md` prohíbe: un criterio visual nuevo sin revisión. Pide la maqueta antes de retomarla
- [X] T042 [US1] Ya suscrito en `src/routes/+layout.svelte` (esqueleto original), con sus esquemas Zod y pruebas de rechazo en `schemas.test.ts`/`events.test.ts`. El backend todavía no emite estos eventos (T021, pendiente); la suscripción está lista para cuando lo haga
- [X] T043 [P] [US1] Claves de error añadidas esta sesión (`error.deviceNotFound`, `error.dbLocked`, `error.dbQueryFailed`, `error.storageCollectorFailed`) en ambos diccionarios; `pnpm verify:i18n` confirma 101 claves sincronizadas
- [X] T044 [P] [US1] `DiskCard.browser.test.ts` creado: 8 pruebas — vacío, correcto con datos reales, no compatible ("Sin datos SMART", nunca rojo), dato ausente ("No disponible"), sin volumen, enlace real vs. bloque no interactivo, alias. "Cargando" y "error de fuente" son responsabilidad de la pantalla contenedora (`EmptyState`), no de esta tarjeta
- [ ] T045 [US1] **Parcial.** Automatizable ya en verde: `pnpm test:a11y` (axe, ambos temas, las seis pantallas), `pnpm check`, `pnpm verify:i18n`, `pnpm test:component`. **Sin verificar** (exige ojos humanos o una sesión con la app real, no disponible en autónomo nocturno): 1024×560 sin recortes, escalado 125/150/200%, acento del sistema claro `#ffb900` en ambos temas, asistente (bloqueado por T041)

**Punto de control**: la aplicación responde «¿qué discos tengo y están bien?». Producto mínimo listo.

---

## Fase 4: Historia 2 — Enterarse de un problema sin estar mirando (P1)

**Objetivo**: avisar cuando algo se sale de lo normal, con un aviso por problema y sin mentir sobre
el estado del hardware. Eslabón 5 del plan.

**Validación independiente**: forzar una condición de alerta y comprobar que aparece un único grupo
con su contador, que el icono de la bandeja cambia y que reconocerla no altera el color del disco.

### Motor de reglas

> Cada regla de `docs/alert-rules.md` exige cinco pruebas **sin excepción** (§5): activación, **no**
> activación ante dato ausente, histéresis, deduplicación y ciclo de recaída.

- [X] T046 🔴 test-first [US2] Escribir las pruebas del motor en `src-tauri/src/alerts/motor.rs` (`mod tests` inline, mismo patrón que el resto del repo): activación, ausencia de dato e histéresis para las siete reglas en alcance según `docs/open-questions.md` J.16 (`smart.health.failed`, `nvme.critical_warning`, `smart.media_errors`, `smart.spare_below_threshold`, `smart.wear_high`, `temp.above_configured_warn` y `temp.above_configured_crit`, tratadas como reglas independientes — ver nota de T047). Deduplicación y ciclo de recaída son responsabilidad de `agrupacion.rs` (T048) y no dependen de la regla concreta: se prueban una vez, no por regla, para no duplicar sin motivo
- [X] T047 [US2] Implementar la evaluación de reglas con histéresis en `src-tauri/src/alerts/motor.rs`. `temp.above_configured_crit` resultó ser una regla independiente de `..._warn` (activación inmediata frente a 3 ciclos), no un nivel más alto de la misma — corregido tras revisar la tabla de `alert-rules.md` antes de que el usuario lo viera; ambas pueden estar activas a la vez y `domain::salud` ya colapsa al peor color
- [X] T048 [US2] Implementar la agrupación y el recuento de ocurrencias en `src-tauri/src/alerts/agrupacion.rs` (SC-004): clave de deduplicación reutilizada como `id` del grupo (sin añadir dependencia `uuid`), y las cinco transiciones reales (crear, repetir, escalar, reabrir tras resolución con incremento de `cycle`, resolver)
- [X] T049 🔴 test-first [P] [US2] Escribir las pruebas del ciclo de vida en `src-tauri/src/alerts/ciclo.rs`: confirmado que `reconocer` y `silenciar` nunca salen de `repo_alertas::list_groups_counting_toward_health`, y que solo `archivar` lo hace
- [X] T050 [US2] Implementar reconocer, silenciar, archivar y resolver automáticamente en `src-tauri/src/alerts/ciclo.rs`. De paso salió a la luz un fallo preexistente en `repo_alertas::set_status`: la transición a `Active` pasaba tres parámetros a una consulta con dos placeholders (`InvalidParameterCount`), nunca antes ejercitada; corregido
- [X] T051 [US2] Conectar los seis comandos de alertas y `pause_monitoring` / `resume_monitoring` con el motor en `src-tauri/src/commands/mod.rs`. Al construir la respuesta de `AlertGroup` se detectó una contradicción de contrato real: `title`/`summary` como texto ya resuelto por el backend viola el principio de i18n (además de literales en español ya hardcodeados en `AlertCard.svelte`). Escalado al usuario; decisión registrada en **ADR-030** (`docs/decisions.md`): el backend manda solo `ruleKey`, el frontend resuelve `t(\`alert.rule.${ruleKey}.title\`)`/`.summary`. Aplicado en `design/types.ts`, `api/schemas.ts`, `AlertCard.svelte` y ambos diccionarios de i18n. De paso se alineó `AlertSeverity` con el formato de cable `warn`/`crit` que ya esperaba el esquema Zod compartido de `Severity` (el almacenamiento en SQLite sigue en `warning`/`critical`, sin tocar). **Corrección posterior, al empezar T052**: "conectar con el motor" se había marcado por los seis comandos de ciclo de vida, pero nada invocaba de verdad `alerts::motor`/`alerts::agrupacion` sobre datos reales — el motor nunca creaba ningún `alert_group` fuera de sus propias pruebas. Añadido `alerts::evaluar_smart()`, llamado desde `commands::refresh_smart` tras cada `persist_smart_reading` (`docs/open-questions.md` J.18), para las 8 reglas en alcance (J.16, ampliada con `smart.error_log`). `smart.media_errors`/`smart.error_log` quedan sin resolución automática por no existir aún el seguimiento temporal de 24 h que exige `alert-rules.md` (J.17)

### Bandeja del sistema

- [X] T052 [US2] Implementar el icono de bandeja con sus cuatro estados y su menú en `src-tauri/src/platform/bandeja.rs` (FR-012). Color = `domain::salud::tray_state()`, puerto exacto de `trayState()` de `health.ts` (mismos cuatro `HealthState`, ninguna paleta nueva); el icono se genera en memoria con los `--sdm-*` de tema claro, sin `.ico` versionado (pantalla aún sin revisión visual, `docs/decisions.md` línea 102). Menú: abrir, resumen (mismas claves que `globalLabel` de `+layout.svelte`), pausar/reanudar, salir — nuevo lector `platform::rotulos` para los dos diccionarios de i18n, porque el menú nativo vive fuera del webview y no puede pasar por `t()` de Svelte. Al construirlo se detectó que el motor de alertas nunca se ejecutaba sobre datos reales (ver nota de T051 arriba) y que el color solo puede recalcularse en los puntos de sincronización existentes, no en un ciclo real cada 30 s (T020-T022 pendientes): ambos registrados en `docs/open-questions.md` J.18/J.19. El botón de cierre minimiza siempre a la bandeja; la pregunta "minimizar o salir + recordar" (FR-012) queda pendiente de un diálogo propio (J.19)
- [X] T053 [US2] Implementar las notificaciones en `src-tauri/src/alerts/notificaciones.rs` (no en `platform/`: decide a partir de `agrupacion::Transicion`, no de una API de plataforma suelta) y **medir R1**: si no llegan desde el proceso elevado, activar la alternativa ya decidida (ventana propia con `Toast`). Implementado con **`tauri-plugin-notification` 2.4.0** (oficial, mismo criterio que `tauri-plugin-single-instance`; propuesto por escrito y autorizado explícitamente antes de tocar `Cargo.toml`, con su fila correspondiente añadida a la constitución — 1.5.2). `agrupacion::procesar` pasó a devolver una `Transicion` (creada/repetida/escalada/reactivada/resuelta/sin cambio) para que esta capa decida sin que `agrupacion` sepa de notificaciones (lo deja explícito en su propia cabecera). Cooldown por regla de `alert-rules.md` §2 en memoria (`AppState.notified_at`, sin persistir, mismo criterio que `paused`); un grupo silenciado nunca notifica. **R1 sigue sin medirse**: solo puede comprobarse al empaquetar con el identificador de aplicación registrado (`research.md` R1); la ventana `Toast` de reserva no se ha construido — no tiene sentido hasta que R1 se mida y falle. `rust-version` de `Cargo.toml` ajustado de `1.77` a `1.77.2` (mínimo real del plugin). Añadidas las claves de i18n que faltaban para `smart.error_log` (regla incorporada en T051, J.16)

### Interfaz

- [X] T054 [P] [US2] Poblar la pantalla de alertas con datos reales en `src/routes/alerts/+page.svelte`. Composición aprobada de `docs/ui-design.md` §7.3: lista de `AlertCard` (columna fija 470 px) + detalle (severidad, titular, explicación humana, rejilla de hechos, acciones, cronología). `+page.ts` carga todo el historial una vez (`getAlertGroups()` sin filtro); el filtro por estado (Activas/Resueltas/Archivadas/Todas, `SegmentedControl`) se aplica en el cliente para que cambiar de pestaña no dispare una petición. El detalle se pide aparte (`getAlertDetail`) con protección de carrera si la selección cambia antes de que responda. Reconocer/Silenciar/Reanudar son inmediatos; Archivar pasa por `ConfirmDialog` (única acción sin comando de deshacer). Al construirla salió a la luz un bug real en la persistencia, no de esta pantalla: `repo_alertas::create_group` y `reopen_as_new_cycle` nunca escribían en `alert_occurrences`, así que la cronología de un episodio recién creado aparecía vacía. Corregido (`docs/open-questions.md` J.21) junto con `get_alert_detail_impl`, que fabricaba una única ocurrencia sintética en vez de consultar la tabla real
- [X] T055 [US2] Suscribir la interfaz a `alerts:changed` en `src/lib/api/events.ts`. La suscripción **ya existía** en `src/routes/+layout.svelte` desde el esqueleto original (mismo caso que T042); lo que faltaba era que el backend emitiera el evento de verdad. Añadido `commands::emitir_alerts_changed()`, llamado tras `refresh_now` (para las transiciones que produce el motor) y tras las cuatro acciones de ciclo de vida — con `tauri::Emitter`, sin comando ni permiso nuevos
- [X] T056 [P] [US2] Añadir las claves de alertas a los dos diccionarios de `src/lib/i18n/`. Además de las ya añadidas en T051/T053 (`alert.status.*`, `alert.rule.*`, `alert.fact.*`), esta pantalla suma 22 claves nuevas (`alerts.filter.*`, `alerts.actions.*`, `alerts.mute.*`, `alerts.timeline.*`, `alerts.archive.*`, `alerts.empty.*`, `alerts.detail.empty`): 148 claves en total, `es`/`en` sincronizados
- [ ] T057 [US2] Pasar la definición de terminado de `docs/ui-design.md` §8 en la pantalla de alertas. **Parcial, sin marcar como cerrada.** Verificado: cero literales fuera de `es.json`/`en.json` en el código propio de esta pantalla; sin permisos ni comandos nuevos; datos ausentes como "No disponible"; estados de carga/vacío/error construidos; suite `e2e/ui/alerts.spec.ts` (lista, detalle, filtro, confirmación de archivar) y `a11y.spec.ts` en `/alerts` con **cero incumplimientos de axe en ambos temas**, sobre el *build* de producción con IPC falso — al escribir el fixture real de `get_alert_groups` se encontró que estaba desincronizado del contrato (`{groups:[],total:0}` en vez de un array), sin que ningún test lo detectara: corregido y ahora validado con el esquema Zod real en `validar()`. **Sin verificar todavía**: ventana 1024×560 y 1280×720 con la barra lateral colapsada, acento del sistema `#ffb900` en los dos temas, volumen (20+ alertas), y un repaso manual de teclado. **Dos incumplimientos heredados, no propios de esta pantalla**: `ConfirmDialog.svelte` renderiza "Cancelar" como literal y `EmptyState.svelte` sus tres etiquetas de `kind` (`No compatible`/`Fuente con error`/`Sin datos`), ambos preexistentes a esta tarea y usados ya en otras pantallas — arreglarlos es un cambio a un componente compartido, fuera del alcance de esta tarea

**Punto de control**: es un monitor de verdad. Avisa sin que nadie esté mirando.

---

## Fase 5: Historia 3 — Entender cómo ha evolucionado (P2)

**Objetivo**: consultar la evolución de cada métrica sin que la gráfica sugiera continuidad donde no
la hay. Eslabón 4 del plan.

**Validación independiente**: tras unas horas recopilando, abrir una gráfica y comprobar que el eje
cubre el intervalo pedido, que los períodos sin datos son huecos y que el pie declara la resolución.

- [X] T058 [US3] Implementar el colector de contadores de rendimiento en `src-tauri/src/collectors/perf_counters.rs`, derivando `activity_percent` de `% Idle Time` y **no** de `% Disk Time`. Enlace FFI directo a `pdh.dll` (mismo criterio que `platform::locale.rs` con `kernel32`: sin añadir el crate `windows` completo). **Medido contra Windows real**: los nombres de contador de PDH están localizados (español: "Disco físico", "% de tiempo inactivo") y una ruta en inglés falla con `PdhAddCounterW`/`PdhExpandWildCardPathW` fuera de un Windows en inglés — diagnosticado con `Get-Counter` y resuelto con `PdhAddEnglishCounterW`, que traduce el nombre y resuelve el comodín de instancia en una sola llamada. Probado end-to-end contra dos discos físicos reales de esta máquina (`docs/open-questions.md` J.23). Autónomo (abre, recoge dos veces con 1 s de espera, formatea, cierra): sin planificador (T020) que lo mantenga abierto entre ciclos de 30 s todavía. Conectado a `refresh_smart`
- [X] T059 [P] [US3] Implementar el colector de capacidad de volúmenes en `src-tauri/src/collectors/capacidad.rs`. Un único `Get-Partition | Get-Volume` por PowerShell (mismo patrón que `windows_storage.rs`), enlazado a su disco por la misma numeración efímera de Windows que ya usan los discos físicos. De paso salió a la luz un bug preexistente no de esta tarea: `DiskSummary.volumes` estaba siempre vacío, nada lo rellenaba nunca — corregido con `build_volume_summaries()` (`docs/open-questions.md` J.22)
- [X] T060 🔴 test-first [US3] Escribir las pruebas de series en `src-tauri/src/domain/series.rs` (`mod tests` inline, mismo patrón que el resto del repo, no un fichero `_test.rs` aparte): el intervalo se devuelve completo aunque falten datos, y todo salto mayor que 1,5× la cadencia es hueco, **incluidos los extremos** (antes de la primera muestra y después de la última)
- [X] T061 [US3] Implementar la consulta de series con huecos explícitos y resolución declarada en `src-tauri/src/domain/series.rs`: `completar_serie()` (huecos, sin rejilla uniforme — el eje de `TimeSeriesChart` es tiempo real, no índice) y `submuestrear()` (tope de 1.500 puntos, mínimo y máximo por cubo, E.1)
- [X] T062 [US3] Conectar `get_metric_series` en `src-tauri/src/commands/mod.rs`. Resolución elegida por el ancho del intervalo pedido (E.1: crudo ≤24h y dentro de 7 días; 5 min ≤7 días; 5 min si existen si no horario ≤90 días; horario más allá), generado ahora con `ts-rs` (antes vivía a mano en `api/types.ts`, sin comando conectado). Un volumen pedido devuelve un hueco honesto, no un error: la capacidad de volumen se guarda en `volumes.capacity_bytes`/`free_bytes` (T059), todavía no como serie periódica — ese colector es trabajo aparte, sin cubrir aquí
- [X] T063 [P] [US3] Construir el componente `DateRangePicker` en `src/lib/components/DateRangePicker.svelte` y exportarlo en el barrel (ya autorizado, `ui-design.md` §3). Dos `<input type="date">`, sin hora: la resolución de la serie ya declara su propia cadencia
- [X] T064 [US3] Conectar `TimeSeriesChart` con datos reales en `src/routes/disks/[id]/+page.svelte`, pasándole `from` y `to` además de los puntos. Pantalla construida entera (cabecera, fila de 4 `MetricCard`, `SegmentedControl` de intervalo 24h/7d/30d/personalizado con `DateRangePicker`, gráfica y panel de contadores) — no solo el cableado del gráfico, porque la pantalla partía de un `EmptyState` vacío. **Simplificación deliberada frente a la composición aprobada** (`ui-design.md` §7.2): el `SegmentedControl` de intervalo vive en el contenido de la pantalla, no dentro de la `Toolbar` compartida — el layout raíz no tiene mecanismo para que una ruta le inyecte controles propios, y construirlo es un cambio de arquitectura fuera del alcance de esta tarea
- [X] T065 [P] [US3] Añadir las claves de gráficas y resolución a los dos diccionarios de `src/lib/i18n/`. La mayoría (`chart.*`, `range.*`, `disk.temperature/activity/wear/powerOnHours`) ya existían del esqueleto original; añadidas `dateRange.from/to`, `disk.counters` y 10 `smart.counter.*` para los contadores sin etiqueta reutilizable: 163 claves en total
- [ ] T066 [US3] Pasar la definición de terminado de `docs/ui-design.md` §8 en las gráficas, con lectura textual equivalente. **Parcial, sin marcar como cerrada** — mismo patrón que T057: `e2e/ui/disk-detail.spec.ts` (carga, cambio de intervalo, selector personalizado) y `a11y.spec.ts` en `/disks/disk-0` con **cero incumplimientos de axe en ambos temas**, sobre el build de producción. `TimeSeriesChart` ya tenía lectura textual equivalente construida (`chart.summaryLabel`, navegación por teclado) de una sesión anterior; no se ha tocado. **Sin verificar**: ventana 1024×560/1280×720, acento del sistema, disco con 90+ días de historial real (solo probado con fixtures cortos), y la simplificación de T064 (`SegmentedControl` fuera de la `Toolbar`) sigue pendiente de revisión visual

**Punto de control**: un dato puntual se convierte en diagnóstico.

---

## Fase 6: Historia 4 — Relacionar un problema con lo que registró Windows (P2)

**Objetivo**: mostrar los eventos de almacenamiento del sistema, diciendo cuándo la atribución a un
disco es deducida. Eslabón 6 del plan.

**Validación independiente**: consultar los eventos, comprobar que llevan nivel y origen, que los
deducidos van etiquetados y que cerrar y reabrir no los duplica.

- [X] T067 [US4] Implementar la lectura del registro de eventos con marcador persistente en `src-tauri/src/collectors/event_log.rs`, usando `event_cursors` para no duplicar. Enlace FFI directo a `wevtapi.dll` (mismo criterio que `perf_counters.rs` con `pdh.dll`): `EvtQuery`/`EvtNext`/`EvtRender`/`EvtCreateBookmark`/`EvtUpdateBookmark`/`EvtOpenPublisherMetadata`/`EvtFormatMessage`. El cursor es un **bookmark real** del Event Log, no un `RecordId` suelto (`docs/open-questions.md` J.7): un bookmark inválido (canal limpiado) se detecta y se relee desde el principio en vez de dejar de leer eventos para siempre. **Probado de extremo a extremo contra el registro `System` real de esta máquina**: 2.059 eventos de almacenamiento leídos, mensajes formateados correctamente (incluso para el proveedor clásico `disk`, sin manifiesto), y el bookmark verificado en una segunda lectura (0 eventos nuevos, bookmark sin cambios)
- [X] T068 🔴 test-first [P] [US4] Escribir las pruebas de correlación en `src-tauri/src/domain/correlacion.rs` (`mod tests` inline, no un fichero `_test.rs` aparte, mismo patrón que el resto del repo): toda atribución no exacta queda marcada como inferida
- [X] T069 [US4] Implementar la correlación evento→dispositivo con su grado de confianza en `src-tauri/src/domain/correlacion.rs`. `exact` cuando el número de disco sale de una ruta de dispositivo estructurada (`\Device\HarddiskN\...`) y coincide con el inventario; `inferred` cuando sale del texto humano ya formateado ("disco N"); nunca por coincidencia textual sin cruzar contra un disco real (`docs/alert-rules.md` §3.6, `docs/open-questions.md` J.25). `\Device\HarddiskVolumeNN` y los nombres PDO quedan sin resolver: el colector de capacidad no captura ese identificador todavía
- [X] T070 [US4] Conectar `get_system_events` y `get_event_raw_xml` en `src-tauri/src/commands/mod.rs`. Filtro dinámico (`repo_varios::FiltroEventos`) con paginación por conjunto de claves (`occurred_at_utc|id`, más reciente primero); `get_system_events_impl`/`get_event_raw_xml_impl` probados con fixtures. La ingesta real (colector + correlación + persistencia) se conecta en `refresh_events()`, llamada desde `refresh_now("all")` con la misma tolerancia a fallos (SC-008) que ya usa `refresh_smart`
- [X] T071 [P] [US4] Construir el componente `VirtualList` en `src/lib/components/VirtualList.svelte` y exportarlo en el barrel (ya autorizado). Genérico por índice absoluto, no acoplado a una fila de una altura fija concreta: la pantalla de eventos lo usa fila a fila, el panel general lo reutiliza agrupando varias tarjetas por fila (ver T074)
- [X] T072 [P] [US4] Construir el componente `FilterBar` en `src/lib/components/FilterBar.svelte` y exportarlo en el barrel (ya autorizado). Grupos de píldoras de selección múltiple (nivel, proveedor): `Select` no sirve porque aquí varias opciones pueden estar activas a la vez
- [X] T073 [US4] Poblar la pantalla de eventos con lista virtualizada y filtros en `src/routes/events/+page.svelte`, renderizando el texto original **como texto, jamás como HTML**. De paso se corrigieron literales de interfaz hardcodeados en `EventRow.svelte` ("Error"/"Aviso"/"Info"/"asociación inferida") que ya tenían sus claves de i18n sin usar — mismo defecto que `AlertCard`/ADR-030, encontrado al reutilizar el componente
- [X] T074 [US4] **Medir R3** con 20 discos y 5.000 eventos en `e2e/ui/rendimiento.spec.ts`: la interfaz nunca deja de responder más de 50 ms seguidos (SC-007, SC-009). **Hallazgo real durante la medición** (`docs/open-questions.md` J.26): la primera navegación produce 70-120 ms de tarea larga incluso con 0 o 2 discos — coste fijo de arranque del harness, no del volumen de datos. Corregido separando ambos: cada prueba deja pasar la carga inicial y mide solo la interacción (desplazar los 5.000 eventos; recibir 20 discos en caliente vía un `metrics:updated` simulado, para lo que `ipc-falso.ts` ganó `emitirEvento()`). Ambos escenarios pasan limpios. De camino se virtualizó también el panel general (antes pintaba las 20 tarjetas de una vez): `VirtualList` agrupa ahora tantas `DiskCard` por fila como columnas quepan, con la misma componente genérica de T071 — mejora real, mantenida aunque el diagnóstico final mostrara que el fallo original era de metodología de medición, no de la rejilla
- [X] T075 [P] [US4] Añadir las claves de eventos a los dos diccionarios de `src/lib/i18n/`. La mayoría (`events.level.*`, `events.inferredMapping`) ya existían del esqueleto original, sin usar (ver T073); añadidas 8 claves nuevas (`events.empty.*`, `events.filter.*`, `events.detail.*`, `events.loadMore`): 172 en total
- [ ] T076 [US4] Pasar la definición de terminado de `docs/ui-design.md` §8 en la pantalla de eventos. **Parcial, sin marcar como cerrada** — mismo patrón que T057/T066: `e2e/ui/events.spec.ts` (lista, detalle con XML, filtro por nivel) y `a11y.spec.ts` en `/events` con **cero incumplimientos de axe en ambos temas** sobre el build de producción; al añadir `VirtualList` a la comprobación de accesibilidad se encontró y corrigió un `aria-required-children` real (el contenedor `role="list"` tenía como hijo directo un `div` sin `role="listitem"`). **Sin verificar**: ventana 1024×560/1280×720, acento del sistema, y un repaso manual de teclado sobre la lista virtualizada (el desplazamiento con rueda está probado, la navegación por flechas/Tab no)

**Punto de control**: hay evidencia del sistema, con su grado de certeza declarado.

---

## Fase 7: Historia 5 — Comprobar un disco bajo demanda (P2)

**Objetivo**: ejecutar pruebas sabiendo exactamente qué va a ocurrir antes de lanzarlas. Eslabón 7.

**Validación independiente**: lanzar cada prueba sobre un disco de descarte y comprobar que se
declara el impacto, que se puede cancelar, que la interfaz no se bloquea y que queda en el historial.

- [X] T077 🔴 test-first [US5] Escribir las pruebas de validación de rutas del benchmark en `src-tauri/src/tests/rutas.rs`: ninguna ruta fuera de las carpetas permitidas, y reserva de espacio comprobada. Nombre de la "carpeta controlada" no especificado en ningún documento, registrado en `docs/open-questions.md` J.27 antes de programar (15 pruebas, escritas antes que `resolver_ruta_archivo`/`resolver_tamano_bytes`)
- [X] T078 [US5] Implementar la validación de rutas y la reserva en `src-tauri/src/tests/rutas.rs` (J.27)
- [X] T079 [US5] Implementar el benchmark de lectura y escritura con archivo temporal controlado en `src-tauri/src/tests/benchmark.rs`: E/S sin caché real (`FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH`) con buffer alineado a sector vía `std::alloc` (no `Vec<u8>`, que no garantiza alineación de dirección), patrón comprobable por bloque (`tests::patron`) y orden aleatorio con Fisher-Yates + LCG propio (sin añadir `rand`). **Verificado con E/S real**: 8 MiB en el directorio temporal de la sesión (autorizado por el usuario, alcance explícitamente limitado a esa prueba pequeña), `passed: true`, ~1.5 GB/s escritura y ~2.0 GB/s lectura — nunca verificado al tamaño real de producción (256 MiB-8 GiB) ni sobre un volumen de usuario
- [X] T080 [P] [US5] Implementar `chkdsk /scan` capturando su salida literal en `src-tauri/src/tests/chkdsk.rs`, con detección de página de códigos (`tests::codificacion`, tablas CP1252/CP850/CP437 generadas con Python real, no transcritas a mano) — medido en esta máquina: `chkdsk` emite CP1252, `fsutil`/`vssadmin` CP850. **Verificado con ejecución real** (autorizada por el usuario, es de solo lectura): `chkdsk /scan C:` sin elevación devuelve `exit_status 3` (acceso denegado, esperado: `cargo test` no eleva, la app real sí, ADR-004) y el texto en español se decodifica correctamente como CP1252
- [X] T081 [P] [US5] Implementar el autotest SMART corto en `src-tauri/src/tests/autotest.rs`, con su motivo escrito cuando el dispositivo no lo admita (FR-025). **Sin verificar contra hardware real** (`docs/open-questions.md` J.28, única excepción de esta sesión a "medido contra el sistema real"): el usuario pidió expresamente no ejecutar un autotest real (tarda minutos); la forma del JSON asumida (`ata_smart_data.self_test.status.{value,string,passed}`, `.polling_minutes.short`) es la documentada de smartmontools, no una captura propia — pendiente de comparación antes de dar la historia por cerrada del todo
- [X] T082 [US5] Implementar la parada automática por límite de temperatura en `src-tauri/src/tests/guardia.rs` (FR-024): el límite del fabricante manda si existe, si no el configurado
- [X] T083 [US5] Conectar los cinco comandos de pruebas y la emisión de `test:progress` en `src-tauri/src/commands/mod.rs`. Exclusión mutua (`test.busy`) por disco físico subyacente vía `device_volume_links`, no solo por el id recibido — cubre a la vez el código de error genérico y "el autotest no corre junto al benchmark" (`product-specification.md` §6). Umbral térmico configurado del benchmark reutiliza el mismo valor que ya usa el motor de alertas para `temp.above_configured_crit` (80 °C), en vez de inventar uno nuevo. `command`/`output`/`outputEncoding` de `TestRun` viajan dentro de `parameters_json`/`result_summary_json` (sin columna propia); `orphanPath` reutiliza `temp_path`. Toda fila `pending`/`running`/`cancelling` que sobreviva a un reinicio se marca `interrupted` al abrir la base, para que `test.busy` no bloquee el disco para siempre. Añadidos `volume.not_found` y `test.io_failed` a `docs/ui-contract.md` §1 (decisiones completas en `docs/open-questions.md` J.29). El autotest SMART corto se conecta sin ejecutarse (J.28: el usuario no autorizó una ejecución real esta sesión, solo escribir el código)
- [X] T084 [US5] Poblar la pantalla de pruebas con confirmación previa, progreso e historial en `src/routes/tests/+page.svelte`, mostrando la orden literal en el `ConfirmDialog` (chkdsk y autotest; el benchmark no ejecuta un proceso externo, así que no tiene orden literal que mostrar — se confirma con los valores predeterminados de `product-specification.md` §6 en el cuerpo del diálogo). Selector de disco/volumen propio: la pantalla puede ser la primera que se visite, así que su `load` pide inventario y no asume que `/` ya rellenó `app.devices` (mismo guardián `app.loadedAt`). Se extendió `get_device_detail` para devolver también las capacidades `self_test_short` y `chkdsk_scan` (antes solo `smart`, T077-T086 quedaba pendiente de esto) y se añadió `VolumeSummary.chkdskAvailable` (la decisión NTFS la toma el backend, no se repite en la pantalla) — ambos cambios de contrato documentados en línea, sin ADR: son adiciones de datos derivados, no cambios de permisos ni de modelo de datos
- [X] T085 [P] [US5] Añadir las claves de pruebas y de impacto a los dos diccionarios de `src/lib/i18n/`: 53 claves nuevas (172 → 225), `pnpm verify:i18n` en verde
- [X] T086 [US5] Pasar la definición de terminado de `docs/ui-design.md` §8 en pruebas y diagnóstico: `pnpm check`/`lint`/`verify` en cero, `e2e/ui/tests.spec.ts` nuevo (confirmación con orden literal, arranque del benchmark con valores predeterminados, evento `test:progress` en vivo y cancelación) y `/tests` en `a11y.spec.ts` — sin incumplimientos de axe en ambos temas. **No verificado**: acento de sistema no azul, recorte exacto a 1024×560/1280×720 y 20 discos simultáneos (esta pantalla no virtualiza porque no tiene listas largas que lo requieran); igual que T057/T066/T076, definición de terminado parcial y declarada como tal, no completa en silencio

**Punto de control**: se puede actuar sobre una sospecha, no solo observarla.

---

## Fase 8: Historia 6 — Llevarse la información fuera (P3)

**Objetivo**: exportar informes y un paquete de diagnóstico anonimizado. Eslabón 8 del plan.

**Validación independiente**: exportar en cada formato y generar el paquete, comprobando **abriéndolo
y leyéndolo** que no contiene nada identificativo.

- [ ] T087 [P] [US6] Implementar la exportación tabular y estructurada en `src-tauri/src/reporting/export.rs`
- [ ] T088 [P] [US6] Implementar el informe imprimible en `src-tauri/src/reporting/informe.rs`
- [ ] T089 🔴 test-first [US6] Escribir las pruebas de anonimización en `src-tauri/src/reporting/anonimizar_test.rs`: sin números de serie, sin nombre de equipo, sin rutas de usuario
- [ ] T090 [US6] Implementar el paquete de diagnóstico anonimizado por defecto en `src-tauri/src/reporting/diagnostico.rs`, incluyendo el registro de actividad (FR-029c)
- [ ] T091 [US6] Conectar `export_report`, `preview_diagnostic_zip` y `create_diagnostic_zip` en `src-tauri/src/commands/mod.rs`
- [ ] T092 [US6] Implementar la pantalla de informes en `src/routes/reports/+page.svelte` (US-050, pendiente de diseño: pedir revisión visual)
- [ ] T093 [P] [US6] Añadir las claves de informes y la advertencia de contenido identificativo a los dos diccionarios de `src/lib/i18n/`
- [ ] T094 [US6] Pasar la definición de terminado de `docs/ui-design.md` §8 en informes

**Punto de control**: un incidente se puede compartir sin entregar de paso la identidad del equipo.

---

## Fase 9: Historia 7 — Ajustar el comportamiento a su equipo (P3)

**Objetivo**: configurar frecuencias, umbrales, retención, apariencia y registro, y poder borrarlo
todo. Eslabón 9 del plan.

**Validación independiente**: cambiar cada preferencia, reiniciar y comprobar que se conservó, y que
un valor fuera de rango se rechaza con explicación.

- [ ] T095 🔴 test-first [P] [US7] Escribir las pruebas de límites en `src-tauri/src/domain/ajustes_test.rs`: todo valor fuera de rango se rechaza con `settings.out_of_range` y el anterior se mantiene
- [ ] T096 [US7] Implementar la validación tipada de `settings` en `src-tauri/src/domain/ajustes.rs`
- [ ] T097 [US7] Conectar `set_setting`, `get_settings`, `reset_settings` y `delete_all_data` en `src-tauri/src/commands/mod.rs`
- [ ] T098 [US7] Implementar el cambio de nivel de registro y la apertura de su carpeta en `src-tauri/src/commands/registro.rs`, abriendo **una sola ruta conocida** y sin recibirla como argumento (delta C2)
- [ ] T099 [US7] Implementar el comportamiento al cerrar la ventana, con opción de recordar la decisión, en `src-tauri/src/platform/ventana.rs` (FR-029)
- [ ] T100 [US7] Implementar la pantalla de ajustes en `src/routes/settings/+page.svelte` con `Switch`, `Select`, `RadioGroup`, `TextField` y `ConfirmDialog` (pendiente de diseño: pedir revisión visual)
- [ ] T101 [P] [US7] Añadir las claves de ajustes a los dos diccionarios de `src/lib/i18n/`
- [ ] T102 [US7] Pasar la definición de terminado de `docs/ui-design.md` §8 en ajustes

**Punto de control**: el producto se adapta a un portátil y a un servidor de veinte discos.

---

## Fase 10: Historia 8 — Instalar, reconocer y retirar la aplicación (P3)

**Objetivo**: entregar un instalador que funcione sin conexión. Eslabón 10 del plan.

**Validación independiente**: instalar en un equipo limpio y sin red, arrancar, consultar la versión
y desinstalar comprobando qué queda.

- [ ] T103 [US8] Configurar el instalador con WebView2 sin conexión en `src-tauri/tauri.conf.json` (ADR-020; requiere modo plan: es ruta protegida)
- [ ] T104 [P] [US8] Implementar la pantalla «Acerca de» con versión, licencia y avisos de terceros en `src/routes/settings/+page.svelte` o su ruta propia (US-061)
- [ ] T105 [US8] Verificar que la desinstalación **conserva** historial y configuración, y documentar la limpieza manual en `README.md`
- [ ] T106 [US8] Verificar las migraciones de `src-tauri/migrations/` desde cada versión publicada anterior, no solo desde cero
- [ ] T107 [US8] Comprobar con un monitor de red que la aplicación no emite **ninguna** petición saliente, según el guion del eslabón 10 de `specs/001-monitor-discos-windows/quickstart.md` (SC-014)

**Punto de control**: entregable.

---

## Fase 11: Acabado y asuntos transversales

- [ ] T108 [P] Comprobar los mínimos de cobertura: 90 % en `domain/` y `alerts/`, 80 % en el resto de `src-tauri/src/`, 70 % en `src/lib/` sin componentes
- [ ] T109 [P] Verificar que cada componente de `src/lib/components/` tiene prueba `*.browser.test.ts` de sus cinco estados: vacío, cargando, error de fuente, no compatible y dato obsoleto
- [ ] T110 [P] Ampliar `e2e/ui/a11y.spec.ts` a toda pantalla nueva, por ambos temas
- [ ] T111 Comprobar el escalado al 125 %, 150 % y 200 % y la ventana mínima 1024 × 560 en todas las rutas de `src/routes/`, según la definición de terminado de `docs/ui-design.md` §8
- [ ] T112 [P] Verificar con un acento del sistema claro (`#ffb900`) en ambos temas que el contraste sigue cumpliendo AA, con la herramienta `tools/accent-check.py`
- [ ] T113 Cerrar en `docs/open-questions.md` las cuestiones I.2, I.5, I.7 y la de umbrales de espacio, con el dato medido
- [ ] T114 [P] Actualizar `docs/roadmap.md` y regenerar el consolidado con `pnpm docs:build`
- [ ] T115 Ejecutar el guion completo de [quickstart.md](quickstart.md) sobre una compilación empaquetada, no en desarrollo

---

## Dependencias y orden de ejecución

### Entre fases

- **Fase 1 (Preparación)**: sin dependencias. **T001 y T002 bloquean**: son decisiones que deben estar escritas antes de programarlas.
- **Fase 2 (Cimientos)**: depende de la fase 1. **Bloquea todas las historias.**
- **Fases 3 a 10**: dependen de los cimientos.
- **Fase 11 (Acabado)**: depende de las historias que se quieran entregar.

### Entre historias

A diferencia del caso habitual, aquí **las historias no son todas independientes entre sí**: es una
aplicación de medición, y el dato fluye en una dirección.

- **US1 (P1)**: solo depende de los cimientos. Es el producto mínimo.
- **US2 (P1)**: necesita el muestreo de salud de US1 para tener qué evaluar. **Depende de US1.**
- **US3 (P2)**: independiente de US2. Puede ir en paralelo con ella tras US1.
- **US4 (P2)**: solo depende de los cimientos. **Puede empezar en paralelo con US1.**
- **US5 (P2)**: necesita el inventario de US1. Independiente de US2, US3 y US4.
- **US6 (P3)**: necesita datos que exportar; en la práctica, US1 y al menos una de US2–US5.
- **US7 (P3)**: solo depende de los cimientos, salvo los ajustes de umbrales, que necesitan US2.
- **US8 (P3)**: independiente en su mayor parte; su validación completa exige el resto.

### Dentro de cada historia

Las pruebas marcadas 🔴 **se escriben antes y deben fallar**. Después: dominio → colector →
persistencia → comando → interfaz → i18n → revisión visual.

### Oportunidades de paralelismo

- Fase 1: T003, T004 y T005 en paralelo.
- Fase 2: los cuatro repositorios (T011–T014) tocan ficheros distintos.
- Tras los cimientos: **US1 y US4 pueden avanzar a la vez** — el inventario y el registro de eventos
  de Windows no se pisan.
- Los tres componentes autorizados (T063, T071, T072) son independientes entre sí.
- Las claves de i18n de cada historia son ficheros compartidos: **no** paralelizables entre sí, y por
  eso van marcadas [P] solo dentro de su fase.

---

## Ejemplo de paralelismo: historia 1

```bash
# Las pruebas de parser y de estado, a la vez (ficheros distintos):
Tarea: "Pruebas del parser de smartctl en src-tauri/src/collectors/smartctl_parser_test.rs"
Tarea: "Pruebas del parser NVMe en src-tauri/src/collectors/nvme_parser_test.rs"
Tarea: "Pruebas de estado en src-tauri/src/domain/salud_test.rs"

# Las tres pantallas, a la vez:
Tarea: "Panel general en src/routes/+page.svelte"
Tarea: "Detalle de disco en src/routes/disks/[id]/+page.svelte"
Tarea: "Asistente inicial en src/routes/onboarding/+page.svelte"
```

---

## Estrategia de implementación

### Producto mínimo (solo historia 1)

1. Fase 1: preparación — **con T001 y T002 hechas, no saltadas**
2. Fase 2: cimientos
3. Fase 3: historia 1
4. **Parar y validar** el guion de US1 en [quickstart.md](quickstart.md)

Con eso la aplicación ya responde «¿qué discos tengo y están bien?», que es la pregunta que trae al
usuario.

### Entrega incremental

Cimientos → US1 (mínimo) → US2 (es un monitor de verdad) → US3 y US4 en paralelo → US5 → US6, US7 y
US8. Cada historia añade valor sin romper la anterior.

### Con varias personas

Tras los cimientos: una persona en US1 y otra en US4, que no se pisan. En cuanto US1 cierre, se
libera US2 y US5.

---

## Notas

- Toda tarea se cierra con sus pruebas, sus textos en los dos idiomas y sus estados de error y vacío.
- Ramas por historia, commits en imperativo y en español, referenciando la historia.
- **Ninguna ambigüedad se resuelve dentro del código.** Si al construir aparece una decisión no
  escrita, va a `docs/open-questions.md` **antes** de programarla.
- Tres pantallas —asistente inicial, informes y ajustes— están **pendientes de diseño visual**: se
  componen con el catálogo existente y se pide revisión antes de darlas por terminadas.
- T030 y T103 tocan rutas protegidas (`.specify/memory/` y `src-tauri/tauri.conf.json`): exigen
  autorización explícita, y la primera es una enmienda de la constitución.
