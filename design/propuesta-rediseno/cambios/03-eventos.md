# 03 · Eventos (`/events`)

**Cambio solo de tokens más la estructura interna de `EventRow`.** Sin mockup propio: `EventRow` se ve
en el bloque «Sucesos del sistema» del mockup del panel, que usa la misma fila.

## 1. Diagnóstico

Referencia: `capturas/eventos__claro__1280x800__completa.png`.

- Todas las filas pesan lo mismo. Un *error* de `disk` y un *info* de `Ntfs` se distinguen solo por
  el texto de una píldora pequeña: en una lista virtualizada de miles de filas no se barre.
- La etiqueta «asociación inferida» compite en peso con el proveedor y el identificador.

## 2. Cambios de distribución

Ninguno. `FilterBar` arriba, `VirtualList` a la izquierda, panel de detalle de 420 px a la derecha.

**Importante para la virtualización:** la altura de fila **no cambia** (sigue en 42 px). El cuadrado de
icono es de 26 px y cabe dentro; no hay que recalcular la ventana de la `VirtualList`.

## 3. Cambios por componente

- **`EventRow`** — el nivel pasa de píldora de texto a **cuadrado de 26 px, radio 8, fondo `-soft` y
  icono en el color del token**, seguido de la píldora de texto solo en el panel de detalle. Mapa:
  `error → #i-bolt`, `warning → #i-alert`, `info → #i-shield`. El color **nunca** viaja solo: el
  cuadrado lleva `aria-label` con el nombre del nivel.
- La etiqueta «asociación inferida» baja a `text-2xs` sobre `bg-unknown-soft` en `text-unknown`.
  Sigue siendo explícita, pero deja de competir con el dato.
- **`CodeOutput`** (XML del evento) — sin cambios de estructura; hereda los tokens.

## 4. Estados

- **Vacío** (`estado-eventos-vacio`): igual.
- **Detalle con XML** (`estado-evento-detalle`): igual.
- Cargando: filas esqueleto de 42 px, misma altura, para que la lista no salte.
- Error de fuente: si el registro de Windows no se puede leer, `EmptyState kind="error"` en la zona de
  lista, con el detalle técnico plegado. La pantalla no se cae.
- Dato obsoleto: marca de hora en `text-warn` cuando la última lectura pasa del intervalo de muestreo.

## 5. Claro y oscuro

Solo tokens.

## 6. Ventana mínima

Sin cambios. A 1024 px el panel de detalle de 420 px deja 530 px de lista, suficiente para la fila
completa; por debajo de 900 px el detalle se superpone como panel, **igual que hoy**.
