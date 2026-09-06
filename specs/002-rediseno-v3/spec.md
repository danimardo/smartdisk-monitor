# Feature Specification: Rediseño visual «SmartDisk Monitor v3»

**Feature Branch**: `002-rediseno-v3`

**Created**: 2026-09-06

**Status**: Draft

**Input**: Entregable del diseñador en `design/propuesta-redisenov2/` (segunda entrega, que responde a la revisión previa). Fuentes normativas de la entrega: `RESUMEN.md`, `RESPUESTAS-A-LA-REVISION.md`, `cambios/00-tokens.md`, `cambios/00b-tipografia-y-fuentes.md`, `cambios/01`…`09`, `cambios/08b-perfiles-de-alerta.md`, las diez fichas de `cambios/componentes/`, el mockup navegable `mockups/smartdisk-v3.html` y el sprite `mockups/icons-sprite.svg`.

## Contexto y motivación

El diseñador ha revisado la interfaz v2 contra capturas reales (recogidas en `design/entregable-rediseno/salida/capturas/`) y ha detectado seis problemas de presentación:

1. El panel general parece a medio cargar: con dos discos, la rejilla ocupa ~230 px y deja ~570 px de lienzo vacío.
2. Las magnitudes son texto plano del mismo tamaño y color; nada indica si «41 °C» está bien ni cuánto margen queda, y un dato ausente («No disponible» a 20 px) pesa más que un dato presente.
3. El acento azul heredado de Windows es indistinguible de cualquier utilidad del sistema; en oscuro el conjunto queda gris plano.
4. La cifra principal de una pantalla no tiene presencia suficiente, y el peso tipográfico está topado en 600.
5. La barra lateral de 250 px se lleva el 24 % del ancho a 1024 px y repite información que ya está en el panel.
6. El asistente inicial (US-002, prioridad P0 del producto) sigue sin implementar: hoy es un `EmptyState` «no implementado».

Además, las capturas revelan tres defectos de implementación que el rediseño no arregla por sí solo pero que conviene corregir en la misma tanda:

- El título de la barra de herramientas dice «Panel general» en todas las pantallas (no recibe el título de la ruta).
- El estado global se calcula mal: aparece «Sin discos monitorizados» con un disco monitorizado, y la píldora de la barra de herramientas contradice al pie de la barra lateral.
- La gráfica de temperatura del detalle de disco sale vacía: dibuja el marco y dos puntos en los extremos, pero no hay trazo.

Esta feature aplica el entregable v3 completo. **Es un cambio de presentación**: no añade comandos Tauri, no añade permisos y no cambia el modelo de datos del almacenamiento de métricas. Los cambios que sí tocan la frontera con el backend (una preferencia de apariencia cuyo campo ya existe, una marca de asistente completado, y —según se decida— los umbrales de perfil de alerta) se acotan explícitamente en Requisitos y Supuestos.

## Alineación con los documentos normativos

- **`docs/ui-design.md`** pasa a describir el sistema v3. Sus §0, §2, §2.bis y §3 se actualizan; §4 (composición), §6 (accesibilidad) y §8 (definición de terminado) se mantienen y **siguen siendo el criterio de aceptación visual** de cada pantalla.
- **ADR-013** («Sistema de diseño v2 vinculante») y **ADR-017** («El acento heredado se corrige antes de aplicarse») quedan enmendados por sendos ADR nuevos: el salto a v3 con paleta propia, y la herencia del acento de Windows como opción apagada de fábrica. La corrección de contraste de `accessibleAccent()` / `accentOnSurface()` se conserva íntegra para cuando el usuario active esa opción.
- **`docs/user-stories.md` US-002** (asistente inicial) se cubre por primera vez con esta feature.
- **`docs/ui-contract.md`** se actualiza con las claves nuevas de `settings.alerts` (perfiles de alerta, alcance completo — ver US10), la clave `settings.onboarding.completedAt` y el cambio de valor de fábrica de `settings.appearance.useSystemAccent`.
- **`docs/alert-rules.md`** se actualiza: los perfiles de alerta parametrizan reglas existentes (`smart.wear_high`, `temp.above_configured_warn/crit`, `capacity.*`, `smart.media_errors`/`smart.error_log`, `events.controller_reset`/`events.io_retry`) haciendo configurables sus umbrales; **no se añade ninguna mecánica de conteo nueva** (ver Clarificaciones Q1). El umbral térmico de fábrica baja a 60/70 °C (Q2). Requiere su ADR (motor de alertas parametrizado por perfil).
- **`docs/data-model.md`** recoge las claves nuevas de `settings.alerts` y sus límites, y el campo `isSystemVolume` de `VolumeSummary` (Q3).
- **`docs/open-questions.md`** recoge las mediciones y las decisiones adoptadas: ratios de contraste de la paleta Ciruela, valores por defecto y límites de los umbrales de perfil, el descenso del umbral térmico de fábrica, y que la carga de las sparklines del panel es perezosa por tarjeta (decisión adoptada).

## Clarifications

### Session 2026-09-06

- Q: Semántica de los umbrales «errores de medios» y «reintentos del controlador» del perfil (que `08b` define por ventana de 24 h, pero el motor cuenta por ciclo o por hora) → A: Reinterpretarlos sobre las reglas que ya existen — `mediaErrors*` como umbral sobre la magnitud del incremento de `media_errors_total` por ciclo; `driverRetry*` como la `N` (hoy fija) de `events.controller_reset` / `events.io_retry`, ahora configurable. Se renombran en la interfaz sin «/24 h». No se construye conteo por ventana de 24 h.
- Q: ¿Se adopta el descenso del umbral térmico de aviso de 70/80 °C a 60/70 °C que trae el perfil «Equilibrado»? → A: Sí. Equilibrado = valor de fábrica = 60/70 °C. Se actualiza `docs/alert-rules.md` §2 y sus pruebas, y se registra en `docs/open-questions.md`.
- Q: ¿Cómo sabe `selectHeroDisk()` cuál es el disco de sistema, si hoy ningún dato del backend lo indica? → A: El backend añade `isSystemVolume: boolean` a `VolumeSummary` (Zod + ts-rs + prueba). Ningún cálculo ni inferencia en el frontend.
- Q: ¿Cómo se decide que un usuario que actualiza «ya está configurado» para no mostrarle el asistente? → A: Comprobación en `+layout.ts` al arrancar — si `settings.onboarding.completedAt` es nulo pero ya existe cualquier ajuste guardado, alias o disco excluido, se graba `completedAt = ahora` sin mostrar el asistente. Sin cambio de backend.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - El sistema de diseño v3 queda aplicado en toda la interfaz (Priority: P1)

Como usuario quiero una interfaz con identidad propia, coherente y accesible en tema claro y oscuro, para reconocer la aplicación de un vistazo y leer el estado del almacenamiento sin ambigüedad.

**Why this priority**: Es la base de la que dependen todas las demás historias. Sin los tokens nuevos, ningún componente ni pantalla puede componerse según el entregable. Entrega valor por sí sola: aunque no se recompusiera ninguna pantalla, la aplicación ya se vería con la paleta Ciruela, la tipografía de display y el contraste corregido.

**Independent Test**: Arrancar la aplicación tras aplicar los tokens y recorrer las ocho pantallas en tema claro y oscuro; comprobar que ningún texto sobre el acento queda por debajo de 4,5:1, que `pnpm verify:tokens` sigue en verde (una sola copia del sistema de diseño, cero literales) y que `pnpm check` no añade avisos.

**Acceptance Scenarios**:

1. **Given** la aplicación en tema oscuro, **When** se muestra cualquier botón primario, **Then** su texto usa `--sdm-on-accent` (tinta, no blanco) y el contraste contra el acento es ≥ 4,5:1.
2. **Given** el tema claro, **When** se compara la interfaz con el mockup `smartdisk-v3.html`, **Then** los neutros, el acento morado y el rojo crítico bermellón coinciden con `cambios/00-tokens.md`.
3. **Given** una pantalla cualquiera, **When** se ejecuta `pnpm verify`, **Then** no aparece ningún color, radio, sombra ni tamaño de fuente literal fuera de `tokens.css`.
4. **Given** una cifra de display (≥ 22 px), **When** se renderiza, **Then** usa la clase `.sdm-display` con cifras tabulares y el `letter-spacing` del token, sin superar el peso 600.

---

### User Story 2 - Cada magnitud y cada estado llevan icono (Priority: P1)

Como usuario quiero que cada temperatura, desgaste, actividad, hora, nivel de evento y tipo de bus lleve un símbolo reconocible, para entrar en una fila de datos sin leerla entera.

**Why this priority**: El juego de iconos es una dependencia dura de `MetricCard`, `DiskCard`, `StatusPill`, `EventRow`, `Sidebar`, `Toolbar`, `HeroPanel` y las tarjetas de prueba. Se puede probar y entregar de forma aislada montando el sprite en el catálogo.

**Independent Test**: Abrir `mockups/icons-hoja-de-contacto.html` y verificar que los 15 iconos se renderizan a 32/24/16/12 px en los dos temas heredando `currentColor`; comprobar que el componente `Icon` exige `label` cuando el icono es el único portador de significado y queda `aria-hidden` cuando acompaña a texto.

**Acceptance Scenarios**:

1. **Given** el sprite montado una sola vez en `AppShell`, **When** un componente usa `<Icon name="temp" />` sin `label`, **Then** el `<svg>` queda `aria-hidden="true"`.
2. **Given** un `<Icon name="alert" label="Advertencia" />`, **When** lo anuncia un lector de pantalla, **Then** lo lee como imagen con nombre «Advertencia».
3. **Given** un disco USB sin SMART, **When** se pide su icono de bus por `busIcon(deviceType)`, **Then** devuelve `usb`.
4. **Given** un estado de salud `warn`, **When** se resuelve `healthIcon[state]`, **Then** devuelve `alert`, y `crit` devuelve `bolt`.

---

### User Story 3 - El chrome deja de contradecirse y devuelve espacio al contenido (Priority: P1)

Como usuario quiero que la barra lateral no se coma un cuarto de la ventana, que el título de la barra de herramientas diga en qué pantalla estoy, y que el estado global sea el mismo en todos los sitios donde aparece.

**Why this priority**: Corrige dos de los tres defectos visibles en las capturas y libera 176 px de ancho, que es el recurso escaso a 1024 px. Es independiente: se puede montar el riel y la barra de herramientas nuevos sin tocar el contenido de ninguna pantalla.

**Independent Test**: A 1024 × 560, comprobar que la barra lateral ocupa 74 px fijos y el contenido dispone de 950 px; navegar entre pantallas y verificar que el título de la barra de herramientas cambia con la ruta; provocar un disco en advertencia y comprobar que la píldora de la barra de herramientas y el indicador del pie del riel dicen exactamente lo mismo.

**Acceptance Scenarios**:

1. **Given** la pantalla de detalle de disco, **When** se carga, **Then** el título de la barra de herramientas es el de esa ruta, no «Panel general».
2. **Given** un disco monitorizado en estado `ok` y ninguno en `warn`/`crit`, **When** se calcula el estado global, **Then** dice «Todo en orden», nunca «Sin discos monitorizados».
3. **Given** el riel de 74 px, **When** el puntero se posa sobre un icono de sección, **Then** aparece el tooltip nativo de Windows con el nombre de la sección (`title`) y el enlace tiene `aria-label` equivalente.
4. **Given** la sección Alertas con 3 alertas sin revisar, **When** se anuncia su enlace, **Then** el `aria-label` es «Alertas, 3 sin revisar» y el punto de aviso no es el único portador del dato.
5. **Given** el riel, **When** se busca la lista de discos, el texto de estado global o el botón de pausar recopilación, **Then** ya no están en la barra lateral (la pausa vive en Ajustes y en el menú de la bandeja).

---

### User Story 4 - La gráfica de temperatura vuelve a comunicar (Priority: P1)

Como usuario quiero ver la curva de temperatura de un disco con su eje, su umbral y sus huecos, para saber a qué altura está el dato y si lo que veo es nuevo o lleva así todo el día.

**Why this priority**: Corrige el tercer defecto de las capturas (el elemento más grande de la pantalla de detalle no comunica nada) y aporta el primitivo `Sparkline` del que dependen `HeroPanel`, `MetricCard` y `DiskCard`. Es entregable de forma aislada dentro de la pantalla de detalle.

**Independent Test**: Abrir el detalle de un disco con una serie que tenga un hueco en medio; comprobar que se dibuja el trazo (con `vector-effect="non-scaling-stroke"`), el eje Y con cuatro marcas, el relleno degradado bajo la curva, la banda gris sobre el hueco con su leyenda «sin datos HH:MM – HH:MM», y que nunca se interpola sobre el hueco.

**Acceptance Scenarios**:

1. **Given** una serie de temperatura con dos tramos continuos separados por `null`, **When** se dibuja, **Then** hay una `polyline` por tramo y el hueco se pinta como banda `--sdm-unknown` al 12 %.
2. **Given** el contenedor de la gráfica estirado a cualquier ancho, **When** se renderiza el trazo, **Then** el grosor se mantiene constante (2,6 px) y no desaparece.
3. **Given** un umbral del fabricante conocido, **When** se dibuja la gráfica, **Then** aparece como línea discontinua `--sdm-warn` con su leyenda propia.
4. **Given** una serie sin muestras en el intervalo, **When** se abre la gráfica, **Then** se mantiene el marco con el eje Y y el mensaje «Sin muestras en las últimas 24 h» sobre `bg-glass-3`, sin dibujar una línea a cero.
5. **Given** un `Sparkline` de 22 px dentro de una `MetricCard`, **When** se renderiza, **Then** no lleva eje ni etiqueta y la serie vacía deja el hueco en blanco.

---

### User Story 5 - El panel responde «¿tengo un problema?» sin leer (Priority: P2)

Como usuario quiero que la pantalla principal me diga de un vistazo si hay algún disco que necesita atención, sin tener que comparar cifras entre tarjetas.

**Why this priority**: Es el cambio de mayor impacto percibido, pero depende de US1, US2 y US4. Entrega valor claro: la pantalla que más se abre deja de parecer a medio cargar.

**Independent Test**: Abrir el panel con cuatro discos (dos SSD, un HDD, un USB sin SMART) y uno de ellos en advertencia térmica; comprobar que el `HeroPanel` muestra ese disco con su curva de fondo, la cifra a 76 px, el umbral y dos acciones; que la rejilla llena no deja hueco; que el bloque «Reparto de estados» sustituye al anillo; y que el disco USB aparece en la rejilla en gris, nunca en el héroe ni contado como avería.

**Acceptance Scenarios**:

1. **Given** al menos un disco en `warn` o `crit`, **When** se abre el panel, **Then** el `HeroPanel` muestra el de mayor severidad (empate: el de ocurrencia más reciente) con las dos acciones «Ver la alerta» y «Abrir el disco».
2. **Given** ningún disco en `warn`/`crit`, **When** se abre el panel, **Then** el héroe muestra el disco de sistema en tono neutro, la píldora «Todo en orden», la cifra en `--sdm-text` (no en verde) y solo la acción «Abrir el disco».
3. **Given** que la curva de temperatura cruza la zona de texto del héroe, **When** se lee el texto, **Then** el velo de legibilidad garantiza ≥ 4,5:1 independientemente de la forma de la serie.
4. **Given** cero discos, **When** se abre el panel, **Then** no se monta el héroe: se muestra `EmptyState kind="empty"` centrado.
5. **Given** un disco sin volúmenes montados en la rejilla, **When** se compone su tarjeta, **Then** la barra de capacidad se sustituye por «Sin volúmenes montados» con la misma altura, sin descuadrar la rejilla.
6. **Given** el disco USB sin SMART, **When** aparece en la rejilla, **Then** sus tres magnitudes se muestran como «—» a `text-xs` en `text-fg-dim` (texto completo en el `title`), su barra de capacidad sí se muestra, y nunca cuenta para el héroe ni para «necesitan atención».

---

### User Story 6 - El detalle de disco da contexto a cada cifra (Priority: P2)

Como usuario quiero que cada métrica del detalle muestre su icono, su evolución reciente y su procedencia, y que el intervalo de la gráfica esté junto a la gráfica.

**Why this priority**: Completa la lectura del disco individual. Depende de US1, US2 y US4.

**Independent Test**: Abrir el detalle de un disco compatible; comprobar que las cuatro `MetricCard` llevan icono, cifra `.sdm-display`, sparkline de 22 px y línea de procedencia; que el `SegmentedControl` de intervalo está junto a la gráfica y ya no en la barra de herramientas; que los contadores muestran delta solo cuando significa algo; y que un valor ausente se compone como «No disponible» a `text-lg` en gris, no como cifra.

**Acceptance Scenarios**:

1. **Given** una `MetricCard` con `value === null`, **When** se renderiza, **Then** muestra «No disponible» a `text-lg` en `fg-dim`, sin sparkline, con la procedencia explicando el motivo.
2. **Given** la cabecera de identidad, **When** se compone, **Then** incluye alias a 26 px `.sdm-display`, píldora de estado con icono, línea de identidad (modelo · bus · serie enmascarada · firmware · frescura), el `SegmentedControl` de intervalo y el botón «Probar disco».
3. **Given** un contador SMART con delta `+3` en el registro de errores, **When** se muestra, **Then** el delta va en `text-warn`; un `+0` o `+1` va en `text-fg-faint`.
4. **Given** el peso `font-black` (800) que hoy usa `MetricCard`, **When** se aplica el rediseño, **Then** desaparece: la cifra usa `.sdm-display` (600).

---

### User Story 7 - Pruebas, alertas, eventos, informes y ajustes hablan el nuevo lenguaje (Priority: P2)

Como usuario quiero que las pantallas secundarias sean coherentes con el resto: que la prueba en curso tenga la cifra de progreso con presencia, que la severidad de una alerta o un evento se barra por su icono, y que Ajustes separe visualmente lo destructivo.

**Why this priority**: Cierra la coherencia del conjunto. Salvo Pruebas, son cambios de tokens más estructura interna de componentes; bajo riesgo.

**Independent Test**: Recorrer las cinco pantallas en los dos temas; en Pruebas, lanzar una prueba y comprobar que la cifra de progreso está a 58 px (`--sdm-text-display`) y la barra a 12 px con relleno degradado; en Alertas y Eventos, comprobar que la severidad/el nivel lleva icono con `aria-label` y que el color nunca viaja solo; en Ajustes, comprobar que «Borrado de datos» queda separada con `space-8` y borde `border-crit` (fondo sin teñir).

**Acceptance Scenarios**:

1. **Given** una prueba en curso, **When** se muestra su bloque, **Then** aparece arriba y a ancho completo con `ProgressBar emphasis="display"` (12 px, degradado, filo interior) y la cifra de progreso a 58 px la pone la pantalla.
2. **Given** que no hay ninguna prueba en curso, **When** se abre Pruebas, **Then** el bloque de prueba en curso no se muestra y las tarjetas suben; no se deja hueco.
3. **Given** un evento de nivel `error`, **When** se muestra su fila, **Then** el nivel es un cuadrado de 26 px con `#i-bolt`, `aria-label` con el nombre del nivel, y la altura de fila sigue siendo 42 px (la `VirtualList` no recalcula).
4. **Given** una `AlertCard` de severidad `crit`, **When** se muestra su píldora, **Then** lleva `#i-bolt` y el contador `×N` en `.sdm-num`.
5. **Given** el `Switch` activo en tema oscuro, **When** se pinta su punto, **Then** sigue siendo blanco (el punto no es texto; `--sdm-on-accent` no aplica ahí).

---

### User Story 8 - Asistente inicial funcional (Priority: P1)

Como usuario nuevo quiero configurar la aplicación con un asistente de cuatro pasos para empezar a monitorizar sin conocer SMART, con una salida clara en cada paso.

**Why this priority**: Cubre US-002 del producto (P0), hoy sin implementar. Depende de US1 y US2 para el lenguaje visual, pero es una historia de producto por derecho propio, no solo presentación.

**Independent Test**: Con `settings.onboarding.completedAt` nulo, arrancar la aplicación y comprobar que redirige al asistente; recorrer los cuatro pasos (Bienvenida · Discos · Alertas · Listo); en el paso 2, excluir un disco y ponerle alias; pulsar «Omitir y usar los valores de fábrica» en el paso 3 y comprobar que se aplican los valores del perfil recomendado y se marca `completedAt`; reiniciar y comprobar que el asistente ya no aparece; relanzarlo desde Ajustes y comprobar que abre con los valores actuales sin borrar datos.

**Acceptance Scenarios**:

1. **Given** el primer arranque (`completedAt` nulo), **When** carga la aplicación, **Then** se muestra el asistente sin riel ni barra de herramientas, con cabecera propia de 56 px e indicador de paso.
2. **Given** cualquier paso, **When** se mira la cabecera, **Then** «Omitir y usar los valores de fábrica» está visible.
3. **Given** el paso 2, **When** se detectan los discos, **Then** cada fila lleva casilla de 24 px, icono de bus, modelo, campo de alias, ocupación del volumen principal con barra y píldora de compatibilidad SMART; los compatibles están marcados de inicio.
4. **Given** el disco USB en el paso 2, **When** se muestra, **Then** el bloque explicativo dice «no es una avería» en negrita y no se pinta en rojo.
5. **Given** que se pulsa «Omitir» en el paso 3, **When** se procesa, **Then** se aplican los valores del perfil recomendado, se graba `completedAt` y se navega al panel.
6. **Given** que no se detecta ningún disco, **When** se llega al paso 2, **Then** se muestra `EmptyState kind="empty"` con «Volver a buscar» y la opción de continuar igualmente.
7. **Given** que falla la detección, **When** se llega al paso 2, **Then** se muestra `EmptyState kind="error"` con frase humana, `<details>` técnico, «Reintentar» y «Omitir».
8. **Given** un asistente completado, **When** el usuario elige en Ajustes «Repetir la configuración inicial», **Then** el asistente se reabre con los valores actuales y no se borra ningún dato.

---

### User Story 10 - Perfiles de alerta configurables (Priority: P2)

Como usuario quiero elegir «cuánto me avisa» la aplicación con un perfil en vez de doce números, y poder afinar cada umbral después en Ajustes.

**Why this priority**: Da sentido al paso 3 del asistente y es la única parte de esta feature que toca el motor de alertas. Es independientemente testable: los perfiles y los umbrales se pueden probar desde Ajustes sin pasar por el asistente.

**Independent Test**: En Ajustes → Alertas, elegir el perfil «Prudente» y comprobar que los doce umbrales se escriben con los valores de `cambios/08b-perfiles-de-alerta.md` y que `settings.alerts.profile` pasa a `cautious`; cambiar un umbral a mano y comprobar que el perfil pasa a `custom` y la interfaz muestra «Personalizado (a partir de Prudente)»; provocar una condición que supere un umbral nuevo (p. ej. reintentos de controlador por encima del límite del perfil) y comprobar que el motor de alertas genera el grupo correspondiente; comprobar que un disco que declara límite térmico del fabricante avisa al mínimo de (límite del fabricante, valor del perfil).

**Acceptance Scenarios**:

1. **Given** el perfil «Equilibrado», **When** se elige «Solo lo grave», **Then** los doce valores de `settings.alerts` se reescriben y `settings.alerts.profile` pasa a `quiet`.
2. **Given** un perfil aplicado, **When** el usuario edita un umbral a mano en Ajustes, **Then** `settings.alerts.profile` pasa a `custom` y la interfaz muestra «Personalizado (a partir de \<perfil anterior\>)».
3. **Given** un disco que declara límite térmico de 70 °C y el perfil «Solo lo grave» (que avisaría a 70 °C), **When** se evalúa la temperatura, **Then** la advertencia salta al mínimo de los dos, y la regla `temp_above_vendor_limit` sigue activa.
4. **Given** un valor de umbral fuera de su rango permitido, **When** se intenta guardar, **Then** el backend lo rechaza con un `AppError` y el control vuelve a su valor anterior con el error junto al control.
5. **Given** un disco sin SMART, **When** se evalúan los perfiles, **Then** no participa de temperatura, desgaste ni errores de medios; sí de capacidad y eventos; nunca cuenta como avería.
6. **Given** el espacio libre de un volumen, **When** se evalúa, **Then** gana el criterio que salte primero entre porcentaje y valor absoluto, igual que hoy.

---

### User Story 9 - El usuario puede volver al acento de Windows (Priority: P3)

Como usuario que prefiere que la aplicación se integre con mi sistema quiero un interruptor que devuelva el color de acento de Windows.

**Why this priority**: Preserva la posibilidad que ADR-013 valoraba, ahora como opción. Bajo impacto: el campo de persistencia ya existe en el backend.

**Independent Test**: En Ajustes → Apariencia, activar «Usar el color de acento de Windows» y comprobar que el acento morado se sustituye por el color del sistema (corregido a AA por `accessibleAccent()`/`accentOnSurface()`); desactivarlo y comprobar que vuelve el morado; reiniciar y comprobar que la preferencia persiste; comprobar que de fábrica está apagado.

**Acceptance Scenarios**:

1. **Given** una instalación nueva, **When** se leen los ajustes de apariencia, **Then** `useSystemAccent` es `false`.
2. **Given** el interruptor apagado, **When** se pinta la interfaz, **Then** el acento es el morado Ciruela del tema activo.
3. **Given** el interruptor encendido con un acento de Windows claro (p. ej. amarillo `#ffb900`), **When** se pinta el botón primario y el texto de acento, **Then** ambos cumplen 4,5:1 en los dos temas y el cambio de tema recalcula `--sdm-accent-fg`.
4. **Given** el interruptor, **When** se cambia su estado, **Then** solo se sobrescriben (o restauran) los tres tokens de acento, sin tocar el resto de la paleta.

---

### Edge Cases

- **Escalado de Windows** (125 %, 150 %, 200 %): a 1024 × 560 CSS efectivos, ninguna pantalla recorta contenido en silencio; la región de contenido hace scroll. El riel se mantiene en 74 px fijos (no colapsa más).
- **Muchos discos** (20+): la rejilla del panel hace scroll; el `HeroPanel` elige un solo protagonista; el cálculo de estado global y el héroe no bloquean la interfaz por encima del presupuesto de 50 ms.
- **Todos los discos sin SMART**: el `HeroPanel` muestra el único disco con cifra «No disponible» en gris y explica que el bus no expone SMART; el estado global no lo cuenta como avería.
- **Dato obsoleto**: la frescura pasa a `text-warn`; las series terminan donde terminan los datos, sin extenderse ni interpolarse; el héroe añade «último dato válido a las HH:MM».
- **Recopilación en pausa**: el estado global dice «En pausa» con `#i-clock`; el botón «Actualizar» de la barra de herramientas sigue disponible.
- **Fallo de una fuente**: degrada la tarjeta o el bloque afectado a `EmptyState kind="error"` local, nunca la aplicación entera.
- **Motor sin `backdrop-filter`**: el material cae a `--sdm-solid` opaco con el mismo layout (comportamiento actual, no cambia).
- **`prefers-reduced-motion`**: los esqueletos de carga usan solo opacidad, sin animación de brillo; no se añade ninguna animación decorativa nueva.
- **Fuente de display no disponible**: con la opción elegida (sin segunda familia) no aplica; `.sdm-display` usa la familia sans ya empaquetada.

## Requirements *(mandatory)*

### Functional Requirements

#### Tokens y sistema de diseño

- **FR-001**: El sistema MUST incorporar la paleta «Ciruela» de `cambios/00-tokens.md` en `tokens.css`, con valor claro y oscuro para cada token, y su mapeo en `tailwind.config.cjs` y `tokens.json`.
- **FR-002**: El sistema MUST redefinir `--sdm-on-accent` a un valor de tinta en tema oscuro y MUST corregir todo punto del catálogo que hoy escriba texto blanco sobre el acento para que use `--sdm-on-accent` (`text-fg-onAccent`), salvo las excepciones documentadas (filo interior de `ProgressBar` en modo display y punto del `Switch` activo, que son brillos, no texto).
- **FR-003**: El sistema MUST añadir los tokens `--sdm-font-display`, `--sdm-text-display` (58 px), `--sdm-text-hero` (76 px), `--sdm-tracking-display`, `--sdm-rail-width` (74 px), `--sdm-hero-height` (246 px), `--sdm-icon-stroke` (1,7) y `--sdm-icon-size` (24 px), y la utilidad `.sdm-display` (familia + peso 600 + cifras tabulares + tracking + `line-height: 1`).
- **FR-004**: `--sdm-font-display` MUST resolver a la familia sans ya empaquetada (opción sin segunda familia tipográfica); `.sdm-display` es el único punto de cambio si en el futuro se decide empaquetar Bricolage Grotesque.
- **FR-005**: El sistema MUST mantener sin cambios el material de tres capas y sus desenfoques, la escala de radios concéntricos, la escala de espaciado y las curvas y duraciones de movimiento.
- **FR-006**: `pnpm verify` (assets, tokens, i18n, boundaries) MUST seguir en verde: una sola copia del sistema de diseño y cero valores visuales literales en `src/lib/components` y `src/routes`.
- **FR-007**: El sistema MUST actualizar `docs/ui-design.md` para que describa v3 (paleta, tipografía de display, iconografía, riel), conservando §4, §6 y §8 como criterio vinculante.

#### Iconografía

- **FR-008**: El sistema MUST montar el sprite de `mockups/icons-sprite.svg` una sola vez en `AppShell` (sin el bloque `<metadata>` de procedencia), con los 15 símbolos heredando `currentColor` y sin ningún literal de color.
- **FR-009**: El sistema MUST ofrecer un componente `Icon` en el catálogo con `name` obligatorio, `size` opcional (16 px por defecto) y `label` opcional; con `label` emite `role="img"` + `aria-label`, sin `label` queda `aria-hidden="true"`.
- **FR-010**: El sistema MUST centralizar los mapas semánticos (`healthIcon`, `eventLevelIcon`, `busIcon`, `testIcon`) en `$lib/design/icons.ts`, no repartidos por los componentes.
- **FR-011**: Cuando un icono sea el único portador de un significado, `label` MUST ser obligatorio; cuando acompañe a texto que ya lo dice, MUST ir sin `label`.

#### Componentes nuevos y recompuestos

- **FR-012**: El sistema MUST ofrecer `Sparkline` en el catálogo: trazo por tramos continuos (un `polyline` por tramo, nunca interpola sobre `null`), `vector-effect="non-scaling-stroke"`, eje X por tiempo real, rango con 8 % de margen, `id` de degradado único por instancia, y submuestreo por encima de ~400 puntos conservando mínimos y máximos por columna.
- **FR-013**: `TimeSeriesChart` MUST usar `Sparkline` para el trazo y añadir eje Y de 26 px con cuatro marcas, relleno degradado bajo la curva, banda `--sdm-unknown` al 12 % sobre cada hueco con su leyenda, umbral del fabricante como discontinua con leyenda propia, y pie con inicio/hueco/fin en hora local. La gráfica MUST volver a dibujar el trazo (corrección del defecto actual).
- **FR-014**: El sistema MUST ofrecer `HeroPanel` en el catálogo: dato dominante con la serie de fondo a sangre, velo de legibilidad horizontal entre el SVG y el texto, cifra `.sdm-display` a `--sdm-text-hero`, umbral, banda de hueco, cuatro hechos a la derecha y hasta dos acciones; todos los callbacks opcionales. La elección del disco protagonista MUST vivir en `selectHeroDisk()` junto a `worstState()` en `$lib/design/health.ts`, no en el componente. Cuando no hay ningún disco en `warn`/`crit`, `selectHeroDisk()` MUST elegir el disco cuyo `VolumeSummary.isSystemVolume` sea `true`; a falta de él, el primero del inventario.
- **FR-014a**: El backend MUST añadir `isSystemVolume: boolean` a `VolumeSummary` (esquema Zod + tipo ts-rs + prueba de rechazo). Ningún componente ni módulo de presentación MUST inferir el disco de sistema por su cuenta.
- **FR-015**: `DiskCard` MUST recomponerse con cabecera de 52 px que hereda el color del estado, sparkline de temperatura de fondo (cuando reciba serie; sin ella, cabecera plana), cuadrado de icono de bus, píldora de estado, alias `.sdm-display`, tres magnitudes con icono y cifra de display, y `CapacityBar` sin cambios de API. Un dato ausente MUST componerse como «—» a `text-xs` en `text-fg-dim` con el texto completo en `title`.
- **FR-016**: `MetricCard` MUST ganar ranura de icono obligatoria y sparkline opcional, y MUST eliminar el peso 800 (`font-black` → `.sdm-display`). Con `value === null` MUST componer «No disponible» a `text-lg` en `fg-dim`, sin sparkline, con procedencia obligatoria.
- **FR-017**: `StatusPill` MUST ganar una ranura de icono opcional (`icon`, con `"auto"` resolviendo por `healthIcon[state]`), manteniendo `label` obligatorio, el icono `aria-hidden`, y `icon`/`withDot` como excluyentes.
- **FR-018**: `ProgressBar` MUST ganar la prop `emphasis` (`"inline"` por defecto, `"display"` para la prueba en curso: 12 px, relleno degradado `accent → accent-hi`, filo interior), manteniendo `role="progressbar"`, el modo indeterminado y la regla de leyenda + tiempo restante siempre.
- **FR-019**: `Button` variante `primary` MUST escribir `text-fg-onAccent`, no `text-white`, sin más cambios de variantes, tamaños, radios ni movimiento.
- **FR-020**: Ningún componente del catálogo MUST eliminarse. `HealthDonut` se conserva en el catálogo aunque salga del panel general.

#### Chrome y estado global

- **FR-021**: `Sidebar` MUST pasar a un riel fijo de `--sdm-rail-width` (74 px): logotipo, seis secciones + «Acerca de» como botones de 44 px con `title` y `aria-label`, punto de aviso sobre Alertas (con el recuento también en el `aria-label`), y el indicador de estado global al pie como cuadrado con icono y contador. MUST desaparecer la lista de discos, el texto de estado global y el botón de pausa.
- **FR-022**: El botón de pausar/reanudar recopilación MUST quedar accesible desde Ajustes y desde el menú de la bandeja; no se pierde funcionalidad.
- **FR-023**: `Toolbar` MUST recibir `title` y `subtitle` de la ruta y mostrarlos (corrección del defecto «Panel general» fijo); MUST subir a 56 px; MUST mostrar la píldora de estado global con icono como única fuente de ese dato; MUST perder el botón «?» (Acerca de pasa al riel) y la ranura de controles contextuales.
- **FR-024**: El estado global MUST calcularse una sola vez con `worstState()` sobre los discos **monitorizados**, con los discos sin SMART sin contar como avería, y presentarse en dos sitios (píldora de la barra de herramientas y pie del riel) sin posibilidad de contradicción. Las tres formas son «Todo en orden» (`ok`), «N necesita/n atención» (`warn`/`crit`, el peor) y «En pausa» (`unknown`).
- **FR-025**: El intervalo del detalle de disco (`SegmentedControl` + `DateRangePicker` en modo personalizado) MUST vivir en el contenido de esa pantalla, junto a la gráfica que modifica, no en la barra de herramientas.

#### Pantallas

- **FR-026**: El panel general MUST componerse como `HeroPanel` (alto `--sdm-hero-height`) + rejilla `repeat(auto-fill, minmax(272px, 1fr))` de `DiskCard` + fila inferior con «Sucesos del sistema» y «Reparto de estados»; el lienzo MUST llevar el degradado `--sdm-bg → --sdm-bg-2`.
- **FR-027**: El bloque «Reparto de estados» del panel MUST ser composición de pantalla (`Card` + `Icon` + barra de proporción), no un componente nuevo, y sustituye al anillo en el panel porque con 2–4 discos un anillo de cuatro segmentos es ilegible.
- **FR-028**: El detalle de disco MUST componerse como cabecera de identidad + cuatro `MetricCard` en fila + rejilla `1.6fr 1fr` con `TimeSeriesChart` y contadores; los contadores (`DataRow`) MUST ganar columna de delta con color solo cuando significa algo.
- **FR-029**: Alertas, Eventos, Informes y Ajustes MUST recomponerse solo con tokens y con la estructura interna de `AlertCard` y `EventRow` (píldora/nivel con icono); su distribución no cambia. En Ajustes, «Borrado de datos» MUST separarse con `space-8` y borde `border-crit` (sin teñir el fondo).
- **FR-030**: Pruebas MUST componerse como bloque de prueba en curso (cuando exista) a ancho completo con la cifra de progreso a 58 px + tres tarjetas de prueba con icono + historial con columna de icono de estado.
- **FR-031**: Todas las pantallas MUST cubrir los cinco estados de `docs/ui-design.md` (cargando con esqueletos de la misma geometría, vacío, no compatible en gris, error de fuente local, dato obsoleto) y MUST pasar la definición de terminado de §8, incluida la verificación a 1024 × 560 y a 1280 × 720 en los dos temas.

#### Asistente inicial

- **FR-032**: El sistema MUST mostrar el asistente inicial en la ruta `/onboarding` mientras `settings.onboarding.completedAt` sea nulo, redirigiendo a él desde el arranque, y MUST no volver a mostrarlo una vez grabada la marca.
- **FR-033**: El asistente MUST tener cuatro pasos (Bienvenida · Discos · Alertas · Listo), uno por pantalla, sin riel ni barra de herramientas, con cabecera propia de 56 px, indicador de paso y la salida «Omitir y usar los valores de fábrica» visible en los cuatro.
- **FR-034**: El paso 2 MUST permitir excluir discos y asignar alias, mostrar la ocupación del volumen principal y la píldora de compatibilidad SMART, marcar de inicio los discos compatibles, y explicar el disco USB sin SMART como «no es una avería» sin pintarlo en rojo.
- **FR-035**: El paso 3 MUST permitir elegir uno de los tres perfiles de alerta de `cambios/08b-perfiles-de-alerta.md` (Prudente / Equilibrado —preseleccionado— / Solo lo grave), presentados como tres tarjetas seleccionables con un `<details>` que muestra la tabla de umbrales filtrada al perfil elegido y reacciona al cambio; MUST incluir además los dos `Switch` (notificación nativa de Windows, arrancar con el sistema). Elegir un perfil aplica su juego completo de umbrales (ver FR-045).
- **FR-036**: «Omitir» en cualquier paso MUST aplicar el perfil **Equilibrado**, grabar `settings.onboarding.completedAt` y navegar al panel.
- **FR-037**: El asistente MUST ser relanzable desde Ajustes → «Repetir la configuración inicial» sin borrar datos: reabre con los valores actuales.
- **FR-038**: El asistente MUST cubrir sus estados: cargando (detección en curso), vacío (ningún disco detectado, con opción de continuar), no compatible (el USB, en el cuerpo del paso), error de fuente (fallo de detección, con «Reintentar» y «Omitir»).

#### Perfiles de alerta (esquema y motor)

- **FR-045**: El sistema MUST añadir a `settings.alerts` las claves de umbral que hoy no existen y que los perfiles necesitan: `wearWarnPercent` / `wearCritPercent`; `mediaErrorsWarnPer24h` / `mediaErrorsCritPer24h` (nombre histórico; su semántica real es umbral sobre la magnitud del incremento de `media_errors_total` por ciclo, ver Q1 — el rótulo de la interfaz no dice «/24 h»); `driverRetryWarnPer24h` / `driverRetryCritPer24h` (nombre histórico; su semántica real es la `N` configurable de las reglas `events.controller_reset` / `events.io_retry`); más `settings.alerts.profile` con valores `cautious` | `balanced` | `quiet` | `custom`. El espacio libre por porcentaje y por valor absoluto ya tiene claves (`capacity*`), que pasan a leerse de `settings`. Cada clave nueva con su valor de fábrica (el del perfil Equilibrado, con temperatura 60/70 °C — Q2), su rango permitido y su validación Zod + serde estricta, con prueba de rechazo.
- **FR-046**: Elegir un perfil MUST escribir los umbrales correspondientes **y** `settings.alerts.profile` con su identificador. Editar a mano cualquiera de los umbrales de perfil en Ajustes MUST cambiar `settings.alerts.profile` a `custom`, y la interfaz MUST mostrar «Personalizado (a partir de \<perfil anterior\>)» (el «perfil anterior» se deriva en la UI del último `profile` no-`custom`, no se persiste un segundo campo).
- **FR-047**: El motor de alertas MUST consumir los umbrales configurables como parámetros de las reglas **que ya existen** (`smart.wear_high`, `temp.above_configured_warn/crit`, `capacity.low/critical`, `smart.media_errors`/`smart.error_log`, `events.controller_reset`/`events.io_retry`), con el mismo criterio de operadores que `docs/alert-rules.md`. NO se añade ninguna mecánica de conteo por ventana de 24 h ni ninguna regla nueva (Q1).
- **FR-048**: El límite del fabricante MUST mandar sobre el perfil: si el disco declara su límite térmico, la advertencia salta al mínimo de (límite del fabricante, valor del perfil), y la regla `temp_above_vendor_limit` no se desactiva por el perfil.
- **FR-049**: Los discos sin SMART MUST no participar de las reglas de temperatura, desgaste y errores de medios en ningún perfil; sí de capacidad y de eventos del sistema; nunca cuentan como avería.
- **FR-050**: El espacio libre MUST evaluarse tomando el criterio que salte primero entre porcentaje y valor absoluto, sin cambios en `capacityState()` salvo leer los umbrales de `settings.alerts` en vez de constantes.
- **FR-051**: `docs/alert-rules.md`, `docs/ui-contract.md` y `docs/data-model.md` MUST actualizarse con las reglas, claves y límites nuevos; la decisión (motor de alertas parametrizado por perfil) MUST registrarse como ADR en `docs/decisions.md`.
- **FR-052**: Este bloque toca `src-tauri/` (esquema de `settings`, validación, motor de alertas) y `docs/` normativa; su implementación MUST ir precedida de modo plan y MUST no añadir ningún comando Tauri nuevo ni ningún permiso nuevo (se persiste con el `set_setting` genérico existente).

#### Textos y preferencias

- **FR-039**: El sistema MUST añadir a `es.json` y `en.json` las claves nuevas listadas en `RESUMEN.md` («Cambios de texto visible») y las del asistente, con paridad de claves y de interpolaciones (`pnpm verify:i18n` en verde). Ningún literal de interfaz fuera de los diccionarios, incluidos `aria-label`, `title` y `alt`.
- **FR-040**: El sistema MUST invertir el valor de fábrica de `settings.appearance.useSystemAccent` a `false` y actualizar su rótulo y su `hint` para reflejar que la opción sustituye el morado de la aplicación por el acento de Windows.
- **FR-041**: Activar `useSystemAccent` MUST sobrescribir exactamente los tres tokens de acento mediante `applySystemAccent()`; desactivarlo MUST restaurarlos mediante `clearSystemAccent()`. La corrección de contraste (`accessibleAccent()`, `accentOnSurface()`, `refreshAccentForTheme()`) se conserva.

#### Frontera con el backend (acotada)

- **FR-042**: El sistema MUST dar de alta la clave `settings.onboarding.completedAt` en la lista blanca de `set_setting` del backend y en `docs/ui-contract.md`, sin añadir ningún comando Tauri nuevo ni ningún permiso nuevo.
- **FR-043**: En el arranque, si `settings.onboarding.completedAt` es nulo **pero** ya existe cualquier ajuste guardado por el usuario, algún alias de disco o algún disco excluido, el sistema MUST grabar `completedAt = ahora` y no mostrar el asistente. La comprobación vive en `+layout.ts`; no requiere migración de backend. Una instalación v3 desde cero interrumpida antes de guardar nada vuelve a mostrar el asistente (correcto: aún no hay nada configurado).
- **FR-044**: Toda serie temporal que el panel necesite (temperatura de 24 h por disco para `HeroPanel` y `DiskCard`) MUST obtenerse por la API existente (`getMetricSeries`), sin comando nuevo, con **carga perezosa por tarjeta tras el primer render**: el panel pinta de inmediato con las cabeceras y las cifras del inventario, y cada sparkline aparece cuando llega su serie. El patrón MUST respetar el presupuesto de 50 ms de SC-007 con la lista virtualizada y hasta 20 discos, y MUST no disparar peticiones para tarjetas fuera de la ventana visible. La medición se registra en `docs/open-questions.md`.

### Key Entities *(include if data involved)*

- **Token de diseño**: variable CSS `--sdm-*` con valor en tema claro y en tema oscuro; fuente única de verdad visual en `tokens.css`, espejada en `tailwind.config.cjs` y `tokens.json`.
- **Icono**: símbolo de línea identificado por `name`, `viewBox` 24×24, sin color propio; vive en el sprite y se referencia por `<use>`.
- **Serie temporal de métrica**: lista de puntos `{ t: ms epoch, v: number | null }`; un `v` nulo es un hueco que nunca se interpola. Ya existe en el modelo de datos; no cambia.
- **Volumen** (`VolumeSummary`): gana `isSystemVolume: boolean` — lo calcula el backend, la presentación no lo infiere. Único cambio de forma en el contrato de inventario.
- **Perfil de alerta**: `settings.alerts.profile` con valor `cautious` | `balanced` | `quiet` | `custom`; y el juego de umbrales de `settings.alerts` que un perfil escribe de golpe (temperatura 60/70 en Equilibrado, desgaste, capacidad, y los dos umbrales de nombre histórico `mediaErrors*`/`driverRetry*` con la semántica de Q1). `custom` es el estado tras editar un umbral a mano. Valores en `cambios/08b-perfiles-de-alerta.md` y en `docs/data-model.md`.
- **Marca de asistente completado**: `settings.onboarding.completedAt`, fecha ISO UTC o nula; nula = mostrar el asistente, salvo que ya exista otra configuración guardada (FR-043).
- **Preferencia de acento del sistema**: `settings.appearance.useSystemAccent`, booleano; el campo ya existe, cambia su valor de fábrica a `false`.
- **Disco protagonista del héroe**: resultado de `selectHeroDisk()` sobre los discos monitorizados; con todo en orden, el que contiene el volumen `isSystemVolume`. No se persiste, se deriva en cada render.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: En el panel general con cuatro discos y uno en advertencia, un usuario identifica qué disco necesita atención en **menos de 3 segundos** sin interactuar, frente a tener que comparar cuatro tarjetas hoy.
- **SC-002**: A 1024 px de ancho, la región de contenido dispone de **950 px** (frente a 774 px hoy), y ninguna de las ocho pantallas ni el asistente recortan contenido en silencio a 1024 × 560 en los dos temas.
- **SC-003**: **El 100 %** de los textos sobre el acento y de los textos de acento sobre el material cumplen contraste AA (≥ 4,5:1) en tema claro y oscuro, con la paleta Ciruela y con un acento de Windows claro de prueba.
- **SC-004**: La gráfica de temperatura del detalle de disco dibuja el trazo, el eje y los huecos en **el 100 %** de las series con datos, incluidas las que tienen uno o más huecos (hoy: 0 %).
- **SC-005**: `pnpm verify`, `pnpm check`, `pnpm test`, `pnpm test:component`, `pnpm test:e2e` y `pnpm test:a11y` terminan **sin errores y sin avisos nuevos** tras cada fase.
- **SC-006**: El panel general se pinta y responde al desplazamiento **sin bloqueo perceptible** (tareas por debajo de 50 ms, SC-007 del producto) con 20 discos monitorizados y sus sparklines.
- **SC-007**: Un usuario nuevo completa el asistente inicial (o lo omite) en **menos de 2 minutos** y termina con la configuración grabada, sin poder quedarse en un estado a medias.
- **SC-008**: Cada pantalla entregada pasa **todas** las casillas de la definición de terminado de `docs/ui-design.md` §8.
- **SC-009**: Cero literales de interfaz fuera de `es.json` / `en.json` y cero valores visuales literales fuera de `tokens.css`, verificado automáticamente.
- **SC-010**: El número de componentes del catálogo aumenta en **exactamente 3** (`Icon`, `Sparkline`, `HeroPanel`) y ninguno se elimina.
- **SC-011**: Un usuario elige un perfil de alerta en **una sola interacción** y los doce umbrales quedan aplicados de forma coherente; editar un umbral a mano refleja «Personalizado» **de inmediato**, sin que la interfaz muestre nunca un perfil que no corresponde con los valores reales.
- **SC-012**: Cada clave nueva de `settings.alerts` tiene su prueba de rechazo (valor fuera de rango, tipo incorrecto) y el motor de alertas genera el grupo correcto en **el 100 %** de los casos de prueba de superación de umbral.

## Assumptions

- **Dirección aprobada**: el usuario ha aprobado la dirección «escena de datos» con paleta Ciruela y el salto a v3; los dos ADR que enmiendan ADR-013 y ADR-017 se redactan dentro de esta feature (fase de plan), no se vuelven a someter a decisión.
- **Sin segunda familia tipográfica** (opción «b» elegida por el usuario): `--sdm-font-display` resuelve a la familia sans ya empaquetada; no se descargan ni empaquetan binarios de Bricolage Grotesque, no se toca `THIRD_PARTY_NOTICES.md` por tipografía y no hace falta un ADR de fuente. La observación del entregable sobre que Instrument Sans se carga por `@import` de Google Fonts está **desactualizada**: el proyecto ya la empaqueta localmente (ADR-018).
- **Sin comandos ni permisos Tauri nuevos**: no cambia el esquema de `metric_samples` ni el de eventos, y todo se persiste con el `set_setting` genérico existente. Sí hay cambios de frontera acotados: la clave `settings.onboarding.completedAt` y las claves nuevas de `settings.alerts` para los perfiles (FR-045), estas últimas con impacto en el motor de alertas de Rust y en `docs/alert-rules.md`.
- **Modo plan para el bloque de backend**: la parte de perfiles de alerta (US10, FR-045–FR-052) toca `src-tauri/` y documentación normativa; su fase de implementación se aborda en modo plan y con su propio ADR, y puede entregarse como uno o dos PR separados del resto del rediseño.
- **Perfiles de alerta = alcance completo** (decisión del usuario): se añaden los ~10 umbrales configurables, `settings.alerts.profile`, la lógica «perfil → custom» y el consumo en el motor. El asistente y Ajustes comparten el mismo mecanismo.
- **Carga perezosa de sparklines** (decisión del usuario): el panel no espera a las series en el `load`; las pide por tarjeta visible tras el primer render.
- **`worstState()` y `deviceState()` ya existen** en `$lib/design/health.ts` con la semántica correcta (reconocer no cambia el color, sin datos frescos es `unknown`, no compatible no cuenta como avería); se reutilizan, se añade `selectHeroDisk()`.
- **`getMetricSeries` cubre la necesidad de series** del panel y del detalle sin comando nuevo.
- **El mockup `smartdisk-v3.html` y el sprite son la referencia visual**; la fuente de verdad para implantar tokens es la tabla de `cambios/00-tokens.md`, no los valores inline del mockup.
- **Los ratios de contraste del entregable están medidos** sobre el material compuesto; se re-verifican al implantar y se registran en `docs/open-questions.md`. El diseñador ha señalado que dos valores del tema oscuro sobre bloque interno quedaban justos: se comprueban explícitamente.
- **Alcance por fases**: la implementación se entrega en PR independientes siguiendo el orden de prioridad de las historias (US1 → US2 → US3 → US4 → US10 → US5 → US6 → US7 → US8 → US9). US10 (perfiles de alerta) se coloca antes de US8 porque el asistente depende de que el mecanismo de perfiles exista. Cada PR deja la aplicación en verde y pasa la definición de terminado de las pantallas que toca.
- **Ventana mínima y objetivo de diseño** siguen siendo 1024 × 560 y 1280 × 720; no cambian.
- **`design/propuesta-rediseno/`** (primera entrega) queda obsoleta frente a `design/propuesta-redisenov2/`; su borrado o archivado no es parte de esta feature.
