# 07 · Detalle de disco (`/disks/:id`)

Mockup: `mockups/smartdisk-v3.html` → pestaña **Detalle**, claro y oscuro.

## 1. Diagnóstico

Referencia: `capturas/detalle-disco__oscuro__1280x800__completa.png`. Es la captura más reveladora.

- **La gráfica sale vacía.** El `TimeSeriesChart` dibuja el marco, las líneas de rejilla y dos puntos
  azules en los extremos, pero no hay trazo. Con un hueco en la serie y dos tramos, el resultado es un
  rectángulo hueco de 230 px de alto: el elemento más grande de la pantalla no comunica nada.
- **No hay eje Y.** Aunque hubiera trazo, no se sabe a qué altura está 41 °C ni dónde queda el límite.
- El título de la ventana dice «Panel general» en la pantalla de detalle: la `Toolbar` no recibe el
  título de la ruta.
- La píldora de estado global dice «Sin discos monitorizados» mientras la pantalla muestra un disco
  monitorizado. Estado global mal calculado.
- Los cuatro `MetricCard` son cuatro cifras sin contexto: ni procedencia, ni evolución, ni umbral.
- «Contadores» son dos filas sin delta, al final de la pantalla y sin peso.

> Los tres últimos son **defectos de implementación**, no de diseño. Los anoto porque el rediseño no los
> arregla por sí solo: el título de pantalla, el estado global y la serie con dos tramos hay que
> corregirlos igualmente.

## 2. Cambios de distribución

Cuatro bloques, `gap` 18 px:

1. **Cabecera de identidad** — `Card` de una fila: cuadrado de icono de 56 px (radio 16) con el color
   del estado, alias a 26 px `.sdm-display`, píldora de estado con icono, línea de identidad
   (modelo · bus · serie enmascarada · firmware · frescura), `SegmentedControl` de intervalo y botón
   «Probar disco». El intervalo **sale de la `Toolbar`** y vive aquí, junto a lo que modifica.
2. **Cuatro `MetricCard`** en fila, cada una con icono, cifra a 30 px `.sdm-display`, **sparkline de
   22 px** y línea de procedencia.
3. **Rejilla `1.6fr 1fr`** — a la izquierda, el `SegmentedControl` de intervalo (único, gobierna
   las dos gráficas; segmentos **`1 h` / `24 h` / `7 d` / `30 d` / `Personalizado`**) y **dos
   paneles apilados** (`gap` 20 px), cada uno con fondo `bg-glass-3`, `rounded-inner`, `p-4` y un
   encabezado `<h3>` con la unidad —«Temperatura · °C» arriba, «Actividad · %» debajo—, con su
   `TimeSeriesChart` dentro. A la derecha, los contadores. En modo personalizado, el
   `DateRangePicker` comparte fila con el `SegmentedControl` y sus etiquetas «Desde»/«Hasta» van
   **en línea** con el campo (no encima), para que los campos queden alineados con las píldoras.
4. Cada panel usa el alto por defecto de su gráfica (≈220 px de trazo + pie) más el encabezado; la
   columna crece con los dos y la región hace scroll cuando no cabe.

Los **paneles separados** son lo que deja claro que son dos magnitudes distintas: sin ellos, mismo
color, mismo ancho y ejes parecidos hacían que parecieran una sola gráfica.

La gráfica de **actividad** (`activity_percent`, ADR-055/056): **eje con suelo en 0 y techo
ajustado a los datos** (con eje fijo 0–100 % la actividad de un disco parado —~0 %— no se vería),
**sin líneas de umbral**, **color de acento siempre** (no sigue el estado del disco). Los huecos
«sin datos» se pintan como banda; con la serie recién arrancada, «Recopilando datos…» en vez de
«Sin muestras».

## 3. Cambios por componente

- **`TimeSeriesChart`** — el cambio más importante del rediseño. Ver `componentes/Sparkline.md` §2
  para el trazado por tramos. **Se apilan dos** en el detalle (temperatura y actividad, ADR-055),
  cada uno dentro de su panel `bg-glass-3` con encabezado propio. La prop `titulo` es **solo** el
  prefijo de la etiqueta accesible (no un título visible: lo pone el `<h3>` del panel), para que su
  `role="img"` se distinga con un lector de pantalla. Resumen:
  - **eje Y** de 26 px de ancho con cuatro marcas (`text-2xs`, `tabular-nums`), fuera del área de trazo;
  - **relleno degradado** bajo la curva (`stop-opacity` .32 → 0) con el color de la serie;
  - **grosor 2,6 px** con `vector-effect="non-scaling-stroke"`, para que `preserveAspectRatio="none"`
    no deforme el trazo (esta es la causa probable de que hoy no se vea);
  - **un `polyline` por tramo continuo**: cada `null` corta la línea, y el hueco se pinta como banda
    `--sdm-unknown` al 12 % con su leyenda «sin datos HH:MM – HH:MM». Nunca se interpola;
  - **umbral del fabricante** como discontinua `--sdm-warn` con leyenda propia arriba a la derecha;
  - pie con inicio, hueco y fin en hora local.
- **`MetricCard`** — icono + sparkline + procedencia. Ver `componentes/MetricCard.md`.
- **`DataRow`** — gana columna de delta de 60 px alineada a la derecha, con color **solo** cuando
  significa algo: `+3` en el registro de errores va en `text-warn`; `+0` y `+1` en `text-fg-faint`.
- **`Toolbar`** — recibe título y subtítulo de la ruta. Ver `componentes/Toolbar.md`.
- Bloque de vida estimada: composición de pantalla (`bg-glass-3` + barra + nota de proyección),
  no componente nuevo. Declara siempre que es una proyección y sobre qué ventana se calcula.

## 4. Estados

| Estado | Cómo queda |
|---|---|
| **Cargando** | Cabecera real (viene del inventario) + métricas y gráficas en esqueleto. |
| **Vacío** | Serie sin muestras en el intervalo: el marco de la gráfica se mantiene con el eje Y y en el centro «Sin muestras en el intervalo» sobre `bg-glass-3`. **No** se dibuja una línea a cero. Cada gráfica lo resuelve por su cuenta: la de actividad puede estar vacía (equipo recién arrancado) mientras la de temperatura tiene datos. |
| **No compatible** | Disco sin SMART: las métricas de firmware muestran «No disponible» y la gráfica de temperatura se sustituye por `EmptyState kind="unsupported"` con el detalle técnico plegado. La de actividad **sí** se dibuja (no depende de SMART). La capacidad y los eventos asociados **sí** se muestran. |
| **Error de fuente** | Cada serie falla por su cuenta (`smartctl` para la temperatura, contadores de rendimiento para la actividad): la gráfica afectada pasa a `EmptyState kind="error"` y la otra sigue. Cabecera y capacidad siguen (vienen de otra fuente). La pantalla no se cae. |
| **Dato obsoleto** | La frescura de la cabecera pasa a `text-warn`, y la curva termina donde terminan los datos, con la banda gris cubriendo el resto hasta «ahora». |

## 5. Claro y oscuro

Mockup en los dos temas. La serie de temperatura usa `--sdm-warn` cuando el disco está en advertencia
térmica y `--sdm-accent` cuando está correcto: el color de la curva sigue el estado, no es decorativo.
La serie de actividad es **siempre `--sdm-accent`**: no tiene estado.

## 6. Ventana mínima (1024 × 560)

- Cabecera: por debajo de **1100 px** el `SegmentedControl` (ya con cinco segmentos) y, si está,
  el `DateRangePicker` bajan a una segunda línea, alineados a la izquierda.
- Métricas: a 950 px pasan a **2 × 2**.
- Rejilla inferior: por debajo de **1024 px** pasa a una columna — primero las **dos gráficas
  apiladas** (cada una ≈220 px de trazo, mínimo ≈200 px), después los contadores. La región hace
  scroll. En la ventana mínima (1024 × 560) las dos gráficas más los contadores no caben a la vez:
  el scroll de la región es el comportamiento esperado, no un recorte.
