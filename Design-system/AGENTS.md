# Instrucciones de UI para agentes — SmartDisk Monitor

Este documento es **vinculante** para cualquier agente (humano o IA) que escriba interfaz en este
repositorio. Describe cómo construir pantallas con el sistema de diseño aprobado: **v2, material
translúcido** (evolución de la dirección 1b). Si algo no está aquí, no lo inventes: pregunta o propón
una extensión del sistema.

Referencias funcionales: `docs/product-specification.md`, `docs/user-stories.md`, `docs/data-model.md`.
Boceto aprobado (v2, cuatro pantallas y ambos temas): `SmartDisk Monitor v2.dc.html`.
Guía visual de tokens y componentes: `Sistema de diseno SmartDisk.dc.html` (pendiente de actualizar a v2).
El boceto original de la dirección 1b queda como referencia histórica en `Bocetos SmartDisk Monitor.dc.html`.

---

## 1. Pila y convenciones de código

- **Tauri 2 + Rust** (backend privilegiado) · **Svelte 5 con runes** + **TypeScript** · **Tailwind CSS**.
- Nada de React. Nada de librerías de componentes de terceros: los componentes de `src/lib/components/`
  son el catálogo completo.
- El frontend **solo** llama comandos Tauri enumerados y tipados (`invoke("get_devices")`, …).
  Nunca construyas comandos, rutas o argumentos de `smartctl` desde la UI.
- Todo texto de interfaz pasa por i18n (`$lib/i18n`, claves `es` + `en`). Ningún literal suelto en JSX/markup.
- Runes: `let { … } = $props()`, `$state`, `$derived`, `$effect`. No uses `export let` ni stores para estado local.

## 2. Regla cero: los tokens

```
design-system/tokens.css     ← fuente única de verdad (importar una sola vez en el arranque)
design-system/tokens.json    ← misma información, legible por herramientas
tailwind.config.cjs          ← mapeo de tokens a utilidades
```

- **Prohibido** escribir un color, radio, sombra o tamaño de fuente literal en un componente.
  Usa la utilidad Tailwind (`bg-surface`, `text-fg-dim`, `rounded-xl`, `shadow-card`) o `var(--sdm-*)`.
- El tema se conmuta con `document.documentElement.dataset.theme = "light" | "dark"`; lo gestiona
  `$lib/design/theme.svelte.ts`. **Todo componente debe verse correcto en ambos temas sin condicionales.**
- Preferencia de tema y de idioma se persisten en la tabla `settings`, no en `localStorage`.
- **El acento lo hereda de Windows.** `applySystemAccent()` (`$lib/design/accent.ts`) sobreescribe
  los tokens de acento al arrancar. Nunca codifiques el azul: el respaldo ya vive en `tokens.css`.
  El acento **no** comunica salud.
- **Hay dos tokens de acento y no son intercambiables:**

  | Token | Uso | Contra qué se mide su contraste |
  |---|---|---|
  | `--sdm-accent` | **fondo**: botón primario, relleno de selección | contra `--sdm-on-accent` |
  | `--sdm-accent-fg` | **texto e iconos** sobre el material: enlaces, etiqueta seleccionada, serie principal de la gráfica | contra la superficie del tema |

  Un color no puede cumplir las dos cosas: el azul `#0078d4` que Windows trae de fábrica da 4,31:1
  como texto sobre el material claro, **por debajo de AA**. Barriendo el espacio sRGB completo, el
  65 % de los acentos posibles son ilegibles como texto en tema claro y el 50 % en oscuro
  (`tools/accent-check.py`, `open-questions.md` §O).

  Por eso: **texto de acento ⇒ `text-accent-fg`. Fondo de acento ⇒ `bg-accent`.** Nunca al revés,
  y nunca `--sdm-accent` para pintar texto.

## 2.bis Material translúcido (lo propio de v2)

- Tres capas y nada más: `.sdm-material-chrome` (barra lateral y barra de herramientas),
  `.sdm-material` (tarjetas) y `.sdm-material-overlay` (diálogos y menús). **No escribas
  `backdrop-filter` a mano** ni inventes nuevos niveles de desenfoque.
- **No apiles materiales**: una tarjeta nunca contiene otra tarjeta. Los bloques internos usan
  `bg-glass-3` + `rounded-inner`.
- Toda superficie de material lleva su filo de 1 px (`shadow-edge`, es decir
  `inset 0 1px 0 var(--sdm-highlight)`) y su hairline (`border-hairline`). Sin ellos el cristal se ve plano.
- **Radios concéntricos**: exterior 18 → interior 13 → navegación 9 → controles en cápsula (999).
  El radio interior se calcula como exterior − padding; no mezcles radios al azar.
- **Contraste primero.** La translucidez es sutil por decisión: si un texto queda por debajo de AA sobre
  el material, sube la opacidad de la capa, nunca bajes el contraste del texto.
- El lienzo lleva un degradado muy tenue (`--sdm-bg` → `--sdm-bg-2`): es lo que da vida al desenfoque.
  No lo sustituyas por un color plano ni por un degradado de color saturado.
- Peso tipográfico máximo **600**. La jerarquía la aporta el material y el tamaño, no la grasa.
- La escala tipográfica **no se toca sin volver a medir**. Parece pequeña sobre el papel y no lo es:
  Instrument Sans tiene una altura de x de 0,5175 em frente a los 0,50 de Segoe UI, así que el cuerpo
  denso de 12,5 px equivale ópticamente a Segoe UI 12,9 px, por encima de los 12 px (9 pt) que
  Windows usa para el texto de interfaz. Medido, no estimado (`open-questions.md` K.4).
- Movimiento: `duration-base` (220 ms) con `ease-sdm` (`cubic-bezier(.32,.72,0,1)`) en selección,
  cambio de pantalla y aparición de diálogos; `active:scale-[0.98]` en los botones. Nada decorativo,
  y todo anulado por `prefers-reduced-motion`.
- Sin soporte de `backdrop-filter` el material cae a `--sdm-solid` automáticamente: no añadas ramas propias.

### Paleta semántica (no decorativa)

| Token | Significado | Uso |
|---|---|---|
| `ok` verde | Correcto | estado dentro de umbrales |
| `warn` ámbar | Advertencia | umbral cruzado, degradación no bloqueante |
| `crit` rojo | Crítico | requiere atención inmediata |
| `unknown` gris | Desconocido / no compatible / sin datos | **jamás rojo** |
| `accent` violeta | Acción primaria, selección, serie de datos principal | no comunica salud |

## 3. Catálogo de componentes

Importa siempre desde el barrel: `import { Card, DiskCard } from "$lib/components";`

| Componente | Para qué | Notas de uso obligatorias |
|---|---|---|
| `Card` | contenedor de toda información | radio xl + `shadow-card`; no anides sombras |
| `Button` | acciones | **una sola** `variant="primary"` por pantalla; `disabledReason` siempre que esté deshabilitado |
| `StatusPill` / `StatusDot` | estado de salud | requieren `label`; el color nunca es el único portador de significado |
| `MetricCard` | cifra destacada + procedencia | `value={null}` ⇒ "No disponible", **compuesto como texto en `text-base`, no como cifra**: a 27 px no cabe en ninguna celda realista. Es un bloque interno (`bg-glass-3` + `rounded-inner`), nunca material sobre material |
| `DataRow` | contador SMART etiqueta/valor/delta | color en el delta solo si significa algo |
| `CapacityBar` | ocupación de volumen | el color lo decide `capacityState()`, no el llamante |
| `ProgressBar` | operación en curso | siempre con leyenda y tiempo restante |
| `Sidebar` | navegación principal + lista de discos | material de chrome; la selección se marca con material elevado y punto de acento; navega con `<a href>`, **nunca** con callback |
| `Toolbar` | barra de herramientas unificada | título y subtítulo de pantalla, controles contextuales, estado global y acción primaria |
| `SegmentedControl` | intervalos 24 h / 7 d / 30 d / personalizado | |
| `DiskCard` | tarjeta de disco del panel | recibe `href`; sin él se renderiza como bloque no interactivo |
| `HealthDonut` | reparto de estados del equipo | acompañar de leyenda numérica |
| `AlertCard` | grupo de alertas en lista | contador `×N`; claves técnicas solo en el detalle |
| `EventRow` | evento de Windows | etiqueta "asociación inferida" cuando `mappingConfidence !== "exact"` |
| `TimeSeriesChart` | gráficas históricas | huecos como huecos; umbral del fabricante discontinuo |
| `ConfirmDialog` | confirmación previa | declarar acción, destino, impacto y comando literal |
| `EmptyState` | vacío / no compatible / error de fuente | distingue los tres casos |
| `AppShell` | raíz de la aplicación | se monta una sola vez; contiene el lienzo con degradado y la región de scroll |
| `Toast` | aviso efímero no bloqueante | solo para confirmar acciones del usuario; **nunca** para alertas de salud, que van al centro de alertas |
| `Switch` | preferencia booleana de efecto inmediato | etiqueta a la izquierda, control a la derecha; nunca dentro de un formulario con botón Guardar |
| `Select` | elección entre 4 o más opciones excluyentes | por debajo de 4 opciones usa `RadioGroup` o `SegmentedControl` |
| `RadioGroup` | 2–4 opciones excluyentes con explicación | cada opción admite descripción; obligatorio para tema e idioma |
| `TextField` | entrada de texto o número | `suffix` para la unidad; validar en `onblur`, nunca en cada pulsación |
| `CodeOutput` | salida literal de un proceso auxiliar | monoespaciada, `white-space: pre`, scroll propio; **renderiza texto, jamás HTML**; botón de copiar obligatorio |

### Autorizados y pendientes de construir

Estos cuatro patrones son necesarios para pantallas ya especificadas y **no requieren una decisión
nueva**: la regla de las ≥3 pantallas no les aplica. Siguen todos los requisitos de un componente
del catálogo (solo tokens, ambos temas, `null` admitido, etiqueta accesible, export en el barrel).

| Componente | Lo exige | Por qué no se puede componer |
|---|---|---|
| `DateRangePicker` | US-020, US-050 (intervalo "personalizado") | no hay ningún control de fecha en el catálogo |
| `FilterBar` | US-021 (filtrar eventos por disco, volumen, nivel y proveedor) | requiere selección múltiple, que `Select` no ofrece |
| `VirtualList` | US-021 (un servidor genera miles de eventos) | renderizar 5.000 `EventRow` bloquea la interfaz |
| `Tooltip` | `Button.disabledReason`, procedencia de métricas | hoy la norma exige el dato pero no hay dónde mostrarlo |

### Cuándo crear un componente nuevo

Solo si (a) el patrón aparece en ≥3 pantallas y (b) no se puede expresar componiendo el catálogo.
Excepción ya autorizada: los cuatro componentes de la tabla "Autorizados y pendientes de construir"
no requieren nueva decisión, solo revisión visual antes de darlos por terminados.
Un componente nuevo debe: consumir solo tokens, funcionar en ambos temas, aceptar `null` en todo dato
opcional, tener etiqueta accesible y exportarse en `src/lib/components/index.ts`.

## 4. Reglas de composición de pantalla

0. **Medidas de ventana.** Hay tres números y no significan lo mismo:

   | | Valor | Para qué |
   |---|---|---|
   | Mínimo técnico | **1024 × 560** | `minWidth`/`minHeight` de `tauri.conf.json`. Nada puede romperse aquí |
   | Objetivo de diseño | **1280 × 720** | El tamaño contra el que se compone y se revisa |
   | Predeterminado | **1360 × 880** | Acotado a lo que quepa en la pantalla del usuario |

   El mínimo técnico no es un capricho: el escalado de Windows **no encoge el texto, encoge el
   espacio disponible en píxeles CSS**. Un portátil de 1920 × 1080 al 150 % deja una ventana máxima
   de 1280 × 672, y un 1366 × 768 al 125 % deja 1092 × 566. Con un mínimo de 720 de alto, en esas dos
   configuraciones la ventana no cabría en la pantalla. Medido, no estimado (`open-questions.md` K.4).

   Debe seguir siendo correcta al 125 %, 150 % y 200 % de escalado.

0.bis **Degradación por ancho.** Dos umbrales, y solo dos:

   - Por debajo de **1180 px** la `Sidebar` se reduce a iconos (56 px), conservando el estado de cada
     disco en su punto de color. A 250 px fijos se comería la cuarta parte de una ventana de 1024.
   - La rejilla del panel es `repeat(auto-fill, minmax(460px, 1fr))`. **460, no 420**: por debajo de
     460 px una `DiskCard` no puede mostrar cuatro métricas sin recortar la más ancha ("684 GB" mide
     94 px a 27 px de cuerpo, y la celda se queda en 61 px).
1. Estructura: `Sidebar` fija a la izquierda (250 px) → `Toolbar` fija arriba de la columna derecha →
   región de contenido con `overflow: auto` y `padding: var(--sdm-space-6)`.
   **La ventana nunca recorta contenido en silencio**: si no cabe, la región hace scroll.
   El contenido pasa por debajo del chrome translúcido: no le pongas fondo opaco.
2. Rejilla: `display: grid` / `flex`. **Gap canónico entre tarjetas: `var(--sdm-space-5)` (18 px)**, que es
   el del boceto aprobado; `var(--sdm-space-4)` (16 px) para bloques dentro de una tarjeta y
   `var(--sdm-space-2)` (8 px) para elementos de una misma fila de controles.
   Nunca márgenes sueltos entre hermanos.
3. Jerarquía por pantalla: un título (`text-xl font-black`), una acción primaria, el resto secundario.
4. Densidad: aire generoso. Si una pantalla necesita más densidad, es señal de que sobra información.
5. Máximo dos niveles dentro de una tarjeta (`glass` → `glass-3`); `glass-2` queda para controles y bloques del chrome.
6. **La fila de métricas de una tarjeta es una rejilla, no un flex.**
   `grid-template-columns: repeat(auto-fit, minmax(104px, 1fr))`: con tarjetas anchas las cuatro
   métricas quedan en línea y con tarjetas estrechas se reorganizan solas, en vez de comprimirse
   hasta recortar. Una fila flex con `flex: 1 1 0` recorta en silencio, que es justo lo que prohíbe
   la regla 1.
6. Tamaño mínimo de objetivo interactivo: **30 px de alto (`--sdm-tap-min`)**, que es la altura de
   `--sdm-control-md`. Ningún control baja de ahí: `--sdm-control-sm` (30 px) es el suelo, no una excepción.
   Queda por encima de los 24 px que exige WCAG 2.2 AA (2.5.8) y por debajo de los 44 px táctiles,
   decisión consciente para una aplicación de escritorio con ratón. Texto mínimo 11 px, y solo dentro de píldoras.
7. Cifras: clase `.sdm-num` (`tabular-nums`) para que las columnas no bailen al actualizarse.
8. Todos los controles son cápsulas (`rounded-pill`). No hay botones rectangulares en v2.

## 5. Reglas de producto que la UI debe respetar

Estas no son estéticas: vienen de la especificación y su incumplimiento es un bug.

- **Nunca inventes ceros.** Dato ausente ⇒ `formatBytes/formatTemperature/...` devuelven "No disponible".
- **No compatible ≠ averiado.** Un disco USB, RAID o virtual sin SMART se presenta en gris como
  "Sin datos SMART", nunca en rojo y nunca como alerta activa.
- **Procedencia visible.** Cada métrica puede mostrar fuente (`smartctl`, `filesystem`, contador de rendimiento)
  y antigüedad de la última lectura válida. Si el dato es `stale`, dilo.
- **Inferencia etiquetada.** Una asociación evento→disco inferida se marca como inferida.
- **Alertas agrupadas.** Se muestra un grupo con contador y cronología de ocurrencias, no una fila por muestra.
  Estados: activa · reconocida · resuelta automáticamente · archivada. Silencio: 15 min · 1 h · 8 h · indefinido.
- **Confirmación antes de escribir o cargar.** Benchmark, chkdsk y autotest abren `ConfirmDialog` con impacto
  explícito (rendimiento, temperatura, escrituras del SSD) y comando literal cuando exista.
- **La UI no se bloquea.** Recopilaciones, exportaciones y pruebas van en el backend; la pantalla muestra
  progreso o actividad. Un colector caído degrada su tarjeta, no la aplicación.
- **Errores comprensibles + detalle técnico conservado**: frase humana visible, detalle en `<details>`.
- **Hora local en presentación, UTC en persistencia.** Unidades: bytes y °C en el dato; formato legible en la vista.
- **Contenido de eventos y dispositivos se renderiza como texto**, nunca como HTML.
- **Reconocer una alerta no cambia el color.** El estado de un disco es la peor severidad de sus
  alertas `active` **o** `acknowledged`; solo `resolved` y `archived` dejan de contar. Reconocer la
  saca de la lista de pendientes y le pone un distintivo, nada más: el color no puede mentir sobre el
  estado del hardware. El silencio es ortogonal y solo afecta a la notificación, jamás al color.
  Todo esto vive en `deviceState()` y `alertCountsTowardHealth()`; ninguna pantalla lo recalcula.
- **Un disco sin datos frescos es `unknown`, no `ok`.** No saber que algo está bien no es saber que
  está bien. `deviceState()` lo resuelve con su parámetro `hasFreshData`.
- **El acento heredado se corrige antes de aplicarse.** `accessibleAccent()` elige texto blanco o
  negro sobre el acento del usuario y lo oscurece si aún así no llega a AA, y `accentOnSurface()`
  deriva el tono legible como texto en el tema activo, prefiriendo la paleta que Windows ya expone.
  Nunca escribas `--sdm-on-accent` ni `--sdm-accent-fg` a mano, ni supongas que el texto sobre el
  acento es blanco. Verificado sobre los 262.144 colores del barrido: cero por debajo de AA.
- **El eje X de una gráfica es tiempo, no índice de muestra.** Las series no son equiespaciadas.
  `TimeSeriesChart` recibe `from`/`to` (el intervalo pedido, no el que cubren los datos) y dibuja
  como hueco todo salto mayor que 1,5× la cadencia, incluidos los extremos.
- **Toda gráfica declara su resolución.** Si se están viendo promedios de 5 minutos u horarios en
  vez de muestras crudas, se dice en el pie (`resolutionLabel`): un máximo promediado no es un pico.

## 6. Accesibilidad

- Contraste mínimo AA (4.5:1) para texto en ambos temas. Los tokens de texto y de salud están verificados
  contra el material de su tema **como texto de píldora a 11 px**: no los aclares, no bajes opacidades sobre
  texto y no pongas texto directamente sobre `glass-3`. Si necesitas más translucidez en una capa, sube la
  opacidad del material, nunca rebajes el color del texto.
- Foco visible en todo elemento interactivo (`:focus-visible` global en `tokens.css`; no lo anules).
  La regla se escribe **con la pseudoclase repetida**, `:focus-visible:focus-visible`, y eso no es
  un descuido: con una sola (0,1,0) empata en especificidad con cualquier utilidad de Tailwind
  —`shadow-edge`, `shadow-[...]`— y pierde por orden, porque las utilidades se generan después de
  `tokens.css`. Se midió: antes de arreglarlo, **ningún** botón mostraba anillo de foco, ni siquiera
  la variante `ghost`, y como la regla hace `outline: none`, los controles quedaban sin ningún
  indicador. Si añades otra regla de estado que compita con una utilidad, súbele la especificidad
  igual. Lo vigila `src/lib/components/Button.browser.test.ts`.
- **Un `role="img"` sin nombre accesible es peor que no ponerlo.** `StatusDot` emitía
  `aria-label=""` cuando no recibía etiqueta y un lector de pantalla anunciaba «imagen» y nada más.
  Regla: si hay etiqueta visible al lado, el gráfico es decorativo y va con `aria-hidden="true"`;
  si no la hay, lleva su propio nombre traducido. Lo detectó `axe` en `e2e/ui/a11y.spec.ts`.
- Navegación completa por teclado: pestañas con `role="tablist"`, diálogos con `role="dialog" aria-modal` y foco atrapado.
- Toda gráfica y todo anillo llevan `role="img"` con `aria-label` que resume el dato, y una lectura textual equivalente cerca.
- `prefers-reduced-motion` respetado globalmente; no añadas animaciones decorativas.

## 7. Pantallas y su composición aprobada

1. **Panel general** — `HealthDonut` + leyenda + tarjeta de atención a la izquierda; rejilla 2×2 de `DiskCard`;
   tarjeta "Sucesos recientes" con `EventRow` al pie. La lista de discos vive además en la `Sidebar`.
2. **Detalle de disco** — cabecera con alias y estado; el `SegmentedControl` de intervalo va en la `Toolbar`; fila de 4 `MetricCard`;
   `TimeSeriesChart` de temperatura (2/3) + panel de contadores con `DataRow` (1/3) y acciones al pie.
3. **Alertas** — lista de `AlertCard` (columna fija ~470 px) + detalle: severidad, titular, explicación humana,
   rejilla de hechos, acciones (Reconocer / Silenciar / Archivar) y cronología de ocurrencias.
4. **Pruebas y diagnóstico** — tres tarjetas de prueba; tarjeta de ejecución en curso con `ProgressBar` y
   cinco métricas; aviso ámbar de parada automática; historial de `test_runs`.
5. **Informes**, **Ajustes** y **asistente inicial** (US-002) están pendientes de diseño: compón con este mismo
   catálogo y pide revisión antes de introducir patrones nuevos.

### Comportamiento con muchos discos

El boceto v2 está dibujado con cuatro discos, pero Windows Server entra en el alcance y un equipo
puede tener veinte o más. Reglas obligatorias, no opcionales:

- La lista de discos de la `Sidebar` tiene su propio `overflow-y: auto`; la navegación principal y el
  estado global **nunca** hacen scroll con ella.
- El panel general pasa de rejilla fija 2×2 a `repeat(auto-fill, minmax(460px, 1fr))` (véase §4.0.bis:
  460 es el ancho por debajo del cual las cuatro métricas dejan de caber).
- A partir de **12 discos monitorizados**, `DiskCard` usa su variante compacta (una sola fila de
  métricas, sin gráfica en miniatura) y el panel muestra primero los que no están en `ok`.
- `HealthDonut` cuenta solo los discos monitorizados. Los excluidos por el usuario no aparecen en el
  reparto ni en el recuento; se listan aparte, como exige US-011.

## 8. Definición de terminado para una pantalla

- [ ] Solo tokens; cero literales de color/tamaño; cero `backdrop-filter` escrito a mano.
- [ ] Correcta en tema claro y oscuro, con el acento del sistema y con el azul de respaldo.
- [ ] Correcta en la ventana mínima de la app sin recortes, y legible con el material sobre contenido denso.
- [ ] Textos en `es` y `en`.
- [ ] Estados diseñados: cargando, vacío, no compatible, error de fuente, dato obsoleto.
- [ ] Datos ausentes como "No disponible"; ningún cero inventado.
- [ ] Teclado y foco verificados; `aria-label` en gráficas e iconos.
- [ ] Acciones con carga o escritura confirmadas con impacto explícito.
- [ ] Sin permisos Tauri nuevos ni comandos genéricos.
- [ ] Verificada a 1024 × 560 (mínimo técnico) y a 1280 × 720 (objetivo de diseño), con la barra
      lateral colapsada y sin un solo recorte silencioso.
- [ ] Verificada con un valor ausente en cada métrica: "No disponible" cabe y se distingue de una cifra.
- [ ] Verificada con un acento del sistema claro (por ejemplo el amarillo `#ffb900`) **y en los dos
      temas**: el texto sobre el acento y el texto de acento sobre el material cumplen AA, y el
      cambio de tema recalcula `--sdm-accent-fg`.
- [ ] Verificada con 20 discos y con 5.000 eventos, sin bloqueo perceptible al desplazarse.
- [ ] Ni un solo literal de interfaz fuera de `es.json` / `en.json`, incluidos `aria-label` y títulos.
