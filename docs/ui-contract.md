# Contrato UI ↔ backend

Superficie completa entre la interfaz Svelte y el backend Rust. Es normativo: la UI **solo** puede
invocar lo que aparece aquí, y el backend no puede cambiar una firma sin actualizar este documento.

Los tipos se escriben en TypeScript porque es el lado que los consume, pero **la fuente es Rust**:
se generan con `ts-rs` y CI falla si el `.ts` generado no coincide con el del repositorio
(`open-questions.md` G.3). Este archivo documenta la intención; el `.ts` generado documenta la forma.

Convenciones:

- Toda fecha es **ISO 8601 en UTC**. La presentación en hora local es responsabilidad de la UI.
- Todo tamaño es **bytes**; toda temperatura, **grados Celsius**; todo caudal, **bytes por segundo**.
- Un dato que no se ha podido obtener es `null`. **Nunca `0`, nunca `-1`, nunca cadena vacía.**
- Un array vacío significa "ninguno"; `null` en su lugar significa "no se ha podido saber".

---

## 1. Errores

Todo comando que falle rechaza con un `AppError`. No hay excepciones: un `invoke` nunca devuelve
una cadena suelta ni un error de serialización sin envolver.

```ts
interface AppError {
  code: string;                                   // "smartctl.timeout", "db.locked", "test.busy"
  messageKey: string;                             // clave i18n de la frase humana
  messageVars?: Record<string, string | number>;
  detail?: string | null;                         // stderr, código de salida… literal, sin traducir
  source?: MetricSource | null;                   // qué fuente falló, si aplica
  retryable: boolean;                             // ¿tiene sentido repetir la misma acción?
}
```

La UI muestra siempre `t(messageKey, messageVars)` y guarda `detail` dentro de un `<details>`
copiable. Nunca enseña `detail` solo, y nunca oculta `detail` del todo.

**Un fallo de fuente no es un fallo de aplicación.** Si un recopilador cae, se degrada su tarjeta
con el error y el resto de la interfaz sigue funcionando (`AGENTS.md` §5).

### Códigos previstos

| `code` | Cuándo | `retryable` |
|---|---|---|
| `smartctl.not_found` | falta el binario auxiliar | no |
| `smartctl.timeout` | la consulta excedió el tiempo máximo | sí |
| `smartctl.exit_status` | código de salida con bits de error | sí |
| `smartctl.unsupported` | el dispositivo no expone SMART | no |
| `device.not_found` | el `device_id` ya no existe | no |
| `volume.not_found` | el `volume_id` ya no existe | no |
| `test.busy` | ya hay una prueba en ese disco (mismo disco físico subyacente, no solo el mismo id) | no |
| `test.unsupported` | el dispositivo no admite esa prueba | no |
| `test.insufficient_space` | no cabe el archivo con la reserva | no |
| `test.io_failed` | fallo de E/S al preparar o ejecutar la prueba (crear la carpeta, lanzar el proceso auxiliar…) | sí |
| `db.locked` | SQLite ocupado más allá del tiempo de espera | sí |
| `db.migration_failed` | migración fallida; se ha restaurado la copia previa | no |
| `path.invalid` | ruta fuera de las carpetas permitidas | no |
| `export.write_failed` | no se pudo escribir el destino | sí |
| `settings.out_of_range` | valor fuera de los límites de `open-questions.md` D.1 | no |
| `db.query_failed` | fallo de SQLite que no es un bloqueo (`db.locked`, más arriba, es el que sí lo es) | no |
| `windows_storage.failed` | falló la consulta de inventario vía PowerShell | sí |
| `app.log_reload_failed` | no se pudo aplicar en caliente el nuevo nivel de registro | sí |
| `app.open_folder_failed` | no se pudo abrir el explorador de archivos en la carpeta de registro | sí |

---

## 2. Tipos compartidos

Los que ya viven en `src/lib/design/types.ts` no se repiten aquí: `HealthState`,
`Severity`, `AlertStatus`, `TestStatus`, `MetricSource`, `MetricQuality`, `UnknownReason`,
`Provenance`, `DiskSummary`, `VolumeSummary`, `AlertGroup`.

```ts
type Resolution = "raw" | "five_minutes" | "hourly";
type MappingConfidence = "exact" | "inferred" | "unknown";
type IdentityConfidence = "serial" | "fingerprint";

/** Estado de una fuente de datos. Arquitectura §7. */
type SourceStatus = "ok" | "partial" | "unsupported" | "timeout" | "error";

interface SourceHealth {
  source: MetricSource;
  status: SourceStatus;
  lastSuccessAt: string | null;
  lastAttemptAt: string | null;
  error?: AppError | null;
}
```

---

## 3. Comandos

### 3.1 Apariencia y ajustes

```ts
invoke<AppearanceSettings>("get_appearance_settings")
interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "es" | "en" | null;   // null = seguir al sistema
  systemLocale: string;           // BCP-47 de Windows, p. ej. "es-ES". NO usar navigator.language
  useSystemAccent: boolean;       // valor de fábrica: false (v3, ADR-035) — la app estrena paleta propia
}

invoke<WindowsAccent>("get_system_accent_color")   // error si el usuario lo tiene desactivado
interface WindowsAccent {
  hex: string;        // #RRGGBB. OJO: el registro lo guarda en ABGR, no en RGB (open-questions.md O.7)
  palette?: string[]; // los 7 tonos de AccentPalette, del más claro al más oscuro
}

invoke<Settings>("get_settings")
invoke<void>("set_setting", { key: string, value: unknown })   // valida rango; AppError si no cabe
invoke<Settings>("reset_settings", { scope: "all" | "alerts" | "schedule" | "retention" })

interface Settings {
  schedule: {
    metricsFastSeconds: number;   // 30 s de fábrica, 10-300 (D.1)
    smartFullSeconds: number;     // 300 s de fábrica, 60-3600
    eventsSeconds: number;        // 30 s de fábrica, 15-300
    discoverySeconds: number;     // 60 s de fábrica, 30-600
  };
  alerts: {
    profile: "cautious" | "balanced" | "quiet" | "custom";  // "balanced" de fábrica (ADR-036). Elegir un perfil concreto reescribe los 12 umbrales; editar un umbral a mano ⇒ "custom"
    tempConfiguredWarnC: number;  // 60 de fábrica (ADR-036), 40-95 — solo sin límite del fabricante
    tempConfiguredCritC: number;  // 70 de fábrica, entre tempConfiguredWarnC y 100
    wearWarnPercent: number;      // 80 de fábrica, 50-99
    wearCritPercent: number;      // 90 de fábrica, entre wearWarnPercent y 100
    capacityWarnPercent: number;  // 10 de fábrica, 1-50 (C.1/ADR-019)
    capacityCritPercent: number;  // 5 de fábrica, 1-50, menor que capacityWarnPercent
    capacityAbsoluteFloorMinCapacityBytes: number;  // 256 GiB de fábrica: a partir de aquí también cuenta el suelo absoluto
    capacityAbsoluteFloorWarnBytes: number;         // 20 GiB de fábrica
    capacityAbsoluteFloorCritBytes: number;         // 10 GiB de fábrica, menor que el de aviso
    mediaErrorsWarnPer24h: number;  // 1 de fábrica, 1-1000. Nombre histórico: es el incremento del contador que basta para avisar, no una ventana de 24 h (ADR-036)
    mediaErrorsCritPer24h: number;  // 5 de fábrica, entre el de aviso y 1000
    driverRetryWarnPer24h: number;  // 5 de fábrica, 1-1000. Aún sin consumidor: la regla events.* necesita el colector de eventos
    driverRetryCritPer24h: number;  // 12 de fábrica, entre el de aviso y 1000
  };
  onboarding: {
    // ISO-8601 UTC o null. null ⇒ el guardián de `+layout.ts` redirige a `/onboarding` al arrancar,
    // salvo que ya exista configuración previa (FR-043), en cuyo caso lo graba solo. Se escribe con
    // `set_setting("settings.onboarding.completed_at", <fecha>|null)` (clave en la lista blanca desde
    // PR 5). «Repetir la configuración inicial» de Ajustes lo pone a null.
    completedAt: string | null;
  };
  retention: {
    rawDays: number;              // 7 de fábrica, 1-30 (J.14)
    fiveMinutesDays: number;      // 90 de fábrica, 7-365
    hourlyDays: number;           // 730 de fábrica, 90-1825
    freeSpaceWarnBytes: number;   // 1 GiB de fábrica (J.13); sin límites de edición propios
    freeSpaceHaltBytes: number;   // 256 MiB de fábrica
  };
  lifecycle: {
    closeAction: "minimize" | "exit";   // "minimize" de fábrica
    closeActionRemembered: boolean;
    startWithSystem: boolean;     // false de fábrica; al activarlo se registra una tarea programada
                                  //   elevada (`schtasks`, ADR-038). Clave: `lifecycle.start_with_system`
  };
  notifications: {
    soundEnabled: boolean;        // false de fábrica (US-072)
    enabled: boolean;             // true de fábrica; false oculta el toast sin pausar (ADR-037).
                                  //   Clave: `notifications.enabled`
  };
  logging: {
    verbose: boolean;             // ver `set_log_level`, §3.9: no se cambia con `set_setting`
  };
}
```

`Settings` es un objeto tipado, no un diccionario libre. Sus límites están en `open-questions.md`
D.1/C.1/J.13/J.14/J.32 y los valida el backend: la UI puede confiar en que un valor guardado es un
valor legal. La apariencia (`theme`/`language`/`useSystemAccent`) no vive en `Settings`: sigue
teniendo su propio `get_appearance_settings()`; se persiste con el mismo `set_setting(key, value)`
genérico, con las claves `settings.appearance.theme`, `settings.appearance.language` y
`settings.appearance.use_system_accent`. `reset_settings` con `scope: "all"` también restaura
`lifecycle`/`notifications`/`logging`, que no tienen su propio ámbito de reinicio (y al borrar
`lifecycle.start_with_system` también quita la tarea programada de autoarranque); nunca toca la
apariencia ni `settings.onboarding.completedAt`.

### 3.2 Inventario

```ts
invoke<DeviceListResponse>("get_devices")
interface DeviceListResponse {
  devices: DiskSummary[];
  excluded: DiskSummary[];        // desactivados por el usuario; US-011 exige mostrarlos aparte
  sources: SourceHealth[];        // estado de cada recopilador
  paused: boolean;
  pausedSince: string | null;
  historyWriteHalted: boolean;    // FR-020a/b: volumen del historial bajo el umbral de parada
}

invoke<DeviceDetail>("get_device_detail", { deviceId: string })
interface DeviceDetail extends DiskSummary {
  fingerprint: string;
  identityConfidence: IdentityConfidence;
  serialNumber: string | null;
  firmware: string | null;
  busType: string | null;
  capabilities: DeviceCapability[];
  counters: SmartCounter[];       // contadores normalizados con su delta
  smartRaw: SmartRawInfo | null;  // metadatos; el JSON completo se pide aparte
  firstSeenAt: string;
  lastSeenAt: string;
  removedAt: string | null;
}

interface DeviceCapability {
  key: "smart" | "nvme_log" | "self_test_short" | "chkdsk_scan" | "temperature";
  available: boolean;
  reasonKey: string | null;       // por qué no, en lenguaje humano
}

interface SmartCounter {
  metricKey: string;
  value: number | null;
  unit: string | null;
  /** Incremento respecto a la lectura anterior; null si no hay lectura previa. */
  delta: number | null;
  /** Si el incremento es en sí mismo una mala señal. Decide si `DataRow` colorea el delta. */
  deltaIsMeaningful: boolean;
  provenance: Provenance;
}

invoke<void>("set_device_monitoring", { deviceId: string, enabled: boolean })
invoke<void>("set_device_alias", { deviceId: string, alias: string | null })
invoke<void>("refresh_now", { scope: "all" | "device", deviceId?: string })
```

`refresh_now` es idempotente: si ya hay una recopilación igual en curso, devuelve sin encolar otra
(US-013). No es un error; la respuesta lo indica en el evento `metrics:updated` correspondiente.

### 3.3 Series temporales

```ts
invoke<MetricSeries>("get_metric_series", {
  deviceId?: string,
  volumeId?: string,
  metricKey: string,
  fromUtc: string,
  toUtc: string
})

interface MetricSeries {
  metricKey: string;
  unit: string;
  /** Resolución realmente servida, que puede no ser la ideal: la UI la muestra al usuario. */
  resolution: Resolution;
  /** true si se ha submuestreado para respetar el tope de 1.500 puntos. */
  downsampled: boolean;
  /** El intervalo pedido, devuelto tal cual: el eje lo cubre entero aunque falten datos. */
  fromUtc: string;
  toUtc: string;
  /** Cadencia esperada entre puntos; alimenta la detección de huecos del gráfico. */
  expectedIntervalMs: number;
  points: { t: number; v: number | null }[];
  /** Umbrales que la gráfica debe dibujar como línea discontinua. */
  vendorLimit: number | null;
  vendorCritical: number | null;
}
```

La tabla intervalo → resolución está en `open-questions.md` E.1. **Los huecos se devuelven como
huecos**: el backend no interpola ni rellena con ceros, y un tramo sin datos simplemente no trae
puntos.

### 3.4 Alertas

```ts
invoke<AlertGroup[]>("get_alert_groups", { status?: AlertStatus[], deviceId?: string })
invoke<AlertDetail>("get_alert_detail", { alertGroupId: string })

interface AlertDetail extends AlertGroup {
  facts: { labelKey: string; value: string | null }[];
  occurrences: {
    occurredAt: string;
    cycle: number;
    value: number | null;
    eventId: string | null;
    context: string | null;
  }[];
  relatedEvents: SystemEvent[];
}

invoke<void>("acknowledge_alert", { alertGroupId: string })
invoke<void>("mute_alert", { alertGroupId: string, minutes: 15 | 60 | 480 | null })  // null = indefinido
invoke<void>("unmute_alert", { alertGroupId: string })
invoke<void>("archive_alert", { alertGroupId: string })
```

Reconocer **no** cambia el color de nada: el color lo decide `deviceState()` sobre las alertas
`active` y `acknowledged` (`alert-rules.md` §1).

### 3.5 Eventos

```ts
invoke<SystemEventPage>("get_system_events", {
  deviceId?: string, volumeId?: string,
  levels?: ("error" | "warning" | "info")[],
  providers?: string[],
  fromUtc?: string, toUtc?: string,
  cursor?: string, limit?: number        // paginación: la lista se virtualiza
})

interface SystemEvent {
  id: string;
  occurredAt: string;
  provider: string;
  eventId: number;
  level: "error" | "warning" | "info";
  message: string;                       // en el idioma de Windows, no en el de la app
  deviceId: string | null;
  volumeId: string | null;
  mappingConfidence: MappingConfidence;
  hasRawXml: boolean;
}

interface SystemEventPage { events: SystemEvent[]; nextCursor: string | null; total: number | null }

invoke<string>("get_event_raw_xml", { eventId: string })
```

El `message` llega en el idioma de Windows y se muestra tal cual, marcado como texto original del
sistema (`open-questions.md` J.4). **Se renderiza como texto, jamás como HTML.**

### 3.6 Pruebas

```ts
invoke<string>("start_benchmark", {          // devuelve testRunId
  volumeId: string,
  sizeBytes: number,
  blockSizeBytes: number,
  mode: "sequential" | "random",
  passes: number
})
invoke<string>("run_chkdsk_scan", { volumeId: string })
invoke<string>("run_smart_short_test", { deviceId: string })
invoke<void>("cancel_test", { testRunId: string })

invoke<TestRun[]>("get_test_runs", { deviceId?: string, limit?: number })
interface TestRun {
  id: string;
  type: "benchmark" | "chkdsk_scan" | "smart_short";
  deviceId: string | null;
  volumeId: string | null;
  status: TestStatus;
  startedAt: string;
  finishedAt: string | null;
  progressPercent: number | null;
  /** Comando literal ejecutado, para mostrarlo en el ConfirmDialog y en el historial. */
  command: string | null;
  parameters: Record<string, unknown>;
  result: TestResult | null;
  output: string | null;                 // salida capturada; texto, nunca HTML
  /** Página de códigos deducida para `output` ("cp1252", "cp850", "utf-8"…). Los bytes originales
   *  se conservan aparte: la decodificación es de presentación (open-questions.md §Q). */
  outputEncoding: string | null;
  /** Si quedó un archivo temporal sin borrar, su ruta, para que el usuario pueda limpiarla. */
  orphanPath: string | null;
}

interface TestResult {
  passed: boolean | null;
  readBytesPerSecond: number | null;
  writeBytesPerSecond: number | null;
  readLatencyMs: number | null;
  writeLatencyMs: number | null;
  maxTemperatureC: number | null;
  stoppedReason: "completed" | "cancelled" | "thermal" | "space" | "error" | null;
}
```

Parámetros por defecto del benchmark en `product-specification.md` §6. La UI nunca construye el
comando: lo recibe ya formado en `command` solo para mostrarlo.

### 3.7 Informes y diagnóstico

```ts
invoke<string>("export_report", {          // devuelve la ruta escrita
  format: "csv" | "json" | "html",
  fromUtc: string, toUtc: string,
  deviceIds: string[] | null,              // null = todos los monitorizados
  includeSerials: boolean,
  destinationPath: string
})

invoke<DiagnosticPreview>("preview_diagnostic_zip", { includeIdentifiers: boolean })
interface DiagnosticPreview {
  entries: { path: string; sizeBytes: number; descriptionKey: string }[];
  totalBytes: number;
  redactedFields: string[];                // qué se va a anonimizar, para enseñarlo antes de guardar
}
invoke<string>("create_diagnostic_zip", { includeIdentifiers: boolean, destinationPath: string })
```

US-051 exige mostrar un resumen del contenido antes de guardar: para eso está
`preview_diagnostic_zip`, que no escribe nada.

### 3.8 Ciclo de vida

```ts
invoke<void>("pause_monitoring")
invoke<void>("resume_monitoring")
invoke<AppInfo>("get_app_info")            // nombre, versión y autor desde el manifiesto (ADR-011)
invoke<void>("delete_all_data", { confirmationPhrase: string })   // US-073
```

`delete_all_data` exige que el usuario escriba una frase de confirmación, no solo que pulse un
botón: es irreversible y borra el historial completo.

### 3.9 Registro de actividad

```ts
invoke<LogLevel>("get_log_level")
invoke<void>("set_log_level", { verbose: boolean })   // US-071, FR-029a
invoke<void>("open_log_folder")                       // US-071, FR-029b
```

`open_log_folder` abre **una sola ruta conocida** —la carpeta de registro resuelta por
`platform::paths`—, sin recibirla como argumento desde la interfaz: un parámetro de ruta abriría
una segunda vía de acceso al sistema de ficheros, que es justo lo que el principio IX prohíbe.

`log_from_ui` no es de este bloque: es el envoltorio interno que usa `$lib` para escribir en la
única API de registro (constitución §XV); no lo invoca ninguna pantalla directamente.

---

## 4. Eventos emitidos por el backend

La UI **no hace sondeo**. El backend empuja (ADR-015). Cada carga útil lleva `emittedAt` para poder
descartar mensajes fuera de orden.

| Evento | Carga útil | Cuándo |
|---|---|---|
| `metrics:updated` | `{ emittedAt, devices: DiskSummary[], sources: SourceHealth[], historyWriteHalted: boolean }` | al cerrar cada ciclo de recopilación |
| `alerts:changed` | `{ emittedAt, changed: AlertGroup[], removed: string[] }` | alta, cambio de severidad o de estado, resolución |
| `inventory:changed` | `{ emittedAt, added: DiskSummary[], removed: string[], updated: DiskSummary[] }` | alta o retirada de disco o volumen |
| `test:progress` | `{ emittedAt, testRun: TestRun }` | mientras una prueba avanza |
| `source:degraded` | `{ emittedAt, source: SourceHealth }` | una fuente pasa a `timeout` o `error` |
| `system:accent-changed` | `{ hex }` | el usuario cambia el acento de Windows |
| `system:theme-changed` | `{ dark: boolean }` | el usuario cambia el tema de Windows |
| `monitoring:paused` / `monitoring:resumed` | `{ emittedAt, since }` | pausa desde la bandeja o desde la UI |

Los eventos son **incrementales pero autosuficientes**: cada uno trae el objeto completo que ha
cambiado, no un parche. La UI puede reemplazar por identificador sin reconciliar.

Al montar, la UI pide el estado completo con los comandos `get_*` y a partir de ahí solo escucha.
Tras una reconexión o un error de deserialización, vuelve a pedir el estado completo en lugar de
intentar recomponerlo.

---

## 5. Permisos Tauri

La política de capacidades es de mínimo privilegio dentro de un proceso ya elevado (ADR-004):

- **Sin** `shell:allow-execute` genérico. Los procesos auxiliares se lanzan desde Rust con
  ejecutable y argumentos de una lista cerrada.
- **Sin** `fs` genérico. Las rutas se calculan en Rust y se validan canónicamente antes de crear o
  borrar nada.
- El `dialog` de selección de destino es la **única** vía por la que una ruta elegida por el usuario
  entra en el backend, y aun así se valida.
- Cualquier permiso nuevo requiere una entrada en `docs/decisions.md`. La definición de terminado de
  una pantalla incluye "sin permisos Tauri nuevos".
