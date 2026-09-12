# Quickstart — Validación: Riel de navegación expandible

## Prerrequisitos

- `pnpm install` ya ejecutado.

## Validación automática (referencia)

```powershell
pnpm check
cargo test --manifest-path src-tauri/Cargo.toml sidebar_expanded
pnpm test:component -- Sidebar
pnpm test:e2e -- sidebar
```

Casos que debe cubrir `Sidebar.browser.test.ts` (ver `spec.md`, User Stories 1-4):

1. Plegado (por defecto): solo iconos, cada uno con `title`/`aria-label`, como hoy.
2. Pulsar el botón de expandir: aparece un panel con icono + nombre de cada sección; el foco entra
   en el panel.
3. Pulsar una sección dentro del panel: navega (es un `<a href>` real) y el panel se pliega a la
   vez, sin persistir el plegado (solo el botón/`Escape`/clic fuera cambian la preferencia
   guardada).
4. `Escape` con el panel abierto: se cierra y el foco vuelve al botón que lo abrió.
5. Un clic fuera del panel (en el contenido) con el panel abierto: se cierra.
6. El botón lleva `aria-expanded` reflejando el estado real.

Casos que debe cubrir el e2e (`e2e/ui/sidebar.spec.ts` o ampliación de `smoke.spec.ts`):

1. Expandir el riel, comprobar que `set_setting` se llamó con
   `settings.appearance.sidebar_expanded` y `true`.
2. Recargar con `get_appearance_settings` devolviendo `sidebarExpanded: true` en el fixture:
   el riel arranca ya expandido, sin pulsar nada.
3. Con la ventana en 1024×560 (`escalado.spec.ts` reutiliza este patrón): expandir el riel y
   comprobar que ninguna región de contenido cambia de ancho ni aparece un recorte nuevo.

## Validación manual (con la aplicación real)

1. `pnpm app:dev` (pide UAC: la app va elevada).
2. Pulsar el botón de expandir en el riel: deben verse el icono y el nombre de las seis secciones
   más «Acerca de», sin tener que pasar el ratón por ninguno.
3. Pulsar una sección con el panel abierto: debe navegar y plegarse a la vez, sin tapar la
   pantalla de destino.
4. Pulsar `Escape`: el panel se cierra; comprobar que el foco vuelve visualmente al botón.
5. Cerrar la aplicación con el panel expandido y volver a abrirla: debe arrancar ya expandido.
6. Repetir plegándolo antes de cerrar: debe arrancar plegado.
7. Redimensionar la ventana hasta el mínimo (1024 × 560) con el panel abierto: ninguna pantalla
   debe recortarse ni mostrar scroll horizontal nuevo — el panel se superpone, no empuja.
8. Repetir en tema claro y oscuro.
9. Navegar solo con teclado: `Tab` hasta el botón de expandir, `Enter` para abrirlo, `Tab` para
   recorrer las secciones del panel, `Escape` para cerrarlo.

## Qué NO debe ocurrir

- El ancho de ninguna región de contenido debe cambiar al expandir o plegar el riel.
- El panel expandido no debe tener sus propias etiquetas de texto nuevas: debe reutilizar
  exactamente los mismos textos que ya usan `title`/`aria-label` hoy.
- Ninguna sección debe dejar de ser un enlace real (`<a href>`) dentro del panel expandido.
