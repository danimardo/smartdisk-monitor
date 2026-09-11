# Contrato — evento `report:progress` (nuevo)

Se añade a `docs/ui-contract.md` §4 (eventos emitidos por el backend), misma forma que
`test:progress`.

## Forma

```ts
"report:progress" → {
  emittedAt: string,      // RFC3339
  done: number,           // discos resumidos hasta ahora (0..total)
  total: number,          // discos incluidos en el informe
  deviceLabel: string     // etiqueta del disco que se está resumiendo ahora (para la persona)
}
```

## Cuándo

- Solo durante `export_report` con `format: "html"` e `includeAiSummary: true`.
- Se emite **antes** de la llamada al modelo de cada disco: el primero con `done: 0`, y así hasta
  `done: total - 1` para el último.
- El fin de la exportación lo marca la **resolución de la promesa** de `export_report` (ruta
  escrita) o su **rechazo** (`export.cancelled`, u otro error). No hay un evento de «fin».

## Uso en la interfaz

- La pantalla de Informes muestra «Resumiendo disco {done+1} de {total}: {deviceLabel}» con
  `aria-live`.
- Si no llega ningún `report:progress` en un plazo razonable tras confirmar, la interfaz mantiene
  un indicador indeterminado (la primera llamada puede tardar en arrancar la conexión TLS).

## Pruebas (e2e con IPC falso)

- Confirmar la vista previa con 3 discos → se reciben 3 `report:progress` con `done` 0,1,2 y
  `total` 3.
