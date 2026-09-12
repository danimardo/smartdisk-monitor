# Feature Specification: Barra de título propia, integrada con el sistema de diseño

**Feature Branch**: `[011-barra-titulo-propia]`

**Created**: 2026-09-12

**Status**: Draft

**Input**: User description: "Sustituir la barra de título nativa de Windows por una barra propia, construida por la aplicación, del mismo color de fondo que el resto de la interfaz y con los controles de ventana (minimizar, maximizar, cerrar) usando el color de acento de la aplicación en vez de los colores por defecto del sistema. Aceptado explícitamente: se pierde el menú de Snap Layouts de Windows 11 y el snap nativo a bordes de pantalla."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Controlar la ventana desde la barra propia (Priority: P1) 🎯 MVP

Al quitar la barra de título del sistema, la persona necesita poder seguir haciendo exactamente lo
mismo que hacía con ella: mover la ventana arrastrándola, minimizarla, maximizarla o restaurarla, y
cerrarla. Sin esto la aplicación queda inutilizable (no habría forma de mover ni cerrar la
ventana).

**Por qué esta prioridad**: es la condición para que quitar la decoración nativa sea siquiera
viable. Sin paridad funcional, no hay app.

**Prueba independiente**: con la aplicación abierta, arrastrar la ventana por la barra propia y
comprobar que se mueve; pulsar minimizar, maximizar/restaurar y cerrar, y comprobar que cada uno
hace exactamente lo que hacía el control nativo equivalente.

**Escenarios de aceptación**:

1. **Given** la ventana está en su tamaño normal, **When** la persona arrastra la barra superior,
   **Then** la ventana se mueve con el puntero, igual que con la barra nativa.
2. **Given** la ventana está en su tamaño normal, **When** la persona pulsa el control de
   maximizar, **Then** la ventana ocupa toda la pantalla disponible y el control pasa a ofrecer
   "restaurar".
3. **Given** la ventana está maximizada, **When** la persona pulsa el control de restaurar,
   **Then** la ventana vuelve al tamaño y posición que tenía antes de maximizarse.
4. **Given** la ventana está en cualquier estado, **When** la persona pulsa minimizar, **Then** la
   ventana se oculta a la barra de tareas de Windows, igual que con el control nativo.
5. **Given** la ventana está en cualquier estado, **When** la persona pulsa cerrar, **Then** ocurre
   exactamente lo mismo que ocurría al pulsar la X nativa (incluida una posible minimización a la
   bandeja del sistema, si la aplicación está configurada así).
6. **Given** la ventana no está maximizada, **When** la persona arrastra desde cualquier borde,
   **Then** la ventana se redimensiona, igual que con la decoración nativa.

---

### User Story 2 - Barra integrada con el diseño de la aplicación (Priority: P2)

Una vez que la barra propia funciona, tiene que dejar de parecer "una barra de sistema pegada
encima" y pasar a leerse como parte de la aplicación: mismo color de fondo que el resto de la
ventana, sin franja diferenciada, y los tres controles con el acento propio de la aplicación en vez
del gris/azul por defecto de Windows.

**Por qué esta prioridad**: es el motivo por el que se pide el cambio, pero depende de que la
historia 1 ya dé paridad funcional — el valor visual no sirve de nada si los botones no funcionan.

**Prueba independiente**: comparar visualmente la barra superior con el resto de la ventana (deben
compartir fondo, sin línea ni franja que las separe) y confirmar que los tres controles usan el
mismo color de acento que, por ejemplo, el botón primario de un diálogo — en tema claro y en tema
oscuro, y también con el acento heredado de Windows activado.

**Escenarios de aceptación**:

1. **Given** la aplicación está abierta en tema claro, **When** se observa la barra superior,
   **Then** su color de fondo es el mismo que el del resto de la ventana, sin franja ni borde que
   la distinga como una zona de sistema aparte.
2. **Given** la aplicación está abierta en tema oscuro, **When** se observa la barra superior,
   **Then** ocurre lo mismo que en el escenario anterior, con los tonos del tema oscuro.
3. **Given** cualquier tema, **When** se observan los tres controles de ventana, **Then** usan el
   mismo color de acento que el resto de acciones primarias de la aplicación, no los colores por
   defecto de Windows.
4. **Given** la persona tiene activada la opción de heredar el acento de Windows, **When** cambia
   ese acento del sistema, **Then** los controles de ventana lo reflejan igual que el resto de la
   interfaz.

---

### User Story 3 - Paridad con gestos estándar de Windows que sí se conservan (Priority: P3)

Aunque se acepta perder Snap Layouts y el snap nativo a bordes, hay un gesto muy extendido en
Windows que cuesta poco conservar y que la persona esperará por costumbre: doble clic sobre la
barra para maximizar o restaurar la ventana.

**Por qué esta prioridad**: es una comodidad esperada, no una condición de uso — la historia 1 ya
cubre maximizar/restaurar con el botón.

**Prueba independiente**: hacer doble clic sobre una zona vacía de la barra (no sobre los
controles) y comprobar que maximiza si estaba restaurada, y restaura si estaba maximizada.

**Escenarios de aceptación**:

1. **Given** la ventana está en su tamaño normal, **When** la persona hace doble clic sobre una
   zona vacía de la barra, **Then** la ventana se maximiza.
2. **Given** la ventana está maximizada, **When** la persona hace doble clic sobre una zona vacía
   de la barra, **Then** la ventana se restaura a su tamaño anterior.

---

### Edge Cases

- Si la aplicación está configurada para minimizar a la bandeja del sistema en vez de cerrarse, el
  botón cerrar propio debe disparar ese mismo comportamiento — no uno nuevo ni distinto.
- El menú de Snap Layouts de Windows 11 (al pasar el ratón por el botón maximizar nativo) y el
  snap automático al arrastrar contra los bordes de pantalla **no** se reproducen: aceptado
  explícitamente como pérdida al quitar la decoración nativa.
- La zona de cambio de tamaño en las esquinas debe seguir siendo utilizable aunque ya no haya una
  barra de título nativa que delimite visualmente dónde empieza.
- La posición guardada de la ventana de una sesión anterior debe seguir restaurándose correctamente
  con la barra nueva (no debe quedar una ventana con parte fuera de pantalla ni mal posicionada).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: La ventana NO DEBE mostrar la decoración de título nativa de Windows.
- **FR-002**: El sistema DEBE mostrar una barra superior propia que sustituya funcionalmente a la
  nativa.
- **FR-003**: La persona DEBE poder mover la ventana arrastrándola desde la barra propia.
- **FR-004**: La barra propia DEBE ofrecer tres controles — minimizar, maximizar/restaurar y
  cerrar — con el mismo efecto que los controles nativos que sustituyen.
- **FR-005**: El control de maximizar/restaurar DEBE reflejar visualmente cuál de las dos acciones
  corresponde al estado actual de la ventana.
- **FR-006**: El botón cerrar DEBE disparar exactamente el mismo comportamiento de cierre que ya
  existe hoy (incluida una posible minimización a la bandeja del sistema si está configurada), sin
  introducir un camino de cierre nuevo o distinto.
- **FR-007**: La persona DEBE poder seguir redimensionando la ventana arrastrando sus bordes,
  igual que con la decoración nativa.
- **FR-008**: La barra propia DEBE compartir el color de fondo con el resto de la ventana, sin
  franja ni borde que la distinga como una zona de sistema aparte.
- **FR-009**: Los tres controles de ventana DEBEN usar el color de acento de la aplicación (el
  mismo que las demás acciones primarias, incluida la variante de acento heredado de Windows si
  está activa) en vez de los colores por defecto del sistema.
- **FR-010**: La barra y sus controles DEBEN mostrarse correctamente tanto en tema claro como en
  tema oscuro, sin condicionales de plataforma y sin recortes.
- **FR-011**: Los tres controles DEBEN tener un nombre accesible equivalente al de los controles
  nativos que sustituyen (minimizar/maximizar/restaurar/cerrar).
- **FR-012**: Un doble clic sobre una zona vacía de la barra (fuera de los controles) DEBE
  maximizar la ventana si está restaurada, y restaurarla si está maximizada.
- **FR-013**: La posición y el tamaño de ventana que ya se guardan entre sesiones DEBEN seguir
  restaurándose correctamente con la barra nueva, sin regresión sobre el comportamiento actual.
- **FR-014**: La ventana mínima admitida (1024×560) DEBE seguir mostrándose sin recortes con la
  barra nueva.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: El 100 % de las acciones que antes se hacían desde la barra nativa (mover, minimizar,
  maximizar, restaurar, cerrar, redimensionar) siguen siendo posibles desde la barra propia, sin
  recurrir a atajos de teclado como alternativa obligatoria.
- **SC-002**: En una comparación visual directa, la barra superior no se distingue del resto de la
  ventana como una franja de sistema aparte, ni en tema claro ni en tema oscuro.
- **SC-003**: Los tres controles de ventana usan el mismo color de acento que el resto de acciones
  primarias de la aplicación, verificable comparándolos con un botón primario existente.
- **SC-004**: La ventana mínima (1024×560) se sigue mostrando sin ningún elemento recortado tras el
  cambio.
- **SC-005**: La posición y el tamaño de la ventana de la sesión anterior se restauran
  correctamente el 100 % de las veces tras el cambio, igual que antes.

## Assumptions

- El doble clic para maximizar/restaurar (US3) se implementa por ser el comportamiento estándar de
  Windows y de otras aplicaciones con barra de título propia (Visual Studio Code, Windows
  Terminal); no se ha pedido explícitamente pero tiene un default razonable y bien establecido.
- La comprobación de que la ventana guardada cae dentro de un monitor visible
  (`aplicar_geometria_guardada` / `geometria_visible`) se revisa durante la planificación para
  confirmar que no depende de un concepto de "barra de título nativa" que vaya a dejar de existir;
  se asume que basta con referenciar el borde superior de la ventana, a falta de confirmarlo contra
  el código real.
- El presupuesto de píxeles medido para la ventana mínima (`docs/open-questions.md` §L, que resta
  32 px de barra de título nativa) se remide durante la planificación con la altura real de la
  barra propia.
- Se pierden a propósito, aceptado explícitamente por el responsable del producto: el menú de Snap
  Layouts de Windows 11 y el snap nativo a bordes de pantalla — ninguno de los dos se reproduce.
- Alcance solo Windows: la aplicación no se distribuye para macOS ni Linux (plataforma soportada ya
  fijada en `docs/decisions.md`/constitución), así que no aplican las variantes de barra de título
  de otros sistemas.
- No se toca el icono de la aplicación ni la bandeja del sistema — ya se resolvieron aparte.
