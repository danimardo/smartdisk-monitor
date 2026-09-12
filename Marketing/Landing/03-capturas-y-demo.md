# 03 · Capturas y demo

Cada captura existe en versión clara y oscura en `docs/screenshots/` (mismo par de imágenes que usa
el `README.md`, generadas con la suite de interfaz — no son montajes). Un bloque por pantalla, en el
orden recomendado para la página.

## Panel general

**Ficheros:** `docs/screenshots/panel-general-claro.png` / `panel-general-oscuro.png`

**Título del bloque:** Todo el equipo, de un vistazo

**Texto:** El disco protagonista con su serie de fondo, una tarjeta por disco con temperatura,
salud, actividad y capacidad, los últimos sucesos del sistema y el reparto de estados. La primera
pantalla que ves al abrir la aplicación, y normalmente la única que necesitas.

## Detalle de disco

**Ficheros:** `docs/screenshots/detalle-disco-claro.png` / `detalle-disco-oscuro.png`

**Título del bloque:** Cuando un disco necesita más atención

**Texto:** Temperatura y actividad en el tiempo, con las zonas de aviso y crítico dibujadas sobre
la propia gráfica, desgaste y horas de encendido, y los contadores SMART completos con su delta
respecto a la lectura anterior.

## Alertas

**Ficheros:** `docs/screenshots/alertas-claro.png` / `alertas-oscuro.png`

**Título del bloque:** Avisos agrupados, no un aluvión

**Texto:** Cada alerta agrupa su cronología de ocurrencias en vez de repetirse por cada muestra, con
acciones para reconocerla, silenciarla o archivarla. El color que ves siempre refleja la peor
alerta sin resolver.

## Eventos del sistema

**Ficheros:** `docs/screenshots/eventos-claro.png` / `eventos-oscuro.png`

**Título del bloque:** Los eventos de Windows, ya explicados

**Texto:** Los eventos del registro de Windows relacionados con almacenamiento, correlacionados con
el disco al que corresponden —de forma exacta, inferida o marcados como desconocidos cuando no hay
certeza—, con el XML original disponible si lo necesitas.

## Pruebas y diagnóstico

**Ficheros:** `docs/screenshots/pruebas-claro.png` / `pruebas-oscuro.png`

**Título del bloque:** Ponlo a prueba cuando quieras, con red de seguridad

**Texto:** Benchmark de lectura/escritura con Microsoft DiskSpd, `chkdsk /scan` y autotest SMART
corto, con progreso en vivo. La prueba se detiene sola si el disco se calienta o si el espacio
libre baja demasiado.

## Informes

**Ficheros:** `docs/screenshots/informes-claro.png` / `informes-oscuro.png`

**Título del bloque:** Comparte un incidente, no todo tu disco

**Texto:** Exportación a CSV, JSON y HTML imprimible, y un paquete ZIP de diagnóstico anonimizado
por defecto —sin números de serie ni nombres de equipo o usuario— listo para enviar a quien te
ayude a diagnosticar.

## Ajustes

**Ficheros:** `docs/screenshots/ajustes-claro.png` / `ajustes-oscuro.png`

**Título del bloque:** A tu manera, sin perder el control

**Texto:** Apariencia, frecuencias de recopilación, perfiles y umbrales de alerta, retención del
historial, y la asistencia con IA —opcional y apagada de fábrica— en un único sitio.

## Nota sobre vídeo

Hay material de vídeo ya producido en `Marketing/Videos/` (guiones en `Prompts.md`) pensado para
redes sociales en formato vertical de 10 s por bloque. Si la landing admite un vídeo de cabecera o
una sección de demo animada, esos clips —o el guion que los describe— son el punto de partida, no
haría falta escribir un guion nuevo.
