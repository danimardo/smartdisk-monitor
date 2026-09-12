# Implementation Plan: Reprocesar la explicación con IA con otro modelo gratuito

**Branch**: `010-reprocesar-explicacion-ia` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/010-reprocesar-explicacion-ia/spec.md`

## Summary

En el modal de "Explícamelo en lenguaje claro" (`ExplicacionModal.svelte`, spec 005/006), añadir la
posibilidad de elegir otro modelo gratuito y reprocesar la misma petición sin repetir el gesto
original, tanto desde una respuesta correcta como desde un error. Cada respuesta que deja de estar
en primer plano queda accesible en una fila plegable, sin límite ni persistencia. Sobre una
respuesta obtenida así, se puede fijar ese modelo como predeterminado para toda la aplicación
(mismo ajuste `settings.ai.model` que ya comparte el resumen con IA del informe HTML). Los modelos
de pago se muestran deshabilitados —nunca ocultos— mientras la credencial activa sea la clave de
demostración compartida, en los dos selectores de la aplicación (este modal y Ajustes), lo que
enmienda ADR-054.

Enfoque técnico: extender piezas existentes, no crear superficie nueva. Un campo opcional en
`OrigenExplicacion` para pedir un modelo concreto; el historial vive en el store de runas que ya
orquesta este modal; el patrón visual de fila plegable ya existe en el propio componente; el
criterio de deshabilitado se calcula en el cliente a partir de un dato (`usandoClaveCompartida`) que
el backend ya expone. Ningún comando IPC nuevo, ninguna tabla nueva, ninguna dependencia nueva.

## Technical Context

**Language/Version**: TypeScript 5.5 (Svelte 5.0, SvelteKit 2.0, `adapter-static`) + Rust 1.77.2
(edición 2021), Tauri 2.

**Primary Dependencies**: ninguna nueva. Reutiliza `$lib/api` (invoke + Zod), `ts-rs` para los tipos
espejo, `reqwest`/`native-tls` ya integrados para la llamada a OpenRouter
(`platform::ia_openrouter`).

**Storage**: SQLite vía `rusqlite` para `settings.ai.model` (ya existente, sin cambios de esquema).
El historial de respuestas de una sesión es memoria de proceso de la interfaz (runas de Svelte 5),
nunca persistido — ver `data-model.md`.

**Testing**: `pnpm test` (Vitest/jsdom, lógica y stores `*.svelte.ts`), `pnpm test:component`
(Vitest en Chromium real vía `vitest-browser-svelte`, componentes `*.browser.test.ts`),
`pnpm test:e2e` (Playwright con IPC falso), `cargo test` (Rust, `src-tauri/src/**`).

**Target Platform**: Windows 10/11 de escritorio (aplicación Tauri, elevación obligatoria).

**Project Type**: aplicación de escritorio (frontend SvelteKit estático + backend Rust/Tauri), un
único proyecto con dos árboles de código (`src/`, `src-tauri/src/`) — no aplica la distinción
librería/CLI/servicio web de la plantilla genérica.

**Performance Goals**: sin objetivo nuevo. La operación es de red (llamada a OpenRouter), acotada
por los mismos tiempos de espera que ya rigen `explicar_detalle_tecnico`; no hay renderizado masivo
nuevo (el historial de una sesión de explicación es, en la práctica, de un puñado de entradas).

**Constraints**: principio XVI de la constitución (destino único, gesto explícito, sin dato nuevo
que salga del proceso — FR-003/SC-005 lo garantizan reutilizando el texto ya anonimizado). Sin
permiso de Tauri nuevo, sin variable de entorno nueva.

**Scale/Scope**: un componente (`ExplicacionModal.svelte`), un store (`explicacion.svelte.ts`), un
componente compartido (`AiModelSelect.svelte`) y su primitiva (`Select.svelte`), un campo de un tipo
IPC existente (`OrigenExplicacion`), dos diccionarios i18n, una enmienda de ADR.

## Constitution Check

*GATE: debe pasar antes de la fase 0. Repetido tras el diseño de la fase 1 — ver el final de esta
sección.*

| Principio | Aplica | Cómo se cumple |
|---|---|---|
| I. Veracidad del dato | Sí | La explicación de IA ya está marcada como orientación, no como dato de salud (spec 005); esta feature no cambia esa naturaleza ni toca ninguna métrica SMART. |
| II. Orden de prioridades ante conflicto | No aplica | Principio de gobernanza para cuando dos fuentes de verdad chocan; esta feature no introduce ningún conflicto de esa clase (la única tensión, con ADR-054, se resuelve como enmienda explícita, no como conflicto sin resolver). |
| III. Pila fija | Sí | Cero dependencias nuevas (ver Technical Context). |
| IV. Dominio/presentación | Sí | Toda la orquestación nueva (historial, reproceso, fijar por defecto) va en `explicacion.svelte.ts`, no en el componente — ver D1 de `research.md`. |
| V. Persistencia local | Sí | El historial no se persiste (FR-012); el único dato que se escribe es el ya existente `settings.ai.model`, sin esquema nuevo. |
| VI. Sistema de diseño | Sí | Sin literales visuales nuevos; reutiliza `Select`, `Button`, el patrón `<details>` ya presente en el propio modal. Claves i18n nuevas en los dos diccionarios. |
| VII. Accesibilidad AA | Sí | `<option disabled>` nativo (accesible de fábrica); las filas plegables usan `<details>/<summary>`, igual que el detalle técnico ya existente en este modal. `pnpm check` en cero avisos es condición de cierre. |
| VIII. Testeabilidad | Sí | Ver quickstart.md: prueba de rechazo del campo nuevo en Rust, store en jsdom, componentes en Chromium real. |
| IX. Seguridad y privacidad | Sí | Reprocesar reutiliza exactamente el texto ya anonimizado (FR-003); no se añade ninguna llamada de red fuera del único destino ya autorizado. |
| X. Errores comprensibles | Sí | Un reproceso fallido usa la misma fase `error`/`AppError` que ya existe; sin caso especial. |
| XI. Validación en frontera | Sí | `modeloSolicitado` entra en el esquema Zod de `OrigenExplicacion` con su prueba de rechazo (D7 de `research.md`). |
| XII. Config sin variables de entorno | No aplica | Ninguna variable de entorno nueva; ninguna existente se lee desde la aplicación (ver Constraints). |
| XIII. Validación de tipos de la interfaz antes que nada | No aplica | Cubierto ya por XI: el único tipo nuevo que cruza la frontera es `modeloSolicitado`, validado por el mismo esquema Zod. |
| XIV. Arquitectura SvelteKit/Tauri | Sí | Ningún `invoke` directo desde la pantalla; todo pasa por `$lib/api`, como ya exige `frontera-ipc.md`. |
| XV. Registro de actividad | Sí | Sin contenido de petición/respuesta en el log; como mucho metadatos ya existentes (hubo consulta, modelo, si hubo error). |
| XVI. Asistencia con IA en la nube | Sí, con una enmienda | Reprocesar sigue siendo un gesto explícito (clic), un único destino, la misma anonimización. **Requiere enmienda a ADR-054** (FR-010/FR-011: ADR-054 hoy decide explícitamente no validar el modelo contra el tipo de clave). Ya autorizada por el responsable del producto en la conversación que originó esta spec; se redacta como ADR-058 en la fase de implementación (D6 de `research.md`). |

**Resultado**: PASA. La única entrada que no es un "sí" simple (XVI) es una enmienda ya autorizada,
no una violación sin resolver — no hace falta registrarla en Complexity Tracking (esa tabla es para
justificar una violación, no una decisión ya tomada con el responsable del producto).

*Repetido tras la fase 1 (`data-model.md`, `contracts/`, `quickstart.md`): sin cambios. El diseño no
introdujo ningún comando, tabla, dependencia ni permiso nuevo que no estuviera ya previsto aquí.*

## Project Structure

### Documentation (this feature)

```text
specs/010-reprocesar-explicacion-ia/
├── plan.md              # Este fichero
├── research.md          # Fase 0: decisiones D1-D7
├── data-model.md         # Fase 1: entidad efímera + campo nuevo de OrigenExplicacion
├── contracts/
│   └── comandos-ia.md    # Fase 1: delta sobre docs/ui-contract.md §3.10
├── quickstart.md         # Fase 1: guía de validación
└── tasks.md              # Fase 2 (/speckit-tasks — no la crea este comando)
```

### Source Code (repository root)

```text
src-tauri/src/
├── domain/
│   └── ia.rs                       # OrigenExplicacion: + modelo_solicitado
├── commands/
│   └── mod.rs                      # reunir_datos_explicacion: usa el override si llega
└── tests/... (o #[cfg(test)] in-file, patrón ya existente)

src/
├── lib/
│   ├── stores/
│   │   └── explicacion.svelte.ts   # + historial, reprocesar(), fijarPorDefecto()
│   ├── components/
│   │   ├── ExplicacionModal.svelte # + selector, botón reproceso, filas plegables, aviso
│   │   ├── AiModelSelect.svelte    # + prop incluirAutomatico, deshabilitado por clave
│   │   └── Select.svelte           # + opción `disabled` por entrada
│   ├── api/
│   │   ├── schemas.ts              # + modeloSolicitado en el esquema de OrigenExplicacion
│   │   └── generated/
│   │       └── OrigenExplicacion.ts  # regenerado por ts-rs
│   └── i18n/
│       ├── es.json                 # + claves nuevas (mensaje, botones, motivo deshabilitado)
│       └── en.json
└── routes/
    └── settings/+page.svelte       # sin cambio de lógica; sigue pasando onchange a AiModelSelect

docs/
├── decisions.md                    # + ADR-058 (enmienda a ADR-054)
└── ui-contract.md                  # §3.10: documentar modeloSolicitado
```

**Structure Decision**: proyecto único (Tauri + SvelteKit) ya establecido por el repositorio;
ninguna carpeta nueva. Todos los ficheros tocados son extensiones de piezas existentes — no hay
"Option 1/2/3" de plantilla genérica que elegir, la estructura ya la fija `AGENTS.md`.

## Complexity Tracking

*Sin violaciones que justificar.* La única entrada de la tabla de Constitution Check que no es un
cumplimiento directo (principio XVI, enmienda a ADR-054) es una decisión de producto ya tomada
explícitamente, con su ADR de reemplazo planeado — no una excepción a la arquitectura que requiera
una alternativa más simple descartada.
