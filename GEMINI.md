@AGENTS.md

<!--
  MANTENIMIENTO: aquí NO se duplica nada de AGENTS.md. Solo lo específico de Gemini CLI.
  Si una regla vale para cualquier agente, va en AGENTS.md. Una copia paralela se desincroniza:
  es exactamente el fallo que documenta ADR-029.
-->

## Notas específicas para Gemini

- **Responde siempre en español**, incluidos los planes y los resúmenes de progreso.
- `AGENTS.md`, importado arriba, es la **fuente canónica**. Todo lo que necesitas para trabajar en
  este repositorio sale de ahí y de los documentos que enumera en su orden de autoridad.
- **Antes de escribir una sola pantalla**, lee `docs/ui-design.md` entero. Es vinculante, y su §0
  dice dónde vive cada pieza: tokens, catálogo de componentes, diccionarios y boceto aprobado.
  `AGENTS.md` § «Sistema de diseño» trae el resumen, pero no sustituye a la norma.
- Los ficheros `.dc.html` de `design/` son **bocetos para abrir en el navegador**, no código a
  imitar. Su marcado lo genera la herramienta de diseño; el catálogo real está en
  `src/lib/components/`.
- Este proyecto es de **Windows**: PowerShell como shell, y las rutas con espacios necesitan
  comillas.
- Antes de tocar `src-tauri/`, `.specify/memory/` o `third-party/`, **presenta el plan y espera
  respuesta**: son rutas donde un cambio tiene efecto global o es difícil de revertir. Las tres
  primeras entradas de «Límites duros» en `AGENTS.md` están además bloqueadas por un hook que solo
  se ejecuta bajo Claude Code; aquí dependen de que las respetes.
