# Fase 1 — Quickstart de validación: Informe HTML por disco

Escenarios ejecutables que demuestran la feature de extremo a extremo. No incluye código de
implementación; el detalle vive en `contracts/` y `data-model.md`.

## Prerequisitos

- Repo en `F:\Apps\smartdisk`, rama `009-informe-mejorado`.
- `pnpm install` hecho; toolchain de Rust y `cargo` disponibles.
- Para los escenarios con IA en la app real: ayuda con IA configurada (clave propia o
  «Usar la clave de demostración»). Los escenarios automáticos usan el IPC falso, no red.

## Puertas automáticas (deben pasar antes de dar la feature por terminada)

```powershell
pnpm check ; pnpm lint ; pnpm verify ; pnpm test ; pnpm test:component ; pnpm test:e2e ; pnpm test:a11y ; pnpm build ; pnpm docs:check
# desde src-tauri/
cargo fmt --check ; cargo clippy --all-targets -- -D warnings ; cargo test
```

## Escenario 1 — Informe HTML legible por disco, sin IA (Historia 1)

**Unit (Rust)** — `reporting::informe`:
1. Preparar un `conn` en memoria con 3 discos (uno sin SMART, uno con SMART obsoleto, uno normal),
   volúmenes, alertas (`smart.error_log` con ocurrencias en el intervalo; una alerta de evento
   **sin** `target_device_id`), eventos de Windows, y series de temperatura y actividad.
2. Llamar a la generación HTML con un `alertLabels` que traduzca `smart.error_log`.
3. **Esperado**:
   - 3 secciones de disco; la alerta de evento sin objeto **no** aparece en ninguna.
   - El disco normal: salud actual, ≥1 contador con delta numérico, alerta pintada como
     «Entradas nuevas en el registro de errores del disco» (no `smart.error_log`), lista de
     eventos, dos `<svg>` con trazo.
   - El disco con SMART obsoleto: valores con «última lectura: hace X», no «No disponible».
   - El disco sin SMART: firmware/temperatura «No disponible», actividad y eventos sí.
   - La cadena generada **no** contiene `http://`, `https://`, `<script src`, `<link ` ni
     `<img src="http`.

**e2e (Playwright + IPC falso)** — `e2e/ui/reports.spec.ts`:
1. Abrir `/reports`, elegir intervalo, exportar HTML (IPC falso devuelve una ruta).
2. **Esperado**: `export_report` invocado con `format: "html"` y `alertLabels` no vacío;
   `includeAiSummary` ausente o `false`.

**Manual (app real)**:
1. Con la app llevando ≥1 día recopilando, exportar el informe HTML de las últimas 24 h de todos
   los discos y abrirlo **con el equipo sin red** (o el navegador en modo offline).
2. **Esperado**: se ve completo; cada disco con su retrato; las mini-gráficas se dibujan.

## Escenario 2 — Vista previa del resumen con IA (Historia 2)

**Unit (Rust)** — `preview_informe_ia` + anonimización:
1. `conn` con un disco cuyo número de serie, etiqueta de volumen y (vía `COMPUTERNAME`) nombre de
   equipo aparezcan en su payload (p. ej. el mensaje de un suceso menciona la etiqueta de volumen).
2. Llamar a `preview_informe_ia`.
3. **Esperado**: en `discos[0].textoEnviado` la serie, la etiqueta y el equipo aparecen como
   `<SERIE-1>` / `<VOLUMEN-1>` / `<EQUIPO>`; `redactedFields` los lista; `totalLlamadas == 1`;
   **no** se ha llamado a `platform::ia_openrouter` (no hay red).
4. Con 2 discos: `discos[0].textoEnviado` no contiene ningún dato del disco 2.

**e2e**:
1. IPC falso con IA configurada. Marcar «incluir resumen con IA», exportar HTML.
2. **Esperado**: se invoca `preview_informe_ia` **antes** de `export_report`; aparece el modal de
   vista previa con el texto por disco y el aviso del número de llamadas.

## Escenario 3 — Generación con IA, degradación y progreso (Historias 2 y 3)

**e2e** (IPC falso simula las respuestas de IA):
1. IA configurada, 3 discos. Confirmar la vista previa.
2. IPC falso: disco 2 devuelve error de IA; discos 1 y 3 devuelven markdown.
3. **Esperado**:
   - Se reciben 3 eventos `report:progress` (`done` 0,1,2 / `total` 3).
   - `export_report` resuelve con una ruta.
   - El HTML (contenido devuelto por el IPC falso o leído) tiene resumen en los discos 1 y 3 y una
     **nota** «resumen con IA no disponible» en el disco 2.
4. Repetir cancelando tras el `report:progress` del disco 2 (`cancelar_informe`):
   - **Esperado**: `export_report` rechaza con `export.cancelled`; no se escribió fichero; no hubo
     `report:progress` del disco 3.

**Unit (Rust)** — `export_report_impl` con un doble de `chat_completions`:
- Disco `k` → `Err` ⇒ `SeccionDiscoInforme[k].resumen_ia == NoDisponible`, resto `Generado`, HTML
  escrito.
- Bandera de cancelación a `true` antes del disco `k` ⇒ sin escritura, `export.cancelled`, sin
  llamada para `k` ni posteriores.

## Escenario 4 — La IA apagada no toca la red (SC-004)

**e2e / manual**:
1. IA **no** configurada → la casilla «incluir resumen con IA» no se ofrece; exportar HTML.
2. IA configurada, casilla **apagada** → exportar HTML.
3. **Esperado (ambos)**: ninguna llamada a `preview_informe_ia` ni al proveedor; exportación
   inmediata; el HTML no lleva sección de resumen IA.

## Escenario 5 — Accesibilidad y diseño

```powershell
pnpm test:a11y     # /reports en tema claro y oscuro, sin incumplimientos de axe
```
- El modal de vista previa: foco atrapado, `Escape`, devuelve el foco al disparador.
- El progreso: región `aria-live`.
- Textos nuevos en `es` y `en` (`pnpm verify:i18n`).
