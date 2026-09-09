# HeroPanel · **NUEVO**

> **Nota (2026-09-09, ADR-051):** la curva de fondo dejó de ser puramente decorativa: al pasar el
> ratón por la **mitad derecha despejada** aparece un globo con la temperatura y la hora del punto
> (y el teclado la recorre entera). El resto de este documento sigue vigente.

## Justificación (`ui-design.md` §3)

Ningún componente del catálogo resuelve «un dato dominante con su serie de fondo y sus acciones».
`Card` es el contenedor, sí, pero el contenido específico —curva a sangre detrás del texto, cifra de
76 px, umbral, banda de hueco, cuatro hechos y dos acciones— no se compone con lo que hay, y aparece en
una sola pantalla con reglas propias de qué disco mostrar. Es un componente de pantalla, como `DiskCard`.

Alternativa descartada: componerlo dentro de `+page.svelte`. Se descarta porque la lógica de selección
(qué disco protagoniza) y la de estado (qué pasa si no hay ninguno en advertencia) merecen prueba propia.

## API

```ts
interface HeroPanelProps {
  disk: DiskSummary | null;                     // el disco protagonista; null → estado vacío
  series: { t: number; v: number | null }[];    // temperatura del disco en el intervalo
  metric?: "temperature" | "wear";              // magnitud protagonista, por defecto temperature
  threshold?: number | null;                    // límite del fabricante, si lo hay
  alert?: AlertGroup | null;                    // alerta que lo explica, si existe
  facts?: { label: string; value: string | null; state?: HealthState; icon: IconName }[];
  onopen?: (diskId: string) => void;            // «Abrir el disco»
  onviewalert?: (alertId: string) => void;      // «Ver la alerta» — solo si alert != null
}
```

Todos los callbacks son opcionales: el componente se puede montar sin ninguno (para el mockup y para
las pruebas de render).

## Elección del protagonista

**No lo decide el componente.** La pantalla le pasa el disco ya elegido, con este criterio
(en `selectHeroDisk()`, junto a `worstState()` en `$lib/design/health.ts`):

1. el disco con la alerta activa de mayor severidad; si hay empate, el de ocurrencia más reciente;
2. si no hay ninguna alerta activa, el disco de sistema (el que contiene el volumen de arranque);
3. si no se sabe cuál es el de sistema, el primero por orden de inventario;
4. **los discos sin SMART nunca son protagonistas**, salvo que sean los únicos.

## Estructura

```
Card (radio card, material, overflow hidden, alto var(--sdm-hero-height))
├─ Sparkline  fill  strokeWidth 2.6  posición absoluta, inset 0     ← fondo
│   └─ + línea de umbral discontinua + banda de hueco
├─ velo de legibilidad: absoluto inset 0, pointer-events none,
│   linear-gradient(90deg, var(--sdm-glass) 0%, var(--sdm-glass) 42%, transparent 68%)
├─ columna izquierda (flex 1)
│   ├─ StatusPill con icono  +  identidad del disco (text-xs, fg-dim)
│   ├─ cifra .sdm-display a var(--sdm-text-hero) + unidad + nota del umbral
│   ├─ explicación humana (text-sm, fg-dim, max-width 520px, text-wrap pretty)
│   └─ acciones: Button primary «Ver la alerta» + secondary «Abrir el disco» + frescura
└─ columna derecha (290px)
    └─ 4 × fila de hecho (bg-glass-3, radio inner, Icon 30px + etiqueta + valor)
```

**Contraste sobre la curva — importante.** El trazo de la serie puede cruzar la zona de texto a cualquier
altura: depende de los datos, no de la composición. Confiar en que «la curva pasa por abajo» no es
aceptable. Por eso entre el SVG y el contenido va un **velo de legibilidad**: un degradado horizontal de
`--sdm-glass` opaco hasta el 42 % del ancho que se desvanece a transparente en el 68 %, con
`pointer-events: none`.

Así el texto siempre se lee sobre material limpio y la curva sigue visible en los dos tercios derechos.
Medido con el velo: `--sdm-text` da 16,4:1 en claro y 13,9:1 en oscuro, independientemente de la serie.
Si se cambian los porcentajes del degradado hay que volver a medir con la curva en su punto más alto.

## Estados

| Estado | Cómo queda |
|---|---|
| **Todo en orden** | Protagonista = disco de sistema. Píldora `ok` «Todo en orden», cifra en `--sdm-text` (no en verde: no es una alarma), serie en `--sdm-accent`, sin línea de umbral, texto «Ningún disco necesita atención ahora mismo». Solo la acción «Abrir el disco». |
| **Advertencia / crítico** | Píldora y cifra en `warn`/`crit`, serie del mismo color, umbral discontinuo, explicación desde la alerta, las dos acciones. |
| **Cargando** | Cifra y curva en bloques `bg-glass-3`; los cuatro hechos con sus etiquetas ya visibles y el valor en esqueleto. |
| **Vacío** (`disk === null`) | El componente no se monta: la pantalla muestra `EmptyState` en su lugar. No hay «héroe vacío». |
| **Sin serie** | Cifra y hechos sí; el fondo queda sin curva y aparece «Sin muestras en las últimas 24 h» en `text-2xs` junto a la frescura. |
| **Dato obsoleto** | Bajo la cifra, «último dato válido a las HH:MM» en `text-warn`. La curva no se extiende hasta «ahora». |
| **Protagonista sin SMART** | Solo si es el único disco: cifra «No disponible» en `fg-dim`, píldora gris, sin curva, y el texto explica que el bus no expone SMART. |
