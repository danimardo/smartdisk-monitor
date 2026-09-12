# Fase 0 — Investigación: Riel de navegación expandible

No queda ningún `NEEDS CLARIFICATION`: el comportamiento superpuesto y la persistencia entre
sesiones ya se acordaron con el usuario antes de escribir la spec. Este documento recoge las
decisiones técnicas para implementarlo reutilizando patrones ya existentes en el repositorio.

## D1 · El panel se superpone dentro del propio `AppShell`, con un hueco fijo que evita el salto

**Decisión**: `Sidebar.svelte` renderiza su `<aside>` de siempre (74 px, en flujo normal) cuando
está plegado. Al expandirse, ese `<aside>` pasa a `position: absolute` (232 px, `inset-y-0 left-0`,
`z-40`) **dentro** del contenedor `relative` que ya define `AppShell.svelte`, y se añade un
`<div>` invisible de 74 px en el flujo para que el resto de `AppShell` (barra de herramientas +
contenido) no se mueva ni un píxel al expandir o plegar.

**Razón**: `position: absolute` dentro de un ancestro `relative` (que `AppShell` ya es) posiciona
el panel sin tener que calcular a mano la altura de la barra de título propia (spec 011); con
`position: fixed` habría que restar esa altura explícitamente. Es la superficie de cambio mínima:
`AppShell.svelte` no se toca en absoluto.

**Alternativas consideradas**: cambiar la rejilla de `AppShell` a un `grid-template-columns`
dinámico según el estado. Descartada: obliga a que `AppShell` conozca el estado de expansión de la
`Sidebar` (rompe la separación de responsabilidades: hoy `AppShell` no sabe nada de navegación,
solo compone tres huecos) y es exactamente lo que `ADR-034` quería evitar (que el ancho del
contenido dependa del estado del riel).

## D2 · Reutiliza `sdm-material-overlay` y `--sdm-shadow-lift`, cero tokens nuevos

**Decisión**: el panel expandido lleva la clase `sdm-material-overlay` (ya la usan
`ConfirmDialog`/`ExplicacionModal`/`AboutDialog`/`Tooltip`) y `shadow-lift` para la elevación, tal
como ya apuntaba `design/propuesta-rediseno/cambios/componentes/Sidebar.md`.

**Razón**: es literalmente el material "elevado sobre el resto" que el sistema de diseño ya define
para este propósito exacto; no hace falta un cuarto nivel de material.

## D3 · Foco y cierre: mismo patrón que los diálogos, sin velo que oscurezca

**Decisión**: al abrir, un `$effect` mueve el foco al panel (`panel.focus()`, `tabindex="-1"`),
igual que `ConfirmDialog`. Un `<div class="fixed inset-0 z-30" role="presentation">` invisible
(sin `background`, a diferencia del velo de un diálogo) captura el clic fuera y el `Escape`
mientras el panel está abierto. Al cerrarse, el foco vuelve al botón que abrió el panel
(`bind:this` sobre el propio botón, guardado antes de abrir).

**Razón**: el patrón de foco/`Escape`/clic-fuera de los diálogos existentes ya está resuelto y
probado (`docs/known-issues.md` #2); no hay que inventar un mecanismo de foco nuevo. La diferencia
frente a un diálogo es que este panel **no bloquea** el resto de la aplicación (no es una
confirmación ni una ventana modal): por eso el velo no lleva `background`/`backdrop-blur`, solo
sirve para detectar el clic fuera.

**Nota sobre "foco atrapado"**: en este proyecto ningún diálogo existente implementa un ciclo de
`Tab` que impida salir del panel (no hay una utilidad de *focus trap* real, ver búsqueda en el
código) — "foco atrapado" en la práctica de esta base de código significa **foco inicial dentro +
cierre con `Escape`**, no un ciclo de `Tab` bloqueado. Este panel sigue exactamente esa misma
convención, no una más estricta.

## D4 · `AppearanceSettings` gana `sidebarExpanded`, mismo mecanismo que `useSystemAccent`

**Decisión**: nuevo campo `sidebar_expanded: bool` en el `struct AppearanceSettings` de
`commands/mod.rs`, leído con `leer_ajuste_bool(conn, "settings.appearance.sidebar_expanded",
false)` — exactamente el mismo patrón que `use_system_accent`. `+layout.svelte` inicializa su
`expanded = $state(...)` desde `appearance.sidebarExpanded` en el mismo punto donde ya inicializa
tema e idioma, y el manejador que alterna el panel llama a
`setSetting("settings.appearance.sidebar_expanded", nuevoValor)`.

**Razón**: no hay que inventar un mecanismo de persistencia nuevo; `settings` ya es una tabla
genérica clave/valor, así que una clave más no exige migración.

**Alternativas consideradas**: un comando dedicado (`set_sidebar_expanded`). Descartado: sería un
comando nuevo — más superficie en un binario privilegiado — para algo que el mecanismo genérico ya
resuelve.

## D5 · El contenido de la etiqueta reutiliza `nav.*`, cero copy nuevo salvo el propio botón

**Decisión**: cada fila del panel expandido muestra el mismo texto que ya usa el `aria-label`/
`title` de cada icono hoy (`t("nav.dashboard")`, etc. — ya vive en los dos diccionarios). Solo hace
falta i18n nuevo para el propio botón de alternar: `nav.sidebar.expand` / `nav.sidebar.collapse`
(nombre accesible según el estado, con `aria-expanded`).

**Razón**: cumple FR-009 (mismo texto que ya existe) y evita duplicar claves de sección.

## D6 · Qué pasa al navegar con el panel abierto: se pliega (revisado tras la validación manual)

**Decisión original** (fase de planificación): el panel se quedaba abierto tras navegar. **Revisada
el 2026-09-12** tras probarlo de verdad: el usuario reportó que tapaba la franja derecha del
contenido hasta que hacía un segundo gesto (pulsar el botón, `Escape` o fuera) para quitarlo de en
medio — molesto en el uso real, aunque parecía razonable sobre el papel.

**Decisión final**: al pulsar una sección (o «Acerca de») dentro del panel expandido, la navegación
ocurre con normalidad (enlace real, sin interceptar el clic) y el panel **se pliega** a la vez,
mediante un `onclick` en cada fila que invoca un callback nuevo, `onSelect`.

**Consecuencia para D4 — no toda forma de plegar persiste igual**: hay ahora dos vías, con efecto
distinto sobre la preferencia guardada:

- **Explícita** (botón, `Escape`, clic fuera): la persona dice "ya no lo quiero expandido". Sigue
  llamando a `onToggleExpand`, que persiste con `setSetting`.
- **Por selección** (elegir una sección o «Acerca de»): es una consecuencia de navegar, no un
  cambio de preferencia. Llama a un callback distinto, `onSelect` (en `+layout.svelte`,
  `colapsarSidebarAlNavegar`), que solo pone `expandedSidebar = false` **sin** persistir. Si
  también persistiera, la preferencia "empezar expandido" se perdería en cuanto la persona
  pulsara el primer enlace de la sesión, contradiciendo FR-007 para cualquiera que
  habitualmente lo tenga expandido — el caso que motivó pedir la persistencia en primer lugar.

**Alternativa descartada** (la que sí se adoptó en la primera versión del análisis): dejarlo
abierto tras navegar. Sonaba razonable para "verlo siempre visible", pero en el uso real resultó
peor que un menú flotante clásico que se cierra al elegir, que es lo que se implementa ahora.

## Resumen

Ninguna decisión de esta fase introduce una dependencia, un comando, un permiso o una migración
nuevos. El único fichero de `src-tauri/` que cambia es `commands/mod.rs`, con un campo booleano más
en una respuesta ya existente.
