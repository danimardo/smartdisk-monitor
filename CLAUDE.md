@AGENTS.md

<!--
  MANTENIMIENTO: aquí NO se duplica nada de AGENTS.md. Solo lo que es específico de Claude Code.
  Si una regla vale para cualquier agente, va en AGENTS.md.
-->

## Notas específicas para Claude Code

- **Responde siempre en español**, incluidos los planes y los resúmenes de progreso.
- Usa **modo plan** antes de tocar `src-tauri/`, `.specify/memory/`, `third-party/` o
  `src-tauri/tauri.conf.json`: son las rutas donde un cambio tiene efecto global o es difícil de revertir.
- Las **skills** del proyecto están en `.claude/skills/`. Actívalas en vez de reconstruir el
  procedimiento de memoria. Las diez `speckit-*` cubren el flujo de especificación completo.
- Las **reglas por ámbito** están en `.claude/rules/` y se cargan solas al leer o editar los
  ficheros que declaran en `paths:`. Si dudas de que una regla esté activa, ejecuta `/context` y
  míralo bajo **Memory files**.
- Este proyecto es de **Windows**: la shell por defecto de las herramientas es PowerShell, y el
  Bash disponible es Git Bash. Las rutas con espacios necesitan comillas.
