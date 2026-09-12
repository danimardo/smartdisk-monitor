# Feature Specification: Riel de navegación expandible con etiquetas de texto

**Feature Branch**: `[013-sidebar-expandible]`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "Añadir un botón en la Sidebar (riel de navegación de 74px) que alterne un panel expandido de navegación con icono + etiqueta de texto visible para cada sección, en vez de depender del tooltip al pasar el ratón. El panel expandido (~232px) se superpone al contenido con sombra (no lo empuja ni cambia su tamaño), para no comprometer el mínimo de ventana de 1024px ya medido y protegido por ADR-034. Se cierra al volver a pulsar el botón, con Escape, o al pulsar fuera del panel; el foco queda atrapado dentro mientras está abierto, mismo patrón que los diálogos existentes (ConfirmDialog/ExplicacionModal/AboutDialog). El estado expandido/plegado se recuerda entre sesiones como una preferencia más en `settings` (igual que tema o idioma), no en localStorage. Retoma la idea ya esbozada en design/propuesta-rediseno/cambios/componentes/Sidebar.md (\"Alternativa: riel expansible... superpuesto sobre el contenido, no empujándolo\"), pero con un botón explícito en vez de solo hover/foco, y persistiendo el estado."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Ver el nombre de cada sección sin adivinar por el icono (Priority: P1) 🎯 MVP

Hoy el riel de navegación solo tiene iconos; el nombre de cada sección solo aparece como *tooltip*
nativo al dejar el ratón quieto encima. Una persona que no recuerda todavía qué icono es cada
sección, o que navega con teclado (donde el tooltip nativo tarda o no aparece igual), quiere una
forma de ver el nombre de todas las secciones a la vez, con un gesto explícito.

**Por qué esta prioridad**: es el problema que se pidió resolver; sin esto no hay funcionalidad
que entregar.

**Prueba independiente**: pulsar el botón de expandir en el riel y comprobar que aparece un panel
con el icono y el nombre de cada sección, sin necesidad de pasar el ratón por ninguno.

**Escenarios de aceptación**:

1. **Given** el riel está plegado (solo iconos), **When** la persona pulsa el botón de expandir,
   **Then** aparece un panel más ancho con el icono y el nombre de texto de cada sección, incluida
   la de «Acerca de» y el indicador de estado global.
2. **Given** el panel expandido está abierto, **When** la persona pulsa una sección, **Then**
   navega a esa sección (la navegación funciona igual que hoy: es un enlace real, no un
   `onclick` con redirección programática) **y** el panel se pliega a la vez, sin tapar la
   pantalla de destino (corrección post-validación: en la primera versión se quedaba abierto).
3. **Given** el panel expandido está abierto, **When** la persona vuelve a pulsar el botón de
   expandir, **Then** el panel se cierra y el riel vuelve a mostrar solo iconos.

---

### User Story 2 - El panel expandido no reduce el espacio de la pantalla (Priority: P1)

Como el mínimo técnico de la ventana (1024 × 560, `ADR-034`) ya cuenta con que el riel de
navegación mide 74 px, un panel expandido que **redujera** el contenido pondría en riesgo pantallas
que ya están medidas contra ese mínimo. El panel expandido debe convivir con el contenido sin
encogerlo.

**Por qué esta prioridad**: es una condición no negociable para no reabrir el problema que
`ADR-034` cerró; sin esto, la funcionalidad tendría un coste oculto en el peor caso de ventana.

**Prueba independiente**: con la ventana en su tamaño mínimo (1024 × 560), expandir el riel y
comprobar que ninguna pantalla cambia de estrechez ni aparece un recorte nuevo; el panel se dibuja
por encima del contenido, no lo desplaza.

**Escenarios de aceptación**:

1. **Given** la ventana está en su tamaño mínimo (1024 × 560), **When** la persona expande el
   riel, **Then** el ancho del contenido de la pantalla no cambia; el panel expandido se dibuja
   superpuesto, con su propia sombra de elevación.
2. **Given** el panel expandido está abierto y superpuesto, **When** la persona mira el contenido
   que queda debajo, **Then** lo que el panel tapa es exactamente su propia franja (la del riel
   ampliado), nunca más.

---

### User Story 3 - Cerrar el panel expandido sin usar el ratón (Priority: P2)

Como el panel se superpone al contenido, alguien que navega con teclado necesita poder cerrarlo sin
tener que pulsar fuera con el ratón, y el foco no debe quedar «perdido» en un elemento que ya no se
ve.

**Por qué esta prioridad**: es una condición de accesibilidad (constitución §VII), pero el valor
central de la User Story 1 ya existe sin esto — es un complemento que la completa, no una historia
independiente de mostrar las etiquetas.

**Prueba independiente**: abrir el panel con teclado, comprobar que el foco entra en él, pulsar
`Escape` y comprobar que se cierra y el foco vuelve al botón que lo abrió.

**Escenarios de aceptación**:

1. **Given** el panel expandido está abierto, **When** la persona pulsa `Escape`, **Then** el
   panel se cierra y el foco vuelve al botón que lo abrió.
2. **Given** el panel expandido está abierto, **When** la persona pulsa en cualquier punto del
   contenido fuera del panel, **Then** el panel se cierra.
3. **Given** el panel expandido se acaba de abrir, **When** la persona empieza a moverse con
   `Tab`, **Then** el foco recorre solo los elementos del panel (foco atrapado), igual que ya
   hacen los diálogos existentes de la aplicación.

---

### User Story 4 - La preferencia se recuerda entre sesiones (Priority: P3)

Quien prefiere trabajar con el riel siempre expandido no quiere tener que volver a pulsar el botón
cada vez que abre la aplicación.

**Por qué esta prioridad**: mejora la comodidad de uso habitual, pero el valor de las historias
1-3 ya existe sin esto — cada sesión simplemente empezaría plegada.

**Prueba independiente**: expandir el riel, cerrar la aplicación, volver a abrirla y comprobar que
sigue expandido.

**Escenarios de aceptación**:

1. **Given** la persona dejó el riel expandido, **When** cierra y vuelve a abrir la aplicación,
   **Then** el riel aparece expandido desde el primer momento, sin tener que volver a pulsar el
   botón.
2. **Given** la persona dejó el riel plegado (el estado de fábrica), **When** abre la aplicación,
   **Then** el riel aparece plegado.

---

### Edge Cases

- **La ventana se redimensiona mientras el panel expandido está abierto** (incluida una
  reducción hasta el mínimo técnico): el panel sigue siendo una superposición, así que el
  contenido nunca queda más estrecho por su culpa; el propio panel no debe salirse de la ventana
  ni quedar cortado.
- **Cambiar de pantalla con el panel abierto** (pulsar una sección): la navegación ocurre con
  normalidad y el panel se pliega a la vez — decisión revisada tras la validación manual: dejarlo
  abierto tapaba la pantalla de destino hasta un segundo gesto (ver User Story 1, escenario 2). En
  cualquier caso el foco nunca debe quedar en un elemento que haya dejado de existir.
- **Tema claro y oscuro, y escalado de Windows (125 %, 150 %)**: el panel expandido debe verse
  correctamente en ambos temas y no romperse con el escalado, igual que el resto de la interfaz.
- **Sin JavaScript de terceros ni comportamiento oculto**: el nombre de cada sección en el panel
  expandido es el mismo texto que ya lleva el `title`/`aria-label` de hoy, no uno nuevo.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El riel de navegación DEBE incluir un control (botón) que alterne entre su estado
  plegado (solo iconos, como hoy) y un estado expandido.
- **FR-002**: En estado expandido, cada sección de navegación DEBE mostrar su icono y el nombre de
  la sección como texto visible, sin depender de pasar el ratón por encima.
- **FR-003**: El estado expandido DEBE mostrarse como una superposición sobre el contenido de la
  pantalla (con su propia elevación/sombra), sin reducir ni desplazar el ancho del contenido en
  ningún tamaño de ventana, incluido el mínimo técnico (1024 × 560).
- **FR-004**: La persona DEBE poder cerrar el estado expandido de al menos estas tres formas:
  volviendo a pulsar el control que lo abrió, pulsando `Escape`, y pulsando fuera del panel.
- **FR-005**: Mientras el panel expandido está abierto, el foco del teclado DEBE quedar contenido
  dentro de él (foco atrapado), y al cerrarse DEBE volver al control que lo abrió.
- **FR-006**: Cada sección del panel expandido DEBE seguir siendo un enlace real de navegación
  (no un controlador de clic que redirija de forma programática), igual que exige la arquitectura
  ya vigente de la aplicación.
- **FR-007**: El estado (expandido o plegado) DEBE persistir entre cierres y aperturas de la
  aplicación, igual que ya persisten el tema y el idioma.
- **FR-008**: El estado de fábrica (primera instalación, sin preferencia guardada) DEBE ser
  plegado, igual que el riel de hoy.
- **FR-009**: El nombre de cada sección en el panel expandido DEBE salir de los mismos textos ya
  traducidos que usan hoy el `title`/`aria-label` de cada icono (ambos idiomas de la aplicación).
- **FR-010**: El panel expandido DEBE cumplir el mismo nivel de accesibilidad que el resto de la
  interfaz: contraste AA, tamaño de objetivo mínimo, y anuncio correcto por lector de pantalla de
  su apertura y cierre.

### Key Entities

- **Preferencia de navegación** (nueva): un valor persistido que dice si el riel se abre expandido
  o plegado. Vive junto al resto de preferencias de la persona (tema, idioma), no en el navegador.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Una persona que no recuerda el significado de un icono del riel puede ver el nombre
  de todas las secciones a la vez, sin tener que pasar el ratón uno por uno, en una sola
  interacción (pulsar un botón).
- **SC-002**: Expandir el riel no cambia el ancho disponible de ninguna pantalla, verificado en la
  ventana en su tamaño mínimo (1024 × 560): ninguna pantalla que hoy cabe sin recortes deja de
  caber por culpa de esta funcionalidad.
- **SC-003**: Una persona que navega solo con teclado puede abrir el panel, leer las secciones y
  cerrarlo sin usar el ratón en ningún momento.
- **SC-004**: Una persona que prefiere el riel expandido no tiene que repetir la acción de
  expandirlo en cada apertura de la aplicación.

## Assumptions

- El panel expandido reutiliza el mismo patrón de foco atrapado / cierre con `Escape` / cierre al
  pulsar fuera que ya usan los diálogos existentes de la aplicación (`ConfirmDialog`,
  `ExplicacionModal`, `AboutDialog`), en vez de inventar un mecanismo nuevo.
- El ancho aproximado del panel expandido (232 px) y el tratamiento visual (superposición con
  sombra de elevación) parten de la propuesta de diseño ya esbozada en
  `design/propuesta-rediseno/cambios/componentes/Sidebar.md`; el valor exacto puede afinarse en la
  fase de diseño de detalle sin cambiar el criterio de aceptación (que no empuje el contenido).
- Qué ocurre con el panel expandido al navegar a otra sección (si se cierra automáticamente o
  permanece abierto en la pantalla siguiente) es una decisión de diseño de interacción que se
  resuelve en la fase de planificación técnica; ninguna de las dos opciones cambia el valor de
  esta funcionalidad para la persona.
- La preferencia se guarda como un campo nuevo en `settings`, siguiendo el mismo mecanismo que ya
  usan el tema y el idioma (persistencia en SQLite, nunca en `localStorage`, principio V de la
  constitución).
- Fuera de alcance: cambiar el contenido de las etiquetas de sección respecto a lo que ya dicen
  `title`/`aria-label` hoy; añadir o quitar secciones del riel; cualquier cambio a la lista de
  discos o al botón de pausa (ya se sacaron del riel en ADR-034 y no vuelven).
