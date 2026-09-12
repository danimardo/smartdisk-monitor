# Fase 1 — Modelo de datos: Eventos de Windows en el detalle de disco

## Entidades

Ninguna entidad nueva. Esta funcionalidad consume `SystemEvent` y `SystemEventPage`, ya definidas y
validadas (`src/lib/api/types.ts`, `docs/ui-contract.md` §3.5) y sin cambios de forma:

```ts
interface SystemEvent {
  id: string;
  occurredAt: string;
  provider: string;
  eventId: number;
  level: "error" | "warning" | "info";
  message: string;                       // idioma de Windows; se renderiza como texto, nunca HTML
  deviceId: string | null;
  volumeId: string | null;
  mappingConfidence: "exact" | "inferred" | "unknown";
  hasRawXml: boolean;
}

interface SystemEventPage { events: SystemEvent[]; nextCursor: string | null; total: number | null }
```

## Estado de pantalla (no persistido)

Un único bloque de estado local en `src/routes/disks/[id]/+page.svelte`, con la misma forma que ya
usan `serieTemp`/`serieActividad` en la misma pantalla (`EstadoSerie`), adaptado a una lista:

| Campo | Tipo | Qué representa |
|---|---|---|
| `eventos` | `SystemEvent[]` | Los hasta 5 eventos más recientes del disco actual. |
| `error` | `AppError \| null` | Fallo de la última petición, si lo hubo. |
| `loading` | `boolean` | Petición en curso. |

Se recalcula en un `$effect` que depende de `disk.id` (cambia de disco → nueva petición), con su
función de limpieza para descartar una respuesta tardía de un disco que ya no es el actual — mismo
patrón que el `$effect` de las dos series históricas de esta pantalla.

No hay escritura: esta funcionalidad es de solo lectura, no añade ninguna mutación ni comando nuevo.

## Reglas de validación

Ninguna nueva. La validación en tiempo de ejecución de `SystemEventPage` ya la hace
`S.systemEventPage` (Zod) dentro de `getSystemEvents`, sin cambios.
