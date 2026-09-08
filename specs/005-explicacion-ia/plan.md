# Implementation Plan: Explicación en lenguaje llano con IA

**Branch**: `005-explicacion-ia` | **Date**: 2026-09-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/005-explicacion-ia/spec.md`

## Summary

Se añade una capacidad **opcional** que traduce el detalle técnico de una alerta o del detalle
SMART de un disco a lenguaje llano, llamando a un LLM de OpenRouter. La función solo existe si la
persona introduce una clave de API (en el asistente inicial o en Ajustes); la clave se guarda en el
Administrador de credenciales de Windows y nunca en disco. Cada explicación la dispara un gesto
explícito, muestra un indicador de progreso y aparece en un modal renderizado como markdown seguro.
El detalle técnico se anonimiza en Rust —reutilizando `reporting::anonimizar::Anonimizador`— antes
de salir del proceso; si queda texto libre no verificable, la persona ve el texto exacto y decide.

**Enfoque técnico**: la llamada de red se hace desde el backend con `reqwest` (ya compilado vía
Tauri, se le añade el backend TLS `rustls`), invocada desde un comando `#[tauri::command]` async.
El Administrador de credenciales se maneja con el crate `windows` (ya compilado) y sus APIs
`Win32_Security_Credentials`. El markdown se renderiza con un analizador de subconjunto propio en
`src/lib/design/markdown.ts` + un componente `Markdown.svelte` que emite solo marcado Svelte, nunca
`{@html}`. No se usa `tauri-plugin-http` ni se añade ningún permiso de *capabilities*: la red la
origina Rust, no el WebView. La lógica de prompt, anonimización, detección de coste y análisis de
respuesta vive en `domain/ia.rs` (pura, test-first); el transporte HTTP en
`platform/ia_openrouter.rs`; el comando es fino.

## Technical Context

**Language/Version**: Rust 1.77.2 (edición 2021) · TypeScript 5.9 · Svelte 5 (runes)

**Primary Dependencies**:
- `reqwest` — se promueve a dependencia **directa** con `features = ["rustls-tls", "json"]`. Ya
  está en el árbol (lo arrastra `tauri` 2.11.5) pero sin backend TLS; `rustls-tls` añade
  `rustls` + `ring` + `webpki-roots` + `rustls-pki-types` (≈6 crates). Justificación: ADR-046.
- `windows` 0.61.3 — se promueve a dependencia **directa** en
  `[target.'cfg(windows)'.dependencies]` con `features = ["Win32_Security_Credentials",
  "Win32_Foundation"]`. Ya compilado (lo arrastra `tauri`). Cero crates transitivos nuevos.
  Justificación: ADR-046.
- Sin dependencias de frontend nuevas. Markdown = analizador propio.

**Storage**: SQLite (`settings` k/v vía `repo_varios`). **Tres** claves nuevas:
`settings.ai.model`, `settings.ai.preview_acknowledged`, y `settings.ai.enabled` (espejo de
«existe credencial», para que la UI y el `load` no tengan que tocar el almacén en cada render).
La **clave de API no toca SQLite**: va al Administrador de credenciales de Windows con nombre
`SmartDisk Monitor/OpenRouter`. Sin migración: son filas nuevas en una tabla existente.

**Testing**: `cargo test` (dominio `ia.rs` test-first: prompt, anonimización aplicada, detección
de modelo de pago, barrido de texto residual, análisis de respuesta y de error) · `pnpm test`
(`markdown.ts`) · `pnpm test:component` (`Markdown.svelte`, modal de explicación, sección de
Ajustes, paso del asistente — estados vacío/cargando/error) · `pnpm test:e2e`
(`e2e/ui/ia.spec.ts`: activar clave simulada, explicar una alerta, fallo de red) · `pnpm test:a11y`
(modal y formularios nuevos)

**Target Platform**: Windows x64, aplicación de escritorio elevada (`requireAdministrator`)

**Project Type**: desktop-app (frontend SvelteKit estático + backend Rust/Tauri)

**Performance Goals**: la explicación depende de un tercero; SC-002 fija ≤20 s en condiciones
normales y FR-019 un tope de 60 s. Ninguna ruta caliente de la app se ve afectada: el cliente
`reqwest` solo se construye dentro del comando cuando hay clave.

**Constraints**: cero valores visuales literales · cero literales de interfaz (claves en `es` y
`en`) · toda llamada de red iniciada por gesto explícito (principio XVI) · respuesta del LLM
tratada como contenido no confiable (markdown seguro, nunca HTML, nunca decisión de la app) ·
la clave nunca en disco · correcto en tema claro y oscuro y en 1024 × 560 · sin `console.*` ni
`println!` · el contenido de peticiones/respuestas nunca al log

**Scale/Scope**: una capacidad transversal. Backend: 1 módulo de dominio nuevo (`domain/ia.rs`),
1 de plataforma (`platform/ia_openrouter.rs`), ~6 comandos nuevos, 4 DTO nuevos. Frontend: 1
componente de catálogo (`Markdown.svelte`), 1 modal de explicación, 1 sección de Ajustes, 1 paso
del asistente, la acción «Explícamelo» en 2 pantallas (`/alerts`, `/disks/[id]`). ~35 claves i18n.

## Constitution Check

*GATE: pasa antes de Phase 0. Re-evaluado tras Phase 1.*

| Principio | Efecto de esta feature | Veredicto |
|---|---|---|
| I. Veracidad del dato | La explicación es orientación de un tercero, **nunca** un dato de salud: el modal lo rotula así (FR-013), no cambia ningún color ni regla (FR-023, SC-007). No se inventan valores: si falla, es error, no modal vacío. | ✅ |
| II. Orden de prioridades | Privacidad antes que comodidad: anonimización obligatoria (FR-009), revisión de texto libre (FR-026), aviso de coste (FR-015a). La función caída degrada solo su modal (FR-017). | ✅ |
| III. Pila fija / cero red | **Excepción del principio XVI** (enmienda 1.8.0), apagada de fábrica: sin clave no se construye cliente ni hay DNS (FR-005, SC-001). Dos dependencias nuevas, ambas **ya compiladas**, con su justificación en ADR-046. Sin biblioteca de componentes de terceros (markdown propio). | ✅ (bajo XVI) |
| IV. Dominio / presentación | `domain/ia.rs` no conoce Tauri ni la red: construye el prompt, aplica el `Anonimizador`, detecta coste y analiza la respuesta. `platform/ia_openrouter.rs` hace el transporte. Comandos finos. Ningún componente decide reglas. | ✅ |
| V. Persistencia local | Solo 3 filas nuevas en `settings` (UTC no aplica). La clave **no** va a SQLite ni a `localStorage` (FR-004). Sin migración. La retención no toca nada nuevo. | ✅ |
| VI. Sistema de diseño | Modal sobre `.sdm-material-overlay` como `ConfirmDialog`; `Select` del catálogo para el modelo; tokens y utilidades; claves en los dos diccionarios; sin condicionales de tema. `Markdown.svelte` entra al catálogo con el criterio de `ui-design.md` §3 (tarea). | ✅ (verificar en implementación) |
| VII. Accesibilidad AA | Modal `role="dialog" aria-modal` con foco atrapado y `Escape` (FR/US2.6); indicador de progreso con `aria-live`; el markdown produce encabezados y listas reales; objetivos ≥30 px. | ✅ |
| VIII. Testeabilidad | `domain/ia.rs` es un parser y es crítico en privacidad → **test-first** (constitución VIII: «parsers» y «donde el fallo es silencioso»). Cobertura de estados en los componentes nuevos. | ✅ |
| IX. Seguridad y privacidad | Cero telemetría propia (FR-022). Mínimo privilegio: **sin permiso de capabilities nuevo** (red desde Rust). La clave en almacén del SO (DPAPI). Anonimización con sustitución consistente reutilizando el módulo ya auditado. Tiempo máximo y salida acotada (FR-019, FR-021). | ✅ |
| X. Errores comprensibles | Todo fallo → `AppError` con `code` estable (`ia.no_key`, `ia.unauthorized`, `ia.rate_limited`, `ia.timeout`, `ia.network`, `ia.empty_response`, `ia.provider`), frase i18n y `detail` técnico conservado. `retryable` en timeout/red/límite. | ✅ |
| XI. Validación de fronteras | La respuesta de OpenRouter (JSON de tercero, formato que puede cambiar) se deserializa con tipos `serde` explícitos en Rust; nunca `serde_json::Value` para decidir. En TS, el DTO del comando se valida con Zod como toda frontera. | ✅ |
| XII. Configuración sin env | Nada en `process.env` ni `$env/*`. Endpoint y nombre de credencial son constantes de código. La clave en el almacén del SO, exactamente la vía que este principio ya anticipaba. | ✅ |
| XIII. Validación de tipos UI | `pnpm check` cero errores/avisos antes de pruebas. DTO nuevos generados con `ts-rs` y versionados. | ✅ |
| XIV. SvelteKit idiomático | `load` de `/alerts` y `/disks/[id]` no cambia; la explicación es una acción bajo demanda (mismo patrón que `getAlertSmartRawJson`). Mutaciones (guardar/borrar clave, modelo) por `$lib/api`, nunca `invoke` directo. Navegación del asistente con `goto` (programática, legítima). | ✅ |
| XV. Registro | `tracing` con eventos sin contenido: `info` al activar/desactivar, `debug` «consulta IA modelo=… resultado=ok|error». Nunca el prompt ni la respuesta. El envoltorio del frontend para `warn`/`error`. | ✅ |
| XVI. Asistencia con IA | Ver tabla de cumplimiento punto por punto en `research.md` §7. | ✅ |

**Límites duros de `AGENTS.md`**:
- **Añade dependencias**: sí, dos. Ambas ya compiladas transitivamente; justificación en ADR-046
  (ya aceptado). Tarea: fijar en ADR-046 la elección concreta (reqwest+rustls, windows/Win32) y
  precisar en la tabla de pila de la constitución (cambio **patch**, redacción, lo aplica el
  usuario porque el fichero está protegido por hook).
- **Cambia contratos**: sí, comandos nuevos → se documenta en `docs/ui-contract.md` (tarea).
- **Permisos de Tauri**: **ninguno nuevo** (red desde Rust). Se anota en ADR-046 que la vía
  elegida no requiere entrada de capabilities.
- No toca `third-party/`, ni `.specify/memory/constitution.md` salvo la precisión patch de la
  tabla de pila, que prepara un parche para que lo aplique el usuario.

**Resultado del gate**: PASA bajo el principio XVI. Sin violaciones que justificar en
`Complexity Tracking`.

### Re-evaluación tras Phase 1 (diseño)

El diseño no introduce violaciones nuevas:
- Las dos dependencias siguen siendo crates **ya compilados**; el diseño solo fija sus *features*
  (`reqwest` → `rustls`, `json`; `windows` → `Win32_Security_Credentials`, `Win32_Foundation`).
- No aparece ningún permiso de *capabilities* (la red la origina Rust) — se confirma en
  `research.md` §D1 y §7.
- `Markdown.svelte` es el único componente de catálogo nuevo y no usa `{@html}` (research §D3).
- La respuesta del proveedor se deserializa con tipos `serde` explícitos (principio XI) y se
  valida con Zod al cruzar a TS.
- La única interacción con `constitution.md` es una precisión **patch** de la tabla de pila, con
  parche preparado para que lo aplique el usuario.

Gate post-diseño: **PASA**.

## Project Structure

### Documentation (this feature)

```text
specs/005-explicacion-ia/
├── plan.md              # Este fichero
├── spec.md              # Especificación (con Clarifications)
├── research.md          # Phase 0 — decisiones técnicas + cumplimiento del principio XVI
├── data-model.md        # Phase 1 — settings, credencial, DTO, entidades efímeras
├── quickstart.md        # Phase 1 — guía de validación end-to-end
├── contracts/
│   └── comandos-ia.md   # Phase 1 — firmas de los comandos y forma de los DTO
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 — lo genera /speckit-tasks
```

### Source Code (repository root)

```text
src-tauri/
├── Cargo.toml                          # + reqwest (directo, rustls-tls, json); + windows (Win32_Security_Credentials)
├── src/
│   ├── domain/
│   │   ├── mod.rs                       # + pub mod ia;
│   │   └── ia.rs                        # NUEVO — puro: PromptExplicacion, aplicar_anonimizador,
│   │                                    #   modelo_es_de_pago, barrer_texto_residual,
│   │                                    #   analizar_respuesta / analizar_error
│   ├── platform/
│   │   ├── mod.rs                       # + pub mod credenciales; + pub mod ia_openrouter;
│   │   ├── credenciales.rs              # NUEVO — CredReadW/CredWriteW/CredDeleteW envueltos
│   │   └── ia_openrouter.rs             # NUEVO — cliente reqwest: chat_completions(), listar_modelos()
│   │                                    #   timeout 60 s, endpoint constante, sin cliente si no hay clave
│   ├── reporting/
│   │   └── anonimizar.rs                # SIN CAMBIOS — se reutiliza Anonimizador::para_esta_maquina
│   ├── commands/
│   │   └── mod.rs                       # + estado_ia, guardar_clave_ia, borrar_clave_ia,
│   │                                    #   probar_clave_ia, listar_modelos_ia, explicar_detalle_tecnico
│   ├── error.rs                         # SIN CAMBIOS (se usan códigos nuevos, no tipos nuevos)
│   └── lib.rs                           # registra los 6 comandos en generate_handler!
│
src/
├── lib/
│   ├── api/
│   │   ├── generated/                   # + ExplicacionIaWire, ModeloIaWire, EstadoIaWire, RevisionAnonimizacionWire
│   │   ├── types.ts                     # reexporta los DTO nuevos con su comando real
│   │   ├── schemas.ts                   # + esquemas Zod de los DTO nuevos
│   │   ├── client.ts                    # + wrappers: estadoIa(), guardarClaveIa(), etc.
│   │   └── index.ts                     # reexporta
│   ├── design/
│   │   ├── markdown.ts                  # NUEVO — analizador de subconjunto → árbol de tokens
│   │   └── markdown.test.ts             # NUEVO
│   ├── components/
│   │   ├── Markdown.svelte              # NUEVO — render del árbol, solo marcado Svelte
│   │   ├── ExplicacionModal.svelte      # NUEVO — modal de explicación (progreso, error, markdown, modelo)
│   │   └── index.ts                     # + exports
│   ├── stores/
│   │   └── ia.svelte.ts                 # NUEVO — estado de la consulta en curso (evita doble envío, FR-020)
│   └── i18n/
│       ├── es.json                      # + ~35 claves
│       └── en.json                      # + ~35 claves
├── routes/
│   ├── onboarding/+page.svelte          # + 5.º paso opcional «Ayuda con IA»
│   ├── settings/+page.svelte            # + sección «Ayuda con IA» (clave, probar, borrar, modelo)
│   ├── settings/+page.ts                # el load ya trae settings; + estadoIa()
│   ├── alerts/+page.svelte              # + botón «Explícamelo» en el detalle
│   └── disks/[id]/+page.svelte          # + botón «Explícamelo» en la tarjeta de contadores SMART
│
e2e/ui/ia.spec.ts                        # NUEVO
docs/                                    # ui-contract.md, architecture.md, data-model.md,
                                         # product-specification.md, engineering-conventions.md,
                                         # open-questions.md, decisions.md (ADR-046) — tareas de cierre
```

**Structure Decision**: se respeta la arquitectura de tres capas (principio IV). Lo puro y
testeable (`domain/ia.rs`) queda aislado de la red y de Tauri; el transporte HTTP se agrupa en
`platform/` junto al resto de integración con el SO; los comandos son finos. En el frontend,
`Markdown.svelte` entra al catálogo cerrado (no se importa una biblioteca); el estado de la
consulta en curso vive en un store de runes, no en un componente.

## Complexity Tracking

> Sin violaciones de la constitución que justificar. La tabla se omite.
