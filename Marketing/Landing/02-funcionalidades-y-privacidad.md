# 02 · Funcionalidades y privacidad

## Antetítulo de sección

Todo lo que necesitas saber de tus discos, en una sola pantalla

## Cuadrícula de funcionalidades (título + una frase cada una)

### Salud real, no solo una cifra
Lee los datos SMART y NVMe nativos de cada disco —temperatura, desgaste, errores, horas de
encendido— con `smartctl` y las fuentes propias de Windows. Si el hardware no expone SMART (RAID,
USB, una máquina virtual), te lo dice sin fingir una alerta que no existe.

### Panel general de un vistazo
Estado global del equipo, cuántos discos están bien, cuáles tienen un aviso y cuáles están en
crítico, y una tarjeta por disco con temperatura, salud, actividad y capacidad. Sin menús que
abrir para saber si algo va mal.

### Gráficas que avisan antes de que preguntes
Series de temperatura y actividad con las zonas de aviso y crítico dibujadas sobre la propia
gráfica: se ve si un valor es bueno o peligroso sin tener que memorizar umbrales.

### Alertas que no gritan por nada
Agrupadas por condición, con histéresis para no repetir el mismo aviso en cada muestra, visibles
desde la aplicación y desde el icono de la bandeja del sistema — verde, ámbar o rojo, de un
vistazo.

### Los eventos de Windows, ya traducidos a "esto es de tu disco"
Correlaciona automáticamente los eventos del registro de Windows relacionados con almacenamiento
con el disco al que pertenecen, para no tener que bucear en el Visor de eventos.

### Pon un disco a prueba cuando quieras
Benchmark de lectura/escritura con Microsoft DiskSpd, `chkdsk /scan` y autotest SMART corto, bajo
demanda. La prueba se detiene sola si el disco se calienta o si el espacio libre baja demasiado.

### Explicación en lenguaje llano, si la quieres
Ayuda opcional con IA —apagada de fábrica— que traduce el detalle técnico de una alerta o un disco
a lenguaje claro, con posibles pasos a seguir. El instalador incluye una clave de demostración
gratuita para probarlo con un clic, sin necesidad de cuenta.

### Comparte un incidente sin compartir tu disco entero
Exportación a CSV, JSON y HTML imprimible, más un paquete ZIP de diagnóstico anonimizado por
defecto: número de serie, nombre de equipo y usuario fuera antes de que el fichero salga de tu
mano.

## Bloque destacado de privacidad

### Titular
Tus datos no salen de tu equipo. Punto.

### Cuerpo
SmartDisk Monitor no tiene servidor central, no pide cuentas y no manda telemetría a ningún sitio.
Todo el historial vive en una base de datos SQLite local, en tu propio equipo. La única conexión a
Internet que existe es opcional y bajo tu control: la ayuda con IA, apagada de fábrica, que solo se
activa si tú configuras una clave. Sin clave, la aplicación no hace ninguna conexión a Internet.

### Puntos de apoyo
- Sin cuentas ni inicio de sesión.
- Sin telemetría, nunca.
- Historial local en SQLite, con retención configurable.
- La IA es opcional, apagada de fábrica, y anonimiza el texto técnico antes de enviarlo.
