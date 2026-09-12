# Quickstart — Validación: Eventos de Windows en el detalle de disco

## Prerrequisitos

- `pnpm install` ya ejecutado.
- Un disco real con algún evento de Windows asociado, o los fixtures de `e2e/ui/fixtures/respuestas.ts`
  (`eventos`, `paginaEventos`, `detalleDisco0`) para validar sin hardware real.

## Validación automática (referencia)

```powershell
pnpm check
pnpm test:e2e -- disk-detail
```

Casos que debe cubrir `e2e/ui/disk-detail.spec.ts` (ver `spec.md`, User Stories 1-3):

1. Un disco con eventos asociados muestra hasta 5, del más reciente al más antiguo, con nivel,
   mensaje, proveedor, ID y hora.
2. Un evento con `mappingConfidence` distinta de `exact` lleva la etiqueta de asociación no
   confirmada (`es["events.inferredMapping"]`).
3. Pulsar un evento navega a `/events?focus={id}` con ese suceso ya resaltado y su detalle abierto.
4. Pulsar "Ver todos" navega a `/events?deviceId={id}` con el filtro de dispositivo ya aplicado.
5. Un disco sin eventos asociados (`get_system_events` devolviendo una página vacía) muestra el
   estado vacío diseñado de la sección, sin ocultarla.
6. Un fallo de `get_system_events` (`__rechazar__`) muestra el estado de error de la sección sin
   afectar a la cabecera, las métricas, las gráficas ni los contadores.

## Validación manual (con la aplicación real)

1. `pnpm app:dev` (pide UAC: la app va elevada).
2. Abrir el panel general y entrar en el detalle de un disco que tenga eventos recientes en
   `/events` (compruébalo primero en esa pantalla).
3. Bajar hasta el final del detalle del disco: debe aparecer la sección de eventos con los mismos
   sucesos que se ven en `/events` filtrados a ese disco, más recientes primero.
4. Pulsar sobre un evento: debe abrir `/events` con ese suceso ya resaltado y su panel de detalle
   (XML y, si la IA está activa, «Explícamelo con IA») ya abierto.
5. Volver atrás, pulsar «Ver todos»: debe abrir `/events` con el filtro de dispositivo ya aplicado a
   ese disco (comprobar en la `FilterBar` o en la URL).
6. Repetir con un disco sin eventos asociados: la sección debe mostrarse igualmente, con su mensaje
   de vacío, nunca desaparecer ni mostrar un hueco en blanco.
7. Repetir en tema claro y oscuro, y en la ventana mínima (1024 × 560): la sección no debe recortar
   contenido en silencio.

## Qué NO debe ocurrir

- La sección no debe llevar sus propios controles de filtro, paginación ni un panel de XML inline
  — eso indicaría que se implementó la opción de "lista completa embebida", descartada.
- Un fallo de esta sección no debe impedir ver la cabecera, las métricas, las gráficas de
  temperatura/actividad ni los contadores SMART.
