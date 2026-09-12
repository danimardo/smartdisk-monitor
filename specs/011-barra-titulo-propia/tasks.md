---
description: "Task list — Barra de título propia, integrada con el sistema de diseño (011)"
---

# Tasks: Barra de título propia, integrada con el sistema de diseño

**Input**: `specs/011-barra-titulo-propia/` (plan.md, spec.md, research.md, data-model.md,
quickstart.md)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ir en paralelo (fichero distinto, sin dependencias pendientes)
- **[Story]**: US1 / US2 / US3 (mapea a las historias de `spec.md`)
- Rutas exactas en cada tarea. Comandos de Rust desde `src-tauri/`.

**Tests**: constitución §VIII. `TitleBar.svelte` se prueba como componente puro (callbacks
simulados, sin `@tauri-apps/api` real — mismo patrón que `ExplicacionModal`). El wrapper
`src/lib/window.ts` no lleva test propio: es reenvío directo a la API de Tauri sin ramas, y el nivel
más barato que demuestra que funciona de verdad es la validación manual de `quickstart.md` (no hay
runtime de Tauri en `pnpm test`/`pnpm test:component`). `geometria_visible` ya tiene pruebas y no
cambia (research.md D1) — no hace falta tocarlas.

---

## Phase 1: Setup

- [X] T001 Confirmar baseline verde antes de empezar: `cargo test` (desde `src-tauri/`) y, desde la
      raíz, `pnpm check && pnpm lint && pnpm verify`. Anotar cualquier fallo previo no relacionado.
      **Hecho** — mismo fallo preexistente y sin relación en `export.rs` que en la feature anterior;
      resto en verde.
- [X] T002 [P] Añadir a `src/lib/i18n/es.json` y `src/lib/i18n/en.json` las claves nuevas:
      `titlebar.minimize`, `titlebar.maximize`, `titlebar.restore` (nombres accesibles de los tres
      controles). El botón cerrar reutiliza la clave ya existente `common.close` — no se crea una
      nueva para lo mismo. **Hecho**.

**Checkpoint**: `cargo test` y `pnpm verify` en verde. **Verificado.**

---

## Phase 2: Foundational (prerequisito bloqueante)

**Purpose**: sin esto no hay bar que probar — quitar la decoración nativa sin permisos ni
reemplazo deja la ventana inutilizable. **Bloquea las tres historias.**

- [X] T100 Redactar **el ADR de los permisos de Tauri nuevos** en `docs/decisions.md` (límite duro,
      `AGENTS.md`): `core:window:allow-close`, `allow-minimize`, `allow-toggle-maximize`,
      `allow-start-dragging`, `allow-is-maximized`. Explicar que Tauri 2 bloquea estas acciones por
      defecto y que es el único mecanismo del framework para una barra de título propia (no es una
      alternativa entre varias); referenciar esta spec. Siguiente número de ADR libre en
      `docs/decisions.md` en el momento de escribir esta tarea. **Hecho** — ADR-059.
- [X] T101 En `src-tauri/capabilities/default.json`, añadir los 5 permisos del ADR de T100 al array
      `permissions`, junto a `core:default` y `dialog:allow-save` ya existentes. **Hecho**.
- [X] T102 [P] En `src-tauri/tauri.conf.json`: `app.windows[0].decorations: false`; subir
      `minHeight` de 560 a 595 y `height` (tamaño por defecto) de 988 a 1023 — +35 px, la altura de
      la barra propia (reutiliza `--sdm-control-lg` de `tokens.css`, ver research.md D3). Dejar un
      comentario en el propio commit (o en `docs/decisions.md` si se prefiere) señalando que este
      +35 debe coincidir siempre con T103. **Hecho**.
- [X] T103 [P] En `src-tauri/src/platform/ventana.rs`, subir `MIN_H` de 560 a 595 (mismo +35 que
      T102 — comentario cruzado entre los dos ficheros, ninguno puede leer el token CSS del otro).
      **Hecho** — comentario en el propio `MIN_H` referenciando `tauri.conf.json`. Los 5 tests de
      `ventana::tests` (incluida `geometria_visible`) siguen en verde sin tocarlos (research.md D1).
- [X] T104 [P] En `src/lib/components/IconSprite.svelte`, añadir tres `<symbol>` nuevos
      (`i-minimize`, `i-maximize`, `i-restore`), mismo estilo que los 17 existentes (`viewBox="0 0
      24 24"`, `stroke="currentColor"`, `stroke-width="1.7"`, `fill="none"`): minimizar = una línea
      horizontal; maximizar = un cuadrado; restaurar = dos cuadrados solapados (icono estándar de
      Windows). En `src/lib/design/icons.ts`, añadir `"minimize" | "maximize" | "restore"` a
      `IconName` y a `ICON_NAMES`. **Hecho** — el "restaurar" son dos cuadrados solapados sin
      relleno (ambos `fill="none"`): un relleno sólido para ocultar la esquina trasera habría
      necesitado un color literal (`pnpm verify:tokens` lo habría rechazado) o un token de fondo que
      no encaja bien dentro de un `<symbol>` reutilizable en cualquier contexto. Se actualizó también
      `icons.test.ts` (17 → 20 símbolos).
- [X] T105 Crear `src/lib/window.ts` — único módulo que importa `@tauri-apps/api/window` en todo el
      frontend (research.md D5). Expone: `minimizeWindow()`, `toggleMaximizeWindow()`,
      `closeWindow()`, `isWindowMaximized(): Promise<boolean>`,
      `onWindowResized(cb: () => void): Promise<() => void>` (envuelve `onResized`, para que quien
      llama no dependa del tipo `PhysicalSize` de Tauri). Todo como reenvío directo a
      `getCurrentWindow()`, sin lógica de negocio. **Hecho**.

**Checkpoint**: `cargo build` compila con la ventana sin decoración (aunque todavía no haya barra
propia — se verifica visualmente en la historia 1, no aquí). `pnpm check` sigue en cero errores con
`window.ts` ya creado. **Verificado** en frontend (`pnpm check`/`pnpm verify` en verde); `cargo
build` pendiente de repetir en Polish — la app estaba corriendo y bloqueaba el `.exe` de depuración
(mismo bloqueo que en la feature anterior).

---

## Phase 3: User Story 1 - Controlar la ventana desde la barra propia (Priority: P1) 🎯 MVP

**Goal**: paridad funcional completa con la barra nativa que se quita: mover, minimizar,
maximizar/restaurar, cerrar, redimensionar.

**Independent Test**: con la app abierta, arrastrar la barra (se mueve), pulsar los tres controles
(cada uno hace lo mismo que su equivalente nativo) y redimensionar arrastrando un borde.

### Tests para User Story 1

- [X] T200 [P] [US1] En un nuevo `src/lib/components/TitleBar.browser.test.ts`: con
      `maximized={false}`, se ve un botón con nombre accesible `t("titlebar.maximize")`; con
      `maximized={true}`, el mismo botón pasa a `t("titlebar.restore")`. Al pulsar minimizar,
      maximizar/restaurar y cerrar se llama a `onminimize`/`ontogglemaximize`/`onclose`
      respectivamente, cada uno una sola vez. El botón cerrar usa el nombre accesible
      `t("common.close")` (reutilizado, no uno nuevo). **Hecho, ampliado con lo que salió en
      `/speckit-analyze`**: también se afirma qué símbolo del sprite (`#i-maximize`/`#i-restore`) se
      usa en cada estado (no solo el nombre accesible, E1), y un test de operabilidad por teclado
      con `userEvent.tab()` + `userEvent.keyboard("{Enter}")` real de Chromium — no un
      `dispatchEvent` simulado, que no dispara la activación nativa del `<button>` (E4).
- [X] T201 [P] [US1] En el mismo fichero: el contenedor con `data-tauri-drag-region` existe y los
      tres botones quedan **fuera** de él (comprobar que el atributo no está presente en los
      botones ni en sus ancestros hasta la raíz del componente, para no dejarlos inutilizables por
      arrastre — ver FR-004). **Hecho**.

### Implementación

- [X] T210 [US1] Crear `src/lib/components/TitleBar.svelte` (catálogo, presentacional, sin
      `@tauri-apps/api`): props `maximized: boolean`, `onminimize`, `ontogglemaximize`, `onclose`
      (todos `(() => void) | undefined`, patrón ya usado en `ExplicacionModal`). Estructura: un
      contenedor de altura `--sdm-control-lg` con `data-tauri-drag-region` cubriendo toda la franja
      salvo los tres botones (que van dentro pero sin heredar el atributo); cada botón con
      `<Icon name={...} />`, `aria-label` vía `t()`, fondo transparente con `hover:` del color de
      acento (mismo patrón que el botón `ghost` de `Button.svelte`, pero sin usar `Button` — los
      controles de ventana necesitan la geometría rectangular a tope de borde característica de
      Windows, no la forma de píldora del catálogo de botones). Hace pasar T200/T201. **Hecho** —
      US1 se queda deliberadamente en tratamiento neutro (`text-fg-dim`/`hover:bg-glass-3`, mismo
      que `ghost`), sin acento todavía: eso es US2 (T311), tal como ya decía la propia "Implementation
      Strategy" de este fichero. La región de arrastre lleva `role="presentation"` (limpia el aviso
      de accesibilidad de Svelte por el `ondblclick` en un elemento no interactivo, sin necesitar
      `svelte-ignore` ni entrada en `known-issues.md`).
- [X] T211 [US1] En `src/routes/+layout.svelte`: importar `TitleBar` y las funciones de
      `src/lib/window.ts`; estado local `maximizada = $state(false)`, poblado con
      `isWindowMaximized()` en el `onMount` ya existente y mantenido con `onWindowResized(...)`
      (limpiar el listener en el `return` del `onMount`, igual que ya hace `unsubscribe`). Montar
      `<TitleBar maximized={maximizada} onminimize={minimizeWindow} ontogglemaximize={toggleMaximizeWindow} onclose={closeWindow} />`
      **fuera** de las tres ramas (`startupError`/`esOnboarding`/normal) — mismo motivo que
      `<IconSprite />` ya está ahí fuera (research.md D4). **Hecho** — el sondeo de
      `isWindowMaximized()`/`onWindowResized` va en su propio `try/catch` aparte del arranque
      principal, para que un fallo ahí no dispare `startupError` (perder el reflejo del estado de
      la ventana no debe tumbar la aplicación).
- [X] T212 [US1] Reestructurar el `<body>` de `+layout.svelte`: envolver `<TitleBar />` + el bloque
      `{#if startupError}...{/if}` existente en un contenedor nuevo `flex flex-col h-screen`, con
      `<TitleBar />` como primera fila (altura fija) y un segundo `<div class="min-h-0 flex-1
      overflow-hidden">` alrededor del `{#if}` existente. **Hecho**.
- [X] T213 [US1] Cambiar `h-screen` → `h-full` en `src/lib/components/AppShell.svelte` y en los dos
      bloques de `+layout.svelte` que hoy usan `h-screen` directamente (`startupError` y el
      `esOnboarding` en su estado `ready`/cargando) — ahora ocupan el alto que les deja el
      contenedor de T212, no el 100 % de la ventana entera. **Hecho, con un fichero más de lo
      previsto**: `src/routes/onboarding/+page.svelte` también tenía su propio `min-h-screen` en la
      raíz del asistente (no estaba en el plan original — se detectó al revisar todo lo que asumía
      100 % del alto de la ventana). Cambiado a `h-full min-h-0`, mismo patrón que `AppShell`.
      Verificado con la suite completa de `pnpm test:e2e` (140 pasan, incluida `escalado.spec.ts`,
      que comprueba "no recorta en silencio" en `/onboarding` y en las seis secciones a varios
      tamaños, y `onboarding.spec.ts` entero).
- [X] T214 [US1] Verificar (sin cambio de código esperado, solo comprobación) que el
      redimensionado por los bordes sigue funcionando: Tauri 2 gestiona el resize de ventanas sin
      decoración de forma nativa desde la versión 2.0, incluso con un `data-tauri-drag-region` en
      el borde superior — no hace falta código propio para esto (ver hallazgo en `research.md`,
      changelog de Tauri 2.0 "Enhancements"). Si la comprobación manual (`quickstart.md`) revela lo
      contrario, se convierte en una tarea de seguimiento, no se bloquea esta historia por ello.
      **Pendiente de la validación manual** (T901) — nada que hacer en código.

**Checkpoint**: se puede mover, minimizar, maximizar/restaurar, cerrar y redimensionar la ventana
solo desde la barra propia, en las tres ramas de `+layout.svelte` (normal, onboarding, error de
arranque). Historia entregable de forma independiente (MVP).

---

## Phase 4: User Story 2 - Barra integrada con el diseño de la aplicación (Priority: P2)

**Goal**: la barra deja de leerse como una franja de sistema: mismo fondo que el resto de la
ventana, controles con el acento de la app, tema claro y oscuro.

**Independent Test**: comparar visualmente la barra con el resto de la ventana (mismo fondo, sin
franja) y los tres controles con un botón primario existente (mismo acento), en los dos temas.

### Tests para User Story 2

- [X] T300 [P] [US2] En `TitleBar.browser.test.ts`: el color de fondo computado del contenedor de
      la barra coincide con el `background-color`/gradiente que usa `AppShell` (mismo token CSS,
      no un valor distinto) — comprobar por clase/variable CSS aplicada, no por captura visual.
      **Hecho, ampliado con lo que salió en `/speckit-analyze`**: se añadieron dos tests más en el
      mismo bloque — el color de los tres iconos en reposo coincide con `--sdm-accent` **resuelto de
      verdad** (una sonda con `getComputedStyle`, no comparar cadenas `var(--sdm-*)`, E2); y el mismo
      contraste con `data-theme="dark"` activo, confirmando primero que claro y oscuro dan colores
      distintos de verdad (si no, la prueba no demostraría nada) y luego que la barra seguía el
      oscuro (E3).

### Implementación

- [X] T310 [US2] En `TitleBar.svelte`: el contenedor de la barra usa el mismo fondo que
      `AppShell.svelte` (`bg-[linear-gradient(160deg,var(--sdm-bg),var(--sdm-bg-2))]` o la utilidad
      Tailwind equivalente ya mapeada a esos tokens — sin duplicar el valor a mano, reutilizar la
      misma clase o extraerla a una utilidad compartida si `verify:tokens` lo exige). Sin borde ni
      sombra que la separe visualmente del resto de la ventana. **Hecho** — se reutilizó la clase
      literal tal cual (no hizo falta extraerla: `verify:tokens` no marca valores `var(...)` dentro
      de `bg-[...]`, solo colores/radios/sombras/tamaños de fuente escritos a mano).
- [X] T311 [US2] En `TitleBar.svelte`: los tres iconos usan `text-accent`/`var(--sdm-accent)` en
      reposo y su variante `hover`/`-hi` al pasar el ratón (mismo patrón que ya usan los botones
      `ghost`/`ConfirmDialog` para el acento) — nunca un color de Windows por defecto. El botón
      cerrar puede usar el tono crítico (`--sdm-crit`) en su `hover`, como ya hacen otras apps de
      Windows con barra propia (VS Code, Windows Terminal) — confirmar que este matiz no rompe
      FR-009 (sigue siendo un acento propio de la aplicación, no el rojo por defecto del sistema, y
      solo aparece al interactuar, no en reposo). **Hecho** — los tres botones parten de `text-accent
      hover:bg-accent-soft hover:text-accent-hi`; el botón cerrar sobrescribe solo el `hover` a
      `hover:bg-crit-soft hover:text-crit`, dejando el reposo igual que los otros dos (confirmado por
      el test de T300, que pasa los tres botones por el mismo bucle).
- [X] T312 [US2] Comprobar en `pnpm test:component` y en la validación manual que la barra se ve
      correctamente en tema claro y oscuro sin condicionales de plataforma (ya cubierto por que los
      tokens `--sdm-*` resuelven solos por tema — sin código nuevo esperado, solo verificación).
      **Hecho en automático** (el test de tema oscuro de T300 cubre justo esto); validación manual
      visual pendiente de T901/T902 igual que el resto de la app.

**Checkpoint**: la barra no se distingue del resto de la ventana como zona de sistema aparte, en
los dos temas, con los controles en el acento de la aplicación. **Verificado** — 10/10 pruebas de
`TitleBar.browser.test.ts` en verde, `pnpm check` sin errores ni avisos, `pnpm verify` en verde.

**Corrección post-validación** (tras ver la barra en marcha, tres ajustes pedidos por el usuario):

1. Fondo: el degradado `bg`/`bg-2` se leía más oscuro que el resto del lienzo a la altura tan fina
   de la barra. Cambiado a `bg-solid` (`--sdm-solid`, #fdfcfe en claro) — el mismo plano opaco que
   ya usan `.sdm-material-*` como respaldo sin `backdrop-filter`, indistinguible a ojo del resto de
   la interfaz (que es justo lo que pedía FR-008). El usuario pidió el literal `#FBF9FC`; se explicó
   que un literal nuevo violaría "cero valores visuales literales" (`pnpm verify:tokens`) y que
   `--sdm-solid` ya existente es visualmente idéntico (diferencia de 2-3/255 por canal, invisible) —
   aceptado.
2. Icono de la app + `t("app.name")` añadidos a la izquierda de la barra, dentro de la propia
   región de arrastre (para que también se pueda arrastrar la ventana desde ahí). El icono es
   `src/lib/assets/icono-app.png` (copia del `32x32.png` ya generado, mismo precedente que la foto
   de `AboutDialog`, ADR-052) con `alt=""`: es decorativo, el nombre de al lado ya lo anuncia a un
   lector de pantalla.
3. `AppShell.svelte`: `rounded-window` (4 esquinas) → `rounded-b-window` (solo abajo). Arriba ya no
   linda con el borde real de la ventana sino con `TitleBar`, así que esas dos esquinas quedaban
   flotando en mitad de la ventana sin ningún borde que imitar; abajo sigue siendo el borde real,
   que Windows 11 redondea por su cuenta (DWM, D7), así que se mantiene.

Pruebas actualizadas en `TitleBar.browser.test.ts` (fondo `bg-solid` resuelto de verdad, icono +
nombre presentes y dentro de la región de arrastre) — 14/14 en verde. `pnpm check`/`verify`/
`test:component` (188/188) reverificados tras el cambio.

---

## Phase 5: User Story 3 - Paridad con gestos estándar de Windows que sí se conservan (Priority: P3)

**Goal**: doble clic sobre una zona vacía de la barra maximiza o restaura.

**Independent Test**: doble clic sobre la franja de arrastre (no sobre los botones) maximiza si
estaba restaurada, restaura si estaba maximizada.

### Tests para User Story 3

- [X] T400 [P] [US3] En `TitleBar.browser.test.ts`: doble clic sobre el contenedor con
      `data-tauri-drag-region` llama a `ontogglemaximize`; doble clic sobre cualquiera de los tres
      botones **no** lo llama (solo dispara la acción de ese botón). **Hecho, con un matiz
      detectado al escribirla**: el botón maximizar/restaurar ya llama a `ontogglemaximize` por su
      propio `onclick`, así que "doble clic sobre él no llama a `ontogglemaximize`" es una
      aserción falsa de partida (fallaba con 2 llamadas, no 0) — no es un fallo de la
      implementación, es que la propia prueba tal como estaba escrita en la plantilla no
      distinguía "la acción propia del botón" de "burbujeo desde la región de arrastre". Reescrita
      en tres pruebas: (1) doble clic sobre la región llama una vez; (2) doble clic sobre minimizar
      o cerrar no llama nunca a `ontogglemaximize`, confirmando que no hay burbujeo; (3) doble clic
      sobre el propio botón maximizar produce exactamente 2 llamadas (las de su `onclick`, una por
      clic) y no 3, lo que habría delatado que la región también lo capturó.
- [X] T410 [US3] En `TitleBar.svelte`: `ondblclick={() => ontogglemaximize?.()}` en el contenedor
      de arrastre. Como los botones están fuera de ese contenedor (T210/T201), un doble clic sobre
      ellos no burbujea a este manejador salvo que la disposición del DOM diga lo contrario —
      confirmar con T400. **Hecho** — ya implementado desde T210 (Fase 3); T400 lo confirma ahora
      con prueba automática real en vez de darlo por hecho.

**Checkpoint**: doble clic para maximizar/restaurar funciona, sin interferir con los tres botones.
**Verificado** — 13/13 pruebas de `TitleBar.browser.test.ts` en verde.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T900 Ejecutar `pnpm check`, `pnpm lint`, `pnpm verify`, `pnpm test`, `pnpm test:component`,
      `cargo clippy --all-targets -- -D warnings` y `cargo fmt --check` (desde `src-tauri/`); todo
      en verde. **Hecho, con una nota**: `pnpm check` (0/0), `pnpm lint` (0, tras formatear dos
      HTML de depuración sueltos en `.claude/tmp/` que no tenían que ver con esta feature), `pnpm
      verify` (assets/tokens/i18n 608 claves/fronteras, los 4 en verde), `pnpm test` (354/354),
      `pnpm test:component` (187/187, incluidas las 13 de `TitleBar.browser.test.ts`), `cargo
      clippy --all-targets -- -D warnings` (verde; solo ruido preexistente de `ts-rs` con
      `serde(skip_serializing_if)`, ajeno a esta feature) y `cargo fmt --check` (verde). `cargo
      build` estuvo bloqueado un momento por el mismo motivo que en la Fase 2 (el `.exe` de
      depuración en uso); tras confirmar con la persona que la app estaba cerrada, quedaba un
      proceso `smartdisk-monitor.exe` residual (PID visible en `tasklist`, no en pantalla) que se
      cerró solo entre el aviso y el intento de `taskkill` — reintentado, **`cargo build` termina
      en verde**.
      **`pnpm lint` encontró además un error preexistente y ajeno a esta feature**, sin tocar
      aquí: `src/lib/stores/explicacion.svelte.ts` importa `estadoIa` sin usarlo (cambio sin
      commitear de trabajo anterior, spec 010) — se avisa al usuario en el informe de cierre en vez
      de corregirlo en silencio, por no ser parte del alcance de esta feature.
- [ ] T901 Recorrer `quickstart.md` a mano (`pnpm app:dev`) siguiendo las 3 historias en orden, más
      los puntos de riesgo anotados ahí (esquinas de la ventana, ventana mínima, multi-monitor).
      **Requiere a la persona** — UAC, ventana nativa.
- [ ] T902 [P] Comprobar tema claro y oscuro y la ventana mínima (1024×595) sin recortes (§8 de
      `ui-design.md`, definición de terminado). **Requiere a la persona**, mismo motivo que T901.
- [ ] T903 Actualizar `docs/open-questions.md` §L si la validación manual revela que el presupuesto
      de píxeles medido (que asumía 32 px de barra nativa) necesita una nota nueva con los 35 px de
      la barra propia; si no hace falta nada más allá de lo ya razonado en `research.md` D3, dejarlo
      dicho explícitamente al cerrar la tarea. **Pendiente de T901/T902** — la tabla de §L.2 es una
      medición real sobre hardware, no se recalcula a mano sin repetir la medición; se decide al
      cerrar, con el resultado de la validación manual delante.
- [X] T904 `pnpm docs:build` si se tocó `docs/decisions.md` (el ADR de T100) u otro fichero de
      `docs/` aparte de los ya cerrados.

---

## Dependencies & Execution Order

- **Setup (Fase 1)** → **Foundational (Fase 2, bloqueante)** → **US1 (Fase 3, P1, MVP)**.
- **US2 (Fase 4)** y **US3 (Fase 5)** dependen de US1 (necesitan que la barra ya exista y funcione)
  pero son independientes entre sí — se pueden hacer en paralelo o en cualquier orden una vez
  cerrada US1.
- **Polish (Fase 6)** depende de las historias que se vayan a entregar.

### Paralelismo

- T102/T103 (tamaño de ventana en dos lenguajes) en paralelo, pero **deben llevar el mismo número**
  — no son independientes en contenido, solo en fichero.
- T104/T105 (iconos y wrapper de ventana) en paralelo entre sí y con T102/T103.
- T300 y T310/T311 tocan el mismo fichero (`TitleBar.svelte`) que T210 — secuenciales dentro de la
  misma historia, en paralelo solo entre historias distintas si se reparte el trabajo.

## Implementation Strategy

### MVP primero (User Story 1)

1. Fase 1 (Setup) → Fase 2 (Foundational, incluye el ADR y los permisos) → Fase 3 (US1).
2. **Parar y validar**: recorrer el escenario 1 de `quickstart.md` a mano — es la primera vez que
   la ventana pierde la decoración nativa, conviene confirmar que no queda inutilizable antes de
   seguir.
3. US1 sola ya es funcionalmente completa (paridad con la barra nativa), aunque todavía tenga el
   aspecto de una barra "neutra" sin el acento de la aplicación.

### Entrega incremental

1. Setup + Foundational (con su ADR) → base lista.
2. US1 → MVP funcional (paridad, sin pulido visual todavía).
3. US2 → integración visual (el motivo real de la feature).
4. US3 → doble clic, comodidad esperada.
5. Polish.
