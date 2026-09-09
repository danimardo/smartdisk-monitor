# Sistema de diseño — reglas de interfaz (VINCULANTES)

Este documento es **vinculante** para cualquier agente (humano o IA) que escriba interfaz en este
repositorio. Describe cómo construir pantallas con el sistema de diseño aprobado: **v3, «escena de
datos»** sobre el material translúcido de v2 (paleta propia «Ciruela», tipografía de «display» para
cifras grandes, riel de navegación; ADR-034). Si algo no está aquí, no lo inventes: pregunta o propón
una extensión del sistema.

> **v2 → v3 (ADR-034/ADR-035).** No es una reescritura: el material de tres capas, los radios
> concéntricos, el movimiento, el catálogo cerrado y todas las reglas de producto siguen intactos.
> Cambian la paleta (acento morado de tinta, crítico bermellón), `--sdm-on-accent` (tinta en oscuro,
> ya no blanco), se añaden `.sdm-display` y tres componentes (`Icon`, `Sparkline`, `HeroPanel`), y la
> `Sidebar` pasa a un riel de 74 px. La herencia del acento de Windows deja de ser el comportamiento
> de fábrica y pasa a un interruptor apagado por defecto.

Es el par visual de `docs/ui-contract.md`: aquel dice **qué** puede pedirle la interfaz al backend,
este dice **cómo** se pinta lo que recibe. Referencias funcionales: `docs/product-specification.md`,
`docs/user-stories.md`, `docs/data-model.md`.

---

## 0. Dónde vive cada cosa

Esta tabla es el punto de entrada. Un agente que empieza una pantalla no debería tener que buscar
ninguna de estas rutas.

| Qué | Dónde |
|---|---|
| **Reglas vinculantes de interfaz** | `docs/ui-design.md` (este fichero) |
| **Tokens: fuente única de verdad visual** | `src/design-system/tokens.css` |
| Los mismos tokens, legibles por herramientas | `src/design-system/tokens.json` |
| Mapeo de tokens a utilidades Tailwind | `tailwind.config.cjs` (raíz del proyecto) |
| **Catálogo de componentes** | `src/lib/components/` — se importa del barrel `$lib/components` |
| Tipos, formato, salud, iconos, tema y acento | `src/lib/design/` (incluye `icons.ts`) |
| Diccionarios de idioma | `src/lib/i18n/es.json` y `src/lib/i18n/en.json` |
| Tipografía empotrada | `src/design-system/fonts/` |
| Juego de iconos de línea (sprite, 17 símbolos) | `src/lib/components/IconSprite.svelte`, montado una vez en `src/routes/+layout.svelte` (fuera de `AppShell`, para que resuelva también en `/onboarding`); se usa vía `<Icon name="…" />` |
| **Boceto aprobado v3** (4 pantallas, ambos temas) | `design/propuesta-redisenov2/mockups/smartdisk-v3.html` |
| Hoja de contacto de los iconos | `design/propuesta-redisenov2/mockups/icons-hoja-de-contacto.html` |
| Fichas de cambio del rediseño v3 | `design/propuesta-redisenov2/cambios/` · spec: `specs/002-rediseno-v3/` |
| Boceto v2 (referencia histórica) | `design/SmartDisk Monitor v2.dc.html` |
| Comandos y eventos que la UI puede llamar | `docs/ui-contract.md` |
| Verificadores que fallan la integración | `pnpm verify:tokens`, `pnpm verify:i18n` |

**Una sola copia.** El sistema de diseño vive en `src/` y en ningún otro sitio. `design/` contiene
únicamente los bocetos, que son referencia visual y no código reutilizable. Una segunda copia de
`tokens.css` o del catálogo diverge en silencio, y el consolidado acaba publicando valores caducos:
lo comprueba `pnpm verify:tokens` (ADR-029).

Las notas de arranque de la aplicación y las advertencias para quien programa están en el
**apéndice** al final de este documento.

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
src/design-system/tokens.css     ← fuente única de verdad (importar una sola vez en el arranque)
src/design-system/tokens.json    ← misma información, legible por herramientas
tailwind.config.cjs              ← mapeo de tokens a utilidades
```

- **Prohibido** escribir un color, radio, sombra o tamaño de fuente literal en un componente.
  Usa la utilidad Tailwind (`bg-surface`, `text-fg-dim`, `rounded-xl`, `shadow-card`) o `var(--sdm-*)`.
- El tema se conmuta con `document.documentElement.dataset.theme = "light" | "dark"`; lo gestiona
  `$lib/design/theme.svelte.ts`. **Todo componente debe verse correcto en ambos temas sin condicionales.**
- Preferencia de tema y de idioma se persisten en la tabla `settings`, no en `localStorage`.
- **El acento es propio: morado «Ciruela»** (`#7a3f9d` claro / `#c79aec` oscuro), definido en
  `tokens.css`. Heredar el acento de Windows es un **interruptor de Ajustes → Apariencia, apagado de
  fábrica** (`settings.appearance.useSystemAccent`, ADR-035): al encenderlo, `applySystemAccent()`
  (`$lib/design/accent.ts`) sobrescribe los tres tokens de acento, siempre corregidos a AA por
  `accessibleAccent()`/`accentOnSurface()`; al apagarlo, `clearSystemAccent()` restaura el morado.
  Nunca codifiques un color de acento a mano. El acento **no** comunica salud.
- **`--sdm-on-accent` no es blanco en tema oscuro.** El acento oscuro es claro y el texto blanco
  encima daba 2,27:1. Todo texto o icono sobre el acento —o sobre un color de estado— usa
  `text-fg-onAccent` (`var(--sdm-on-accent)`), **nunca `text-white`**. Excepciones (son brillos, no
  tinta): el filo interior de `ProgressBar` en modo `display` y el punto del `Switch` activo.
- **Hay dos tokens de acento y no son intercambiables:**

  | Token | Uso | Contra qué se mide su contraste |
  |---|---|---|
  | `--sdm-accent` | **fondo**: botón primario, relleno de selección | contra `--sdm-on-accent` |
  | `--sdm-accent-fg` | **texto e iconos** sobre el material: enlaces, etiqueta seleccionada, serie principal de la gráfica | contra la superficie del tema |

  Un color no puede cumplir las dos cosas: el azul `#0078d4` que Windows trae de fábrica da 4,31:1
  como texto sobre el material claro, **por debajo de AA**. Barriendo el espacio sRGB completo, el
  65 % de los acentos posibles son ilegibles como texto en tema claro y el 50 % en oscuro
  (`tools/accent-check.py`, `docs/open-questions.md` §O).

  Por eso: **texto de acento ⇒ `text-accent-fg`. Fondo de acento ⇒ `bg-accent`.** Nunca al revés,
  y nunca `--sdm-accent` para pintar texto.

## 2.bis Material translúcido (lo propio de v2)

- Tres capas y nada más: `.sdm-material-chrome` (barra lateral y barra de herramientas),
  `.sdm-material` (tarjetas) y `.sdm-material-overlay` (diálogos y menús). **No escribas
  `backdrop-filter` a mano** ni inventes nuevos niveles de desenfoque.
- **Excepción, solo para el tooltip de ayuda con mucho texto** (`Tooltip`, tooltip local de
  `DiskCard`): mantienen `.sdm-material-overlay` pero pintan el fondo con `--sdm-glass-strong`
  (casi opaco). El texto largo sobre el fondo translúcido normal molesta la lectura. `ChartTip` y
  el resto de overlays **no** cambian: son de una línea o llevan velo detrás.
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
- **Tipografía de «display» (v3).** Para **cifras y titulares**, nunca para texto corrido, se usa la
  clase `.sdm-display` (`--sdm-font-display` + peso 600 + `tabular-nums` + `--sdm-tracking-display`).
  Sus usos: la cifra del héroe (`--sdm-text-hero`, 76 px), el progreso de una prueba
  (`--sdm-text-display`, 58 px), las cifras de `MetricCard`/`DiskCard`, el alias de la cabecera del
  detalle y el título de la barra de herramientas. `--sdm-font-display` hoy resuelve a la familia
  sans ya empotrada: no se empaqueta una segunda familia (ADR-034).
- **El riel de navegación (v3).** La `Sidebar` es un riel de `--sdm-rail-width` (74 px) solo con
  iconos; cada botón lleva `title` **y** `aria-label`. No lleva texto de sección ni lista de discos.
- **Crítico bermellón.** `--sdm-crit` se desplazó al bermellón (`#b03434` / `#ef8080`) para no
  confundirse con el acento morado. Sigue siendo el único rojo, y `unknown` sigue sin ser nunca rojo.
- La escala tipográfica **no se toca sin volver a medir**. Parece pequeña sobre el papel y no lo es:
  Instrument Sans tiene una altura de x de 0,5175 em frente a los 0,50 de Segoe UI, así que el cuerpo
  denso de 12,5 px equivale ópticamente a Segoe UI 12,9 px, por encima de los 12 px (9 pt) que
  Windows usa para el texto de interfaz. Medido, no estimado (`docs/open-questions.md` K.4).
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
| `Card` | contenedor de toda información | radio xl + `shadow-card`; no anides sombras; ranura `leading` opcional (cuadrado de icono a la izquierda del título, v3); prop `border` (`hairline` por defecto, `crit` para una zona destructiva — solo el filo, el fondo no se tiñe) |
| `Button` | acciones | **una sola** `variant="primary"` por pantalla; `variant="feature"` es la acción **estrella** de una pantalla (degradado diagonal del acento + halo de `--sdm-accent-soft`) y convive con una `primary` porque son roles distintos — **una sola `feature` por pantalla** (hoy: «Explícamelo en lenguaje claro», spec 006); `disabledReason` siempre que esté deshabilitado; `hint` (ayuda breve como `title` nativo cuando está activo) para acciones cuyo efecto no es obvio por el rótulo; `primary` escribe `text-fg-onAccent`, nunca `text-white` |
| `Icon` (v3) | símbolo de línea que hereda `currentColor` | uno de los 17 del sprite (`sparkles` marca la ayuda con IA); `label` **obligatorio** si es el único portador de significado, si no `aria-hidden`; mapas semánticos en `$lib/design/icons.ts` |
| `Sparkline` (v3) | trazo de serie sin ejes ni etiqueta | un **`path` curvo** (spline monótona, `rutaSuave`) por tramo continuo, **nunca interpola** un hueco; `vector-effect="non-scaling-stroke"`. Por defecto es contexto; con `interactivo` gana el cursor de lectura (ratón + teclado) y el globo `ChartTip`, igual que `TimeSeriesChart` — lo usa `MetricCard`, no el fondo decorativo de `HeroPanel`/`DiskCard` |
| `HeroPanel` (v3) | dato dominante del panel con su serie de fondo | componente de pantalla (como `DiskCard`); la elección del disco protagonista vive en `selectHeroDisk()`, no en el componente; velo de legibilidad entre la curva y el texto |
| `OnboardingArt` (v3) | ilustración plana decorativa del asistente inicial | cinco escenas (`welcome` / `disks` / `alerts` / `ai` / `done`); solo `currentColor` y `var(--sdm-*)`, correcta en ambos temas sin condicionales; `aria-hidden` siempre (ADR-039); **solo se usa en `/onboarding`** |
| `StatusPill` / `StatusDot` | estado de salud | requieren `label`; el color nunca es el único portador de significado; `StatusPill` admite ranura de icono (`icon="auto"` ⇒ `healthIcon[state]`) |
| `MetricCard` | cifra destacada + procedencia | icono obligatorio + `sparkline` opcional; cifra con `.sdm-display` (peso 600, **no** 800); `value={null}` ⇒ "No disponible" **compuesto como texto en `text-lg`, no como cifra**. Bloque interno (`bg-glass-3` + `rounded-inner`), nunca material sobre material |
| `DataRow` | contador SMART etiqueta/valor/delta | color en el delta solo si significa algo |
| `CapacityBar` | ocupación de volumen | el color lo decide `capacityBarTone()` —imita al Explorador de Windows: rojo cuando queda poco espacio (≥ 91 % ocupado), ámbar ≥ 85 %—, no el llamante ni la severidad de la alerta `capacity.*` |
| `ProgressBar` | operación en curso | siempre con leyenda y tiempo restante; prop `emphasis` (`inline` por defecto, `display` para la prueba en curso) |
| `Sidebar` | navegación principal (riel de 74 px, v3) | material de chrome; solo iconos con `title`+`aria-label`; selección con material elevado e icono en acento, **nunca** barra de color lateral; navega con `<a href>`; sin lista de discos ni texto de estado global |
| `Toolbar` | barra de herramientas unificada | `title`/`subtitle` **de la ruta**; píldora de estado global con icono (única fuente); acción primaria; sin botón «?» (Acerca de va al riel) ni ranura de controles contextuales |
| `SegmentedControl` | intervalos 24 h / 7 d / 30 d / personalizado | |
| `DiskCard` | tarjeta de disco del panel | recibe `href`; cabecera de 52 px que hereda el color del estado con `sparkline` de temperatura de fondo (`temperatureSeries` opcional); dato ausente como «—» discreto, no «No disponible» a 23 px |
| `HealthDonut` | reparto de estados del equipo | acompañar de leyenda numérica. **En v3 sale del panel general** (lo sustituye el bloque «Reparto de estados», que con 2–4 discos se lee mejor); se conserva en el catálogo |
| `AlertCard` | grupo de alertas en lista | píldora de severidad con icono (`severityIcon[severity]`: `info→shield`, `warn→alert`, `crit→bolt`); contador `×N` en `.sdm-num`; claves técnicas solo en el detalle |
| `EventRow` | evento de Windows | nivel como **cuadrado de 26 px con icono** (`eventLevelIcon`) en el color del token, `aria-label` con el nombre del nivel — el color nunca viaja solo; altura de fila **fija en 42 px** (la `VirtualList` no recalcula); etiqueta "asociación inferida" a `text-2xs` sobre `bg-unknown-soft` cuando `mappingConfidence !== "exact"` |
| `TimeSeriesChart` | gráficas históricas | trazo curvo por tramo (comparte `rutaSuave`/`tramos` con `Sparkline`); huecos como huecos; umbral del fabricante discontinuo; cursor de lectura (ratón + teclado) con el valor del punto en un globo `ChartTip` + región `aria-live` |
| `ChartTip` | globo de lectura de una gráfica | valor + instante del punto señalado, posicionado en píxeles por el llamante; `pointer-events-none`, `aria-hidden` (lo anuncia la región `aria-live` de la gráfica); voltea en los bordes; lo comparten todas las gráficas |
| `Tooltip` | ayuda sobre un elemento al pasar el ratón / al enfocar (patrón WAI-ARIA) | dos modos: `focusable` (disparador `<button>`, ratón **y** teclado, `Escape`, `aria-describedby`, cumple WCAG 1.4.13) y `focusable={false}` (disparador `<span>`, **solo ratón**, para dentro de un `<a>`). Filo de color opcional por `HealthState`. Fondo casi opaco (`--sdm-glass-strong`): lleva párrafos y el material translúcido normal dificultaba la lectura. Lo usa `MetricCard` (detalle de disco). En la `DiskCard` del panel las métricas llevan un tooltip local ligero (mismo aspecto, sin componente): con 20 discos serían 60 instancias y el panel debe pintarse rápido (SC-006). Distinto de `ChartTip`, que sigue al puntero sobre un lienzo |
| `ConfirmDialog` | confirmación previa | declarar acción, destino, impacto y comando literal |
| `EmptyState` | vacío / no compatible / error de fuente | distingue los tres casos |
| `AppShell` | raíz de la aplicación | se monta una sola vez; contiene el lienzo con degradado y la región de scroll |
| `Toast` | aviso efímero no bloqueante | solo para confirmar acciones del usuario; **nunca** para alertas de salud, que van al centro de alertas |
| `Switch` | preferencia booleana de efecto inmediato | etiqueta a la izquierda, control a la derecha; nunca dentro de un formulario con botón Guardar |
| `Select` | elección entre 4 o más opciones excluyentes | por debajo de 4 opciones usa `RadioGroup` o `SegmentedControl` |
| `RadioGroup` | 2–4 opciones excluyentes con explicación | cada opción admite descripción; obligatorio para tema e idioma |
| `TextField` | entrada de texto o número | `suffix` para la unidad; validar en `onblur`, nunca en cada pulsación |
| `CodeOutput` | salida literal de un proceso auxiliar | monoespaciada, `white-space: pre`, scroll propio; **renderiza texto, jamás HTML**; botón de copiar obligatorio |
| `Markdown` | render de un subconjunto de Markdown (respuesta del LLM, spec 005) | analizador propio en `src/lib/design/markdown.ts` (encabezados, listas, código, cita, negrita, cursiva, enlace); **nunca `{@html}`**; los enlaces se muestran como texto + URL entre paréntesis, sin `href`. Sin biblioteca de terceros |
| `ExplicacionModal` | modal de la ayuda con IA (spec 005) | `role="dialog" aria-modal`, foco atrapado, `Escape`, devuelve el foco al disparador; fases progreso (con «Cancelar»), resultado (`Markdown` + modelo + advertencia de IA), error (frase + detalle + «Reintentar»), y vista previa / revisión de FR-010/FR-026 |

### Autorizados y pendientes de construir

Estos patrones son necesarios para pantallas ya especificadas y **no requieren una decisión nueva**:
la regla de las ≥3 pantallas no les aplica. Siguen todos los requisitos de un componente del
catálogo (solo tokens, ambos temas, `null` admitido, etiqueta accesible, export en el barrel).

| Componente | Lo exige | Por qué no se puede componer |
|---|---|---|
| `DateRangePicker` | US-020, US-050 (intervalo "personalizado") | no hay ningún control de fecha en el catálogo |
| `FilterBar` | US-021 (filtrar eventos por disco, volumen, nivel y proveedor) | requiere selección múltiple, que `Select` no ofrece |
| `VirtualList` | US-021 (un servidor genera miles de eventos) | renderizar 5.000 `EventRow` bloquea la interfaz |

`Tooltip` **ya está construido** (ver la tabla del catálogo). El uso pendiente es migrar
`Button.disabledReason` y `Button.hint` del `title` nativo a `<Tooltip>`: hoy `Tooltip` aporta su
propio disparador `<button>` y no puede envolver otro control interactivo sin anidar botones.

### Cuándo crear un componente nuevo

Solo si (a) el patrón aparece en ≥3 pantallas y (b) no se puede expresar componiendo el catálogo.
Excepción ya autorizada: los componentes de la tabla "Autorizados y pendientes de construir"
no requieren nueva decisión, solo revisión visual antes de darlos por terminados.
Un componente nuevo debe: consumir solo tokens, funcionar en ambos temas, aceptar `null` en todo dato
opcional, tener etiqueta accesible y exportarse en `src/lib/components/index.ts`.

**Componentes añadidos en v3** (ADR-034): `Icon`, `Sparkline` y `HeroPanel`. Cada uno cumple el
criterio (a)+(b) y su justificación completa está en `design/propuesta-redisenov2/cambios/componentes/`
y en `specs/002-rediseno-v3/`. Ninguno del catálogo se elimina.

## 4. Reglas de composición de pantalla

0. **Medidas de ventana.** Hay tres números y no significan lo mismo:

   | | Valor | Para qué |
   |---|---|---|
   | Mínimo técnico | **1024 × 560** | `minWidth`/`minHeight` de `tauri.conf.json`. Nada puede romperse aquí |
   | Objetivo de diseño | **1280 × 720** | El tamaño contra el que se compone y se revisa |
   | Predeterminado | **1695 × 988** | Solo el **primer** arranque (`tauri.conf.json`). Después manda la geometría que el usuario dejó, que se recuerda en `settings` (ADR-040). Windows la acota si no cabe en la pantalla |

   El mínimo técnico no es un capricho: el escalado de Windows **no encoge el texto, encoge el
   espacio disponible en píxeles CSS**. Un portátil de 1920 × 1080 al 150 % deja una ventana máxima
   de 1280 × 672, y un 1366 × 768 al 125 % deja 1092 × 566. Con un mínimo de 720 de alto, en esas dos
   configuraciones la ventana no cabría en la pantalla. Medido, no estimado (`docs/open-questions.md` K.4).

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

1. **Panel general** (v3) — `HeroPanel` con el disco que necesita atención (`selectHeroDisk()`) y su
   serie de temperatura de fondo; rejilla `repeat(auto-fill, minmax(272px, 1fr))` de `DiskCard`; fila
   inferior `1fr 300px` con «Sucesos del sistema» (`EventRow`) y «Reparto de estados» (composición de
   pantalla, sustituye a `HealthDonut` en el panel — `HealthDonut` sigue en el catálogo). La región
   entera hace scroll; **sin `VirtualList`** en la rejilla (véase `open-questions.md` §U). La lista de
   discos ya **no** vive en la `Sidebar` (riel de solo iconos, v3).
2. **Detalle de disco** (v3) — cabecera de identidad (`Card` de una fila: cuadrado de `Icon` con el
   color del estado, alias `.sdm-display`, `StatusPill` con icono, línea de identidad, botón «Probar
   disco»); fila de 4 `MetricCard` con icono y sparkline de 24 h; rejilla `1.6fr 1fr` con
   `TimeSeriesChart` de temperatura y panel de contadores con `DataRow`. El `SegmentedControl` de
   intervalo va **junto a la gráfica**, ya no en la `Toolbar`.
3. **Alertas** — lista de `AlertCard` (columna fija ~470 px) + detalle: severidad, titular, explicación humana,
   rejilla de hechos (los dos primeros — valor y umbral — en `text-metric` con `.sdm-display`), acciones
   (Reconocer / Silenciar / Archivar / **Ignorar**, y **Dejar de ignorar** en el detalle de una
   alerta ya ignorada) y cronología de ocurrencias. El `SegmentedControl` de filtro tiene cinco
   segmentos: Activas / Resueltas / Archivadas / **Ignoradas** / Todas. «Ignorar» (ADR-044) abre
   `ConfirmDialog` con su impacto; para las seis reglas no ignorables (ADR-045) el botón aparece
   deshabilitado con `disabledReason` (`alerts.ignore.notIgnorable`).
4. **Pruebas y diagnóstico** (v3) — **si hay una prueba en curso**, su bloque va arriba y a ancho
   completo: cabecera con píldora «Prueba en curso» + tipo de prueba `.sdm-display` + cifra de progreso
   a `text-display` (58 px, a `text-metric` por debajo de 1100 px) + botón Cancelar; `ProgressBar
   emphasis="display"`; rejilla de métricas en cuadros `bg-glass-3`; aviso de parada automática en
   `bg-warn-soft` con `Icon` (nunca un badge `text-white`). Debajo, las tres tarjetas de prueba (cada
   una con su cuadrado de `Icon`, `testIcon`), y el historial con columna de icono de estado. Sin
   prueba en curso, el bloque no se muestra y las tarjetas suben.
5. **Ajustes** — secciones apiladas, cada una en su `Card`; controles internos sobre `bg-glass-3`
   (no material sobre material). «Borrar todos los datos» separada al final con `border="crit"` y
   ~32 px extra de separación; el fondo no se tiñe.
6. **Asistente inicial** (`/onboarding`, US-002, v3) — **sin `AppShell`**: `+layout.svelte` omite el
   riel y la barra de herramientas en esta ruta. Cabecera propia de 56 px (logo, indicador de paso,
   «Omitir y usar los valores de fábrica» siempre visible), cuerpo `max-w-[1000px]` centrado, pie de
   navegación `sticky bottom-0` con `.sdm-material-chrome`. Cinco pasos, uno por pantalla, cada uno
   con su escena de `OnboardingArt` (ADR-039): `welcome` y `done` centradas sobre el título;
   `disks`, `alerts` y `ai` compactas junto al encabezado, ocultas por debajo de 720 px de cuerpo
   (el paso `ai` —ayuda con IA, spec 005— se añadió con su escena el 2026-09-09). El guardián de
   redirección vive en `+layout.ts` (`open-questions.md` §V).
7. **Informes**: hereda tokens; sin composición nueva.

### Comportamiento con muchos discos

El boceto v2 está dibujado con cuatro discos, pero Windows Server entra en el alcance y un equipo
puede tener veinte o más. Reglas obligatorias, no opcionales:

- La lista de discos de la `Sidebar` tiene su propio `overflow-y: auto`; la navegación principal y el
  estado global **nunca** hacen scroll con ella.
- El panel general usa rejilla `repeat(auto-fill, minmax(272px, 1fr))` (v3; la `DiskCard` v3 encaja
  tres magnitudes y la barra de capacidad en 272 px).
- A partir de **12 discos monitorizados**, `DiskCard` usa su variante compacta: **sin la sparkline de
  temperatura de cabecera** (`conSparklines = devices.length <= 12`). Es también lo que mantiene
  SC-006 sin virtualizar la rejilla (`open-questions.md` §U); medido en `e2e/ui/rendimiento.spec.ts`.
- El «Reparto de estados» y el recuento cuentan solo los discos monitorizados. Los excluidos por el
  usuario no aparecen; se listan aparte, como exige US-011.

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

---

## Apéndice A. Arranque de la aplicación

La base es **SvelteKit con `adapter-static` y SSR desactivado** (ADR-014). `$lib` ya apunta a
`src/lib`: no toques el alias.

`tokens.css` es la **única** importación de CSS global y va en `src/routes/+layout.svelte`, antes
del primer render:

```ts
import "../design-system/tokens.css";
import { invoke } from "@tauri-apps/api/core";
import { theme } from "$lib/design/theme.svelte";
import { applySystemAccent } from "$lib/design/accent";
import { i18n } from "$lib/i18n";

const s = await invoke<AppearanceSettings>("get_appearance_settings");
theme.init(s.theme);
i18n.init(s.language, s.systemLocale); // el locale viene del backend, no de navigator
await applySystemAccent();
```

`i18n.init()` fija además `<html lang>`, e `i18n.formatLocale` es el locale que usan **todas** las
funciones de `format.ts`: los números siguen al idioma de la aplicación, no al de Windows.

La aplicación se monta con `AppShell` + `Sidebar` + `Toolbar`; ninguna pantalla monta su propio
chrome. El estado inicial de cada pantalla llega por `load` en `+page.ts`, no por `onMount`; las
actualizaciones vienen después por eventos. **No hagas sondeo con `setInterval`**: el backend empuja
(ADR-015). Los comandos y eventos disponibles son los de `docs/ui-contract.md`, que es el normativo:
la UI solo llama comandos enumerados.

## Apéndice B. Notas para quien programe

Errores que se cometen aunque las reglas de arriba estén leídas:

- Ningún literal de color, radio, sombra o tamaño en un componente: utilidad Tailwind o `var(--sdm-*)`.
- Nunca escribas `backdrop-filter` a mano: usa `.sdm-material`, `.sdm-material-chrome`, `.sdm-material-overlay`.
- Los tokens de texto y de salud están verificados a 4.5:1 sobre el material de su tema. No los aclares.
- Un dato ausente es "No disponible"; un dispositivo sin SMART es gris, nunca rojo ni alerta activa.
- Toda acción que escriba datos o genere carga pasa por `ConfirmDialog` con impacto y comando literal.
- Contenido procedente de eventos o dispositivos se renderiza como texto, jamás como HTML.
- Reconocer una alerta **no** devuelve el disco a verde: el color lo decide `deviceState()`, que
  cuenta las alertas `active` y `acknowledged`. El silencio nunca toca el color.
- El acento del sistema pasa por `accessibleAccent()` antes de aplicarse; no supongas texto blanco
  sobre el acento, usa `--sdm-on-accent`.
- `TimeSeriesChart` necesita `from`/`to` además de los puntos: el eje es tiempo real, y el intervalo
  pedido debe verse entero aunque falten datos.
- **La navegación se hace con enlaces, no con callbacks.** `DiskCard` recibe `href` y `Sidebar`
  recibe secciones con su `href`: un `onclick` con `goto()` rompe el ctrl+clic, el menú contextual y
  el anuncio como enlace de un lector de pantalla.
- **Un enlace que envuelve un bloque entero** (tarjeta de disco, fila de suceso del panel) lleva la
  clase `sdm-block-link`: no se subraya al pasar el ratón ni muestra el cursor de mano. Es una zona
  pulsable, no texto; debe comportarse como una lista nativa de Windows, no como una página web. El
  subrayado en `:hover` y el `cursor: pointer` se reservan para los enlaces **de texto en línea**
  («Ver todos», «Ver el suceso»).
- **Realce de hover de una zona pulsable de bloque** (tarjeta de disco, fila de suceso, **grupo de
  alerta**): clase `sdm-hover-bloque` de `tokens.css`. Pinta una **pátina del violeta de acento a
  media intensidad** (`--sdm-accent-soft` al 50 %) mediante una capa `::after` — no un aclarado, y
  no un `hover:bg-*`: el `::after` hace falta porque el material de `Card` taparía cualquier fondo
  del propio elemento y `overflow-hidden` recortaría su sombra. Hereda el radio del elemento (quien
  la use fija su `border-radius`). Es la única señal de que la zona es pulsable, igual que las
  listas del Explorador o de Configuración de Windows. Un elemento **seleccionado** no la lleva: ya
  lo marca su borde de acento.
- **Todo control pulsable tiene estado de hover visible.** Los botones cápsula (`Button`) lo traen
  por variante; los controles de formulario que no lo tenían (`Select`, `Switch`) ganan
  `hover:border-fg-faint` (+ `hover:bg-glass` en `Select`); los selectores de segmento
  (`SegmentedControl`, `FilterBar`) realzan el fondo del segmento inactivo con `hover:bg-glass-2`
  además del texto. El riel (`Sidebar`) y las opciones de `RadioGroup` ya realzaban con `bg-glass-3`
  / `bg-glass`.
- **Indicador de navegación / operación global.** `AppShell` pinta una barra fina (2 px) pegada al
  borde superior de la ventana mientras `navigating` (de `$app/state`) sea no nulo **o** mientras la
  prop `busy` esté activa (el refresco manual de datos la usa): `role="progressbar"`, color
  `bg-accent`, con un `animation-delay` de ~150 ms para que una operación instantánea no la haga
  parpadear (bajo `prefers-reduced-motion` el retardo sigue vigente; solo se anula el avance). Es la
  red de seguridad para cuando un `load` o un comando largo tarda —no sustituye a que la interfaz
  responda al instante, que es lo normal tras ADR-042; el comando largo corre en un hilo bloqueante
  del backend, nunca en el hilo principal—.
- **`Select size="sm"`** es la variante compacta (misma altura que un `Button size="sm"`) para
  usarlo **en línea junto a botones** —p. ej. la duración del silencio en el detalle de una alerta—.
  No pinta el rótulo ni el `hint` visibles, pero conserva `aria-label={label}`: el nombre accesible
  no se pierde.
- **`ConfirmDialog` con `dismissible`** muestra una cruz de cerrar en la esquina. Se usa solo en
  diálogos **informativos** (Acerca de), donde cerrar y «cancelar» son lo mismo; una confirmación
  real de escritura/carga no la lleva — se decide con sus botones.

## Apéndice C. Pantallas pendientes de diseño

Todas tienen ya criterios de aceptación en `docs/user-stories.md` (épica H y US-070 a US-074); lo
que falta es la composición visual, no la definición funcional.

- **Informes** (US-050): selector de intervalo, resumen de contenido y destino de exportación.
- **Ajustes**: apariencia, frecuencias, umbrales, retención, comportamiento al cerrar, borrado de datos.
- **Asistente inicial** (US-002): detección, exclusión de discos y alias.
- **Acerca de** (US-061).

**Icono de la bandeja del sistema** — primer paso visual hecho (`platform/bandeja.rs`,
`open-questions.md` J.53); el rediseño fino sigue pendiente. Se genera en memoria, sin fichero
`.ico`: un **tile redondeado del color de estado** (los cuatro de la regla B.5: verde, ámbar, rojo,
gris) con un **glifo que también cambia con el estado** —cilindro de datos lleno (todo en orden),
con «!» (advertencia), con «×» (crítico), hueco (sin datos / sin discos / fallo de recopilador),
dos barras (en pausa)—: a 16 px el color y la forma van juntos (§VII). El texto emergente es
`«SmartDisk Monitor — <resumen>»` (`tray.tooltip`), nunca el resumen a secas: entre muchos iconos
de bandeja tiene que decir de quién es.

Casi todo se compone con el catálogo actual (`Switch`, `Select`, `TextField`, `RadioGroup`,
`SegmentedControl`, `ConfirmDialog`, `EmptyState`, `CodeOutput`). Las excepciones ya están
autorizadas y no requieren decisión nueva: `DateRangePicker`, `FilterBar`, `VirtualList` y `Tooltip`
(§3, "Autorizados y pendientes de construir").
