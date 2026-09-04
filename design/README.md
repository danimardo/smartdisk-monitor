# Bocetos del sistema de diseño

Referencia **visual**, no código. Aquí solo hay lienzos navegables que enseñan cómo debe verse la
aplicación; lo que se implementa vive en `src/`.

| Fichero | Qué es |
|---|---|
| `SmartDisk Monitor v2.dc.html` | **APROBADO.** Cuatro pantallas (panel general, detalle de disco, alertas, pruebas), en tema claro y oscuro, con el diálogo de confirmación incluido. Es la referencia contra la que se revisa una pantalla |
| `Sistema de diseno SmartDisk.dc.html` | Guía visual de tokens y componentes. Sigue en estilo v1: **pendiente de refresco a v2**. Ante una diferencia con el boceto aprobado, manda el aprobado |
| `Bocetos SmartDisk Monitor.dc.html` | Exploración inicial de las direcciones 1a y 1b. Referencia histórica: no se implementa nada de aquí |

`support.js` es el runtime que los tres necesitan para funcionar, y `.thumbnail` la miniatura de
previsualización. Ninguno de los dos se edita a mano.

## Cómo se abren

Doble clic en el `.html`, o desde la terminal:

```powershell
Invoke-Item ".\design\SmartDisk Monitor v2.dc.html"
```

Los tres cargan `support.js` desde la misma carpeta, así que **no funcionan si se copian sueltos** a
otro sitio.

## Lo que estos ficheros NO son

- **No son la fuente de verdad visual.** Esa es `src/design-system/tokens.css`. Si un boceto y un
  token discrepan, gana el token, y la discrepancia se anota en `docs/open-questions.md`.
- **No contienen código reutilizable.** Su marcado es de la herramienta de diseño; el catálogo real
  es `src/lib/components/`.
- **No son normativos por sí solos.** Las reglas están escritas en `docs/ui-design.md`, que es lo
  vinculante. El boceto muestra el resultado; el documento dice por qué y con qué límites.

## Procedencia

El diseñador entregó estos bocetos dentro del paquete `Design-system/`, que además traía una copia
del sistema de diseño (tokens, componentes, i18n). Esa copia se integró en `src/` y **se eliminó de
aquí** porque las dos versiones habían empezado a divergir en silencio: el motivo completo y la
divergencia medida están en ADR-029 (`docs/decisions.md`). El paquete original íntegro sigue
disponible en el historial de git, anterior a ese cambio.
