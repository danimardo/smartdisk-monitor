# Fase 0 — Investigación: reprocesar la explicación con IA con otro modelo

Todas las incógnitas de esta feature son de diseño de integración, no de tecnología: la pila ya
está fijada (constitución §III) y esta función se apoya entera en piezas que ya existen (spec
005-explicacion-ia, 006-explicacion-ia-contexto-crudo, ADR-046/047/054). No hay ningún
`NEEDS CLARIFICATION` pendiente de la spec (ya resuelto en `/speckit-clarify`); lo que sigue es
cómo encajar los ocho requisitos en el código real.

## D1 — Dónde vive el estado del reproceso

**Decisión**: se extiende la clase `Explicacion` de `src/lib/stores/explicacion.svelte.ts`
(instancia única compartida por `/alerts` y `/disks/[id]`), no el componente `ExplicacionModal.svelte`.

**Justificación**: la constitución §IV exige separar dominio/orquestación de presentación; este
proyecto ya sigue el patrón "modal presentacional + store de runas que orquesta" para toda esta
función. `ExplicacionModal.svelte` ya declara explícitamente en su cabecera: *"Presentacional: toda
la orquestación (llamadas, reintentos, revisión) vive en la pantalla"*. Añadir estado de reproceso
al store es continuar ese mismo patrón, no crear uno nuevo.

**Alternativas descartadas**: estado local en el componente (`$state` dentro de
`ExplicacionModal.svelte`) — rompe la separación ya establecida y duplicaría lógica si en el futuro
otra pantalla necesitara el mismo modal con reproceso.

## D2 — Cómo llega el modelo elegido al backend

**Decisión**: se añade un campo opcional `modelo_solicitado: Option<String>` (TS:
`modeloSolicitado?: string`) a `OrigenExplicacion` (`src-tauri/src/domain/ia.rs`). En
`reunir_datos_explicacion` (`src-tauri/src/commands/mod.rs`), cuando este campo está presente,
sustituye a `settings.ai.model` solo para esa llamada; el ajuste persistido no se toca. El comando
`explicar_detalle_tecnico` no cambia de firma (sigue recibiendo un único `origen`).

**Justificación**: `OrigenExplicacion` ya es el vehículo de "variaciones de una misma petición"
(`revision`, `preview_confirmada`); un modelo por llamada es una variación de la misma clase. Evita
un segundo parámetro de comando y mantiene un solo punto de entrada IPC para esta función
(`docs/ui-contract.md` §3.10).

**Alternativas descartadas**: un comando nuevo `reprocesar_explicacion` — duplicaría toda la lógica
de anonimización, vista previa y revisión de `explicar_detalle_tecnico` sin aportar nada; el propio
FR-003 exige reutilizar exactamente esa lógica.

## D3 — Cómo se recuerdan las respuestas anteriores de la sesión

**Decisión**: un array `historial` en el propio store (`$state<Intento[]>([])`), con
`Intento = { modelo: string; markdown?: string; error?: AppError }`. `lanzar()` lo vacía al abrir
una explicación nueva (FR-012); cada llamada a `reprocesar()` empuja primero un snapshot del
resultado o error que estaba en primer plano antes de sobrescribirlo. Sin límite de tamaño
(clarificación del 2026-09-12).

**Justificación**: es contenido efímero de una sesión de UI, igual que el resto del estado de esta
clase (`markdown`, `error`, etc.); no hay motivo para que sobreviva a cerrar el modal ni para que
pase por SQLite (constitución §V: solo se persiste lo que debe sobrevivir a un reinicio).

## D4 — Cómo se presenta el historial sin componente nuevo

**Decisión**: cada entrada del historial se pinta con el mismo patrón `<details>/<summary>` que
`ExplicacionModal.svelte` ya usa para el detalle técnico de un error (fase `error`, líneas 93-100
del componente). Un `<details>` colapsado por entrada, con el modelo como texto del `<summary>`.

**Justificación**: AGENTS.md prohíbe un componente fuera del catálogo cerrado sin pasar el criterio
de `ui-design.md` §3; reutilizar un patrón nativo ya presente en el mismo fichero no necesita ese
trámite y cumple el punto 5 de lo acordado con el responsable del producto (nada de carrusel).

## D5 — Cómo se deshabilitan los modelos de pago según la clave activa

**Decisión**: dos cambios pequeños, ambos en componentes ya existentes:

1. `Select.svelte` gana un campo opcional `disabled` por opción (`{ id, label, disabled? }`),
   renderizado como `<option disabled>`. Es una extensión compatible hacia atrás: ninguna opción
   existente en la aplicación pasa `disabled`, así que no cambia nada fuera de esta feature.
2. `AiModelSelect.svelte` calcula `disabled: m.esDePago && ia.usandoClaveCompartida` para cada
   modelo de pago al construir `opciones`, importando el store `ia` ya existente
   (`src/lib/stores/ia.svelte.ts`, que ya expone `usandoClaveCompartida` desde ADR-054). Gana
   también un prop `incluirAutomatico = true` para que el selector de esta función pueda excluir la
   opción "automático" (FR-006) sin duplicar el componente.

**Justificación**: `usandoClaveCompartida` ya está calculado en el backend y ya cruza a la interfaz
(`EstadoIaWire`); no hace falta ninguna llamada ni campo nuevo, solo leerlo donde antes no se leía.
Reutilizar `AiModelSelect` en los dos sitios (Ajustes y el modal) es lo que garantiza que el mismo
criterio rige en los dos selectores (FR-010/FR-011), en vez de mantener dos implementaciones que
puedan divergir.

**Alternativas descartadas**: ocultar los modelos de pago en vez de deshabilitarlos — es justo lo
que el responsable del producto pidió evitar, y además ya rompería con `catalogo_modelos` (que hoy
siempre los incluye) y con la prueba existente de `AiModelSelect.browser.test.ts`.

## D6 — Encaje con ADR-054

**Decisión**: la implementación requiere una enmienda a ADR-054, con el mismo patrón que ADR-045
enmendó a ADR-044 (se referencia la decisión original, no se reescribe). El siguiente número libre
en `docs/decisions.md` es **ADR-058** (el último existente es ADR-057). Redactar esa entrada es
tarea de la fase de implementación (`tasks.md`), no de este plan — pero queda reservado aquí el
número para que no haya colisión si otra rama abre un ADR entre tanto.

**Justificación**: fue una decisión explícita tomada con el responsable del producto en esta misma
conversación (ver clarificaciones de la spec), no una interpretación de este plan.

## D7 — Frontera de tipos (constitución §XI)

**Decisión**: el nuevo campo `modeloSolicitado` en `OrigenExplicacion` necesita su entrada en el
esquema Zod de `src/lib/api/schemas.ts` (espejo del tipo generado por `ts-rs` en
`src/lib/api/generated/OrigenExplicacion.ts`) y su prueba de rechazo correspondiente
(`.claude/rules/frontera-ipc.md`: "todo esquema nuevo necesita su prueba de rechazo").

**Justificación**: regla dura del proyecto, sin alternativa.

## Resumen de impacto (para `data-model.md` y `tasks.md`)

- **Backend**: `domain/ia.rs` (campo nuevo + prueba), `commands/mod.rs`
  (`reunir_datos_explicacion`), `docs/decisions.md` (ADR-058), `docs/ui-contract.md` §3.10
  (documentar el campo nuevo).
- **Frontend**: `stores/explicacion.svelte.ts` (historial, `reprocesar`, `fijarPorDefecto`),
  `components/ExplicacionModal.svelte` (UI de selector + reproceso + historial plegable),
  `components/AiModelSelect.svelte` (prop `incluirAutomatico`, deshabilitado por clave),
  `components/Select.svelte` (opción `disabled`), `lib/api/schemas.ts` + tipos generados, `i18n/es.json`
  + `en.json`.
- **Sin cambios**: informe HTML (spec 009) — hereda el modelo por defecto sin tocar su código,
  exactamente como pide el alcance.
