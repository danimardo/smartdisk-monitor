# Implementation Plan: Rediseño visual «SmartDisk Monitor v3»

**Branch**: `002-rediseno-v3` | **Date**: 2026-09-06 | **Spec**: [spec.md](./spec.md)

**Input**: `specs/002-rediseno-v3/spec.md` + entregable del diseñador en `design/propuesta-redisenov2/`

## Summary

Aplicar el paquete de rediseño v3: paleta «Ciruela» y tokens nuevos, tres componentes de catálogo (`Icon`, `Sparkline`, `HeroPanel`), cinco recompuestos, chrome reducido a un riel de 74 px, las ocho pantallas recompuestas, el asistente inicial (US-002) por primera vez, y —por decisión del usuario— los perfiles de alerta con alcance completo (nuevos umbrales configurables consumidos por el motor de alertas de Rust).

**Enfoque técnico**: cambio dirigido por el sistema de diseño. La fuente de verdad visual sigue siendo `src/design-system/tokens.css`; todo lo demás lo hereda por tokens y por el catálogo. La entrega se parte en **nueve PR** ordenados por dependencia, cada uno dejando `pnpm verify && pnpm check && pnpm test && pnpm test:component` en verde y pasando la definición de terminado de `docs/ui-design.md` §8 para las pantallas que toca. El bloque de backend (perfiles de alerta, PR 5) va precedido de modo plan y lleva su propio ADR.

## Technical Context

**Language/Version**: TypeScript 5 / Svelte 5 (runes) en el frontend; Rust (edición fija del proyecto) en `src-tauri/`. Versiones exactas en `docs/engineering-conventions.md` y en la constitución §«Pila y versiones» — **no se cambian**.

**Primary Dependencies**: SvelteKit con `adapter-static` y `ssr = false`; Tailwind mapeado 1:1 sobre tokens; Zod para validar la frontera IPC; Tauri 2. **Ninguna dependencia nueva** (constitución §III). En particular: **sin segunda familia tipográfica** (decisión del usuario, opción «b») → no se añade Bricolage Grotesque ni se toca `THIRD_PARTY_NOTICES.md` por fuentes.

**Storage**: SQLite en `%ProgramData%\SmartDisk Monitor\`, tabla `settings`. Los valores nuevos (umbrales de perfil, `settings.alerts.profile`, `settings.onboarding.completedAt`) se persisten con el comando genérico `set_setting` ya existente, ampliando su lista blanca de claves. **Sin comandos Tauri nuevos, sin permisos nuevos, sin migración de esquema de tablas** (la tabla `settings` es clave/valor JSON).

**Testing**: `vitest` en dos configuraciones (Node y Browser Mode/Chromium), Playwright para e2e y a11y, `cargo test` en Rust. Test-first obligatorio en el motor de alertas (constitución §VIII): las reglas nuevas (desgaste configurable, errores de medios/24 h, reintentos de controlador/24 h) se escriben rojo→verde con sus cinco pruebas (activación, no-activación ante dato ausente, histéresis, deduplicación, ciclo de recaída).

**Target Platform**: Windows x64 dentro de WebView2. Ventana mínima 1024 × 560, objetivo de diseño 1280 × 720. Debe seguir correcta al 125 %, 150 % y 200 % de escalado.

**Project Type**: Aplicación de escritorio (Tauri + SvelteKit estático). Estructura de tres capas: Presentación (`src/routes`, `src/lib/components`) → Acceso (`src/lib/api`) ↔ Comandos (`src-tauri/src/commands`) → Dominio (`src-tauri/src/domain`, `src-tauri/src/alerts`).

**Performance Goals**: SC-007 del producto — ninguna tarea de render por encima de 50 ms; el panel se pinta de inmediato y las sparklines llegan por carga perezosa por tarjeta visible; con 20 discos y 5.000 eventos, sin bloqueo perceptible al desplazarse.

**Constraints**: cero red; cero valores visuales literales fuera de `tokens.css`; cero literales de interfaz fuera de `es.json`/`en.json`; una sola copia del sistema de diseño; `:focus-visible` global inviolable; contraste AA en los dos temas sobre el material compuesto.

**Scale/Scope**: 8 pantallas + 1 nueva (asistente, 4 pasos) + chrome; catálogo pasa de N a N+3 componentes; ~35 claves i18n nuevas; ~7 claves de `settings.alerts` nuevas + 2 de otras secciones; 9 PR.

## Constitution Check

*GATE: debe pasar antes de Phase 0. Reevaluado tras Phase 1.*

| Principio | Aplicación a esta feature | Estado |
|---|---|---|
| **I. Veracidad del dato** | El rediseño refuerza el principio: dato ausente pasa de «No disponible» a 20 px a «—» discreto (sigue sin inventar cero); huecos de serie se pintan como banda y nunca se interpolan (`Sparkline` FR-012, `TimeSeriesChart` FR-013); no compatible sigue en gris y sin alerta (FR-049). | ✅ PASA |
| **II. Orden de prioridades** | La carga perezosa de sparklines (FR-044) prioriza «la aplicación responde» (3) sobre «completitud» — el panel pinta sin esperar. La claridad (4) es el objetivo declarado del héroe. | ✅ PASA |
| **III. Pila fija** | Sin React, sin librería de componentes, **sin dependencias nuevas**, sin fuente nueva (opción «b»), sin red. Los 3 componentes nuevos siguen `AGENTS.md` §3 (cada uno con su justificación en `cambios/componentes/`). | ✅ PASA |
| **IV. Dominio/presentación** | `selectHeroDisk()` y el cálculo de estado global viven en `$lib/design/health.ts`, no en componentes (FR-014, FR-024). El motor de alertas consume umbrales de `settings`, la UI no recalcula reglas (FR-047). Ningún componente decide colores: `healthToken`/`healthIcon` son la única fuente. | ✅ PASA |
| **V. Persistencia** | Todo en `settings` (SQLite), nada en `localStorage`. Sin migración de tablas. Bytes/°C en el dato, formato en presentación. | ✅ PASA |
| **VI. Sistema de diseño vinculante** | El objeto de la feature. `docs/ui-design.md` se actualiza a v3 conservando §4/§6/§8. Cero literales verificado por `pnpm verify:tokens` en cada PR. Una sola copia. El acento deja de heredarse **por defecto** — esto **modifica una regla de §VI** («El acento… se hereda de Windows»): requiere ADR que enmiende ADR-013 y ADR-017, y una nota en la constitución. Ver Complexity Tracking. | ⚠️ REQUIERE ADR |
| **VII. Accesibilidad AA** | Contraste medido sobre el material compuesto para toda la paleta (FR, SC-003); iconos con `aria-label` cuando son únicos portadores; `role="img"` + lectura textual en gráficas; foco visible intacto; objetivos de 44 px en el riel (por encima del mínimo). El velo de legibilidad del héroe garantiza AA con cualquier forma de curva. | ✅ PASA |
| **VIII. Testeabilidad** | Test-first en las reglas de alerta nuevas (motor). Cobertura de estados de los 3 componentes nuevos y los 5 recompuestos. Cobertura mínima de `domain/alerts` (90 %) se mantiene con las pruebas de las reglas nuevas. | ✅ PASA (con disciplina test-first en PR 5) |
| **IX. Seguridad** | Sin comandos nuevos, sin permisos nuevos, sin ampliar la superficie del binario. El sprite se monta como marcado estático sin `<metadata>`. Contenido de eventos sigue como texto. | ✅ PASA |
| **X. Errores** | Un fallo de `getMetricSeries` por disco degrada solo esa sparkline/tarjeta (FR local). El asistente nunca deja al usuario encallado (FR-038). Guardar un umbral fuera de rango devuelve `AppError` y revierte el control (US10 escenario 4). | ✅ PASA |
| **XIV. SvelteKit idiomático** | El asistente usa `load` universal en `+page.ts`/`+layout.ts` para el guardado de `completedAt`; navegación con `<a href>`, no `goto()` en `onclick`; `$derived` antes que `$effect`. La redirección de arranque va en `+layout.ts` load (redirección programática, permitida). | ✅ PASA |

**Veredicto de la puerta**: PASA con una condición — el cambio de la regla «el acento se hereda de Windows» (§VI) exige ADR antes de implementar la parte de acento (PR 8). No bloquea las fases anteriores.

### Reevaluación tras Phase 1 (diseño)

Los artefactos de diseño (`research.md`, `data-model.md`, `contracts/`) no introducen ninguna violación nueva:

- **Sin dependencias, sin comandos, sin permisos, sin tablas nuevas** — confirmado en `contracts/settings-alerts.md` (todo va por `set_setting` genérico) y `contracts/componentes-nuevos.md` (3 componentes de catálogo, ninguno de terceros).
- **§I** reforzado: los contratos de `Sparkline` y `HeroPanel` codifican «un hueco es un hueco» y «dato ausente ≠ cero».
- **§VIII**: `data-model.md` §3.2 y `contracts/settings-alerts.md` exigen prueba de rechazo por cada clave nueva y las 5 pruebas por regla de alerta.
- **Puntos abiertos del diseño, resueltos por `/speckit-clarify`** (2026-09-06): D12 (semántica de `mediaErrors*`/`driverRetry*` → parametrizar reglas existentes, sin conteo por 24 h); temperatura de fábrica → 60/70 °C; disco de sistema → `VolumeSummary.isSystemVolume` nuevo; guardián del asistente → comprobación en `+layout.ts` sin migración. Todos integrados en `spec.md` §Clarifications.
- El ADR del acento (ADR-035) se clasifica como **precisión del mecanismo**, no enmienda de principio: el acento sigue sin comunicar salud, sigue siendo acción/selección y sigue corregido si se hereda. Se añade una nota al pie de §VI remitiendo al ADR.

**Veredicto post-diseño**: PASA. Una condición antes de programar: ADR-034/ADR-035 redactados en el PR 1. `VolumeSummary.isSystemVolume` (Zod + ts-rs + prueba) entra en el PR 5 (adelantado, ya toca `src-tauri` y regenera tipos) o al principio del PR 6.

## Project Structure

### Documentation (this feature)

```text
specs/002-rediseno-v3/
├── plan.md              # Este fichero
├── research.md          # Phase 0 — decisiones técnicas
├── data-model.md        # Phase 1 — claves de settings, mapas, i18n
├── quickstart.md        # Phase 1 — guía de validación por historia
├── contracts/
│   ├── settings-alerts.md      # Delta del esquema settings.alerts + set_setting
│   ├── componentes-nuevos.md   # Props de Icon, Sparkline, HeroPanel
│   └── i18n-claves.md          # Claves nuevas es/en
├── checklists/
│   └── requirements.md  # (ya existe)
└── tasks.md             # Phase 2 — lo genera /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── design-system/
│   ├── tokens.css               # PR1: paleta Ciruela, tokens display/riel/héroe/icono, .sdm-display
│   └── tokens.json              # PR1: espejo
├── app.css                      # sin cambios previstos
├── lib/
│   ├── design/
│   │   ├── health.ts            # PR3/PR4: + selectHeroDisk(); estado global una sola vez
│   │   ├── icons.ts             # PR2: NUEVO — healthIcon, eventLevelIcon, busIcon, testIcon
│   │   └── format.ts            # PR4/PR6: helper "—" para dato ausente compacto
│   ├── components/
│   │   ├── Icon.svelte          # PR2: NUEVO
│   │   ├── Sparkline.svelte     # PR4: NUEVO
│   │   ├── HeroPanel.svelte     # PR6: NUEVO
│   │   ├── Sidebar.svelte       # PR3: riel de 74 px
│   │   ├── Toolbar.svelte       # PR3: título de ruta, píldora con icono, sin "?"
│   │   ├── TimeSeriesChart.svelte  # PR4: usa Sparkline, eje Y, relleno, banda de hueco
│   │   ├── DiskCard.svelte      # PR6: cabecera con sparkline
│   │   ├── MetricCard.svelte    # PR6: icono + sparkline, fuera peso 800
│   │   ├── StatusPill.svelte    # PR2: ranura de icono
│   │   ├── ProgressBar.svelte   # PR7: prop emphasis
│   │   ├── Button.svelte        # PR1: text-fg-onAccent
│   │   ├── AlertCard.svelte     # PR7: severidad con icono
│   │   ├── EventRow.svelte      # PR7: nivel como cuadrado con icono
│   │   ├── AppShell.svelte      # PR2: monta el sprite
│   │   └── index.ts             # PR2/PR4/PR6: exporta Icon, Sparkline, HeroPanel
│   ├── i18n/{es,en}.json        # PR1..PR9: claves nuevas por fase
│   └── api/
│       ├── schemas.ts           # PR5: settings.alerts delta (Zod)
│       └── generated/*.ts       # PR5: regenerados desde ts-rs
└── routes/
    ├── +layout.svelte / +layout.ts  # PR3/PR8: chrome, redirección de arranque al asistente
    ├── +page.svelte / +page.ts      # PR6: panel general recompuesto + carga perezosa de series
    ├── disks/[id]/+page.svelte      # PR6: detalle recompuesto, intervalo junto a la gráfica
    ├── alerts|events|reports/+page.svelte   # PR7: tokens + estructura interna
    ├── settings/+page.svelte        # PR7/PR5/PR8: sección destructiva, perfiles de alerta, acento
    ├── tests/+page.svelte           # PR7: prueba en curso con cifra de display
    └── onboarding/+page.svelte      # PR9: asistente de 4 pasos

src-tauri/src/                   # SOLO PR5 (modo plan)
├── commands/                    # ampliar lista blanca de claves de set_setting
├── domain/
│   ├── ajustes.rs               # validación de rangos de los umbrales nuevos
│   ├── salud.rs / espacio.rs    # leer umbrales de settings en vez de constantes
│   └── tipos.rs                 # AlertsSettings + AlertProfile (ts-rs)
└── alerts/motor.rs              # consumir umbrales de desgaste, errores de medios/24h, reintentos/24h

docs/                            # actualizaciones normativas
├── ui-design.md                 # PR1: v3
├── decisions.md                 # PR1: ADR v3 + ADR acento · PR5: ADR motor parametrizado
├── ui-contract.md               # PR5: claves settings.alerts · PR9: settings.onboarding.completedAt
├── alert-rules.md               # PR5: reglas parametrizadas por perfil
├── data-model.md                # PR5: claves y límites
├── open-questions.md            # PR1: ratios Ciruela medidos · PR6: medición carga perezosa
└── user-stories.md              # PR9: US-002 marcada como cubierta (regenerar historias.md)
```

**Structure Decision**: se mantiene la estructura de tres capas y el catálogo cerrado. Los tres componentes nuevos entran en `src/lib/components/` con su export en el barrel. `$lib/design/icons.ts` es el único sitio nuevo de lógica de presentación. El backend solo se toca en PR 5.

## Fases de entrega (PR)

| PR | Historia(s) | Alcance | Puerta |
|---|---|---|---|
| **1 · Tokens y base v3** | US1 | `tokens.css` + `tailwind.config.cjs` + `tokens.json`: paleta Ciruela, `--sdm-on-accent` tinta en oscuro, tokens display/riel/héroe/icono, `.sdm-display`. `Button` → `text-fg-onAccent`. Barrer `text-white` sobre acento en todo el catálogo. `docs/ui-design.md` a v3. **ADR v3 + ADR acento** (redacción; la implementación del interruptor es PR 8). Ratios medidos → `open-questions.md`. | `verify` + `check` verdes; captura visual de las 8 pantallas en los 2 temas sin regresión de contraste |
| **2 · Iconografía** | US2 | `Icon.svelte`, sprite en `AppShell` (sin `<metadata>`), `$lib/design/icons.ts` con los 4 mapas. `StatusPill` gana ranura de icono. Hoja de contacto como referencia de revisión. | `verify` + `check`; prueba de estados de `Icon`; a11y del `label` obligatorio |
| **3 · Chrome** | US3 | `Sidebar` → riel de 74 px (fuera lista de discos, texto de estado, pausa). `Toolbar` → título de ruta, píldora con icono única fuente, sin «?». Estado global una sola vez (`worstState` sobre monitorizados) — **corrige los 2 bugs de chrome**. Pausa → Ajustes + bandeja. | e2e de navegación (título cambia con la ruta); a11y del riel; captura a 1024×560 |
| **4 · La gráfica vuelve** | US4 | `Sparkline.svelte` (trazo por tramos, non-scaling-stroke, submuestreo). `TimeSeriesChart` usa `Sparkline` + eje Y + relleno + banda de hueco + umbral — **corrige el bug de gráfica vacía y la serie de dos tramos**. | prueba de componente con serie con hueco; e2e del detalle; a11y de la gráfica |
| **5 · Perfiles de alerta (backend)** | US10 | **MODO PLAN.** `settings.alerts` + claves nuevas (Zod + ts-rs + serde + rangos + pruebas de rechazo). `settings.alerts.profile`. Lógica «perfil → custom». Motor de alertas consume los umbrales nuevos (**test-first**, 5 pruebas por regla). Ajustes → sección de perfiles. `docs/alert-rules.md` + `ui-contract.md` + `data-model.md`. **ADR motor parametrizado por perfil.** | `cargo test` + `cargo clippy -D warnings`; cobertura `domain/alerts` ≥ 90 %; `verify:i18n` |
| **6 · Panel y detalle** | US5, US6 | `HeroPanel.svelte` + `selectHeroDisk()`. Panel: héroe + rejilla + «Reparto de estados» + «Sucesos». **Carga perezosa** de series por tarjeta visible → medición a `open-questions.md`. `DiskCard`/`MetricCard` recompuestos (fuera peso 800). Detalle: cabecera de identidad, 4 `MetricCard`, intervalo junto a la gráfica, `DataRow` con delta. | prueba de estados de `HeroPanel`/`DiskCard`/`MetricCard`; e2e panel con 4 y con 20 discos (SC-006); §8 |
| **7 · Pantallas secundarias** | US7 | `ProgressBar` prop `emphasis`. Pruebas: bloque en curso con cifra a 58 px. Alertas/Eventos: `AlertCard`/`EventRow` con icono. Informes: solo tokens. Ajustes: «Borrado de datos» separada con borde crítico. | prueba de estados de `ProgressBar`/`EventRow`/`AlertCard`; §8 de cada pantalla |
| **8 · Acento opcional** | US9 | Interruptor en Ajustes → Apariencia; `useSystemAccent` default → `false`; rótulo y `hint` nuevos; `applySystemAccent()`/`clearSystemAccent()` cableados. | e2e con acento claro de prueba en los 2 temas (SC-003); `accent.test.ts` |
| **9 · Asistente inicial** | US8 | Ruta `/onboarding`, 4 pasos, cabecera propia, «Omitir» en cada paso. Guardián de arranque en `+layout.ts`. `settings.onboarding.completedAt` en la lista blanca de `set_setting` + `ui-contract.md`. «Repetir configuración inicial» en Ajustes. Instalaciones existentes → completadas. `user-stories.md` + `historias.md`. | e2e del flujo completo (SC-007); estados del asistente; `verify` |

Dependencias: PR1 es prerequisito de todo. PR2 antes de PR3/PR4/PR6/PR7/PR9. PR4 antes de PR6. PR5 antes de PR9 (el paso 3 usa los perfiles). PR8 y PR9 son independientes entre sí.

## Complexity Tracking

| Desviación | Por qué es necesaria | Alternativa más simple, y por qué se descarta |
|---|---|---|
| **Modificar la regla de §VI «el acento se hereda de Windows»** (pasa a opt-in apagado) + enmendar ADR-013 y ADR-017 | La dirección de diseño aprobada da identidad propia a la aplicación; el azul heredado es indistinguible de cualquier utilidad de Windows y en oscuro aplana el conjunto. El usuario ha aprobado la dirección. | Mantener la herencia por defecto y ofrecer Ciruela solo como tema alternativo: descartado porque duplica el mantenimiento de dos identidades y deja el problema original (no se reconoce la app) sin resolver para la mayoría. Se conserva `accessibleAccent()`/`accentOnSurface()` intactos para quien active el interruptor. |
| **Perfiles de alerta = trabajo de backend dentro de un rediseño de presentación** | Decisión explícita del usuario (opción A). Sin umbrales configurables reales, el paso 3 del asistente sería decorativo. | Alcance reducido (solo temperatura + capacidad) o sin paso 3: descartados por el usuario. Se aísla en el PR 5 con su ADR y su modo plan para que no contamine el resto de la entrega. |
| **`--sdm-font-display` = `--sdm-font-sans`** (token que hoy no aporta variación) | Deja `.sdm-display` como único punto de cambio si en el futuro se decide empaquetar una segunda familia, sin volver a tocar 6 componentes. | No crear el token y usar `.sdm-font-sans` directamente: descartado porque el entregable y `ui-design.md` v3 razonan sobre «tipografía de display» como concepto; el token documenta la intención aunque hoy no varíe. |
| **Carga perezosa de series en el panel** (patrón de datos nuevo en `+page.svelte`) | Cargar 20 series en el `load` bloquea la primera pintura y rompe SC-007. | Cargar en `load`: descartado (medido como problema en la spec). Comando agregado: descartado (comando nuevo, contra el alcance). |
