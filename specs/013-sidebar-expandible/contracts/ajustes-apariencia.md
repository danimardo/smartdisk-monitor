# Contrato — `AppearanceSettings` gana `sidebarExpanded`

Delta sobre `docs/ui-contract.md` §3.1 (Apariencia y ajustes). No es un comando nuevo: es un campo
más en la respuesta de un comando ya existente.

## Antes

```ts
invoke<AppearanceSettings>("get_appearance_settings")
interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "es" | "en" | null;
  systemLocale: string;
  useSystemAccent: boolean;
}
```

## Después

```ts
interface AppearanceSettings {
  theme: "light" | "dark" | "system";
  language: "es" | "en" | null;
  systemLocale: string;
  useSystemAccent: boolean;
  sidebarExpanded: boolean;   // NUEVO. Fábrica: false
}
```

- Se lee con `get_appearance_settings`, igual que el resto de campos.
- Se escribe con el `set_setting(key, value)` genérico ya existente:
  `setSetting("settings.appearance.sidebar_expanded", boolean)`.
- Clave en `settings`: `settings.appearance.sidebar_expanded` — mismo patrón que
  `settings.appearance.use_system_accent`.
- Sin migración de esquema (tabla `settings` genérica clave/valor).
- Sin permiso de Tauri nuevo: ni el comando ni su mecanismo de persistencia cambian de forma.

## Validación

`src/lib/api/schemas.ts`, esquema `appearanceSettings`: se añade `sidebarExpanded: z.boolean()`.
Prueba de rechazo nueva en `schemas.test.ts` (no había ninguna sobre el tipo de `useSystemAccent`,
así que esta es la primera del grupo que cubre un campo booleano): un valor no booleano para
`sidebarExpanded` hace que `safeParse` falle (`success: false`).
