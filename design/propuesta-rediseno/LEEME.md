# Propuesta de rediseño — SmartDisk Monitor v3

Empieza por **`RESUMEN.md`**: los seis cambios de fondo, qué no se toca y los cambios de texto.

Después, en este orden:

1. `cambios/00-tokens.md` — la tabla completa de tokens, con los ratios de contraste medidos sobre el
   material compuesto. **Nada visual vive fuera de este fichero.**
2. `cambios/componentes/` — antes/después de los diez componentes afectados. Tres son nuevos
   (`Icon`, `Sparkline`, `HeroPanel`), cada uno con su justificación contra `ui-design.md` §3.
3. `cambios/01`…`09` — un fichero por pantalla, con diagnóstico referido a tus capturas, cambios de
   caja, estados y comportamiento a 1024 × 560.
4. `mockups/smartdisk-v3.html` — mockup navegable y autocontenido, cuatro pantallas y dos temas.

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

## Una decisión que conviene que confirmes

La `Sidebar` pasa de 250 px con etiquetas a un riel de 74 px con solo iconos. Devuelve 176 px de ancho al
contenido, pero se pierden las etiquetas de texto y la temperatura por disco de un vistazo. En
`cambios/componentes/Sidebar.md` §Alternativa hay una variante expansible al pasar el ratón que conserva
las etiquetas, a cambio de gestionar foco y Escape. Dilo antes de montar el riel y lo diseño.

## Pendiente de diseño

Los pasos 1, 3 y 4 del asistente inicial están descritos pero no dibujados (`cambios/08` §2), porque
reutilizan componentes del catálogo sin composición nueva. Si los quieres como mockup, se añaden.
