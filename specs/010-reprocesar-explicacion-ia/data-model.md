# Fase 1 — Modelo de datos: reprocesar la explicación con IA

Esta feature no añade ninguna tabla ni columna a SQLite. Todo lo nuevo es estado de interfaz
efímero (vive en memoria del proceso de la interfaz, se pierde al cerrar el modal) o una variación
por-llamada de un tipo que ya existe. Confirma la asunción de la spec: *"El historial de respuestas
dentro de una sesión de explicación es efímero (...); no se guarda en base de datos"*.

## Entidad: Intento de explicación

Solo en el frontend (`src/lib/stores/explicacion.svelte.ts`), nunca serializado hacia el backend ni
hacia disco.

| Campo | Tipo | Notas |
|---|---|---|
| `modelo` | `string` | El identificador de modelo con el que se pidió este intento (el elegido en el selector, o el que resolvió el modo automático la primera vez). |
| `markdown` | `string \| undefined` | Presente si el intento terminó en éxito. Contenido no confiable (principio XVI): se renderiza igual que hoy, nunca como HTML. |
| `error` | `AppError \| undefined` | Presente si el intento terminó en error. Mutuamente excluyente con `markdown`. |

Además del array `historial`, el store guarda un booleano `esReprocesada` sobre **la respuesta en
primer plano** (no sobre cada entrada del historial): `true` si esa respuesta se obtuvo llamando a
`reprocesar()`, `false` si es la respuesta inicial del modo automático. Es lo que decide si el botón
"fijar como predeterminado" (FR-008) se ofrece o no — nunca sobre la respuesta automática inicial.
Vive y se resetea igual que el resto del estado de la sesión: `false` en `lanzar()`, `true` desde el
primer `reprocesar()` en adelante (no vuelve a `false` dentro de la misma sesión).

**Reglas**:
- Un intento se crea únicamente al reprocesar: se archiva el que estaba en primer plano justo antes
  de sobrescribirlo con el nuevo (FR-007).
- No tiene identificador propio ni orden explícito más allá de su posición en el array: el más
  reciente archivado es el primero en aparecer bajo la respuesta actual.
- Sin máximo de elementos (clarificación del 2026-09-12, FR-007).
- Se descarta entero al llamar a `lanzar()` para una explicación nueva (FR-012), y por tanto también
  al cerrar y reabrir el modal para otro caso.

## Tipo existente ampliado: `OrigenExplicacion`

`src-tauri/src/domain/ia.rs`, espejo en `src/lib/api/generated/OrigenExplicacion.ts` vía `ts-rs`.

| Campo | Tipo | Estado |
|---|---|---|
| `tipo`, `deviceId`, `alertGroupId`, `eventId`, `idioma`, `revision`, `previewConfirmada` | — | Sin cambios. |
| `modeloSolicitado` | `string \| null` (Rust: `Option<String>`) | **Nuevo.** Sigue la misma convención que el resto de campos `Option<String>` del struct (`deviceId`, `alertGroupId`, `eventId`): campo **obligatorio**, `null` cuando no aplica — nunca ausente. Con un valor no nulo, sustituye a `settings.ai.model` para esa llamada; no se persiste. Con `null`, comportamiento actual. Solo lo rellena el store (`explicacion.svelte.ts`); queda fuera del tipo `Origen` que reciben los llamantes de `lanzar()` (ver `research.md` D2). |

**Validación**: el backend no valida que `modeloSolicitado` pertenezca al catálogo de OpenRouter —
igual que hoy no valida `settings.ai.model` al leerlo—; un identificador inválido simplemente
produce el error del proveedor, que ya tiene su tratamiento (`ia.*`, principio X). El esquema Zod en
`src/lib/api/schemas.ts` valida solo la forma (cadena o `null`), no su contenido.

## Sin cambios: `ModeloIaWire`, `EstadoIaWire`, `settings.ai.model`

- `ModeloIaWire` (catálogo de OpenRouter) y `EstadoIaWire.usandoClaveCompartida` ya existen y ya
  cruzan a la interfaz (ADR-054); esta feature los **lee**, no los modifica.
- `settings.ai.model` (SQLite, tabla de ajustes) es el mismo ajuste que ya usan Ajustes y el informe
  HTML con resumen IA (spec 009); "fijar como predeterminado" (FR-008/FR-009) escribe ahí con el
  comando `set_setting` ya existente, sin comando nuevo.

## Diagrama de flujo del estado (frontend)

```
lanzar(origen)
  └─ historial = []; modeloOverride = undefined; esReprocesada = false
  └─ #pedir(previewConfirmada=false) → fase progreso → resultado | error | vistaPrevia | revision

reprocesar(modelo)                          [solo disponible en fase resultado | error]
  └─ historial.push(snapshot de fase actual: {modelo: modeloUsado o modelo del intento, markdown|error})
  └─ modeloOverride = modelo; esReprocesada = true
  └─ #pedir(previewConfirmada=true) → fase progreso → resultado | error
       (reutiliza la revisión ya resuelta de esta sesión, igual que `reintentar` hoy)

fijarPorDefecto(modelo)                      [solo si esReprocesada === true]
  └─ set_setting("settings.ai.model", modelo)
  └─ ia.refrescar()                          (para que Ajustes y el resto de la app lo reflejen ya)
```
