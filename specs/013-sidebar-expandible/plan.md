# Implementation Plan: Riel de navegación expandible con etiquetas de texto

**Branch**: `013-sidebar-expandible` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/013-sidebar-expandible/spec.md`

## Summary

Un botón en el propio riel de navegación alterna un panel más ancho (~232 px) que muestra icono +
nombre de cada sección, superpuesto sobre el contenido (nunca lo empuja ni lo encoge) para no
comprometer el mínimo de ventana de 1024 px que `ADR-034` ya protegió. Se cierra con el mismo
botón, con `Escape` o pulsando fuera; el foco entra al panel al abrir y vuelve al botón al cerrar,
mismo patrón que `ConfirmDialog`/`ExplicacionModal`/`AboutDialog`, pero sin velo que oscurezca el
resto de la pantalla (no es un diálogo bloqueante). El estado se recuerda entre sesiones como un
campo más de `AppearanceSettings`, junto a tema e idioma.

## Technical Context

**Language/Version**: TypeScript 5.9.3 (Svelte 5.57.0 con runas, SvelteKit 2.70.3) + Rust 1.94.0
(edición 2021). Sin cambios de versión de ninguna pieza de la pila.

**Primary Dependencies**: ninguna nueva. Reutiliza `getAppearanceSettings`/`setSetting` (ya
existentes) y los tokens de material ya existentes (`sdm-material-overlay`, `--sdm-shadow-lift`).

**Storage**: SQLite vía la tabla `settings` genérica (clave/valor), mismo mecanismo que
`settings.appearance.use_system_accent` — sin migración: una clave nueva en una tabla ya
genérica no necesita cambio de esquema.

**Testing**: `pnpm check`; `pnpm test` (Rust: lectura del nuevo campo con su valor por defecto);
`pnpm test:component` (`Sidebar.browser.test.ts`: abrir/cerrar, foco, `Escape`, clic fuera,
navegar con un enlace real); `pnpm test:e2e` (recorrido completo: expandir, ver las etiquetas,
navegar, y que la preferencia sobrevive a una recarga simulada); `cargo test` (el nuevo campo de
`AppearanceSettings` y su lectura con valor por defecto `false`).

**Target Platform**: Windows de escritorio (sin cambio).

**Project Type**: aplicación de escritorio, proyecto único ya establecido.

**Performance Goals**: ninguno nuevo — es una superposición de CSS, sin nueva petición de red ni
recálculo de datos.

**Constraints**: **ninguna dependencia ni permiso de Tauri nuevos**. El único campo de datos nuevo
es un booleano en una respuesta de comando ya existente (`get_appearance_settings`), leído/escrito
con el mecanismo `set_setting(key, value)` genérico que ya usan tema, idioma y acento — no es un
comando nuevo, así que no hace falta un permiso nuevo de *capabilities*. Aun así, toca
`src-tauri/src/commands/mod.rs`: se sigue "modo plan" de Claude Code antes de escribir ese cambio,
según las instrucciones del proyecto para esa carpeta.

**Scale/Scope**: `Sidebar.svelte` (el cambio principal), `+layout.svelte` (estado del panel +
persistencia), `commands/mod.rs` (campo nuevo), el `.ts` generado por `ts-rs`, el esquema Zod de
`AppearanceSettings`, dos claves i18n nuevas (abrir/cerrar el panel) en los dos diccionarios,
`docs/ui-contract.md` (el campo nuevo) y una entrada en `docs/decisions.md` explicando por qué esto
no reabre el problema que cerró `ADR-034`.

## Constitution Check

*GATE: debe pasar antes de la fase 0. Repetido tras el diseño de la fase 1 — ver el final de esta
sección.*

| Principio | Aplica | Cómo se cumple |
|---|---|---|
| I. Veracidad del dato | Sí | El campo nuevo tiene un valor de fábrica explícito (`false`, plegado) — nunca se infiere ni se deja `undefined`. |
| III. Pila fija | Sí | Cero dependencias nuevas, cero permisos de Tauri nuevos. |
| IV. Dominio/presentación | Sí | `Sidebar.svelte` sigue sin invocar `$lib/api`; el estado y la llamada a `setSetting` viven en `+layout.svelte`, que ya es quien orquesta apariencia (tema, idioma, acento). |
| V. Persistencia | Sí | Nueva clave en la tabla `settings` genérica, mismo mecanismo que `use_system_accent`; sin `localStorage`, sin migración de esquema. |
| VI. Sistema de diseño | Sí | Reutiliza `sdm-material-overlay` y `--sdm-shadow-lift` ya existentes; cero valores visuales literales nuevos; los nombres de sección son los mismos textos ya traducidos (`nav.*`), sin duplicar copy. |
| VII. Accesibilidad AA | Sí | Foco al abrir, `Escape` para cerrar, foco de vuelta al botón al cerrar (mismo patrón que los diálogos existentes); tamaño de objetivo ya cumplido (44 px, herencia del riel); el botón de alternar lleva `aria-expanded` y `aria-label` según el estado. |
| VIII. Testeabilidad | Sí | Cobertura de estados del componente (plegado, expandido, con foco, cerrado por `Escape`/clic fuera) vía Playwright/component tests; el campo nuevo de Rust con su valor por defecto. |
| IX. Seguridad | Sí | Ningún permiso de Tauri nuevo: `set_setting`/`get_appearance_settings` ya existen y ya están expuestos. |
| XI. Validación de fronteras | Sí | El campo nuevo se añade al esquema Zod `appearanceSettings` (validado en ejecución), con su prueba de rechazo para un valor que no sea booleano. |
| XIII. Validación de tipos primero | Sí | `pnpm check` en cero errores y cero avisos. |
| XIV. Arquitectura SvelteKit/Tauri | Sí | Navegación por `<a href>` dentro del panel (nunca `goto()` en `onclick`); `$derived`/`$effect` con el mismo patrón que `ConfirmDialog` para el foco; la preferencia se actualiza tras confirmar el comando, nunca antes (aunque aquí, al ser una preferencia de UI sin riesgo de dato incorrecto, se refleja de inmediato en la interfaz igual que hace `theme.svelte.ts`/`i18n.svelte.ts`, y `set_setting` se dispara en paralelo). |

**Resultado**: PASA, sin excepciones y sin `Complexity Tracking`.

*Repetido tras la fase 1 (`data-model.md`, `contracts/`, `quickstart.md`): sin cambios — el diseño
no introduce ningún comando, permiso ni dependencia que no estuviera ya previsto aquí.*

## Project Structure

### Documentation (this feature)

```text
specs/013-sidebar-expandible/
├── plan.md              # Este fichero
├── research.md          # Fase 0: decisiones D1-D6
├── data-model.md        # Fase 1: el campo nuevo de AppearanceSettings + estado de pantalla
├── contracts/
│   └── ajustes-apariencia.md   # Fase 1: delta sobre docs/ui-contract.md §3.1
├── quickstart.md        # Fase 1: guía de validación
└── tasks.md             # Fase 2 (/speckit-tasks — no la crea este comando)
```

### Source Code (repository root)

```text
src-tauri/src/
└── commands/mod.rs             # AppearanceSettings: + sidebar_expanded; su clave y su lectura

src/
├── lib/
│   ├── components/
│   │   └── Sidebar.svelte      # + botón de alternar, panel expandido, foco/Escape/clic fuera
│   ├── api/
│   │   ├── schemas.ts          # + sidebarExpanded en appearanceSettings
│   │   ├── schemas.test.ts     # + prueba de rechazo del campo nuevo
│   │   └── generated/
│   │       └── AppearanceSettings.ts   # regenerado por ts-rs (cargo test)
│   └── i18n/
│       ├── es.json              # + nombres accesibles de abrir/cerrar el panel
│       └── en.json
└── routes/
    └── +layout.svelte           # estado `expanded`, inicializado desde AppearanceSettings,
                                  # onToggleExpand llama a setSetting

docs/
├── ui-contract.md               # §3.1: sidebarExpanded en AppearanceSettings
└── decisions.md                 # nota/enmienda: por qué esto no reabre ADR-034
```

**Structure Decision**: proyecto único ya establecido; ningún componente ni carpeta nueva en el
catálogo — se amplía `Sidebar.svelte`, que ya es el único responsable de la navegación.

## Complexity Tracking

*Sin entradas: el Constitution Check no encontró ninguna violación que justificar.*
