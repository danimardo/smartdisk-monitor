# Constitución de SmartDisk Monitor

Marco innegociable del proyecto. Está por encima de cualquier especificación, plan o
implementación: si una fase posterior la contradice, **la fase está mal**, no la constitución.

Este documento dice **qué no se puede cambiar sin una enmienda**. El *cómo* de cada área vive en
los documentos normativos, que desarrollan estos principios sin poder relajarlos:

| Documento | Desarrolla |
|---|---|
| `docs/product-specification.md` | Alcance funcional |
| `docs/architecture.md` | Estructura interna |
| `docs/data-model.md` | Entidades y retención |
| `docs/alert-rules.md` | Motor de alertas, regla por regla |
| `docs/ui-contract.md` | Comandos, eventos y errores |
| `docs/engineering-conventions.md` | Versiones, estructura, linters |
| `docs/testing-strategy.md` | Niveles de prueba, lotes y checkpoints (§VIII) |
| `docs/decisions.md` | Registro de decisiones técnicas (ADR) |
| `docs/open-questions.md` | Decisiones adoptadas y mediciones |
| `docs/ui-design.md` | Sistema de diseño y reglas de interfaz, vinculantes |
| `docs/known-issues.md` | Registro de silencios y fallos conocidos (§XIII) |

Toda la documentación del proyecto está en español, y esta constitución también.

---

## Principios fundamentales

### I. La veracidad del dato está por encima de todo lo demás (INNEGOCIABLE)

Este producto existe para que alguien pueda confiar en lo que le dice sobre sus discos. Un dato
falso es peor que ningún dato, porque destruye esa confianza sin avisar.

- **Nunca se inventa un valor.** Un dato que no se ha podido obtener es `null` y se presenta como
  «No disponible». Jamás `0`, `-1`, `"—"` ni cadena vacía.
- **No compatible ≠ averiado.** Un dispositivo que declara no exponer SMART se presenta en gris y
  **no** genera alerta. Un fallo de privilegios tampoco es incompatibilidad: se distinguen.
- **Una inferencia nunca se presenta como certeza.** Toda asociación deducida —un evento de Windows
  a un disco, una identidad por huella en vez de por número de serie— se etiqueta como tal.
- **Toda métrica lleva procedencia y antigüedad.** El usuario puede saber de qué fuente sale un
  número y de cuándo es. Un dato obsoleto se marca como obsoleto.
- **Un hueco se dibuja como hueco.** Las series temporales no se interpolan ni se rellenan; el eje
  cubre el intervalo pedido aunque falten datos, y toda gráfica declara su resolución, porque un
  máximo promediado no es un pico.
- **Nunca se alerta por ausencia de dato.** La falta de una métrica no es una anomalía.

*Por qué es el primer principio:* todos los demás pueden negociarse con un plazo o un presupuesto.
Este no. Un monitor de discos que miente es peor que no tener monitor.

### II. Orden de prioridades cuando hay conflicto (INNEGOCIABLE)

Cuando dos objetivos legítimos chocan y hay que elegir, **este es el orden**, y no admite
interpretación local:

1. **Veracidad del dato** — antes que la completitud, la velocidad o la estética.
2. **Seguridad y privacidad del usuario** — antes que la comodidad de uso o la de desarrollo.
3. **La aplicación responde** — una fuente caída degrada su tarjeta, nunca la interfaz entera.
4. **Claridad** — antes que densidad de información. Si una pantalla necesita más densidad, es que
   sobra información.
5. **Simplicidad de la solución** — antes que su generalidad. No se construye para casos que la
   versión 1.0 no tiene.

*Ejemplo de aplicación:* mostrar una temperatura de hace diez minutos marcada como obsoleta cumple
1 y 3. Mostrarla sin marcar viola 1. Bloquear la interfaz hasta tener una lectura fresca viola 3.

### III. Pila tecnológica fija (INNEGOCIABLE)

Las tecnologías de la sección **Pila y versiones** son obligatorias y exclusivas. En particular:

- **Nada de React**, ni de ninguna otra biblioteca de interfaz. La interfaz es Svelte 5 con runes.
- **Ninguna biblioteca de componentes de terceros.** El catálogo de `src/lib/components/` es el
  conjunto completo, y su ampliación está regulada por `AGENTS.md` §3.
- **Ninguna dependencia nueva sin justificación escrita** en `docs/decisions.md`. Toda dependencia
  añadida amplía la superficie de un binario privilegiado que se distribuye a terceros.
- **Cero red en funcionamiento normal.** No hay endpoints, ni telemetría, ni actualizador, ni
  fuentes o recursos remotos. Cualquier petición de red es una violación, no una optimización.
- **Windows x64 es la única plataforma de la 1.0.** El código se estructura para no impedir Linux o
  macOS en el futuro, pero no se construye para ellos hoy.

### IV. Separación entre dominio y presentación (INNEGOCIABLE)

Tres capas, con una regla de dependencias que solo apunta hacia dentro:

```
  Presentación  (src/routes, src/lib/components)
        │  solo llama a  ↓
  Acceso        (src/lib/api)  ←→  Comandos Tauri  (src-tauri/src/commands)
        │                                 solo llama a  ↓
  Dominio       (src-tauri/src/domain, alerts, persistence)
```

Reglas verificables:

- **`domain/` no conoce Tauri, ni Windows, ni SQLite.** Recibe datos y devuelve decisiones. Es lo
  que permite probar el motor de alertas con fixtures y sin hardware.
- **Ninguna pantalla invoca `invoke` directamente.** Todo pasa por `src/lib/api/`. Cuando una firma
  cambia, debe romper en un sitio y no en once.
- **Ningún componente decide reglas de negocio.** El color de un estado lo decide `health.ts`, el
  nivel de capacidad lo decide `capacityState()`, el estado de la bandeja lo decide `trayState()`.
  Un componente que recalcula una regla por su cuenta es un defecto, aunque acierte.
- **Los comandos Tauri son funciones finas**: validan argumentos y delegan. La lógica no vive ahí.
- **La interfaz no conoce el formato de los datos crudos.** No construye órdenes de `smartctl`, no
  interpreta códigos de salida, no analiza XML de eventos.

### V. Persistencia local, íntegra y trazable (INNEGOCIABLE)

- **Todo se almacena en SQLite**, en `%ProgramData%\SmartDisk Monitor\`, con WAL, transacciones
  breves y migraciones versionadas con checksum. Ningún otro almacén de datos estructurados.
- **`localStorage` está prohibido para estado del producto.** Idioma, tema, umbrales, frecuencias y
  selección de discos viven en la tabla `settings`. El navegador no es una base de datos.
- **Persistencia en UTC, presentación en hora local.** Sin excepciones.
- **Bytes para tamaños, bytes por segundo para caudales, Celsius para temperaturas.** La conversión
  a unidades legibles ocurre solo en la capa de presentación.
- **Los contadores acumulativos se guardan en bruto**, para poder calcular incrementos de forma
  fiable. No se guardan solo sus diferencias.
- **Antes de cada migración se hace una copia consistente**, y se conservan las tres últimas.
- **La retención nunca borra** alertas, ocurrencias críticas, eventos vinculados ni ejecuciones de
  pruebas. Solo compacta muestras.
- **Los datos sobreviven a la desinstalación.** Borrarlos es una acción explícita, separada y
  confirmada con una frase escrita por el usuario.
- **Toda escritura en disco ocurre en rutas calculadas y validadas canónicamente.** Nunca se
  sobrescribe un archivo existente.

### VI. Interfaz: sistema de diseño vinculante (INNEGOCIABLE)

`docs/ui-design.md` es normativo. Sus reglas duras, elevadas aquí porque su incumplimiento no
es un detalle estético:

- **Cero valores visuales literales.** Ni un color, radio, sombra o tamaño de fuente escrito a mano
  en un componente. Todo sale de `tokens.css` o de su mapeo Tailwind. Verificado en CI.
- **El color nunca es el único portador de significado.** Verde, ámbar, rojo y gris significan
  correcto, advertencia, crítico y desconocido, y siempre van acompañados de texto o icono.
- **El acento no comunica salud.** Es acción y selección. La aplicación tiene acento propio de
  fábrica; heredar el de Windows es una opción **apagada por defecto** (ADR-035). Cuando se
  activa, el acento del sistema se corrige antes de aplicarse para no romper el contraste
  (ADR-017), regla que no se relaja.
- **Cero literales de interfaz.** Todo texto visible —incluidos `aria-label`, `title` y `alt`— sale
  de los diccionarios español e inglés, que deben tener exactamente las mismas claves.
- **La ventana nunca recorta contenido en silencio.** Si no cabe, la región hace scroll.
- **Toda acción que escriba datos o genere carga se confirma** con su impacto explícito y el comando
  literal cuando exista.
- **El contenido de eventos y dispositivos se renderiza como texto, jamás como HTML.**
- **Reconocer una alerta no apaga su color.** El estado refleja la peor alerta no resuelta.
- **Una sola copia del sistema de diseño.** La fuente de verdad visual es
  `src/design-system/tokens.css` y el catálogo de `src/lib/components/`. No puede existir una
  segunda copia en el repositorio: una copia paralela diverge en silencio y el consolidado acaba
  publicando tokens caducos. Verificado en CI.

### VII. Accesibilidad: WCAG 2.2 nivel AA (INNEGOCIABLE)

El compromiso es el **estándar completo**, no una selección de criterios cómodos.

Ya verificado y medido, y que no puede degradarse:

| Criterio | Compromiso |
|---|---|
| 1.4.3 Contraste mínimo | 4.5:1 en ambos temas, verificado sobre los 262.144 acentos posibles |
| 1.4.11 Contraste de componentes | 3:1 en controles y elementos gráficos |
| 2.1.1 Teclado | Toda función alcanzable sin ratón, incluida la lectura de gráficas |
| 2.4.7 Foco visible | `:focus-visible` global; **anularlo es una violación** |
| 2.4.11 Foco no tapado | El elemento con foco nunca queda bajo el chrome translúcido |
| 2.3.3 Movimiento | `prefers-reduced-motion` respetado globalmente |
| 2.5.8 Tamaño del objetivo | 30 px de alto mínimo, por encima de los 24 px exigidos |
| 3.2.6 Ayuda consistente | Los mecanismos de ayuda aparecen en el mismo sitio en todas las pantallas |
| 3.3.7 Entrada redundante | No se pide dos veces un dato que la aplicación ya tiene |

Además: toda gráfica y todo anillo llevan `role="img"` con `aria-label` que resume el dato **y** una
lectura textual equivalente cerca. Los diálogos son `role="dialog" aria-modal` con foco atrapado y
cierre por `Escape`.

*Una pantalla que no cumple esto no está terminada, aunque funcione.*

### VIII. Testeabilidad y cobertura mínima (INNEGOCIABLE)

**Test-first donde el fallo es silencioso.** Es obligatorio escribir la prueba antes que el código,
con ciclo rojo-verde-refactor, en:

- parsers de `smartctl` y de logs NVMe;
- motor de alertas: activación, histéresis, deduplicación, ciclo de recaída;
- retención y agregación;
- validación de rutas del benchmark;
- migraciones de SQLite.

Son las áreas donde un error no se manifiesta como un fallo visible, sino como un dato equivocado
que alguien se cree. En interfaz, comandos finos y formateadores, las pruebas acompañan al código
pero no tienen que precederlo: una pantalla se diseña viéndola.

**Cobertura mínima, que bloquea la integración:**

| Área | Mínimo |
|---|---|
| `src-tauri/src/domain/`, `src-tauri/src/alerts/` | **90 %** |
| Resto de `src-tauri/src/` | **69 %** |
| `src/lib/` sin contar componentes | **70 %** |
| `src/lib/components/`, `src/routes/` | Sin umbral numérico |

En componentes y pantallas no se exige porcentaje **pero sí cobertura de estados**: cada componente
debe tener prueba de sus estados vacío, cargando, error de fuente, no compatible y dato obsoleto.
Un porcentaje uniforme incentivaría escribir pruebas triviales de marcado mientras el parser de
`smartctl` —donde un error inventa datos de salud— queda igual de cubierto que un botón.

**Pruebas obligatorias por cada regla de alerta**, sin excepción (`alert-rules.md` §5): activación,
**no** activación ante dato ausente, histéresis, deduplicación y ciclo de recaída.

**Los datos fixture se anonimizan al capturarlos**, nunca al usarlos. Ninguno contiene números de
serie ni nombres de equipo reales.

### IX. Seguridad y privacidad por construcción (INNEGOCIABLE)

- **Cero telemetría.** No se recopila ni se transmite nada, ni siquiera anónimo, ni siquiera para
  diagnóstico.
- **La interfaz web no puede ejecutar órdenes arbitrarias.** Sin shell genérica, sin `fs` abierto.
  El frontend solo alcanza comandos de una lista cerrada y enumerada.
- **Mínimo privilegio dentro de un proceso elevado.** La aplicación corre con `requireAdministrator`
  por necesidad de acceso a dispositivos; eso obliga a ser más restrictivo, no menos. Cada permiso
  de Tauri nuevo exige una entrada en `docs/decisions.md`.
- **Las exportaciones se anonimizan por defecto**, con sustitución consistente dentro de cada
  paquete, y el usuario ve un resumen del contenido antes de guardar.
- **SQL siempre parametrizado.**
- **Los procesos auxiliares tienen tiempo máximo, salida acotada y cancelación.**
- **Los binarios redistribuidos se verifican por hash antes de empaquetar.** Un binario privilegiado
  que se distribuye a terceros no se copia a ciegas.

### X. Errores comprensibles, con el detalle intacto (INNEGOCIABLE)

- **Todo fallo que cruce hacia la interfaz tiene la forma `AppError`**: código estable, clave i18n
  de la frase humana, detalle técnico literal sin traducir, fuente afectada y si merece reintento.
- **La frase humana siempre se muestra; el detalle técnico siempre se conserva** y es copiable.
  Nunca se enseña solo el detalle, ni se oculta del todo.
- **Un fallo de fuente no es un fallo de aplicación.** Degrada su tarjeta y el resto sigue.
- **Nada falla en silencio.** Un archivo temporal que no se pudo borrar, una fuente ilegible o una
  detección de codificación dudosa se dicen; no se dan por buenas.

### XIV. Arquitectura idiomática de SvelteKit, adaptada a Tauri (INNEGOCIABLE)

SvelteKit tiene un modelo mental propio y seguirlo ahorra trabajo. Pero **este proyecto no tiene
servidor**: se compila con `adapter-static` y `ssr = false`, y vive dentro de un WebView. Buena
parte del SvelteKit canónico —`+page.server.ts`, *form actions*, `use:enhance`, validación de
`formData` en servidor, mejora progresiva sin JavaScript— **no existe aquí y no se puede usar**. No
es una simplificación: sin JavaScript no hay ni ventana.

Lo que sigue es la traducción de ese modelo mental a esta arquitectura. Donde el patrón original no
aplica, se dice qué ocupa su lugar y por qué.

#### Carga de datos: `load` universal en `+page.ts`

- El estado inicial de una pantalla se pide en un **`load` de `+page.ts`** —universal, nunca de
  servidor— que llama a `$lib/api` y lo entrega por la prop `data`, **listo para renderizar**.
- Una pantalla **no arranca vacía para llenarse un instante después**. Ese salto es el que `load`
  evita, y además deja que SvelteKit gestione los estados de carga y de error por ruta.
- **`load` es solo para la carga inicial.** Las actualizaciones llegan por los eventos que empuja el
  backend y viven en el store (ADR-015). Nunca se recarga una ruta para refrescar datos.
- Un `load` que falle deja que el error suba a `+error.svelte`, que lo presenta con la forma del
  principio X. No se capturan errores para devolver datos vacíos que parezcan válidos.
- Las rutas con parámetro llevan `export const prerender = false`: sus identificadores no existen
  hasta que el backend enumera el hardware.

#### Mutación de estado: comandos, no formularios de servidor

No hay *actions* porque no hay servidor. Su lugar lo ocupan los comandos Tauri:

- Toda mutación pasa por **`$lib/api`**, nunca por `invoke` directo (principio IV).
- **La interfaz no aplica el cambio por su cuenta.** Invoca el comando y espera el evento que
  confirma el nuevo estado. Actualizar el store antes de la confirmación es inventarse un dato.
- Un comando que falla presenta su `AppError` y **deja la interfaz como estaba**. Nada de estados a
  medias.

#### Formularios: validar dos veces, confiar en una

- El **mismo esquema Zod** valida en el cliente al perder el foco, para dar respuesta inmediata y un
  mensaje concreto. Los esquemas de formulario viven en `src/lib/forms/`.
- **El backend revalida siempre.** Es quien conoce los límites reales y quien escribe en disco.
- **La validación del cliente es comodidad, nunca seguridad.** No se confía en ella para nada que
  toque el disco, los umbrales de alerta o la retención.
- Se usan elementos `<form>` y `<button type="submit">` de verdad: dan el `Enter` para enviar, el
  agrupamiento semántico y el anuncio correcto en lectores de pantalla, aunque el envío lo maneje
  JavaScript.

#### Navegación: enlaces, no `goto()`

- **Se navega con `<a href>`.** SvelteKit lo intercepta y hace la navegación de cliente. Un
  `onclick` con `goto()` destruye el ctrl+clic, el clic central, el menú contextual, el foco y el
  anuncio como enlace de un lector de pantalla: es un incumplimiento del principio VII, no una
  preferencia de estilo.
- Un componente que lleva a otra pantalla **recibe un `href`**, no un callback de navegación.
- **`goto()` queda para navegación programática real**: redirigir al terminar el asistente, volver
  al panel tras borrar los datos. Nunca como respuesta a un clic sobre algo que es un enlace.

#### Runes: `$derived` antes que `$effect`

- **`$effect` es el último recurso, no el primero.** Si un valor se calcula a partir de otros, es
  `$derived`. Usar un efecto para asignar estado derivado crea ciclos, ejecuciones de más y errores
  que solo aparecen bajo carga.
- `$effect` se reserva para lo que de verdad es un efecto secundario: suscribirse a algo externo,
  tocar el DOM directamente, arrancar un temporizador.
- **Todo `$effect` que suscriba algo devuelve su función de limpieza.** Esta aplicación se ejecuta
  durante semanas sin recargarse: una fuga que en una web nadie notaría, aquí crece hasta hacerse
  visible.
- `$derived.by` cuando el cálculo necesita varias sentencias; `$derived` para una expresión.
- **Nada de stores clásicos** (`writable`, `readable`) para estado de la aplicación. El estado vive
  en clases `.svelte.ts` con runes.
- Props con `$props()`; `export let` está prohibido.
- Los callbacks de un componente son **opcionales por defecto** (`onalgo = undefined`), porque un
  componente debe poder usarse sin ellos.

#### Componentes: snippets y composición

- **Snippets, no slots.** `{#snippet}` y `{@render}` son la API de Svelte 5; los slots quedan
  descartados.
- Un componente de presentación **no invoca comandos ni conoce `$lib/api`**. Recibe datos y emite
  intenciones (principio IV).
- `{#key}` para reiniciar un componente cuando cambia su identidad, en vez de un efecto que
  reinicialice a mano.
- `$app/state` en lugar de `$app/stores`, que está en desuso.

#### Dónde vive la lógica

El texto canónico de SvelteKit sitúa la lógica de negocio en `src/lib/services`. **Aquí no**, porque
la lógica de negocio vive en Rust (`domain/`, `alerts/`). En TypeScript solo hay:

| Carpeta | Qué contiene |
|---|---|
| `src/lib/api/` | Acceso al backend y esquemas de validación. Ningún cálculo |
| `src/lib/design/` | Lógica de **presentación**: estado→color, formato, tema, acento |
| `src/lib/stores/` | Estado de la aplicación, alimentado por eventos |
| `src/lib/forms/` | Esquemas de validación de formularios |
| `src/lib/components/` | Catálogo cerrado. Sin lógica de negocio |
| `src/routes/` | Composición de pantallas y `load`. Sin lógica de negocio |

**No se crea `src/lib/services/`.** Si algo parece necesitarlo, o es presentación y va a `design/`,
o es negocio y va a Rust.

#### Ciclo de vida largo

Una aplicación web se recarga cada pocos minutos y perdona las fugas. Esta se abre el lunes y sigue
abierta el viernes:

- **Toda suscripción se suelta** al desmontar: los oyentes de eventos, los `matchMedia`, los
  temporizadores.
- **Ningún `setInterval` para pedir datos** (ADR-015). Los temporizadores de presentación —refrescar
  el texto «hace 2 min»— sí son legítimos, y también se limpian.
- No se usa `window.location` ni recarga completa: destruiría el estado y las suscripciones.

### XV. Registro de actividad: una sola API, sin ruido y sin datos personales (INNEGOCIABLE)

Un log sirve para dos cosas: diagnosticar un fallo que ya ocurrió y entender qué estaba haciendo la
aplicación cuando ocurrió. Todo lo que no sirva a eso es ruido, y el ruido no es neutral: entierra
lo que importa y llena el disco del usuario, en un producto cuyo trabajo es precisamente vigilar
ese disco.

#### Una sola API, en los dos lados

- **Prohibido `console.log`, `console.debug`, `console.warn`, `console.error` y `println!` en el
  código de la aplicación.** Verificado en CI.
- En TypeScript, todo pasa por **`$lib/logger`**. En Rust, por **`tracing`**.
- **El código de una funcionalidad no conoce la biblioteca de registro.** Usa el envoltorio, para
  que cambiar de implementación no obligue a tocar cien ficheros.
- Excepción única: el propio envoltorio, y las herramientas de `tools/` y `scripts/`, que no son la
  aplicación.

#### Niveles, y qué significa cada uno

| Nivel | Para qué | Ejemplo |
|---|---|---|
| `error` | Algo falló y el usuario lo va a notar | No se pudo abrir la base de datos |
| `warn` | Algo falló y se ha podido continuar | Una fuente agotó su tiempo y se conserva la lectura anterior |
| `info` | Hitos del ciclo de vida, pocos y espaciados | Arranque, disco detectado, prueba iniciada, migración aplicada |
| `debug` | Detalle para diagnosticar | Orden ejecutada, duración de un ciclo, decisión de una regla |
| `trace` | Volcado exhaustivo, solo a mano | Contenido de una respuesta de `smartctl` |

**El nivel por defecto es `info`**, y `info` debe poder leerse entero después de un día de uso. Si
un mensaje se repite cada ciclo, no es `info`: es `debug`.

#### Cómo se elige el nivel

Por orden de precedencia:

1. **`--log-level=<nivel>`** en la línea de órdenes. Prevalece durante esa ejecución, y es la única
   forma de diagnosticar un fallo que ocurre **antes** de poder leer la configuración, que es justo
   cuando más falta hace. Un valor inválido no arranca en modo detallado «por si acaso»: avisa y usa
   el predeterminado.
2. **`settings.logging.level`**, la configuración persistida (principio V).
3. **`info`**.

**No se usan variables de entorno.** El principio XII lo prohíbe y aquí no hay servidor: `LOG_LEVEL`
y `PUBLIC_LOG_LEVEL` no tendrían dónde vivir.

#### Qué nunca aparece en un log

Este producto no maneja contraseñas ni tokens, pero sí datos que identifican a una persona y a su
equipo. **Nunca se registran**:

- números de serie de dispositivos;
- nombre del equipo ni nombre de usuario;
- rutas que contengan un perfil de usuario;
- etiquetas de volumen, que a menudo llevan nombres propios;
- el contenido de un fichero de prueba.

Cuando haga falta identificar un disco en el log se usa **su identificador interno**, que no
significa nada fuera de esta instalación. La misma regla que ya rige las exportaciones (principio
IX) rige el registro: si algo no puede salir en un ZIP de diagnóstico, tampoco puede entrar en un
log, porque **el log va dentro del ZIP**.

#### Formato

- **Hora local del equipo con su desplazamiento explícito**, no UTC y no una zona fija.
  La razón es concreta: esta aplicación correlaciona sus métricas con el Visor de eventos de
  Windows, que muestra hora local. Un log en otra zona obligaría a convertir mentalmente cada vez
  que se coteja un pico de temperatura con un evento de disco. El desplazamiento explícito evita la
  ambigüedad cuando el fichero viaja por correo.
- **Legible por una persona con un editor de texto.** El fichero de log se abre con el Bloc de notas
  el 90 % de las veces; ese es el caso que hay que optimizar.
- Cada entrada lleva: hora, nivel, módulo de origen y mensaje. El contexto va como pares
  `clave=valor`, no incrustado en la frase.
- Los mensajes se escriben en **español**, como el resto del proyecto, y **no pasan por i18n**: no
  son interfaz, son diagnóstico.

#### Frontend

El WebView no puede escribir ficheros. Por tanto:

- `debug` e `info` van **solo a la consola** del WebView. Son ruido de desarrollo y enviarlos por
  IPC costaría más que el valor que aportan.
- `warn` y `error` van a la consola **y se envían a Rust**, que los escribe en el fichero único. Así
  un fallo de interfaz en el equipo de un usuario aparece en el ZIP de diagnóstico sin tener que
  pedirle que abra las herramientas de desarrollo.
- **El envío nunca puede tumbar la interfaz**: si el comando de registro falla, se ignora. Un log
  que rompe la aplicación es peor que no tener log.

#### Rotación y retención

Rotación diaria, 30 días y unos 100 MB como máximo (especificación §7). El registro **nunca crece
sin límite**: es un monitor de discos y llenar el disco del usuario sería una ironía cara.

### XI. Todo dato que cruza una frontera se valida en ejecución (INNEGOCIABLE)

Un tipo de TypeScript es una promesa sobre lo que *debería* llegar, no una comprobación de lo que
llega. `invoke<DeviceListResponse>(...)` no valida nada: es una aserción de tipo sobre un dato que
viene de otro proceso. Si el backend cambia un campo, TypeScript no se entera y la interfaz muestra
`undefined` como si fuera un dato — exactamente lo que prohíbe el principio I.

**Regla general:** ningún dato procedente de fuera del módulo que lo consume entra en la lógica sin
haber sido validado en ejecución. Sin excepciones por comodidad ni por rendimiento.

#### En TypeScript: Zod

- **Zod es la biblioteca estándar** de validación y análisis. No se añade otra.
- **Los tipos se infieren del esquema** con `z.infer<typeof Esquema>`, nunca se declaran a mano en
  paralelo. Dos declaraciones de la misma forma acaban divergiendo; una sola no puede.
- **Queda prohibida la aserción de tipo directa** (`as`, `<T>`, `satisfies` usado como escape) sobre
  cualquier dato que provenga de: respuestas de comandos Tauri, cargas útiles de eventos, contenido
  de ficheros, portapapeles, `FormData`, parámetros de URL, o cualquier cosa que no haya construido
  el propio módulo. `as` sigue siendo legítimo para estrechar un tipo que ya se ha validado.
- **Toda frontera tiene su esquema**, y el esquema vive junto al contrato que describe, no disperso
  por las pantallas.
- **Un fallo de validación produce un `AppError`** con código `ipc.schema_mismatch`, la frase humana
  correspondiente y, en el detalle técnico, la ruta del campo y lo que se esperaba. Nunca se ignora
  en silencio ni se rellena con un valor por defecto: un dato con forma inesperada es un dato
  desconocido, y el principio I dice qué hacer con eso.

#### En Rust: serde estricto y validación explícita

La misma regla, con las herramientas del lenguaje. Toda fuente externa se deserializa en tipos
explícitos, nunca se navega como valor libre:

| Frontera | Cómo se valida |
|---|---|
| JSON de `smartctl` | Tipos serde explícitos. La versión del binario puede cambiar el formato |
| XML del registro de eventos | Tipos explícitos; los campos ausentes son `Option`, no cadenas vacías |
| Filas de SQLite | Validadas al leer: pueden haberlas escrito versiones anteriores de la aplicación |
| Registro de Windows | Validado antes de convertir, incluido el orden de bytes |
| Salida de procesos auxiliares | Codificación detectada, nunca supuesta |

**`serde_json::Value` no es un tipo de dominio.** Puede usarse como paso intermedio para conservar
una captura en bruto, jamás para alimentar una decisión.

#### Por qué aquí importa más que en una aplicación web

Este producto no tiene red, así que la tentación es pensar que no hay datos no confiables. Los hay,
y son peores: la salida de un binario de terceros cuya versión puede cambiar, un registro de eventos
escrito por controladores de fabricantes distintos, y una base de datos que la propia aplicación
escribió hace seis meses con otro esquema. Ninguno de esos tres avisa cuando cambia de forma.

### XII. Configuración: sin variables de entorno en la aplicación (INNEGOCIABLE)

Esta aplicación **no tiene servidor**: se compila con `adapter-static` y `ssr = false`, y se ejecuta
dentro de Tauri. De ahí se derivan reglas que no son preferencias, sino consecuencias técnicas.

#### Prohibido en el código de la aplicación

- **`process.env`**, en cualquier fichero de `src/` o `src-tauri/`. En el navegador no existe, y en
  el backend la configuración vive en `settings`.
- **`$env/dynamic/private` y `$env/static/private`**: no existen sin un runtime de servidor. No hay
  ficheros `.server.ts` en este proyecto y no puede haberlos.
- **`$env/dynamic/public`**: no funciona con prerenderizado. Si alguna vez hiciera falta exponer una
  constante de compilación al cliente, la vía es `$env/static/public` con prefijo `PUBLIC_`, y
  requiere enmienda de esta constitución.

#### Permitido, y solo ahí

`.env` existe **únicamente para variables de construcción**, leídas por Node durante el build o al
ejecutar las pruebas: `vite.config.ts`, `svelte.config.js`, `vitest.config.ts`,
`vitest.browser.config.ts` y `playwright.config.ts`. Hoy son las `TAURI_*` que inyecta la propia
herramienta, y `CI`, que decide reintentos y formato de informe. Ninguno de esos ficheros se
empaqueta, y leer `CI` no es configurar el producto. La lista vinculante está en
`scripts/verify-boundaries.mjs`, que es quien lo comprueba.

#### La configuración del producto vive en SQLite

Idioma, tema, frecuencias, umbrales, retención, comportamiento al cerrar y selección de discos van a
la tabla `settings` (principio V). No hay una segunda fuente de configuración, porque dos fuentes
acaban discrepando y nadie sabe cuál manda.

#### No hay secretos, y si los hubiera no irían en un `.env`

La aplicación no tiene cuentas, ni claves de API, ni servicios externos (ADR-007). Y conviene
dejarlo escrito para el futuro: **un `.env` empaquetado en un instalador de escritorio no es
secreto**. Cualquiera puede abrir el instalador y leerlo. Si algún día hiciera falta guardar una
credencial, la vía es el almacén de credenciales de Windows (DPAPI), nunca un fichero de texto junto
al ejecutable.

### XIII. Validación de tipos de la interfaz antes que nada (INNEGOCIABLE)

Todo cambio que toque componentes Svelte, TypeScript, rutas, layouts, configuración de SvelteKit o
contratos de componentes **se valida con `svelte-check`**.

- **El comando es reproducible y único**: `pnpm check` ejecuta primero `svelte-kit sync` y después
  `svelte-check` con el `tsconfig.json` real del repositorio. No hay invocaciones alternativas ni
  banderas locales.
- **Se ejecuta antes** que las pruebas unitarias, la compilación y las pruebas de extremo a extremo,
  tanto en local como en integración continua. Un error de tipos hace inútil todo lo que venga
  después: mejor descubrirlo en el primer minuto que en el décimo.
- **Cero errores y cero avisos.** Una implementación con avisos de `svelte-check` no está terminada.
  Los avisos de accesibilidad son, además, incumplimientos del principio VII.
- **Quien valida, ejecuta.** Ningún agente ni persona puede afirmar que la validación ha pasado sin
  haber ejecutado el comando y registrado su resultado. Suponer que algo compila no es comprobarlo.
- **Un fallo preexistente se documenta, no se esconde.** Si aparece un aviso que no puede resolverse
  dentro del alcance del cambio, se registra en `docs/known-issues.md` con su causa, su fecha y quién
  debe resolverlo, y el silencio en el código enlaza a esa entrada. **Un `svelte-ignore` sin enlace
  a una entrada del registro hace fallar la integración.** Corregir un fallo preexistente fuera del
  alcance tampoco es libre: se acuerda antes, para que la revisión no mezcle dos cambios distintos.

---

## Pila y versiones

Obligatorias y exclusivas. Las versiones son las verificadas en el proyecto; cambiar una **minor**
o **major** requiere enmienda de esta constitución.

### Ejecución

| Pieza | Versión | Nota |
|---|---|---|
| Tauri | 2.11.5 | ADR-001 |
| Rust | 1.94.0, edición 2021, MSRV 1.77 | fijado en `rust-toolchain.toml` |
| Node | 24 LTS (mínimo 20) | fijado en `.nvmrc` |
| pnpm | 11.6.0 | gestor exclusivo; npm y yarn quedan excluidos |
| SvelteKit | 2.70.3 con `adapter-static`, SSR off | ADR-014 |
| Svelte | 5.57.0, runes | sin `export let`, sin stores para estado local |
| TypeScript | 5.9.3, `strict: true` | sin `any` implícito ni `@ts-ignore` sin justificar |
| Vite | 6.4.3 | |
| Tailwind CSS | 3.4.19 | solo utilidades mapeadas desde tokens |
| SQLite | vía `rusqlite`, `bundled` | ADR-006; sin depender de la DLL del sistema |
| Zod | 4.5.4 | validación en ejecución de toda frontera (principio XI, ADR-022) |
| Registro | `tracing` en Rust, envoltorio propio en TS | principio XV, ADR-024. No se usa Pino ni `loglevel` |

### Dependencias de Rust

| Crate | Versión |
|---|---|
| `tauri` | 2.11.5 |
| `tauri-build` | 2.6.3 |
| `tauri-plugin-single-instance` | 2.4.4 |
| `tauri-plugin-notification` | 2.4.0 |
| `serde` | 1.0.229 |
| `serde_json` | 1.0.151 |
| `thiserror` | 2.0.20 |
| `windows-registry` | 0.6.1 |
| `rusqlite` | 0.40.2, `bundled` |
| `sha2` | 0.10.9 |
| `ts-rs` | 10.1.0 |
| `tracing` + `tracing-subscriber` + `tracing-appender` | 0.1 / 0.3 / 0.2 |
| `time` | 0.3 |

### Herramientas de calidad

| Herramienta | Versión |
|---|---|
| `vitest` | 5.0.0 |
| `@vitest/browser` + `@vitest/browser-playwright` | 5.0.0 |
| `vitest-browser-svelte` | 3.1.0 |
| `playwright` + `@playwright/test` | 1.62.1 |
| `@axe-core/playwright` | 4.13.0 |
| `svelte-check` | 4.7.6 |
| `eslint` | 9.39.5 |
| `prettier` | 3.9.6 |
| `jsdom` | 25.0.1 |
| `rustfmt`, `clippy` | los del toolchain 1.94.0 |

### Recursos redistribuidos

| Recurso | Versión | Licencia |
|---|---|---|
| smartmontools (`smartctl` + `drivedb.h`) | 7.5, x64, r5714 | GPL-2.0-or-later, fuente incluida en el instalador |
| Instrument Sans | v4, subconjuntos latin y latin-ext | SIL OFL 1.1 |
| WebView2 Runtime | Evergreen, instalador sin conexión | ADR-020 |

### Plataforma soportada

Windows 10 1809 (build 17763) o superior y Windows Server 2016, 2019, 2022 y 2025 con Experiencia de
escritorio, **x64**, con CPU compatible con SSE3.

### Política de versiones

Se usan **rangos compatibles** (`^` en npm, compatible por defecto en Cargo) para que los parches de
seguridad entren al reinstalar. La reproducibilidad la garantizan dos reglas que **no son
opcionales**:

- **El lockfile se versiona siempre**, tanto `pnpm-lock.yaml` como `Cargo.lock`.
- **CI instala con `--frozen-lockfile` y `--locked`.** Una compilación jamás resuelve versiones por
  su cuenta.
- **Actualizar dependencias es un cambio deliberado**, en su propio commit y con su revisión. Nunca
  un efecto colateral de otra tarea.

---

## Puertas de calidad

Ninguna se puede saltar. Todas son automáticas salvo las marcadas como revisión humana.

Implementadas en `.github/workflows/ci.yml`. La tabla es la norma; el workflow es su ejecución.

**El orden importa** (principio XIII): `pnpm check` se ejecuta **antes** que las pruebas, la
compilación y las pruebas de extremo a extremo. Un error de tipos invalida todo lo que venga
después.

### Por cada cambio (bloquean la integración)

| Puerta | Comprobación |
|---|---|
| Formato | `rustfmt` y `prettier` sin diferencias |
| Análisis estático | `clippy -D warnings`; `eslint` sin errores ni avisos; `svelte-check` con **cero errores y cero avisos** |
| Tipos del contrato | Los DTO generados desde Rust con `ts-rs` coinciden con los versionados en `src/lib/api/generated/`; `cargo test` los regenera y una diferencia es un fallo de la puerta |
| Pruebas | `cargo test`, `pnpm test`, `pnpm test:component`, `pnpm test:e2e` y `pnpm test:a11y` en verde; cobertura por encima de los mínimos del principio VIII |
| Sistema de diseño | Cero colores, radios, sombras o tamaños literales; cero `backdrop-filter` a mano |
| i18n | `es.json` y `en.json` con idénticas claves; interpolaciones coherentes; ninguna clave usada que no exista |
| Recursos | Hashes de la tipografía y de `smartctl` coinciden con `THIRD_PARTY_NOTICES.md` |
| Permisos | Ningún permiso de Tauri nuevo sin su ADR |
| Documentación | `historias.md` regenerado si cambió algún documento normativo |
| Validación de fronteras | Ninguna aserción de tipo sobre datos de `invoke`, `listen` o ficheros; todo esquema Zod tiene su prueba de rechazo |
| Variables de entorno | Ningún `process.env` ni `$env/*` en `src/` o `src-tauri/` |
| Silencios justificados | Todo `svelte-ignore` o `eslint-disable` enlaza a una entrada de `docs/known-issues.md` |
| Registro | Ningún `console.*` ni `println!` en el código de la aplicación |

**Puerta activa.** `ts-rs` genera los DTO en `src/lib/api/generated/` desde los comandos de
inventario (T029-T030, spec `001-monitor-discos-windows`). `cargo test` los regenera; una
diferencia entre lo generado y lo versionado bloquea la integración. `src/lib/api/types.ts`
reexporta los tipos con comando real y mantiene a mano solo los que aún no lo tienen.

### Por cada pantalla (revisión humana, `AGENTS.md` §8)

Tema claro y oscuro; acento del sistema y de respaldo; 1024 × 560 y 1280 × 720; escalado 125 %,
150 % y 200 %; estados cargando, vacío, no compatible, error de fuente y dato obsoleto; teclado y
foco; textos en los dos idiomas.

### Por cada versión publicada

Compilación empaquetada probada, no solo en desarrollo. Migraciones verificadas desde cada versión
publicada anterior. Avisos de terceros actualizados. Nombre y versión tomados del manifiesto.

---

## Flujo de desarrollo

1. **Nada se implementa contra una ambigüedad.** Si al construir aparece una decisión que no está
   escrita, se añade a `docs/open-questions.md` con su valor propuesto **antes** de programarla.
   Resolver una ambigüedad dentro del código, en silencio, es la infracción más grave del proceso.
2. **Una historia se cierra** con sus criterios de aceptación cumplidos, sus pruebas, sus textos en
   los dos idiomas, sus estados de error y vacío diseñados, y la definición de terminado de
   `AGENTS.md` §8 si toca interfaz.
3. **Ramas por historia**, commits en imperativo y en español, referenciando la historia.
4. **Versionado semántico.** Nombre y versión salen del manifiesto (ADR-011): no se escriben a mano
   en ningún otro sitio.
5. **Publicación manual**, sin actualizador automático (ADR-007).
6. **Los riesgos de la Fase 0 se miden, no se estiman.** Una decisión de arquitectura basada en una
   suposición no comprobada no es una decisión: es una apuesta. Cuatro suposiciones de la
   especificación original resultaron falsas al medirlas (`open-questions.md` §L a §Q).

---

## Gobernanza

**Esta constitución prevalece sobre cualquier otra práctica, documento o costumbre del proyecto.**
Un plan, una especificación de fase o una implementación que la contradiga es defectuoso por esa
sola razón, sin necesidad de más argumento.

### Enmiendas

- Toda enmienda se hace **en este documento y antes** de escribir el código que la necesita.
- Requiere: la razón del cambio, qué se rompe, y qué documentos normativos hay que actualizar en
  consecuencia.
- Cambiar una versión **major** o **minor** de la pila, relajar un umbral de cobertura, rebajar el
  nivel de accesibilidad o añadir una tecnología son enmiendas, no ajustes.
- Corregir una errata o precisar una redacción sin cambiar la norma es una revisión **patch** y no
  requiere aprobación.

#### Historial

| Versión | Fecha | Cambio |
|---|---|---|
| 1.0.0 | 2026-09-04 | Ratificación inicial: principios I a X |
| 1.1.0 | 2026-09-04 | Principios XI (validación de fronteras con Zod), XII (configuración sin variables de entorno) y XIII (validación de tipos antes que nada). Ninguna norma anterior se relaja |
| 1.2.0 | 2026-09-04 | Principio XIV (arquitectura idiomática de SvelteKit adaptada a Tauri): carga con `load`, navegación por enlaces, `$derived` antes que `$effect`, y dónde vive la lógica. Ninguna norma anterior se relaja |
| 1.3.0 | 2026-09-04 | Principio XV (registro de actividad): API única, niveles y su significado, precedencia del nivel, prohibición de datos personales, formato en hora local. Ninguna norma anterior se relaja |
| 1.3.1 | 2026-09-04 | `tauri-plugin-single-instance` 2.4.4 entra en la pila fija (ADR-025). No se añade, relaja ni reinterpreta ningún principio: solo actualiza la tabla de dependencias de Rust que exige el principio III |
| 1.4.0 | 2026-09-04 | Infraestructura de pruebas de componente, interfaz y accesibilidad (ADR-027, ADR-028): Vitest sube a 5, entran Browser Mode, Playwright y axe, y sale `@testing-library/svelte`. La puerta de pruebas pasa a exigir las tres suites nuevas, y `playwright.config.ts` y `vitest.browser.config.ts` se suman a los ficheros donde `process.env` es legítimo. Es `minor` por lo que **añade** a la puerta de calidad; ningún principio cambia de contenido ni se relaja |
| 1.5.0 | 2026-09-04 | Una sola copia del sistema de diseño (ADR-029): la norma de interfaz pasa a `docs/ui-design.md`, absorbe el antiguo `HANDOFF.md` y gana un §0 con el mapa de rutas; el principio VI añade la prohibición de una segunda copia, que `pnpm verify:tokens` comprueba. Es `minor` porque **añade** una regla y corrige rutas; ningún principio cambia de contenido ni se relaja |
| 1.5.1 | 2026-09-05 | `sha2` 0.10.9 y `ts-rs` 10.1.0 entran en la pila fija; la puerta de tipos del contrato pasa de pendiente a activa (spec `001-monitor-discos-windows`, T029-T030). No se añade, relaja ni reinterpreta ningún principio: actualiza la tabla de dependencias que exige el principio III y cumple lo que el principio VIII ya preveía |
| 1.5.2 | 2026-09-05 | `tauri-plugin-notification` 2.4.0 entra en la pila fija (T053, notificaciones nativas de alertas). No se añade, relaja ni reinterpreta ningún principio: solo actualiza la tabla de dependencias que exige el principio III |
| 1.6.0 | 2026-09-06 | Principio VIII: el mínimo de «resto de `src-tauri/src/`» baja de 80 % a 69 % (K.6, medido con `cargo llvm-cov`: 69,80 %). No es una excepción de dos casos como se planteó al principio — separar los envoltorios `#[tauri::command]` a un fichero excluido de la medición resultó no ser honesto: ~20 de los 35 tienen lógica real sin extraer a un `_impl`, y excluirlos habría escondido código sin probar. El déficit es arquitectónico (el envoltorio no se puede instanciar en un `#[test]` sin un proceso de Tauri real), no pereza de pruebas; extraer esa lógica a funciones `_impl` con prueba propia es mejora futura que subirá este mínimo de nuevo, no una condición para esta enmienda. Es `minor` porque **ajusta** un número a la realidad medida sin relajar la disciplina de prueba de ningún otro principio |
| 1.7.0 | 2026-09-06 | Principio VI, viñeta del acento: se precisa que heredar el acento de Windows es una opción **apagada de fábrica** (ADR-035, spec `002-rediseno-v3`), no el comportamiento por defecto. La corrección de contraste del acento heredado (ADR-017) no se toca. Es `minor` porque **añade** una precisión que refleja una decisión ya adoptada; ningún principio se relaja ni cambia de contenido |

### Cumplimiento

- Cada revisión de código verifica el cumplimiento de forma explícita, no por omisión.
- **La complejidad se justifica o se retira.** Ante dos soluciones que funcionan, se elige la más
  simple; la más general solo gana si el caso adicional existe hoy, no si podría existir.
- Las puertas automáticas son la línea de defensa, no la sustitución del criterio. Que algo pase CI
  no lo hace correcto.

### Interpretación

Ante una duda sobre qué exige esta constitución, la lectura correcta es **la más restrictiva de las
razonables**. Si dos principios entran en conflicto, decide el orden de prioridades del principio II.

---

**Versión**: 1.7.0 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-06
