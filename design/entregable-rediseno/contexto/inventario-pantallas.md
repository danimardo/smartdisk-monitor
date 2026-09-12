# Inventario de pantallas

Ocho secciones más los estados de cada una. Entre paréntesis, el nombre del fichero de captura
(sin el sufijo `__claro` / `__oscuro` / viewport).

## Secciones

### 1. Panel general (`panel-general`) — ruta `/`
Rejilla de tarjetas de disco (`DiskCard`), una por dispositivo. Datos de ejemplo: un NVMe Samsung
990 PRO sano (41 °C, 3 % de desgaste, 12 % de actividad, volumen C: con 1,10 TB libres de 1,82 TB)
y un disco USB "Copia externa" sin SMART (todo "No disponible", rotulado en gris, **no** en rojo).
- **Vacío** (`estado-panel-vacio`): sin discos. Mensaje "sin dispositivos".
- **Error** (`estado-error-pantalla`): fallo al cargar el inventario. `+error.svelte`: frase humana +
  detalle técnico plegable.

### 2. Alertas (`alertas`) — ruta `/alerts`
Columna izquierda de 470 px con `SegmentedControl` (activas / resueltas / archivadas / todas) y
lista de `AlertCard`; a la derecha, el detalle: píldora de severidad, titular, explicación humana,
rejilla de hechos, acciones (reconocer / silenciar / archivar) y cronología de ocurrencias. Ejemplo:
una alerta de "Desgaste elevado" en el Samsung, severidad *warn*, 3 ocurrencias.
- **Vacío** (`estado-alertas-vacio`): sin alertas en el filtro actual.
- **Diálogo destructivo** (`estado-dialogo-destructivo`): confirmación de "Archivar" con impacto.

### 3. Eventos (`eventos`) — ruta `/events`
Lista virtualizada de eventos del registro de Windows con `FilterBar` (nivel + proveedor) y panel
de detalle de 420 px con el XML original. Ejemplo: un evento *info* de `Microsoft-Windows-Ntfs` con
asociación exacta y uno *error* de `disk` con asociación **inferida** (etiquetada como tal).
- **Vacío** (`estado-eventos-vacio`): sin eventos.
- **Detalle con XML** (`estado-evento-detalle`): fila seleccionada, `CodeOutput` con el XML.

### 4. Pruebas y diagnóstico (`pruebas`) — ruta `/tests`
Selector de disco y volumen, tres tarjetas (`Card`) de prueba —lectura/escritura, escaneo del
sistema de archivos, autotest SMART corto—, la prueba en curso si la hay, y el historial.
- **Diálogo de prueba** (`estado-dialogo-prueba`): confirmación de benchmark con impacto y comando.
- **Prueba en curso** (`estado-prueba-en-curso`): `ProgressBar` al 40 %, aviso de carga, botón
  cancelar.

### 5. Informes (`informes`) — ruta `/reports`
Selección de intervalo (`SegmentedControl` + `DateRangePicker`), discos incluidos, y el paquete de
diagnóstico: vista previa del contenido del ZIP y campos que se anonimizan.

### 6. Ajustes (`ajustes`) — ruta `/settings`
Secciones apiladas: apariencia (tema, idioma, acento), frecuencias de muestreo, umbrales de alerta,
retención e historial, registro de actividad, comportamiento al cerrar la ventana, y borrado de
datos. Es la pantalla más densa en controles.

### 7. Detalle de disco (`detalle-disco`) — ruta `/disks/disk-0`
Identidad del disco (modelo, serie, firmware, bus), capacidades, contadores SMART en `DataRow`, y
`TimeSeriesChart` de temperatura (con un hueco explícito en la serie).

### 8. Configuración inicial (`configuracion-inicial`) — ruta `/onboarding`
**Sin diseñar ni implementar todavía.** Hoy es un `EmptyState` "no implementado". Debería ser un
asistente de primer arranque (US-002). Es la pantalla que más necesita propuesta desde cero.

## Chrome siempre presente

- **Sidebar** izquierda: navegación + lista de discos con punto de estado y temperatura + estado
  global abajo + botón pausar/reanudar.
- **Toolbar** superior: título de la pantalla, botón "?" (diálogo *Acerca de*), estado global
  ("Todo en orden" / "N necesitan atención" / "En pausa"), frescura ("hace X"), botón "Actualizar".
- **Diálogo Acerca de** (`estado-dialogo-acerca-de`): nombre, versión y autor; cierra con Escape.

## Datos de ejemplo (fixtures)

Todas las capturas usan `e2e/ui/fixtures/respuestas.ts`, validado contra los esquemas Zod reales.
Resumen: 2 discos (1 NVMe sano, 1 USB sin SMART), 1 alerta activa de desgaste, 2 eventos de Windows,
historial de pruebas vacío + 1 benchmark en curso al 40 %, ajustes de fábrica, acento del sistema
`#0067c0`.
