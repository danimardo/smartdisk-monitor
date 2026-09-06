# 02 · Alertas (`/alerts`)

**Cambio solo de tokens y de estructura interna de `AlertCard`.** La distribución no se toca.
No hay mockup propio: la paleta se resuelve sola con los tokens, y `AlertCard` se ve en el
`HeroPanel` del mockup (comparten la píldora de severidad y el contador).

## 1. Diagnóstico

Referencia: `capturas/alertas__oscuro__1280x800__completa.png`.

- La columna de 470 px con una sola `AlertCard` deja la lista casi vacía, pero eso es el fixture, no
  el diseño: con 3-5 grupos la proporción es correcta. **No se cambia.**
- La píldora de severidad es solo texto. En una lista larga cuesta barrer por severidad.
- La rejilla de hechos del detalle usa el mismo tamaño de cifra que el cuerpo, así que «71 °C» y
  «70 °C» (valor y umbral, el par que importa) no destacan sobre el resto.

## 2. Cambios de distribución

Ninguno. Columna izquierda de 470 px, detalle a la derecha, mismos bloques y mismo orden.

## 3. Cambios por componente

- **`AlertCard`** — la píldora de severidad gana ranura de icono: `#i-alert` para *warn*, `#i-bolt`
  para *crit*, `#i-shield` para *info*. El contador `×N` pasa a `.sdm-num`. Nada más.
  Ver `componentes/StatusPill.md` (la ranura de icono se añade ahí, no en `AlertCard`).
- **Rejilla de hechos del detalle** — los dos primeros hechos (valor actual y umbral) usan
  `text-metric` con `.sdm-display`; los otros dos se quedan en `text-base`. Es composición de
  pantalla, no cambio de componente.
- El resto (titular, explicación humana, acciones, cronología) intacto.

## 4. Estados

Los cinco siguen cubiertos y **ninguno cambia de estructura**:

- **Vacío** (`estado-alertas-vacio`): igual, con los tokens nuevos.
- **Diálogo destructivo** (`estado-dialogo-destructivo`): igual. Recordatorio: en oscuro el botón
  primario necesita `text-fg-onAccent`, no `text-white`.
- Cargando, error de fuente y dato obsoleto: sin cambios.

## 5. Claro y oscuro

Solo tokens. No hace falta mockup.

## 6. Ventana mínima

Sin cambios respecto a hoy, salvo que el riel devuelve 176 px: a 1024 px la columna de alertas y el
detalle conviven mejor que ahora. Por debajo de 900 px, el detalle pasa a ocupar la pantalla completa y
la lista se convierte en un paso atrás, **igual que hoy**.
