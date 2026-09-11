# Contrato — `export_report` (ampliado)

Amplía el comando existente (`docs/ui-contract.md` §3.7). CSV y JSON **no cambian**.

## Firma

```ts
invoke<string>("export_report", {
  format: "csv" | "json" | "html",
  fromUtc: string,          // RFC3339
  toUtc: string,
  deviceIds: string[] | null,        // null = todos los monitorizados
  includeSerials: boolean,
  destinationPath: string,
  alertLabels?: Record<string, string> | null,   // NUEVO
  includeAiSummary?: boolean,                     // NUEVO — ignorado si format != "html"
  previewConfirmada?: boolean                     // NUEVO
})
// → devuelve la ruta escrita
```

## Parámetros nuevos

| Parámetro | Aplica a | Comportamiento |
|---|---|---|
| `alertLabels` | `html` | Mapa `clave_de_regla → texto legible` (idioma activo, ~15 claves de `alert-rules.md` §2). El HTML lo usa para las filas de alerta. Clave ausente → se pinta la clave cruda. `null`/omitido → todas crudas (comportamiento actual). |
| `includeAiSummary` | `html` | `true` añade un resumen por disco generado con IA. **Requiere** `previewConfirmada=true` (la interfaz llama antes a `preview_informe_ia`). Solo se ofrece si la ayuda con IA está configurada. |
| `previewConfirmada` | `html` + `includeAiSummary` | La persona ha visto la vista previa de `preview_informe_ia` y confirma. |

## Comportamiento

- **`csv` / `json`**: idéntico a hoy. Los tres parámetros nuevos se ignoran.
- **`html` sin `includeAiSummary`**: **síncrono**, sin red. Genera el HTML por disco (identidad,
  salud, SMART+delta, alertas legibles, eventos, mini-gráficas) y lo escribe. Percepción «al
  instante».
- **`html` con `includeAiSummary=true`**:
  - sin `previewConfirmada=true` → `AppError` `report.preview_required`.
  - con `previewConfirmada=true` → **asíncrono**:
    1. resetea `AppState.informe_cancelado` a `false`;
    2. por cada disco incluido, en serie: emite `report:progress`, comprueba la bandera de
       cancelación, construye el payload **anonimizado** del disco, llama a
       `platform::ia_openrouter::chat_completions`;
    3. un fallo de un disco → ese disco lleva `resumen_ia = NoDisponible`; **se sigue** con los
       demás; **sin reintento**;
    4. si la bandera de cancelación pasa a `true` → **no se escribe** el fichero, devuelve
       `export.cancelled`;
    5. al terminar, ensambla el HTML completo y lo escribe en `destinationPath`.

## Errores

| Código | Cuándo |
|---|---|
| `ipc.schema_mismatch` | `format` desconocido, fechas no RFC3339 |
| `device.not_found` | un `deviceId` no existe |
| `report.preview_required` | `includeAiSummary=true` sin `previewConfirmada=true` |
| `ia.no_key` | `includeAiSummary=true` pero no hay clave (la interfaz no debería llegar aquí) |
| `export.cancelled` | la persona canceló durante la fase de IA |
| `export.write_failed` | fallo al escribir el fichero (reintentable) |

Un fallo de red/cuota/límite/timeout **por disco** **no** es un error del comando: el informe se
genera igualmente con la nota en ese disco.

## Pruebas

- `csv`/`json` sin cambios de salida (regresión).
- `html` sin IA: cada sección de disco contiene identidad, salud, ≥1 contador con delta o «sin
  referencia», alertas legibles (no `smart.error_log`), eventos (o «sin eventos»), dos `<svg>`.
- `html` sin IA: la salida no contiene ninguna URL externa ni `<script src>`/`<link>`.
- `html` con IA + un disco con fallo simulado: HTML escrito, ese disco con la nota, el otro con su
  resumen.
- `html` con IA + cancelación entre discos: no se escribe fichero, `export.cancelled`.
