---
paths:
  - "src/lib/components/**/*.svelte"
  - "src/routes/**/*.svelte"
  - "src/design-system/**"
---

# Interfaz

Las reglas completas están en `Design-system/AGENTS.md` y son **vinculantes**. Léelo antes de
escribir una pantalla. Aquí van solo las que más se incumplen.

## Nunca

- **Un color, radio, sombra o tamaño de fuente literal.** Todo sale de `tokens.css` o de su mapeo
  Tailwind. `pnpm verify:tokens` lo comprueba y falla la integración.
- **`backdrop-filter` escrito a mano.** Usa `.sdm-material`, `.sdm-material-chrome` o
  `.sdm-material-overlay`.
- **Un literal de interfaz**, ni siquiera en `aria-label`, `title` o `alt`. Todo pasa por `t()` o
  `tp()`, y las claves deben existir en los dos diccionarios.
- **Renderizar como HTML** contenido que venga de un dispositivo o del registro de eventos.
- **Navegar con `goto()` dentro de un `onclick`.** Rompe ctrl+clic, menú contextual, foco y el
  anuncio como enlace. Se navega con `<a href>`; `goto()` queda para redirecciones programáticas.
- **Anular `:focus-visible`.**

## Siempre

- **Un dato ausente es «No disponible»**, nunca cero, guion ni cadena vacía.
- **No compatible ≠ averiado**: gris, jamás rojo, y sin generar alerta.
- **El color nunca es el único portador de significado**: va acompañado de texto o icono.
- **Estados diseñados**: cargando, vacío, no compatible, error de fuente y dato obsoleto. Una
  pantalla sin ellos no está terminada.
- **`$derived` antes que `$effect`.** Un efecto que asigna estado derivado crea ciclos y
  ejecuciones de más.
- **Callbacks opcionales** (`onalgo = undefined`): un componente debe poder usarse sin ellos.

## Al terminar

`pnpm check` con **cero errores y cero avisos**. Un aviso de accesibilidad es un incumplimiento del
principio VII de la constitución, no una molestia. Si uno no se puede resolver, se registra en
`docs/known-issues.md` y el silencio del código enlaza a su entrada por número.
