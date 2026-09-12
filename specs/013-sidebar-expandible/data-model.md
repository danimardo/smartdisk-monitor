# Fase 1 — Modelo de datos: Riel de navegación expandible

## Entidades

### `AppearanceSettings` (existente, gana un campo)

```ts
interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "es" | "en" | null;
  systemLocale: string;
  useSystemAccent: boolean;
  sidebarExpanded: boolean;   // NUEVO. Fábrica: false (plegado, igual que el riel de hoy)
}
```

Persistida en la tabla `settings` genérica bajo la clave `settings.appearance.sidebar_expanded`,
mismo mecanismo que `settings.appearance.use_system_accent`. Sin migración de esquema: una clave
más en una tabla clave/valor ya existente.

## Estado de pantalla (no persistido)

En `+layout.svelte`, junto al resto del estado de apariencia:

| Campo | Tipo | Qué representa |
|---|---|---|
| `expanded` | `boolean` | Si el panel del riel está abierto ahora mismo. Se inicializa desde `appearance.sidebarExpanded` al arrancar. |

En `Sidebar.svelte` (presentacional, recibe `expanded` como prop, no lo posee):

| Campo | Tipo | Qué representa |
|---|---|---|
| `botonRef` | referencia a elemento | El botón que abrió el panel, para devolverle el foco al cerrar. |
| `panelRef` | referencia a elemento | El panel expandido, para moverle el foco al abrir. |

Dos callbacks distintos, con distinto efecto sobre la persistencia (revisado tras la validación
manual, D6 de `research.md`):

| Prop | Quién lo llama | Persiste con `setSetting` |
|---|---|---|
| `onToggleExpand` | El propio botón, `Escape`, clic fuera | Sí — gesto explícito de la persona |
| `onSelect` | Elegir una sección o «Acerca de» | No — consecuencia de navegar, no una preferencia nueva |

## Reglas de validación

El campo nuevo se valida como cualquier otro booleano ya presente en `appearanceSettings`
(`z.boolean()` en el esquema Zod de `src/lib/api/schemas.ts`). Un valor que no sea booleano
produce el mismo `AppError` de `ipc.schema_mismatch` que cualquier otro campo mal formado
(principio XI) — se añade su prueba de rechazo, no un mecanismo nuevo.

## Transiciones de estado

```
plegado --(pulsar el botón)--------------------------------------> expandido   [persiste]
expandido --(pulsar el botón | Escape | clic fuera)---------------> plegado    [persiste]
expandido --(elegir una sección o «Acerca de», navegando)---------> plegado    [no persiste]
```

La tercera transición (revisada tras la validación manual, D6 de `research.md`) es una
consecuencia de elegir a dónde ir, no un "ya no lo quiero expandido" explícito: si también
persistiera, la preferencia "empezar expandido" se perdería en cuanto la persona pulsara el
primer enlace de la sesión.
