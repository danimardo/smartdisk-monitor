# Contrato — componentes de catálogo nuevos y recompuestos

Todos: solo tokens, ambos temas, `null` admitido en todo dato opcional, etiqueta accesible, export en `src/lib/components/index.ts`, callbacks opcionales (`= undefined`). Fichas de origen en `design/propuesta-redisenov2/cambios/componentes/`.

## Icon (NUEVO) — PR 2

```ts
interface IconProps {
  name: IconName;      // obligatorio; uno de los 15 del sprite
  size?: number;       // px, 16 por defecto
  label?: string;      // presente ⇒ role="img" + aria-label; ausente ⇒ aria-hidden="true"
}
```

- Renderiza `<svg width={size} height={size}><use href="#i-{name}" /></svg>`.
- El sprite se monta una sola vez en `AppShell` (sin el bloque `<metadata>`).
- Sin estado, sin foco, sin interacción.
- Regla: si el icono es el único portador de significado, `label` es obligatorio (lo vigila la prueba de a11y).

## Sparkline (NUEVO) — PR 4

```ts
interface SparklineProps {
  points: { t: number; v: number | null }[];
  min?: number; max?: number;         // por defecto, de la serie con 8 % de margen
  color?: string;                     // var(--sdm-*), por defecto var(--sdm-accent)
  height?: number;                    // 22 por defecto
  fill?: boolean;                     // relleno degradado, false por defecto
  strokeWidth?: number;               // 1.8 por defecto (2.6 en TimeSeriesChart)
  label?: string;                     // aria-label; si falta, aria-hidden
}
```

Reglas de trazado (constitución §I «un hueco es un hueco»):
1. Un `<polyline>` por tramo continuo; cada `null` corta. Nunca interpola.
2. `vector-effect="non-scaling-stroke"` en todo trazo (con `preserveAspectRatio="none"`).
3. Eje X por tiempo real: `x = (t - t0) / (t1 - t0) * W`.
4. `id` de degradado único por instancia.
5. Submuestreo por encima de ~400 puntos: un mínimo y un máximo por columna de píxel.

Estados: serie `[]` o toda `null` ⇒ no renderiza nada (no dibuja línea a cero); un punto ⇒ punto de 2 px; con huecos ⇒ tantos trazos como tramos.

## HeroPanel (NUEVO) — PR 6

```ts
interface HeroPanelProps {
  disk: DiskSummary | null;                     // null ⇒ el componente no se monta
  series: { t: number; v: number | null }[];
  metric?: "temperature" | "wear";              // "temperature" por defecto
  threshold?: number | null;
  alert?: AlertGroup | null;
  facts?: { label: string; value: string | null; state?: HealthState; icon: IconName }[];
  onopen?: (diskId: string) => void;
  onviewalert?: (alertId: string) => void;
}
```

- Estructura: `Card` (alto `--sdm-hero-height`, overflow hidden) con `Sparkline fill` absoluta de fondo + **velo de legibilidad** (`linear-gradient(90deg, var(--sdm-glass) 0%, var(--sdm-glass) 42%, transparent 68%)`, `pointer-events:none`) + columna izquierda (píldora con icono, cifra `.sdm-display` a `--sdm-text-hero`, explicación, acciones) + columna derecha de 290 px con 4 hechos sobre `bg-glass-3`.
- La elección del disco protagonista **no** vive aquí: la pantalla pasa el disco ya elegido por `selectHeroDisk()`, cuya regla de «todo en orden» usa `VolumeSummary.isSystemVolume` (campo nuevo del backend, clarify Q3).
- Estados: «todo en orden» (píldora `ok`, cifra en `--sdm-text`, solo «Abrir el disco»), «advertencia/crítico» (color del estado, umbral, dos acciones), cargando (bloques `bg-glass-3`), sin serie («Sin muestras en las últimas 24 h»), dato obsoleto («último dato válido a las HH:MM»), protagonista sin SMART (solo si es el único: «No disponible» en gris).

## StatusPill (recompuesto) — PR 2

```ts
interface StatusPillProps {
  state: HealthState;
  label: string;                 // sigue obligatorio
  icon?: IconName | "auto";      // NUEVA — "auto" ⇒ healthIcon[state]
  withDot?: boolean;             // excluyente con icon
}
```

Icono `aria-hidden` (el texto ya dice el estado). `icon` y `withDot` juntos ⇒ gana `icon` + aviso en consola en dev.

## MetricCard (recompuesto) — PR 6

```ts
interface MetricCardProps {
  label: string;
  value: string | null;          // null ⇒ "No disponible" a text-lg en fg-dim, sin sparkline
  icon: IconName;                 // NUEVA, obligatoria
  state?: HealthState | null;
  series?: { t: number; v: number | null }[];   // NUEVA
  provenance?: string;
  age?: string | null;           // se mantiene
}
```

Cambio no estético: `font-black` (peso 800) → `.sdm-display` (600). Es la única pieza del sistema que se saltaba el máximo de 600.

## DiskCard (recompuesto) — PR 6

```ts
interface DiskCardProps {
  disk: DiskSummary;
  href?: string;                                 // se mantiene
  temperatureSeries?: { t: number; v: number | null }[];  // NUEVA — sin ella, cabecera plana
}
```

Cabecera de 52 px que hereda el color del estado (no el del bus). Dato ausente ⇒ «—» a `text-xs` en `text-fg-dim`, texto completo en `title` (no «No disponible» a 23 px).

## ProgressBar (recompuesto) — PR 7

```ts
interface ProgressBarProps {
  value: number; caption?: string; trailing?: string; indeterminate?: boolean;
  emphasis?: "inline" | "display";   // NUEVA — "inline" por defecto
}
```

`display`: alto 12 px, relleno `linear-gradient(90deg, var(--sdm-accent), var(--sdm-accent-hi))`, filo interior. La cifra grande (58 px) la pone la pantalla, no el componente.

## Button (recompuesto) — PR 1

Único cambio: variante `primary` escribe `text-fg-onAccent`, no `text-white`. Sin cambios de variantes, tamaños, radios ni movimiento.

## Sidebar / Toolbar (reescritos) — PR 3

`Sidebar`: props pasan a `{ sections: {id,label,icon,href,badge?}[], active, globalState, globalLabel, globalCount?, onabout? }`. Fuera: `disks`, `activeDiskId`, `diskHref`, `footerNote`, `paused`, `ontogglepause`.

`Toolbar`: props pasan a `{ title, subtitle?, globalState, globalLabel, freshness?, stale?, primaryLabel?, primaryLoading?, onprimary? }`. Fuera: `onabout`, `controls`.

`title`/`subtitle` vienen de la ruta (`$page.data`/`$page.route`), corrigiendo el bug de «Panel general» fijo.
