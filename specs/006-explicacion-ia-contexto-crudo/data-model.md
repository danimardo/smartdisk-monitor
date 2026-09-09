# Phase 1 — Modelo de datos: Contexto crudo para la explicación con IA

Feature `006-explicacion-ia-contexto-crudo`. Este incremento **no crea tablas ni columnas** y **no
requiere migración**. Solo añade una clave k/v en `settings` y amplía entidades efímeras que ya
existían en la 005.

---

## Cambios en `settings` (tabla existente, k/v)

### Grupo `settings.ai` — de tres claves a cuatro

Las tres de la 005 no cambian (`enabled`, `model`, `preview_acknowledged`). Se añade:

- **`settings.ai.send_without_review`**: booleano, fábrica `false`. `true` cuando la persona ha
  activado —con confirmación de riesgo (FR-008)— el modo «enviar sin revisar».
  - La escribe **solo** el comando `establecer_envio_sin_revision` (nunca `set_setting`), igual que
    `enabled`/`preview_acknowledged` en la 005.
  - `borrar_clave_ia` la pone a `false` (junto a `preview_acknowledged`): desactivar y reactivar la
    ayuda con IA devuelve el modo a fábrica (FR-012).
  - `reset_settings` en el ámbito `"ai"` (o `"all"`) la borra, junto con las otras tres claves y la
    credencial. Se añade a la lista `AI` de `claves_por_ambito`.
  - Sin migración: es una fila nueva en una tabla existente; su ausencia se lee como `false`.

`docs/data-model.md`, sección «Ayuda con IA», pasa de decir «Exactamente tres claves» a «Exactamente
cuatro claves» y añade la viñeta de `send_without_review`.

---

## Entidades efímeras (no se persisten)

### Consulta de explicación — ampliada

Lo que `domain::ia::componer_consulta` compone y `platform::ia_openrouter::chat_completions` envía.
Sigue siendo **efímera**: no toca ninguna tabla.

| Parte | 005 | 006 |
|---|---|---|
| `system` | constante por idioma | igual |
| Resumen estructurado | regla + valor + tendencia + contexto del disco (marca, modelo, tipo, firmware, antigüedad) | igual, va **primero** en el `user` |
| Volcado técnico | — | **nuevo**: salida completa de `smartctl -a -j` del disco implicado, anonimizada (D1, D3). Va tras el resumen. |
| Contenido del suceso | — | **nuevo** (solo alertas de suceso): `message` del evento + pares `EventData`, anonimizado (D1, D2). Va al final. |
| Recorte | `recortar(user, 8_000)` | `recortar(user, 40_000)`; el orden hace que el suceso se pierda antes que el volcado y el resumen nunca (FR-013) |

Datos que **se conservan sin sustituir** en el volcado y el contexto (FR-005): marca, modelo,
tipo/interfaz, versión de firmware.

Datos que **se sustituyen por marcadores** antes de salir del proceso (FR-004):

| Dato | Origen | Marcador | Capa (D1) |
|---|---|---|---|
| Número de serie del disco | `serial_number` del JSON + inventario | `<SERIE-n>` | 1 (literal) |
| WWN del disco | `wwn.id` del JSON | `<WWN>` | 1 (literal) + 2 (patrón `0x…`) |
| Nombre del equipo | `COMPUTERNAME` + `<Computer>` descartado de raíz | `<EQUIPO>` | 1 |
| Nombre de usuario / rutas de perfil | `USERNAME` | `<USUARIO>` | 1 |
| Etiquetas de volumen | inventario (ya en 005) | marcador de la 005 | 1 |
| SID | texto libre del suceso | `<SID>` | 2 (patrón `S-1-…`) |
| Ruta NT de dispositivo | `EventData`, mensaje del suceso | `<DISPOSITIVO>` | 2 (patrón `\Device\…`) |
| Fragmento no clasificable | cualquiera | pantalla de revisión / `<OMITIDO>` | 3 (`barrer_texto_residual`) |

### Explicación devuelta

`ExplicacionIaWire` **gana dos booleanos** respecto a la 005: `sin_volcado` y `sin_suceso`, para
los avisos de FR-014 y FR-015. Queda: `markdown`, `modelo_usado`, `detalle_recortado`,
`sin_volcado`, `sin_suceso`. Se regenera con `ts-rs`, se añade a `src/lib/api/schemas.ts` y el
modal muestra `t("ia.explain.withoutDump")` / `t("ia.explain.withoutEvent")` según corresponda. No
se persiste.

(Se descartó reutilizar el canal de `detalle_recortado` con otro motivo: son tres condiciones
independientes —recorte, sin volcado, sin suceso— que pueden darse a la vez, y colapsarlas en un
solo campo obligaría a un enum de motivos más frágil que dos booleanos.)

### Volcado técnico del disco

La cadena JSON de `smartctl -a -j`, obtenida en el momento de la consulta vía
`collectors::smartctl::query_device_json`. No se persiste para esta función (el detalle SMART que
sí se guarda son las métricas parseadas, no el JSON entero — igual que en la 005 con «Ver detalle
técnico»).

### Contenido del suceso de Windows

Se deriva de la fila de `system_events` del evento disparador (ya persistida por la 003):
`message` (`Option<String>`) + los `<Data>` de `raw_xml`. No se crea nada nuevo en la base de
datos; solo se lee `get_event_by_id` con el `triggering_event_id` de la ocurrencia.

---

## DTO del contrato (`ts-rs` → `src/lib/api/generated/`)

### `EstadoIaWire` — un campo nuevo

```ts
type EstadoIaWire = {
  activa: boolean;
  modelo: string;
  previewAcknowledged: boolean;
  sendWithoutReview: boolean;   // NUEVO (FR-011)
  claveValida: boolean | null;
};
```

Se regenera con `ts-rs`, se añade a `src/lib/api/schemas.ts` (Zod) y al store `ia`
(`get sendWithoutReview`). Rompe en el mapeo Rust→TS y en el schema, en un solo sitio cada uno
(principio XIII).

DTO que cambian de forma en la 006: **`EstadoIaWire`** (campo `sendWithoutReview`) y
**`ExplicacionIaWire`** (campos `sinVolcado`, `sinSuceso` — ver §«Explicación devuelta»).
`OrigenExplicacion`, el etiquetado de `ResultadoExplicacion`, `RevisionAnonimizacionWire` y
`ModeloIaWire` se mantienen. (`AiSettingsWire`, si expone las claves del grupo, gana
`sendWithoutReview` por coherencia — verificar en implementación si ese wire se usa.)
