# Phase 1 — Contrato de comandos: Ayuda con IA

Feature `005-explicacion-ia`. Seis comandos nuevos, registrados en `lib.rs` (`generate_handler!`).
Se documentan en `docs/ui-contract.md` al cerrar. Todos los DTO se generan con `ts-rs` a
`src/lib/api/generated/` y se validan con Zod en `src/lib/api/schemas.ts`.

Convención del proyecto: los comandos que fallan devuelven `AppError` (nunca cadena suelta); el
frontend accede siempre por `$lib/api`, nunca `invoke` directo (principio IV, XIV).

---

## `estado_ia`

```
estado_ia() -> Result<EstadoIaWire, AppError>
```

Síncrono. No toca la red. Lee la presencia de credencial + `settings.ai.*` + la validez cacheada
en `AppState`. Lo llama el `load` de `/settings` y el store `ia`.

```ts
type EstadoIaWire = {
  activa: boolean;
  modelo: string;
  previewAcknowledged: boolean;
  claveValida: boolean | null;
};
```

---

## `guardar_clave_ia`

```
guardar_clave_ia(clave: String) -> Result<EstadoIaWire, AppError>
```

Async. Pasos:
1. Valida forma de `clave` (no vacía, sin espacios, longitud plausible). Forma inválida →
   `AppError { code: "ia.invalid_key_format", message_key: "error.ia.invalidKeyFormat" }`.
2. Llamada mínima de validación a OpenRouter (`GET /models` con `Authorization`).
   - `401/403` → **no guarda** → `AppError { code: "ia.unauthorized", … }`.
   - error de red / timeout → **no guarda** → `AppError { code: "ia.network" | "ia.timeout", retryable: true }`.
   - `2xx`, `429`, `402` → clave válida.
3. `CredWriteW` con la clave. `settings.ai.enabled = true`. `claveValida` en `AppState` según 2.
4. Devuelve `estado_ia()`.

---

## `borrar_clave_ia`

```
borrar_clave_ia() -> Result<EstadoIaWire, AppError>
```

Síncrono. `CredDeleteW` (idempotente). `settings.ai.enabled = false`,
`settings.ai.preview_acknowledged = false`. `settings.ai.model` se conserva (si vuelve a activar,
mantiene su preferencia de modelo). Devuelve `estado_ia()` (`activa: false`).

---

## `probar_clave_ia`

```
probar_clave_ia() -> Result<EstadoIaWire, AppError>
```

Async. Como el paso 2 de `guardar_clave_ia` pero sobre la clave ya guardada, sin cambiarla.
Actualiza `claveValida` en `AppState`. Sin credencial → `AppError { code: "ia.no_key" }`.
Botón «Probar» de la sección de Ajustes.

---

## `listar_modelos_ia`

```
listar_modelos_ia() -> Result<Vec<ModeloIaWire>, AppError>
```

Async. `GET /api/v1/models` (sin `Authorization`; no requiere clave, pero solo se invoca desde el
selector abierto por la persona). Antepone siempre `openrouter/free` («automático»,
`esDePago: false`). Ordena: automático primero, luego gratuitos, luego de pago, alfabético.
Fallo → `AppError` (`ia.network`, `ia.timeout`, `ia.provider`); el frontend deja usar «automático»
y avisa de que la lista no está disponible (US3 escenario 3).

```ts
type ModeloIaWire = { id: string; nombre: string; esDePago: boolean };
```

---

## `explicar_detalle_tecnico`

```
explicar_detalle_tecnico(origen: OrigenExplicacion) -> Result<ResultadoExplicacion, AppError>
```

Async. Es el comando de la acción «Explícamelo».

```ts
type OrigenExplicacion = {
  tipo: "alerta" | "smart";
  deviceId: string;
  alertGroupId: string | null;        // requerido si tipo === "alerta"
  idioma: "es" | "en";
  revision: "ninguna" | "enviar_igual" | "quitar_fragmentos";
  previewConfirmada: boolean;
};

type ResultadoExplicacion =
  | { estado: "ok"; markdown: string; modeloUsado: string }
  | { estado: "revision"; textoCompleto: string; spans: SpanRevision[] };

type SpanRevision = { inicio: number; fin: number; motivoKey: string };
```

Flujo en el backend:

1. **Sin credencial** → `AppError { code: "ia.no_key", message_key: "error.ia.noKey" }`.
2. Revalida `origen` contra inventario/alertas. `alertGroupId` ausente con `tipo:"alerta"`, o
   `deviceId` desconocido → `AppError { code: "ipc.schema_mismatch" }` (principio XI).
3. `domain::ia` compone el detalle técnico y el contexto del disco, **anonimiza** cada fragmento
   con `reporting::anonimizar`.
4. `barrer_texto_residual`:
   - devuelve spans **y** `origen.revision == "ninguna"` → responde
     `{ estado: "revision", textoCompleto, spans }`. No se ha llamado al proveedor.
   - `origen.revision == "quitar_fragmentos"` → sustituye los spans por `<OMITIDO>` y sigue.
   - `origen.revision == "enviar_igual"` → sigue con el texto tal cual.
5. **Vista previa** (FR-010): si `settings.ai.preview_acknowledged == false` y
   `origen.previewConfirmada == false` → responde `{ estado: "revision", textoCompleto, spans: [] }`
   (mismo canal; `spans` vacío = «esto es la vista previa completa, no hay nada dudoso»). El
   frontend distingue vista previa de revisión por `spans.length`.
   Si `origen.previewConfirmada == true` → marca `settings.ai.preview_acknowledged = true` y sigue.
6. `platform::ia_openrouter::chat_completions(clave, modelo, prompt)` con timeout 60 s.
7. `domain::ia::analizar_respuesta` → `{ estado: "ok", markdown, modeloUsado }` o `AppError`
   (`ia.empty_response`).
8. Errores de transporte → `analizar_error` → `AppError` según la tabla de `research.md` §D8.

**Idempotencia / doble envío**: el frontend evita el doble disparo (store `ia`, FR-020). El
backend no mantiene estado de consulta.

---

## Errores (`error.ia.*`)

| `code` | `message_key` | `retryable` | Frase (es, borrador) |
|---|---|---|---|
| `ia.no_key` | `error.ia.noKey` | no | «La ayuda con IA no está activada.» |
| `ia.invalid_key_format` | `error.ia.invalidKeyFormat` | no | «Esa clave no tiene el formato esperado.» |
| `ia.unauthorized` | `error.ia.unauthorized` | no | «OpenRouter ha rechazado la clave.» |
| `ia.rate_limited` | `error.ia.rateLimited` | sí | «Has alcanzado el límite de uso de OpenRouter. Inténtalo más tarde.» |
| `ia.timeout` | `error.ia.timeout` | sí | «La respuesta ha tardado demasiado (más de 60 segundos).» |
| `ia.network` | `error.ia.network` | sí | «No se ha podido conectar con OpenRouter.» |
| `ia.empty_response` | `error.ia.emptyResponse` | sí | «El modelo no ha devuelto una explicación.» |
| `ia.provider` | `error.ia.provider` | sí (5xx) | «OpenRouter ha devuelto un error.» |

`detail` = texto técnico crudo (HTTP, `error.message` de OpenRouter, mensaje de `reqwest`).
**Nunca** el prompt ni la respuesta.

Las mismas claves en `en.json`.

---

## Endpoint (constante, no configurable)

`platform/ia_openrouter.rs`:

```rust
const ENDPOINT: &str = "https://openrouter.ai/api/v1";
const TIMEOUT: Duration = Duration::from_secs(60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_DETALLE_CHARS: usize = 8_000;   // FR-021 (PROPUESTO en open-questions)
```

Cabeceras: `Authorization: Bearer <clave>`, `Content-Type: application/json`,
`HTTP-Referer` / `X-Title` con el nombre de la app (recomendación de OpenRouter, no lleva datos).
