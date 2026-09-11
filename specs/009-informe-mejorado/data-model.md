# Fase 1 — Modelo de datos: Informe HTML por disco

**No hay cambios de esquema SQLite.** Las entidades nuevas son tipos de dominio en Rust y tipos de
contrato (wire) que cruzan la frontera IPC. La generación del informe **lee** tablas que ya
existen.

---

## 1. Lectura de tablas existentes (sin cambios)

| Tabla | Uso en el informe |
|---|---|
| `devices` | Identidad, tipo, bus, firmware, número de serie (si `includeSerials`), `first_seen_at` (antigüedad). |
| `volumes` + `device_volume_links` | Volúmenes de cada disco: etiqueta, letras, capacidad, espacio libre. |
| `metric_samples` / `metric_aggregates` | Contadores SMART (valor final + base para el delta), series de temperatura y actividad para las mini-gráficas y el resumen numérico. |
| `alert_groups` | Alertas cuyo `target_device_id` es el disco y con ocurrencia en `[desde, hasta]`. |
| `alert_occurrences` | (Vía `repo_alertas`) primera/última ocurrencia y recuento, ya en `alert_groups`. |
| `system_events` | Eventos del disco (`device_id`) en `[desde, hasta]`: fecha, proveedor, id, nivel, mensaje. |

**Alertas del informe**: `repo_alertas::list_groups(conn)` filtrado por
`a.target_device_id == Some(device.id)` **y** `alerta_en_rango(a, desde, hasta)` (helper que ya
existe en `reporting::informe`). Las alertas con `target_device_id == None` (nacidas de sucesos de
volumen sin correlación de disco) **no** entran (Clarificación 2026-09-10).

---

## 2. Tipos de dominio nuevos (Rust, `reporting/` y `domain/ia.rs`)

### 2.1 `SeccionDiscoInforme` — `reporting/informe.rs`

Todo lo que se pinta de un disco. Estado en memoria durante la generación; no se persiste.

| Campo | Tipo | Notas |
|---|---|---|
| `identidad` | `IdentidadDisco` | alias/modelo, tipo, bus, firmware, serie (opt), antigüedad en meses |
| `salud` | `SaludActual` | estado, temperatura, desgaste, horas — cada uno con `Frescura` |
| `volumenes` | `Vec<VolumenInforme>` | etiqueta, letras, capacidad, libre, % libre |
| `contadores` | `Vec<ContadorConDelta>` | clave, valor final, delta o `SinReferencia` |
| `alertas` | `Vec<AlertaLegible>` | descripción (del mapa inyectado), severidad, estado, primera, última, veces |
| `eventos` | `EventosSeccion` | hasta 50 `EventoInforme` + `omitidos: u32` |
| `minigrafica_temp` | `Option<String>` | SVG embebido, o `None` si no hay datos |
| `minigrafica_actividad` | `Option<String>` | idem |
| `resumen_ia` | `Option<ResumenIaSeccion>` | solo si `includeAiSummary`; ver 2.5 |

### 2.2 `Frescura` — `reporting/informe.rs`

| Variante | Significado |
|---|---|
| `Actual { valor }` | última lectura dentro del intervalo o muy reciente |
| `Obsoleta { valor, hace_texto }` | última lectura antigua → «última lectura: hace X» |
| `Nunca` | jamás hubo lectura → «No disponible» |

Regla: `Obsoleta` cuando la última muestra es anterior al inicio del intervalo **o** más vieja que
`3 ×` la cadencia esperada de esa métrica (mismo umbral de «dato obsoleto» del detalle de disco).

### 2.3 `ContadorConDelta` — `reporting/informe.rs`

| Campo | Tipo | Notas |
|---|---|---|
| `clave` | `String` | `error_log_entries_total`, `media_errors_total`, … |
| `etiqueta` | `String` | del mapa inyectado (`smart.counter.*`), o la clave si falta |
| `valor_final` | `Option<f64>` | última muestra en el intervalo |
| `delta` | `DeltaContador` | `Valor(f64)` \| `SinReferencia` |

### 2.4 `ResumenMetrico` — `reporting/resumen_metricas.rs`

| Campo | Tipo |
|---|---|
| `minimo` / `media` / `maximo` / `pico` | `Option<f64>` (`None` = «sin datos») |
| `muestras` | `u32` |
| `resolucion` | `Resolution` (`raw`/`five_minutes`/`hourly`) |

`pico` = `maximo` para temperatura; para actividad es el máximo de la serie (coherente con el
«pico» que ya muestra la aplicación).

### 2.5 `ResumenIaSeccion` — `reporting/informe_ia.rs`

| Variante | Contenido |
|---|---|
| `Generado { markdown, modelo_usado }` | respuesta del modelo, ya validada como markdown seguro |
| `NoDisponible { motivo }` | clave i18n del error (`ia.rate_limited`, `ia.network`, `ia.quota`, …) |

### 2.6 `DetalleInforme<'a>` — `domain/ia.rs`

Payload de **un** disco para el modelo. **Ya anonimizado** por quien lo construye (el comando);
este módulo no conoce el número de serie ni la etiqueta de volumen.

| Campo | Tipo | Notas |
|---|---|---|
| `contexto` | `ContextoDisco<'a>` | ya existe: modelo, tipo, bus, firmware, antigüedad |
| `alertas` | `Vec<AlertaParaModelo<'a>>` | descripción legible, severidad, primera/última, veces |
| `contenidos_suceso` | `Vec<&'a str>` | `extraer_contenido_suceso` de cada suceso que originó una alerta |
| `contadores` | `Vec<ContadorSmart<'a>>` | ya existe: nombre, valor, unidad, significativo |
| `resumen_temp` | `ResumenMetricoTexto` | mín/media/máx/pico o «sin datos» |
| `resumen_actividad` | `ResumenMetricoTexto` | idem |
| `intervalo` | `(&'a str, &'a str)` | desde/hasta en hora local, para que el modelo lo cite |

---

## 3. Tipos de contrato nuevos (wire, `commands/mod.rs` + `src/lib/api`)

### 3.1 `export_report` — parámetros añadidos

```ts
invoke<string>("export_report", {
  format: "csv" | "json" | "html",
  fromUtc: string, toUtc: string,
  deviceIds: string[] | null,
  includeSerials: boolean,
  destinationPath: string,
  alertLabels?: Record<string, string> | null,   // NUEVO — clave de regla → texto legible
  includeAiSummary?: boolean,                     // NUEVO — solo aplica a format "html"
  previewConfirmada?: boolean                     // NUEVO — obligatorio si includeAiSummary
})
```

Reglas: `alertLabels`/`includeAiSummary`/`previewConfirmada` se **ignoran** para `csv`/`json`.
`includeAiSummary=true` sin `previewConfirmada=true` → `AppError` `report.preview_required`.

### 3.2 `preview_informe_ia` — comando nuevo (sin red)

```ts
invoke<PreviewInformeIaWire>("preview_informe_ia", {
  fromUtc: string, toUtc: string,
  deviceIds: string[] | null,
  alertLabels: Record<string, string> | null
})

type PreviewInformeIaWire = {
  discos: {
    deviceLabel: string;          // etiqueta ya mostrable (no anonimizada — es para la persona)
    textoEnviado: string;         // system + user, exactamente lo que saldría, ya anonimizado y recortado
    fragmentos: FragmentoDudosoWire[];   // ya existe (spec 006)
    recortado: boolean;
  }[];
  redactedFields: string[];       // categorías anonimizadas en todo el informe (i18n keys)
  totalLlamadas: number;          // = discos.length; la interfaz avisa si es alto
};
```

### 3.3 `cancelar_informe` — comando nuevo

```ts
invoke<void>("cancelar_informe")   // pone la bandera; el export en curso corta entre discos
```

### 3.4 Evento `report:progress` — nuevo (`docs/ui-contract.md` §4)

```ts
{ emittedAt: string, done: number, total: number, deviceLabel: string }
```

Se emite antes de la llamada de cada disco. `done == total` en el último; el fin de la exportación
lo marca la resolución de la promesa de `export_report`.

---

## 4. Estado del proceso

### 4.1 `AppState.informe_cancelado: std::sync::Arc<std::sync::atomic::AtomicBool>`

- Uno solo: una exportación de informe a la vez (la interfaz deshabilita el botón mientras corre).
- `export_report` lo pone a `false` al empezar la fase de IA; `cancelar_informe` lo pone a `true`;
  el bucle por disco lo comprueba antes de cada llamada.
- **No se persiste** (como `test_cancel_flags`, `paused`).

### 4.2 `settings` — sin claves nuevas de IA

La casilla «incluir resumen con IA» es **estado de la pantalla de Informes**, no un ajuste
persistido: cada exportación decide. Se reutilizan `settings.ai.preview_acknowledged` y
`settings.ai.send_without_review` existentes (el «acknowledged» de la vista previa vale para ambos
usos de la IA; la enmienda XVI 1.11.0 dice «la vista previa … se muestra igual», y una sola
confirmación por informe).

---

## 5. Reglas de validación / invariantes

- **Alcance del payload IA** (probado): `DetalleInforme` de un disco **nunca** contiene datos de
  otro disco; el comando construye y envía uno por uno.
- **Anonimización antes de salir**: el texto que llega a `chat_completions` ha pasado por
  `Anonimizador` (con series + etiquetas de volumen) **y** `redactar_identificadores`. Test: un
  número de serie / etiqueta de volumen / nombre de equipo presente en el payload aparece como
  marcador en `textoEnviado` de `preview_informe_ia`.
- **Degradación**: si `chat_completions` devuelve `Err` para el disco `k`, `SeccionDiscoInforme.resumen_ia`
  del disco `k` es `NoDisponible`; los discos `≠ k` conservan su `Generado`; el HTML se escribe.
- **Cancelación**: si `informe_cancelado` está a `true` antes de la llamada del disco `k`, no se
  llama a `k` ni a los siguientes, **no se escribe** el fichero, se devuelve `export.cancelled`.
- **HTML autónomo** (probado con verificador o test): el HTML generado no contiene `http://`,
  `https://`, `<link`, `<script src`, `<img src` con URL externa; las mini-gráficas son `<svg>`
  en línea.
- **Alertas**: solo `target_device_id == disco`; una alerta de evento sin objeto no aparece en
  ninguna sección.
- **Frescura**: una magnitud con última lectura anterior al intervalo → `Obsoleta` con
  `hace_texto`, nunca `Nunca`.
