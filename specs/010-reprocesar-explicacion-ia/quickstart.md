# Quickstart — validar «reprocesar la explicación con IA con otro modelo»

## Prerrequisitos

- Repositorio con dependencias instaladas (`pnpm install`) y compilando (`pnpm check`, `cargo
  build` desde `src-tauri/`).
- Ayuda con IA activada: o bien la clave de demostración compartida (botón "Usar la clave de
  demostración" en Ajustes, si el binario la trae compilada), o bien una clave propia de
  OpenRouter. Sin ninguna de las dos, esta función no aparece (principio XVI: apagada de fábrica).
- Al menos una alerta activa o un disco con detalle SMART disponible, para tener algo sobre lo que
  pedir "Explícamelo en lenguaje claro".

## Pruebas automáticas (la validación de fondo)

| Capa | Comando | Qué cubre |
|---|---|---|
| Rust | `cargo test` (desde `src-tauri/`) | `modelo_solicitado` sustituye a `settings.ai.model` solo en esa llamada; sin campo, comportamiento sin cambios |
| Store (`*.svelte.ts`, jsdom) | `pnpm test` | `historial`, `reprocesar()`, `fijarPorDefecto()`, reinicio en `lanzar()` |
| Componentes (Chromium) | `pnpm test:component` | `ExplicacionModal` en fases `resultado`/`error` con selector + reproceso + historial plegable; `AiModelSelect` con modelos de pago deshabilitados bajo clave compartida y con `incluirAutomatico=false` |
| Tipos y accesibilidad | `pnpm check` | Cero avisos (principio VII) |
| Formato y estático | `pnpm lint` | Rust: `cargo clippy --all-targets -- -D warnings` |
| Verificadores propios | `pnpm verify` | Tokens visuales, i18n de las claves nuevas |

`cargo test`, `pnpm test` y `pnpm test:component` en verde, con `pnpm verify` sin fallos, son la
condición de cierre técnico (skill `cierre-tarea`) antes de la validación manual.

## Validación manual — un recorrido por las 4 historias

Con `pnpm app:dev` (la app arranca elevada, hay que aceptar el UAC):

1. **US1 — probar otro modelo**: pide "Explícamelo en lenguaje claro" sobre una alerta o un detalle
   SMART. En la respuesta debe aparecer un mensaje invitando a decidir, un selector de modelo (sin
   la opción "automático") y un botón de reprocesar. Elige un modelo gratuito distinto del que
   respondió y pulsa reprocesar: debe verse el indicador de progreso y luego una respuesta nueva,
   con su modelo indicado. Repite provocando un error (por ejemplo, desconectando la red antes de
   pedir la explicación) y comprueba que el mismo selector + botón aparecen también ahí. Reprocesa
   de nuevo y, mientras se ve el indicador de progreso, cierra el modal (FR-013): confirma que no
   reaparece solo ni con contenido inesperado cuando la respuesta llegue de fondo.

2. **US2 — fijar por defecto**: tras reprocesar y obtener una respuesta nueva, debe verse una acción
   para fijarla como predeterminada. Al usarla, abre Ajustes → asistencia con IA y confirma que el
   selector de esa pantalla ya muestra el modelo elegido. Genera después un informe con resumen de
   IA (pantalla Informes) y confirma en el fichero exportado (o en el registro, si lo expone) que
   usó ese mismo modelo. Comprueba también que esa acción **no** aparece sobre la primerísima
   respuesta (la del modo automático, antes de reprocesar nada).

3. **US3 — historial plegable**: reprocesa dos o tres veces seguidas con modelos distintos. Cada
   respuesta que deja de estar en primer plano debe quedar como una fila plegada con el nombre de
   su modelo; despliega una y confirma que se lee entera y se puede volver a plegar sin que
   desaparezcan las demás. Cierra el modal y pide otra explicación distinta: no debe quedar rastro
   del historial anterior.

4. **US4 — modelos de pago según la clave**: con la clave de demostración activa, abre el selector
   de este modal y el de Ajustes; los modelos de pago deben verse listados pero no seleccionables.
   Configura una clave propia de OpenRouter y repite: los mismos modelos deben poder elegirse con
   normalidad en los dos sitios.

## Qué NO debe verse (regresiones a vigilar)

- El texto que se reenvía al reprocesar no debe volver a pasar por una pantalla de vista previa o
  revisión si la sesión ya la superó una vez (mismo comportamiento que "Reintentar" hoy).
- Ningún modelo de pago debe desaparecer de la lista (deshabilitado, nunca oculto).
- El informe HTML (spec 009) no debe cambiar de aspecto ni de flujo salvo por qué modelo usa de
  fábrica.
