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
frecuencia por defecto: 90 s para temperatura, 15 min para SMART. Recogido en `alert-rules.md`.

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
| Temperatura, actividad, capacidad, latencia | 30 s | 10 s | 5 min |
| SMART completo | 5 min | 1 min | 60 min |
| Eventos de Windows | 30 s | 15 s | 5 min |
| Detección de altas y bajas | 60 s | 30 s | 10 min |

Por debajo del mínimo el coste de CPU y de despertar el disco deja de compensar; por encima del
máximo la aplicación deja de merecer el nombre de monitor.

### D.2 · Comportamiento en batería · `PROPUESTO`

Con el equipo a batería se multiplica por **4** el intervalo de temperatura/actividad/capacidad y de
detección de altas y bajas. SMART completo y eventos de Windows **no se alteran**: son las fuentes de
las alertas graves, y spec §4 exige no suspenderlas. Al volver a red se restauran de inmediato y se
fuerza un ciclo completo.

### D.3 · Qué hace exactamente "Pausar" · `PROPUESTO`

Pausa la recopilación y la evaluación de reglas; por tanto también las notificaciones. **No** se
persiste entre reinicios: arrancar la aplicación siempre reanuda. Mientras está pausada:

- la `Toolbar` muestra un aviso permanente con el tiempo transcurrido;
- las alertas ya existentes conservan su estado y su color;
- el icono de la bandeja sigue la regla B.5.

*Por qué no se persiste:* una pausa olvidada es un monitor que no monitoriza y no lo dice. El coste
de reanudar sin querer es mucho menor que el de no vigilar durante semanas.

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

### E.2 · Interacción de la gráfica · `DECIDIDO`

Cursor de lectura con ratón (el punto más cercano en tiempo) y con teclado (flechas, `Inicio`,
`Fin`, `Esc`), que muestra hora y valor en el pie. Sin zoom ni selección por arrastre en la v1.0:
el `SegmentedControl` de intervalo cubre la necesidad y evita un patrón nuevo. Implementado.

### E.3 · Retención mínima frente a US-022 · `DECIDIDO`

US-022 promete "al menos 30 días de historial": se cumple con los agregados de 5 minutos, no con las
muestras crudas (7 días). La historia se reformula para decirlo explícitamente y no dar a entender
que habrá 30 días de detalle.

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
| I.5 | `smartctl` tras controladoras RAID y puentes USB | Qué cascada de `-d` (`sat`, `nvme`, `sntjmicron`, `csmi`) merece la pena antes de declarar "no compatible" | Se documenta la limitación por modelo de puente | `ABIERTO` — necesita hardware: **US-010** |
| I.6 | Instancia única y ACL de `ProgramData` | ~~Pendiente~~ **Resuelto**: eran dos problemas. La instancia única exige comunicar procesos, no solo detectarlos (ADR-025). Y `ProgramData` **no** restringe la escritura a administradores: un usuario sin privilegios se apropia de la carpeta pre-creándola (ADR-026). Véase §R | — | `DECIDIDO` |
| I.7 | Rendimiento de la interfaz con 20 discos y 5.000 eventos | Que la lista virtualizada y el panel aguantan sin bloqueo perceptible | Se recorta la densidad del panel o se pagina | `ABIERTO` — necesita la lista virtualizada: **US-021** |

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
| J.8 | Tamaño de ventana | Mínimo técnico 1024 × 560, objetivo de diseño 1280 × 720, predeterminada 1360 × 880. Medido en §L.2 |
| J.9 | Acerca de | Diálogo modal sobre la pantalla actual, no sección de la `Sidebar` |
| J.10 | Eventos en la navegación | Sección propia en la `Sidebar`, con filtro preaplicado al entrar desde el detalle de un disco |
| J.11 | Plurales en i18n | Función `tp()` con `Intl.PluralRules`; claves `<clave>.one` / `<clave>.other` |
| J.12 | Persistencia de tema e idioma | `theme.set()` e `i18n.set()` devuelven la clave a guardar, pero **no** persisten: el llamante debe invocar `set_setting`. Es fácil de olvidar; conviene un envoltorio que lo haga |

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


| # | Cuestión | Por qué no se ha decidido |
|---|---|---|

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
