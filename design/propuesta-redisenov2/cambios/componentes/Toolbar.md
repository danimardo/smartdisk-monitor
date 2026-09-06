# Toolbar

## Antes

```
header  h-52  material chrome
├─ título (siempre «Panel general» — bug) + subtítulo
├─ flex-1
├─ {controles contextuales de pantalla}
├─ StatusPill de estado global (texto suelto, a veces contradice a la Sidebar)
├─ frescura «hace X»
├─ Button primary «Actualizar»
└─ botón «?» (Acerca de)
```

## Después

```
header  h-56  material chrome
├─ título .sdm-display 19px (de la RUTA) + subtítulo text-2xs
├─ flex-1
├─ StatusPill con Icon (única fuente del estado global)
├─ frescura «hace X»  (text-warn si el dato está obsoleto)
└─ Button primary «Actualizar»
```

Cambios: el alto pasa de 52 a 56 px; el título usa la familia de display; sale el botón «?» (se va al
riel); salen los controles contextuales (van junto a lo que modifican); la píldora gana icono.

## Props

```ts
interface ToolbarProps {
  title: string;                 // del $page.route / de la carga de la ruta
  subtitle?: string;
  globalState: HealthState;
  globalLabel: string;
  freshness?: string;            // «hace 12 s»
  stale?: boolean;               // pinta la frescura en text-warn
  primaryLabel?: string;         // «Actualizar» por defecto
  primaryLoading?: boolean;
  onprimary?: () => void;
}
```

Fuera: `onabout` y la ranura `controls`.

## Corregir a la vez (defecto visible en las capturas)

El título y el estado global salen mal en `detalle-disco__oscuro`: «Panel general» y «Sin discos
monitorizados» en una pantalla que muestra un disco monitorizado. La `Toolbar` tiene que recibir ambos
de la ruta y del mismo `worstState()` que alimenta el riel. **Un solo cálculo, dos presentaciones.**

## Estados

- **Actualizando** — `primaryLoading`: el botón muestra su indicador y queda `aria-disabled`. El resto de
  la barra sigue interactivo; **la UI no se bloquea**.
- **En pausa** — píldora `unknown` «En pausa» con `#i-clock`. El botón Actualizar sigue disponible: una
  lectura manual puntual es legítima con la recopilación pausada.
- **Dato obsoleto** — `stale`: frescura en `text-warn`.
- **Ventana mínima** — ver `cambios/09-chrome-y-estados.md` §5 para el orden de sacrificio.
