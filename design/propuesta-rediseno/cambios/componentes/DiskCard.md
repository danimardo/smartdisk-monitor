# DiskCard

## Antes

```
Card p-4
├─ fila: alias (14.5px) + modelo·tipo (12px)  |  StatusPill
├─ fila de 3 métricas: etiqueta 11.5px + valor 20px  (texto plano)
└─ CapacityBar
```

## Después

```
Card  radio card  overflow hidden  padding 0
├─ CABECERA 52px  (fondo = --sdm-{state}-soft)
│   ├─ Sparkline de temperatura 24h, absoluta inset 0, color del estado, opacidad .85
│   ├─ cuadrado 30px (radio nav) con Icon de bus, fondo = color del estado
│   └─ StatusPill sobre bg-glass (arriba a la derecha)
└─ CUERPO padding 14/16/16
    ├─ alias .sdm-display 15px  +  modelo · tipo (text-2xs, fg-dim)
    ├─ 3 columnas: [Icon 12px + etiqueta 10.5px] sobre cifra .sdm-display 23px
    └─ CapacityBar (sin cambios de API)
```

## Props

```ts
interface DiskCardProps {
  disk: DiskSummary;
  temperatureSeries?: { t: number; v: number | null }[];  // NUEVA — cabecera; sin ella, cabecera plana
  onopen?: (id: string) => void;
}
```

Una sola prop nueva, opcional. El resto de la API no cambia.

## Reglas de contenido

- La cabecera **hereda el color del estado**, no el del bus: un NVMe en advertencia va en ámbar.
- La cifra de temperatura va en `--sdm-warn` **solo** si alcanza el límite del fabricante; si no, en
  `--sdm-text`. El desgaste y la actividad nunca se colorean en la tarjeta (su umbral no es un número
  redondo y colorearlos genera falsas alarmas).
- **Dato ausente**: «—» a `text-xs` en `text-fg-dim`, no «No disponible» a 23 px. El texto completo va
  en el `title` del elemento. Así el disco sin SMART deja de pesar más que el disco con datos, que es
  el defecto que se ve en la captura del panel.
- Si el disco no tiene volúmenes montados, la `CapacityBar` se sustituye por una línea `text-2xs`
  «Sin volúmenes montados», **con la misma altura** que la barra, para que la rejilla no se descuadre.
- La tarjeta entera es un `<button>` con `aria-label` «Abrir {alias}». El foco visible es el anillo del
  sistema (`:focus-visible` de `tokens.css`); no se le añade borde propio.

## Estados

| Estado | Cómo queda |
|---|---|
| **Correcto** | Cabecera `ok-soft`, sparkline verde, cifras en `--sdm-text`. |
| **Advertencia / crítico** | Cabecera `warn-soft`/`crit-soft`, sparkline del color, temperatura coloreada. |
| **Sin datos SMART** | Cabecera `unknown-soft` **sin sparkline** (no hay serie), icono USB gris, píldora «Sin datos SMART», las tres magnitudes «—». La `CapacityBar` sí se muestra. |
| **Cargando** | Cabecera plana `bg-glass-3`; alias y cifras en esqueleto; alto idéntico al real (252 px). |
| **Error de fuente** | El cuerpo se sustituye por `EmptyState kind="error"` compacto; la cabecera se mantiene con el alias, para que se sepa de qué disco se habla. |
| **Dato obsoleto** | Bajo las cifras, «hace N min» en `text-warn` a `text-2xs`. La sparkline termina donde terminan los datos. |
