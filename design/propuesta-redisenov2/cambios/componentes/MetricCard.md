# MetricCard

## Antes

```
div (material, radio card, p-4)
├─ etiqueta (text-xs, fg-dim)
├─ valor (text-metric, font-black)   ← peso 800: ya no existe en el sistema
└─ procedencia (text-2xs, fg-faint)
```

## Después

```
div (material, radio card, p-4, gap 9px)
├─ fila: cuadrado 28px (radio nav, bg = tono del icono) con Icon 15px  +  etiqueta
├─ valor .sdm-display 30px
├─ Sparkline height 22 (color = color del icono)
└─ procedencia (text-2xs, fg-faint)
```

## Props

```ts
interface MetricCardProps {
  label: string;
  value: string | null;                          // null ⇒ "No disponible"
  icon: IconName;                                // NUEVA, obligatoria
  state?: HealthState | null;                    // colorea la cifra y el icono
  series?: { t: number; v: number | null }[];    // NUEVA — sin ella, no hay sparkline
  provenance?: string;
}
```

`icon` es obligatoria a propósito: si una métrica no tiene icono claro, probablemente no merece una
`MetricCard` y va como `DataRow`.

## Reglas

- El **peso 800** desaparece (`font-black` → `.sdm-display`, que es 600). Es un arreglo, no un cambio de
  estilo: el sistema declara 600 como máximo y esta era la única pieza que se lo saltaba.
- Cuando `value === null`: cifra «No disponible» a `text-lg` (no a 30 px) en `fg-dim`, sin sparkline,
  y la procedencia explica por qué falta. Aquí sí se escribe la frase completa, al contrario que en
  `DiskCard`: en el detalle hay sitio y el motivo importa.
- La sparkline **nunca** lleva eje ni etiqueta: es contexto, no lectura. Quien quiera el valor exacto
  tiene la gráfica grande debajo.
- El cuadrado de icono usa el `-soft` del estado si hay `state`, y `accent-soft` si no. Un icono en
  `accent` no comunica salud (`ui-design.md`), así que no confunde.

## Estados

| Estado | Cómo queda |
|---|---|
| **Con dato** | Como arriba. |
| **Sin dato** | «No disponible» en `fg-dim`, sin sparkline, procedencia obligatoria. |
| **Cargando** | Etiqueta e icono reales; cifra y sparkline en `bg-glass-3`. Mismo alto. |
| **Obsoleto** | Procedencia en `text-warn` con «última lectura válida hace N min». |
| **No compatible** | Igual que «sin dato», y la procedencia dice el motivo técnico. |
