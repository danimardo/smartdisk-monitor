# 05 · Informes (`/reports`)

**Cambio solo de tokens.** Sin mockup propio.

## 1. Diagnóstico

Referencia: `capturas/informes__claro__1280x800__completa.png`.

La pantalla está bien resuelta: selección de intervalo, discos incluidos y vista previa del ZIP con los
campos que se anonimizan. Es la única pantalla del entregable que no sufre el problema de vacío, porque
su contenido es un formulario.

Dos detalles menores, ambos de token o de composición:

- La vista previa del contenido del ZIP usa el mismo peso que las etiquetas del formulario, así que el
  bloque más informativo de la pantalla no destaca.
- La lista de campos anonimizados no lleva ninguna marca visual de «esto se enmascara».

## 2. Cambios de distribución

Ninguno.

## 3. Cambios por componente

- **`SegmentedControl`**, **`DateRangePicker`**, **`Switch`** — solo tokens.
- **Vista previa del ZIP** — el árbol de contenido pasa a `font-mono` `text-2xs` dentro de un bloque
  `bg-glass-3` con radio `inner`, en lugar de texto suelto. Es composición, no cambio de componente
  (equivale a usar `CodeOutput` sin la barra de copiar).
- **Campos anonimizados** — cada fila lleva `#i-shield` de 14 px en `text-fg-dim` delante. El icono
  comunica «protegido», no un estado de salud, así que **no** usa color semántico.
- **`HealthDonut`** — este es su sitio: en Informes el reparto se calcula sobre muchas muestras del
  intervalo y el anillo sí es legible. Si hoy no se usa aquí, queda como sugerencia, no como cambio.

## 4. Estados

- **Cargando**: botón de exportar en `loading`; el formulario sigue usable.
- **Vacío**: si el intervalo elegido no tiene muestras, `EmptyState kind="empty"` en la vista previa,
  con el intervalo citado. El botón de exportar queda deshabilitado con `disabledReason`.
- **No compatible**: los discos sin SMART aparecen en la lista de inclusión marcados «solo capacidad
  y eventos». Se pueden incluir; el informe dirá qué falta.
- **Error de fuente**: fallo al generar el ZIP → `Toast` de error **más** frase persistente en la
  tarjeta, nunca solo el toast (un toast se pierde).
- **Dato obsoleto**: el informe declara en su portada la marca de la última lectura válida por disco.

## 5. Claro y oscuro

Solo tokens.

## 6. Ventana mínima

Sin cambios. El formulario es de una columna con anchos máximos; a 1024 × 560 hace scroll como hoy.
