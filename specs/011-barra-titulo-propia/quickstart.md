# Quickstart — validar «barra de título propia»

## Prerrequisitos

- Repositorio compilando (`pnpm check`, `cargo build` desde `src-tauri/`).
- `pnpm app:dev` requiere UAC (la app va elevada) — solo lo puede ejecutar la persona, no este
  agente.

## Pruebas automáticas

| Capa | Comando | Qué cubre |
|---|---|---|
| Rust | `cargo test` (desde `src-tauri/`) | `geometria_visible` sigue en verde sin cambios (D1); las constantes `MIN_H` compiladas con el nuevo valor |
| Tipos y accesibilidad | `pnpm check` | Cero avisos |
| Componentes (Chromium) | `pnpm test:component` | `TitleBar.svelte` en sus dos estados (restaurada/maximizada), callbacks simulados, nombres accesibles |
| Formato y estático | `pnpm lint` | Rust: `cargo clippy --all-targets -- -D warnings` |
| Verificadores propios | `pnpm verify` | Tokens visuales, i18n de las claves nuevas |

## Validación manual — recorrido por las 3 historias

Con `pnpm app:dev`:

1. **US1 — paridad funcional**: arrastra la ventana por la barra propia (se mueve); pulsa
   minimizar (se oculta a la barra de tareas); pulsa maximizar (ocupa la pantalla, el control pasa
   a "restaurar"); pulsa restaurar (vuelve al tamaño anterior); arrastra un borde de la ventana (se
   redimensiona); pulsa cerrar (dispara el mismo comportamiento de siempre — minimizar a la bandeja
   si está configurado así, o cerrar de verdad).
2. **US2 — integración visual**: compara la barra superior con el resto de la ventana en tema
   claro y en tema oscuro (mismo fondo, sin franja); compara el color de los tres controles con un
   botón primario de un diálogo (mismo acento); si tienes activado "usar el acento de Windows",
   cambia el acento del sistema y confirma que los controles lo siguen.
3. **US3 — doble clic**: doble clic sobre una zona vacía de la barra maximiza; doble clic de nuevo
   restaura.

## Qué comprobar de paso (no historias propias, pero riesgos identificados en `research.md`)

- **D7 — esquinas de la ventana**: sin decoración nativa, comprobar si las esquinas *superiores* de
  la ventana (ahora en `TitleBar`, que no lleva redondeo propio) se ven redondeadas por el
  compositor de Windows 11 o cuadradas. Las esquinas *inferiores* sí llevan redondeo propio
  (`AppShell` usa `rounded-b-window`, corrección post-validación: arriba se quitó porque ya no
  lindaba con el borde real de la ventana) — comprobar que esas dos coinciden con el redondeo real
  del compositor y no se ve un doble borde. Si algo se ve mal, es una tarea de seguimiento (añadir
  `"transparent": true`), no un bloqueante de esta feature.
- **Ventana mínima**: redimensionar a 1024×(560 + altura de la barra) y confirmar que no hay ningún
  elemento recortado, igual que antes del cambio.
- **Multi-monitor**: con la ventana en un segundo monitor, cerrar y volver a abrir la app; confirmar
  que la posición se restaura igual que antes (D1: `geometria_visible` no cambia).
