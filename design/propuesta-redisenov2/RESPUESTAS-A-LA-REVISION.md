# Respuestas a la revisión

Los seis puntos, en el mismo orden. Tres eran material que faltaba y va incluido; tres eran decisiones
que había dejado abiertas y aquí las cierro.

---

## 1. Los 15 iconos como SVG — **incluido**

Tenías razón: describirlos con palabras no es entregable.

- `mockups/icons-sprite.svg` — el sprite completo, listo para montar. 15 `<symbol>`, `viewBox` 24×24,
  `stroke="currentColor"`, `stroke-width` 1.7, cero literales de color. Copia el bloque `<svg>` entero
  en `AppShell` (una sola vez, antes del contenido) y usa `<use href="#i-temp"/>`.
- `mockups/icons-hoja-de-contacto.html` — los 15 a 32 / 24 / 16 / 12 px, en claro y en oscuro, para
  revisarlos de un vistazo antes de montarlos. Ábrelo con doble clic.
- `cambios/componentes/Icon.md` sigue teniendo la API, los mapas semánticos y la regla de `aria-label`.

Están hechos para este proyecto, sin dependencias ni licencias de terceros. Si alguno no te convence
—el de disco mecánico es el que menos me gusta— dime cuál y lo redibujo.

## 2. Ficheros de Bricolage Grotesque — **instrucciones incluidas, binarios no**

Ver `cambios/00b-tipografia-y-fuentes.md`. Resumen:

- No adjunto los `.woff2` a propósito: los binarios deben venir de la fuente oficial con su hash, no de
  un ZIP de diseño. Va el origen (`github.com/ateliertriay/bricolage`), la licencia (**SIL OFL 1.1**,
  que permite empaquetarla en un producto comercial conservando `OFL.txt` y el aviso de copyright), y
  los comandos exactos de `fonttools` para generar el subconjunto.
- **Corrección a lo que puse en `00-tokens.md`:** Bricolage es una fuente **variable** (peso, anchura,
  óptico). Sale mejor **un** `woff2` variable con el eje recortado a 500-700 que tres estáticos.
- Ojo con el subconjunto: hay que incluir explícitamente `U+00B0` (°), `U+00B7` (·) y la característica
  `tnum`. Un subconjunto latino por defecto se come los dos primeros y esta interfaz los usa en todas
  las pantallas.
- Aprovecha para empaquetar también **Instrument Sans**: hoy `tokens.css` la carga por `@import` de
  Google Fonts, y una aplicación de escritorio no debería hacer una petición de red para pintarse.
- **Si prefieres no gestionar una segunda familia**, el §«Si prefieres no empaquetar…» de `00b` explica
  cómo usar Instrument Sans 600 para las cifras. Se pierde carácter, no función, y **no hay que tocar
  ningún componente**: `--sdm-font-display` pasa a valer lo mismo que `--sdm-font-sans` y ya está.

## 3. Pasos 1, 3 y 4 del asistente — **dibujados**

Los tres están en `mockups/smartdisk-v3.html` → pestaña **Asistente inicial**. Los cuatro pasos son
navegables desde el indicador de la cabecera, y los botones «Atrás» / «Continuar» encadenan.

- **Paso 1 · Bienvenida** — qué hace la aplicación en dos frases, la garantía «SmartDisk solo lee. No
  modifica, no repara y no borra nada de tus discos» destacada en verde, tres tarjetas (leer, avisar,
  probar) y el primario «Buscar mis discos».
- **Paso 3 · Alertas** — el que decías, y estoy de acuerdo en que era el importante. Tres perfiles
  seleccionables, tabla de umbrales que reacciona al perfil elegido, y dos `Switch` (notificación de
  Windows, arrancar con el sistema).
- **Paso 4 · Listo** — resumen de lo elegido en tres líneas, la primera lectura ya en marcha con barra
  de progreso, y «Ir al panel».

`cambios/08-onboarding.md` §2 sigue siendo la descripción de referencia; ahora todos los pasos tienen
su mockup en claro y en oscuro.

## 4. Tabla de umbrales por perfil — **incluida**

`cambios/08b-perfiles-de-alerta.md`: los doce valores de los tres perfiles, con la clave de
`settings.alerts` al lado de cada uno.

Cuatro reglas que importan para implementarlo:

1. El **límite del fabricante manda sobre el perfil** (mínimo de los dos). El perfil no desactiva
   `temp_above_vendor_limit`.
2. Espacio libre: gana el criterio que salte primero entre porcentaje y valor absoluto, como hoy.
3. Los discos sin SMART no participan de temperatura, desgaste ni errores de medios; sí de capacidad y
   eventos. **Nunca** cuentan como avería.
4. Elegir perfil escribe los doce valores **y** `settings.alerts.profile`. En cuanto el usuario cambie un
   número a mano, el perfil pasa a `custom` y la interfaz muestra «Personalizado (a partir de
   Equilibrado)», para no mentir sobre qué está activo.

Las claves son orientativas: si tu esquema las llama de otra forma, manda tu esquema.

## 5. Sidebar: **riel fijo de 74 px**. Decidido.

Me pediste que eligiera, así que elijo, con el razonamiento por si no te convence:

- El problema medido era el **ancho**: a 1024 px, 250 px de barra son el 24 % de la ventana, y la ventana
  mínima es donde el rediseño tiene que aguantar. El riel devuelve 176 px al contenido siempre, sin
  interacción ni estado.
- El riel expansible devuelve los mismos 176 px **solo mientras no lo usas**. Y añade lo que ya
  anticipabas: atrapar el foco, cerrar con Escape, decidir el comportamiento táctil, y un solapamiento
  que tapa contenido justo cuando el usuario está navegando. Es coste de mantenimiento permanente a
  cambio de una etiqueta de texto.
- Lo que se pierde, y no lo minimizo: las **etiquetas** (mitigadas con `title` + `aria-label`, que dan el
  tooltip nativo de Windows sin librería) y la **temperatura por disco de un vistazo** — que estaba
  duplicada, porque ya está en cada `DiskCard` del panel, y ahora también en el héroe.

Seis destinos con iconos claros se aprenden en dos usos. Si al probarlo en la aplicación real ves que la
gente no encuentra «Eventos» o «Informes», el arreglo barato no es el riel expansible: es **etiquetas
permanentes de 9 px bajo cada icono**, que caben en 74 px y no añaden ningún estado. Dímelo y te paso la
variante.

## 6. `HealthDonut` en Informes: **no entra ahora.** Fuera de esta propuesta.

Lo dejé como sugerencia y con razón: no debería estar en un entregable de rediseño.

- El `HealthDonut` **se queda en el catálogo** y **no se toca**. No lo elimines.
- En el **panel general sale** y lo sustituye el bloque «Reparto de estados» (iconos + cifras + barra de
  proporción), porque con 2-4 discos un anillo de cuatro segmentos no se lee. Eso sí es parte del
  rediseño y está en `cambios/01`.
- En **Informes** no lo pongas por mi propuesta. Ahí el reparto se calcula sobre muchas muestras del
  intervalo y el anillo sí sería legible, pero es una **funcionalidad nueva** —hay que decidir qué mide
  exactamente: ¿tiempo en cada estado?, ¿número de muestras?, ¿discos?— y eso es una conversación de
  producto, no de presentación. Si os interesa, lo abordamos como cambio propio.

---

## Y una cosa que no preguntaste

Al dibujar el paso 3 me di cuenta de que el asistente necesita una salida clara **en cada paso**, no solo
al principio: «Omitir y usar los valores de fábrica» está en la cabecera y es visible en los cuatro. Si
alguien lo pulsa en el paso 3, se aplican los valores del perfil **Equilibrado** y se marca
`onboarding.completedAt`. Omitir no debe dejar la configuración a medias.
