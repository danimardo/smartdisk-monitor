# Fase 0 — Investigación y decisiones

Feature: Puente del registro de eventos de Windows al motor de alertas.
Todas las decisiones que la especificación dejó para el plan o que surgen del código real.

Recordatorio del flujo (constitución, «Flujo de desarrollo» §1): cada decisión que no estuviera ya
escrita se lleva a `docs/open-questions.md` con su valor propuesto **antes** de programarla. Este
fichero es el borrador de esas entradas.

---

## D1 · ¿Migración de SQLite, o basta con la columna que ya existe?

**Decisión**: **sin migración nueva.** `alert_occurrences.triggering_event_id INTEGER REFERENCES
system_events (id) ON DELETE SET NULL` ya está en `migrations/0001_esquema_inicial.sql` y hoy nunca
se escribe (`repo_alertas` lo lee pero los tres `INSERT INTO alert_occurrences` lo omiten). Esta
funcionalidad empieza a rellenarlo. `alert_groups` admite `target_device_id`/`target_volume_id`
nulos y `context_json`, suficiente para las reglas sin objeto (Q2 → A) y para la pareja de discos de
`inventory.duplicate_id` (va en `context` de la clave de deduplicación, texto, igual que
`provider:event_id`).

**Razón**: el esquema se diseñó ya previendo la Historia 4. Abrir una migración por columnas que
existen sería trabajo sin valor y contra el principio de simplicidad (§II.5).

**Alternativas descartadas**:
- *Tabla nueva `alert_burst_events`* para la ráfaga: no hace falta. La cronología del grupo
  (`alert_occurrences`, una fila por evento con su `triggering_event_id` y su `occurred_at_utc`) ya
  reconstruye la secuencia (FR-010).
- *Columna `alert_groups.last_event_at_utc`* para la histéresis temporal: `last_occurrence_at_utc`
  ya cumple ese papel (se actualiza en cada `record_occurrence`).

**Pendiente de confirmar en implementación**: que escribir `triggering_event_id` no rompe el
recuento de ocurrencias de ninguna prueba existente (mismo tipo de hallazgo que `smart_query_ok` en
`build_smart_counters`).

---

## D2 · Señal de «extracción imprevista» para `device.removed_unexpected`

**Contexto**: `reconciliar_inventario` ya detecta las bajas (`ids_dados_de_baja`) y deja un comentario
explícito: «La clasificación de "expulsión segura" frente a "retirada sin aviso" (T026) exige el
colector de eventos de la Historia 4». La tabla de `alert-rules.md` §2 da como fuente
«inventario + `disk` 157».

**Decisión**: la regla se activa por **cualquiera** de estas dos vías, unificadas por la ventana de
correlación de 60 s:

1. **Llega un evento `disk` 157** («El disco N se ha extraído de forma imprevista»), correlacionado
   a un disco del inventario. Es la señal directa y la que la propia observación real registró 63
   veces.
2. **Un disco monitorizado desaparece del inventario** (`ids_dados_de_baja`) y **no** es USB. Un
   disco fijo no desaparece en operación normal; su ausencia es, por sí misma, imprevista. Si es
   USB, se aplica la vía 1 (solo alerta —advertencia— si hay `disk` 157); una baja USB silenciosa
   se trata como retirada esperada y **no** alerta.

Severidad: crítica, salvo `bus_type == "USB"` → advertencia (tal cual la tabla).

La correlación de 60 s evita el doble grupo cuando ambas vías se disparan por el mismo suceso: si en
la ventana hay un `disk` 157 **y** una baja de inventario del mismo disco, es un único grupo
`device.removed_unexpected` (el `disk` 157 es la causa, FR-009).

Resolución: el disco reaparece en el inventario con el mismo `fingerprint` (tabla §2).

**Razón**: Windows no deja un rastro fiable y único de «expulsión limpia» (no hay un id de evento
equivalente a 157 para el caso bueno). Apoyarse en «fijo que desaparece = imprevisto» es la lectura
honesta: es cierto en la práctica y no inventa una certeza que no tenemos. El caso USB, más
ambiguo, se queda del lado prudente (solo alerta con señal explícita).

**Alternativas descartadas**:
- *Solo `disk` 157*: se perdería la baja de un disco SATA interno cuyo controlador no emita el 157
  (algunos no lo hacen), justo el caso más grave.
- *Toda baja alerta, USB incluido*: produciría una alerta cada vez que alguien quita legítimamente
  un disco externo. Ruido garantizado.
- *Buscar un evento de expulsión limpia y alertar en su ausencia*: no existe ese evento de forma
  fiable; construir la regla sobre algo que no se observa es una apuesta (§Flujo de desarrollo §6).

**A `open-questions.md`**: sí, entrada nueva en §J.

---

## D3 · Dónde vive la tabla `(proveedor, id) → regla`

**Decisión**: **en código**, en un módulo nuevo `src-tauri/src/alerts/reglas_eventos.rs`, como una
tabla estática que refleja `docs/alert-rules.md` §3.2 fila por fila. La lista de **proveedores**
vigilados (`PROVEEDORES_VIGILADOS`) se mantiene donde está (`event_log.rs`) para el filtro de
ingesta; la clasificación por id (qué regla, qué severidad base, qué umbral de frecuencia) es lógica
de dominio y va con el motor.

**Razón**: `alert-rules.md` es normativo y esta tabla lo **implementa**; tenerla en código permite
que `cargo test` verifique que cada fila de §3.2 tiene su entrada (prueba de completitud, análoga a
`las_reglas_en_alcance_tienen_titulo_y_resumen_propios`). El `TODO` de `event_log.rs` sobre mover
`PROVEEDORES_VIGILADOS` a `settings` es para **ampliar la ingesta sin recompilar**; la semántica de
una regla (severidad, histéresis) no es configuración de usuario y no debe vivir en `settings`.

**Alternativas descartadas**:
- *En `settings` como JSON*: convertiría la definición normativa de una regla en dato mutable sin
  validación de esquema fuerte; contra el principio XI y contra «`alert-rules.md` es la fuente de
  verdad».
- *Repartida por función de evaluación*: cada `evaluar_*` con su propio `match` de ids. Se duplica
  el conocimiento y la prueba de completitud se vuelve imposible.

**Forma prevista** (detalle en data-model.md): `&[(proveedor, &[ids], rule_key, severidad_base,
objetivo, ventana_frecuencia_opcional)]`.

---

## D4 · Ventana de correlación de 60 s a caballo entre dos ciclos

**Contexto**: `refresh_events` corre cada 30 s (`EVENTOS_WINDOWS`). Una ráfaga real dura segundos,
pero puede quedar partida: el `disk` 157 en el lote del ciclo N, el `Ntfs` 50 derivado en el ciclo
N+1.

**Decisión**: la correlación **no** se hace solo sobre el lote del ciclo. Al evaluar los eventos
nuevos de un ciclo, para cada evento se consulta `system_events` los eventos **del mismo disco (o
sin disco, por proximidad temporal) en los 60 s anteriores** a su `occurred_at_utc`, ya persistidos.
Así:

- Si el `disk` 157 llegó en el ciclo anterior y ya creó su grupo `device.removed_unexpected`, el
  `Ntfs` 50 del ciclo actual, al ver un `disk` 157 en su ventana de 60 s, se registra como
  **ocurrencia de ese grupo** en vez de crear `events.delayed_write`.
- Si el `disk` 157 llega **después** que un evento derivado (orden invertido en el registro), el
  derivado ya habrá creado su propio grupo. Al procesar el `disk` 157 y detectar en su ventana un
  grupo de evento reciente del mismo disco, esos grupos derivados se **reasignan**: se marcan
  resueltos con nota de «absorbido por la extracción imprevista» y sus eventos se re-registran como
  ocurrencias del `device.removed_unexpected`. (Caso poco frecuente; se cubre con una prueba.)

**Razón**: la ventana es tiempo de reloj (`alert-rules.md` §2, «60 s»), no ciclos. Consultar
`system_events` hacia atrás es barato (índices `idx_system_events_time` y `idx_system_events_device`
ya existen) y es la única forma correcta con un colector que va a lotes.

**Alternativas descartadas**:
- *Buffer en memoria de los últimos 60 s de eventos*: se pierde al reiniciar y complica el ciclo de
  vida largo (§XIV). La base ya tiene los eventos.
- *Ignorar el cruce de ciclos*: produciría el doble grupo justo en el escenario que motivó la
  ventana (P.5). Inaceptable.
- *Reducir `EVENTOS_WINDOWS` a < 60 s para que una ráfaga quepa en un ciclo*: no garantiza nada (una
  ráfaga en el borde del ciclo sigue partida) y aumenta el coste de sondeo.

**A `open-questions.md`**: sí — la mecánica de «mirar 60 s hacia atrás en `system_events`» y el caso
de reasignación por orden invertido.

---

## D5 · Cómo llega el evento disparador al detalle de la alerta

**Contexto**: la spec (FR-016, SC-004) pide que desde el detalle de una alerta de evento se pueda
llegar al evento del sistema que la originó. `alert_occurrences` ya guardará `triggering_event_id`;
falta exponerlo.

**Decisión**: el DTO de ocurrencia que viaja a la interfaz (el que alimenta la cronología del
detalle de alerta) gana un campo `triggeringEventId: number | null`. El detalle de alerta, para las
filas de cronología que lo tengan, muestra un enlace **`<a href="/events?...">`** que abre la
pantalla de sucesos con ese evento resaltado/filtrado. No se duplica el contenido del evento dentro
del detalle de alerta: se navega a donde ya vive, con su XML crudo y su etiqueta de certeza (que la
pantalla de eventos ya renderiza, T073).

**Razón**: reutiliza la pantalla de eventos existente (que ya cumple la definición de terminado de
interfaz), respeta «navegación por enlaces, no `goto()`» (§XIV) y no mete el renderizado de texto de
evento en dos sitios. El resaltado por id necesita que `/events` acepte un parámetro de consulta
para centrar un evento — adición pequeña a esa ruta.

**Alternativas descartadas**:
- *Incrustar el evento completo en el detalle de alerta*: dos renderizados del mismo dato,
  divergencia asegurada, y duplica la superficie de «texto de evento como texto plano».
- *Solo mostrar el `provider:event_id` como texto sin enlace*: no cumple SC-004 («en como mucho una
  interacción»).

**Contrato**: la adición del campo a la ocurrencia y del parámetro a `/events` se documenta en
`contracts/eventos-alertas.md` y en `docs/ui-contract.md`. Si el DTO de ocurrencia se genera con
`ts-rs`, su `.ts` se regenera; si se mantiene a mano, su esquema Zod gana el campo y su prueba de
rechazo.

---

## D6 · «Solo hacia delante» (Q1 → A): ¿hace falta un marcador de corte?

**Decisión**: **no hace falta un marcador nuevo.** El puente evalúa solo los eventos que
`repo_varios::insert_event_if_new` devuelve como **nuevos** (`Ok(true)`) en ese ciclo. Los eventos
ya presentes en `system_events` cuando se despliega esta versión nunca se re-leen (el bookmark del
canal está por delante de ellos) y, si el bookmark se invalidara y el canal se releyera entero,
`insert_event_if_new` devuelve `Ok(false)` para los ya conocidos → no se evalúan. El «punto de
corte» de FR-018 lo da, de facto, el bookmark existente más la unicidad `(channel, record_id)`.

**Razón**: la infraestructura de deduplicación entre sesiones (J.7) ya resuelve exactamente esto. Un
marcador adicional sería estado redundante que podría discrepar del bookmark.

**Consecuencia**: FR-018 se cumple sin código de corte explícito; la prueba correspondiente inyecta
eventos «históricos» (insertados directamente en `system_events`), corre un ciclo del puente y
comprueba que no se crea ningún grupo por ellos.

---

## D7 · Permisos de Tauri

**Decisión**: **ningún permiso nuevo.** La lectura del Event Log ya la hace `event_log.rs` bajo la
ejecución elevada existente (`requireAdministrator`, ADR-004). El puente solo consume datos ya
persistidos y crea `alert_groups`, como el resto del motor. No hay comando Tauri nuevo: las alertas
de eventos salen por `alerts:changed`, la notificación nativa y la bandeja, exactamente igual que
las de SMART. La única superficie de interfaz nueva es un parámetro de consulta en la ruta
`/events`, que no es un permiso.

**Razón / verificación**: la puerta de calidad «Permisos» (ningún permiso de Tauri nuevo sin ADR) se
cumple sola. Se confirmará ejecutando `pnpm verify` y revisando `tauri.conf.json` sin cambios.

---

## Resumen de entradas a abrir en `docs/open-questions.md` (§J) antes de programar

| Ref | Tema | Valor propuesto |
|---|---|---|
| D2 | Disparadores de `device.removed_unexpected` | `disk` 157 correlacionado **o** baja de inventario de un disco no-USB; USB solo con `disk` 157; correlación de 60 s evita el doble grupo |
| D3 | Ubicación de la tabla `(proveedor,id)→regla` | En código (`alerts/reglas_eventos.rs`), con prueba de completitud contra `alert-rules.md` §3.2 |
| D4 | Ventana de correlación a caballo de dos ciclos | Consulta a `system_events` 60 s hacia atrás; reasignación de grupos derivados si el `disk` 157 llega después |
| D5 | Evento disparador en el detalle de alerta | `triggeringEventId` en la ocurrencia + enlace a `/events` con parámetro de foco |
| D6 | «Solo hacia delante» sin marcador | Se apoya en `insert_event_if_new` + bookmark; sin estado nuevo |

Ninguna de estas decisiones cambia la arquitectura ni un contrato de forma incompatible; todas caben
en el patrón de motor de alertas ya establecido.
