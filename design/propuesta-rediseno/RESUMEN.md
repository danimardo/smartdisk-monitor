# Propuesta de rediseño — SmartDisk Monitor v3

Sigue la estructura de `COMO-ENTREGAR-EL-REDISENO.md`. Todo cambio de color, tamaño, radio o sombra
está en `cambios/00-tokens.md`; los ficheros por pantalla solo describen distribución y composición.

**Dirección elegida:** «escena de datos» con paleta **Ciruela**. No es una reescritura del sistema:
el material de tres capas, los radios concéntricos, el catálogo y las reglas de producto siguen intactos.

---

## Los seis cambios de fondo

### 1. El panel deja de estar vacío: gana un héroe

**Problema observado** — `capturas/panel-general__claro__1280x800__completa.png`: con dos discos, la
rejilla ocupa 230 px de alto y deja ~570 px de lienzo vacío. La pantalla más importante de la aplicación
parece a medio cargar.

**Cambio** — Sobre la rejilla entra un bloque de 246 px de alto (`HeroPanel`, componente nuevo) dedicado
al disco que necesita atención: la curva de su temperatura de las últimas 24 h dibujada a sangre como
fondo, la cifra a 76 px, el umbral del fabricante como línea discontinua, el hueco de datos como banda
gris, cuatro hechos a la derecha y dos acciones. Si no hay ningún disco en advertencia o crítico, el
héroe muestra el disco de sistema en tono neutro con el texto «Todo en orden» (ver `cambios/01`).

**Por qué es mejor** — La pregunta que trae al usuario a la aplicación es «¿tengo un problema?». Hoy hay
que leer cuatro tarjetas y comparar números para responderla; con el héroe se responde sin leer.

### 2. Cada magnitud lleva icono y forma, no solo cifra

**Problema observado** — «41 °C», «3 %» y «12 %» aparecen como texto plano del mismo tamaño y color.
Nada indica si 41 °C está bien, ni cuánto margen queda. El disco USB muestra tres «No disponible»
de 20 px que pesan más que los datos reales del disco sano de al lado.

**Cambio** — Juego propio de **15 iconos de línea** de 24 px (`Icon`, componente nuevo) que heredan
`currentColor`. Toda magnitud lleva el suyo: termómetro, medidor de desgaste, pulso de actividad, reloj
de horas, escudo de salud. `MetricCard` gana ranura de icono y **sparkline** de 22 px con la evolución
del valor. `DiskCard` gana cabecera de 52 px con la sparkline de temperatura sobre el tono del estado.
Los «No disponible» bajan a `text-xs` en `text-fg-dim` (ver `cambios/componentes/MetricCard.md`).

**Por qué es mejor** — El icono da entrada rápida a la fila y la sparkline responde «¿esto es nuevo o
lleva así todo el día?» sin abrir el detalle. Y el dato ausente deja de gritar más que el dato presente.

### 3. Paleta Ciruela, e identidad propia en lugar del acento del sistema

**Problema observado** — El azul `#0067c0` heredado de Windows es correcto pero indistinguible de
cualquier utilidad del sistema; en las capturas oscuras el conjunto queda gris plano.

**Cambio** — Neutros malva y acento morado de tinta (`#7a3f9d` claro / `#c79aec` oscuro). El rojo
crítico se desplaza al bermellón `#b03434` para no confundirse con el acento. Heredar el acento de
Windows pasa a ser **un interruptor opcional** en Ajustes → Apariencia, apagado de fábrica: cuando se
activa sobrescribe exactamente tres tokens. Tabla completa y ratios medidos en `cambios/00-tokens.md`.

**Por qué es mejor** — La aplicación se reconoce de un vistazo, y el usuario que prefiera integrarse con
su sistema sigue pudiendo. Además el morado no compite con verde/ámbar/rojo, que aquí *significan* salud.

### 4. Segunda familia tipográfica, solo para cifras grandes

**Problema observado** — Instrument Sans a 20 px en `MetricCard` no tiene presencia suficiente para ser
el dato principal de una pantalla; a la vez, subir el peso está prohibido (máximo 600).

**Cambio** — `--sdm-font-display`: **Bricolage Grotesque** 600, con `letter-spacing: -0.04em` y
`tabular-nums`, mediante la clase `.sdm-display`. **Solo** para cifras y titulares de héroe **a partir de
22 px**; el texto corrido no la usa nunca. Dos escalones nuevos: 58 px (progreso de prueba) y 76 px
(temperatura del héroe).

**Por qué es mejor** — Da jerarquía sin tocar los pesos ni inventar un 700 que el sistema no tiene.

### 5. La barra lateral se convierte en riel de iconos

**Problema observado** — La `Sidebar` de 250 px con seis entradas y dos discos usa ~380 px de sus 800 px
de alto; el resto es vacío. A 1024 px de ancho se come el 24 % de la ventana.

**Cambio** — Riel de **74 px**: solo el logotipo, seis iconos de 44 px con `title`/`aria-label`, punto de
aviso sobre el icono de Alertas y el indicador de estado global al pie. La lista de discos se elimina de
la barra: ya está en la rejilla del panel, y con el héroe encima era una tercera repetición.

**Por qué es mejor** — Devuelve 176 px de ancho al contenido, que es donde estaba el problema, y elimina
información duplicada. **Contrapartida honesta:** se pierden las etiquetas de texto y la temperatura por
disco de un vistazo. Si prefieres conservarlas, el riel puede expandirse al pasar el ratón; queda
propuesto en `cambios/componentes/Sidebar.md` §Alternativa.

### 6. El asistente inicial, diseñado desde cero

**Problema observado** — `capturas/configuracion-inicial__*`: hoy es un `EmptyState` «no implementado».

**Cambio** — Cuatro pasos (Bienvenida · Discos · Alertas · Listo), uno por pantalla, sin chrome de
aplicación, con indicador de paso en la cabecera y salida «Omitir y usar los valores de fábrica» siempre
visible. El paso de discos permite excluir y renombrar, muestra la ocupación de cada volumen y explica
por qué el disco USB no expone SMART **sin tratarlo como avería**. Detalle en `cambios/08-onboarding.md`.

---

## Qué NO se toca

- El material de tres capas y sus desenfoques (28 / 24 / 44), el filo de 1 px y los hairlines.
- La escala de radios concéntricos (18 → 13 → 9 → cápsula) y que todo control sea cápsula.
- El movimiento: 220 ms con `cubic-bezier(.32,.72,0,1)`, `active:scale-[0.98]`, `prefers-reduced-motion`.
- El catálogo: no se elimina ningún componente. Se añaden tres y se recomponen cinco.
- Todas las reglas de producto: «No disponible» en vez de ceros, no-compatible en gris y nunca en rojo,
  procedencia visible, inferencia etiquetada, alertas agrupadas, confirmación antes de escribir o cargar,
  UI no bloqueante, contenido de eventos como texto y nunca como HTML.
- La arquitectura: ningún comando Tauri nuevo, ningún permiso nuevo. El rediseño es solo de presentación.

---

## Cambios de texto visible

Tres frases nuevas y un cambio de rótulo. No hace falta que toques los diccionarios ahora; aquí queda
qué frase va en cada sitio.

| Clave | es | en |
|---|---|---|
| `dashboard.hero.allGood` | Todo en orden | All good |
| `dashboard.hero.allGoodBody` | Ningún disco necesita atención ahora mismo. | No disk needs attention right now. |
| `dashboard.hero.overVendorLimit` | {count} picos por encima del límite desde las {time} | {count} peaks above the limit since {time} |
| `dashboard.hero.openDisk` | Abrir el disco | Open disk |
| `dashboard.hero.viewAlert` | Ver la alerta | View alert |
| `onboarding.step.welcome` | Bienvenida | Welcome |
| `onboarding.step.disks` | Discos | Disks |
| `onboarding.step.alerts` | Alertas | Alerts |
| `onboarding.step.done` | Listo | Done |
| `onboarding.disks.title` | Hemos encontrado {count} discos en este equipo | We found {count} disks on this PC |
| `onboarding.disks.body` | Puedes dejar fuera los que no te interesen y ponerles un nombre reconocible. Todo esto se cambia después en Ajustes, y ningún disco se modifica: SmartDisk solo lee. | You can leave out the ones you don't care about and give them a recognisable name. All of this can be changed later in Settings, and no disk is modified: SmartDisk only reads. |
| `onboarding.disks.aliasLabel` | Nombre para esta aplicación | Name inside this app |
| `onboarding.disks.usbNote` | El disco externo por USB no expone datos SMART: su puente no reenvía esos comandos. Eso no es una avería. Si lo dejas marcado, vigilaremos su capacidad y los sucesos de Windows que lo mencionen, pero no verás temperatura ni desgaste. | The external USB disk does not expose SMART data: its bridge does not forward those commands. That is not a fault. If you leave it checked we will watch its capacity and the Windows events that mention it, but you will not see temperature or wear. |
| `onboarding.skip` | Omitir y usar los valores de fábrica | Skip and use factory defaults |
| `onboarding.selectedCount` | {selected} de {total} discos seleccionados | {selected} of {total} disks selected |
| `settings.appearance.useSystemAccent` | Usar el color de acento de Windows | Use the Windows accent colour |
| `settings.appearance.useSystemAccentHint` | Sustituye el morado de la aplicación por el color que tengas configurado en Windows. | Replaces the app's purple with the colour configured in Windows. |
| `sidebar.globalStatus` (rótulo) | *(se elimina el texto; el estado global queda como icono con `aria-label`)* | — |

---

## Índice de la entrega

```
RESUMEN.md                        <- este fichero
cambios/
  00-tokens.md                    <- TODOS los cambios de token, con ratios medidos
  01-panel-general.md
  02-alertas.md
  03-eventos.md
  04-pruebas.md
  05-informes.md
  06-ajustes.md
  07-detalle-disco.md
  08-onboarding.md
  09-chrome-y-estados.md          <- Sidebar, Toolbar, diálogos y los cinco estados
  componentes/
    Icon.md                       <- NUEVO
    Sparkline.md                  <- NUEVO
    HeroPanel.md                  <- NUEVO
    Sidebar.md
    Toolbar.md
    DiskCard.md
    MetricCard.md
    Button.md
    ProgressBar.md
    StatusPill.md
mockups/
  smartdisk-v3.html               <- mockup navegable, 4 pantallas × 2 temas, autocontenido
  README-mockups.md               <- qué cubre y qué no
```

## Lista de comprobación

- [x] Ningún color, radio, sombra o tamaño fuera de `cambios/00-tokens.md`.
- [x] Todo token nuevo tiene valor claro y oscuro.
- [x] Mockup en claro y oscuro para panel, detalle, pruebas y onboarding. Alertas, eventos, informes y
      ajustes son **solo tokens**: nota explícita en cada fichero de pantalla.
- [x] Los cinco estados siguen cubiertos; los que cambian están en `09-chrome-y-estados.md`.
- [x] Comportamiento a 1024 × 560 descrito en cada pantalla.
- [x] No se apilan materiales.
- [x] El color siempre va con texto o icono.
- [x] Propuesta para `onboarding`.
