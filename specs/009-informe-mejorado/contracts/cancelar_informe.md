# Contrato — `cancelar_informe` (nuevo)

## Firma

```ts
invoke<void>("cancelar_informe")
```

## Comportamiento

Pone `AppState.informe_cancelado` (`Arc<AtomicBool>`) a `true`. La exportación de informe en curso
(`export_report` con `includeAiSummary=true`) comprueba esa bandera **antes de la llamada de cada
disco**; si está a `true`, corta, **no escribe** el fichero y devuelve `export.cancelled`.

- Es inofensivo llamarlo sin exportación en curso (la próxima exportación resetea la bandera).
- Una sola exportación de informe a la vez (la interfaz deshabilita el botón mientras corre); no
  hace falta identificar cuál se cancela.
- **No** aborta una petición HTTP ya en vuelo del disco actual: esa se deja terminar (su respuesta
  se descarta) y el corte ocurre antes del siguiente disco. Con el límite de tiempo de
  `chat_completions`, el retardo máximo es ese límite.

## Precedente

Mismo patrón que `test_cancel_flags` + la cancelación de benchmark/chkdsk, pero con una única
bandera (una exportación a la vez, no un mapa por prueba).

## Pruebas

- `export_report` con IA + 3 discos, `cancelar_informe` tras el `report:progress` del disco 2 →
  no se escribe fichero, `export.cancelled`, no hay llamada para el disco 3.
