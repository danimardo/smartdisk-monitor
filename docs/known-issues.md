# Registro de silencios y fallos conocidos

Constitución §XIII: **un fallo preexistente se documenta, no se esconde.** Todo `svelte-ignore`,
`eslint-disable`, `@ts-expect-error` o equivalente en el código debe tener aquí su entrada, y el
comentario del código debe enlazarla por número. `pnpm verify:boundaries` falla si un silencio no
enlaza, o si enlaza a una entrada que no existe.

Esto no es burocracia: un silencio disperso por el código no lo revisa nadie nunca. Reunidos en una
tabla se pueden repasar de una vez y se nota cuándo la lista crece.

## Cómo se usa

En el código:

```svelte
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- Justificación breve. Ver docs/known-issues.md #1 -->
```

En esta tabla, una fila con: el número, qué se silencia, dónde, por qué sigue ahí, desde cuándo y
quién debe resolverlo. Cuando se resuelve, la fila se mueve a **Resueltos** con su fecha, no se
borra: saber que algo estuvo mal y por qué dejó de estarlo tiene valor.

## Abiertos

| # | Qué se silencia | Dónde | Por qué sigue | Desde | Responsable |
|---|---|---|---|---|---|
| 1 | `a11y_no_noninteractive_tabindex` | `CodeOutput.svelte` | El `tabindex` de la región desplazable es **deliberado**: WCAG 2.1.1 exige poder recorrer con teclado una salida larga de `chkdsk`. La regla no distingue un `<pre>` con scroll de un `<div>` decorativo. Lleva `role="region"` y etiqueta accesible | 2026-09-04 | — (no requiere acción) |
| 2 | `a11y_click_events_have_key_events` | `ConfirmDialog.svelte` | El velo solo captura el clic **fuera** del diálogo; el cierre por teclado lo cubre el `onkeydown` de `Escape` de esa misma capa, y el foco entra al panel al abrir (`$effect` + `panel.focus()`, corregido en la Historia 8 — antes el foco se quedaba en el disparador y `Escape` nunca llegaba a burbujear hasta el velo, error detectado al construir el diálogo «Acerca de», T104). La regla no puede ver esa relación | 2026-09-04 | — (no requiere acción) |
| 3 | `a11y_no_noninteractive_element_interactions` y `a11y_no_noninteractive_tabindex` | `TimeSeriesChart.svelte` | El `<svg>` es interactivo a propósito: tiene cursor de lectura con ratón y con teclado (flechas, Inicio, Fin, Escape), como exige `AGENTS.md` §6 para las gráficas. Lleva `role="img"` con `aria-label` que resume el dato | 2026-09-04 | — (no requiere acción) |

**Ninguno de los tres es un defecto pendiente**: los tres son casos donde la herramienta se equivoca
y la alternativa que sugiere sería *peor* para la accesibilidad real. Se registran igualmente,
porque la norma es que no haya silencios sin rastro, no que no haya silencios justificados.

## Resueltos

| # | Qué era | Cómo se resolvió | Fecha |
|---|---|---|---|

*(vacío)*
