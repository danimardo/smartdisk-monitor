# StatusPill

## Cambio

Ranura de icono opcional, para que el estado se barra sin leer. Es la pieza que más se repite del sistema
(panel, alertas, eventos, pruebas, historial, chrome), así que el icono se añade **aquí** y no en cada uno.

```ts
interface StatusPillProps {
  state: HealthState;
  label: string;                 // sigue siendo obligatorio
  icon?: IconName | "auto";      // NUEVA — "auto" resuelve por healthIcon[state]
  withDot?: boolean;             // se mantiene; excluyente con icon
}
```

Con `icon="auto"` el mapa `healthIcon` de `$lib/design/icons.ts` decide:
`ok → shield`, `warn → alert`, `crit → bolt`, `unknown → usb`.

## Reglas

- `label` **sigue siendo obligatorio**. El icono acompaña al texto, no lo sustituye: el color nunca es el
  único portador y el icono tampoco.
- El icono va `aria-hidden`: el texto de la píldora ya dice el estado y anunciarlo dos veces molesta.
- `icon` y `withDot` son excluyentes. Si se pasan los dos, gana `icon` (y conviene un aviso en consola
  en desarrollo).
- En `unknown`, `usb` es el icono por defecto porque el 90 % de los casos reales son puentes USB. Cuando
  el motivo sea otro (RAID, disco virtual, colector pausado), la pantalla pasa el icono explícito.

## Sin cambios

Radio cápsula, `padding` 3/10, `text-2xs` peso 600, fondo `-soft` del token y color del token.
Contraste medido en `cambios/00-tokens.md` (columna «píldora»): entre 4,53 y 5,79 según estado y tema.
