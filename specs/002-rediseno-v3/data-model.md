# Phase 1 — Modelo de datos

No hay tablas nuevas. Todo son claves de la tabla `settings` (clave → JSON) y vocabulario de presentación. Las series temporales de métricas ya existen y no cambian.

## 1. `settings.appearance` — cambio de valor de fábrica

| Clave | Tipo | Hoy | v3 | Nota |
|---|---|---|---|---|
| `useSystemAccent` | `boolean` | **`true`** de fábrica | **`false`** de fábrica | El campo ya existe en `AppearanceSettings` (Zod + ts-rs). Solo cambia el valor por defecto y el texto asociado. |

## 2. `settings.onboarding` — sección nueva (una clave)

| Clave | Tipo | Fábrica | Validación | Nota |
|---|---|---|---|---|
| `completedAt` | `string` ISO-8601 UTC \| `null` | `null` | fecha válida o nulo | `null` ⇒ mostrar el asistente al arrancar. Se graba al terminar o al omitir cualquier paso. «Repetir configuración inicial» lo pone a `null`. Instalaciones con configuración previa se marcan como completadas en el primer arranque tras actualizar (comprobación en `+layout.ts`, no migración de backend). |

Alta en la lista blanca de `set_setting` (`src-tauri/src/commands`) y en `docs/ui-contract.md` §3.

## 3. `settings.alerts` — claves nuevas y perfiles

### 3.1 Claves existentes (no cambian de nombre)

`tempConfiguredWarnC`, `tempConfiguredCritC`, `capacityWarnPercent`, `capacityCritPercent`, `capacityAbsoluteFloorMinCapacityBytes`, `capacityAbsoluteFloorWarnBytes`, `capacityAbsoluteFloorCritBytes`.

Cambio de comportamiento: `capacityState()` (`src/lib/design/health.ts` y su gemela de `src-tauri/src/domain/espacio.rs`) pasa a **leer estos valores de `settings`** en vez de las constantes `CAPACITY_ABSOLUTE_FLOOR_MIN_BYTES` etc. Sin cambio de firma pública.

### 3.2 Claves nuevas

| Clave | Tipo | Rango permitido | Fábrica (= perfil Equilibrado) | Regla que parametriza |
|---|---|---|---|---|
| `profile` | `"cautious" \| "balanced" \| "quiet" \| "custom"` | enum | `"balanced"` | — (identificador) |
| `wearWarnPercent` | `number` | 50–99 | 80 | `smart.wear_high` (advertencia) |
| `wearCritPercent` | `number` | `wearWarnPercent`–100 | 90 | `smart.wear_high` (crítico) |
| `mediaErrorsWarnPer24h` | `number` int | 1–100 | 1 | umbral sobre la **magnitud del incremento** de `media_errors_total` por ciclo (`smart.media_errors` / `smart.error_log`). Nombre histórico; la UI no muestra «/24 h» (clarify Q1) |
| `mediaErrorsCritPer24h` | `number` int | `…Warn`–1000 | 5 | id. (crítico) |
| `driverRetryWarnPer24h` | `number` int | 1–100 | 5 | la `N` (hoy fija) de `events.controller_reset` / `events.io_retry`, ahora configurable. Nombre histórico (clarify Q1) |
| `driverRetryCritPer24h` | `number` int | `…Warn`–1000 | 12 | id. (crítico) |

Rangos derivados de `08b-perfiles-de-alerta.md` ampliados con un margen razonable (mismo criterio que J.32 para los umbrales existentes). Los rangos exactos se fijan en el PR 5 y se documentan en `docs/data-model.md`.

### 3.3 Tabla perfil → valores

De `cambios/08b-perfiles-de-alerta.md`. Los valores de capacidad y temperatura se aplican a las claves existentes.

| Clave | Prudente (`cautious`) | Equilibrado (`balanced`) | Solo lo grave (`quiet`) |
|---|---|---|---|
| `tempConfiguredWarnC` | 55 | 60 | 70 |
| `tempConfiguredCritC` | 65 | 70 | 80 |
| `wearWarnPercent` | 70 | 80 | 90 |
| `wearCritPercent` | 85 | 90 | 95 |
| `capacityWarnPercent` | 15 | 10 | 5 |
| `capacityAbsoluteFloorWarnBytes` | 30 GiB | 20 GiB | 10 GiB |
| `capacityCritPercent` | 8 | 5 | 2 |
| `capacityAbsoluteFloorCritBytes` | 15 GiB | 10 GiB | 5 GiB |
| `mediaErrorsWarnPer24h` | 1 | 1 | 5 |
| `mediaErrorsCritPer24h` | 3 | 5 | 15 |
| `driverRetryWarnPer24h` | 2 | 5 | 12 |
| `driverRetryCritPer24h` | 6 | 12 | 30 |

> **Nota (clarify Q2)**: la temperatura de «Equilibrado» (60/70) pasa a ser también el **valor de fábrica**, en lugar de los 70/80 que hoy tiene `temp.above_configured_warn/crit`. Decisión adoptada: se actualiza `docs/alert-rules.md` §2 (la tabla pasa a decir «valor configurado», no «> 70 °C»), sus pruebas del motor, y `docs/open-questions.md`.

### 3.4 Reglas de aplicación (FR-046–FR-050)

1. Elegir perfil ⇒ escribir los 12 valores **y** `profile`.
2. Editar a mano un umbral en Ajustes ⇒ `profile = "custom"`; UI muestra «Personalizado (a partir de \<perfil anterior\>)» — el «perfil anterior» se guarda como parte del literal mostrado, derivado en la UI del último `profile` no-`custom` conocido (no se persiste un segundo campo).
3. Límite del fabricante manda: advertencia térmica al `min(vendorTempLimitC, tempConfiguredWarnC)`; `temp.above_vendor_limit` sigue activa.
4. Espacio libre: gana el criterio (porcentaje o absoluto) que salte primero.
5. Discos sin SMART: no participan de temperatura, desgaste ni errores de medios; sí de capacidad y eventos; nunca avería.

## 3.5 `VolumeSummary.isSystemVolume` — campo nuevo (clarify Q3)

| Campo | Tipo | Nota |
|---|---|---|
| `isSystemVolume` | `boolean` | `true` para el volumen que contiene el sistema operativo. Lo calcula el backend (ya distingue el sistema de archivos y `chkdskAvailable` por volumen). Zod: `z.boolean()`. ts-rs: campo del struct `VolumeSummary`. Prueba de rechazo: un `VolumeSummary` sin el campo produce `ipc.schema_mismatch`. |

`selectHeroDisk()` regla 2 lo consume; ningún componente lo infiere.

## 4. Vocabulario de presentación

### 4.1 `$lib/design/icons.ts` (NUEVO)

```
IconName = "temp" | "wear" | "pulse" | "clock" | "nvme" | "hdd" | "usb"
         | "shield" | "alert" | "bolt" | "flask" | "plug" | "diskStack" | "check" | "tag"

healthIcon:     Record<HealthState, IconName>  = { ok:"shield", warn:"alert", crit:"bolt", unknown:"usb" }
eventLevelIcon: { error:"bolt", warning:"alert", info:"shield" }
busIcon(deviceType): IconName  — /usb/i → "usb"; /hdd|spindle|rpm/i → "hdd"; resto → "nvme"
testIcon: { benchmark:"flask", chkdsk_scan:"shield", smart_short:"bolt" }
```

### 4.2 `$lib/design/health.ts` (AMPLIADO)

```
selectHeroDisk(disks: DiskSummary[], alerts: AlertGroup[], systemDiskId: string | null): DiskSummary | null
```

Criterio (de `HeroPanel.md`): (1) disco con alerta activa de mayor severidad, empate → `lastOccurredAt` más reciente; (2) disco que contiene un volumen con `isSystemVolume === true`; (3) primero del inventario; (4) los `unknown` sin SMART nunca, salvo que sean los únicos. Devuelve `null` solo si no hay discos.

`worstState()` y `deviceState()` no cambian.

### 4.3 Estado global (una sola derivación en `+layout`)

```
globalState: HealthState = worstState(discos monitorizados .map(d => d.state))   // unknown-por-USB no cuenta
globalLabel: string  — "Todo en orden" | "{n} necesita/n atención" | "En pausa"
globalCount: number | null
```

Se pasa igual a `Toolbar` (píldora con icono) y a `Sidebar` (icono + contador, sin texto).

## 5. Tokens nuevos (`tokens.css` + `tailwind.config.cjs` + `tokens.json`)

Ver `cambios/00-tokens.md` para los valores exactos. Resumen de lo que se añade:

| Token | Valor | Tailwind |
|---|---|---|
| `--sdm-font-display` | `var(--sdm-font-sans)` | `font-display` |
| `--sdm-text-display` | `3.625rem` (58 px) | `text-display` |
| `--sdm-text-hero` | `4.75rem` (76 px) | `text-hero` |
| `--sdm-tracking-display` | `-0.04em` | (en `.sdm-display`) |
| `--sdm-rail-width` | `74px` | `w-rail` |
| `--sdm-hero-height` | `246px` | `h-hero` |
| `--sdm-icon-stroke` | `1.7` | — |
| `--sdm-icon-size` | `24px` | — |

Lo demás son **cambios de valor** de tokens existentes (paleta Ciruela): `--sdm-bg`, `--sdm-bg-2`, `--sdm-glass-3`, `--sdm-solid`, `--sdm-hairline`, `--sdm-scrim`, `--sdm-text*`, `--sdm-accent*`, `--sdm-crit*`, `--sdm-unknown*`, `--sdm-on-accent` (oscuro), `--sdm-shadow*`. Cada uno con su ratio medido registrado en `docs/open-questions.md`.

Utilidad nueva en `tokens.css`:

```css
.sdm-display {
  font-family: var(--sdm-font-display);
  font-weight: var(--sdm-weight-semibold);
  font-variant-numeric: tabular-nums;
  letter-spacing: var(--sdm-tracking-display);
  line-height: 1;
}
```

## 6. Claves i18n

Ver `contracts/i18n-claves.md` para la lista completa (es + en). Bloques: `dashboard.hero.*`, `dashboard.spread.*`, `onboarding.*` (pasos, discos, alertas, perfiles, skip), `settings.appearance.useSystemAccent*` (reescritura), `settings.alerts.profile.*`, `alert.rule.*` nuevas si D12 añade reglas, `nav.about` (ya existe), `chart.*` (eje/hueco/leyenda si faltan).
