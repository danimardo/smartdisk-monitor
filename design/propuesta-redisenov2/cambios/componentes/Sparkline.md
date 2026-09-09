# Sparkline · **NUEVO**

> **Nota (2026-09-09, ADR-051):** el fondo de la cabecera de `DiskCard` que se menciona abajo dejó
> de ser la temperatura de 24 h y pasó a la actividad de disco de ventana corta. La primitiva
> `Sparkline` no cambia; solo qué serie se le pasa.

## Justificación (`ui-design.md` §3)

`TimeSeriesChart` no sirve para esto: trae ejes, leyenda, umbral y pie, y mide 240 px de alto. Aquí hace
falta un trazo de 18-52 px sin ejes ni etiquetas, dentro de `MetricCard` y de la cabecera de `DiskCard`.
Meter un modo «mini» en `TimeSeriesChart` obligaría a condicionar casi todo su marcado; sale más limpio
separar la primitiva de trazado.

De hecho conviene lo contrario: **`TimeSeriesChart` pasa a usar `Sparkline` internamente** para el trazo,
y se queda con los ejes, el umbral, la banda de hueco y el pie. Así la regla de «un hueco es un hueco»
se implementa una sola vez.

## API

```ts
interface SparklineProps {
  points: { t: number; v: number | null }[];   // t en ms epoch; v null = sin dato
  min?: number;                                 // por defecto, mínimo de la serie
  max?: number;                                 // por defecto, máximo de la serie
  color?: string;                               // var(--sdm-*), por defecto var(--sdm-accent)
  height?: number;                              // px, por defecto 22
  fill?: boolean;                               // relleno degradado bajo la curva, por defecto false
  strokeWidth?: number;                         // por defecto 1.8 (2.6 en TimeSeriesChart)
  label?: string;                               // aria-label; si falta, aria-hidden
}
```

## Estructura y las cuatro reglas de trazado

1. **Un `polyline` por tramo continuo.** Los `null` parten la serie; cada tramo se dibuja aparte.
   Jamás se interpola sobre un hueco ni se sustituye por cero.
2. **`vector-effect="non-scaling-stroke"`** en todo trazo. Con `preserveAspectRatio="none"` (necesario
   para que el SVG se estire al ancho del contenedor) el grosor se deforma y a alturas pequeñas el trazo
   desaparece. **Es la causa probable de que la gráfica del detalle salga vacía hoy.**
3. **Eje X por tiempo real, no por índice.** `x = (t - t0) / (t1 - t0) * width`. Las series no son
   equiespaciadas y tratarlas como índice miente sobre cuándo pasó cada cosa.
4. **Rango con margen.** Si `min`/`max` no se pasan, se calculan de la serie y se les da un 8 % de aire
   arriba y abajo, para que la curva no toque los bordes. Si la serie es plana, se centra.

```
<svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" role="img"|aria-hidden>
  <defs><linearGradient>…</linearGradient></defs>   <!-- solo si fill -->
  {#each runs as run}
    <polygon points="…" fill="url(#grad)" />        <!-- solo si fill -->
    <polyline points="…" stroke={color} vector-effect="non-scaling-stroke" />
  {/each}
</svg>
```

El `id` del degradado tiene que ser **único por instancia** (`crypto.randomUUID()` o un contador del
módulo): con varias sparklines en pantalla, un `id` repetido hace que todas usen el primer degradado.

## Estados

| Estado | Cómo queda |
|---|---|
| Serie vacía (`[]`) | No se renderiza nada: el hueco de 22 px se queda en blanco. No dibuja una línea a cero. |
| Serie de un punto | Un punto de 2 px de radio en el centro vertical. |
| Serie toda `null` | Igual que vacía. |
| Con huecos | Tantos trazos como tramos. En `TimeSeriesChart`, además, banda gris y leyenda. |

## Rendimiento

Con más de ~400 puntos se submuestrea a la anchura en píxeles antes de trazar (un mínimo y un máximo por
columna, para no perder los picos). Importa en el detalle con intervalo de 30 días.
