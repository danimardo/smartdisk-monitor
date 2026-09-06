# 06 · Ajustes (`/settings`)

**Cambio de tokens, más una preferencia nueva.** Sin mockup propio: son controles del catálogo ya
existentes, sin composición nueva.

## 1. Diagnóstico

Referencia: `capturas/ajustes__claro__1280x800__completa.png`.

- Es la pantalla más densa en controles y las siete secciones pesan lo mismo. «Borrado de datos»
  (destructivo, irreversible) tiene la misma presencia que «Idioma».
- Las secciones se separan solo por espacio, así que en scroll largo cuesta situarse.

## 2. Cambios de distribución

Ninguno estructural: siguen siendo secciones apiladas en una columna con ancho máximo.

Dos ajustes de composición:

- Cada sección se envuelve en su propia `Card` (hoy varias comparten superficie). Recordatorio: **no se
  apilan materiales**, así que los controles dentro van sobre `bg-glass-3`, no sobre otra `Card`.
- La sección «Borrado de datos» se separa al final con un espacio de `space-8` (32 px) y su
  `Card` lleva `border-crit` en lugar de `border-hairline`. El fondo **no** cambia: solo el borde,
  para no teñir de rojo una zona que el usuario visita sin querer borrar nada.

## 3. Cambios por componente

- **`Switch`, `Select`, `RadioGroup`, `TextField`** — solo tokens. Ojo: el `Switch` activo usa el
  degradado del acento, así que en oscuro su punto blanco sigue siendo blanco (el punto no es texto;
  `--sdm-on-accent` no aplica ahí).
- **Preferencia nueva** en Apariencia: `Switch` «Usar el color de acento de Windows», **apagada de
  fábrica**, con `hint` «Sustituye el morado de la aplicación por el color que tengas configurado en
  Windows». Al encender llama a `applySystemAccent()`; al apagar, a `clearSystemAccent()`.
  Clave de persistencia: `settings.appearance.useSystemAccent` (booleano).
- **Botones destructivos** — siguen siendo `Button variant="danger"` y siguen abriendo `ConfirmDialog`.

## 4. Estados

- **Cargando**: los controles aparecen deshabilitados hasta que llegan los valores; no se muestran con
  valores por defecto que luego salten.
- **Vacío**: no aplica.
- **No compatible**: los ajustes que dependen de una capacidad ausente (p. ej. autotest SMART) se
  deshabilitan con `disabledReason`, no se ocultan: ocultarlos hace pensar que la aplicación los perdió.
- **Error de fuente**: si falla guardar un ajuste, el control vuelve a su valor anterior y aparece el
  error **junto al control**, con `TextField error` o texto `text-crit` bajo el `Switch`. Nunca un toast solo.
- **Dato obsoleto**: no aplica.

## 5. Claro y oscuro

Solo tokens. Esta pantalla es la que más superficies de control tiene: conviene revisarla primero al
implantar `--sdm-on-accent`.

## 6. Ventana mínima

Sin cambios: columna única con ancho máximo, scroll vertical.
