# Reglas de alerta

Especificación normativa del motor de alertas. Sustituye a la prosa de
`product-specification.md` §5, que queda como resumen. Si esta tabla y aquel texto discrepan, manda
esta tabla.

Referencias: `docs/data-model.md` (`alert_groups`, `alert_occurrences`), `docs/open-questions.md`
§B, `src/lib/design/health.ts`.

---

## 1. Vocabulario

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

## 2. Tabla de reglas

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

### Lo que explícitamente NO genera alerta

- Un dispositivo que **declara** no soportar SMART (`unsupported`): es normalidad. Aparece en gris.
- Un contador que falta: `unavailable` no es `0` y no se evalúa.
- Un evento informativo de Windows: se muestra en la cronología, no crea grupo. En particular
  `Microsoft-Windows-Ntfs` 98, que dice que el volumen **está bien** y aparece cientos de veces.
- Los eventos de `Volsnap` (instantáneas VSS) y `volmgr` 161 (volcado de memoria): no hablan de la
  salud del disco.
- Un `disk` 51 aislado, o cualquier evento sobre un medio extraíble que se acaba de desconectar:
  véase §3.3 y §3.5.
- La primera lectura de un contador acumulativo: sin lectura previa no hay incremento que medir.

### Cooldown de notificación

El cooldown suprime la **notificación**, no la ocurrencia: la cronología del grupo lo registra todo.
Las reglas de capacidad son las más restrictivas por exigencia de spec §5: solo notifican al cruzar
un umbral, al cambiar de nivel, o al recaer tras haberse recuperado. Nunca una notificación por
muestra.

---

## 3. Proveedores y eventos de Windows vigilados

Lista verificada el 2026-09-04 contra los manifiestos de proveedor de un Windows 11 real y contra
180 días de su registro `System` (2.038 eventos de almacenamiento sobre 32.620 totales). Sustituye a
la lista tentativa anterior, que **clasificaba mal varios eventos** (véase §3.4). Se guarda en
`settings` para poder ampliarla sin recompilar.

### 3.1 Proveedores: dos familias que se consultan distinto

| Familia | Proveedores | Cómo se enumeran |
|---|---|---|
| **Clásicos** (sin manifiesto) | `disk`, `Ntfs`, `volmgr`, `volsnap`, `storahci`, `stornvme`, `partmgr`, `iaStor*` | No declaran eventos: `Get-WinEvent -ListProvider` devuelve cero. Sus mensajes viven en el binario del controlador. Hay que filtrar por proveedor e id, no descubrirlos |
| **Con manifiesto** | `Microsoft-Windows-Ntfs`, `Microsoft-Windows-NvmeDisk`, `Microsoft-Windows-StorageSpaces-Driver`, `Microsoft-Windows-StorPort`, `Microsoft-Windows-ReFS` | Declaran sus eventos con nivel y plantilla, y se pueden enumerar |

**`Microsoft-Windows-Disk` no sirve para alertas.** Declara 22 eventos y todos son informativos: son
trazas de E/S ("Distribuyendo una solicitud de lectura"), no diagnósticos. Los errores de disco
vienen del proveedor clásico `disk`.

### 3.2 Eventos vigilados

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

### 3.3 Lo que NO genera alerta, y por qué importa

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

### 3.4 Errores de la lista anterior, corregidos

La lista tentativa que traía la especificación tenía fallos que solo se ven con datos reales:

| Antes | Realidad |
|---|---|
| `Ntfs` 98 → "metadatos inconsistentes", **crítico** | Es `Microsoft-Windows-Ntfs` 98, de nivel **Información**, y significa que el volumen está bien. Habrían sido 319 falsos críticos |
| `disk` 51 y 52 → **crítico** | 51 es ruido de fondo masivo (839 casos). Advertencia con umbral de frecuencia |
| `Ntfs` 130 → "marcado para comprobación" | 130 es "se **reparó** la estructura". El que indica daño irreparable es el **131**, que faltaba |
| `volmgr` 46, 49 | No aparecen en el sistema. El que sí aparece, 161, no habla de salud del disco |
| — | Faltaban `disk` **157** (extracción imprevista, que es justo el evento que necesita la regla `device.removed_unexpected`), `disk` 158, `Ntfs` 50, `Microsoft-Windows-Ntfs` 140 y el proveedor `Microsoft-Windows-NvmeDisk` entero |

### 3.5 Los eventos llegan en ráfagas correlacionadas

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

### 3.6 Asociación evento → disco

Puede ser `exact` (el evento nombra el dispositivo y hay correspondencia inequívoca), `inferred` (se
deduce por volumen o por controladora) o `unknown`. Una inferencia **nunca** se presenta como
certeza: `EventRow` la etiqueta.

Ojo con los identificadores que aparecen en los mensajes: conviven formas como `\Device\Harddisk1\DR18`,
`\DR39`, "disco 1", `\Device\HarddiskVolume23` y nombres PDO como `\Device\0003d2a5`. **No son
intercambiables**, y el número que sigue a `DR` no es el número de disco físico. La resolución se
hace contra el inventario, nunca por coincidencia textual.

### 3.7 Deduplicación entre sesiones

Se persiste un *bookmark* del canal, no un `RecordId` suelto: al limpiar un canal los identificadores
se reinician, y un cursor numérico se quedaría por delante de los eventos nuevos y dejaría de
importarlos sin dar ningún error. La identidad de un evento es `(canal, RecordId)`, no su fecha, de
modo que un cambio del reloj del sistema tampoco produce duplicados.

### 3.8 Alcance de esta verificación

Los datos vienen de **un** equipo Windows 11 x64 en español, con discos internos NVMe y SATA y uso de
discos externos USB. Sirve para saber qué es ruido de fondo y qué identificadores existen de verdad,
que era lo que faltaba. **No** cubre servidores, RAID por hardware ni Storage Spaces en producción:
esos eventos están tomados de los manifiestos, no observados, y siguen pendientes de contraste en la
Fase 0. Un equipo sano no produce eventos de fallo real, así que la ausencia de `Ntfs` 55 o `disk` 7
en la muestra es una buena noticia, no una señal de que no existan.

## 4. Textos

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

## 5. Pruebas exigidas

Toda regla de la tabla §2 necesita, como mínimo:

1. Un test de activación con datos fixture reales (ATA, NVMe, USB, RAID, VM).
2. Un test de **no** activación en el caso "dato ausente": nunca se alerta por falta de dato.
3. Un test de histéresis: oscilar alrededor del umbral no debe producir más de un grupo.
4. Un test de deduplicación: N evaluaciones equivalentes producen 1 grupo y N ocurrencias.
5. Un test de ciclo: resolver y recaer incrementa `cycle` y conserva el contador histórico.
