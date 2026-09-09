# ProgressBar

## Cambio

Una prop nueva para el bloque de prueba en curso, que hoy es el elemento más pequeño de su propia tarjeta.

```ts
interface ProgressBarProps {
  value: number;                          // 0-100
  caption?: string;
  trailing?: string;                      // «quedan 48 s»
  indeterminate?: boolean;
  emphasis?: "inline" | "display";        // NUEVA, "inline" por defecto
}
```

| `emphasis` | Alto | Relleno | Dónde |
|---|---|---|---|
| `inline` (actual) | 10 px | `bg-accent` plano | listas, historial, tarjetas |
| `display` | 12 px | `linear-gradient(90deg, var(--sdm-accent), var(--sdm-accent-hi))` + filo interior | prueba en curso |

La cifra grande **no** va dentro del componente: la pone la pantalla con `.sdm-display` a
`--sdm-text-display` (58 px), porque su posición depende de la composición de la cabecera.

## Sin cambios

`role="progressbar"` con `aria-valuenow/min/max`, el modo indeterminado, y la regla de que **siempre**
lleva texto de estado y tiempo restante: una barra sin leyenda no dice nada.

## Estados

- **En curso**: como arriba.
- **Indeterminado**: un segmento (35 % de la pista) que la **recorre de lado a lado en bucle** —el
  patrón universal de «cargando, sin saber cuánto falta»—, no un relleno fijo que aparenta un
  porcentaje. Antes era un tercio con pulso de opacidad: parecía una barra encallada al 30 %, y
  peor en pantallas como el «Todo listo» del asistente. Con `prefers-reduced-motion` no hay
  recorrido: la pista se llena entera y se atenúa (opacidad 0,5), que sigue leyéndose como
  «activo». La leyenda la pone la pantalla (p. ej. «en curso, sin estimación»).
- **Cancelando**: relleno congelado al 40 % de opacidad y leyenda «cancelando…». El botón Cancelar queda
  deshabilitado con `disabledReason`, para que no se pulse dos veces.
