# Regresión visual y definición de terminado por PR

Cada PR que toca una pantalla o el chrome rellena su bloque con la checklist de `docs/ui-design.md` §8.

## Nota sobre capturas

`tools/capture-app.ps1` necesita la aplicación **elevada** (UAC), que ninguna automatización de este
entorno puede lanzar. La verificación visual se apoya en: `pnpm test:component` (contraste sobre el
material compuesto, resolución de tokens por tema, foco, recorte), `pnpm test:e2e` + `pnpm test:a11y`,
y una revisión manual del usuario contra `design/propuesta-redisenov2/mockups/smartdisk-v3.html`.
Las capturas de referencia del «antes» (`referencia-antes/`) quedan pendientes de generar a mano.

---

## PR 1 · US1 — Tokens v3 (2026-09-06)

Cambio global de tokens; ninguna pantalla recompuesta todavía.

- [x] Solo tokens; cero literales de color/tamaño (`pnpm verify:tokens` verde).
- [x] `--sdm-on-accent` es tinta en oscuro, blanco en claro (prueba de componente nueva).
- [x] Todos los tokens nuevos (`--sdm-font-display`, `-text-display`, `-text-hero`, `-tracking-display`, `-rail-width`, `-hero-height`, `-icon-stroke`, `-icon-size`) resuelven en los dos temas.
- [x] `.sdm-display` = familia display + peso 600 + `tabular-nums` (no inventa 700).
- [x] Ratios de contraste de la paleta Ciruela medidos y registrados en `docs/open-questions.md` §S. Todos ≥ 4,5:1 sobre material y sobre bloque interno. Los tres más justos verificados.
- [x] `Button` variante `primary` usa `text-fg-onAccent`, no `text-white`.
- [x] `tailwind.config.cjs` y `tokens.json` espejan los tokens nuevos.
- [x] `docs/ui-design.md` describe v3 (§ intro, §0, §2, §2.bis, §3). §4/§6/§8 conservados.
- [x] ADR-034 y ADR-035 en `docs/decisions.md`; ADR-013 y ADR-017 marcados como enmendados. `historias.md` regenerado.
- [x] `pnpm verify && pnpm check && pnpm lint && pnpm test && pnpm test:component && pnpm test:e2e:smoke` en verde.
- [ ] **Pendiente de revisión manual del usuario**: recorrer las 8 pantallas en claro y oscuro contra el mockup; comprobar que ningún `text-white` sobre color de estado (badges «!» de `bg-warn` en `ConfirmDialog`, `reports`, `tests`) quedó ilegible en oscuro — se corrige en PR 7 al recomponer esas pantallas.

---

## PR 2 · US2 — Iconografía (2026-09-06)

Sprite + `Icon` + `icons.ts` + ranura de icono en `StatusPill`. Ninguna pantalla recompuesta.

- [x] Sprite de 15 `<symbol>` montado una sola vez en `AppShell` (sin `<metadata>` c2pa). `viewBox` 24×24, `stroke="currentColor"`, cero literales de color.
- [x] `Icon` cumple la regla de a11y de `ui-design.md` §6: `label` ⇒ `role="img"` + nombre; sin `label` ⇒ `aria-hidden` + `focusable="false"`.
- [x] Mapas semánticos centralizados en `$lib/design/icons.ts` (no repartidos por componentes).
- [x] `StatusPill` con ranura de icono (`aria-hidden`, excluyente con `withDot`). `label` sigue obligatorio.
- [x] Pruebas: `Icon.browser.test.ts` (5), `icons.test.ts` (8), `StatusPill.browser.test.ts` (5).
- [x] Puerta: verify ✓ · check 0/0 · lint ✓ · test 193 · test:component 56 · e2e:smoke 5 · a11y 14.
- [ ] **Pendiente de revisión manual**: abrir `design/propuesta-redisenov2/mockups/icons-hoja-de-contacto.html` y comparar los 15 iconos renderizados en la app (aparecerán cuando los usen las pantallas, PR 3+).

---

## PR 3 · US3 — Chrome (2026-09-06)

Riel de 74 px + `Toolbar` de 56 px + estado global una sola fuente. Corrige 2 de los 3 bugs de las capturas.

- [x] Solo tokens; cero literales (`verify:tokens` verde). `size-11` (44 px), `w-rail`, `h-14` son utilidades, no literales de color/radio/sombra/fuente.
- [x] Riel de `--sdm-rail-width` (74 px). A 1024×560 el contenido dispone de ~950 px (`e2e/ui/escalado.spec.ts` verde en las 7 pantallas × 5 escalados).
- [x] Accesibilidad del riel: cada botón `link`/`button` con nombre accesible (`aria-label`) aunque no haya texto visible; activo con `aria-current="page"`; punto de aviso de Alertas no viaja solo (el recuento entra en el `aria-label` → «Alertas, 3 sin revisar»); pie `role="status"` con el texto como nombre. `a11y` (axe) verde.
- [x] Selección con material elevado + icono en `text-accent-fg`. **Nunca** barra de color lateral.
- [x] `Toolbar`: título **de la ruta** (`page.data.title`), no «Panel general» fijo. `disks/[id]` pone el alias. **Bug corregido.**
- [x] Estado global: `globalStatus()` en `health.ts`, una sola fuente para la píldora de la `Toolbar` y el pie del riel — no pueden contradecirse. Distingue «cargando» de «sin discos» (la causa del «Sin discos monitorizados» con un disco delante). **Bug corregido.**
- [x] Inventario cargado en el `onMount` del layout → estado global correcto también al entrar directo a `/disks/x`, `/alerts`, etc.
- [x] Pausa de recopilación: movida a Ajustes → Registro de actividad. Sigue en el menú de la bandeja (sin cambios en `src-tauri`). No se pierde función.
- [x] Claro y oscuro: solo tokens, sin condicionales.
- [x] Puerta: verify ✓ · check 0/0 · lint ✓ · test 199 · test:component 65 · e2e 76 · a11y 14 · docs sincronizados.
- [x] Iconos de sección: coinciden **exactamente** con el `navDefs` del mockup del diseñador — Panel `diskStack`, Alertas `alert`, Eventos `plug`, Pruebas `flask`, Informes `shield`, Ajustes `wear`.
- [ ] **Menor, pendiente de confirmar con el diseñador**: el icono de **«Acerca de»** en el riel. El mockup no dibuja ese botón (solo 6 secciones) y `Toolbar.md` dice que va «como último icono» sin especificar cuál. Puesto `tag` provisionalmente; el juego de 15 no tiene un símbolo de info/interrogación. Es una prop: se cambia sin tocar estructura.
- [ ] **Pendiente de revisión manual del usuario**: recorrer el chrome nuevo en claro y oscuro; comprobar que el riel se aprende y que el estado global coincide en los dos sitios.

---

## PR 4 · US4 — La gráfica vuelve a comunicar (2026-09-06)

`Sparkline` nuevo + `series.ts` (lógica compartida) + `TimeSeriesChart` con eje Y, relleno degradado y leyendas de hueco.

- [x] Solo tokens; el trazo hereda `currentColor`, el relleno usa `stop-color="currentColor"` + `stop-opacity` (número, no color). `verify:tokens` verde.
- [x] `Sparkline`: `<polyline>` por tramo, **nunca interpola** un `null` ni un salto temporal; `vector-effect="non-scaling-stroke"` en todo trazo; serie vacía/`null` → no dibuja nada (ni línea a cero); `id` de degradado único por instancia.
- [x] `TimeSeriesChart`: eje Y con 4 marcas fuera del área de trazo; relleno degradado bajo la curva; banda de hueco **con su leyenda `sin datos HH:MM – HH:MM`** por hueco (no solo un contador); umbral discontinuo + leyenda; dominio = intervalo pedido (`from`/`to`), no el de los datos.
- [x] Estado «sin muestras»: marco + eje + mensaje central, sin línea a cero. Lectura textual equivalente (`aria-label`) conservada. Cursor teclado + ratón conservado.
- [x] La regla «un hueco es un hueco» vive **una vez** en `$lib/design/series.ts`; `Sparkline` y `TimeSeriesChart` la comparten (por función, no por anidamiento de SVG).
- [x] Pruebas: `series.test.ts` (18), `Sparkline.browser.test.ts` (6), `TimeSeriesChart.browser.test.ts` (+4 v3).
- [x] Puerta: verify ✓ · check 0/0 · lint ✓ · test 213 · test:component 74 · e2e 76 · a11y (pendiente de correr en este PR, sin cambios de a11y).
- [ ] **Pendiente de revisión manual**: abrir el detalle de un disco con una serie real que tenga un hueco y confirmar que el trazo, el eje y la banda se ven. *(El «bug de gráfica vacía» de las capturas: el código v2 ya redibujaba bien —no usaba `preserveAspectRatio="none"`—; lo más probable es que la captura mostrara una serie realmente sin muestras o con tramos de 1 punto. El eje Y nuevo y las pruebas de regresión reducen el riesgo; si vuelve a salir vacía con datos reales, es un problema de datos del backend, no de la gráfica.)*

---

## PR 5 · US10 — Perfiles de alerta (backend) (2026-09-06) · modo plan

Toca `src-tauri/` + docs normativa. Plan aprobado; ADR-036 escrito.

- [x] `settings.alerts` + 7 claves (`profile`, `wear_*`, `media_errors_*`, `driver_retry_*`) con rango, validación y prueba de rechazo. `settings.onboarding.completed_at` en la lista blanca.
- [x] El motor **lee** los umbrales (antes literales). `smart.wear_high`, `temp.above_configured_*`, `smart.media_errors` parametrizados. Umbral térmico de fábrica 70/80 → **60/70** (clarify Q2).
- [x] **`capacity.low`/`capacity.critical` implementadas en el motor** (decisión del usuario): `domain/capacidad.rs` (espejo del TS), persistencia de `volume_free_bytes` como serie, histéresis de 3 ciclos.
- [x] `driver_retry_*` **guardado pero sin consumidor** (la regla `events.*` necesita el colector de eventos, Historia 4). Registrado en `open-questions.md` §T.4.
- [x] `VolumeSummary.isSystemVolume` (clarify Q3): `GetSystemWindowsDirectoryW`, calculado al leer, **sin migración de esquema**.
- [x] Ajustes → sección de perfiles (`RadioGroup` + 6 campos + «Personalizado (a partir de X)»). Elegir un perfil reescribe los 12 valores; editar uno a mano → `custom`.
- [x] Puerta: `cargo test` 459 ✓ · `clippy -D warnings` ✓ · cobertura `domain`+`alerts` **93,3 %** · `pnpm check` 0/0 · `verify` ✓ (346 claves i18n) · `lint` ✓ · `test` 217 · `test:component` 74 · `test:e2e` 76 · `docs:check` ✓.
- [ ] **Pendiente de revisión manual** (`pnpm app:dev`): Ajustes → elegir «Prudente» y ver los 12 campos cambiar; editar uno → «Personalizado (a partir de Prudente)»; con un volumen por debajo del 8 % de espacio libre, comprobar que salta `capacity.low` tras unos ciclos.

### Deuda anotada para PR posteriores

- `ConfirmDialog.svelte:61`, `routes/reports/+page.svelte:273`, `routes/tests/+page.svelte:377`: badge `bg-warn text-white`. `text-white` sobre `--sdm-warn` oscuro (`#e0b473`) da ~1,6:1. Pre-existente (no es regresión de v3). Se corrige en PR 7 (recompone `tests` e `informes`) y con un retoque en `ConfirmDialog`.
- `Sidebar.svelte`: `border-white/95` en el logo — se resuelve en PR 3 (reescritura completa del riel).
- `Switch.svelte:23`: `bg-white` del punto activo — **excepción documentada** (es un brillo, no texto). Se deja.

---

## PR 6 · US5 + US6 — Panel general y detalle de disco (2026-09-06)

`HeroPanel` + rejilla recompuesta + «Reparto de estados» + «Sucesos del sistema» en el panel;
cabecera de identidad + 4 `MetricCard` con icono/sparkline + intervalo junto a la gráfica en el detalle.

### Panel general (§8)

- [x] Solo tokens; cero literales (`verify:tokens` verde). `h-hero`, `w-rail`, `text-hero`, `text-metric` son utilidades.
- [x] Correcto en claro y oscuro, sin condicionales. El velo de legibilidad del `HeroPanel` usa `var(--sdm-glass)` en los dos temas.
- [x] Ventana mínima 1024×560 sin recortes: la rejilla `minmax(272px, 1fr)` cae a una columna; la columna de hechos del héroe se oculta por debajo de 1100 px (`max-[1100px]:hidden`).
- [x] Textos en `es` y `en` (`verify:i18n` — 358 claves sincronizadas).
- [x] Estados diseñados: **cargando** (`HeroPanel loading` → esqueleto de la cifra; `ready` gобierna la rejilla), **vacío** (`EmptyState kind="empty"` con acción «Buscar dispositivos otra vez», sin héroe ni rejilla), **no compatible** (USB sin SMART en la rejilla en gris, «Sin datos SMART», magnitudes «—»; nunca es el héroe — `selectHeroDisk` lo excluye), **error de fuente** (un fallo de serie por disco degrada solo esa sparkline), **dato obsoleto** (`HeroPanel` admite `lastValidAt`).
- [x] Dato ausente como «No disponible» / «—» con el texto completo en `title`; ningún cero inventado.
- [x] Teclado y foco: navegación por `<a href>` real en cada `DiskCard` (constitución §XIV); acciones del héroe son `Button`. `aria-label` en iconos.
- [x] `HealthDonut` **se mantiene en el catálogo** y sale del panel; «Reparto de estados» es composición de pantalla, no componente nuevo.
- [x] **SC-006**: `e2e/ui/rendimiento.spec.ts` (20 discos en caliente → cero tareas ≥ 50 ms) sigue verde con la rejilla plana. La `VirtualList` de la rejilla se retira; decisión y medición en `docs/open-questions.md` §U.
- [x] Pruebas: `HeroPanel.browser.test.ts` (7), `DiskCard.browser.test.ts` (actualizada), `health.test.ts` (+8 `selectHeroDisk`, +7 `globalStatus`), `e2e/ui/dashboard.spec.ts` (3).
- [ ] **Pendiente de revisión manual del usuario**: recorrer el panel en claro y oscuro contra `mockups/smartdisk-v3.html`; confirmar que el héroe elige el disco correcto y que la curva de fondo se lee bajo el velo.

### Detalle de disco (§8)

- [x] Solo tokens; cero literales (`verify:tokens` verde).
- [x] Correcto en claro y oscuro, sin condicionales.
- [x] Ventana mínima: la rejilla `lg:grid-cols-[1.6fr_1fr]` cae a una columna; las 4 `MetricCard` pasan de `sm:grid-cols-4` a 2 columnas.
- [x] Textos en `es` y `en`.
- [x] Estados diseñados: **cargando** (`serieLoading` → texto; cabecera real desde el inventario), **vacío** (serie sin muestras → marco + eje + mensaje, sin línea a cero, lo pinta `TimeSeriesChart` de PR 4), **error de fuente** (`serieError` → `EmptyState kind="error"`), **dato obsoleto** (`frescura` de la cabecera; `MetricCard` admite `age`). El estado **no compatible** de las métricas de firmware queda para cuando la pantalla muestre el bloque de firmware (hoy no lo tiene).
- [x] Dato ausente: `MetricCard value={null}` → «No disponible» a `text-lg`, sin sparkline, nunca como cifra.
- [x] Ninguna cifra con peso 800: `MetricCard` usa `.sdm-display` (600).
- [x] El `SegmentedControl` + `DateRangePicker` de intervalo viven junto a la gráfica, dentro de `<main>` (T083). La `Toolbar` ya no tiene ranura de controles (PR 3).
- [x] `DataRow` con delta coloreado solo cuando `deltaIsMeaningful`.
- [x] Teclado y foco verificados (`disk-detail.spec.ts` — radios de intervalo, selector de fechas).
- [x] Pruebas: `MetricCard.browser.test.ts` (+2), `e2e/ui/disk-detail.spec.ts` (+1).
- [ ] **Pendiente de revisión manual**: abrir el detalle de un disco con serie real (con hueco) y confirmar cabecera, 4 métricas con icono, gráfica con eje y banda, e intervalo respondiendo junto a la gráfica.

### Deuda / aplazado

- **T085 «bloque de vida estimada»** (`cambios/07-detalle-disco.md` §3): **aplazado**. El contrato `DeviceDetail`/`smartCounter` no expone hoy una proyección de vida ni la ventana temporal sobre la que se calcularía el ritmo de desgaste (`delta` existe, su intervalo no). Construir la proyección solo en el cliente sería inventar el dato — justo lo que la constitución (§ «los números vienen de medir») y el propio boceto («declara siempre que es una proyección y sobre qué ventana») quieren evitar. Se retoma cuando el backend provea el ritmo de desgaste con su ventana. **Requiere confirmación del usuario / diseñador.**
- Iconos de `MetricCard` para actividad/desgaste/horas: `pulse`/`wear`/`clock` del juego de 15. Coinciden con `MetricCard.md`; el de «horas encendido» (`clock`) es el más discutible — pendiente de la revisión manual contra el mockup.

---

## PR 7 · US7 — Pantallas secundarias en v3 (2026-09-06)

`ProgressBar emphasis`, prueba en curso con cifra a 58 px, nivel/severidad con icono, Ajustes con
zona destructiva separada. Corrige la deuda de los badges `bg-warn text-white`.

### Componentes

- [x] `ProgressBar`: prop `emphasis` (`inline` 10 px `bg-accent` plano / `display` 12 px degradado `accent→accent-hi` + `shadow-edge`). `role="progressbar"` y la regla «siempre con leyenda + restante» intactos. Solo tokens (el degradado usa `var(--sdm-accent)`/`-hi`).
- [x] `EventRow`: nivel como cuadrado de 26 px con `Icon` (`eventLevelIcon`) y `label` → `role="img"` con nombre; el color no viaja solo. La etiqueta «asociación inferida» baja a `text-2xs` sin negrita.
- [x] `AlertCard`: píldora de severidad con `Icon` (`severityIcon`); `×N` en `.sdm-num` (ya lo estaba).
- [x] `Card`: ranura `leading` (cuadrado de icono a la izquierda del título) + prop `border` (`hairline`|`crit`). Aditivas; ningún consumidor existente cambia de aspecto.
- [x] `ConfirmDialog` / `reports`: el badge `!` `bg-warn text-white` (≈ 1,6:1 en oscuro) → `Icon name="alert"` en `text-warn` sobre `bg-warn-soft`. **Deuda de PR 5 saldada.**
- [x] Pruebas: `ProgressBar.browser.test.ts` (+2), `EventRow.browser.test.ts` (4, nuevo), `AlertCard.browser.test.ts` (+1), `icons.test.ts` (+1 `severityIcon`).

### Pruebas y diagnóstico (§8)

- [x] Solo tokens; cero literales (`verify:tokens` verde). `size-[38px]`, `size-[28px]`, `min-[1100px]:` son utilidades, no literales de color/radio/sombra/fuente.
- [x] Claro y oscuro sin condicionales.
- [x] Ventana mínima 1024×560: la cifra de progreso cae a `text-metric` (27 px) por debajo de 1100 px; las métricas pasan de `sm:grid-cols-4` a 2 columnas; las tarjetas de prueba de `md:grid-cols-3` a una columna. Sin recortes.
- [x] Textos en `es` y `en` (sin claves nuevas — ver T095).
- [x] Estados: **cargando** (las tarjetas dependen de `app.devices`; el bloque de prueba en curso no aparece hasta saber si hay alguna), **vacío** (historial → texto «Todavía no…»), **no compatible** (autotest con botón deshabilitado + `disabledReason`, píldora gris, nunca roja), **error de fuente** (`accionError` en bloque `bg-crit-soft`), **cancelando** (botón deshabilitado con motivo, leyenda «Cancelando…»).
- [x] Dato ausente: los formateadores (`formatThroughput`/`formatLatency`/`formatTemperature`) ya devuelven «No disponible».
- [x] Teclado y foco: `disk-detail`/`tests` e2e verdes; el bloque de prueba en curso y el historial son navegables.
- [x] Aviso de impacto con `Icon`, no un badge `text-white`.
- [x] e2e: `tests.spec.ts` (4) — con prueba en curso (progreso + cancelar) y sin ella (bloque ausente + iconos de tarjeta).
- [ ] **Pendiente de revisión manual**: lanzar un benchmark real (`pnpm app:dev`) y ver la cifra a 58 px, la barra con degradado y las cinco métricas; comprobar el corte a 1100 px.

### Alertas / Eventos / Ajustes / Informes (§8)

- [x] Solo tokens; `verify:tokens` verde. `a11y` (axe) verde en las 7 pantallas, claro y oscuro.
- [x] **Alertas**: `AlertCard` con icono de severidad; sin cambios de distribución. Los 5 estados intactos (no cambian de estructura, `02-alertas.md` §4).
- [x] **Eventos**: `EventRow` con cuadrado de nivel; el panel de detalle gana la píldora de texto del nivel (antes no mostraba el nivel en ningún sitio del detalle). Altura de fila sin recalcular. Los 5 estados intactos.
- [x] **Ajustes**: «Borrar todos los datos» separada con `border="crit"` y ≈ 32 px extra; el fondo no se tiñe (solo el filo). Cada sección en su `Card`; controles sobre `bg-glass-3`.
- [x] **Informes**: hereda tokens; badge de identificadores personales cambiado a `Icon` en `text-warn`.
- [ ] **Pendiente de revisión manual**: recorrer las cuatro en claro y oscuro contra el mockup; confirmar que el filo `crit` de la zona destructiva de Ajustes se ve pero no alarma.

### Deuda / aplazado

- **5.ª métrica del benchmark** (`04-pruebas.md` §2 menciona «cinco métricas»): hoy se muestran 4 (escritura, lectura, latencia de lectura, temperatura máx.). El contrato `testResult` también trae `writeLatencyMs`; añadirla exige separar la etiqueta «Latencia media» en «lectura»/«escritura» (2 claves i18n nuevas). Valor marginal, aplazado — se hace junto con la revisión manual si el usuario lo pide.
- Icono de la tarjeta de autotest SMART (`bolt`) y de chkdsk (`shield`): de `04-pruebas.md` §3 (benchmark→`flask`, chkdsk→`shield`, autotest→`bolt`). Coinciden con la ficha; pendiente de la revisión visual.

---

## PR 8 · US9 — El acento de Windows, apagado de fábrica (2026-09-06) · modo plan

ADR-035: la herencia del acento de Windows deja de ser el comportamiento de fábrica. Toca `src-tauri/`
(un valor de fábrica). **No añade comando, permiso, dependencia ni clave de `settings`.**

- [x] Backend: `get_appearance_settings_impl` — valor de fábrica de `use_system_accent` `true` → **`false`**. Prueba nueva: conexión sin claves → `use_system_accent == false`; y `el_acento_del_sistema_es_un_booleano_simple` cubre ahora los dos sentidos (`false` y `true`).
- [x] `settings/+page.ts`: el `load` añade `getAppearanceSettings()` (en paralelo con `getSettings()`).
- [x] `settings/+page.svelte`: `useSystemAccent` arranca en `false` y toma el valor real del `load` en el mismo `$effect` que rellena `settings` (evita el aviso `state_referenced_locally`). `cambiarAcentoSistema` ya persistía `settings.appearance.use_system_accent` y llamaba a `applySystemAccent()`/`clearSystemAccent()` — sin cambios.
- [x] Texto reescrito (es + en): `settings.appearance.useSystemAccent.label` → «Usar el color de acento de Windows» / «Use the Windows accent colour»; `.hint` → «Sustituye el morado de la aplicación por el color que tengas configurado en Windows.» / «Replaces the app's purple with the colour configured in Windows.» (literal de `RESUMEN.md`).
- [x] `applySystemAccent()`/`clearSystemAccent()` sobrescriben/restauran **cinco** propiedades CSS (los 3 roles de acento + `-hi` + `-soft`) — la redacción «tres tokens» del ADR-035 se precisó a «tres roles / cinco propiedades». `refreshAccentForTheme()` sigue enganchado al cambio de tema.
- [x] Pruebas: `accent.test.ts` (+2, acento claro `#ffb900` de fondo y de texto, AA en los 2 temas); `e2e/ui/settings.spec.ts` (+1: apagado de fábrica → activar persiste `value: true` y sobrescribe `--sdm-accent` con un par que cumple AA ≥ 4,5 → desactivar persiste `value: false` y quita la sobreescritura inline). Fixture `apariencia.useSystemAccent`: `true` → `false`.
- [x] Puerta: `cargo test` 460 ✓ · `clippy -D warnings` ✓ · `fmt` ✓ · `pnpm check` 0/0 · `verify` ✓ (358 claves) · `lint` ✓ · `test` 228 · `test:component` 89 · `test:e2e` 82 · `a11y` 14 (`/settings` con Ciruela, claro y oscuro).

### Checklist §8 — Ajustes con el acento del sistema

- [x] Con el acento de respaldo (Ciruela): `a11y` (axe) verde en claro y oscuro; `verify:tokens` verde.
- [x] Con un acento del sistema claro (`#ffb900`): el e2e comprueba que `--sdm-accent` / `--sdm-on-accent` cumplen AA ≥ 4,5:1 tras activar el interruptor (`accessibleAccent()` corrige lo que haga falta). El camino de texto (`accentOnSurface()`) queda cubierto por `accent.test.ts` sobre `LIGHT` y `DARK`.
- [ ] **Pendiente de revisión manual del usuario** (`pnpm app:dev`): instalación nueva → morado Ciruela; activar con un acento de Windows claro real → botón primario, enlaces y selección legibles en los dos temas; desactivar → vuelve el morado; reiniciar → persiste.

### Fuera de alcance

- **Nota al pie de la constitución §VI** (ADR-035 la menciona: «la herencia es opcional»). `.specify/memory/constitution.md` está protegida por hook y exige autorización explícita del usuario — es el mismo **T007** que sigue pendiente. No se toca en este PR.
- La clave i18n huérfana `settings.useSystemAccent` (sin `.label`/`.hint`, línea 71 de los diccionarios) ya trae el texto nuevo pero **no la usa nadie**; `verify:i18n` no marca claves sin usar. Se deja para una limpieza aparte, fuera del alcance de v3.

---

## PR 9 · US8 — Asistente inicial `/onboarding` (2026-09-06) · modo plan

Cubre US-002 del producto por primera vez. Toca `src-tauri/` (2 claves de `settings`, el motor de
notificaciones, `platform/autoarranque.rs`) y `src/routes/+layout.ts` (guardián). **Dos ADR nuevos**
(ADR-037 notificaciones, ADR-038 autoarranque). Sin dependencia, comando ni permiso nuevo.

### Backend

- [x] `notifications.enabled` (fábrica `true`) y `lifecycle.start_with_system` (fábrica `false`) en `SettingsWire` + Zod + `claves_por_ambito("all")` + prueba de rechazo. `reset_settings("all")` reconcilia la tarea programada.
- [x] `alerts::notificaciones`: `debe_enviar(...)` puro con 5 pruebas; `procesar_una` respeta `notifications.enabled`.
- [x] `platform/autoarranque.rs`: `aplicar(bool)` vía `schtasks /Create /SC ONLOGON /RL HIGHEST` · `/Delete`; solo código de salida; `#[cfg(not(windows))]` no-op. Prueba del formato de la línea de comando y del nombre de la tarea. **`schtasks` real no se ejecuta en `cargo test`** — verificación manual (`open-questions.md` §V.3).
- [x] Puerta: `cargo test` **468** ✓ · `clippy -D warnings` ✓ · `fmt` ✓.

### Guardián de arranque (`+layout.ts`)

- [x] `completedAt` nulo + sin configuración previa observable → `redirect(307, "/onboarding")` (FR-032). Con configuración previa (tema/idioma/perfil/alias/exclusión) → graba `completed_at` y sigue (FR-043). Cualquier fallo del backend → seguir sin redirigir (`open-questions.md` §V.2).
- [x] `pnpm build` en verde: el `load` del layout no se ejecuta al prerenderizar (`ssr = false`).
- [x] e2e: redirige / no redirige con configuración previa / «Repetir» pone la marca a null.

### Asistente `/onboarding` (§8)

- [x] Solo tokens; cero literales (`verify:tokens` verde). `max-[860px]:hidden`, `min-h-screen`, `max-w-[1000px]` son utilidades.
- [x] **Sin `AppShell`**: `+layout.svelte` omite riel y barra de herramientas en `/onboarding` (FR-033).
- [x] Correcto en claro y oscuro (`a11y.spec.ts` → `/onboarding` en los 2 temas, axe verde — se corrigió el contraste de la garantía: icono en `text-ok`, texto en color normal).
- [x] Ventana mínima 1024×560: contenedor a 100 % con `padding` 24 px; el indicador de 4 puntos colapsa a «Paso N de 4» por debajo de 860 px; el pie es `sticky bottom-0`.
- [x] Textos en `es` y `en` (`verify:i18n` — 400 claves sincronizadas). «Omitir y usar los valores de fábrica» visible en los 4 pasos.
- [x] Estados: **cargando** (`+page.ts` bloquea), **vacío** (`EmptyState kind="empty"` + «Volver a buscar»; el primario del pie permite continuar), **no compatible** (USB con `StatusPill` gris + bloque «no es una avería» sin rojo), **error de fuente** (`EmptyState kind="error"` con `<details>` + «Reintentar»; «Omitir» siempre).
- [x] Dato ausente: `CapacityBar` y `StatusPill` heredan su tratamiento; el disco sin SMART no se pinta en rojo.
- [x] Teclado y foco: navegación por `Button`, radios de perfil `role="radio"`, casillas `<input type=checkbox>` con `aria-label`; el `<details>` es nativo.
- [x] Nada irreversible: «Repetir la configuración inicial» solo pone la marca a null (FR-037).
- [x] Pruebas: `e2e/ui/onboarding.spec.ts` (6), `a11y.spec.ts` (+2). `perfiles.ts` compartido con Ajustes.
- [x] Puerta: `verify` ✓ · `check` 0/0 (600 ficheros) · `lint` ✓ · `test` 229 · `test:component` 89 · `test:e2e` 88 · `a11y` 16 · `docs:check` ✓.
- [ ] **Pendiente de revisión manual del usuario** (`pnpm app:dev`): primer arranque → asistente; recorrer y «Ir al panel»; reiniciar → no reaparece; Ajustes → «Repetir…» → reaparece con los valores actuales; activar «Arrancar con el sistema» → comprobar la tarea en el Programador con «privilegios más altos»; con «Avisarme…» apagado, forzar una alerta → sin toast.

### Deuda / fuera de alcance

- **T007** (nota al pie de la constitución §VI, ADR-035): sigue pendiente de autorización explícita del usuario. No entra aquí.
- **Fase 13 (Polish)** T121–T128: cierre transversal de la feature, después de este PR.
- Prueba de componente del `+page.svelte` del asistente: no se hace (necesita simular `$app/navigation`); el nivel e2e cubre los estados a menor coste.

---

## Fase 13 · Cierre — verificación manual pendiente (usuario)

La puerta automática entera está verde (`cargo test` 468, `clippy`, `check` 0/0, `verify`,
`lint`, `test` 229, `test:component` 89, `test:e2e` 95, `a11y` 16, `escalado` 40, `docs:check`).
Lo que **solo** puede comprobar una persona con la app compilada y elevada (`pnpm app:build`,
UAC):

- [ ] Recorrer `quickstart.md` entero (un escenario por historia) sobre el instalador real.
- [ ] Las 8 pantallas + el asistente en **claro y oscuro** contra `mockups/smartdisk-v3.html`.
- [ ] El riel de 74 px y el estado global (píldora de la barra = pie del riel).
- [ ] La gráfica del detalle con una serie real con hueco: trazo, eje Y y banda.
- [ ] Perfil de alerta: elegir «Prudente» en Ajustes → los 12 campos y la temperatura cambian;
      editar uno → «Personalizado (a partir de Prudente)».
- [ ] Acento de Windows: instalación nueva → morado Ciruela; activar con un acento claro real →
      botón primario y enlaces AA en los dos temas; desactivar → vuelve el morado.
- [ ] Asistente: primer arranque → aparece; «Ir al panel» → no reaparece; «Repetir…» desde
      Ajustes → reaparece con los valores actuales.
- [ ] Autoarranque: activarlo → en el Programador de tareas existe «SmartDisk Monitor - Autostart»
      con «privilegios más altos» y disparador «al iniciar sesión»; reiniciar → abre elevada sin
      UAC; desactivar → desaparece.
- [ ] `notifications.enabled` a `false` → forzar una alerta → no sale toast (pero sí entra en la
      lista y cuenta para el color).
- [ ] Icono provisional de «Acerca de» (`tag`) y de «horas encendido» (`clock`): confirmar o
      cambiar (son props, sin tocar estructura).

## Pendiente con autorización del usuario

- **T007** — nota al pie de la constitución §VI apuntando a ADR-035. `.specify/memory/constitution.md`
  está protegida por hook; además tiene cambios sin commitear de K.6 (no del rediseño). Requiere
  que el usuario lo pida explícitamente.
- **Archivar `design/propuesta-rediseno/`** (v1, superada). Sin trackear por ahora.
