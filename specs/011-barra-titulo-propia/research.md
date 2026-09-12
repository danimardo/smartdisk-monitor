# Fase 0 — Investigación: barra de título propia

Todas las incógnitas de esta feature son de encaje con código real ya existente, no de tecnología
nueva: Tauri 2 ya trae todo lo necesario (`decorations`, `@tauri-apps/api/window`,
`data-tauri-drag-region`) sin dependencia nueva. Lo que hace falta es verificar, contra el código
real de este proyecto, los puntos que quedaron como `Assumptions` en la spec.

## D1 — `decorations: false` no rompe la comprobación de geometría guardada

**Verificado, sin cambios necesarios.** `geometria_visible` (`src-tauri/src/platform/ventana.rs`)
ya trata "la barra de título" como un concepto abstracto: `let barra = v.y + 20;` — un punto 20 px
por debajo de la esquina superior de la ventana (`v.y`), sin ninguna llamada a una API de
decoración nativa. `v.y` es la coordenada de la ventana en sí (`outer_position`), que existe igual
con o sin decoración nativa. Con una barra propia de altura razonable (≥ 20 px), ese punto sigue
cayendo dentro de la franja superior real. **No se toca `ventana.rs` para esto.**

## D2 — El botón cerrar propio no necesita replicar la lógica de "minimizar a la bandeja"

**Verificado, sin cambios necesarios.** La intercepción de cierre (`src-tauri/src/lib.rs`,
`ventana.on_window_event` sobre `WindowEvent::CloseRequested`) es un evento **de la ventana**, no
del botón nativo: se dispara igual llamando a `getCurrentWindow().close()` desde JS que pulsando la
X nativa. El botón cerrar propio solo tiene que llamar a `close()`; toda la lógica de
`lifecycle.close_action` (`gestionar_cierre`, `avisar_primera_minimizacion`) sigue funcionando sin
tocarla.

## D3 — Presupuesto de tamaño: la barra propia se añade, no resta

**Decisión.** `docs/open-questions.md` §L.2 mide la ventana mínima (1024×560, constante `MIN_W`/
`MIN_H` en `ventana.rs` y `minWidth`/`minHeight` en `tauri.conf.json`) como presupuesto de
**contenido**, con la barra de título nativa (32 px) como chrome añadido por Windows **fuera** de
ese presupuesto. Al quitar la decoración nativa, la barra propia pasa a pintarse **dentro** del área
que Tauri considera "la ventana" (ya no hay chrome del sistema que la lleve aparte). Para no volver
a medir todo el presupuesto de píxeles ya validado, la barra propia se trata como una adición: se
sube `minHeight`/`height` de `tauri.conf.json` y las constantes `MIN_W`/`MIN_H` de `ventana.rs` en
exactamente la altura de la barra, de modo que el área bajo la barra conserve los mismos 1024×560
que ya se midieron. Es una suma en un sitio (dos ficheros, mismo número), no una repetición de la
medición de §L.

**Altura de la barra**: se reutiliza un token de altura ya existente en `tokens.css`
(`--sdm-control-lg: 35px`, o el que se ajuste mejor al tamaño táctil de los tres controles) en vez
de un valor nuevo — cumple "cero valores visuales literales" (`AGENTS.md`). El número exacto en
píxeles que se sume a `MIN_H`/`height` en Rust debe **coincidir exactamente** con ese token: es un
punto de sincronización manual entre CSS y Rust (Rust no puede leer `tokens.css`), así que ambos
lados se anotan con un comentario cruzado el uno al otro para que un cambio futuro de altura no
desincronice el uno del otro.

## D4 — Dónde vive la barra en el árbol de componentes

**Decisión.** No puede vivir dentro de `AppShell.svelte`: `src/routes/+layout.svelte` tiene tres
ramas de pintado (`startupError`, `esOnboarding`, la normal con `AppShell`) y las tres necesitan
poder mover/cerrar la ventana — si la barra solo estuviera dentro de `AppShell`, el asistente
inicial y la pantalla de error de arranque se quedarían sin forma de cerrar la ventana. La barra se
monta una sola vez en `+layout.svelte`, al mismo nivel que `<IconSprite />` (justo el mismo motivo
por el que `IconSprite` ya vive ahí fuera de las ramas: "para que también resuelva en rutas sin
chrome").

**Consecuencia estructural**: hoy `AppShell.svelte` y las dos ramas sin `AppShell` usan `h-screen`
(100 % del alto de la ventana) cada una por su cuenta. Con la barra añadida como primera fila, hace
falta un contenedor nuevo en `+layout.svelte` que sea el `h-screen` (columna: barra de altura fija +
resto), y que `AppShell`/las dos ramas pasen de `h-screen` a `h-full` (ocupan el alto que les deja
el contenedor nuevo, no el 100 % de la ventana entera). Sin este cambio, el contenido quedaría
recortado por la altura de la barra o la barra quedaría tapada.

## D5 — Los controles de ventana no pasan por `$lib/api`

**Decisión.** `frontera-ipc.md` ("ninguna pantalla llama a `invoke` directamente") regula las
llamadas a **comandos propios** (`invoke("nuestro_comando", …)`, con esquema Zod y DTO generado).
`getCurrentWindow().minimize()/.toggleMaximize()/.close()` son métodos ya tipados del SDK oficial de
Tauri, no comandos nuestros: no hay esquema que mantener ni DTO que generar. Aun así, por
coherencia (nada de acceso a APIs de Tauri disperso por componentes de presentación) y para poder
sustituirlos en las pruebas de componente, las tres llamadas se centralizan en un módulo nuevo y
pequeño (`src/lib/window.ts`), que es lo único que importa `@tauri-apps/api/window` en todo el
frontend. El nuevo componente de la barra los recibe como *callbacks*, igual que hace
`ExplicacionModal` con `onreprocesar`/`oncancel`: sigue siendo presentacional.

## D6 — Icono nuevo en el catálogo, no SVG suelto

**Decisión.** `src/lib/design/icons.ts` ya tiene `"close"` pero no `"minimize"`/`"maximize"`/
`"restore"`. Se añaden esos tres glifos al sprite existente (mismo mecanismo que ya usa `"close"`),
no un SVG a mano dentro del componente nuevo — mantiene un solo sitio de verdad para iconos
(`AGENTS.md`: "ningún componente fuera del catálogo").

## D7 — Esquinas redondeadas de la ventana: verificar en vivo, no asumir

**Sin decisión cerrada, verificación pendiente en `quickstart.md`.** `AppShell.svelte` ya usa la
clase `rounded-window` (`--sdm-radius-window: 18px`), pero ese mismo token también lo usan diálogos
flotantes normales (`ConfirmDialog`, `AboutDialog`) — no es evidencia de que la ventana en sí sea
transparente a nivel de sistema operativo. `tauri.conf.json` no tiene `"transparent": true` hoy.
Windows 11 redondea las esquinas de las ventanas de nivel superior de forma automática a nivel de
compositor (DWM), típicamente independiente de si la app dibuja su propia decoración — pero es un
comportamiento del sistema operativo, no algo que este proyecto controle, y solo se puede confirmar
abriendo la aplicación de verdad tras el cambio. Si las esquinas quedan cuadradas y se ven mal
contra el `rounded-window` interior, la corrección (añadir `"transparent": true`) es una tarea de
seguimiento, no un bloqueante de este plan.

## D8 — Doble clic para maximizar (US3)

**Decisión.** Se implementa en el mismo `data-tauri-drag-region` de la barra: un `ondblclick` que
llama a `toggleMaximize()`, con cuidado de que el evento no se dispare al hacer doble clic sobre los
tres botones (que ya están fuera de la región de arrastre, así que tampoco heredan este manejador
si se ata al contenedor de arrastre y no a la barra entera).

## Resumen de impacto (para `data-model.md`/`tasks.md`)

- **Backend**: `src-tauri/src/platform/ventana.rs` (constantes `MIN_W`/`MIN_H` +altura de barra),
  `src-tauri/tauri.conf.json` (`decorations: false`, `minHeight`/`height` +altura de barra). Sin
  comandos nuevos, sin cambio de `ventana.on_window_event` ni de `gestionar_cierre`.
- **Frontend**: `src/lib/window.ts` (nuevo, wrapper de `@tauri-apps/api/window`),
  `src/lib/components/TitleBar.svelte` (nuevo, catálogo), `src/lib/design/icons.ts` + sprite
  (`minimize`/`maximize`/`restore`), `src/routes/+layout.svelte` (monta la barra + nuevo contenedor
  `h-screen`), `src/lib/components/AppShell.svelte` (`h-screen` → `h-full`), diccionarios `es`/`en`
  (nombres accesibles de los tres controles).
- **Sin cambios**: `platform/bandeja.rs`, la lógica de `lifecycle.close_action`, el resto de
  pantallas (ninguna monta su propio chrome, `AGENTS.md` §4).
