# Instrucciones para Codex

**Las instrucciones de este repositorio están en [`AGENTS.md`](AGENTS.md)**, que Codex ya lee de
forma nativa. Este fichero existe para que no quede duda, no para añadir una segunda versión: una
copia paralela se desincroniza, que es exactamente el fallo que documenta ADR-029.

Tres cosas que conviene tener presentes antes de empezar:

- **Interfaz**: `docs/ui-design.md` es vinculante y se lee entero antes de escribir una pantalla.
  Su §0 es el mapa de dónde vive cada pieza —tokens, catálogo, diccionarios, boceto aprobado—.
- **Límites duros**: constitución, contratos, permisos de Tauri, dependencias, `third-party/` y
  cualquier commit necesitan autorización explícita. Están enumerados en `AGENTS.md`.
- **Idioma**: toda comunicación con el usuario en español, incluidos planes y resúmenes.
