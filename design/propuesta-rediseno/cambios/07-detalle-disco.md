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
3. **Rejilla `1.6fr 1fr`** — `TimeSeriesChart` de temperatura a la izquierda, contadores a la derecha.
4. La gráfica ocupa el alto restante (`flex: 1`), mínimo 240 px.

## 3. Cambios por componente

- **`TimeSeriesChart`** — el cambio más importante del rediseño. Ver `componentes/Sparkline.md` §2
  para el trazado por tramos. Resumen:
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
| **Cargando** | Cabecera real (viene del inventario) + métricas y gráfica en esqueleto. |
| **Vacío** | Serie sin muestras en el intervalo: el marco de la gráfica se mantiene con el eje Y y en el centro «Sin muestras en las últimas 24 h» sobre `bg-glass-3`. **No** se dibuja una línea a cero. |
| **No compatible** | Disco sin SMART: las métricas de firmware muestran «No disponible» y la gráfica se sustituye por `EmptyState kind="unsupported"` con el detalle técnico plegado. La capacidad y los eventos asociados **sí** se muestran. |
| **Error de fuente** | `smartctl` falla: cabecera y capacidad siguen (vienen de otra fuente), y el bloque de firmware pasa a `EmptyState kind="error"`. La pantalla no se cae. |
| **Dato obsoleto** | La frescura de la cabecera pasa a `text-warn`, y la curva termina donde terminan los datos, con la banda gris cubriendo el resto hasta «ahora». |

## 5. Claro y oscuro

Mockup en los dos temas. La serie de temperatura usa `--sdm-warn` cuando el disco está en advertencia
térmica y `--sdm-accent` cuando está correcto: el color de la curva sigue el estado, no es decorativo.

## 6. Ventana mínima (1024 × 560)

- Cabecera: por debajo de **1100 px** el `SegmentedControl` baja a una segunda línea, alineado a la izquierda.
- Métricas: a 950 px pasan a **2 × 2**.
- Rejilla inferior: por debajo de **1000 px** pasa a una columna — primero la gráfica (mínimo 240 px de
  alto), después los contadores. La región hace scroll.
