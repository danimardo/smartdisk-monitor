# Propuesta de rediseño — SmartDisk Monitor v3

> **Segunda entrega.** `RESPUESTAS-A-LA-REVISION.md` responde a los seis puntos de tu revisión: incluye el
> sprite de los 15 iconos, las instrucciones de empaquetado de la fuente, los pasos 1/3/4 del asistente
> dibujados, la tabla de umbrales por perfil, y cierra las dos decisiones que estaban abiertas
> (**riel fijo de 74 px** y **`HealthDonut` fuera de esta propuesta**). Empieza por ahí si ya leíste el resto.

Empieza por **`RESUMEN.md`**: los seis cambios de fondo, qué no se toca y los cambios de texto.

Después, en este orden:

1. `cambios/00-tokens.md` — la tabla completa de tokens, con los ratios de contraste medidos sobre el
   material compuesto. **Nada visual vive fuera de este fichero.**
   Y `cambios/00b-tipografia-y-fuentes.md` — origen, licencia y subconjunto de las dos familias.
2. `cambios/componentes/` — antes/después de los diez componentes afectados. Tres son nuevos
   (`Icon`, `Sparkline`, `HeroPanel`), cada uno con su justificación contra `ui-design.md` §3.
3. `cambios/01`…`09` — un fichero por pantalla, con diagnóstico referido a tus capturas, cambios de
   caja, estados y comportamiento a 1024 × 560.
4. `cambios/08b-perfiles-de-alerta.md` — los doce umbrales de los tres perfiles de alerta.
5. `mockups/smartdisk-v3.html` — mockup navegable y autocontenido, cuatro pantallas (asistente con sus
   cuatro pasos) y dos temas. `mockups/icons-sprite.svg` y `mockups/icons-hoja-de-contacto.html` para
   los iconos.

## Tres avisos antes de tocar código

1. **`--sdm-on-accent` deja de ser blanco en el tema oscuro** (pasa a `#20132a`). Con el acento claro de
   la paleta Ciruela, el texto blanco encima daba 2,27:1. Cualquier `text-white` sobre el acento hay que
   cambiarlo por `text-fg-onAccent`. Ver `cambios/componentes/Button.md`, que lista dónde.
2. **La gráfica del detalle sale vacía hoy** (`capturas/detalle-disco__oscuro__1280x800__completa.png`).
   La causa más probable es `preserveAspectRatio="none"` sin `vector-effect="non-scaling-stroke"`: el
   grosor se deforma hasta desaparecer. Está explicado en `cambios/componentes/Sparkline.md` §2.
3. **Hay tres defectos que el rediseño no arregla solo** y conviene corregir a la vez: el título de la
   `Toolbar` (dice «Panel general» en todas las pantallas), el estado global mal calculado (dice «Sin
   discos monitorizados» con un disco monitorizado) y la serie con dos tramos. Ver `cambios/07` §1 y
   `cambios/componentes/Toolbar.md`.

## Lo que NO cambia

Ningún comando Tauri nuevo, ningún permiso nuevo, ningún cambio de esquema. El rediseño es solo de
presentación. Se mantienen el material de tres capas, los radios concéntricos, el movimiento, el catálogo
cerrado (se añaden tres componentes, no se elimina ninguno) y **todas** las reglas de producto.

## Decisiones ya cerradas

- **Sidebar: riel fijo de 74 px.** Sin variante expansible. Razonamiento en
  `RESPUESTAS-A-LA-REVISION.md` §5, con el plan B si en uso real cuesta encontrar las secciones.
- **`HealthDonut`: fuera de esta propuesta.** Se queda en el catálogo, sale del panel general, y en
  Informes no entra: sería funcionalidad nueva. Ver §6.

## Nada pendiente de diseño

Las cuatro pantallas rediseñadas y los cuatro pasos del asistente están dibujados en los dos temas.
Alertas, Eventos, Informes y Ajustes son cambio de tokens y no necesitan mockup (ver
`mockups/README-mockups.md`).
