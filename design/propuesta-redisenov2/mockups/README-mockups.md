# Mockups

## `icons-sprite.svg` y `icons-hoja-de-contacto.html`

El sprite de los 15 iconos, listo para montar en `AppShell`, y una hoja de contacto para revisarlos a
32 / 24 / 16 / 12 px en claro y en oscuro. Ver `cambios/componentes/Icon.md` para la API y los mapas
semánticos.

## `smartdisk-v3.html`

Un solo fichero **autocontenido**: ábrelo con doble clic, sin servidor ni dependencias. Todo va inline
(tipografías incluidas), así que se puede comparar directamente contra los snapshots de `salida/html/`.

Arriba tiene dos conmutadores:

- **pantalla** — Panel · Detalle · Pruebas · Asistente inicial;
- **tema** — Claro · Oscuro.

Dentro del asistente, el indicador de pasos de la cabecera cambia entre los cuatro pasos, y en el paso 3
las tres tarjetas de perfil son seleccionables: la tabla de umbrales reacciona.

Además es navegable: el riel cambia de pantalla, «Abrir el disco» lleva al detalle y «Probar disco»
lleva a pruebas.

### Qué cubre

| Pantalla | Claro | Oscuro | Notas |
|---|---|---|---|
| Panel general | ✔ | ✔ | héroe, rejilla de 4 discos, sucesos, reparto de estados |
| Detalle de disco | ✔ | ✔ | cabecera de identidad, 4 métricas con sparkline, gráfica con eje y hueco |
| Pruebas y diagnóstico | ✔ | ✔ | prueba en curso con cifra de display, 3 tarjetas, historial |
| Asistente inicial | ✔ | ✔ | **los cuatro pasos**, navegables desde el indicador de la cabecera |

### Qué NO cubre, y por qué

- **Alertas, Eventos, Informes y Ajustes**: son cambio de tokens (más la estructura interna de
  `AlertCard` y `EventRow`). No hay composición nueva que mostrar, así que un mockup no aportaría nada
  que no esté en `cambios/02`, `03`, `05` y `06`. La píldora con icono y la fila de evento con icono se
  pueden ver en el bloque «Sucesos del sistema» del panel, que usa el mismo `EventRow`.
- *(Los pasos 1, 3 y 4 del asistente ya están dibujados: se añadieron en la segunda entrega.)*
- **Diálogos y estados de carga/vacío/error**: descritos pantalla a pantalla y en
  `cambios/09-chrome-y-estados.md` §3. Su estructura no cambia respecto a lo que ya tienes.
- **Ventana mínima (1024 × 560)**: el mockup está fijado a 1280 × 800 para poder compararlo con
  `*__1280x800__completa.png`. El comportamiento a 1024 px está descrito en el §6 de cada pantalla.

### Cómo leerlo

Los datos son **cuatro** discos (dos SSD, un HDD y un USB sin SMART), no los dos del fixture
`e2e/ui/fixtures/respuestas.ts`. Es a propósito: con dos discos no se ve cómo respira la rejilla llena,
que era justamente el problema del panel actual. Con el fixture de dos, la rejilla
`repeat(auto-fill, minmax(272px, 1fr))` estira cada tarjeta hasta ~580 px y **no** deja hueco.

Las series temporales del mockup son sintéticas (una suma de senos), no datos reales. Sirven para juzgar
grosor, color, relleno y cómo se ve el hueco; no para juzgar la forma de una curva de temperatura real.

### Si necesitas medir algo

Es HTML normal: inspecciona con las herramientas del navegador y verás los `--sdm-*` resueltos en cada
elemento. Ojo, en el mockup los tokens están **repetidos inline** en un `<style>` propio para que el
fichero sea autocontenido; la fuente de verdad para implantar es la tabla de `cambios/00-tokens.md`.
