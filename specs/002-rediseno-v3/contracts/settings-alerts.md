# Contrato — `settings` (delta) y `set_setting`

Sin comandos Tauri nuevos. Sin permisos nuevos. Todo se persiste con `set_setting(key, value)` (genérico, ya existe) y se lee con `get_settings()` / `get_appearance_settings()`.

## 1. `set_setting` — claves añadidas a la lista blanca

`src-tauri/src/commands` valida hoy una lista cerrada de claves (`set_setting_impl`, `open-questions.md` J.32). Se añaden:

```
settings.onboarding.completedAt          → string ISO-8601 UTC | null
settings.alerts.profile                  → "cautious" | "balanced" | "quiet" | "custom"
settings.alerts.wearWarnPercent          → number en [50, 99]
settings.alerts.wearCritPercent          → number en [wearWarnPercent, 100]
settings.alerts.mediaErrorsWarnPer24h    → integer en [1, 100]   (umbral sobre el incremento por ciclo — clarify Q1)
settings.alerts.mediaErrorsCritPer24h    → integer en [mediaErrorsWarnPer24h, 1000]
settings.alerts.driverRetryWarnPer24h    → integer en [1, 100]   (N configurable de controller_reset/io_retry — clarify Q1)
settings.alerts.driverRetryCritPer24h    → integer en [driverRetryWarnPer24h, 1000]
```

Nombres con sufijo `Per24h` conservados por continuidad con `08b`; la semántica real (Q1) no es una ventana de 24 h y el rótulo de la interfaz no lo dice. Temperatura de fábrica: `tempConfiguredWarnC`/`CritC` pasan a 60/70 (Q2).

Cada clave con:
- validación de rango en `src-tauri/src/domain/ajustes.rs` (serde estricto + comprobación explícita),
- **prueba de rechazo** (valor fuera de rango, tipo incorrecto, crítico < advertencia) — constitución §XI,
- un `AppError` con código estable (`settings.out_of_range` / `settings.invalid_value`) cuando falla.

## 2. Zod — `src/lib/api/schemas.ts`

`appearanceSettings`: sin cambio de forma (solo cambia el valor de fábrica que devuelve el backend).

`settings.alerts` pasa de 7 a 14 campos:

```ts
alerts: z.object({
  profile: z.enum(["cautious", "balanced", "quiet", "custom"]),
  tempConfiguredWarnC: z.number(),
  tempConfiguredCritC: z.number(),
  wearWarnPercent: z.number(),
  wearCritPercent: z.number(),
  capacityWarnPercent: z.number(),
  capacityCritPercent: z.number(),
  capacityAbsoluteFloorMinCapacityBytes: z.number().int(),
  capacityAbsoluteFloorWarnBytes: z.number().int(),
  capacityAbsoluteFloorCritBytes: z.number().int(),
  mediaErrorsWarnPer24h: z.number().int(),
  mediaErrorsCritPer24h: z.number().int(),
  driverRetryWarnPer24h: z.number().int(),
  driverRetryCritPer24h: z.number().int()
})
```

`settings` gana la sección:

```ts
onboarding: z.object({ completedAt: isoUtc.nullable() })
```

`volumeSummary` gana un campo (clarify Q3):

```ts
isSystemVolume: z.boolean()
```

Prueba de rechazo: un `VolumeSummary` sin `isSystemVolume` produce `AppError` `ipc.schema_mismatch` con la ruta del campo.

**Prueba de rechazo obligatoria** para el esquema ampliado: un `settings` al que le falte cualquiera de los campos nuevos, o con `profile` fuera del enum, debe producir `AppError` con código `ipc.schema_mismatch` y la ruta del campo en el detalle. No se rellena con valor por defecto.

## 3. ts-rs — `src-tauri/src/domain/tipos.rs`

`AlertsSettings` (o el struct equivalente) gana los siete campos y `profile: AlertProfile`. `AlertProfile` es un enum `#[derive(Serialize, Deserialize, TS)]` con `cautious | balanced | quiet | custom`. Se regenera `src/lib/api/generated/*.ts` con el flujo de tipos del proyecto (`pnpm` script correspondiente) — no se editan a mano.

## 4. Evento `alerts:changed`

Sin cambio de forma. El motor emite los mismos `AlertGroup`; solo cambian los umbrales con los que decide activar. Las reglas nuevas/parametrizadas usan `ruleKey` existentes (`smart.wear_high`, `smart.media_errors`/`smart.error_log`, `events.controller_reset`/`events.io_retry`) salvo que D12 introduzca claves nuevas, en cuyo caso se añaden a `alert.rule.<key>.title`/`.summary` en los dos diccionarios (ADR-030).

## 5. `getMetricSeries` — sin cambios

El panel lo llama una vez por disco visible (carga perezosa, research.md D2). Firma actual:

```
getMetricSeries({ deviceId, metricKey: "temperature_celsius", fromUtc, toUtc }) → MetricSeries
```

`MetricSeries.points` es `{ t, v: number | null }[]` — encaja directamente con `SparklineProps.points` y `HeroPanelProps.series`.
