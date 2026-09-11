# Fase 0 — Investigación: Informe HTML por disco

Resuelve las decisiones que la spec dejó para el plan y las incógnitas del contexto técnico.

---

## D1 · Frases legibles de alerta sin romper ADR-030

**Decisión**: el HTML lo sigue generando el **backend** (`reporting::informe`); la **interfaz le
pasa un mapa `clave_de_regla → texto`** en la llamada `export_report`, con las ~15 claves de
`docs/alert-rules.md` §2 traducidas al idioma activo. El backend usa el mapa para pintar cada fila
de alerta; si una clave no está en el mapa, cae a la clave cruda (comportamiento actual).

**Rationale**:
- ADR-030 prohíbe que el backend **posea/mantenga** una copia del texto de alerta; no prohíbe que
  la interfaz —que ya tiene los diccionarios `$lib/i18n`— le pase el texto para un render puntual,
  igual que ya le pasa `includeSerials`, `deviceIds` y el destino.
- Mantener el HTML en un solo sitio (backend) evita reconstruir en la interfaz toda la recogida de
  datos (alertas + SMART + eventos + series + resúmenes IA), que ya vive en Rust.
- El mapa es pequeño y estable (las reglas de `alert-rules.md`), y `pnpm verify:i18n` ya garantiza
  que las claves existan en los dos diccionarios.

**Alternativas descartadas**:
- *La interfaz genera todo el HTML*: tendría que pedir y ensamblar todos los datos (varias llamadas
  nuevas), y los resúmenes IA vienen del backend de todos modos. Mucho más código y más superficie.
- *Diccionario mínimo duplicado en Rust*: es exactamente la «segunda copia sin mantener» que
  ADR-030 quiso evitar; `pnpm verify:i18n` no lo cubriría.

**Impacto de contrato**: `export_report` gana un parámetro opcional `alertLabels: Record<string,
string> | null`. Se documenta en ADR-057 y en `docs/ui-contract.md` §3.7.

---

## D2 · Conjunto de contadores SMART del informe

**Decisión**: los **mismos** que muestra el panel «Contadores» del detalle de disco: todas las
métricas SMART persistidas del dispositivo **salvo** `smart_query_ok` y `vendor_temp_limit_celsius`
(el mismo filtro que ya aplica `get_device_detail`). Por cada uno: valor de la última muestra
dentro del intervalo y **delta** = ese valor menos el de la última muestra **en o antes** del
inicio del intervalo; sin muestra base → «sin referencia».

**Rationale**: coherencia con lo que la persona ve en la aplicación; no inventar una lista nueva
que haya que mantener sincronizada. `repo_metricas` ya da `device_series` y
`latest_device_sample`.

**Alternativas descartadas**: una lista curada «solo lo importante» — subjetiva y divergente del
detalle de disco; un informe es un volcado legible, no una criba.

---

## D3 · Mini-gráficas SVG en el informe

**Decisión**: SVG generado en Rust (`reporting/minigrafica.rs`), una por métrica (temperatura y
actividad), **embebido** en el HTML. Trazo: `completar_serie` (ya en `domain::series`) rellena los
huecos con puntos `null`; el generador emite un `<polyline>` por tramo continuo y **rompe el trazo
en cada `null`** (un hueco es un hueco, nunca se interpola ni se dibuja a cero). Eje Y con dos o
tres marcas numéricas; sin interacción. **Print-safe**: trazo en un gris oscuro fijo, relleno muy
tenue o ninguno, legible en escala de grises; tamaño fijo (p. ej. 480×120) para que la impresión
no lo deforme.

**Rationale**: el informe debe abrirse sin conexión → nada de librerías ni `<img>` remotos. La
lógica de tramos/huecos ya está resuelta conceptualmente en la aplicación (`TimeSeriesChart`); aquí
se necesita solo un subconjunto (sin cursor, sin umbrales). `domain::series::completar_serie` +
`submuestrear` (tope de puntos) se reutilizan; `tramos`/`huecos` no existen en Rust pero partir la
polilínea en los `null` de `completar_serie` es equivalente y trivial.

**Alternativas descartadas**:
- *Reusar el `TimeSeriesChart` de Svelte renderizándolo a SVG*: no hay pipeline para eso en el
  backend y el informe no ejecuta la aplicación.
- *PNG generado en Rust*: exige un crate de rasterizado (dependencia nueva, prohibido salvo ADR);
  SVG en línea es texto, sin dependencias.

**Resolución de la serie**: sigue la correspondencia intervalo→resolución de `open-questions.md`
E.1 (cruda ≤24 h dentro de 7 días; agregados de 5 min; horaria). El pie de cada mini-gráfica
declara la resolución mostrada, como en la aplicación.

---

## D4 · Anonimización del payload del resumen IA

**Decisión**: dos capas, reutilizando la pila existente:
1. **`Anonimizador` (sustitución literal)** — `para_esta_maquina(&series)` (equipo, usuario,
   números de serie de todos los discos) **más una ampliación nueva `.con_etiqueta_volumen(label)`**
   alimentada con las etiquetas de volumen del inventario. Las etiquetas de volumen están en la
   lista del principio XVI desde 1.8.0 pero el `Anonimizador` aún no las cubría; el informe las
   necesita (lista de volúmenes) y es una adición acotada.
2. **`ia::redactar_identificadores` (barrido de patrones)** — SID, rutas `\Device\`, WWN hex, sin
   `regex`, ya existe (spec 006). Es la red de seguridad sobre el mensaje de los sucesos de
   Windows, que es el único texto libre del payload.

Los **valores numéricos** (contadores SMART, mín/media/máx/pico) no identifican a nadie y se
conservan; marca, modelo, interfaz y firmware se conservan como contexto (principio XVI).

**Rationale**: el payload del informe es más estructurado que el de «Explícamelo» (listas de
alertas, contadores, resúmenes numéricos) y su único texto libre es el mensaje de los sucesos, que
ya recibe las dos capas en el flujo existente. La ampliación de etiquetas de volumen también cierra
un hueco latente del flujo de «Explícamelo» cuando un mensaje de suceso menciona una etiqueta.

**Fragmentos dudosos** (`ia::barrer_texto_residual`): se aplican **por disco** sobre el texto ya
anonimizado; la vista previa los señala. Con «enviar sin revisar» (ADR-047) se omite el señalado
manual, no la vista previa ni las dos capas de anonimización.

**Alternativas descartadas**: un anonimizador nuevo específico del informe — duplicaría lógica ya
probada; mejor ampliar el existente.

---

## D5 · Composición de la consulta por disco

**Decisión**: nueva variante `ia::Detalle::Informe(DetalleInforme<'a>)` y rama en
`ia::componer_consulta` (o función hermana `componer_consulta_informe`). `DetalleInforme` agrupa,
**todo del mismo disco**:
- lista de alertas del intervalo (descripción legible, severidad, primera/última, veces);
- por cada alerta nacida de un suceso, el **contenido del suceso** (`extraer_contenido_suceso`, ya
  existe — mensaje + campos de datos, sin el bloque de metadatos de sistema);
- contadores SMART (nombre + valor + delta);
- resumen numérico de temperatura y de actividad del intervalo (mín/media/máx/pico o «sin datos»).

El `system prompt` reutiliza el de la ayuda existente adaptado: «resume en lenguaje llano para una
persona no técnica el estado de **este** disco en el periodo; es orientación, no un diagnóstico».

**Rationale**: `domain::ia` ya es el sitio de la composición y no conoce Tauri ni red; añadir una
variante es coherente. `extraer_contenido_suceso` y `ContextoDisco` se reutilizan tal cual.

**Alcance verificado contra XVI 1.11.0**: exactamente lo que la enmienda autoriza; nada de
inventario completo, configuración, otras pantallas ni otros discos; nunca dos discos en una
petición.

---

## D6 · Flujo de exportación con IA: comandos, progreso y cancelación

**Decisión**:
- **`preview_informe_ia(fromUtc, toUtc, deviceIds, alertLabels)` → `PreviewInformeIaWire`**
  (sin red): por cada disco incluido, el `system`/`user` **ya anonimizado y recortado** que se
  enviaría, más sus `fragmentos` dudosos y `redactedFields`. Análogo a `preview_diagnostic_zip`.
- **`export_report`** gana `includeAiSummary: boolean` y `previewConfirmada: boolean`. Con
  `includeAiSummary=false` → comportamiento actual (síncrono, sin red). Con `true` y sin
  `previewConfirmada` → devuelve un error/estado que dice «falta confirmar la vista previa»
  (la interfaz llama antes a `preview_informe_ia`). Con `true` + `previewConfirmada` → el comando
  es **asíncrono**: por cada disco emite `report:progress` (`{ done, total, deviceLabel }`),
  hace su llamada (`platform::ia_openrouter::chat_completions`), y **comprueba la bandera de
  cancelación** entre discos. Al terminar, ensambla el HTML (con los resúmenes o sus notas) y lo
  escribe. Si se canceló, **no escribe** el fichero y devuelve `export.cancelled`.
- **`cancelar_informe()`**: pone a `true` `AppState.informe_cancelado` (un
  `Arc<AtomicBool>`, uno solo — una exportación a la vez). El comando de exportación la resetea al
  empezar.
- **Evento `report:progress`**: nuevo en `docs/ui-contract.md` §4, misma forma que `test:progress`.

**Rationale**: hay precedente directo — `test_cancel_flags` + `test:progress` para benchmark/chkdsk
(cancelables, con progreso). Se replica el patrón. Mantener `export_report` como puerta única (con
más parámetros) evita un segundo comando de exportación casi idéntico.

**Sin reintento automático** (principio XVI): un fallo por disco → ese disco lleva su nota, se
sigue. `platform::ia_openrouter::chat_completions` ya tiene su propio límite de tiempo; no se
añade backoff.

**Alternativas descartadas**:
- *Un comando `generar_informe_ia` separado del `export_report`*: dos rutas de exportación que
  divergen; peor de mantener.
- *Cancelación por `Drop`/abort de la tarea Tauri*: menos predecible que una bandera comprobada
  entre discos (que además deja el punto de corte limpio, sin fichero a medias).

---

## D7 · Momento de elegir la ruta de guardado

**Decisión**: **igual que hoy** — el diálogo nativo de «guardar como» se abre **antes** de generar
(la interfaz ya lo hace así para CSV/JSON/HTML). Con IA: se elige la ruta → se llama a
`preview_informe_ia` → se muestra la vista previa → al confirmar, `export_report` genera y escribe
en esa ruta con progreso. Si se cancela, no se escribe nada en la ruta elegida.

**Rationale**: coherente con el flujo actual; la persona ya sabe dónde irá el fichero cuando
decide si merece la pena esperar. El fichero solo aparece cuando está completo.

**Alternativas descartadas**: generar a temporal y pedir la ruta al final — cambia el flujo actual
sin ganar nada y complica el manejo de errores de escritura.

---

## D8 · `schemaVersion` del HTML y tope de la lista de eventos

**Decisiones menores**:
- **`schemaVersion`** del HTML sube a **2** (cambio de estructura). El HTML es para personas, no
  se parsea; el número queda por consistencia con CSV/JSON (que **no** cambian y siguen en su
  versión).
- **Tope de eventos por disco en el informe**: **50**, con nota «y N más» y remisión al JSON/CSV
  para el detalle completo. Valor de afinado, en el mismo espíritu que el tope de 1.500 puntos de
  serie (E.1) o la ventana de ráfaga de 60 s.
- **Modelo usado para el resumen**: `settings.ai.model` (el configurado), igual que «Explícamelo».

---

## D9 · Estados y degradación (recordatorio para el diseño)

- **IA no configurada**: la casilla no se ofrece (o deshabilitada con motivo). El HTML se exporta
  como la Historia 1.
- **Casilla apagada**: sin red, exportación inmediata.
- **Fallo de red / cuota / límite / timeout por disco**: ese disco → nota «resumen con IA no
  disponible»; los demás conservan el suyo; el HTML se genera.
- **Cancelación**: sin fichero; `export.cancelled`.
- **Serie de actividad vacía en el intervalo** (equipo recién arrancado tras ADR-056): la
  mini-gráfica muestra su vacío y el resumen numérico dice «sin datos»; ese hecho viaja al modelo.
- **Última lectura SMART antigua**: valor con «última lectura: hace X» (no «No disponible»).

---

## D10 · Cómo se incrusta la respuesta del modelo en el HTML

**Decisión**: la respuesta del modelo se incrusta como **texto plano HTML-escapado** dentro de un
bloque con `white-space: pre-wrap` (conserva saltos de línea y párrafos), precedido de una marca
«Resumen generado por IA con el modelo X — orientación, no un diagnóstico». **No** se renderiza el
markdown (ni negritas ni listas ni enlaces).

**Rationale**: el principio XVI exige «texto o **markdown seguro**, jamás como HTML, jamás se
ejecuta». Renderizar markdown a HTML en el backend exigiría un parser/saneador nuevo (código y
riesgo). Texto escapado con saltos de línea preservados **es** «texto», cumple al pie de la letra,
y es imposible de inyectar. La aplicación ya trata la respuesta del modelo como no confiable en su
propio `Markdown.svelte`; aquí, al ser un fichero que se distribuye, el criterio conservador (sin
render) es el correcto.

**Alternativas descartadas**:
- *Markdown→HTML seguro en Rust*: parser + lista blanca de etiquetas + escape = superficie nueva
  para un beneficio cosmético (negritas en un párrafo).
- *Incrustar el markdown crudo tal cual*: se vería con los `**` y `-` literales; feo y confuso.
