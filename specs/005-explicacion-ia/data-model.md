# Phase 1 — Modelo de datos

Feature `005-explicacion-ia`. No hay migración de SQLite: solo filas nuevas en `settings` y una
credencial en un almacén del sistema operativo. Las entidades de la consulta son efímeras.

---

## 1. Persistencia

### 1.1 `settings` (tabla k/v existente, `repo_varios`)

| Clave | Tipo JSON | Fábrica | Quién escribe | Notas |
|---|---|---|---|---|
| `settings.ai.enabled` | `bool` | `false` | `guardar_clave_ia` (→`true`), `borrar_clave_ia` (→`false`) | Espejo de «existe credencial». La UI no lo edita. Evita tocar el almacén de credenciales en cada `get_settings`. |
| `settings.ai.model` | `string` | `"openrouter/free"` | `set_setting` (desde Ajustes / asistente) | Identificador del modelo. `"openrouter/free"` = «modelo gratuito automático». |
| `settings.ai.preview_acknowledged` | `bool` | `false` | `explicar_detalle_tecnico` al confirmar la vista previa; `borrar_clave_ia` (→`false`) | FR-010, FR-027. Se reinicia si se desactiva y reactiva la función. |

Validación (`domain/ajustes.rs`, nuevas funciones puras):
- `model`: cadena no vacía, ≤120 chars, sin espacios ni saltos. No se valida contra la lista del
  proveedor (puede cambiar); si el proveedor lo rechaza luego, es `ia.provider`.
- `enabled`, `preview_acknowledged`: booleanos.

### 1.2 Grupo en `SettingsWire` (contrato)

```rust
#[derive(Serialize, TS)] #[serde(rename_all = "camelCase")]
pub struct AiSettingsWire {
    pub enabled: bool,
    pub model: String,
    pub preview_acknowledged: bool,
}
// SettingsWire gana el campo:  pub ai: AiSettingsWire,
```

### 1.3 `reset_settings`

`ai` se añade a un ámbito de reseteo. **Resetear `ai` ejecuta también `borrar_clave_ia`**: borrar
el modelo y el ack sin borrar la credencial dejaría la función medio configurada. Decisión
registrada en `open-questions.md`.

### 1.4 Credencial (fuera de SQLite)

| Atributo | Valor |
|---|---|
| Almacén | Administrador de credenciales de Windows (`CRED_TYPE_GENERIC`) |
| `TargetName` | `SmartDisk Monitor/OpenRouter` |
| `CredentialBlob` | la clave de API, UTF-16LE, sin BOM |
| `Persist` | `CRED_PERSIST_LOCAL_MACHINE` |
| Ciclo de vida | alta: `guardar_clave_ia` · baja: `borrar_clave_ia` / `reset` · lectura: cada comando que sale a la red |

Estados observables (nunca se expone el valor):
- **no existe** → función inactiva.
- **existe, no comprobada** → recién guardada sin validar (no debería ocurrir: `guardar_clave_ia`
  valida antes de confirmar).
- **existe, válida** → última llamada de validación dio 2xx o 429/402.
- **existe, inválida** → última llamada dio 401/403; la UI invita a corregirla.

La validez se mantiene en memoria del proceso (`AppState`), no se persiste: es una pista para la
UI, no un dato de verdad.

---

## 2. Contexto del disco que viaja al proveedor

Derivado de `DiskSummary` / `DeviceDetail` ya existentes. **Se envía** (FR-008, aclaración Q2):

| Campo | Origen | Por qué es seguro |
|---|---|---|
| `model` | `DiskSummary.model` | lo comparten millones de unidades |
| `deviceType` / `busType` | `DiskSummary.deviceType`, `DeviceDetail.busType` | genérico (HDD/SSD/NVMe, SATA/NVMe/USB) |
| `firmware` | `DeviceDetail.firmware` | versión, no identifica a una persona |
| `powerOnHours` | `DiskSummary.powerOnHours` | magnitud, no identifica |
| antigüedad | `DeviceDetail.firstSeenAt` (se envía como «X meses», no la fecha) | aproximada |

**No se envía**: `serialNumber`, `fingerprint`, `alias`, `volumes[].label`, `id` interno,
`lastReadAt`/fechas exactas, cualquier ruta.

---

## 3. Entidades efímeras (no se persisten)

### 3.1 `OrigenExplicacion` (entrada del comando, construida por el frontend, revalidada en Rust)

| Campo | Tipo | Notas |
|---|---|---|
| `tipo` | `"alerta" \| "smart"` | |
| `deviceId` | `string` | para recuperar el contexto del disco del inventario |
| `alertGroupId` | `string \| null` | obligatorio si `tipo = "alerta"` |
| `idioma` | `"es" \| "en"` | del `settings.appearance.language`; el backend lo revalida |
| `revision` | `"ninguna" \| "enviar_igual" \| "quitar_fragmentos"` | respuesta al diálogo de FR-026 |
| `previewConfirmada` | `bool` | el frontend la pone a `true` tras la vista previa; si `settings.ai.preview_acknowledged` ya es `true`, se ignora |

### 3.2 `ConsultaExplicacion` (interna de `domain/ia.rs`)

Detalle técnico ya anonimizado · bloque de contexto del disco · idioma · modelo solicitado ·
prompt system+user compuesto. Vive lo que dura la llamada.

### 3.3 `ExplicacionIaWire` (salida OK)

| Campo | Tipo |
|---|---|
| `markdown` | `string` (respuesta del modelo, sin tocar salvo recorte de longitud defensivo) |
| `modeloUsado` | `string` (campo `model` de la respuesta de OpenRouter, FR-016) |

### 3.4 `RevisionAnonimizacionWire` (salida cuando FR-026 se dispara)

| Campo | Tipo |
|---|---|
| `textoCompleto` | `string` — exactamente lo que se enviaría |
| `spans` | `Array<{ inicio: u32, fin: u32, motivoKey: string }>` — fragmentos dudosos |

El comando devuelve **una** de: `ExplicacionIaWire`, `RevisionAnonimizacionWire`, `AppError`.
En el contrato se modela como `enum` serde `#[serde(tag = "estado")]`:
`{ estado: "ok", ... } | { estado: "revision", ... }` (el `AppError` va por la vía de error de
Tauri, no en el `Ok`).

### 3.5 `ModeloIaWire`

| Campo | Tipo | Notas |
|---|---|---|
| `id` | `string` | identificador OpenRouter (`openrouter/free`, `vendor/model:free`, …) |
| `nombre` | `string` | nombre legible del proveedor |
| `esDePago` | `bool` | `pricing.prompt != "0" \|\| pricing.completion != "0"` |

### 3.6 `EstadoIaWire`

| Campo | Tipo | Notas |
|---|---|---|
| `activa` | `bool` | = existe credencial |
| `modelo` | `string` | `settings.ai.model` |
| `previewAcknowledged` | `bool` | `settings.ai.preview_acknowledged` |
| `claveValida` | `bool \| null` | última comprobación; `null` = sin comprobar en esta sesión |

---

## 4. Estado en el frontend

`src/lib/stores/ia.svelte.ts` — clase con runes:

| Campo | Para |
|---|---|
| `enCurso: Set<string>` | claves de consulta activas (`${tipo}:${id}`) → FR-020, no doble envío |
| `estado: EstadoIaWire \| null` | cacheado del `load`, refrescado tras guardar/borrar |

Sin store clásico (`writable`) — constitución XIV. Sin `localStorage` — constitución V.

---

## 5. Registro (`tracing`)

| Evento | Nivel | Campos (sin contenido) |
|---|---|---|
| función activada / desactivada | `info` | — |
| consulta IA | `debug` | `tipo`, `modelo_solicitado`, `resultado=ok\|revision\|error`, `codigo` si error, `ms` |
| validación de clave | `debug` | `resultado`, `http` |

Nunca: el prompt, la respuesta, el detalle técnico, la clave.
