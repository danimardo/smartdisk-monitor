# Paquete de entrega — UI de SmartDisk Monitor

Contiene el sistema de diseño aprobado (**v2, material translúcido**) y el catálogo de componentes
Svelte listos para montar la aplicación Tauri. No es un proyecto ejecutable: son los archivos de UI
para integrar en el esqueleto Tauri 2 + Svelte + TypeScript + Tailwind.

## Qué hay dentro

```
AGENTS.md                        Reglas VINCULANTES de UI. Leer antes de escribir una sola pantalla.
HANDOFF.md                       Este archivo.
tailwind.config.cjs              Tokens → utilidades Tailwind.

design-system/
  tokens.css                     Fuente única de verdad: variables --sdm-*, temas claro/oscuro,
                                 utilidades de material y base. Importar UNA vez al arrancar.
  fonts/                         Instrument Sans variable, empotrada. La app NO descarga tipografías
                                 (spec §11). Falta el .woff2: véase fonts/README.md.
  tokens.json                    Los mismos valores, legibles por herramientas.
  README.md                      Principios, anatomía y arranque.

src/lib/design/
  types.ts                       Vocabulario de UI alineado con docs/data-model.md.
  format.ts                      Formateo de presentación. Todo dato ausente → "No disponible".
  health.ts                      Único mapa estado→color + umbrales de capacidad de la spec.
  theme.svelte.ts                Preferencia claro/oscuro/sistema → data-theme en <html>.
  accent.ts                      Hereda el color de acento de Windows sobre los tokens de acento.

src/lib/i18n/
  index.ts, es.json, en.json     i18n mínimo. Idioma inicial del sistema (es-* → es, resto → en).

src/lib/components/              Catálogo cerrado (25 componentes). Importar del barrel index.ts.
                                 Otros 4 quedan autorizados y por construir: véase AGENTS.md §3.

*.dc.html                        Bocetos navegables (abrir en el navegador):
  SmartDisk Monitor v2.dc.html      ← APROBADO: 4 pantallas, claro y oscuro, diálogo incluido.
  Sistema de diseno SmartDisk.dc.html  Guía visual de tokens (estilo v1, pendiente de refresco).
  Bocetos SmartDisk Monitor.dc.html    Exploración inicial 1a/1b, referencia histórica.
```

## Integración en 5 pasos

La base es **SvelteKit con `adapter-static` y SSR desactivado** (ADR-014). Rutas de destino exactas
en `docs/engineering-conventions.md`; resumen:

| Origen | Destino en el proyecto |
|---|---|
| `design-system/` (con `fonts/`) | `src/design-system/` |
| `src/lib/` | `src/lib/` |
| `tailwind.config.cjs` | raíz del proyecto |

1. Copia los tres bloques de la tabla. `$lib` ya apunta a `src/lib` en SvelteKit: no toques el alias.
2. Deja `tokens.css` como **única** importación de CSS global, en `src/routes/+layout.svelte`.
3. En el arranque (`+layout.svelte`, antes del primer render):

   ```ts
   import "../design-system/tokens.css";
   import { invoke } from "@tauri-apps/api/core";
   import { theme } from "$lib/design/theme.svelte";
   import { applySystemAccent } from "$lib/design/accent";
   import { i18n } from "$lib/i18n";

   const s = await invoke<AppearanceSettings>("get_appearance_settings");
   theme.init(s.theme);
   i18n.init(s.language, s.systemLocale); // el locale viene del backend, no de navigator
   await applySystemAccent();
   ```

   `i18n.init()` fija además `<html lang>`, e `i18n.formatLocale` es el locale que usan **todas** las
   funciones de `format.ts`: los números siguen al idioma de la aplicación, no al de Windows.

4. Monta la app con `AppShell` + `Sidebar` + `Toolbar`; ninguna pantalla monta su propio chrome.
5. Suscríbete a los eventos de la tabla de abajo. **No hagas sondeo con `setInterval`**: el backend
   empuja (ADR-015).
6. Lee `AGENTS.md` y su checklist de "terminado" antes de cerrar cada pantalla.

## Comandos Tauri que la UI espera

**El contrato exacto y normativo vive en `docs/ui-contract.md`**: firmas, DTO, eventos de
actualización en vivo y forma de los errores. Lo de abajo es solo el índice. La UI **solo** llama
comandos enumerados.

| Comando | Devuelve |
|---|---|
| `get_appearance_settings` | tema e idioma persistidos en `settings` |
| `set_setting` | guarda una clave tipada de `settings` |
| `get_system_accent_color` | `{ hex }` del acento de Windows (o error si está desactivado) |
| `get_devices` / `get_device_detail` | inventario y detalle con procedencia por métrica |
| `get_metric_series` | serie temporal con huecos explícitos (`v: null`), no interpolados |
| `get_alert_groups` / `acknowledge_alert` / `mute_alert` / `archive_alert` | ciclo de vida de alertas |
| `get_system_events` | eventos con `mappingConfidence` |
| `start_benchmark` / `cancel_test` / `run_chkdsk_scan` / `run_smart_short_test` | pruebas manuales |
| `get_test_runs` | historial de `test_runs` |
| `export_report` / `create_diagnostic_zip` | informes y ZIP anonimizado |
| `get_settings` / `set_setting` / `reset_settings` | pantalla de Ajustes (US-070) |
| `pause_monitoring` / `resume_monitoring` | pausa desde la bandeja (US-032) |
| `delete_all_data` | borrado explícito y confirmado del historial (US-073) |

Además el backend **emite eventos**; la UI no hace sondeo (ADR-015):

| Evento | Cuándo |
|---|---|
| `metrics:updated` | cada ciclo de recopilación, con el lote de métricas frescas |
| `alerts:changed` | alta, cambio de severidad, resolución o cambio de estado de un grupo |
| `inventory:changed` | alta o retirada de un disco o volumen |
| `test:progress` | progreso de una prueba en curso |
| `system:accent-changed` / `system:theme-changed` | el usuario cambia la apariencia de Windows |

## Pendiente de diseño (no incluido)

Todas estas pantallas tienen ya criterios de aceptación en `docs/user-stories.md` (épica H y US-070
a US-074); lo que falta es la composición visual, no la definición funcional.

- **Informes** (US-050): selector de intervalo, resumen de contenido y destino de exportación.
- **Ajustes**: apariencia, frecuencias, umbrales, retención, comportamiento al cerrar, borrado de datos.
- **Asistente inicial** (US-002): detección, exclusión de discos y alias.
- **Acerca de** (US-061) y estados de systray.

Casi todo se compone con el catálogo actual (`Switch`, `Select`, `TextField`, `RadioGroup`,
`SegmentedControl`, `ConfirmDialog`, `EmptyState`, `CodeOutput`). Las excepciones ya están
autorizadas y no requieren decisión nueva: `DateRangePicker`, `FilterBar`, `VirtualList` y `Tooltip`
(AGENTS.md §3, "Autorizados y pendientes de construir").

## Notas para quien programe

- Ningún literal de color, radio, sombra o tamaño en un componente: utilidad Tailwind o `var(--sdm-*)`.
- Nunca escribas `backdrop-filter` a mano: usa `.sdm-material`, `.sdm-material-chrome`, `.sdm-material-overlay`.
- Los tokens de texto y de salud están verificados a 4.5:1 sobre el material de su tema. No los aclares.
- Un dato ausente es "No disponible"; un dispositivo sin SMART es gris, nunca rojo ni alerta activa.
- Toda acción que escriba datos o genere carga pasa por `ConfirmDialog` con impacto y comando literal.
- Contenido procedente de eventos o dispositivos se renderiza como texto, jamás como HTML.
- Reconocer una alerta **no** devuelve el disco a verde: el color lo decide `deviceState()`, que
  cuenta las alertas `active` y `acknowledged`. El silencio nunca toca el color.
- El acento del sistema pasa por `accessibleAccent()` antes de aplicarse; no supongas texto blanco
  sobre el acento, usa `--sdm-on-accent`.
- `TimeSeriesChart` necesita `from`/`to` además de los puntos: el eje es tiempo real, y el intervalo
  pedido debe verse entero aunque falten datos.
- **La navegación se hace con enlaces, no con callbacks.** `DiskCard` recibe `href` y `Sidebar`
  recibe secciones con su `href`: un `onclick` con `goto()` rompe el ctrl+clic, el menú contextual y
  el anuncio como enlace de un lector de pantalla.
- El estado inicial de cada pantalla llega por `load` en `+page.ts`, no por `onMount`. Las
  actualizaciones vienen después por eventos.
