# 04 · Pruebas y diagnóstico (`/tests`)

Mockup: `mockups/smartdisk-v3.html` → pestaña **Pruebas**, claro y oscuro. Incluye el estado
«prueba en curso», que es el que más cambia.

## 1. Diagnóstico

Referencias: `capturas/pruebas__claro__1280x800__completa.png` y `estado-prueba-en-curso__*`.

- La prueba en curso es lo único que se mueve en la pantalla y está **debajo** de las tres tarjetas de
  prueba, que en ese momento son secundarias.
- El progreso es una barra de 10 px con un porcentaje en texto de cuerpo. Es el dato que el usuario mira
  cada pocos segundos y es el elemento más pequeño del bloque.
- Las tres tarjetas de prueba son indistinguibles entre sí de un vistazo: mismo título, mismo bloque de
  texto, mismo botón.
- El aviso de impacto (temperatura, rendimiento) va en texto corrido sin jerarquía.

## 2. Cambios de distribución

Orden nuevo, con `gap` 18 px:

1. **Prueba en curso** (si hay alguna), arriba y a ancho completo. Dentro:
   cabecera con píldora «Prueba en curso» + titular `.sdm-display` a 22 px + cifra de progreso a
   **58 px** (`--sdm-text-display`) alineada a la derecha + botón Cancelar; barra de **12 px**;
   rejilla de cinco métricas de la prueba; aviso de parada automática.
2. **Tres tarjetas de prueba**, rejilla de 3 columnas.
3. **Historial**, tabla con columna de icono de 28 px al principio.

Si **no** hay prueba en curso, el bloque 1 desaparece y las tarjetas suben. No se deja hueco ni un
bloque vacío de «ninguna prueba en curso».

## 3. Cambios por componente

- **`ProgressBar`** — nueva prop `emphasis` («inline» por defecto, «display» para la prueba en curso):
  alto 12 px, relleno con degradado `accent → accent-hi` y filo interior. Ver `componentes/ProgressBar.md`.
- **`Card`** de prueba — gana cuadrado de icono de 38 px (radio 12) a la izquierda del título.
  Mapa: benchmark → `#i-flask`, chkdsk → `#i-shield`, autotest SMART → `#i-bolt`.
- **Métricas de la prueba** — pasan de `text-metric` plano a cuadro `bg-glass-3` con etiqueta + icono
  y cifra `.sdm-display` a 26 px, con la unidad debajo en `text-2xs`. Deja de leerse «3 148» sin unidad.
- **Aviso de impacto** — bloque `bg-warn-soft` radio 14 con icono de 16 px y texto `text-xs`. Mismo
  contenido, jerarquía distinta.
- **`ConfirmDialog`** — sin cambios de estructura. Sigue declarando acción, destino, impacto y comando
  literal (`estado-dialogo-prueba`). Solo hereda tokens.
- **Historial** — columna de icono de estado al principio; el resto de columnas igual.

## 4. Estados

| Estado | Cómo queda |
|---|---|
| **Cargando** | Tarjetas de prueba en esqueleto; el bloque de prueba en curso no se muestra hasta saber si hay alguna. |
| **Vacío** | Historial sin filas: `EmptyState kind="empty"`, «Todavía no has ejecutado ninguna prueba». Las tarjetas siguen visibles y usables. |
| **No compatible** | La tarjeta de autotest SMART queda con botón deshabilitado y `disabledReason` visible en `title`: «Este dispositivo no declara compatibilidad con el autotest corto». Píldora en gris, **nunca en rojo**. |
| **Error de fuente** | Si `smartctl` no responde, la tarjeta de autotest muestra `EmptyState kind="error"` en su cuerpo; las otras dos siguen funcionando. |
| **Dato obsoleto** | No aplica: los datos de una prueba en curso son en vivo. Si se pierde el canal de progreso, la barra pasa a indeterminada y aparece «sin datos del proceso desde hace N s» en `text-warn`. |

## 5. Claro y oscuro

Mockup en los dos temas.

## 6. Ventana mínima (1024 × 560)

- Cabecera de la prueba en curso: por debajo de **1100 px** la cifra de 58 px baja a 40 px y el botón
  Cancelar pasa a la línea de la barra.
- Rejilla de cinco métricas: a 950 px de contenido pasa a **3 + 2**.
- Tarjetas de prueba: por debajo de **900 px**, 2 columnas y la tercera debajo.
- El historial siempre queda por debajo del pliegue: la región hace scroll.
