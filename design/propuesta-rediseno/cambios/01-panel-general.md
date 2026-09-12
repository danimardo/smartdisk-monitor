# 01 · Panel general (`/`)

Mockup: `mockups/smartdisk-v3.html` → pestaña **Panel**, en claro y en oscuro.

## 1. Diagnóstico

Referencia: `capturas/panel-general__claro__1280x800__completa.png` y su gemela oscura.

- Con los dos discos del fixture la rejilla ocupa 230 px de los 800 de la ventana. Quedan ~570 px de
  lienzo vacío bajo las tarjetas: la pantalla parece a medio cargar.
- Las tres magnitudes son texto plano del mismo tamaño y color. Nada dice si 41 °C está bien.
- El disco USB muestra tres «No disponible» a 20 px que pesan visualmente **más** que los datos reales
  del disco sano de al lado. Es exactamente lo contrario de lo que interesa.
- La `DiskCard` del USB acaba en «Sin volúmenes montados» y deja un hueco donde las demás tienen barra.
- El estado global aparece dos veces: píldora en la `Toolbar` y texto al pie de la `Sidebar`.
- En oscuro el conjunto queda gris plano: no se percibe el material porque no hay nada detrás del cristal.

## 2. Cambios de distribución

Estructura nueva, de arriba abajo, con `gap: var(--sdm-space-5)` (18 px) y `padding: var(--sdm-space-6)`:

1. **`HeroPanel`** — alto fijo `var(--sdm-hero-height)` (246 px), ancho completo. Interior en dos
   columnas: contenido flexible a la izquierda, bloque de hechos de **290 px** a la derecha.
2. **Rejilla de `DiskCard`** — `repeat(auto-fill, minmax(272px, 1fr))`, `gap` 16 px. Con cuatro discos
   caben cuatro en fila a 1280 px; con dos, cada tarjeta se estira hasta ~580 px y **no** queda hueco.
3. **Fila inferior** — rejilla `1fr 300px`: «Sucesos del sistema» (3 filas) y «Reparto de estados».

La suma a 1280 × 800 es 246 + 18 + 252 + 18 + 190 = 724 px de contenido en 744 disponibles: la pantalla
llena sin scroll con cuatro discos, y con más discos la región hace scroll (nada se recorta).

El lienzo pasa a llevar el degradado `--sdm-bg` → `--sdm-bg-2` a 160°, que es lo que da vida al
desenfoque de las tarjetas. Hoy es plano y por eso el material no se lee.

## 3. Cambios por componente

- **`HeroPanel`** — nuevo. API y estados en `componentes/HeroPanel.md`.
- **`DiskCard`** — recompuesta: cabecera de 52 px con sparkline, icono de tipo de bus, cifras de display.
  Ver `componentes/DiskCard.md`.
- **`Sidebar`** — riel de 74 px, sin lista de discos ni texto de estado. Ver `componentes/Sidebar.md`.
- **`Toolbar`** — conserva la píldora de estado global (ahora es la única) y añade icono a la píldora.
- **`EventRow`** — el nivel pasa de píldora de texto a cuadrado de 26 px con icono + píldora. Solo
  estructura interna; ver `componentes/Icon.md` para el mapa nivel → icono.
- **«Reparto de estados»** — bloque nuevo compuesto con `Card` + `Icon` + barra de proporción de 9 px.
  **No** es un componente: es composición dentro de la pantalla. Sustituye a `HealthDonut` aquí porque
  con 2-4 discos un anillo de cuatro segmentos es ilegible; `HealthDonut` se mantiene en el catálogo
  para Informes, donde el reparto es sobre muchas muestras.

## 4. Estados

| Estado | Cómo queda |
|---|---|
| **Cargando** | `HeroPanel` en modo esqueleto: cifra y curva sustituidas por bloques `bg-glass-3` sin animación de brillo (solo opacidad, por `prefers-reduced-motion`). La rejilla muestra 2 `DiskCard` esqueleto. |
| **Vacío** (sin discos) | Sin héroe y sin rejilla: `EmptyState kind="empty"` centrado, ancho máximo 520 px, con acción «Buscar dispositivos otra vez». Mismo mensaje que hoy (`estado-panel-vacio`), solo cambia el encuadre. |
| **No compatible** | El disco USB **sí** aparece en la rejilla, con icono USB en gris, píldora «Sin datos SMART», las tres magnitudes como «—» a `text-xs` en `text-fg-dim` y su barra de capacidad normal (esa sí la tenemos). Nunca cuenta para el héroe ni para «necesitan atención». |
| **Error de fuente** | No tumba la pantalla: la tarjeta afectada se degrada a `EmptyState kind="error"` dentro de la rejilla, con frase humana y `<details>` técnico. Si falla el inventario completo, `+error.svelte` como hoy (`estado-error-pantalla`). |
| **Dato obsoleto** | La `Toolbar` cambia «hace 12 s» por «hace 14 min» en `text-warn`, y el `HeroPanel` añade bajo la cifra «último dato válido a las 12:41». La curva no se extiende: termina donde terminan los datos. |

## 5. Claro y oscuro

Mockup en los dos temas. Nada condicional en el marcado: todo sale de los tokens. La única diferencia
real es `--sdm-on-accent`, que en oscuro es tinta (ver aviso en `00-tokens.md` §2).

## 6. Ventana mínima (1024 × 560)

- El riel de 74 px deja 950 px de contenido (hoy la barra de 250 px dejaba 774).
- `HeroPanel`: por debajo de **1100 px** el bloque de hechos de 290 px pasa de columna derecha a
  **fila de cuatro chips** bajo el texto, y el alto del héroe pasa de 246 a **210 px**.
- Rejilla: `minmax(272px, 1fr)` da 3 columnas a 950 px de ancho. Con cuatro discos, 3 + 1.
- Alto disponible 504 px: entran héroe (210) + una fila de tarjetas (252) y la fila inferior queda
  fuera de la vista; **la región hace scroll**, no se recorta ni se solapa.
- Por debajo de **900 px** la fila inferior pasa a una sola columna.
