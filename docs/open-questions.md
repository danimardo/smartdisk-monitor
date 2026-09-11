# Cuestiones abiertas

Registro de todo lo que la especificación dejaba a interpretación, con el valor que se ha adoptado.
Nació de la revisión cruzada de `docs/` contra el paquete de diseño previa a la implementación.

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

## A. Contradicciones internas — corregidas

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

## B. Alertas y estado de salud

### B.1 · Qué color muestra un disco con una alerta reconocida · `DECIDIDO`

El estado de un disco es la peor severidad de sus alertas en estado `active` **o** `acknowledged`.
Reconocer la saca de la lista de pendientes y le añade un distintivo; no cambia el color. Solo
`resolved` y `archived` dejan de contar. Un disco sin alertas pero sin datos frescos es `unknown`,
no `ok`.

*Por qué:* el color es la señal de salud del hardware; si reconocer lo apagara, dejaría de ser
fiable. Implementado en `deviceState()` y `alertCountsTowardHealth()` (`src/lib/design/health.ts`).

*Consecuencia asumida:* un disco con un problema crónico se queda en rojo. Se mitiga con el
distintivo de "reconocida" y con el orden de la lista, no apagando el color.

*Corrección 2026-09-06:* la decisión estaba solo a medias. `enrich_with_smart_data` (Rust) siempre
pasa `None` como severidad a `device_state`, así que `DiskSummary.state` únicamente refleja la
frescura de SMART (`ok` / `unknown`), nunca `warn` / `crit`; y ninguna pantalla fundía `app.alerts`
con `app.devices`. Resultado observado: con tres alertas `active` el panel decía «Todo en orden».
Se cierra con `estadoConAlertas(disk, alerts)` (`src/lib/design/health.ts`), que el panel general y
el chrome aplican antes de leer `disk.state`. **Cuentan las alertas dirigidas al dispositivo
(`…|device:<id>`) y a cualquier volumen suyo (`…|volume:<id>`)**: un volumen lleno es un problema
del disco que lo contiene, no una categoría aparte. `deviceState()` se conserva como la definición
canónica de la regla y la prueba de `health.test.ts` que la fija.

### B.2 · El silencio no es un estado · `DECIDIDO`

`mutedUntil` es ortogonal a `AlertStatus`: una alerta puede estar activa y silenciada a la vez. El
silencio suprime **la notificación**, nunca el color ni la presencia en la lista. Sobrevive a un
reinicio (se persiste como fecha absoluta UTC en `alert_groups`) y se aplica al grupo, no a la regla
entera. Un silencio indefinido se guarda como `"infinite"`.

### B.3 · Histéresis de resolución · `PROPUESTO`

La especificación definía cuándo se activa cada alerta, nunca cuándo se resuelve. Regla general:
una alerta se resuelve automáticamente cuando la condición deja de cumplirse **con margen** durante
**tres ciclos consecutivos** de su recopilador. El margen por tipo de regla está en
`docs/alert-rules.md`, columna *Resolución*.

*Por qué:* sin margen, un disco oscilando en 69,5–70,5 °C generaría un ciclo activa→resuelta→activa
por muestra, que es exactamente el ruido que la agrupación pretende evitar.

### B.4 · Alertas informativas · `DECIDIDO`

`Severity` mantiene `info`, pero **ninguna regla de la v1.0 la produce**. Se conserva en el tipo
porque los eventos de Windows de nivel informativo se muestran en la cronología de un grupo. Un
`info` nunca crea un grupo de alerta por sí solo ni afecta al estado de un disco.

### B.5 · Qué cuenta para el estado global y el color de la bandeja · `DECIDIDO`

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

*Corrección 2026-09-06:* también estaba a medias.
- `enrich_with_smart_data` marcaba `not-yet-sampled` («aún no medido») cuando en realidad había
  habido lecturas y dejaron de llegar. Ahora, si hay al menos una muestra histórica y la última no
  es fresca, el motivo es `unreadable` («dejó de responder»). El caso `not-yet-sampled` queda solo
  para un disco que nunca ha devuelto nada.
- `unknownContributesWarning()` era **código muerto**. **Dónde se aplica** (decisión del usuario,
  tras verlo): un disco `unknown` se presenta siempre como **«Sin datos SMART» en gris** —en la
  tarjeta, el Hero y la fila del reparto—, sea cual sea el motivo. Que un `unknown` por
  `unreadable`/`collector-error` **cuente para «N necesitan atención»** y el color de la bandeja lo
  decide `estadoParaRecuento()` (solo lo usa el chrome), no `estadoConAlertas()` (que lo usan las
  tarjetas y se queda en `unknown`). Con la monitorización en pausa tampoco cuenta. Así el reparto
  es una partición limpia (Correcto + Advertencia + Crítico + Sin datos SMART = total) y el usuario
  ve el mismo texto y color en todas partes; la urgencia del disco que dejó de responder vive en el
  recuento de arriba, en la notificación y en el grupo de alerta `smart.unreadable`.
- `selectHeroDisk()` gana un criterio intermedio: sin alerta de dispositivo, protagoniza el disco
  con problema —`crit`, luego `warn`, luego un `unknown` que cuenta como degradación— antes que el
  de sistema, para que el Hero no muestre «Todo en orden» habiendo un disco en apuros. El
  `HeroPanel` estrena un texto genérico («Este disco necesita atención…») para el caso sin alerta.

*Grupo de alerta `smart.unreadable` — hecho (2026-09-06):* cada ciclo de SMART escribe la métrica
`smart_query_ok` (1.0 leído / 0.0 falló), incluidos los ciclos que fallan
(`commands::registrar_ciclo_smart_fallido`, que ya no hace solo `continue`). `alerts::evaluar_unreadable`
la evalúa con el motor de siempre: 3 ceros seguidos → advertencia, una lectura correcta la resuelve,
cooldown 6 h. La compuerta «un disco que **sí** respondía» la da `repo_metricas::hubo_lectura_smart_correcta`
(un `smart_snapshots` con `query_status` de éxito): un disco que nunca dio datos es «no compatible»,
no «ilegible», y no dispara la regla. Cinco pruebas en `alerts/mod.rs` (`alert-rules.md` §5). El
disco ilegible ahora **sí** aparece en la pantalla de Alertas con su cronología y notifica.

### B.6 · Cambio de severidad de un grupo ya reconocido · `PROPUESTO`

Si un grupo `acknowledged` sube de severidad (advertencia → crítico), vuelve a `active` y se
notifica de nuevo. Si baja, conserva `acknowledged`. El reconocimiento vale para lo que el usuario
vio, no para algo peor que aún no ha visto.

### B.7 · Recaída tras resolución · `DECIDIDO`

Un grupo resuelto que vuelve a cumplirse **no** crea un grupo nuevo: reactiva el existente e
incrementa `cycle`. La cronología separa visualmente los episodios por ciclo. Así el contador
histórico ("esto ha pasado 14 veces en tres meses") no se pierde, que es lo que pedía US-030.

### B.8 · Retirada de un disco USB · `PROPUESTO`

"Disco retirado inesperadamente: crítico inmediato" no puede aplicarse tal cual a USB: expulsar
correctamente un pendrive monitorizado generaría un crítico falso. Reglas:

- Se escucha `WM_DEVICECHANGE`; una retirada precedida de una solicitud de expulsión limpia
  (`DBT_DEVICEQUERYREMOVE` concedida) **no** genera alerta, solo un evento de inventario.
- Una retirada sin aviso previo en un dispositivo con `bus_type = USB` genera **advertencia**.
- En cualquier otro bus, genera **crítico**, como decía la especificación.

### B.9 · Ventanas de conteo · `DECIDIDO`

Donde la especificación decía "tras tres muestras" o "tras tres intentos", se entiende **tres
ciclos consecutivos del recopilador correspondiente**, no tres dentro de una ventana. Con la
frecuencia por defecto: **15 min para SMART** (la temperatura también, D.6 — el «90 s para
temperatura» de la redacción original era incorrecto). Recogido en `alert-rules.md`.

### B.10 · Un disco sin SMART fresco no enseña su última lectura como si fuera de ahora · `DECIDIDO`

`enrich_with_smart_data` conserva `temperatureC` / `percentageUsed` / `powerOnHours` con la última
muestra persistida aunque ya no sea fresca (solo `state` y `unknownReason` se condicionan a la
frescura). En la `DiskCard`, si `unknownReason` no es `null` —bus sin SMART, disco que dejó de
responder, o primera lectura aún no llegada— las tres magnitudes se muestran como «—», nunca el
valor viejo ni un contador de rendimiento en vivo presentado como lectura SMART (boceto
`01-panel-general.md` §4, constitución §I). La marca de dato obsoleto con la hora de la última
lectura válida es trabajo aparte (afecta al `HeroPanel`, ver §K).

---

## C. Capacidad

### C.1 · Suelo absoluto solo en volúmenes grandes · `DECIDIDO`

La regla original ("el mayor entre 10 % y 20 GB") marcaba como crítico un volumen de 64 GB con
15 GB libres, que es el 23 %. Regla adoptada:

- Siempre por porcentaje: <10 % advertencia, <5 % crítico.
- Además, **solo en volúmenes de 256 GB o más**, por valor absoluto: <20 GB advertencia, <10 GB
  crítico.
- Gana el criterio más severo de los dos.

El corte de 256 GB es configurable
(`alerts.capacity.absoluteFloorMinCapacityBytes`). Implementado en `capacityState()`.

### C.2 · Desactivación por volumen · `DECIDIDO`

US-033 permite desactivar las alertas de capacidad por volumen. Es una preferencia por
`volume_guid`, no por letra de unidad, y sobrevive a un cambio de letra.

---

## D. Frecuencias, batería y pausa

### D.1 · Límites de las frecuencias configurables · `PROPUESTO`

La especificación decía "configurables dentro de límites seguros" sin definirlos. La pantalla de
Ajustes no se puede diseñar sin ellos:

| Trabajo | Por defecto | Mínimo | Máximo |
|---|---|---|---|
| Actividad, capacidad, latencia, caudal | 30 s | 10 s | 5 min |
| SMART completo | 5 min | 1 min | 60 min |
| Eventos de Windows | 30 s | 15 s | 5 min |
| Detección de altas y bajas | 60 s | 30 s | 10 min |

Por debajo del mínimo el coste de CPU y de despertar el disco deja de compensar; por encima del
máximo la aplicación deja de merecer el nombre de monitor.

> **Corrección (2026-09-09, D.6):** la primera fila incluía «Temperatura», pero la temperatura solo
> se obtiene del parseo de `smartctl` y va con «SMART completo» (5 min), no con este trabajo. El
> texto original decía «Temperatura, actividad, capacidad, latencia»; se conserva aquí la razón del
> cambio.

### D.2 · Comportamiento en batería · `PROPUESTO`

Con el equipo a batería se multiplica por **4** el intervalo de actividad/capacidad/latencia y de
detección de altas y bajas. SMART completo (que es de donde sale la temperatura, D.6) y eventos de
Windows **no se alteran**: son las fuentes de las alertas graves, y spec §4 exige no suspenderlas. Al
volver a red se restauran de inmediato y se fuerza un ciclo completo.

### D.3 · Qué hace exactamente "Pausar" · `PROPUESTO`

Pausa la recopilación y la evaluación de reglas; por tanto también las notificaciones. **No** se
persiste entre reinicios: arrancar la aplicación siempre reanuda. Mientras está pausada:

- la `Toolbar` muestra un aviso permanente con el tiempo transcurrido;
- las alertas ya existentes conservan su estado y su color;
- el icono de la bandeja sigue la regla B.5.

*Por qué no se persiste:* una pausa olvidada es un monitor que no monitoriza y no lo dice. El coste
de reanudar sin querer es mucho menor que el de no vigilar durante semanas.

### D.4 · Ventana continua de actividad de disco · `PROPUESTO` (2026-09-09)

La actividad (`activity_percent`) dejaba de ser representativa: `perf_counters::leer` abría una
consulta PDH, tomaba una ventana de 1 s y la cerraba, y solo corría en `METRICAS_RAPIDAS` (30 s por
defecto). Una fotografía de 1 s de hace hasta 30 s marca 0–1 % aunque el disco esté trabajando.
Spec `007-actividad-disco-representativa`, ADR-050.

Se pasa a una **consulta PDH persistente** con **muestreo continuo** y una **ventana deslizante**
por disco de la que se derivan **media** y **pico**. Valores adoptados:

| Parámetro | Valor | Nota |
|---|---|---|
| Intervalo de muestreo | **1 s** | Desde un **hilo dedicado** (ADR-056), no el del planificador — ver corrección abajo. En batería, 1 de cada 4 ticks → **4 s** (coherente con D.2). |
| Tamaño de la ventana | **= `schedule.metrics_fast_seconds`** (30 s de fábrica; 10–300 s, D.1) | La cifra agrega «lo que va del último intervalo mostrado». |
| Umbral de hueco | **> 3 × el intervalo de muestreo** (≈3 s en red, ≈12 s en batería) | Por encima, se descartan las muestras anteriores al hueco antes de agregar (suspensión, bloqueo del subsistema de rendimiento). A escala de muestreo reproduce el 2,5× de E.1. |
| Estado del dato | `válido` si la muestra más antigua tiene ≥ el tamaño de la ventana de antigüedad; `parcial` con datos si aún no; `no disponible` si la ventana está vacía | Arranque, reanudación tras pausa y tras un hueco pasan por `parcial`, nunca por 0 (constitución §I). |

Reglas asociadas:

- **La ventana vive solo en memoria** del proceso (`AppState.actividad`), como `paused` o
  `source_health`. Un reinicio arranca con la ventana vacía → primeros ~30 s en `parcial`.
- **La serie histórica** `activity_percent` sigue con una fila por ciclo de `METRICAS_RAPIDAS`, con
  **la media de la ventana**; **no se escribe fila** si en ese ciclo la ventana está `parcial` o
  `no disponible` → hueco en la gráfica, dibujado como hueco (E.1), nunca interpolado.
- **El caudal (bytes/s) y las latencias** siguen leyéndose una vez por ciclo (`perf_counters::leer`):
  ya son tasas medidas sobre su propia ventana, y no entran en la ventana continua en esta entrega.
- Un cambio de inventario **reconstruye la consulta entera** y reinicia brevemente la ventana de
  todos los discos (estado `parcial` ≤ una cadencia): compromiso aceptado para no gestionar
  contadores PDH vivos uno a uno.

> **Corrección (2026-09-10, ADR-056):** el muestreo **no** puede vivir en el hilo del planificador.
> Medido sobre hardware real: `activity_percent` dejó de escribirse el mismo día del despliegue
> (0 filas en 30 h, mientras temperatura y caudal seguían). Causa: cuando toca un trabajo, el bucle
> del planificador se bloquea síncrono —métricas rápidas hacen `sleep(1 s)` por disco (~4 s con 4
> discos, cada 30 s); SMART, decenas de segundos cada 5 min— más que el «umbral de hueco» de 3 s, y
> `VentanaActividad::registrar` vacía la ventana en cada bloqueo. Nunca alcanzaba los ~27 s de
> cobertura que exige `válido`, así que no se persistía ni una fila, y en silencio. El muestreo
> pasa a **su propio hilo** (`iniciar_muestreo_actividad`), independiente de los trabajos, con
> log de principio VIII cuando una ventana no llega a ser representativa. El texto original decía
> «un muestreo por tick del bucle \[del planificador\]»; se conserva aquí la razón del cambio.

### D.5 · Onda de actividad de fondo de la `DiskCard` · `DECIDIDO` (2026-09-09)

El fondo de la cabecera de la `DiskCard` era la serie de **temperatura de 24 h** (ADR-034). Contra
hardware real no servía: la temperatura solo se muestrea en el ciclo SMART (5 min, D.6) y apenas
varía, y la serie se pedía una vez y no se refrescaba nunca. Pasa a ser la **actividad de disco**:

| Parámetro | Valor | Nota |
|---|---|---|
| Métrica | `activity_percent` (media de la ventana deslizante, D.4) | La misma que la cifra de la tarjeta; se dibuja también sin SMART fresco, y también con la ventana `parcial` (hueco solo con `no_disponible`): es contexto, no lectura. |
| Ventana mostrada | **≈ 5 min** (`VENTANA_ACTIVIDAD_TARJETA_MS`) | Corta a propósito: «¿ha estado ocupado ahora mismo?», no histórico. ~10 puntos a 30 s. |
| Siembra | `get_metric_series("activity_percent")` de los últimos ~15 min, al primer render | Para que no arranque vacía. |
| Refresco | un punto por evento `metrics:updated` (~30 s, el de siempre) | **Sin sondeo** (ADR-015): se consume el evento que ya llega. No es el «modo en vivo» de 1-2 s que descartó ADR-050. |
| Dedup | se ignora un evento a < 15 s del último punto | Un ciclo SMART reemite `metrics:updated` sin que la ventana avance. |

Registrado en ADR-051. El `HeroPanel` **no** cambia: su curva de fondo sigue acoplada a su cifra
dominante, la temperatura.

### D.6 · La temperatura es una métrica de cadencia SMART, no de métricas rápidas · `DECIDIDO` (2026-09-09)

`docs/product-specification.md` y la tabla de D.1 agrupaban «Temperatura» con actividad/capacidad/
latencia (30 s), y B.9 hablaba de «90 s para temperatura». Es incorrecto: `temperature_celsius`
**solo** se obtiene del parseo de `smartctl`, que escribe el trabajo `SMART_COMPLETO` (`planificador.rs`,
300 s por defecto). Ningún colector rápido la produce. La cadencia real de un punto de temperatura
es la del ciclo SMART.

Consecuencia práctica: `commands::cadencia_esperada_ms` declaraba `temperature_celsius` a 30 s, lo
que hacía que `domain::series::completar_serie` insertara un hueco entre cada par de muestras (300 s
≫ 2,5 × 30 s, E.1) y la gráfica de temperatura del detalle saliera como puntos sueltos con bandas de
hueco. Corregido: la temperatura declara **300 s**. La ventana de conteo de las reglas de
temperatura ya se contaba en ciclos SMART (3 ciclos = 15 min, `alert-rules.md` §2), así que las
alertas no cambian; se corrige el «90 s» de B.9.

---

## E. Historial y gráficas

### E.1 · Tabla intervalo → resolución · `PROPUESTO`

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
- **Un salto es un hueco a partir de 2,5× la cadencia** (`MULTIPLO_HUECO` en `domain::series.rs`,
  `FACTOR_HUECO` en `src/lib/design/series.ts`). Empezó en **1,5×** (`completar_serie`, T060), pero
  usando la aplicación de verdad el usuario vio que la gráfica del detalle de disco salía como
  **puntos sueltos**: 1,5× marca como hueco cada ciclo de recopilación puntualmente perdido —normal
  en un equipo que se suspende o va cargado—, y cada racha de una muestra se dibujaba como un punto
  solitario. 2,5× ≈ dos ciclos: un salto suelto no parte la línea, una parada de minutos u horas
  sigue quedando como banda gris. `domain::retencion`/`domain::salud` mantienen su 1,5× (cubos,
  frescura: otra decisión).
- **El trazo es una curva suave, no una polilínea recta**: spline cúbica de Hermite **monótona**
  (`rutaSuave` en `series.ts`), por tramo continuo, que no rebasa el mínimo/máximo de cada segmento
  —así no aparenta cruzar un umbral ni inventa un pico—. Es «la onda del boceto» sin falsear el
  dato. `Sparkline` y `TimeSeriesChart` la comparten.

### E.2 · Interacción de la gráfica · `DECIDIDO`

Cursor de lectura con ratón (el punto más cercano en tiempo) y con teclado (flechas, `Inicio`,
`Fin`, `Esc`). **Actualizado**: el valor y el instante del punto salen en un **globo flotante junto
al punto** (`ChartTip`), no en el pie — usando la aplicación real el usuario no miraba el pie
porque tiene la vista en el gráfico. El pie vuelve a mostrar siempre su contexto fijo (rangos «sin
datos» y resolución). Una región `aria-live` anuncia el valor al recorrer la serie con el teclado.
Sin zoom ni selección por arrastre en la v1.0: el `SegmentedControl` de intervalo cubre la
necesidad y evita un patrón nuevo.

**Alcance**: la lectura está en `TimeSeriesChart` y, opt-in (`interactivo`), en `Sparkline` — que
la usa `MetricCard` para las cuatro miniaturas del detalle de disco. La sparkline de fondo
decorativa de `HeroPanel`/`DiskCard` (`pointer-events-none`) no la lleva. `ChartTip` entra en el
catálogo (`ui-design.md` §3) para que toda gráfica futura la muestre igual. Implementado.

### E.3 · Retención mínima frente a US-022 · `DECIDIDO`

US-022 promete "al menos 30 días de historial": se cumple con los agregados de 5 minutos, no con las
muestras crudas (7 días). La historia se reformula para decirlo explícitamente y no dar a entender
que habrá 30 días de detalle.

### E.4 · La sparkline del panel general se ajusta al último tramo continuo · `DECIDIDO` (2026-09-06)

El `HeroPanel` y las miniaturas de la `DiskCard` piden 24 h, pero **dibujan solo el último tramo
sin cortes** (`ultimoTramoVisible()` en `src/lib/design/series.ts`), no las 24 h enteras. Motivo:
con la app parada a ratos —se cierra, se reinicia el equipo, se acaba de instalar— el histórico
tiene huecos de horas que `Sparkline` pinta como rayas sueltas (regla «un hueco es un hueco», que
no cambia). Enseñar el tramo en curso devuelve la onda del boceto y su ancho se adapta a lo que
hay: un minuto de datos → ventana de un minuto (**sin mínimo de zoom**, decisión del usuario).
El trazo curvo (spline monótona, E.1) refuerza ese «devuelve la onda»; el corte por tramo del
`ultimoTramoVisible` (4× P25) es independiente del umbral de hueco de E.1 (2,5×) y no cambia.

- El corte se hace donde una separación supera **4×** el **percentil 25** de las separaciones
  reales (no la mediana: con pocas muestras y un parón, media serie *es* el parón). Un ciclo
  perdido no abre tramo nuevo; un parón de horas sí.
- El pie del Hero muestra la ventana real («Ventana: 8 min» / «Ventana: 24 h», `formatSpanShort`).
- **Recién abierta la app** hay una o dos muestras y no da para una onda: por debajo de **4 puntos
  o minuto y medio** de ventana el Hero no dibuja la rayita casi plana —parecía un fallo—, pone
  «Recopilando datos…» y se rellena solo en unos minutos.
- **No afecta** al detalle de disco (`/disks/[id]`): ahí el `SegmentedControl` de intervalo y los
  ejes son la interfaz, y la ventana la elige el usuario.
- Los factores (4×, p25, umbral de «recopilando») son de afinado; si un histórico real se ve mal,
  se ajustan aquí.

### E.5 · El detalle de disco grafica también la actividad, no solo la temperatura · `DECIDIDO` (2026-09-10)

El detalle de disco (`/disks/[id]`) solo tenía una gráfica histórica, la de temperatura. El usuario
pidió ver también **la carga de trabajo del disco a lo largo del tiempo** —picos de uso y periodos
de reposo— con el mismo control de intervalo. Se trató como **extensión acotada** de la pantalla
del rediseño v3, no como spec nueva: el dato (`activity_percent`) ya se persiste con la misma
retención y agregación que la temperatura (D.4, ADR-050), solo faltaba exponerlo.

| Aspecto | Valor adoptado |
|---|---|
| Disposición | Segundo `TimeSeriesChart` **apilado debajo** del de temperatura, en la columna izquierda de la rejilla `1.6fr 1fr`. No un conmutador: apilar deja comparar de un vistazo si un pico térmico coincidió con una ráfaga de uso. |
| Selector de intervalo | **Único y compartido** (24 h / 7 d / 30 d / personalizado). Un cambio de rango vuelve a pedir las dos series. |
| Eje Y | Fijo **0–100 %**. |
| Líneas de umbral | **Ninguna** (la actividad no genera alertas). |
| Color de la curva | **Acento siempre** (no sigue el estado del disco, a diferencia de la temperatura). |
| Huecos | Banda gris «sin datos» con su leyenda cuando la ventana de actividad estuvo `parcial`/`no_disponible` (arranque, reanudación, pausa, equipo apagado). Nunca un cero. Mismo trazado por tramos que la temperatura. |
| Sparkline de la `MetricCard` de «Actividad» | **Se conserva**: es la cifra de un vistazo, no el histórico navegable. |
| Fallo por fuente | Cada serie carga y falla por su cuenta: si una fuente responde y la otra no, una gráfica se pinta y la otra muestra su `EmptyState kind="error"`. |

Implementación: `TimeSeriesChart` gana la prop opcional `titulo` (título visible + prefijo de la
etiqueta accesible, `chart.titledSummary`) para que los dos `role="img"` apilados se distingan con
un lector de pantalla; con una sola gráfica la prop queda vacía y nada cambia. Sin cambios de
contrato, comando, permiso ni backend. Decisión de diseño en **ADR-055**.

En la ventana mínima (1024 × 560) las dos gráficas más los contadores exceden el alto y la región
hace scroll, como ya contempla `07-detalle-disco.md` §6.

---

## F. Identidad de dispositivo

### F.1 · Composición de la huella · `PROPUESTO`

`fingerprint = sha256(model | capacity_bytes | bus_type | wwn_o_pnp_device_id)`.

**El firmware queda fuera a propósito.** La arquitectura pide detectar cambios de firmware; si
formara parte de la huella, actualizar el firmware partiría el historial del disco en dos
dispositivos distintos. Un cambio de firmware se registra como evento de inventario sobre la misma
entidad.

### F.2 · Dispositivos sin número de serie · `PROPUESTO`

Se monitorizan igual, con `serial_number = null` y la huella de F.1 como identidad, dejando
`identity_confidence = "fingerprint"`. La UI marca esos discos como "identidad inferida" en el
detalle. Si además cambia el `PNPDeviceID` (un USB movido de puerto), se tratará como dispositivo
nuevo: es una limitación conocida y documentada, no un fallo.

### F.3 · Sustitución de disco · `DECIDIDO`

El historial se conserva ligado a la entidad antigua, marcada con `removed_at`, y el disco nuevo
arranca su propia entidad. Nunca se fusionan historiales, ni siquiera con la misma capacidad y
modelo.

---

## G. Contrato UI ↔ backend

### G.1 · Empuje, no sondeo · `DECIDIDO` (ADR-015)

El backend emite eventos Tauri tipados; la UI no usa `setInterval` para pedir datos. Lista completa
en `docs/ui-contract.md`.

### G.2 · Forma del error · `DECIDIDO`

Todo comando que falle devuelve un `AppError { code, messageKey, messageVars, detail, source,
retryable }`. `messageKey` da la frase humana, `detail` el texto técnico literal que se muestra
dentro de un `<details>` y se puede copiar. Definido en `src/lib/design/types.ts`.

### G.3 · Generación de los tipos · `PROPUESTO`

Los DTO se generan desde Rust con `ts-rs` y se comprueban en CI: si un tipo de Rust cambia y el
`.ts` generado no coincide con el del repositorio, la compilación falla. Evita que
`docs/ui-contract.md` envejezca en silencio, que es el destino habitual de este tipo de documento.

---

## H. Decisiones de ingeniería

### H.1 · SvelteKit con `adapter-static` y SSR desactivado · `DECIDIDO` (ADR-014)

Es la vía que Tauri documenta oficialmente y la que el paquete de diseño ya asumía (`$lib`). Aporta
enrutado por ficheros para las siete pantallas sin añadir dependencias de terceros, cosa que un
router externo sí haría y que `AGENTS.md` §1 prohíbe.

### H.2 · Resto de convenciones · `PROPUESTO`

Versiones, gestor de paquetes, estructura de carpetas, linters y CI en
`docs/engineering-conventions.md`.

---

## I. Riesgos técnicos a validar en Fase 0

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
| I.5 | `smartctl` tras controladoras RAID y puentes USB | Qué cascada de `-d` (`sat`, `nvme`, `sntjmicron`, `csmi`) merece la pena antes de declarar "no compatible" | Se documenta la limitación por modelo de puente | **Parcial**: el formato de ruta (`/dev/pdN`, no `\\.\PhysicalDriveN`) ya está medido y corregido contra 2 SATA + 2 NVMe reales, véase J.42. La cascada de modos para puentes USB/RAID exóticos sigue `ABIERTO` — necesita ese hardware concreto: **US-010** |
| I.6 | Instancia única y ACL de `ProgramData` | ~~Pendiente~~ **Resuelto**: eran dos problemas. La instancia única exige comunicar procesos, no solo detectarlos (ADR-025). Y `ProgramData` **no** restringe la escritura a administradores: un usuario sin privilegios se apropia de la carpeta pre-creándola (ADR-026). Véase §R | — | `DECIDIDO` |
| I.7 | Rendimiento de la interfaz con 20 discos y 5.000 eventos | ~~Pendiente~~ **Resuelto**: medido con Playwright + `PerformanceObserver` de tareas largas, aislando el coste fijo de la primera navegación del coste real de la interacción. Cero tareas ≥50 ms en ambos escenarios (desplazar 5.000 eventos, recibir 20 discos en caliente). Véase J.26 | — | `DECIDIDO` |

---

## J. Cuestiones menores resueltas por defecto

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
| J.8 | Tamaño de ventana | Mínimo técnico 1024 × 560, objetivo de diseño 1280 × 720, **predeterminada 1695 × 988** (solo el primer arranque; luego manda la geometría guardada, §W y ADR-040). Medido en §L.2 |
| J.9 | Acerca de | Diálogo modal sobre la pantalla actual, no sección de la `Sidebar` |
| J.10 | Eventos en la navegación | Sección propia en la `Sidebar`, con filtro preaplicado al entrar desde el detalle de un disco |
| J.11 | Plurales en i18n | Función `tp()` con `Intl.PluralRules`; claves `<clave>.one` / `<clave>.other` |
| J.12 | Persistencia de tema e idioma | `theme.set()` e `i18n.set()` devuelven la clave a guardar, pero **no** persisten: el llamante debe invocar `set_setting`. Es fácil de olvidar; conviene un envoltorio que lo haga |
| J.16 | Qué reglas de `alert-rules.md` §2 entran en el primer motor de alertas | Solo las que evalúan datos de `smartctl` ya persistidos (§3, sin colector de eventos/capacidad/estado de recopilador): `smart.health.failed`, `nvme.critical_warning`, `smart.media_errors`, `smart.error_log`, `smart.spare_below_threshold`, `smart.wear_high`, `temp.above_configured_warn/crit` (8 reglas; ampliada desde la lista original al conectar el motor con datos reales — T051 — porque `error_log_entries_total` ya lo produce el parser y `motor::evaluar_error_log` ya estaba probado, sin motivo real para dejarlo fuera). Quedan explícitamente fuera —no implementadas a medias, no simuladas— las que dependen de: registro de eventos (`events.*`, `device.removed_unexpected`, `inventory.duplicate_id`: Historia 4), capacidad de volumen (`capacity.*`: Historia 3), límite del fabricante (`temp.above_vendor_limit/critical`: requiere parsear umbrales de atributo SMART, no implementado), fallo de consulta (`smart.unreadable`) y estado del recopilador (`collector.stalled`): ambos necesitaban el seguimiento de estado por fuente que T020/T021 aportaban. **Actualizado tras T020/T021**: ese seguimiento ya existe (`SourceHealth`/`source:degraded`, `open-questions.md` J.37), pero todavía no hay ninguna regla de `alert-rules.md` que lo consuma para producir `smart.unreadable`/`collector.stalled` — sigue siendo trabajo de una historia de alertas futura, ya no de recopilación. Provisional hasta que exista esa regla (spec 001-monitor-discos-windows, T046). **Actualizado (rediseño v3)**: ya están implementadas `smart.unreadable` (métrica `smart_query_ok`), `capacity.low/critical` (serie `volume_free_bytes`, ADR-036), `temp.above_vendor_limit` (métrica `vendor_temp_limit_celsius` desde `temperature.op_limit_max`; sin límite del fabricante un disco sigue con `temp.above_configured_warn`) y `collector.stalled` (`motor::colector_estancado` + `alerts::evaluar_collector_stalled` sobre `SourceHealth`, evaluada en `post_procesar_ciclo`; una fuente que nunca tuvo éxito no dispara). Siguen fuera: `temp.above_vendor_critical` (`smartctl` no expone un crítico del fabricante fiable en el JSON). **Caso raro asumido**: si al desplegar hay un `temp.above_configured_warn` activo en un disco que justo empieza a reportar `op_limit_max`, ese grupo deja de evaluarse y se queda activo hasta archivarse a mano — el estado «reporta límite del fabricante» de un disco no cambia en la práctica. **Actualizado (spec `003-puente-eventos-alertas`)**: implementadas las 10 reglas `events.*`, `device.removed_unexpected` (por `disk` 157 correlacionado **o** por baja de inventario de un disco no USB, J.47) e `inventory.duplicate_id`, con la ventana de correlación de ráfaga de 60 s (J.49). El motor de eventos vive en `src-tauri/src/alerts/{reglas_eventos,eventos,correlacion_rafaga}.rs`; lo conecta `commands::refresh_events`. **Ya no queda ninguna regla de `alert-rules.md` §2 sin implementar salvo `temp.above_vendor_critical`.** Límite conocido: `inventory.duplicate_id` deduplica por el par de discos solo si los dos números se leen del mensaje del evento; si no, cae a un grupo único (`provider:event_id`). Las reglas de objetivo de volumen (`events.filesystem_error`, etc.) crean alertas **sin objeto** mientras `system_events.volume_id` siga sin poblarse (la correlación por volumen es trabajo futuro del colector) |
| J.17 | Cómo se resuelven `smart.media_errors` y `smart.error_log`, que según `alert-rules.md` resuelven "sin aumento durante 24 h" | **No implementado.** Esa resolución es temporal (tiempo transcurrido sin incremento), no de N ciclos consecutivos sobre el valor como el resto de la histéresis de `motor.rs`, y requeriría persistir cuándo fue el último incremento por grupo — no existe ese seguimiento. Ambas reglas quedan **activas hasta archivarse a mano** una vez creadas, igual que `smart.wear_high` (que sí documenta ese comportamiento como definitivo; estas dos no deberían quedarse así para siempre). Pendiente de una vía real: bien un campo temporal nuevo en `alert_groups`, bien un barrido periódico que compare `last_occurrence_at_utc` contra la ventana de 24 h (spec 001-monitor-discos-windows, T051). **Actualizado (spec `003`)**: la spec 003 construyó justo ese barrido —`alerts::eventos::resolver_grupos_de_eventos_vencidos`, que resuelve un grupo cuando `ahora - last_occurrence_at_utc` supera la ventana de la regla, corriendo cada ciclo en `post_procesar_ciclo`— pero **acotado a las reglas de eventos** (`es_regla_de_eventos`). Cerrar J.17 es ahora barato: darles a `smart.media_errors`/`smart.error_log` una ventana de resolución de 24 h y sumarlas al barrido. Se deja **fuera del alcance de la spec 003** a propósito (no es una regla de eventos) pero deja de necesitar diseño nuevo: es cablear el barrido existente |
| J.18 | Dónde se conecta la evaluación del motor con los datos reales | En `alerts::evaluar_smart(conn, device_id, ahora_utc)`, llamado desde `commands::refresh_smart` justo tras `persist_smart_reading` para cada dispositivo — un fallo al evaluar alertas se registra y no interrumpe el resto del ciclo (mismo criterio SC-008 que ya aplicaba a la propia lectura SMART). Sin esta llamada el motor nunca produce ningún `alert_group` en la aplicación real, por probado que esté en aislamiento; se descubrió al construir la bandeja del sistema (T052), cuando no había ninguna alerta real que mostrarle (spec 001-monitor-discos-windows, T051) |
| J.19 | Cuándo se recalcula el color del icono de la bandeja, y qué hace el botón de cierre | El color (`domain::salud::tray_state`, espejo exacto de `trayState()` en `health.ts`) se recalcula en los puntos de sincronización existentes: arranque, `refresh_now`, las seis acciones sobre alertas y pausar/reanudar. **Actualizado tras T020/T021/T022**: ya hay un ciclo real (el planificador en segundo plano sondea cada 1 s y ejecuta cada trabajo según su propia cadencia, `open-questions.md` J.34), así que el icono se recalcula también al cerrar cada ciclo de recopilación, no solo en los puntos de sincronización manuales de antes. El icono en sí se genera en memoria (RGBA) con los mismos `--sdm-{ok,warn,crit,unknown}` de tema claro, sin fichero `.ico` nuevo — `docs/decisions.md` (línea 102) ya señala systray como pantalla sin revisión visual. **Actualizado en T099**: el botón de cierre (`X`) ya lee `lifecycle.close_action` de `settings` en cada cierre (no solo al arrancar, porque Ajustes puede cambiarlo mientras la aplicación sigue abierta) y minimiza o sale de verdad según lo que diga; sin ninguna clave guardada, sigue minimizando — el lado seguro ya razonado aquí. Lo que **no** se construyó, porque ninguna tarea de la Historia 7 lo pedía explícitamente: un diálogo emergente la primera vez que se cierra, preguntando "¿minimizar o salir?" con una casilla de "recordar". La especificación (§3) lo sugiere ("pregunta... y permite recordar la decisión"), pero `platform::ventana.rs` es un fichero de backend, no de interfaz, y añadir ese diálogo habría exigido un evento nuevo (`docs/ui-contract.md` §4 no tiene ninguno para esto) y un componente nuevo fuera del catálogo cerrado — la vía elegida en su lugar es que la propia pantalla de Ajustes (T100) exponga `lifecycle.close_action` como una preferencia normal, sin ceremonia de primer cierre: cumple igual "se puede elegir... y cambiar la decisión" (US-072) sin inventar un patrón de interfaz nuevo a mitad de una historia sobre el backend de ajustes (spec 001-monitor-discos-windows, T052/T099) |
| J.20 | Con qué se implementó el toast nativo (T053), y cómo se decide cuándo notificar | **`tauri-plugin-notification` 2.0.0** (oficial del equipo de Tauri, mismo criterio que `tauri-plugin-single-instance`; usa WinRT en Windows). Se llama solo desde Rust (`NotificationExt`), nunca desde el webview, así que no necesita permiso de capacidades. Ajustado `rust-version` de `src-tauri/Cargo.toml` de `1.77` a `1.77.2` porque es el mínimo que declara el propio plugin. La decisión de notificar vive en `alerts::notificaciones` (no en `agrupacion`, que lo deja explícito en su cabecera): un episodio nuevo, una recaída o un escalado **siempre** notifican; una ocurrencia repetida respeta el cooldown por regla de `alert-rules.md` §2 (de "ninguno" en `smart.health.failed` a "7 días" en `smart.wear_high`); un grupo silenciado (`muted_until`) nunca notifica, silencio y color son cosas distintas (`ciclo.rs`). El cooldown se guarda en `AppState.notified_at` (id de grupo → instante), **en memoria, sin persistir** — igual que `paused`: perderlo al reiniciar puede como mucho volver a notificar algo ya visto, nunca dejar de notificar algo nuevo. **R1 sigue sin medirse**: si el toast llega de verdad bajo `requireAdministrator` con el identificador de aplicación registrado solo puede comprobarse al empaquetar (`research.md` R1); la alternativa ya decidida (ventana propia con `Toast`) no se ha construido, porque no tiene sentido hasta que R1 se mida y falle (spec 001-monitor-discos-windows, T053) |
| J.21 | Por qué la cronología de un grupo recién creado aparecía vacía | Bug real, no una regla nueva: `repo_alertas::create_group` solo escribía en `alert_groups`, nunca en `alert_occurrences`; `reopen_as_new_cycle` (recaída) tampoco. La primera ocurrencia de cada episodio —y la primera del ciclo nuevo tras una recaída— no tenían fila propia. Corregido: ambas funciones insertan ahora su fila en la misma transacción, y `reopen_as_new_cycle` gana un parámetro `value_real` para poder escribirla. `get_alert_detail_impl` pasó de fabricar una única ocurrencia sintética a partir del grupo a consultar `repo_alertas::list_occurrences` de verdad. Encontrado al construir la pantalla de alertas (T054), al intentar mostrar una cronología que no tenía nada real que mostrar (spec 001-monitor-discos-windows, T054) |
| J.22 | Cómo se implementó el colector de capacidad de volumen (T059), y por qué `DiskSummary.volumes` estaba siempre vacío | **Bug preexistente encontrado, no de esta tarea**: `VolumeSummary.drive_letters`/`DiskSummary.volumes` estaban declarados en el DTO pero nada los rellenaba nunca — `enrich_with_smart_data` fijaba `volumes: vec![]` a secas. Corregido con `build_volume_summaries()`, que lee `repo_inventario::volumes_for_device` + `get_volume` sin condicionarlo a que el disco tenga SMART (un disco sin SMART puede tener volúmenes). El colector en sí (`collectors::capacidad`) hace un único `Get-Partition \| Get-Volume` por PowerShell —mismo patrón que `windows_storage.rs`— porque `Get-Partition` ya sabe el `DiskNumber`, evitando correlacionar dos consultas por letra de unidad (frágil: la letra puede faltar). El enlace disco↔volumen usa esa misma numeración efímera de Windows, capturada en la misma pasada de `reconciliar_inventario` en que ya se conoce para los discos: confianza `Exact` si coincide con un dispositivo reconciliado, `Unknown` si no. Un volumen sin `UniqueId` se omite en vez de usar la letra como clave, que es justo lo inestable (spec 001-monitor-discos-windows, T059) |
| J.23 | Cómo se implementó el colector de contadores de rendimiento (T058), y un hallazgo medido sobre Windows real | Enlace FFI directo a `pdh.dll` (mismo criterio que `platform::locale.rs` con `kernel32`: sin añadir el crate `windows` completo por cinco funciones estables). **Medido en un Windows real en español**: los nombres de objeto y contador de PDH están **localizados** (`PhysicalDisk` = "Disco físico", `% Idle Time` = "% de tiempo inactivo"); `PdhAddCounterW`/`PdhExpandWildCardPathW` con una ruta en inglés fallan con `PDH_CSTATUS_NO_OBJECT` fuera de un Windows en inglés — se comprobó primero con `Get-Counter` (falla con el nombre inglés, funciona con el español) y confirmó el diagnóstico. Solución: **`PdhAddEnglishCounterW`**, que traduce el nombre **y** resuelve el comodín de instancia (`\PhysicalDisk(0 *)\...`) en la misma llamada, sin paso de expansión aparte — probado end-to-end contra dos discos físicos reales de esta máquina, con valores de actividad y latencia coherentes con su carga real en el momento de la medición. Una tasa (bytes/s, sec/operación) exige dos muestras separadas en el tiempo: se recoge dos veces con 1 s de espera entre medias, una sola vez para las cinco fuentes (no cinco esperas), y se cierra la consulta —autónoma, no persistente entre ciclos. **Actualizado tras T020**: el planificador en segundo plano ya existe y llama a esta función una vez por ciclo de `METRICAS_RAPIDAS` debido, pero sigue abriendo y cerrando su propia consulta PDH en cada llamada en vez de mantenerla abierta entre ciclos reales — eso sigue siendo una optimización pendiente, no relacionada con si el bucle existe (spec 001-monitor-discos-windows, T058) |
| J.24 | Qué hash calcula `system_events.dedup_hash` (T067) | No especificado en ningún documento más allá de "hash de deduplicación" (`data-model.md` §2). La identidad real de un evento ya es `UNIQUE(channel, record_id)`, así que este campo no decide duplicados por sí solo. Se calcula como `sha256(provider \| event_id \| occurred_at_utc \| message)`: una huella de contenido pensada para el trabajo futuro de correlación por ventana temporal de `alert-rules.md` §3.5 (un mismo suceso físico produce varios eventos correlacionados en 60 s), no usada todavía por ningún módulo de esta sesión. Provisional hasta que la correlación por ventana (§3.5) se implemente y decida si necesita este campo o algo distinto |
| J.25 | Confianza de la correlación evento→disco por número de disco (T069) según de dónde salga el número | `docs/alert-rules.md` §3.6 exige resolver contra el inventario, nunca por coincidencia textual pura, pero no distingue confianza entre las formas de identificador que "conviven" en un mismo mensaje. Decisión: **`exact`** cuando el número de disco sale de una ruta de dispositivo estructurada (`\Device\HarddiskN\...`, generada por el propio sistema en el XML crudo del evento) y coincide con un disco del inventario; **`inferred`** cuando sale del texto humano ya formateado ("disco N"/"disk N"), porque ese texto está traducido y depende de la plantilla de mensaje del proveedor, una capa menos directa que la ruta de dispositivo. `\Device\HarddiskVolumeNN` y los nombres PDO (`\Device\0003d2a5`) quedan sin resolver (`unknown`): el colector de capacidad (T059) no captura ese identificador por volumen todavía, y añadirlo es trabajo del propio colector, no de la correlación. El número que sigue a `DR` en `\Device\HarddiskN\DRxx` **nunca** se confunde con el número de disco (`alert-rules.md` §3.6, advertencia explícita) (spec 001-monitor-discos-windows, T069) |
| J.26 | Medición de R3 (T074): 20 discos y 5.000 eventos frente al umbral de 50 ms de SC-007/SC-009 | **Medido con el plano de interfaz** (Playwright + IPC propio + `PerformanceObserver` de "long tasks", que solo informa de tareas ≥50 ms). Hallazgo real durante la medición: la **primera navegación** de la prueba produce 70-120 ms de tarea larga **incluso con 0 o 2 discos** — coste fijo de evaluar el paquete en un Chromium recién arrancado, no relacionado con la cantidad de datos. Confundir ese coste con el de renderizar 20 discos habría hecho fallar la prueba por una razón ajena a SC-007 (que habla de seguir respondiendo *durante* el trabajo, no del arranque en sí). Corregido separando ambos: cada prueba dejar pasar la carga inicial y **luego** reinicia el observador, midiendo solo la interacción real — desplazar los 5.000 eventos con `VirtualList`, o recibir 20 discos en caliente vía un `metrics:updated` simulado (`ipc-falso.ts` ganó `emitirEvento()` para poder disparar ese evento desde la prueba). Ambos escenarios pasan limpios, cero tareas largas. De camino se virtualizó también el panel general (T064 ya había virtualizado la lista de eventos): la rejilla `DiskCard` pasó de pintar todas las tarjetas de una vez a virtualizarse **por fila** con el mismo `VirtualList` genérico, agrupando tantas tarjetas por fila como columnas quepan en el ancho disponible — la primera medición (antes de aislar el coste fijo de navegación) señaló la rejilla sin virtualizar como sospechosa, y aunque el diagnóstico final mostró que el problema real estaba en la metodología de medición y no en la rejilla, la virtualización quedó aplicada por ser una mejora real y ya verificada, no se revirtió (spec 001-monitor-discos-windows, T074). **Reemplazado en parte el 2026-09-06 (§U):** el panel v3 retira la `VirtualList` de la rejilla de discos —su nuevo encuadre (héroe + pie) exige una sola región de scroll— y cubre SC-006 con la variante compacta de `DiskCard` (sin sparkline a partir de 12 discos, `ui-design.md` §7); la misma prueba de rendimiento sigue verde. La virtualización de la **lista de eventos** (T064) se mantiene |
| J.27 | Nombre de la "carpeta controlada" del benchmark (T077), no especificado en ningún documento | `<raíz del volumen>\SmartDisk Monitor Benchmark\`: en la raíz del volumen que se está probando, no en `%ProgramData%` —tiene que vivir en el mismo volumen para medir su E/S real, no la del disco del sistema—, con el mismo nombre visible que ya usa la carpeta de datos (`platform::paths::data_dir()`). El nombre de archivo dentro de esa carpeta lleva un sufijo aleatorio (`benchmark-<aleatorio>.tmp`); "nunca se sobrescribe un archivo existente" (product-specification.md §6) se comprueba activamente antes de crear el archivo, no se asume por la aleatoriedad del nombre (spec 001-monitor-discos-windows, T077) |
| J.28 | Forma exacta del JSON de estado del autotest SMART corto (T081), **sin verificar contra hardware real** | A diferencia de todo lo demás de esta sesión (SMART, PDH, wevtapi, chkdsk, benchmark: todo probado contra el sistema real de esta máquina), este dato concreto **no se ha verificado**: un autotest corto real tarda minutos en el disco y el usuario pidió expresamente no ejecutarlo. `tests::autotest::parse_estado_json` asume la forma documentada de `ata_smart_data.self_test.status.{value,string,passed}` y `.polling_minutes.short` que expone `smartctl -a -j`, construida a partir de conocimiento general de su formato JSON, no de una captura propia. Antes de dar el autotest por terminado hay que lanzar uno real (cuando el usuario lo autorice) y comparar el JSON verdadero con lo que este parser espera — el mismo trato que ya se dio a `smartctl_parser.rs` con sus fixtures reales (spec 001-monitor-discos-windows, T081) |
| J.15 | Cómo distinguir "sin compatibilidad SMART" de "aún sin leer" en `get_device_detail` | Ausencia de `smartctl_path` (T025: `Get-PhysicalDisk.DeviceId` no numérico, típico de volúmenes RAID lógicos) se trata como `unsupported`; presencia de `smartctl_path` sin ninguna muestra `metric_samples.source = smartctl` se trata como `not-yet-sampled`. Deliberadamente **no** se interpreta el `exit_status` de `smartctl` como señal de soporte: sus bits documentan fallos de sintaxis/apertura/hallazgos SMART, no "este bus no expone SMART", y esa lectura no se ha podido verificar contra hardware real (`open-questions.md` I.5). Provisional hasta medir (spec 001-monitor-discos-windows, T038) |
| J.13 | Umbrales de espacio libre para detener la escritura de historial | 1 GB para el aviso y 256 MB para la parada, sobre el volumen donde reside el historial (`storage.free_space_warn_bytes` / `storage.free_space_halt_bytes`, spec 001-monitor-discos-windows). Valores de partida razonables para Windows, **no medidos**; confirmar al implementar la retención (T001, T017-T018) |
| J.14 | Cómo se representa la agregación de `metric_samples` | `docs/data-model.md` §4 exige conservar mínimo, máximo, promedio, primera y última lectura, pero el esquema solo tenía una columna de valor por fila. Se añade la tabla `metric_aggregates` (migración 0002) con `value_min/max/avg/first/last`, `bucket_start_utc`/`bucket_end_utc` y `resolution`. Los "tres periodos de retención" de US-071 son las tres resoluciones ya definidas (`raw`, `five_minutes`, `hourly`): `retention.raw_days` (7), `retention.five_minutes_days` (90), `retention.hourly_days` (730); pasado el tercero se purga. `value_last - value_first` da el incremento del bucket para contadores acumulativos, sin columna aparte. Valores por defecto, **no medidos** (spec 001-monitor-discos-windows, T015) |
| J.29 | Cómo conectar los cinco comandos de pruebas (T083): identificadores, exclusión mutua, umbral térmico y columnas sin sitio propio en `test_runs` | **Identificador de `test_run` y sufijo aleatorio del archivo del benchmark** (J.27): `format!("{:x}", OffsetDateTime::now_utc().unix_timestamp_nanos())` — nanosegundos UTC en hexadecimal, sin añadir una dependencia de aleatoriedad (mismo criterio que el LCG de T079); la unicidad real la sigue dando `rutas::confirmar_no_sobrescribe`, no la improbabilidad de colisión. **Exclusión mutua** (`test.busy`, ya previsto en `ui-contract.md` §1: "ya hay una prueba en ese disco"): se aplica por disco físico subyacente vía `device_volume_links`, no solo por el id exacto recibido — antes de arrancar cualquier prueba se comprueba que ni el objetivo ni ningún otro volumen/dispositivo del mismo disco tenga ya un `test_run` en `pending`/`running`/`cancelling`. Esto cubre a la vez la regla genérica del contrato y la regla explícita de `product-specification.md` §6 ("el autotest no se permite simultáneamente con el benchmark de la aplicación"), sin tabla de exclusión aparte. **Umbral térmico "configurado"** de `tests::guardia::limite_critico_efectivo` cuando el fabricante no lo declara (hoy siempre: `vendor_temp_critical_c` no está implementado, J.15/J.16): se reutiliza el mismo valor que ya usa el motor de alertas para `temp.above_configured_crit`, **80 °C** (`alert-rules.md`, `alerts::motor::evaluar_temperatura_configurada_crit`) — mismo concepto normativo, no un valor nuevo. **Columnas sin sitio propio**: `test_runs` (migración 0001) no tiene columna para `command`, `output` ni `outputEncoding` (`ui-contract.md` §3.6); se guardan dentro de `parameters_json` (el comando, fijado al crear la fila) y `result_summary_json` (salida y codificación, solo se conocen al terminar) en vez de abrir una migración nueva. `orphanPath` reutiliza la columna `temp_path` ya existente: mientras la prueba corre, o si el archivo no se pudo borrar al terminar, queda con la ruta; se limpia a `NULL` en cuanto el borrado tiene éxito. `volume.not_found` se añade a la tabla de códigos de `ui-contract.md` §1 en paralelo a `device.not_found`, que hasta ahora solo cubría `device_id`. **Límite conocido, no simulado**: `RazonParada::Space` (T079) solo es alcanzable como rechazo previo (`test.insufficient_space`) antes de crear la fila — `tests::benchmark::ejecutar` no comprueba espacio libre durante la ejecución (T079 solo implementó cancelación y guardia térmica), así que un agotamiento de espacio a mitad de prueba no se detecta hoy (spec 001-monitor-discos-windows, T083). **Actualizado (ADR-053, spec 008)**: el motor propio (`tests::benchmark`) se ha eliminado; la prueba de Rendimiento usa DiskSpd sobre un archivo de tamaño fijo (`-c 1 GiB`) que **no crece**, así que la guardia de espacio durante la ejecución sigue sin existir y sin hacer falta. `RazonParada::Space` ya no existe como estado; `test.insufficient_space` sigue siendo un rechazo previo. El resto de J.29 (identificador de `test_run`, exclusión por disco físico, umbral térmico 80 °C, columnas dentro de `parameters_json`/`result_summary_json`, `orphanPath` sobre `temp_path`) se conserva tal cual con DiskSpd |
| J.30 | Contenido exacto de la exportación tabular/estructurada (T087): ningún documento fija las columnas o campos | **CSV y JSON son el volcado completo**, una fila/objeto por `(dispositivo, metric_key, marca de tiempo)`: `schemaVersion, deviceId, deviceLabel, metricKey, unit, resolution, timestampUtc, value`. Qué métricas incluir no es una lista fija: `repo_metricas::distinct_metric_keys` devuelve las que de verdad tengan dato del dispositivo en el rango (crudo o agregado), para no inventar columnas vacías ni olvidar una real. **Resolución por rango**, igual que ya hace `get_metric_series_impl` para las gráficas (crudo ≤24 h y dentro de los últimos 7 días; `five_minutes` ≤7 días; `hourly` con reserva a `five_minutes` ≤90 días; `hourly` más allá) — implementada de nuevo en `reporting/export.rs`, sin tocar la función existente de `commands/mod.rs`, para no arriesgar una regresión en la gráfica por una necesidad distinta (el `value` de una fila agregada es `value_avg` con reserva a `value_last`, igual que ya hace `leer_agregados_dispositivo`). **Dato ausente**: `null` en JSON, cadena literal `"N/A"` en CSV — nunca vacío ni cero, mismo criterio que el resto de la aplicación. `deviceIds: null` en el comando significa todos los dispositivos monitorizados (no los excluidos). Sigue vigente para CSV/JSON, que no cambian. **Superseded 2026-09-11 en lo que toca al HTML** por spec `009-informe-mejorado` (sección Y): el HTML dejó de ser "identidad + alertas con `ruleKey` crudo"; ahora es un resumen por disco con contadores, eventos, mini-gráficas y frase legible de alerta, y sí gana un evento de progreso (`report:progress`) para el caso —solo ese— de la exportación con resumen IA |
| J.31 | Contenido y disposición del ZIP de diagnóstico (T090): ningún documento fija los ficheros que lleva dentro | **`smart_snapshots.raw_json_path`/`fields_json` están sin usar**: ningún colector escribe hoy el JSON crudo de `smartctl` a disco ni a la base (`insert_smart_snapshot` siempre los llama con `None`, T036). El ZIP no puede leer un archivo que no existe, así que **vuelve a consultar `smartctl` en el momento de generarlo** (`collectors::smartctl::query_device_json`, ya verificado contra hardware real esta sesión) para cada dispositivo con `smartctl_path` — un diagnóstico fresco, no uno reconstruido de una captura que nunca se guardó. **Disposición dentro del ZIP**: `manifest.json` (schemaVersion, generatedAtUtc, versión de la app, `anonymized`, `redactedFields`), `settings.json` (`repo_varios::list_settings`, todas las claves), `events.json` (`repo_varios::list_events` sin filtro, límite alto en vez de paginado: es un volcado, no una pantalla), `smart/<deviceId>.json` (o `smart/<deviceId>.error.txt` si la consulta falla — un fallo de un disco no debe tirar el paquete entero), `logs/<nombre-de-fichero>` (todo lo que haya en `platform::paths::log_dir()`, tal cual lo escribe `tracing_appender::rolling::daily`, FR-029c). **Cada entrada de texto pasa por el mismo `Anonimizador`** antes de escribirse — de ahí que la sustitución sea consistente en todo el paquete (US-051): el mismo número de serie se convierte en el mismo `<SERIE-N>` tanto en `smart/*.json` como en `logs/*` si apareciera ahí. `includeIdentifiers: true` construye un `Anonimizador::sin_anonimizar()`: nada se sustituye, y `redactedFields` viaja vacío en el manifiesto (spec 001-monitor-discos-windows, T090) |
| J.32 | Forma completa de `Settings` (T095/T096): `ui-contract.md` §3.1 nombra `get_settings`/`set_setting`/`reset_settings` pero nunca escribe la interfaz — ningún documento reúne en un solo sitio todos los campos configurables que ya estaban dispersos (D.1, C.1, J.13, J.14) | Cuatro grupos, alineados con los cuatro valores de `reset_settings({scope})`: **`schedule`** (`metricsFastSeconds`/`smartFullSeconds`/`eventsSeconds`/`discoverySeconds`) reutiliza tal cual los límites ya codificados en `collectors::planificador::{METRICAS_RAPIDAS,SMART_COMPLETO,EVENTOS_WINDOWS,ALTAS_Y_BAJAS}` (D.1) — ese módulo ya decía en su propio comentario "esto lo hace `domain::ajustes`, no este módulo", así que no son límites nuevos, son los que ya existían sin consumidor. **`alerts`**: `tempConfiguredWarnC`/`tempConfiguredCritC` (por defecto 70/80, los mismos literales que hoy tiene hardcodeados `alerts::motor` para `temp.above_configured_warn/crit`; límites nuevos, no medidos: 40-95 °C para el aviso, el crítico entre el aviso y 100 °C) y los cinco campos de capacidad ya decididos en C.1/ADR-019 (`capacityWarnPercent` 10, `capacityCritPercent` 5, `capacityAbsoluteFloorMinCapacityBytes` 256 GiB, `capacityAbsoluteFloorWarnBytes` 20 GiB, `capacityAbsoluteFloorCritBytes` 10 GiB — los mismos valores que ya usa `capacityState()` en `src/lib/design/health.ts`, hoy con el suelo fijo en una constante en vez de leído de `settings`). **`retention`**: los tres periodos de J.14 (7/90/730 días, límites nuevos y razonables: crudo 1-30, cinco minutos 7-365, horario 90-1825) más `storage.free_space_warn_bytes`/`halt_bytes` de J.13 (1 GiB/256 MiB, sin límites de UI porque US-071 solo pide poder cambiar los tres periodos, no estos dos bytes). **Fuera de estos tres grupos** (solo se restauran con `scope: "all"`): `lifecycle.closeAction` (`"minimize"` por defecto, `docs/open-questions.md` J.19 seguía abierta y este valor la cierra: minimizar es "el lado seguro" ya razonado en `lib.rs`) y `closeActionRemembered`, `notifications.soundEnabled` (`false` de fábrica, US-072), `logging.verbose` (ya nombrada en `data-model.md`). **Apariencia no vive en `Settings`**: `theme`/`language`/`useSystemAccent` siguen teniendo su propio `get_appearance_settings()` ya construido; se persisten con el mismo `set_setting(key, value)` genérico (`theme.svelte.ts`/`i18n.svelte.ts` ya devuelven `{key: "settings.appearance.theme"/"settings.appearance.language", value}` a la espera de un consumidor, que es exactamente lo que T097 les da). **Límite conocido, no ampliado por esta historia**: ni `domain::espacio` (guardia de espacio del historial) ni `capacityState()` ni ninguna regla `capacity.low/critical` en el motor de alertas leen hoy estos valores de `settings` en un ciclo real — no existe todavía el bucle de recopilación en producción que los invoque (ninguna tarea de esta historia lo pide); esta historia deja el valor correctamente guardado y validado, listo para cuando ese consumidor exista, igual que ya pasaba con `logging.verbose` antes de FR-029a (spec 001-monitor-discos-windows, T095/T096) |
| J.33 | Comprobación de "cero peticiones salientes" (T107, SC-014, `quickstart.md` eslabón 10) — el guion exige "instalar en un equipo sin conexión" y "verificar con un monitor de red", que requiere un instalador real construido, instalado y en ejecución bajo un monitor de paquetes: no ejecutado esta sesión | **Auditoría estática, no medición en vivo** — mismo trato honesto que J.28 (autotest SMART): `src-tauri/Cargo.toml` no declara ningún cliente HTTP (`reqwest`/`hyper`/`ureq`) entre sus dependencias directas; `cargo tree --target x86_64-pc-windows-msvc -e normal` confirma que **ni siquiera aparecen como transitivas** en el árbol real de Windows — sí figuran en `Cargo.lock` (`reqwest`, `hyper`, `tokio`), pero por una dependencia opcional de `tauri` que el propio manifiesto de `tauri` acota a `cfg(target_os = "android", ...apple...)`: nunca se compilan para Windows. Búsqueda en todo `src-tauri/src`: cero usos de `std::net`, `TcpStream`, `UdpSocket` o una URL `http(s)://` real (las únicas coincidencias son un espacio de nombres XML dentro de una fixture de evento de Windows capturada, y las propias aserciones de test que comprueban que el HTML exportado *no* contiene ninguna). En el frontend: cero `fetch`/`XMLHttpRequest`/`WebSocket`/`EventSource` en todo `src/`; `package.json` solo depende de `@tauri-apps/api`, `@tauri-apps/plugin-dialog` y `zod`, ninguno de red. La CSP de `tauri.conf.json` cierra en profundidad: `connect-src 'self' ipc: http://ipc.localhost`, sin ningún origen remoto permitido aunque algo lo intentara. **Pendiente de la medición real** que el guion pide: construir el instalador (`pnpm app:build`), instalarlo en una máquina sin red y confirmar con un monitor de paquetes (Wireshark o similar) que no sale ni un byte — requiere una acción invasiva (instalación elevada de un binario real en el sistema) que esta sesión no ha ejecutado sin autorización explícita (spec 001-monitor-discos-windows, T107) |
| J.34 | Cadencia de sondeo del bucle en segundo plano (T020): ningún documento fija con qué frecuencia el hilo comprueba si algún trabajo ya toca | **1 segundo**, no el intervalo real de cada trabajo. Sondear con un período fijo corto (en vez de dormir el intervalo completo de la próxima tarea) es lo que permite que pausar, cambiar una frecuencia en Ajustes o cerrar la aplicación reaccionen con un retardo máximo de 1 s, en vez de hasta 1 hora (el máximo configurable de SMART completo). El coste de sondear cada segundo con cuatro comparaciones de `Instant` es insignificante frente al beneficio de reactividad; no hay medición que lo respalde porque no hay nada que medir — es un valor de diseño, no un dato empírico (spec 001-monitor-discos-windows, T020) |
| J.35 | Qué dispara `inventory:changed` y qué lleva su campo `updated` (T021): el esquema Zod ya tiene `added`/`removed`/`updated`, pero `ui-contract.md` §4 solo documenta el disparador como "alta o retirada de disco o volumen" | **`updated` viaja siempre `[]`**. El contrato tal cual está escrito no pide detectar cambios de campo en un disco que sigue presente (alias, modelo, capacidades) como disparador de este evento — inventar un diff campo a campo sin que ninguna historia lo pida sería anticipar un requisito que no existe. Si en el futuro se necesita, es una decisión de una historia con su propio criterio de qué cuenta como "cambio relevante" (spec 001-monitor-discos-windows, T021) |
| J.36 | Qué hacer cuando `GetSystemPowerStatus` o `GetDiskFreeSpaceExW` fallan (T020): ninguna de las dos syscalls está garantizada a tener éxito, y ningún documento dice qué asumir si fallan | **Batería**: `ACLineStatus` fuera de `{0, 1}` (incluido `255`, "desconocido", y cualquier error de la llamada) se trata como **red eléctrica** — es el lado que menos reduce la frecuencia de recopilación y menos sorprende si en realidad el equipo funciona con batería (peor caso: se recopila un poco más de lo estrictamente necesario, nunca menos de lo que hace falta para una alerta grave). **Espacio libre**: si `GetDiskFreeSpaceExW` falla, se trata como `EstadoEspacio::Normal` (no se detiene la escritura de historial) y se registra un `tracing::warn!`: un dato desconocido no equivale a "disco lleno", mismo principio que "no compatible ≠ averiado" aplicado aquí a un fallo de sistema en vez de a un disco (spec 001-monitor-discos-windows, T020) |
| J.37 | Cómo se agrega `SourceHealth` por ciclo (T020/T021): `SourceStatus` tiene cinco valores (`ok`/`partial`/`unsupported`/`timeout`/`error`) pero ningún documento dice cuándo usar cada uno, ni con qué granularidad se mide (¿por disco?, ¿por colector?) | El propio struct `SourceHealth` es por **tipo de colector** (`source: MetricSource`, cuatro variantes), no por disco. Se agrega **por ciclo**: cada llamada a un colector externo (`smartctl::query_device_json`, `perf_counters::leer`, `windows_storage::list_physical_disks`/`list_volumes`) cuenta como un intento; al cerrar el ciclo se compara el total de intentos contra los que fallaron. **Alcance de esta implementación**: solo se distinguen `ok` (todos los intentos de ese colector tuvieron éxito) de `timeout`/`error` (al menos uno falló; `timeout` si el `AppError` resultante es `retryable`, `error` si no) — **`partial` y `unsupported` no se sintetizan todavía**: `partial` exigiría decidir un umbral de qué proporción de fallos ya cuenta como degradación parcial frente a total, que nadie ha pedido; `unsupported` exigiría saber que un colector no tiene ningún dispositivo elegible, que no es lo mismo que haber fallado. Si un colector no se invoca en absoluto durante un ciclo (cero dispositivos elegibles), su entrada en `source_health` se deja tal cual estaba, nunca se inventa un valor. `Filesystem` (la cuarta variante de `MetricSource`) no tiene todavía ningún colector que la produzca (no hay muestras `filesystem` en `persistence::repo_metricas`); se queda sin entrada hasta que exista. `source:degraded` solo se emite en el **flanco** de subida a `timeout`/`error` desde cualquier otro estado, nunca en cada ciclo que siga degradado (spec 001-monitor-discos-windows, T020/T021) |
| J.38 | `refresh_smart` mezclaba SMART y contadores de rendimiento en un único bucle por disco (T058); el bucle en segundo plano necesita dos cadencias independientes (`SMART_COMPLETO` y `METRICAS_RAPIDAS`) | **Se separa en dos funciones**: `refresh_smart` (solo SMART, evalúa alertas) y `refresh_metricas_rendimiento` (solo PDH, sin alertas). Antes de esta historia estaban acopladas porque nada las llamaba con cadencias distintas — `refresh_now` las invocaba juntas una sola vez—; el bucle en segundo plano sí necesita invocarlas por separado (30 s frente a 5 min por defecto), así que mantenerlas juntas habría hecho que el SMART completo se ejecutara cada 30 s en vez de cada 5 min, vaciando de sentido `SMART_COMPLETO`. `refresh_now` pasa a llamar a las dos, una tras otra, para conservar exactamente su comportamiento manual de antes ("actualizar ahora" sigue refrescando ambas cosas de golpe) (spec 001-monitor-discos-windows, T020) |
| J.39 | Cómo comparten lógica `refresh_now` (comando manual) y el bucle en segundo plano, sin que el fallo de un trabajo bloquee a los demás en el bucle | **Orquestación separada, post-proceso compartido**. `refresh_now` conserva su semántica de siempre (una operación falla y se aborta esa llamada entera, tal como ya esperan sus pruebas y el guion manual "actualizar ahora"). El bucle en segundo plano (`ejecutar_ciclo`) trata cada trabajo debido de forma independiente: si `AltasYBajas` falla, `EventosWindows`/`SmartCompleto`/`MetricasRapidas` igualmente debidos en el mismo sondeo se siguen ejecutando — son dominios de fallo distintos (un fallo de enumeración de discos no tiene por qué impedir una lectura SMART ya en curso de otro disco), y un bucle autónomo que se bloquea entero por un fallo ajeno sería peor que uno que registra el fallo y sigue. Ambas vías comparten la función `post_procesar_ciclo` para el post-proceso común (notificaciones, `alerts:changed`, `metrics:updated`, `inventory:changed`, `source:degraded`, icono de bandeja): es la parte que sí debe comportarse igual venga de donde venga, y compartirla es lo que evita que diverjan en silencio (spec 001-monitor-discos-windows, T020/T021) |
| J.40 | `metrics:updated.historyWriteHalted` necesita un valor real de la guardia de espacio (T018, ya implementada mas nunca conectada); ¿se aprovecha también para *detener* la escritura, o solo para *informarla*? | **Solo para informarla, esta sesión**. Se calcula de verdad (`platform::energia::espacio_libre_bytes` sobre `%ProgramData%` + `domain::espacio::UmbralesEspacio` con los umbrales de `settings`), así que el campo no miente. **No se ha conectado a ningún punto de escritura** (`persist_smart_reading`, `persist_perf_reading`, `refresh_events`): `evaluar_smart` decide activación/histéresis releyendo `metric_samples` recién persistidas, así que saltarse la escritura sin más dejaría a las alertas sin la lectura que necesitan evaluar — contradiciendo FR-020a ("la vigilancia y las alertas en vivo no dejan de funcionar"). Distinguir qué parte de la escritura debe seguir (la que alimenta alertas) de cuál debe detenerse (el historial de tendencias a largo plazo) exige mirar con cuidado `repo_metricas`/`evaluar_smart`, y no es prudente improvisarlo dentro de esta historia ya grande. Queda como tarea explícita de seguimiento, no como olvido (spec 001-monitor-discos-windows, T020) |
| J.41 | T111 (escalado 125/150/200 %) encontró que `ui-design.md` §4.0.bis (Sidebar a 56 px por debajo de 1180 px) nunca se implementó, y causaba recortes reales de texto ("Panel gene...", "No" en vez de "No disponible") — pero la norma pide "iconos" y ni el boceto aprobado ni el catálogo de componentes definen ninguno para las seis secciones | **Marcador circular con la inicial de cada sección** (P/A/E/P/I/A), como paso intermedio autorizado explícitamente por el usuario tras plantear la disyuntiva (mismo criterio que bloqueó el asistente inicial, T041, por el motivo contrario). No es iconografía nueva: reutiliza la tipografía y el `rounded-pill`/`bg-glass-3` ya existentes, con el nombre completo como `aria-label`/`title` del enlace — el texto oculto no deja de ser accesible. El pie de pausar/reanudar se oculta por completo bajo 1180 px: la misma acción sigue disponible desde el menú de la bandeja (`platform::bandeja`), así que no hay pérdida funcional, solo de acceso redundante. Verificado con capturas en las siete pantallas reales a 1024×560/1280×720/819×448/683×373/512×280 (`e2e/ui/escalado.spec.ts`); sustituir el marcador por iconos reales sigue abierto para cuando haya una revisión de diseño (spec 001-monitor-discos-windows, T111) |
| J.42 | Con el planificador ya arrancando de verdad contra hardware real (T020), `smartctl` fallaba con "Unable to detect device type" en los cuatro modos de la cascada, en los cuatro discos físicos de esta máquina (2 SATA, 2 NVMe) — medido por primera vez, `open-questions.md` I.5 seguía sin datos de hardware real | **El formato de ruta era el equivocado, no un problema de compatibilidad de disco ni de elevación**. `windows_storage::list_physical_disks` construye `smartctl_device_path` como `\\.\PhysicalDriveN` (la ruta nativa de Windows para `CreateFileW`), pero el `smartctl.exe` redistribuido (compilación MinGW, `x86_64-w64-mingw32-w11-b26200`) espera su propia convención POSIX: `smartctl -h` lo documenta explícitamente (`smartctl -a /dev/pd3` → "Prints all information for disk on PhysicalDrive 3"). Verificado a mano contra los cuatro discos reales: `\\.\PhysicalDriveN` falla siempre ("Unable to detect device type" en autodetección, "Invalid argument" en cada modo de `-d` explícito, para SATA **y** NVMe por igual, elevado o no); `/dev/pdN` funciona a la primera en los cuatro, con `model_name` y atributos SMART reales. Corregido cambiando la construcción de la ruta en `windows_storage.rs` y su análisis inverso en `disk_number_from_smartctl_path` (`commands/mod.rs`, usado para los contadores de rendimiento PDH, que sí siguen tomando el número de disco de Windows, no la ruta de `smartctl`). Cierra la parte de I.5 que bloqueaba cualquier lectura SMART en absoluto; la cascada de modos para puentes USB/RAID exóticos sigue abierta tal como I.5 ya la planteaba, pero ahora al menos parte de una ruta que sí abre el dispositivo (spec 001-monitor-discos-windows, T058/US-010) |
| J.43 | `docs/ui-design.md` ya exige "Movimiento: duration-base (220 ms) con ease-sdm... en... cambio de pantalla", pero ningún cambio de sección lo tenía — usuario lo notó al usar la aplicación de verdad por primera vez: "todo aparece de golpe". La norma dice cuánto dura y con qué curva, no qué efecto visual usar | **Entrada con desvanecimiento y una leve subida** (`opacity 0→1`, `translateY(4px)→0`), sin animación de salida — mismo criterio que ya usan `ConfirmDialog`/`Toast` (una sola animación de entrada vía `@keyframes` + `var(--sdm-duration-base)`/`var(--sdm-ease)`, nunca JS). Se dispara con `{#key}` en `AppShell.svelte`, con la clave siendo `page.url.pathname` (pasada desde `+layout.svelte`): cambia de sección → remonta el contenido → repite la animación; cambia solo un parámetro dentro de la misma pantalla (un filtro, una página del listado) → no remonta, no hay parpadeo innecesario. Una animación cruzada (fundido simultáneo de la pantalla saliente y la entrante) se descartó por ser más compleja sin que la norma la pida, y por arriesgar un parpadeo si ambas pantallas comparten elementos con el mismo punto de foco. `prefers-reduced-motion` la anula igual que a `sdm-dialog`/`sdm-toast`, sin código adicional: es la misma regla global de `tokens.css` que ya vigila `animation-duration` (spec 001-monitor-discos-windows) |
| J.44 | J.19 ya dejaba a propósito sin construir el diálogo "¿minimizar o salir? [ ] recordar" de la primera vez que se cierra la ventana; usando la aplicación real, el usuario preguntó si el minimizado silencioso (sin aviso alguno) estaba bien — no lo estaba: la ventana desaparece sin ninguna señal de que sigue vigilando | **Aviso nativo, una sola vez por arranque del proceso**, no el diálogo con casilla de recordar que J.19 seguía dejando pendiente (eso sigue exigiendo un componente y un evento nuevos, fuera de alcance de un cambio pequeño). Al minimizar por primera vez en la sesión, se muestra una notificación de Windows ("SmartDisk Monitor sigue activo... clic para reabrir, o Salir para cerrarla del todo"), reusando el mismo `tauri-plugin-notification` que ya usan las alertas — nada nuevo que aprobar. Un `AtomicBool` en `AppState` (`aviso_bandeja_mostrado`, en memoria, no persistido: cada arranque nuevo vuelve a avisar una vez) evita repetirlo en cada minimizado posterior de la misma sesión, que sería ruido. El "Salir" del menú de la bandeja sigue sin pedir confirmación (una acción explícita de menú no la necesita) (spec 001-monitor-discos-windows, US-072/T099) |
| J.45 | El usuario pidió que la aplicación "se comporte más como una aplicación nativa de Windows": nada de selección de texto libre ni del cursor de I en cualquier etiqueta, como en una página web — pero acotó él mismo el alcance: "el texto que tenga sentido seleccionar y copiar lo vamos a dejar disponible". Qué cuenta como "tiene sentido copiar" no estaba escrito en ningún sitio | **`user-select: none` global en `body`** (`tokens.css`), reactivado solo en el contenido que ya llevaba una marca semántica de "es un valor o un dato técnico": `.sdm-num` (cifras de métrica y contador), `.font-mono`/`code`/`pre` (comando literal de `ConfirmDialog`, salida de `CodeOutput`, detalle técnico de `EmptyState`, XML crudo de un evento) y los campos de formulario (`input`/`textarea`/`contenteditable`, que gestionan su propia selección nativa). No se tocó `cursor`: basta con `user-select: none` para que el navegador deje de mostrar el cursor de texto sobre lo no seleccionable (solo lo muestra sobre contenido seleccionable), sin arriesgar el cursor de mano de enlaces y botones. Se añadió además una clase de escape explícita, `.sdm-selectable`, para marcar caso a caso contenido identificador que no encaja en las categorías anteriores — usada en el modelo/alias del disco (`DiskCard.svelte`, cabecera de `disks/[id]/+page.svelte`), pensado para buscar el modelo exacto o compararlo con la documentación del fabricante. El número de serie no se muestra todavía en ninguna pantalla (`DeviceDetail.serialNumber` existe en el contrato pero no se renderiza); cuando se añada, debe llevar `.sdm-selectable` o una de las clases ya cubiertas |
| J.46 | El usuario notó que el gráfico de temperatura en tiempo real del detalle de disco no indica si un valor (p. ej. 52 °C frente a 100 °C) es bueno o peligroso, y eligió explícitamente la opción más completa entre las dos planteadas: zonas de fondo coloreadas, no solo una línea de umbral | **Dos zonas de fondo** en `TimeSeriesChart.svelte` (`warnThreshold`/`critThreshold`, sustituyendo el `threshold`/`thresholdLabel` que existía pero nunca se conectó desde ninguna pantalla): crítica desde el techo del gráfico hasta el umbral crítico, aviso desde ahí hasta el umbral de aviso, con `--sdm-warn-soft`/`--sdm-crit-soft` (ya usados en otras insignias, nunca un color nuevo) y sin superponerse entre sí. Los umbrales se calculan una sola vez por pantalla (`temperatureThresholds()` en `design/health.ts`, nueva) con la misma precedencia que `alert-rules.md` documenta para `temp.above_vendor_limit`/`_critical`/`temp.above_configured_warn`/`_crit` — el límite del fabricante manda si `smartctl` lo declaró (hoy nunca lo declara, I.5 sigue abierta), si no el configurado en Ajustes — y se reutilizan tal cual para colorear también la cifra grande de `MetricCard` (antes sin color: mismo defecto que el usuario señaló, pero en el número, no solo en el gráfico), así las dos lecturas del mismo dato en la misma pantalla no pueden discrepar entre sí. Ver K.7: el motor de alertas real todavía no lee el umbral configurado, solo el literal 70/80 °C — discrepancia ya registrada, no corregida aquí |
| J.47 | **DECIDIDO** e implementado (spec `003-puente-eventos-alertas`, research.md D2). La tabla de `alert-rules.md` §2 da como fuente de `device.removed_unexpected` «inventario + `disk` 157», sin decir cómo se distingue una expulsión limpia de una imprevista cuando el disparador es la desaparición del inventario. Windows **no** deja un rastro fiable de expulsión ordenada (no hay un id de evento equivalente al 157 para el caso bueno) | La regla se activa por **cualquiera** de: (a) un evento `disk` 157 correlacionado con un disco del inventario; (b) un disco monitorizado **no USB** que desaparece del inventario —un disco fijo no desaparece en operación normal—. Un disco USB que desaparece **sin** `disk` 157 se trata como retirada esperada y no alerta. Severidad crítica salvo `bus_type == "USB"` → advertencia. Resolución: reaparece el mismo `fingerprint`. La ventana de correlación de 60 s (J.49) evita el doble grupo cuando ambas vías se disparan por el mismo suceso |
| J.48 | **DECIDIDO** e implementado (spec `003`, research.md D3). Dónde vive la tabla `(proveedor, id) → regla` que implementa `alert-rules.md` §3.2: `settings` (mutable sin recompilar, como `PROVEEDORES_VIGILADOS`) o código | **En código**, `src-tauri/src/alerts/reglas_eventos.rs`, con una prueba de completitud y fidelidad contra `alert-rules.md` §3.2/§3.3. La lista de *proveedores* vigilados (filtro de ingesta) se queda en `event_log.rs` y sí puede ir a `settings`; la *semántica* de una regla (severidad, resolución, contexto de dedup) es dominio normativo y no la edita el usuario |
| J.49 | **DECIDIDO** e implementado (spec `003`, research.md D4). La ventana de correlación de ráfaga de `alert-rules.md` §3.5 es tiempo de reloj (60 s), pero el colector de eventos va a lotes cada 30 s: una ráfaga puede quedar partida entre dos ciclos | La correlación **no** se hace solo sobre el lote del ciclo: al evaluar cada evento nuevo se consulta `system_events` los eventos del mismo disco en los 60 s anteriores, ya persistidos. Si el `disk` 157 llegó en un ciclo anterior y ya creó su grupo, el derivado del ciclo actual se registra como ocurrencia suya. Si el `disk` 157 llega **después** que un derivado ya agrupado, esos grupos derivados se resuelven con nota de «absorbido por la extracción imprevista» y sus eventos se re-registran bajo `device.removed_unexpected` (caso poco frecuente, con prueba propia) |
| J.50 | **DECIDIDO** e implementado (spec `003`, research.md D5). Cómo llega al detalle de una alerta de evento el suceso que la disparó | `alert_occurrences.triggering_event_id` (columna que ya existe en el esquema, hoy nunca escrita) se empieza a rellenar. El DTO de ocurrencia gana `triggeringEventId: number \| null`; el detalle de alerta muestra, para las filas que lo tengan, un enlace `<a href="/events?focus=<id>">` a la pantalla de sucesos, que ya renderiza el XML crudo y la etiqueta de certeza. No se duplica el contenido del evento dentro del detalle de alerta |
| J.51 | **DECIDIDO** e implementado (spec `003`, research.md D6). Alcance de la primera activación del puente de eventos: ¿evalúa los eventos ya ingeridos con anterioridad? (clarify Q1 → «solo hacia delante») | **Sin marcador de corte nuevo.** El puente evalúa solo los eventos que `repo_varios::insert_event_if_new` devuelve como nuevos (`Ok(true)`) en ese ciclo. Los eventos ya presentes en `system_events` al desplegar nunca se re-leen (el bookmark del canal está por delante) y, si el bookmark se invalidara y el canal se releyera entero, `insert_event_if_new` devuelve `Ok(false)` para los conocidos → no se evalúan. El «punto de corte» lo da el bookmark existente (J.7) más la unicidad `(channel, record_id)` |
| J.52 | **DECIDIDO** e implementado (spec `004-navegacion-sin-congelacion`, ADR-042). Usando la aplicación real, el usuario reportó que casi siempre que cambiaba de sección en el sidebar la interfaz se congelaba varios segundos y parecía colgada. Causa: el bucle de recopilación retenía el mutex de `AppState.conn` mientras lanzaba `smartctl.exe` (cascada de hasta 75 s/disco), dormía entre muestras PDH y leía el registro de eventos; cualquier `load` de ruta que hiciera `conn.lock()` esperaba todo ese tiempo. J.39 solo cubría el reparto de dominios de fallo, no la retención del candado | `refresh_smart`, `refresh_metricas_rendimiento` y `refresh_events` pasan a **tres fases**: candado breve para planificar → E/S externa **sin candado** → candado único para persistir. Guarda `AppState.recoleccion_smart: Mutex<()>` para que el refresco manual siga esperando al ciclo en curso sin retener `conn`. `refresh_inventory` ya cumplía y no se toca. Como red de seguridad —no como sustituto de que la interfaz responda al instante— `AppShell` pinta una barra de progreso fina arriba mientras `navigating` sea no nulo, con retardo de 150 ms para no parpadear. Descartada una 2ª conexión de solo lectura: abre `SQLITE_BUSY` real y no arregla el lado escritor (`set_setting`, `acknowledge_alert`) |
| J.53 | **DECIDIDO** e implementado. Usando la aplicación real, el usuario notó que el icono de la bandeja era un cuadrado de color liso (indistinguible a 16 px de otras aplicaciones) y que el texto emergente decía solo «Todo en orden», sin nombrar a qué aplicación pertenece. `decisions.md` (línea 106) ya marcaba el systray como pantalla sin revisión visual; esto es un primer paso, no el rediseño fino | El icono pasa a un **tile redondeado del color de estado B.5 + un glifo que también cambia con el estado**: cilindro de datos lleno (todo en orden), con «!» (advertencia), con «×» (crítico), hueco (sin datos / sin discos / fallo de recopilador), dos barras (en pausa). Así el color no es el único portador de significado (constitución §VII) y se distingue a 16 px. Se sigue generando en memoria (búfer RGBA supermuestreado 4×, dibujo procedural, sin biblioteca ni fichero `.ico`). El texto emergente pasa a `«SmartDisk Monitor — <resumen>»` (clave `tray.tooltip`), aplicado en `instalar()` y `actualizar()`. Ayuda de QA: `cargo test volcar_iconos_bmp -- --ignored` vuelca los cinco iconos a `src-tauri/target/bandeja/`. El rediseño visual completo del systray (y del resto de pantallas del Apéndice C de `ui-design.md`) sigue abierto |
| J.54 | **DECIDIDO** e implementado. Usando la aplicación real, el usuario notó que el icono de arriba del riel (logo de marca) y el primero de la navegación («Panel general») usan el mismo icono (`diskStack`) y **llevan los dos a `/`** — redundante—, y que el logo tiene un fallo de hover (la regla global `a:hover { color: --sdm-accent-fg }` teñía de violeta el icono blanco sobre el degradado de acento, que quedaba como un cuadrado). El boceto `design/.../Sidebar.md` sí dibujaba un logo aparte | **Se quita el logo.** En un riel de solo iconos no aporta: «Panel general» ya va a `/` con el icono del disco, y la identidad de la app está en la barra de título y en «Acerca de». Con ello desaparece también el fallo de hover. `docs/ui-design.md` no menciona el logo (solo «solo iconos con `title`+`aria-label`»), así que no hay que tocarlo; el boceto de `design/` no es normativo (AGENTS.md §«Fuentes de verdad»). La clave i18n `app.name` se conserva (la usan la bandeja y «Acerca de») |
| J.55 | El usuario preguntó por qué su disco Toshiba en `E:` (y, se descubrió al investigar, también el WDC de esta misma máquina) aparecía siempre como "sin datos SMART" pese a ser perfectamente legible desde el Explorador de Windows — el registro de desarrollo mostraba `NingunModoFunciono` en los cinco modos de la cascada, en cada ciclo, durante horas, mientras los dos discos NVMe de la misma máquina nunca fallaban | **Bug real en `ejecutar_con_limite` (`collectors/smartctl.rs`), no una limitación del disco ni de `smartctl`**: leía los pipes de `stdout`/`stderr` del proceso hijo solo después de que `try_wait()` confirmara su salida, en vez de mientras el proceso seguía vivo. `smartctl -a -j` contra un disco SATA con la tabla de atributos completa produce una salida (10-13 KB medidos en el WDC y el Toshiba de esta máquina) que supera el búfer del pipe que da Windows; el proceso se bloqueaba en su propio `write()` esperando a que alguien leyera, `try_wait()` nunca lo veía terminar, y los 15 s del límite se agotaban siempre — en los cinco modos, indistinguible de un dispositivo que de verdad no responde. Los dos NVMe de la misma máquina producen una salida más corta (7 KB) que nunca superaba el búfer, así que nunca lo mostraban: parecía un problema de SATA hasta medirlo. Reproducido de forma aislada (sin `smartctl.exe`, con cualquier proceso que escriba lo bastante) mediante `System.Diagnostics.Process` replicando el mismo patrón de sondeo, y confirmado que `smartctl -a -j` contra ambos discos devuelve datos SMART completos y reales (temperatura, horas de encendido, tabla de atributos) en cuanto se drenan los pipes correctamente. Corregido vaciando los pipes en dos hilos aparte mientras el hilo principal solo vigila la salida del proceso; prueba de regresión en `collectors::smartctl::tests` que reproduce el bloqueo con un proceso genérico, sin depender de hardware real ni de `smartctl.exe` |
| J.56 | El arreglo de J.55 no resolvió el síntoma: el mismo disco Toshiba seguía "sin datos SMART" tras recompilar y relanzar varias veces. Registro enriquecido temporalmente en `query_device_json` reveló que el fallo era instantáneo (no un agotamiento de los 15 s), con `exit_status: 2` ("apertura fallida") en los cinco modos y sin ningún mensaje de error — el disco se identificaba (`"device": {"type": "ata"}`) pero no llegaba a leerse nada más | **Windows Defender, Control de acceso a carpetas**, confirmado sin ambigüedad en el registro de eventos de Windows (`Microsoft-Windows-Windows Defender/Operational`, id 1127, exactamente a la hora del fallo): "El acceso controlado a carpetas impidió que smartctl.exe realizara cambios en la memoria" contra `\Device\Harddisk0\DR0` (WDC) y `\Device\Harddisk1\DR19` (Toshiba). Esta protección anti-ransomware bloquea el comando ATA PASS THROUGH que `smartctl` necesita para SATA, aunque solo lea — el Explorador de Windows nunca lo dispara (E/S de archivos, camino distinto) y NVMe tampoco (otro camino de E/S), lo que explica por qué solo fallaban los dos discos SATA de la máquina. Resuelto con ADR-043: el instalador añade la excepción con `Add-MpPreference` (`src-tauri/windows/hooks.nsh` + `defender-exception.ps1`), con `Remove-MpPreference` simétrico al desinstalar; la aplicación ofrece reintentarlo desde la pantalla de detalle de disco (`check_smartctl_defender_exception`/`add_smartctl_defender_exception`) para cuando la Protección contra alteraciones de Defender bloquea el cambio del instalador en silencio, o para quien activa la protección después de instalar. `HeroPanel.svelte` (panel general) queda deliberadamente sin este detalle específico — ver ADR-043, "Consecuencias" |
| J.57 | Probando el instalador real (no `pnpm app:dev`), el usuario reportó dos síntomas juntos: (a) varias ventanas de PowerShell parpadeando al arrancar, y (b) la aplicación quedándose "(No responde)" justo después — una vez en el asistente inicial con "Hemos encontrado 0 discos" (paso 2), otra en "Primera lectura en marcha" (paso 4) con la barra de progreso congelada a media carrera. Reiniciando la aplicación varias veces, acabó funcionando y mostrando los cuatro discos con SMART correcto | **Dos causas independientes, ambas en cómo se lanzan los procesos externos, ninguna nueva de esta sesión pero nunca antes ejercitadas contra una instalación real recién hecha**. (1) Ningún `Command::new("powershell.exe")` llevaba `CREATE_NO_WINDOW`: Windows asigna una consola nueva al lanzar un proceso de este tipo desde una aplicación sin terminal propia, y la ventana parpadea aunque el proceso termine en milisegundos — `-WindowStyle Hidden` no lo evita, porque la ventana ya existe antes de que PowerShell decida nada sobre su estilo. `platform::autoarranque` ya lo sabía y lo aplicaba a mano para `schtasks`; los demás puntos no. (2) `windows_storage::list_physical_disks` y `capacidad::list_volumes` usaban `Command::output()`, que espera **sin límite de tiempo**: un WMI lento a inicializar (más probable justo después de instalar, o nada más arrancar Windows, que es exactamente cuando el asistente hace su primer barrido) bloquea el hilo que llama para siempre en vez de devolver "sin discos todavía", que es lo que ya hace cualquier otro fallo de esta consulta. La barra de progreso "congelada a medio camino" del paso 4 no es un tercer bug: es `<ProgressBar indeterminate>` (deliberadamente no ligada a un porcentaje real, `onboarding/+page.svelte`), fotografiada a media animación en el instante exacto en que toda la aplicación dejó de repintarse por (2) — se resuelve solo en cuanto (2) deja de bloquear. Corregido extrayendo `ejecutar_con_limite` (ya escrita para J.55) a `platform::proceso_externo`, compartida por los cuatro puntos que lanzan PowerShell (`windows_storage`, `capacidad`, `proteccion_carpetas` ×2) y por `smartctl.rs`, con `CREATE_NO_WINDOW` aplicado siempre y un límite de 20 s en las dos consultas de inventario. Quedan sin tocar, a propósito, los dos `Command::output()` de `ejecutar_autotest_corto` (`commands/mod.rs`, iniciar/cancelar el autotest SMART manual): son acciones iniciadas por el usuario, no parte del barrido automático de arranque, y su alcance no lo pidió esta tarea — mismo criterio de no ampliar sin que haga falta |
| J.58 | Usando la aplicación real ya instalada, el usuario señaló cuatro cosas sueltas: (1) el icono del fondo del riel lateral no explica nada al pasar el ratón ni hace nada al pulsarlo; (2) el detalle de una alerta no dice a qué disco corresponde, aunque la lista de la izquierda sí lo hace; (3) la alerta `smart.error_log` ("el registro de errores del disco ha aumentado") solo enseña un contador que sube, sin ninguna pista de qué error es; (4) la leyenda "Duración del silencio" queda descuadrada respecto a los botones de al lado | Cuatro causas independientes, todas ya resueltas. **(1)** El icono es un indicador de estado pasivo (`role="status"`, misma fuente que la píldora de la `Toolbar`, que tampoco es clicable — coherente con el resto de la app) al que le faltaba el `title` que sí llevan los demás iconos del riel: añadido, sin hacerlo interactivo. **(2)** `detail.target` ya llegaba al frontend (`AlertDetail` hereda `target` de `AlertGroupWire`) pero nunca se pintaba en el panel de detalle: añadida una línea bajo el título, igual que ya se ve en la tarjeta de la lista. **(3)** SMART no da una descripción legible de cada error — el contador (`error_log_entries_total`) es el dato real; lo más parecido a "más información" es la tabla de errores completa que trae el JSON entero de `smartctl`, que **ya se genera hoy** dentro del paquete de diagnóstico pero no estaba enlazada desde la alerta. Nuevo comando `get_alert_smart_raw_json` (mismo patrón que `get_event_raw_xml` para las alertas de sucesos: se resuelve el `target_device_id` internamente en el backend, la ruta de `smartctl` nunca viaja al frontend) que consulta smartctl al momento y lo muestra con el mismo `CodeOutput` que ya usan `chkdsk` y el XML de eventos; solo se ofrece en alertas `smart.*`/`temp.*`/`nvme.*` (`docs/alert-rules.md`, columna "Fuente"), nunca en `capacity.*`/`events.*`/`device.*`, que no tienen ningún JSON de smartctl que mostrar. **(4)** Maquetación: `Select` es el único control de esa fila con su propia etiqueta encima, y centrar verticalmente toda la fila la descuadraba frente a los botones sin etiqueta — la fila pasa de `items-center` a `items-end` |
| J.59 | **DECIDIDO** e implementado. El usuario ve «Desgaste 5 %» en una tarjeta de disco y no sabe qué significa ni si es preocupante; quiere un tooltip que lo explique al pasar el ratón, en el panel general y en el detalle de disco, con un veredicto sobre el valor actual | Se construye el componente **`Tooltip`** (que `ui-design.md` §3 ya tenía autorizado y pendiente) y un módulo `src/lib/design/metricHelp.ts` con `veredictoMetrica` (puro) + `ayudaMetrica` (texto traducido). El veredicto («normal» / «alto» / «demasiado alto») usa `classifyAgainstThresholds` y los umbrales de `settings.alerts`, así **nunca contradice** al color de la tarjeta ni a una alerta; actividad y horas de encendido son informativas (siempre `ok`), y un disco SATA sin desgaste lo explica. **Alcance**: 3 métricas de `DiskCard` (panel) + las 4 `MetricCard` (detalle); **no** la tabla «Contadores». **Panel: tooltip solo con el ratón**, porque la `DiskCard` es un `<a>` entero y no puede contener un elemento tabulable — con teclado, la versión completa (`Tooltip focusable`, `Escape`, `aria-describedby`, WCAG 1.4.13) está en el detalle. En la `DiskCard` el tooltip es **local y ligero** (no el componente `Tooltip`): con 20 discos serían 60 instancias y el panel debe pintarse rápido (SC-006, `e2e/ui/rendimiento.spec.ts`); el silencio `a11y_no_static_element_interactions` está en `known-issues.md` #4. El panel pide `settings` una vez sin bloquear el pintado; hasta que llega, `metricHelp` usa los umbrales de fábrica. Textos en `metric.help.{temperature,wear,activity,powerOnHours}.*` |
| J.60 | **DECIDIDO** e implementado (ADR-045). Sobre la aplicación real, el usuario señaló que una alerta `smart.error_log` de su NVMe Crucial `CT2000P3SSD8` (contador en 2162) parecía un fallo de disco pero, al mirar el registro de errores, **todas** las entradas eran `"Invalid Field in Command"` (`status_code_type` 0, `status_code` 2) con `media_errors` 0, `critical_warning` 0, `smart_status.passed` verdadero y `percentage_used` 3 — no es daño, y aun así no se podía ignorar porque `smart.error_log` estaba en el conjunto vetado de ADR-044. «Quizá hemos sido demasiado radicales» | **`smart.error_log` sale de `REGLAS_NO_IGNORABLES`** (ADR-044 → seis reglas). En NVMe de consumo ese contador (`num_err_log_entries`) lo dominan rechazos de protocolo benignos: `smartctl` o Windows piden una página de log opcional que la controladora no implementa y esta apunta cada comando rechazado. El daño de medio real lo sigue cubriendo `smart.media_errors`, que **no** se toca y sigue vetada. Cambio de una línea en `alerts::reglas` + su prueba + `docs/alert-rules.md` §1 (los tres juntos, como pide ADR-044), más `ui-contract.md` §3.4 y `ui-design.md` §3. **Pendiente, spec propia**: afinar la regla para que solo dispare con entradas del registro de tipo «media/integridad» (NVMe `status_code_type == 2`) en vez del contador bruto — es el arreglo de raíz, toca el parser de `smartctl` y cambia comportamiento observable; poder ignorarla ya resuelve el caso mientras tanto |
| J.61 | **DECIDIDO** (ADR-054). Al activar la ayuda con IA con la **clave de demostración compartida**, ¿qué modelo queda seleccionado, dado que esa clave solo cubre modelos gratuitos? | `activar_ayuda_ia_compartida` fija `settings.ai.model` en el **router automático** (`openrouter/free`). No se añade una validación que impida luego cambiar a un modelo de pago: si la persona lo hace, OpenRouter rechaza la petición y se muestra el error `ia.*` habitual — misma degradación que ya existe, sin código nuevo para un caso que el proveedor ya cubre. La ofuscación XOR de la clave (patrón fijo en `src-tauri/src/platform/clave_demo_ofuscacion.rs`, incluido también por `build.rs`) **no es cifrado** y así se declara: la clave es extraíble del ejecutable, su valor es la comodidad, no la confidencialidad |
| J.62 | **PROPUESTO** (spec 008 / ADR-053, `research.md` D3). Valores numéricos del benchmark con DiskSpd, aún sin medir contra hardware real variado: **`D_OBJETIVO` = 5 s** (ventana de medición por medición; DiskSpd/CDM usan 5–10 s), **`D_MIN` = 2 s** (por debajo, la cifra es ruido; suelo de medición), **`D_CALENTAMIENTO` (`-W`) = 2 s** (descarta caché SLC y colas frías; escribe pero no cuenta como medido), **`TOPE_DATOS` por medición de escritura = 4 GiB** (techo de desgaste por perfil; la lectura no desgasta → sin tope), **`TAMANO_ARCHIVO` (`-c`) = 1 GiB** (como el motor anterior; DiskSpd itera sobre él, no lo hace crecer). **Perfiles fijos** (4): `seq1m_q8` (`-b1M -o8 -s`), `seq1m_q1` (`-b1M -o1 -s`), `rnd4k_q32` (`-b4K -o32 -r4K`), `rnd4k_q1` (`-b4K -o1 -r4K`); todos `-t1`, `-Sh`, `-Z1M`, `-L`. La `-d` de cada escritura = `clamp(TOPE_DATOS / caudal_lectura_del_mismo_perfil, D_MIN, D_OBJETIVO)`. **Tensión conocida** (D3): en discos muy rápidos (Gen5, ~12 GB/s) el tope de 4 GiB se agota antes de `D_MIN`; en ese caso se mide `D_MIN` igualmente y el resultado se **etiqueta** «disco muy rápido: se escribieron ~N GiB para poder medir con fiabilidad» — una medición de 0,3 s no es un dato. El resultado siempre reporta la `-d` real y los bytes realmente escritos (de la salida de DiskSpd). Peso total de escritura estimado de la matriz: ~1,5 GiB (HDD) a ~30–50 GiB (Gen5, etiquetado); < 0,01 % de la resistencia TBW de cualquier SSD. Se confirmará contra hardware real en la validación manual del `quickstart.md` (T052) |

---

## K. Pendiente de decisión

Cerradas desde la última revisión:

- **K.1** (tipografía empotrada), 2026-09-04 — los dos `.woff2` de Instrument Sans v4 y su `OFL.txt`
  están en `src/design-system/fonts/`, declarados en `tokens.css` con `unicode-range` y
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
- **K.6** (cobertura de «resto de `src-tauri/src/`»), 2026-09-06 — el usuario eligió la vía (a):
  enmendar el principio VIII (constitución 1.6.0) en vez de aceptar el 80 % como excepción
  permanente. Medido de nuevo en esta fecha con `cargo llvm-cov`, ya con todo lo añadido desde
  T108 (planificador, ajustes, informes): **69,80 %** — el déficit había bajado de 72,50 % porque
  `commands/mod.rs` creció mucho más rápido que sus pruebas. Se descartó separar los envoltorios
  `#[tauri::command]` a un fichero excluido de la medición al comprobar, comando a comando, que
  la premisa no se cumplía para unos 20 de los 35: tienen lógica real escrita directamente en el
  comando, sin ningún `_impl` que la recoja (`pause_monitoring`/`resume_monitoring`,
  `acknowledge_alert`/`mute_alert`/`unmute_alert`/`archive_alert`, `refresh_now`,
  `run_chkdsk_scan`/`run_smart_short_test`/`cancel_test`/`start_benchmark`…). Excluirlos tal cual
  habría escondido lógica sin probar detrás de la excepción, no solo pegamento no instanciable.
  El mínimo de esa fila baja a **69 %** (con margen sobre el 69,80 % medido) en vez de mantener
  una excepción de dos casos que no encajaba con el código real; extraer esos ~20 comandos a sus
  propias funciones `_impl` con prueba —lo que sí permitiría separar y excluir el envoltorio de
  verdad— queda como mejora futura, no bloqueante, y subiría el mínimo de nuevo cuando se haga.


| # | Cuestión | Por qué no se ha decidido |
|---|---|---|
| K.7 | Implementando J.46 (zonas de aviso/crítico en el gráfico de temperatura) se encontró que `alerts::motor::evaluar_temperatura_configurada_warn`/`_crit` llevan los umbrales **literales** (`70.0`/`80.0`/`67.0`/`75.0`) en vez de recibir `AlertSettings.temp_configured_warn_c`/`_crit_c` — los mismos campos que `settings.alerts` ya persiste y que la pantalla de Ajustes ya deja editar (`settings.alerts.tempWarn`/`tempCrit`). Cambiar el ajuste en la interfaz no tiene ningún efecto sobre qué alertas se disparan de verdad | **No se ha tocado el motor de alertas en esta tarea**: era un cambio de comportamiento de alertas ya en producción, fuera del alcance autorizado (una mejora de UX en el gráfico de temperatura), y la constitución exige tratar un cambio de comportamiento observable como historia propia, no colarlo dentro de otra. El frontend (`disks/[id]/+page.svelte`, `src/lib/design/health.ts::temperatureThresholds`) sí lee `settings.alerts.tempConfiguredWarnC/CritC` para las zonas del gráfico y el color de la cifra grande, que es el comportamiento **documentado y pretendido** (`alert-rules.md`, columna "configured"): así, en cuanto se corrija el motor, backend y frontend coincidirán sin tocar la interfaz de nuevo. Mientras tanto, un usuario que cambie el umbral en Ajustes verá el gráfico reflejar su cambio, pero las alertas reales seguirán disparándose a 70/80 °C — una discrepancia real que corregir es tarea aparte (pasar `configuradas: (f64, f64)` a `evaluar_temperatura_configurada_warn`/`_crit`, leído de `AlertSettings` en el punto de la llamada, con sus pruebas de umbral actualizadas) |

---

## L. Escala, densidad y escalado de Windows — medido

Cerrada el 2026-09-04. Banco de pruebas: `tools/scale-check.html`, que reproduce la composición
normativa con los tokens reales. Medido en un navegador Chromium con la tipografía ya empotrada.

### L.1 · La escala tipográfica está bien. La sospecha era infundada

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

### L.2 · Lo que sí falla: el espacio en píxeles CSS

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

### L.3 · Lo que falla siempre: "No disponible" no cabe en una `MetricCard`

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

### L.4 · Las cuatro correcciones, verificadas

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

### L.5 · Queda un detalle de afinado

Con `auto-fit`, una tarjeta estrecha reparte las cuatro métricas en 3 + 1 en lugar de 2 × 2. No
recorta nada y se ve correcto, pero 2 × 2 sería más regular. Es una decisión de composición para
quien diseñe la `DiskCard` definitiva, no un defecto.

---

## M. WebView2 en Windows Server — resuelto

Cerrada el 2026-09-04 contra la documentación oficial de Microsoft, sin necesidad de ensayo en
hardware. Decisión completa en el ADR-020.

### M.1 · La matriz de sistemas de la especificación es correcta

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

### M.2 · Lo que sí era un problema real

**El runtime no viene preinstalado en Windows Server, en ninguna versión.** Solo Windows 11 lo
incluye como parte del sistema; en Windows 10 lo tiene la gran mayoría de equipos porque Microsoft
lo desplegó por Windows Update desde diciembre de 2022. En un servidor recién instalado, la
aplicación sencillamente no arrancaría.

### M.3 · Qué se descartó y por qué

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

### M.4 · Lo que queda por hacer

- Declarar `webviewInstallMode: { "type": "offlineInstaller" }` en `tauri.conf.json`.
- Comprobar en tiempo de ejecución que el runtime está presente y, si no, mostrar una frase
  comprensible en lugar de una ventana en blanco. La detección oficial es la clave del registro
  `pv` en `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`,
  con valor mayor que `0.0.0.0`.
- Indicar el tamaño del instalador (~140 MB) en la página de descarga.
- Verificar en Fase 0 que la instalación silenciosa funciona en un Windows Server 2019 limpio y sin
  salida a Internet. Es lo único que sigue requiriendo una máquina de verdad.

---

## N. smartctl: versión y licencia — resuelto

Cerradas K.2 y K.3 el 2026-09-04. Decisión completa en el ADR-021; detalle operativo en
`third-party/smartmontools/README.md`.

### N.1 · Versión elegida

**smartmontools 7.5**, publicada el 12 de mayo de 2025 (compilación r5714). Es la última estable.
El binario ya está en `third-party/smartmontools/`, con sus sumas MD5 verificadas contra las que
publica el propio proyecto y contra el `checksums64.txt` que viaja dentro del paquete oficial.

Un detalle que conviene saber: el paquete oficial de Windows se llama `win32-setup` por razones
históricas, pero **contiene las dos arquitecturas**. El de `bin/` es x64 —verificado leyendo la
cabecera PE, máquina `0x8664`— y el de `bin32/` es x86. No hay ZIP portable: hay que extraer el
instalador.

### N.2 · Hacía falta un fichero que no estaba en ninguna parte de la documentación

`drivedb.h` (268 KB) es la base de datos de unidades de smartmontools. **Sin ella, `smartctl` no
sabe interpretar los atributos específicos de cada fabricante** y los presenta como desconocidos,
que es justo la información que hace útil a un monitor de discos. No aparecía mencionada en ninguno
de los documentos del proyecto. Se empaqueta junto al binario.

Queda congelada con la versión: el script oficial que la actualiza (`update-smart-drivedb.ps1`)
descarga de Internet, así que no se distribuye. Los modelos de disco muy recientes podrían no ser
reconocidos hasta que se actualice la versión de smartmontools; es una limitación conocida, no un
fallo.

### N.3 · Qué se deja fuera

`smartd` y sus utilidades de notificación. La aplicación ya tiene su propio planificador, y un
segundo vigilante competiría por el acceso a los dispositivos. También los binarios de 32 bits.

### N.4 · La GPLv2, resuelta por la vía 3(a)

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

### N.5 · Dos comprobaciones hechas sobre el binario real

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

## O. Contraste del acento heredado — medido

Cerrada el 2026-09-04. Herramienta: `tools/accent-check.py`, que barre el espacio sRGB completo
(262.144 colores, paso 4) contra las superficies efectivas de ambos temas.

### O.1 · La pregunta estaba mal planteada

La duda original hablaba de "los 48 acentos de Windows". **No existe tal lista cerrada**: Windows
ofrece una cuadrícula de sugerencias, pero el usuario puede elegir cualquier color con un selector
completo. La validación por muestreo no servía; había que barrer el espacio entero.

### O.2 · Lo que ya funcionaba

`accessibleAccent()`, el ajuste del acento **como fondo** del botón primario:

| | |
|---|---|
| Colores por debajo de AA tras el ajuste | **0 de 262.144** |
| Colores que necesitaron retoque | 9.477 (3,6 %) |
| Mayor desviación aplicada | 26/255 en un canal, imperceptible |

Validado sin cambios.

### O.3 · Lo que no se había mirado, y fallaba

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

### O.4 · El sistema de diseño ya tenía la respuesta, y el código la destruía

Los respaldos de `tokens.css` estaban bien elegidos, con un tono distinto por tema:

| Tema | Respaldo | Contraste sobre su material |
|---|---|---|
| Claro | `#0067c0` | 5,40:1 ✔ |
| Oscuro | `#3d95ea` | 5,09:1 ✔ |

El defecto estaba en `applySystemAccent()`, que **sobrescribía los dos con el mismo color plano del
sistema**, borrando justamente la distinción que hacía que funcionaran.

### O.5 · Windows ya deriva los tonos que hacen falta

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

### O.6 · La corrección, verificada

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

### O.7 · Un detalle para quien escriba el backend

`HKCU\Software\Microsoft\Windows\DWM\AccentColor` guarda el color en **ABGR**, no en RGB.
Leerlo como RGB devuelve el color invertido: el azul de fábrica (`0xFFD47800`) saldría naranja
`#D47800` en vez de azul `#0078D4`. El comando `get_system_accent_color` debe devolver además la
`AccentPalette` cuando esté disponible, porque es lo que alimenta el punto 2.

---

## P. Eventos de Windows — verificados

Cerrada K.5 el 2026-09-04. La lista normativa vive en `alert-rules.md` §3; aquí queda el método y
lo que cambió.

### P.1 · Método

Dos fuentes, ninguna de ellas un blog:

1. **Los manifiestos de proveedor** del propio Windows (`Get-WinEvent -ListProvider`), que declaran
   cada evento con su identificador, su nivel y su plantilla de mensaje.
2. **180 días de registro `System`** de un equipo real: 2.038 eventos de almacenamiento sobre 32.620
   totales, agrupados por proveedor, identificador y nivel.

La segunda fuente es la que no se puede sustituir por documentación: dice **qué es ruido de fondo**,
que resulta ser la pregunta importante.

### P.2 · El hallazgo que habría hundido el producto

`disk` 51 —"Error detectado en el dispositivo durante una operación de paginación"— apareció
**839 veces en 180 días en un equipo sano**. Es, con diferencia, el evento de almacenamiento más
frecuente de Windows, y es benigno: se dispara al desconectar un medio extraíble, al despertar un
disco o ante cualquier reintento que el sistema resuelve solo.

La lista tentativa de la especificación lo clasificaba como **crítico**. Habría producido 839
alertas críticas falsas en un equipo sin ningún problema. Un monitor que grita todos los días deja
de leerse, así que eso no habría sido un defecto menor: habría inutilizado el producto.

Ahora es advertencia, solo sobre discos no extraíbles y con umbral de ≥ 10 en una hora.

### P.3 · Tres errores más de clasificación

| Antes | Realidad |
|---|---|
| `Ntfs` 98 → "metadatos inconsistentes", crítico | Es `Microsoft-Windows-Ntfs` 98, de nivel **Información**, y significa que el volumen **está bien**. Observado 319 veces: otros 319 falsos críticos |
| `Ntfs` 130 → "marcado para comprobación" | 130 es "se **reparó** la estructura", una advertencia leve. El que indica daño irreparable es el **131**, que no estaba en la lista |
| `volmgr` 46 y 49 | No existen en el sistema. El `volmgr` que sí aparece (161) es un fallo al crear el volcado de memoria y no dice nada del disco |

### P.4 · Lo que faltaba

- **`disk` 157, "El disco se ha extraído de forma imprevista"**. Es exactamente el evento que
  necesita la regla `device.removed_unexpected`, y no estaba. Observado 63 veces.
- `disk` 158, dos discos con identificadores duplicados. Confirma que la colisión de identidad de
  §F.2 no es teórica.
- `Ntfs` 50 (fallo de escritura demorada, 367 veces) y `Microsoft-Windows-Ntfs` 140 (no se pudo
  vaciar el registro de transacción, 173 veces).
- **El proveedor `Microsoft-Windows-NvmeDisk` entero**, con su evento 500 (comando NVM con error) y
  501 (caché de escritura habilitada, que es informativo).
- `Microsoft-Windows-StorageSpaces-Driver`, con eventos muy concretos para disco virtual degradado.

### P.5 · Un cambio de diseño en el motor, no solo de lista

El dato más útil no fue ningún identificador suelto, sino el patrón: **un solo hecho físico produce
una ráfaga de eventos distintos**. Al desconectar en caliente un disco externo, el mismo dispositivo
generó en segundos un `disk` 157, un `disk` 51, un `Ntfs` 50 y un `Microsoft-Windows-Ntfs` 140.

Con la deduplicación por `provider:event_id` que decía la especificación, ese único suceso habría
creado **cuatro grupos de alerta**. El motor necesita, además, una **ventana de correlación de
60 segundos por dispositivo**, con una regla de causa: si en la ráfaga hay un `disk` 157, ese es el
suceso y los demás son ocurrencias suyas. Recogido en `alert-rules.md` §3.5.

### P.6 · Qué sigue pendiente

La muestra es de **un** equipo Windows 11 en español, con NVMe, SATA y discos externos USB. No cubre
servidores, RAID por hardware ni Storage Spaces en producción: esos eventos están tomados de los
manifiestos, no observados. Sigue en la Fase 0 contrastarlos en un servidor.

Y una obviedad que conviene decir: un equipo sano no produce eventos de fallo real, así que la
ausencia de `Ntfs` 55 o `disk` 7 en la muestra es una buena noticia, no una señal de que no existan.

---

## Q. Codificación de la salida de los procesos auxiliares — medido

Cerrada I.3 el 2026-09-04. Herramienta y heurística de referencia: `tools/console-encoding.py`.

### Q.1 · La suposición de partida era falsa

La especificación decía que la salida de `chkdsk` llegaría en la página OEM de la consola, CP850 en
un Windows en español. **No es así.** Capturando los bytes crudos de un proceso sin consola
(`CreateNoWindow`) con la salida por tubería, que es exactamente como lo lanzará Rust:

```
chkdsk  ->  E1 E9 ED F1 F3 FA
            como CP1252: áéíñóú      ✔
            como CP850 : ßÚÝ±¾·      ✘
```

`chkdsk` emite **CP1252, la página ANSI**, no la OEM.

### Q.2 · Y el problema real es peor: Windows no es consistente consigo mismo

En el mismo equipo, el mismo día, con el mismo tipo de tubería:

| Herramienta | Bytes de las vocales acentuadas | Página |
|---|---|---|
| `chkdsk` | `E1 E9 ED F1 F3 FA` | **CP1252** (ANSI) |
| `chkntfs` | `E1 E9 ED F1 F3` | **CP1252** (ANSI) |
| `fsutil` | `A0 A1 A2` | **CP850** (OEM) |
| `vssadmin` | `A0 A1 A2 A3 A4` | **CP850** (OEM) |

No hay una regla del sistema que seguir: depende de cómo se escribió cada herramienta. Cualquier
constante que se codifique acertará con unas y producirá basura con otras.

### Q.3 · La solución evidente no funciona

Fijar la página de códigos antes de invocar, con `chcp 850` o `chcp 65001`, **no cambia nada** si la
salida está redirigida. Comprobado: los volcados de `chkdsk` con la página heredada, con 850 y con
65001 salieron **byte a byte idénticos**, mismo MD5 los tres. La página de consola gobierna lo que
se pinta en una consola, no lo que se escribe en una tubería.

### Q.4 · Detección, validada

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

### Q.5 · Reglas que se derivan

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

## R. Instancia única y ACL de `ProgramData` — medido

Cierra I.6 el 2026-09-04. Decisiones resultantes: ADR-025 (instancia única) y ADR-026 (ACL).

### R.1 · Eran dos preguntas, no una

I.6 juntaba dos cosas sin relación técnica. Separadas:

- **Instancia única.** El requisito real no es «bloquear la segunda», sino «abrir una segunda
  restaura la ventana de la primera». Eso obliga a comunicar dos procesos, no solo a detectarse.
  Resuelto con `tauri-plugin-single-instance` (ADR-025).
- **ACL de la carpeta de datos.** Aquí estaba el hallazgo.

### R.2 · La suposición de partida era falsa

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

### R.3 · Verificado: un usuario sin privilegios se apropia de la carpeta

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

### R.4 · El endurecimiento funciona, y el orden importa

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

### R.5 · SID numéricos, no nombres de grupo

En esta máquina el grupo es `Administradores`; en un Windows en inglés, `Administrators`; en
francés, `Administrateurs`. Un instalador escrito con nombres falla fuera de su idioma. Se usan
siempre `*S-1-5-18`, `*S-1-5-32-544` y `*S-1-5-32-545`.

### R.6 · Reglas que se derivan

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

## S. Paleta v3 «Ciruela» — ratios de contraste medidos

Cerrada el 2026-09-06 al implantar el rediseño v3 (ADR-034, `specs/002-rediseno-v3/`, US1).
Herramienta: `scripts`/`tools/accent-check.py` reutilizado para componer cada color sobre el
material real (`--sdm-glass` sobre la media del degradado del lienzo; y `--sdm-glass-3` encima, para
la columna «bloque interno»; la columna «píldora» mide el color contra su propio `-soft` compuesto
sobre el material, que es el caso de `StatusPill`). Mínimo exigido: 4,5:1 (constitución §VII, WCAG
1.4.3), medido **como texto de píldora**, que es el uso más exigente.

### S.1 · Resultado

Todos los tokens de texto y de salud de la paleta Ciruela cumplen AA en los dos temas, sobre
material y sobre bloque interno. Los valores medidos coinciden con la tabla que entregó el diseñador
en `design/propuesta-redisenov2/cambios/00-tokens.md` dentro de ±0,05.

| | material (claro / oscuro) | bloque interno (claro / oscuro) | píldora (claro / oscuro) |
|---|---|---|---|
| `--sdm-text` | 16,40 / 13,81 | 14,62 / 11,58 | — |
| `--sdm-text-dim` | 6,15 / 6,06 | 5,48 / 5,08 | — |
| `--sdm-text-faint` | 5,42 / 5,72 | **4,83** / **4,80** | — |
| `--sdm-accent-fg` | 6,52 / 6,86 | 5,81 / 5,75 | 5,46 / 4,82 |
| `--sdm-ok` | 5,38 / 7,62 | 4,80 / 6,39 | **4,68** / 5,56 |
| `--sdm-warn` | 6,13 / 8,11 | 5,46 / 6,80 | 5,31 / 5,78 |
| `--sdm-crit` (bermellón) | 5,82 / 5,98 | 5,19 / 5,01 | 4,86 / **4,57** |
| `--sdm-unknown` | 6,04 / 6,16 | 5,38 / 5,17 | 5,19 / 4,80 |

Texto blanco sobre el acento sólido en claro: 6,94:1 (`--sdm-on-accent` sigue siendo `#ffffff`).
`--sdm-on-accent` en oscuro pasa a tinta `#20132a`: 7,80:1 sobre el acento (en blanco daba 2,27:1).

### S.2 · Los tres valores más justos, verificados

- `--sdm-text-faint` sobre bloque interno: 4,83 (claro) / 4,80 (oscuro). El diseñador ya los había
  subido respecto a su primera propuesta (`#988ea0` daba 4,18 en oscuro); estos son los definitivos.
- `--sdm-ok` como texto de píldora en claro: 4,68. Sin margen para aclararlo.
- `--sdm-crit` como texto de píldora en oscuro: 4,57. El bermellón `#ef8080` está calibrado al
  límite: no lo aclares.

### S.3 · Reglas que se derivan

- **Ninguno de estos tokens se aclara.** Si un texto queda justo sobre el material, se sube la
  opacidad de la capa, nunca se rebaja el color (misma regla que §O y que `ui-design.md` §6).
- **No pongas texto directamente sobre `bg-glass-3`** salvo que sea uno de los tokens de esta tabla:
  la columna «bloque interno» es el suelo.
- El interruptor «usar el acento de Windows» (ADR-035) mantiene intacta la corrección de §O:
  `accessibleAccent()` / `accentOnSurface()` siguen barriendo el acento del usuario.

---

## T. Perfiles de alerta — decisiones adoptadas (ADR-036)

Cerrada el 2026-09-06 al implantar US10 del rediseño v3 (`specs/002-rediseno-v3/`).

### T.1 · El motor pasa a leer los umbrales de `settings`

Hasta v3, `alerts::motor` llevaba los umbrales **escritos a mano** (`90/100` desgaste, `70/80`
temperatura) y `settings.alerts.*` se guardaba sin que ninguna regla lo leyera — el mismo hueco que
J.32 describía para las claves de capacidad. Con ADR-036 el motor recibe los umbrales como parámetro
(`ConfigUmbrales`), que `commands::refresh_smart` resuelve de `settings` una vez por ciclo. Reglas
parametrizadas: `smart.wear_high`, `temp.above_configured_warn/crit`, `smart.media_errors` y —nuevas
en el motor— `capacity.low`/`capacity.critical`.

### T.2 · El umbral térmico de fábrica baja a 60/70 °C

Era 70/80. El perfil «Equilibrado» de `cambios/08b-perfiles-de-alerta.md` lo fija en 60/70, y ese
pasa a ser también el valor de fábrica (spec 002, clarify Q2). 60 °C sigue siendo temperatura alta
para un SSD de consumo, y mantener dos números distintos («fábrica» vs «Equilibrado») confundiría.
Actualizado `alert-rules.md` §2 y las pruebas de `alerts::motor` y `commands::set_setting`.

### T.3 · `media_errors_*` no es una ventana de 24 h

El nombre `mediaErrorsWarnPer24h` viene de la propuesta del diseñador, pero la semántica adoptada
(clarify Q1) es **el incremento de `media_errors_total` entre dos lecturas consecutivas** que basta
para avisar. No se construye una mecánica de conteo por ventana de 24 h: reinterpretar sobre la regla
existente cubre el caso. La interfaz no muestra «/24 h».

### T.4 · `driver_retry_*` se guarda pero **aún no lo consume ninguna regla**

Un perfil escribe los doce umbrales, `driver_retry_warn/crit_per24h` incluidos, para que el juego
esté completo. Pero las reglas `events.controller_reset` / `events.io_retry` que los consumirían
**no existen en el motor**: necesitan el colector de eventos de Windows completo (Historia 4). Es el
mismo patrón con el que las claves de capacidad y `logging.verbose` vivieron guardadas sin consumidor
hasta que su regla se implementó (J.32, FR-029a). Cuando exista esa regla, el umbral ya está.

### T.5 · Editar un umbral a mano rompe el perfil

`set_setting` sobre cualquier `alerts.*` (salvo `alerts.profile`) pone `alerts.profile = "custom"`.
La única forma de volver a un perfil concreto es elegirlo, y entonces se reescriben sus doce valores.
La interfaz muestra «Personalizado (a partir de \<perfil anterior\>)» derivando el «anterior» del
último `profile` no-`custom` conocido en memoria, no de un segundo campo persistido.

## U. Panel general v3 — virtualización sustituida por la variante compacta

Cerrada el 2026-09-06 al implantar US5 del rediseño v3 (`specs/002-rediseno-v3/`, PR 6).

### U.1 · Por qué desaparece la `VirtualList` de la rejilla de discos

El panel v2 envolvía la rejilla de `DiskCard` en una `VirtualList` que virtualizaba **por filas**
(J.26): con 20 discos, pintar la rejilla entera producía una tarea de ~100 ms, por encima del umbral
de 50 ms de SC-006/SC-007.

El panel v3 (`cambios/01-panel-general.md`) cambia el encuadre: ahora hay un `HeroPanel` de 246 px
arriba y una fila inferior (sucesos + reparto de estados) abajo, y **el diseñador especifica que la
región entera hace scroll** («con más discos la región hace scroll, nada se recorta»). Anidar una
`VirtualList` de altura fija solo para la rejilla, entre un héroe y un pie que también deben
desplazarse con ella, va contra ese encuadre y contra la constitución §XIV (una sola región de
scroll natural).

En su lugar se aplica lo que ya prescribía `ui-design.md` §7: **a partir de 12 discos monitorizados
la `DiskCard` pierde la sparkline de cabecera** (`conSparklines = devices.length <= 12`). El coste de
render que J.26 midió venía casi todo del SVG por tarjeta; sin él, 20 tarjetas se pintan holgadas.

### U.2 · Medido, no estimado

`e2e/ui/rendimiento.spec.ts` («recibir 20 discos en caliente … no produce ninguna tarea de 50 ms o
más») se conserva sin cambios y **pasa** contra el panel v3 con rejilla plana: `[]` tareas largas.
SC-006 se mantiene por medición, que era el objeto de J.26 — no por la técnica concreta.

### U.3 · Series de temperatura: carga perezosa por disco visible

El panel solo trae el inventario en su `load` (constitución §XIV). Tras el primer render, un
`$effect` pide `getMetricSeries("temperature_celsius", 24 h)` para el disco del héroe y —si
`conSparklines`— para el resto; el store (`app.temperatureSeries`) cachea por disco para no repetir
la petición. Un fallo por disco degrada solo esa sparkline (no se pinta) y no tumba el panel.

## V. Asistente inicial — decisiones adoptadas (US8, ADR-037/038)

Cerrada el 2026-09-06 al implantar US8 del rediseño v3 (`specs/002-rediseno-v3/`).

### V.1 · El guardián de `+layout.ts` detecta «ya configurado» con lo observable

FR-043 pide no mostrar el asistente a quien actualiza desde una versión sin él. El frontend **no
puede** saber si se guardó *cualquier* clave suelta de `settings` sin una señal nueva del backend
(`get_settings` devuelve valores resueltos, no dice cuáles son de fábrica y cuáles guardados). Se
comprueba lo que sí es observable: `theme != "system"`, `language != null`,
`settings.alerts.profile != "balanced"`, algún `device.alias`, o `excluded.length > 0`. Cubre todos
los casos realistas de actualización (quien ya usaba la app renombró un disco, cambió el tema o tocó
las alertas). **Limitación aceptada**: un usuario de v3 desde cero que solo cambió, p. ej., un día de
retención y cerró la app antes de acabar el asistente lo volverá a ver — que es justo lo que FR-043
dice que debe pasar («interrumpida antes de guardar nada vuelve a mostrar el asistente»). No se
añade backend por este caso.

### V.2 · El guardián nunca atrapa: cualquier fallo cae a «seguir normal»

Si `get_settings` / `get_devices` / `get_appearance_settings` fallan, el `load` del layout devuelve
`{}` sin redirigir. Un fallo de arranque real ya lo explica `+layout.svelte`; lo que no puede pasar
es un bucle de redirección a `/onboarding` cuando el backend no responde.

### V.3 · `notifications.enabled` y el autoarranque — verificación

- `notifications.enabled`: la decisión de enviar el toast se factoriza a `debe_enviar(...)` (pura,
  con pruebas). El resto de `alerts::notificaciones::procesar_una` sigue sin prueba automática
  porque necesita un proceso Tauri real (`research.md` R1) — mismo trato que ya tenía.
- `lifecycle.start_with_system`: `platform::autoarranque::aplicar()` lanza `schtasks.exe` y **no se
  ejecuta en `cargo test`** (crearía una tarea en el equipo del desarrollador). Se prueba el formato
  de la línea de comando (`linea_de_comando`) y el nombre estable de la tarea; el registro/borrado
  real en el Programador de tareas se verifica a mano —mismo criterio que J.28 (autotest SMART) y
  `platform/sistema.rs`—. **Pendiente**: activar el interruptor en un Windows real, comprobar en el
  Programador que existe «SmartDisk Monitor - Autostart» con «Ejecutar con los privilegios más
  altos» y disparador «al iniciar sesión», reiniciar y confirmar que la app abre elevada sin UAC.

## W. Geometría de la ventana entre sesiones — decisión adoptada (ADR-040)

Cerrada el 2026-09-06.

### W.1 · Por qué en `settings` y no con `tauri-plugin-window-state`

El plugin estándar guarda su estado en un fichero JSON propio, fuera de SQLite: choca con el
principio **V** (INNEGOCIABLE, «ningún otro almacén de datos estructurados»). Además sería
dependencia nueva (enmienda de la constitución) y permisos de Tauri nuevos. La vía elegida —cinco
claves `window.*` en la tabla `settings`, escritas solo por el backend— no necesita ninguna de esas
tres cosas. El detalle completo, en ADR-040.

### W.2 · La predeterminada 1695 × 988 y las pantallas pequeñas

Es lo que pidió el usuario para el **primer** arranque. No cabe entera en configuraciones con
mucho escalado (1920 × 1080 al 150 % deja ~1280 × 720): en ese primer arranque Windows/Tauri acota
la ventana al área de trabajo, y a partir de ahí manda la geometría que el usuario dejó, que por
definición cabía. El mínimo técnico (1024 × 560, §L) protege el caso extremo. El objetivo de
diseño (1280 × 720) no cambia: se sigue componiendo y revisando contra él.

### W.3 · Qué se verifica a mano

`geometria_visible` (la comprobación de «¿queda dentro de algún monitor?») es pura y tiene pruebas.
`aplicar_geometria_guardada` / `persistir_geometria` **no** se prueban en `cargo test`: necesitan
un proceso Tauri con ventana real, mismo criterio que `platform::autoarranque` (V.3) y
`platform/sistema.rs`. **Pendiente** (recorrido en `specs/002-rediseno-v3/regresion-visual.md` o
al empaquetar): primer arranque a 1695 × 988; redimensionar/mover/cerrar y reabrir en la misma
geometría; maximizar/cerrar/reabrir maximizada; mover a un segundo monitor, cerrarlo y reabrir sin
que la ventana quede fuera de pantalla; «Restaurar valores de fábrica» vuelve a 1695 × 988; salir
desde la bandeja también guarda.

## X. Ayuda con IA — decisiones adoptadas (spec `005-explicacion-ia`, ADR-046)

Cerrada el 2026-09-08 al implementar la spec 005 (principio XVI de la constitución, versión 1.8.1).

### X.1 · Backend TLS: `native-tls` (SChannel), no `rustls`

`DECIDIDO`. En reqwest 0.13 el feature `rustls` declara su dependencia sin `default-features = false`
y arrastra **`aws-lc-sys`** (BoringSSL vendorizado, compilación de C) sin forma de desactivarlo. En
un proyecto solo-Windows, `native-tls` usa **SChannel** —la pila TLS del propio sistema operativo,
crate `schannel`, FFI puro— y añade ~6 crates efectivos frente a ~20. Sin criptografía vendorizada
en el binario privilegiado, y ya parcheada por Windows Update.

### X.2 · Almacén de la clave: FFI a mano contra `advapi32`, sin crate nuevo

`DECIDIDO`. El crate `windows` (ampliado) o `keyring` habrían sido árboles de dependencias nuevos.
Se hace con `extern "system"` contra `advapi32` (`CredReadW`/`CredWriteW`/`CredDeleteW`, struct
`CREDENTIALW`), mismo patrón que `platform::energia`. `CRED_PERSIST_LOCAL_MACHINE` porque el proceso
va elevado (ADR-004); las pruebas usan `CRED_PERSIST_SESSION` para no exigir elevación en CI.

### X.3 · Tiempo máximo de espera: 60 s

`DECIDIDO` (clarify de la spec). Holgado sobre los ~20 s del caso normal (SC-002); superado, se
cancela y se ofrece reintentar.

### X.4 · Recorte del detalle técnico: 40 000 caracteres

`DECIDIDO` (spec `006-explicacion-ia-contexto-crudo`, ADR-047). Valor de `MAX_DETALLE_CHARS` en
`platform::ia_openrouter`. La 005 lo dejó `PROPUESTO` en 8 000 como defensa contra un detalle
absurdamente largo. La 006 añade a la consulta el volcado `smartctl -a -j` completo (6–20 KB
típico, hasta ~35 KB con el registro de errores del disco) y el contenido del suceso de Windows;
40 000 caracteres cubren el caso normal completo. Para `openrouter/free` son ~12–14k tokens de
entrada, dentro de lo que aceptan los modelos gratuitos actuales. El texto se ensambla
`resumen → volcado → suceso`, así que al recortar se pierde antes el suceso y nunca el resumen
(FR-013). Si se recorta, la respuesta lo advierte (FR-021).

### X.5 · La explicación devuelta es efímera

`DECIDIDO` (asunción de la spec). No se guarda ni se cachea: volver a pedirla lanza una consulta
nueva. Por simplicidad y por la cuota gratuita; revisable si el gasto molesta.

### X.6 · Sin `{@html}`: analizador de subconjunto de Markdown propio

`DECIDIDO`. La respuesta del LLM es contenido no confiable (principio XVI). En vez de una biblioteca
de terceros + saneador, `src/lib/design/markdown.ts` analiza un subconjunto a un árbol de tokens y
`Markdown.svelte` lo pinta con marcado Svelte. Los enlaces se muestran como texto + URL entre
paréntesis, nunca como `href`.

### X.7 · La anonimización vive en la capa de comando, no en `domain::ia`

`DECIDIDO`. `reporting/` depende de `domain::tipos`; meter `reporting::anonimizar` dentro de
`domain::ia` crearía un ciclo `domain → reporting → domain`. El comando (que ya usa ambas capas
legítimamente) anonimiza y entrega cadenas limpias a `domain::ia`, que se queda como hoja pura.

### X.8 · Sin streaming en la v1

`DECIDIDO`. Un `await` y el indicador de progreso bastan. El streaming SSE sería mejora futura.

### X.9 · `reset_settings` del ámbito `ai` borra también la credencial

`DECIDIDO`. Borrar el modelo y el `preview_acknowledged` sin borrar la clave dejaría la función
medio configurada; el estado de fábrica es «sin credencial».

### X.10 · Pendiente de verificar a mano

`explicar_detalle_tecnico`, `guardar_clave_ia`, `probar_clave_ia`, `listar_modelos_ia` y
`platform::credenciales` (con `LOCAL_MACHINE`) **no** se prueban de punta a punta en `cargo test`:
necesitan red real, una clave real y/o el proceso elevado. Recorrido en
`specs/005-explicacion-ia/quickstart.md` con una clave de OpenRouter: activar/probar/borrar la
clave; explicar una alerta (vista previa la primera vez, luego no); explicar el detalle SMART;
elegir un modelo de pago (aviso) y uno gratuito; fallo de red (el modal degrada, la pantalla
sigue); fragmento de texto libre no anonimizable (diálogo de revisión).

## Y. Informe HTML por disco y resumen con IA — decisiones adoptadas (spec `009-informe-mejorado`, ADR-057)

### Y.1 · Textos legibles de alerta: mapa desde el frontend, no un diccionario en Rust

`DECIDIDO`. `export_report` gana `alertLabels?: Record<string, string>`, construido en
`reports/+page.svelte` a partir de las claves `alert.rule.<regla>.title` que ya existen en
`es.json`/`en.json` (las mismas que usa `AlertCard`). El backend no guarda ninguna copia del
texto —lo recibe para ese render, igual que `includeSerials` o el destino— y una clave ausente cae
a la clave cruda. Evita la «segunda copia sin mantener» que ADR-030 ya había descartado para este
mismo problema en J.30, sin necesitar tampoco un diccionario mínimo en Rust (la tercera opción que
se había dejado abierta al escribir la spec).

### Y.2 · Tope de 50 eventos de Windows por disco en el informe

`DECIDIDO`. Mismo espíritu que el tope de 1.500 puntos de serie (E.1): un informe legible, no un
volcado. Por encima del tope se listan los más recientes y se dice cuántos se omiten
(`eventos_omitidos: u32`); quien necesite el histórico completo tiene el CSV/JSON o la pantalla de
Eventos.

### Y.3 · `schemaVersion` del HTML sube a 2, independiente del de CSV/JSON

`DECIDIDO`. `SCHEMA_VERSION_HTML = "2"` en `reporting/export.rs`, separada de `SCHEMA_VERSION`
("1") que siguen usando CSV y JSON — el contenido del HTML cambió de forma; el de CSV/JSON no.

### Y.4 · El resumen con IA se incrusta como texto escapado, nunca como markdown renderizado

`DECIDIDO`. El principio XVI exige «texto o markdown seguro, jamás HTML», pero este informe no
tiene el analizador de subconjunto de Markdown que sí usa el componente `<Markdown>` de la
pantalla (X.6): habría que llevarlo a Rust o generar HTML inseguro. Se optó por lo más simple que
cumple la letra del principio: el modelo recibe la instrucción explícita de responder en texto
plano (`SYSTEM_INFORME_ES`/`SYSTEM_INFORME_EN`, sin pedir Markdown), y la respuesta se escapa como
HTML y se pinta con `white-space: pre-wrap` — un texto con saltos de línea **es** «texto», y
escaparlo es más barato que sanear Markdown→HTML en Rust solo para este caso.

### Y.5 · Una llamada por disco, nunca combinando discos

`DECIDIDO`. Impuesto directamente por la constitución 1.11.0: el alcance por disco es exactamente
el que autoriza la enmienda (alertas + contenido de sucesos que las originaron + contadores SMART
+ resumen numérico de temperatura y actividad, todo del mismo disco), y combinar dos discos en una
petición mezclaría datos de identidades distintas en un solo texto enviado a un tercero.

### Y.6 · El motivo de un resumen no disponible se resuelve contra el mismo `alertLabels`

`DECIDIDO`. `ResumenIaSeccion::NoDisponible { motivo_key }` guarda la clave i18n del motivo
(`error.ia.timeout`, `error.ia.provider`, …), y el HTML la busca en el mismo mapa que ya recibe
para las alertas en vez de inventar un segundo parámetro: el frontend puede añadir esas claves al
mapa si quiere una frase concreta; sin ellas, una nota genérica basta y la exportación no falla.

### Y.7 · Cancelación: se comprueba antes de cada disco, nunca a mitad de una llamada en vuelo

`DECIDIDO`. Una sola bandera (`AppState.informe_cancelado`, no un mapa por exportación: solo puede
haber una a la vez, la interfaz deshabilita el botón mientras corre) que `export_report` resetea
al empezar la fase de IA y comprueba antes de la llamada de cada disco. Cancelar no aborta una
petición HTTP ya en vuelo del disco actual —se deja terminar y su respuesta se descarta—, así que
el retardo máximo tras cancelar es el tiempo límite de `chat_completions` (60 s).

### Y.8 · Pendiente de verificar a mano

El resumen con IA (`preview_informe_ia` con red real activada en `export_report`) no se prueba de
punta a punta en `cargo test` por el mismo motivo que X.10: necesita red y una clave real. Recorrido
en `specs/009-informe-mejorado/quickstart.md`: informe HTML abierto sin conexión (escenario 1);
resumen con IA con 4 discos y un fallo forzado en uno (escenario 3, verifica la nota de
degradación); IA apagada = cero tráfico de red (escenario 4).
