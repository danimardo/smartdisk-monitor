# 08 · Configuración inicial (`/onboarding`)

**Pantalla nueva.** Hoy es un `EmptyState` «no implementado» (`capturas/configuracion-inicial__*`).
Mockup: `mockups/smartdisk-v3.html` → pestaña **Asistente inicial**, claro y oscuro. El mockup muestra
el **paso 2**, que es el único con composición propia; los otros tres se describen aquí.

Cubre **US-002**. Objetivo: que el primer arranque termine con el usuario sabiendo qué se vigila, qué se
le va a avisar y que la aplicación no escribe en sus discos.

## 1. Principios

1. **Un paso por pantalla.** Cuatro pasos, sin acordeones ni scroll de formulario largo.
2. **Salida siempre visible.** «Omitir y usar los valores de fábrica» en la cabecera, en todos los pasos.
   Omitir es una opción legítima, no un castigo: los valores de fábrica son buenos.
3. **Sin chrome de aplicación.** No hay riel ni `Toolbar`: el asistente ocupa la ventana. Solo cabecera
   propia de 56 px con logotipo, título, indicador de paso y la salida.
4. **Nada irreversible.** Todo lo que se elige aquí se cambia después en Ajustes, y se dice.
5. **Ancho máximo 1000 px**, centrado. A 1024 px de ventana sigue respirando.

## 2. Los cuatro pasos

### Paso 1 · Bienvenida

Contenido: qué hace la aplicación en tres frases, y la garantía explícita —
**«SmartDisk solo lee. No modifica, no repara y no borra nada de tus discos.»** — porque es la duda
razonable de cualquiera que instala una utilidad de disco.

Composición: bloque de texto centrado, ancho máximo 620 px, titular a 34 px `.sdm-display`, tres
`Card` pequeñas con icono (`#i-shield` leer, `#i-alert` avisar, `#i-flask` probar solo si lo pides),
y botón primario «Buscar mis discos».

### Paso 2 · Discos *(mockup)*

Lista de tarjetas, una por disco detectado. Cada fila: casilla de 24 px, cuadrado de icono de 40 px
según bus (`#i-nvme`, `#i-hdd`, `#i-usb`), modelo y metadatos, **campo de alias**, ocupación del volumen
principal con barra, y píldora de compatibilidad SMART.

La tarjeta seleccionada lleva `border-accent` de 1,5 px; la excluida, `border-hairline`. El estado no
depende solo del borde: la casilla marcada es el portador principal.

Bajo la lista, el bloque explicativo del disco USB en `bg-glass-3`: **«no es una avería»** en negrita.
Si lo dejas marcado se vigila capacidad y eventos, pero no habrá temperatura ni desgaste.

Pie: «Atrás», contador «3 de 4 discos seleccionados» y primario «Continuar con las alertas».

### Paso 3 · Alertas

Tres `RadioGroup` de perfil, no una lista de umbrales numéricos: **Prudente** (avisa antes, más ruido),
**Equilibrado** (recomendado, preseleccionado) y **Solo lo grave** (solo crítico). Debajo, en un
`<details>` plegado, «Ver los umbrales exactos de este perfil» con la tabla real, para quien quiera.

Y dos `Switch`: notificación nativa de Windows cuando la ventana está minimizada (encendido) y arrancar
con el sistema (apagado, con `hint` de lo que implica).

### Paso 4 · Listo

Resumen de lo elegido en tres líneas con `#i-check`, la primera lectura ya en marcha con
`ProgressBar` indeterminada, y primario «Ir al panel». Nota al pie: «Todo esto se cambia en Ajustes».

## 3. Cuándo aparece

Al primer arranque, mientras `settings.onboarding.completedAt` sea nulo. Al terminar (o al omitir) se
graba la marca de tiempo y no vuelve a salir. Se puede volver a lanzar desde Ajustes → «Repetir la
configuración inicial», que **no** borra datos: solo reabre el asistente con los valores actuales.

## 4. Estados

| Estado | Cómo queda |
|---|---|
| **Cargando** | Paso 2 mientras se detecta: tres tarjetas esqueleto y el primario deshabilitado con «Buscando discos…». |
| **Vacío** | No se detecta ningún disco (raro, pero posible en una máquina virtual): `EmptyState kind="empty"` dentro del paso, con «Volver a buscar» y la opción de continuar igualmente. |
| **No compatible** | Es el caso del USB, y está en el cuerpo del paso: se muestra, se puede incluir, se explica y **no se pinta en rojo**. |
| **Error de fuente** | Falla la detección: `EmptyState kind="error"` con frase humana y `<details>` técnico, «Reintentar» y «Omitir y usar los valores de fábrica». El asistente nunca deja al usuario encallado. |
| **Dato obsoleto** | No aplica: la detección es del momento. |

## 5. Claro y oscuro

Mockup en los dos temas.

## 6. Ventana mínima (1024 × 560)

- El contenedor de 1000 px pasa a 100 % con `padding` 24 px.
- Fila de disco: por debajo de **900 px** el bloque de ocupación (190 px) baja a una segunda línea
  dentro de la misma tarjeta, y la fila crece de 70 a 104 px.
- El indicador de paso de la cabecera: por debajo de **860 px** se reduce a «Paso 2 de 4» en texto,
  sin los cuatro puntos.
- El pie de navegación queda **pegado abajo** (`position: sticky; bottom: 0`) con material de chrome, para
  que «Continuar» nunca quede fuera de la vista.
