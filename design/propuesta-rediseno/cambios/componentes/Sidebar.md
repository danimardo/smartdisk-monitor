# Sidebar

## Antes

```
aside  w-[250px]  material chrome
├─ cabecera 52px: logo 24px + «SmartDisk»
├─ «SUPERVISIÓN» + 6 filas de 32px (punto + etiqueta + contador)
├─ «DISCOS MONITORIZADOS» + N filas (StatusDot + alias + temperatura)
├─ flex-1
└─ pie: texto de estado global + Button «Pausar recopilación»
```

## Después

```
aside  w-[var(--sdm-rail-width)]  (74px)  material chrome
├─ logo 34px (radio nav, degradado de acento, Icon diskStack 18px)
├─ 7 botones de 44px (radio 14): 6 secciones + Acerca de al final
│   └─ cada uno: Icon 19px  +  punto de aviso 6px si procede
├─ flex-1
└─ pie: indicador de estado global, cuadrado de 40px (radio inner) con Icon + contador
```

Fuera: etiquetas de texto, lista de discos, botón de pausa, texto de estado global.

**El botón de pausa** se mueve a Ajustes → Registro de actividad, y además queda accesible desde el menú
contextual del icono de bandeja, que es donde tiene más sentido. No se pierde.

## Props

```ts
interface SidebarProps {
  sections: { id: string; label: string; icon: IconName; badge?: number | null }[];
  active: string;
  globalState: HealthState;      // de worstState() sobre los discos monitorizados
  globalLabel: string;           // «Todo en orden» / «1 necesita atención» / «En pausa»
  globalCount?: number | null;   // se pinta bajo el icono del pie
  onselect?: (id: string) => void;
  onabout?: () => void;
}
```

Fuera: `disks`, `activeDiskId`, `onselectdisk`, `footerNote`, `paused`, `ontogglepause`.

## Accesibilidad — es lo crítico de este cambio

Un riel sin texto solo es aceptable si la accesibilidad es impecable:

- `<nav aria-label="Secciones">` y cada botón con **`aria-label` y `title`** (el `title` da el tooltip
  nativo del sistema, sin componente propio y sin retardo de librería).
- El botón activo lleva `aria-current="page"`.
- El punto de aviso **no** es el único portador: el `aria-label` pasa a «Alertas, 3 sin revisar».
- El indicador del pie: `role="status"` con `aria-label` = `globalLabel`, para que el lector lo anuncie
  cuando cambie.
- Tamaño de objetivo 44 px, por encima del mínimo de 30 px del sistema.

## Selección

Material elevado (`bg-glass-2` + `border-hairline` + `shadow-edge`) e icono en `text-accent`.
**Nunca** una barra de color lateral, según `ui-design.md`.

## Alternativa (si quieres conservar las etiquetas)

Riel expansible: 74 px en reposo, 232 px al pasar el ratón o al recibir foco, superpuesto sobre el
contenido (no empujándolo) con `shadow-lift` y transición de 220 ms en `ease-sdm`. Las etiquetas de
texto aparecen al expandir. Coste: hay que fijar el foco al abrir y cerrar con Escape, y decidir qué pasa
en pantallas táctiles. **No** está en el mockup; si lo prefieres, lo diseño aparte antes de que lo montes.
