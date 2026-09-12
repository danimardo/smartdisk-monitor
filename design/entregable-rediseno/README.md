# Entregable de rediseño — SmartDisk Monitor

Paquete para pasar a un diseñador (persona y/o Claude Design) el estado **actual** de la interfaz y
recibir de vuelta una propuesta de mejora estética que se pueda implementar sin ambigüedad.

> **La aplicación es de escritorio (Tauri + WebView2 en Windows).** No es una web: no hay scroll de
> página infinito, la ventana tiene un tamaño mínimo real (1024 × 560 px CSS) y todo tiene que
> funcionar en tema claro y oscuro sin condicionales.

---

## Qué hay en este paquete

| Carpeta / fichero | Contenido |
|---|---|
| `salida/capturas/` | PNG de cada sección y estado, en tema claro y oscuro, a 1280×800 y a la ventana mínima 1024×560. |
| `salida/html/` | Un `.html` autocontenido por sección y tema: DOM real + todo el CSS embebido + la tipografía en base64. Se abre en cualquier navegador sin servidor. **No es interactivo**, es una referencia fiel del render. |
| `salida/indice.html` | Galería con todas las capturas juntas para revisarlas de un vistazo. |
| `contexto/inventario-pantallas.md` | Las 8 secciones, su ruta, sus estados y los datos de ejemplo con que están pintadas. |
| `contexto/catalogo-componentes.md` | Catálogo cerrado de componentes: qué es cada uno y sus reglas. |
| `contexto/tokens-referencia.css` | **Fuente única de verdad visual.** Todo color, radio, sombra, espaciado y tamaño sale de aquí. |
| `contexto/tokens-referencia.json` | Los mismos tokens en JSON. |
| `contexto/tailwind.config-referencia.cjs` | Cómo se mapean los tokens a utilidades de Tailwind. |
| `contexto/ui-design.md` | Sistema de diseño y reglas de interfaz completas. **Vinculante.** |
| `contexto/ui-contract.md` | Contrato de datos y estados que la interfaz tiene que cubrir. |
| `COMO-ENTREGAR-EL-REDISENO.md` | **El formato en que hay que devolver la propuesta.** Léelo antes de empezar a diseñar. |

`generar-indice.mjs` y `empaquetar.ps1` regeneran la galería y el ZIP; no hace falta tocarlos.

---

## Restricciones que el rediseño NO puede romper

Son reglas del proyecto, no preferencias. Una propuesta que las incumpla no se puede implementar.

1. **Cero valores visuales literales.** Ningún color, radio, sombra o tamaño de fuente escrito a
   mano. Todo es un token `--sdm-*` o una utilidad de Tailwind que lo mapea. Si el rediseño
   necesita un valor que no existe, **se propone un token nuevo** (nombre, valor en claro, valor en
   oscuro, para qué sirve).
2. **Tema claro y oscuro, sin condicionales.** Cada pantalla tiene que quedar bien en los dos con
   el mismo marcado; el cambio lo hacen solo los tokens.
3. **Ventana mínima 1024 × 560 sin recortes silenciosos.** A ese tamaño no puede aparecer scroll
   horizontal ni contenido cortado. El escalado de Windows (125 %, 150 %, 200 %) reduce el espacio
   CSS disponible, no el tamaño del texto.
4. **Material traslúcido, nunca `backdrop-filter` a mano.** Hay tres materiales definidos
   (`.sdm-material`, `.sdm-material-chrome`, `.sdm-material-overlay`). No se apilan materiales: una
   tarjeta no contiene otra tarjeta.
5. **Catálogo de componentes cerrado.** Preferible recomponer los que hay. Un componente nuevo
   exige justificación según `ui-design.md` §3.
6. **El color nunca es el único portador de significado.** Siempre lo acompaña texto o icono.
7. **"No compatible" ≠ "averiado":** gris, jamás rojo.
8. **Un dato ausente es "No disponible"**, nunca cero ni guion.
9. **Todo texto pasa por el sistema de traducción** (es/en), incluidos `aria-label`, `title` y
   `alt`. El rediseño puede proponer cambios de copy, pero listándolos aparte.
10. **La tipografía es Instrument Sans**, con peso limitado a 400–600 (no hay negrita 700).

La **definición de "terminado"** está en `contexto/ui-design.md` §8.

---

## Sobre las capturas

- Los datos son de ejemplo (un NVMe sano + un disco USB sin SMART, una alerta de desgaste, dos
  eventos de Windows, un benchmark al 40 %). Son realistas y están validados contra el contrato de
  datos real.
- `configuracion-inicial` (`/onboarding`) todavía **no está diseñada ni implementada**: aparece
  como un estado vacío "no implementado". Es una de las pantallas que más necesita propuesta.
- Los snapshots HTML llevan **todo** el CSS de la aplicación (incluidas todas las utilidades de
  Tailwind que se usan en algún sitio), así que sirven para experimentar cambiando clases y ver el
  efecto con los estilos reales.
- En los snapshots HTML, los **gráficos de serie temporal** (detalle de disco) pueden salir como
  un rectángulo vacío: se dibujan por JavaScript midiendo su contenedor, y el snapshot no lleva
  JS. Para ver el gráfico real, usa la captura PNG correspondiente.
- El título de la Toolbar en el snapshot de detalle de disco dice "Panel general": es una
  peculiaridad de la app actual (esa pantalla no fija su propio título), no del snapshot.

## Cómo se regeneró (para el equipo)

```
SDM_CAPTURAS=1 pnpm exec playwright test capturas
node design/entregable-rediseno/generar-indice.mjs
pwsh design/entregable-rediseno/empaquetar.ps1
```

La spec `e2e/ui/capturas.spec.ts` reutiliza el IPC falso del plano de interfaz; está desactivada
salvo que se pase `SDM_CAPTURAS=1`.
