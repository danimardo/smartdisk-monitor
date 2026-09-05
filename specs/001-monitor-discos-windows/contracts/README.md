# Fase 1 — Contratos

**Funcionalidad**: SmartDisk Monitor 1.0 · **Fecha**: 2026-09-04 · **Plan**: [../plan.md](../plan.md)

## El contrato canónico está en `docs/ui-contract.md`

> **Este directorio no lo reproduce, y es deliberado.** El contrato entre interfaz y backend es
> normativo y ya existe: firmas de los 31 comandos, tipos compartidos, catálogo de códigos de error,
> ocho eventos emitidos por el backend y la política de permisos de Tauri. Copiarlo aquí crearía dos
> versiones que divergirían en silencio —el fallo que ADR-029 acaba de cerrar en este repositorio—,
> y con un agravante: el contrato es lo único que impide que backend e interfaz se separen sin que
> nadie se entere. **Ante cualquier discrepancia, manda `docs/ui-contract.md`.**

Lo que sí aporta este fichero: el **estado real de avance** comando a comando, las **reglas de la
frontera** que la implementación debe respetar, y el **delta** que introducen las aclaraciones.

| Necesitas | Ve a |
|---|---|
| Forma de los errores y códigos estables | `docs/ui-contract.md` §1 |
| Tipos compartidos (DTO) | `docs/ui-contract.md` §2 |
| Firma de cada comando | `docs/ui-contract.md` §3 |
| Eventos que emite el backend | `docs/ui-contract.md` §4 |
| Permisos de Tauri | `docs/ui-contract.md` §5 |

## La lista es cerrada

La interfaz **solo** puede invocar comandos enumerados en `docs/ui-contract.md` y registrados en
`src-tauri/src/lib.rs`. No existe comando genérico, ni shell, ni sistema de ficheros abierto: la
interfaz nunca construye órdenes de `smartctl` ni rutas. Un comando nuevo exige actualizar el
contrato **antes** de escribirlo, y un permiso nuevo exige su ADR.

Regla complementaria del lado de la interfaz: **ninguna pantalla llama a `invoke` directamente**.
Todo pasa por `src/lib/api/`, que es donde vive la validación de la frontera.

## Estado de avance

31 comandos declarados.

| Estado | Nº | Qué significa |
|---|---|---|
| Devuelven `not_implemented` | 19 | Dicen en voz alta lo que falta, que es lo correcto mientras no haya fuente |
| Devuelven vacío estructurado | 3 | `get_alert_groups`, `get_system_events`, `get_test_runs`: forma correcta, contenido vacío |
| Operativos con datos reales | 4 | `get_devices`, `get_device_detail`, `set_device_monitoring`, `set_device_alias` — leen y escriben inventario y salud SMART persistidos |
| `refresh_now` | 1 | `"all"` y `"device"` conectados (reconciliación de inventario + lectura SMART); ningún otro ámbito existe en el contrato |
| Operativos sin datos de disco | 4 | `get_appearance_settings`, `get_system_accent_color`, `get_app_info`, `log_from_ui`, `get_log_level` |

**`sources` sigue vacío en todas las respuestas**: el seguimiento de estado por fuente (T020/T021,
degradar una tarjeta sin tumbar la aplicación) no existe todavía. Un fallo de `smartctl` en un disco
se registra (`tracing::warn!`) y se salta ese disco, pero la interfaz no lo distingue hoy de «este
disco no tiene smartctl_path» — ambos casos devuelven la misma forma sin marca de fallo.

Los tres que devuelven vacío merecen atención: una lista vacía es indistinguible de «no hay
alertas/eventos/pruebas». Al conectarlos con su fuente real hay que asegurar que un fallo de
recopilación se distingue de una lista vacía legítima.

## Qué comando enciende cada pantalla

Correspondencia entre el orden de construcción del plan y lo que el usuario ve funcionar:

| Eslabón | Comandos que dejan de ser marcador | Pantalla que se enciende |
|---|---|---|
| 2 · Inventario | `get_devices`, `get_device_detail`, `set_device_monitoring`, `set_device_alias`, `refresh_now("all")` | Panel general, detalle de disco, asistente inicial |
| 3 · Salud | `get_device_detail` con contadores reales, `refresh_now("device", id)` | Detalle de disco |
| 4 · Métricas | `get_metric_series` | Gráficas |
| 5 · Alertas | `get_alert_groups`, `get_alert_detail`, `acknowledge_alert`, `mute_alert`, `unmute_alert`, `archive_alert`, `pause_monitoring`, `resume_monitoring` | Alertas y bandeja |
| 6 · Eventos | `get_system_events`, `get_event_raw_xml` | Eventos |
| 7 · Pruebas | `start_benchmark`, `run_chkdsk_scan`, `run_smart_short_test`, `cancel_test`, `get_test_runs` | Pruebas y diagnóstico |
| 8 · Informes | `export_report`, `preview_diagnostic_zip`, `create_diagnostic_zip` | Informes |
| 9 · Ajustes | `set_setting`, `delete_all_data` | Ajustes |

## Reglas de la frontera

Vinculantes, y su incumplimiento es un defecto, no una preferencia:

- **Todo lo que cruza se valida en ejecución con Zod**, en `invoke`, en `listen` y al leer ficheros.
  Cada esquema tiene su prueba de rechazo. Un cambio de forma en el backend se detecta al entrar, no
  tres pantallas más tarde en forma de `undefined`.
- **Ninguna aserción de tipo sobre datos externos.** `as` sobre el resultado de `invoke` es
  precisamente lo que la validación existe para evitar, y lo comprueba `pnpm verify:boundaries`.
- **Todo fallo cruza como `AppError`**: código estable, clave i18n y detalle técnico conservado. La
  interfaz muestra la frase humana y guarda el detalle en un desplegable.
- **Un comando que falla deja la interfaz como estaba.** Nada de estados a medias.
- **El backend empuja; la interfaz no sondea.** Ningún `setInterval` para pedir datos. Los
  temporizadores de presentación —refrescar un «hace 2 min»— sí son legítimos, y se limpian.
- **El estado inicial llega por `load` en `+page.ts`**, no por `onMount`. Las actualizaciones vienen
  después por eventos.
- **La interfaz no aplica un cambio por su cuenta**: invoca el comando y espera el evento que
  confirma. Esto evita que la pantalla afirme algo que el backend no llegó a hacer.

## Delta de esta funcionalidad

Dos cambios sobre el contrato canónico, nacidos de las aclaraciones del 2026-09-04. **Ninguno se
implementa antes de estar reflejado en `docs/ui-contract.md`.**

### C1 · Estado de escritura de historial detenida

FR-020a/b exige que la interfaz pueda saber que se ha dejado de escribir historial, para decirlo.
Encaja de forma natural junto a la salud de las fuentes (`sources`), que ya viaja en `get_devices` y
en el evento `metrics:updated`: es información del mismo tipo —qué está funcionando y qué no—, y así
no se añade un evento nuevo.

### C2 · Modo detallado de registro y acceso a su carpeta

FR-029a/b necesitan poder consultar y cambiar el nivel de registro, y abrir su carpeta.
`get_log_level` ya existe y es operativo; falta su contrapartida de escritura y la acción de abrir la
carpeta.

La acción de abrir carpeta **no** puede resolverse con un sistema de ficheros abierto ni con una
shell genérica: sería un permiso nuevo de Tauri, exigiría ADR, y ampliaría la superficie de un
binario privilegiado que se distribuye a terceros. Debe ser un comando específico que abra **una sola
ruta conocida**, la del registro, sin recibirla como argumento desde la interfaz.

## Puerta de DTO generados: activa

Se activó junto con el primer DTO real (T029-T030). `ts-rs` genera 16 tipos en
`src/lib/api/generated/`; `src/lib/api/types.ts` los reexporta en vez de duplicarlos, y lo que aún
no tiene comando conectado sigue a mano con la misma advertencia. `cargo test` regenera los
ficheros; una diferencia con lo versionado bloquea la integración, exactamente como preveía la
constitución antes de esta corrección (pendiente de que la enmienda formal se apruebe).
