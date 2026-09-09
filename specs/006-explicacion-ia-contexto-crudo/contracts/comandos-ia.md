# Phase 1 — Contrato de comandos: delta de la 006 sobre la 005

Feature `006-explicacion-ia-contexto-crudo`. **No hay comandos nuevos salvo uno**; el resto es un
cambio de comportamiento interno de `explicar_detalle_tecnico` y un campo nuevo en `EstadoIaWire`.

El contrato base es `specs/005-explicacion-ia/contracts/comandos-ia.md`. Aquí solo lo que cambia.

---

## `estado_ia` — campo nuevo en la respuesta

```ts
type EstadoIaWire = {
  activa: boolean;
  modelo: string;
  previewAcknowledged: boolean;
  sendWithoutReview: boolean;   // NUEVO
  claveValida: boolean | null;
};
```

Sigue siendo síncrono y sin red. `sendWithoutReview` se lee de `settings.ai.send_without_review`
(fábrica `false`).

---

## `establecer_envio_sin_revision` — comando NUEVO

```
establecer_envio_sin_revision(activar: bool) -> Result<EstadoIaWire, AppError>
```

Síncrono. No toca la red.

1. Sin credencial → `AppError { code: "ia.no_key", message_key: "error.ia.noKey" }` (el modo no
   tiene sentido con la función desactivada).
2. `guardar_ajuste("settings.ai.send_without_review", activar)`.
3. Devuelve `estado_ia()`.

La **confirmación de riesgo (FR-008) es responsabilidad del frontend**: la tarjeta de Ajustes abre
un `ConfirmDialog` antes de llamar con `activar: true`. Apagar (`activar: false`) es directo.

Registro: `tracing::debug!` con `activar` y nada más (no hay contenido sensible). Cumple FR-016.

---

## `borrar_clave_ia` — resetea también el modo

Sin cambio de firma. Además de lo de la 005 (`enabled = false`,
`preview_acknowledged = false`), ahora pone `send_without_review = false`. Así, desactivar y
reactivar la ayuda con IA devuelve el modo a fábrica (FR-012). `settings.ai.model` se sigue
conservando.

---

## `reset_settings` — el ámbito `ai` borra la clave nueva

`claves_por_ambito("ai")` y `("all")` incluyen `settings.ai.send_without_review` en la lista de
claves a borrar (junto a `enabled`, `model`, `preview_acknowledged` y la credencial).

---

## `explicar_detalle_tecnico` — mismo flujo de UI, comportamiento y una forma ampliada

**`OrigenExplicacion` entra igual** y el **etiquetado de `ResultadoExplicacion` no cambia**
(`{ estado: "ok", … }` | `{ estado: "revision", … }`), por lo que el frontend (`explicacion.svelte`)
no cambia su lógica de ramas. **Sí cambia la forma de la variante `ok`**: `ExplicacionIaWire` gana
dos booleanos para los avisos de FR-014/FR-015 (ver la nota de `ExplicacionIaWire` más abajo). Es
un DTO regenerado con `ts-rs` y revalidado con Zod, como `EstadoIaWire`.

```ts
type ExplicacionIaWire = {
  markdown: string;
  modeloUsado: string;
  detalleRecortado: boolean;
  sinVolcado: boolean;   // NUEVO — FR-014: se explicó sin el volcado técnico del disco
  sinSuceso: boolean;    // NUEVO — FR-015: se explicó sin el contenido del suceso de Windows
};
```

Cambios en el flujo del backend (pasos sobre el contrato de la 005):

3'. `reunir_datos_explicacion` compone, además del resumen estructurado, el `user_crudo`
   **en bruto** (sin anonimizar todavía), en el orden `resumen → volcado → suceso`:
   - **el volcado `smartctl -a -j`** del disco implicado (`query_device_json`), para `tipo`
     `"alerta"` con regla `smart.*`/`temp.*`/`nvme.*` y para `tipo` `"smart"`;
   - **el contenido del suceso disparador** (`get_event_by_id` con el `triggering_event_id` de la
     ocurrencia más reciente → `message` + `EventData`), para `tipo` `"alerta"` de regla de suceso.
   Si el volcado no se puede obtener → sigue sin él y `sin_volcado = true` (FR-014). Si el suceso
   no es legible → sigue sin él y `sin_suceso = true` (FR-015). También devuelve, para la capa 1,
   los identificadores extraídos del volcado (`serial_number`, `wwn.id`).

3''. **Anonimización en una sola pasada sobre el `user_crudo` ya ensamblado** (research §D1),
   toda en el dominio, en este orden fijo:
   1. `anon.aplicar(&user_crudo)` — `Anonimizador` con: series de inventario + `serial_number` y
      `wwn.id` del volcado + `COMPUTERNAME`/`USERNAME` (todo esto es la **capa 1**, sustitución
      literal). Marca, modelo, tipo y firmware **no** se añaden como sustituciones (FR-005).
   2. `redactar_identificadores(&texto)` — **capa 2**, barrido por patrones (`S-1-…` → `<SID>`,
      `\Device\…` → `<DISPOSITIVO>`, `0x`+12–16 hex → `<WWN>`), función pura nueva de `domain::ia`.
   3. `recortar(&texto, MAX_DETALLE_CHARS)` con `MAX_DETALLE_CHARS = 40_000`. Como el suceso va al
      final del ensamblado, es lo primero que se pierde al recortar (FR-013); el resumen, al
      principio, nunca se recorta.
   `componer_consulta` solo **ensambla en bruto**; no anonimiza. La anonimización ocurre después,
   sobre el texto completo, lo que evita un modelo mixto (parte antes / parte después de ensamblar).

4'. `barrer_texto_residual` sobre el texto final (**capa 3**):
   - **si `settings.ai.send_without_review == true`**: se ignora el resultado del barrido (no se
     devuelve `{ estado: "revision", … }` por fragmentos) y se continúa con el texto anonimizado.
   - si es `false` (fábrica): comportamiento de la 005 — fragmentos + `revision == "ninguna"` →
     `{ estado: "revision", textoCompleto, fragmentos }`.

5'. **Vista previa (FR-010) sin cambios**: se muestra la primera vez tras activar la función,
   **también** con `send_without_review == true` (FR-009 lo dice explícitamente). El modo solo
   salta la revisión *por fragmentos*, no la vista previa.

Sin errores nuevos. Los `AppError` `ia.*` de la 005 se mantienen. Volcado/suceso no disponibles
**no** son `AppError`: son un aviso dentro del `{ estado: "ok", … }`.

---

## Endpoint y constantes

`platform/ia_openrouter.rs`:

```rust
pub const MAX_DETALLE_CHARS: usize = 40_000;   // 006: cabe un smartctl -a -j completo + suceso
```

El resto (`ENDPOINT`, `TIMEOUT_SEGUNDOS`, `CONNECT_TIMEOUT_SEGUNDOS`) sin cambios.

---

## i18n (claves nuevas, `es` y `en`)

| Clave (borrador) | es |
|---|---|
| `settings.ai.sendWithoutReview.label` | «Enviar sin revisar» |
| `settings.ai.sendWithoutReview.hint` | «Omite la pantalla de revisión de fragmentos dudosos. El texto sigue anonimizándose antes de salir.» |
| `settings.ai.sendWithoutReview.confirmTitle` | «Activar el envío sin revisión» |
| `settings.ai.sendWithoutReview.confirmBody` | «La aplicación seguirá sustituyendo números de serie, nombres y rutas conocidos. Pero fragmentos de texto libre del volcado técnico o de un suceso de Windows podrían salir del equipo sin que los revises antes.» |
| `settings.ai.sendWithoutReview.confirmImpact` | «Podrás desactivarlo cuando quieras. Al desactivar la ayuda con IA se apaga solo.» |
| `settings.ai.sendWithoutReview.confirmCta` | «Activar» |
| `ia.explain.withoutDump` | «Esta explicación se hizo sin el detalle técnico completo del disco (no se pudo leer en este momento).» |
| `ia.explain.withoutEvent` | «Esta explicación se hizo sin el contenido del suceso de Windows que originó la alerta.» |

Las claves de motivo de fragmento (`ia.review.*`) de la 005 se reutilizan; si la capa 2 marca
además SID o dispositivo, se añaden `ia.review.sid` e `ia.review.devicePath` (o se sustituyen
directamente sin marcar — ver research §D1, se sustituyen).
