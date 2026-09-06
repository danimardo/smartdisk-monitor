# 09 · Chrome y estados comunes

El chrome está en todas las pantallas, así que sus cambios se documentan una vez aquí en vez de repetirse
ocho veces. Mockup: cualquier pestaña de `mockups/smartdisk-v3.html`.

## 1. Diagnóstico

- La `Sidebar` de 250 px usa ~380 px de sus 800 de alto; el resto es vacío. A 1024 px se lleva el 24 %
  del ancho de la ventana, que es justo el recurso escaso.
- El estado global aparece dos veces (píldora en `Toolbar`, texto al pie de la `Sidebar`) y en las
  capturas **no coinciden**: una dice «Todo en orden» y la otra «Sin discos monitorizados».
- La `Toolbar` muestra «Panel general» en la pantalla de detalle: no recibe el título de la ruta.
- La lista de discos de la `Sidebar` repite lo que ya está en la rejilla del panel.

## 2. Cambios

### Sidebar → riel de 74 px

Logotipo de 34 px, seis botones de 44 px con icono de 19 px, punto de aviso de 6 px sobre Alertas, y al
pie el indicador de estado global (cuadrado de 40 px con icono y contador). Sin etiquetas de texto: cada
botón lleva `title` **y** `aria-label`. Sin lista de discos.

Detalle completo, mapa de iconos y la alternativa de riel expansible en `componentes/Sidebar.md`.

### Toolbar

- Recibe `title` y `subtitle` **de la ruta**. Corregir el bug de «Panel general» en todas las pantallas.
- La píldora de estado global pasa a ser la **única** fuente de ese dato, y gana icono
  (`#i-shield` todo en orden / `#i-alert` N necesitan atención / `#i-clock` en pausa).
- Se elimina el botón «?» de la barra: **Acerca de** pasa al riel, como último icono. Menos ruido en la
  zona de acciones. El diálogo (`estado-dialogo-acerca-de`) no cambia y sigue cerrando con Escape.
- Los controles contextuales de pantalla salen de la `Toolbar`: el intervalo del detalle de disco vive
  junto a la gráfica que modifica (ver `cambios/07`).

### El estado global, una sola vez

Se calcula con `worstState()` sobre los discos **monitorizados**, y los discos sin SMART **no** cuentan
como avería. Tres formas, con su icono:

| Situación | Texto | Token |
|---|---|---|
| Ningún disco en warn/crit | Todo en orden | `ok` |
| N en warn o crit | N necesita/n atención | `warn` o `crit`, el peor |
| Recopilación pausada | En pausa | `unknown` |

Aparece en la píldora de la `Toolbar` y, en forma de icono, al pie del riel. El riel **no** lleva texto,
así que no puede contradecir a la píldora.

## 3. Los cinco estados, de forma transversal

Reglas que valen para cualquier pantalla, para no repetirlas en cada fichero:

- **Cargando** — esqueletos con la **misma geometría** que el contenido real (misma altura de fila, mismo
  alto de tarjeta), en `bg-glass-3`, sin animación de brillo. Nunca un spinner centrado que tape la pantalla.
- **Vacío** — `EmptyState kind="empty"`. Dice qué falta y ofrece la acción que lo resuelve.
- **No compatible** — `EmptyState kind="unsupported"` o píldora gris en su sitio. **Nunca rojo, nunca
  alerta activa, nunca contado como avería.** Se explica el motivo técnico en `<details>`.
- **Error de fuente** — `EmptyState kind="error"` **local al bloque afectado**. Un colector caído degrada
  su tarjeta, jamás la aplicación. Frase humana visible + detalle técnico plegado y copiable.
- **Dato obsoleto** — la marca de frescura pasa a `text-warn` y se dice cuándo fue la última lectura
  válida. Las series terminan donde terminan los datos; no se extienden ni se interpolan.

## 4. Diálogos

- `ConfirmDialog` y el diálogo Acerca de: sin cambios de estructura. Heredan tokens.
- Recordatorio de `00-tokens.md`: el botón primario de cualquier diálogo necesita `text-fg-onAccent`
  (tinta en oscuro), no `text-white`.
- `Toast` — sin cambios, y sigue prohibido usarlo para una alerta de salud o como **único** portador de
  un error (se pierde).

## 5. Ventana mínima

- Riel: 74 px fijos, sin colapsar. Es ya el mínimo razonable.
- `Toolbar`: por debajo de **900 px** el subtítulo se oculta y la frescura pasa a `title` del botón
  Actualizar. El orden de sacrificio es: subtítulo → frescura → texto de la píldora (que queda como
  icono con `aria-label`). El botón primario **nunca** se oculta.
