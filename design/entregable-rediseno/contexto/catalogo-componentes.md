# Catálogo de componentes

Catálogo **cerrado**: la interfaz se construye solo con estas piezas. Un componente nuevo exige el
criterio de `ui-design.md` §3. El rediseño debería, siempre que pueda, recomponer los que hay antes
que pedir uno nuevo.

Código: `src/lib/components/`. Definición visual: `tokens-referencia.css` + `ui-design.md`.

## Armazón

| Componente | Qué es |
|---|---|
| **AppShell** | Armazón de la ventana: `Sidebar` fija + `Toolbar` fija + región de contenido con scroll propio. El contenido pasa por debajo del chrome translúcido; nada de fondo opaco encima. |
| **Sidebar** | Barra lateral de material: secciones de navegación + lista de discos monitorizados con su punto de estado y temperatura. La selección se marca con material elevado + punto de acento, nunca con una barra de color. |
| **Toolbar** | Barra de herramientas: título/subtítulo de la pantalla a la izquierda; controles contextuales, estado global, frescura del dato y acción primaria a la derecha. Material de chrome: translúcida, sin sombra propia, filo superior. |

## Controles

| Componente | Qué es |
|---|---|
| **Button** | Botón cápsula. `primary` con degradado vertical del acento + brillo interior de 1 px; `secondary` y `ghost` en material translúcido. Una sola `primary` por pantalla. Toda acción que escriba datos o genere carga abre `ConfirmDialog` antes. |
| **SegmentedControl** | Selector excluyente en cápsula sobre material (intervalos 24 h / 7 d / 30 d, filtros de alertas). El segmento activo se eleva con material, no con relleno de color. |
| **RadioGroup** | Elección excluyente con descripción por opción (tema, comportamiento al cerrar, anonimización). Para etiquetas cortas, `SegmentedControl` es mejor. |
| **Select** | Desplegable para listas largas (idioma, retención, volumen objetivo). Para 2–4 opciones usa `SegmentedControl` o `RadioGroup`. |
| **Switch** | Preferencias booleanas. Etiqueta obligatoria; `hint` explica la consecuencia, no repite la etiqueta. |
| **TextField** | Texto o numérico (alias de disco, tamaño del archivo de prueba, retención). `error` en lenguaje comprensible; el detalle técnico aparte. |
| **DateRangePicker** | Intervalo personalizado: dos fechas, sin hora. |
| **FilterBar** | Filtros multiselección (eventos por nivel y proveedor). |

## Indicadores y datos

| Componente | Qué es |
|---|---|
| **StatusPill** | Píldora de estado sobre material: fondo translúcido del token + texto del token. Único indicador textual de salud; el color nunca viaja solo. |
| **StatusDot** | Punto de estado de 9 px. El color nunca es el único portador: lo acompaña una etiqueta o un texto alternativo con el nombre del estado. |
| **MetricCard** | Cifra grande + etiqueta + procedencia. `value` nulo ⇒ "No disponible", nunca 0. |
| **DataRow** | Fila etiqueta / valor / delta para contadores SMART y listas de hechos. El delta usa color solo cuando significa algo. |
| **CapacityBar** | Ocupación de un volumen. El color lo decide `capacityState()`, no el llamante. |
| **ProgressBar** | Progreso de una operación en curso (benchmark, chkdsk, autotest). Siempre con texto de estado y restante. |
| **HealthDonut** | Anillo de estado global: un segmento por estado (ok → warn → crit → unknown). Siempre con leyenda numérica; el anillo solo no es accesible. |
| **TimeSeriesChart** | Serie temporal. El eje X es tiempo real, no índice de muestra; las series tienen huecos y no son equiespaciadas. |

## Compuestos de pantalla

| Componente | Qué es |
|---|---|
| **DiskCard** | Tarjeta de disco del panel general: alias sobre modelo, estado, tres métricas y capacidad del volumen principal. Un disco sin SMART se rotula "Sin datos SMART" en gris. |
| **AlertCard** | Grupo de alertas en la lista: severidad, título, resumen humano, contador de ocurrencias. La clave de deduplicación nunca aparece aquí. |
| **EventRow** | Fila del registro de eventos de Windows. El nivel es una píldora; la asociación inferida se etiqueta explícitamente, nunca como certeza. |
| **Card** | Tarjeta de material: cristal translúcido + filo de 1 px + sombra suave, radio 18. Bloques internos con `rounded-inner` (13) para radios concéntricos. No se apilan materiales. |
| **ConfirmDialog** | Confirmación obligatoria antes de cualquier operación que escriba datos o genere carga. Material sobre velo desenfocado; declara qué hará, dónde, el impacto y el comando literal. |
| **CodeOutput** | Salida literal de un proceso auxiliar (JSON de smartctl, salida de chkdsk). Siempre texto, nunca HTML. Incluye copia al portapapeles y procedencia. |
| **Toast** | Aviso efímero dentro de la app (prueba terminada, exportación guardada). No sustituye al centro de alertas ni a la notificación nativa. Nunca para una alerta de salud. |
| **EmptyState** | Estado vacío o no disponible. Distingue "sin datos todavía", "no compatible" y "error de fuente". No compatible NO es un fallo. |
| **VirtualList** | Lista virtualizada (miles de eventos). Renderiza solo lo visible + margen; la barra de desplazamiento es la del total. |
