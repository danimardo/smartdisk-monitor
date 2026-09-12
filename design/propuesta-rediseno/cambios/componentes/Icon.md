# Icon · **NUEVO**

## Justificación (`ui-design.md` §3)

El catálogo no tiene ninguna pieza que resuelva «símbolo de línea que hereda el color del contexto».
No se puede recomponer: es la unidad más pequeña del rediseño y la usan `MetricCard`, `DiskCard`,
`StatusPill`, `EventRow`, `Sidebar`, `Toolbar`, `HeroPanel` y las tarjetas de prueba.

Alternativa descartada: una fuente de iconos. No sirve porque el exportador de informes trata las
glifos de fuente como texto y se rompen, y porque obligaría a empaquetar una tipografía más.

## API

```ts
interface IconProps {
  name: IconName;        // obligatorio
  size?: number;         // px, por defecto 16
  label?: string;        // si se pasa, role="img" + aria-label; si no, aria-hidden="true"
}
```

```svelte
<Icon name="temp" size={20} />                          <!-- decorativo -->
<Icon name="alert" size={14} label="Advertencia" />     <!-- portador de significado -->
```

**Regla:** si el icono es el único portador de un significado, `label` es obligatorio. Si va junto a un
texto que ya lo dice, se pasa sin `label` y queda `aria-hidden`.

## Estructura

Un único `<svg>` con `<use href="#i-{name}">` que apunta a un **sprite** montado una sola vez en
`AppShell` (`<svg width="0" height="0" style="position:absolute">` con un `<symbol>` por icono).
Ningún icono se inserta como marcado suelto: así no se duplica en cada fila de una `VirtualList`.

Todos los `<symbol>` comparten `viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`,
`stroke-width` `var(--sdm-icon-stroke)` (1,7), `stroke-linecap="round"`, `stroke-linejoin="round"`.
**Cero literales de color:** el color lo pone el contenedor.

## El juego (15)

| `name` | Dibujo | Se usa en |
|---|---|---|
| `temp` | termómetro con bulbo | temperatura, en todas las pantallas |
| `wear` | medidor de aguja | desgaste / porcentaje usado |
| `pulse` | línea de electrocardiograma | actividad, latencia, lectura |
| `clock` | reloj | horas de encendido, frescura, «en pausa» |
| `nvme` | chip con patas | disco NVMe y SATA SSD |
| `hdd` | plato con brazo | disco mecánico |
| `usb` | símbolo USB | disco externo / sin SMART |
| `shield` | escudo con marca | salud correcta, chkdsk, campos anonimizados |
| `alert` | triángulo con exclamación | advertencia, aviso de impacto |
| `bolt` | rayo | crítico, error de E/S, escritura |
| `flask` | matraz | pruebas y benchmark |
| `plug` | conector | sucesos del sistema |
| `diskStack` | pila de discos | logotipo, panel general |
| `check` | marca | casilla marcada, resumen del asistente |
| `tag` | etiqueta | alias de disco |

## Mapas semánticos (única fuente)

Van en `$lib/design/icons.ts`, no repartidos por los componentes:

```ts
export const healthIcon: Record<HealthState, IconName> = {
  ok: "shield", warn: "alert", crit: "bolt", unknown: "usb"
};

export const eventLevelIcon = { error: "bolt", warning: "alert", info: "shield" } as const;

export const busIcon = (deviceType: string): IconName =>
  /usb/i.test(deviceType) ? "usb" : /hdd|spindle|rpm/i.test(deviceType) ? "hdd" : "nvme";

export const testIcon = { benchmark: "flask", chkdsk_scan: "shield", smart_short: "bolt" } as const;
```

## Estados

No tiene. Es una primitiva sin estado, sin interacción y sin foco.
