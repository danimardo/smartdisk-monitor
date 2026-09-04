# Estrategia integral de testing

Documento normativo. Desarrolla el principio VIII de la constitución (testeabilidad y cobertura
mínima) y no puede relajarlo.

**Este proyecto no es una aplicación web.** Es una aplicación de escritorio Tauri con un frontend
SvelteKit compilado a estático. Buena parte del repertorio habitual de testing de SvelteKit —SSR,
hidratación, Server Actions, endpoints, cookies, sesiones— **no existe aquí y no se puede probar**.
Lo primero que hace este documento es decir qué no aplica y por qué, para que nadie monte
infraestructura para algo que no está.

---

## 1. Clasificación del proyecto y runtime real

| | |
|---|---|
| Tipo | Aplicación de escritorio de un solo usuario, sin red |
| Shell | Tauri 2, proceso Rust elevado (`requireAdministrator`) |
| Frontend | SvelteKit con `adapter-static`, `ssr = false`, `prerender = true` |
| Motor de render | WebView2 (Chromium), embebido en el proceso |
| Backend | Rust en el mismo proceso; se comunica por IPC de Tauri |
| Persistencia | SQLite local en `%ProgramData%` |
| Plataforma | Windows x64 exclusivamente |

**Frontera principal a probar: el IPC.** No hay HTTP, no hay sesión, no hay usuario remoto. Todo lo
que cruza entre TypeScript y Rust pasa por `invoke` y `listen`, y es ahí donde se concentra el
riesgo de contrato.

### Reparto del código

| Ámbito | Dónde | Cómo se prueba |
|---|---|---|
| Solo navegador | `src/lib/components/`, `src/routes/` | Componentes en navegador real, E2E de interfaz |
| Solo servidor | `src-tauri/src/` | `cargo test`, con fixtures anonimizados |
| Compartido de facto | `src/lib/api/schemas.ts` ↔ tipos de Rust | Contrato: mismo caso de prueba a ambos lados |
| Presentación | `src/lib/design/`, `src/lib/i18n/` | Unit tests en Node |
| Estado | `src/lib/stores/` | Unit tests en Node |

---

## 2. Lo que NO aplica, y por qué

No se monta infraestructura para nada de esto. Está escrito para que nadie lo proponga otra vez.

| Técnica habitual | Por qué no aplica |
|---|---|
| Testing de SSR | `ssr = false`. No hay render en servidor que comparar |
| Testing de hidratación | Sin SSR no hay hidratación: el HTML llega vacío y lo pinta el cliente |
| Server Actions | No hay servidor. Las mutaciones son comandos Tauri |
| Endpoints `+server` | Cero endpoints. Cero HTTP |
| Hooks (`hooks.server`, `hooks.client`) | No existen ni pueden existir |
| Cookies y sesiones | No hay sesión: un solo usuario local ya autenticado por Windows |
| Autenticación y autorización | La autorización es UAC, del sistema operativo. No hay roles ni permisos de aplicación |
| Testing de APIs externas | Cero red en funcionamiento normal (ADR-007) |
| Cross-browser | **Se distribuye un solo motor**: el WebView2 que instala el propio instalador. No hay Firefox ni WebKit que soportar. Probar en ellos mediría un entorno que ningún usuario tendrá |
| Prerendering dinámico | Las rutas se prerrenderizan al compilar salvo `/disks/[id]`; no hay contenido de servidor que validar |
| Testing de despliegue | La distribución es un instalador manual, no un servidor |

**Consecuencia práctica:** la matriz de navegadores tiene un solo elemento. Eso ahorra la parte más
cara de una estrategia E2E convencional y permite invertir ese presupuesto en regresión visual y
en el dominio.

---

## 3. Inventario de lo que ya existe

Verificado antes de escribir esta estrategia. **No se instala nada que ya esté resuelto.**

| Capa | Herramienta | Estado |
|---|---|---|
| Tipos y accesibilidad | `svelte-check` | En uso, cero errores y cero avisos exigidos (§XIII) |
| Análisis estático TS | `eslint` con `typescript-eslint` | En uso, sin errores. Centrado en promesas sin gestionar |
| Formato | `prettier` | En uso |
| Análisis estático Rust | `clippy -D warnings`, `rustfmt` | En uso |
| Unit TS | `vitest` + `@vitest/coverage-v8` | En uso: 160 pruebas, umbral 70 % |
| Unit Rust | `cargo test` | En uso: 11 pruebas |
| Componentes | `@testing-library/svelte` | **Instalado y sin usar: cero tests de componente** |
| Validación de contrato | `zod` | En uso en la frontera IPC (§XI) |
| Verificadores propios | 5 scripts en `scripts/` | En uso: recursos, tokens, i18n, fronteras |
| E2E | — | **No existe** |
| Visual | — | **No existe** |
| Mutation | — | **No existe** |

Duración medida: `vitest` 22 s, `cargo test` 2 s. Ese es el presupuesto que hay que preservar.

### Deuda identificada

1. **`@testing-library/svelte` instalado sin un solo test.** Se resuelve en el lote L1.
2. **Ningún test de componente ni E2E**, pese a que el sistema de diseño es vinculante y su
   incumplimiento es un defecto de producto, no estético.

---

## 4. Pirámide de niveles

### Nivel 0 — Comprobaciones estáticas

No son tests y no sustituyen a ninguno: demuestran que el código es coherente, no que hace lo
correcto. Comandos reales:

```sh
pnpm check              # svelte-kit sync + svelte-check
pnpm lint               # prettier --check + eslint
pnpm verify             # recursos, tokens, i18n, fronteras
pnpm build              # compilación del frontend
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Se ejecutan **antes** que las pruebas (§XIII): un error de tipos invalida todo lo que venga después.

### Nivel 1 — Unit (Node)

Lógica aislable, sin DOM ni IPC. Es la capa más barata y donde debe estar la mayoría.

### Nivel 2 — Componentes (navegador real)

Componentes Svelte en Chromium mediante Vitest Browser Mode. **No jsdom**: el sistema de diseño
depende de `backdrop-filter`, `color-mix()`, variables CSS resueltas y `prefers-reduced-motion`, y
jsdom no implementa ninguna de esas cosas — un test de contraste o de material en jsdom no probaría
nada, solo daría una falsa sensación de cobertura.

### Nivel 3 — Integración

Colaboración real entre piezas: esquema Zod contra respuesta serializada por Rust, store contra
eventos, retención contra SQLite en fichero temporal.

### Nivel 4 — Aceptación

Criterios de aceptación de las historias convertidos en comportamiento verificable, al nivel más
barato que lo demuestre.

### Nivel 5 — E2E, en dos planos

| Plano | Herramienta | Qué demuestra | Coste |
|---|---|---|---|
| **Interfaz** | Playwright contra `preview`, con `mockIPC` | Navegación, estados, formularios, responsive, accesibilidad, visual | Segundos |
| **Aplicación real** | WebdriverIO + `tauri-driver` contra el `.exe` | Que el binario arranca, pinta y habla con Rust | Minutos |

El segundo plano existe porque **el primero no puede demostrar que la aplicación real funcione**: un
fallo de empaquetado, de política de capacidades o de elevación no aparecería en ningún navegador.

### Nivel 6 — Transversales

Responsive, accesibilidad, regresión visual. Cross-browser no aplica (§2).

### Fuera de la pirámide — Mutation testing

No es una capa superior: mide si los tests que ya existen detectan errores. Se ejecuta de forma
programada, nunca en el bucle de desarrollo.

---

## 5. Lotes funcionales y checkpoints

La unidad de trabajo es el **lote funcional coherente**, no el fichero ni la función. Entre tres y
ocho tareas como orientación, sin convertirlo en regla.

**Lote más pequeño** cuando haya: escritura en disco, motor de alertas, migraciones, rutas del
benchmark, elevación de privilegios, concurrencia entre recopiladores.

**Lote mayor** cuando sea: componente presentacional, adaptador fino, refactorización interna,
traducción de textos.

### Plan previo de cada lote

Antes de implementar se documenta en `tasks.md`: identificador, historia, comportamientos
observables, estados, errores, casos límite, riesgos, niveles de test previstos, datos y dobles
necesarios, suite de cierre y criterios del checkpoint.

No es obligatorio crear los ficheros de test por adelantado.

### Cuándo se escribe el test

**Antes o inmediatamente después** —sin excepción— cuando se trate de:

- parsers de `smartctl` y de logs NVMe;
- motor de alertas: activación, histéresis, deduplicación, ciclo de recaída;
- retención, agregación y migraciones;
- validación de rutas del benchmark;
- cualquier defecto reproducido: la prueba que lo captura se escribe antes de arreglarlo.

Son las áreas donde el fallo es silencioso: no revienta, produce un dato equivocado que alguien se
cree. En interfaz y cableado, la prueba acompaña al código pero no tiene que precederlo.

### Deuda de test

Solo puede existir **dentro del lote activo**. No se acumulan lotes sin pruebas, no se cierra una
historia con pruebas obligatorias pendientes, y una pantalla no está terminada porque funcione al
mirarla.

---

## 6. Unit testing

Framework: `vitest` (ya en uso). Ejecuta en Node.

Cubre: validadores, transformaciones, parsers, formateadores, cálculos, reglas de negocio, mapeadores,
serialización, límites, máquinas de estado, normalización, paginación y fechas.

**Reglas duras derivadas del principio I:**

- Todo formateador tiene prueba de **dato ausente**: `null`, `undefined` y `NaN` producen
  «No disponible», nunca cero.
- Todo cálculo tiene prueba de **cero real**, para demostrar que `0` no se confunde con ausente.
- Toda regla con umbral tiene prueba **en el umbral, justo por encima y justo por debajo**.

**Prohibido en un unit test:** red, hora real, aleatoriedad sin semilla, esperas reales, dependencia
del orden de ejecución, acceso a `%ProgramData%`.

---

## 7. Component testing

**Vitest Browser Mode con proveedor Playwright, en Chromium.** Se retira `@testing-library/svelte`
si el modo navegador lo hace redundante; no se mantienen dos soluciones equivalentes.

Se prueba un componente de forma aislada cuando sea reutilizable, tenga varios estados, contenga
comportamiento, o su contrato accesible importe.

### Qué se comprueba en cada componente del catálogo

Además de lo obvio, la definición de terminado de `AGENTS.md` §8 exige estados que hay que probar:

| Estado | Por qué |
|---|---|
| Vacío | Distinto de «error»: no tener discos no es un fallo |
| Cargando | No debe mostrar ceros mientras llega el dato |
| No compatible | **En gris, nunca en rojo**: un USB sin SMART no está averiado |
| Error de fuente | Degrada su tarjeta, no la aplicación |
| Dato obsoleto | Se marca como tal, no se muestra como fresco |

### Lo que solo se puede probar en navegador real

Y que justifica el modo navegador frente a jsdom:

- **Contraste efectivo** con `getComputedStyle`, sobre el material compuesto y en ambos temas.
- **Que el material cae a `--sdm-solid`** cuando no hay `backdrop-filter`.
- **Que el tema oscuro resuelve todas las variables**: una variable definida solo dentro de un
  bloque de media queda sin valor y el fallo es invisible en jsdom.
- **Que las animaciones desaparecen** con `prefers-reduced-motion`.
- **Que el foco es visible** y que ningún componente anula `:focus-visible`.
- **Que el texto no se recorta** a 1024 × 560, el mínimo técnico.

No se reproduce por interfaz la lógica ya cubierta por unit tests: `capacityState()` se prueba una
vez en Node, no otra vez pintando una barra.

---

## 8. Integration testing

Frontera IPC, que es la única real:

| Qué | Cómo | Riesgo que cubre |
|---|---|---|
| Esquema Zod ↔ serialización de Rust | Un caso de prueba compartido: Rust serializa, TypeScript valida | Que el contrato divergiera en silencio |
| Store ↔ eventos | Emitir una secuencia de eventos y comprobar el estado resultante | Duplicados, orden, reemplazo por identificador |
| Colector ↔ `smartctl` falso | Ejecutable de prueba que devuelve fixtures | Timeouts, códigos de salida con bits, JSON corrupto |
| Repositorios ↔ SQLite | Fichero temporal por prueba, nunca `%ProgramData%` | Migraciones, restricciones, transacciones |
| Detección de codificación | Volcados reales de `chkdsk` (CP1252) y `fsutil` (CP850) | Que una regresión devuelva texto corrupto |

**Nunca se conecta una prueba a la base de datos real ni a un disco real en modo escritura.**

---

## 9. Acceptance testing y trazabilidad

Cada criterio de aceptación se demuestra **al nivel más barato que lo demuestre**. La tabla siguiente
es la trazabilidad inicial; se amplía al implementar cada historia.

| Historia | Criterio | Nivel | Suite | Bloquea |
|---|---|---|---|---|
| US-001 | La elevación se rechaza y la app no queda a medias | E2E app real | `app` | Release |
| US-004 | Cero literales de color; ambos temas correctos | Estático + componente | `verify` + `component` | Lote |
| US-004 | Contraste AA con acento heredado | Unit + componente | `unit` + `component` | Lote |
| US-004 | 1024 × 560 sin recortes | Componente + visual | `component` + `visual` | Historia |
| US-010 | Un RAID o USB sin SMART aparece no compatible, **no averiado** | Unit + componente | `unit` + `component` | Lote |
| US-011 | La selección no se pierde al cambiar una letra de unidad | Integración | `integration` | Historia |
| US-012 | Los valores ausentes muestran «No disponible», nunca cero | Unit | `unit` | Lote |
| US-013 | No inicia dos recopilaciones iguales | Integración | `integration` | Historia |
| US-020 | Las discontinuidades se muestran como ausencia, no como cero | Componente | `component` | Lote |
| US-021 | Reiniciar no duplica eventos | Integración | `integration` | Historia |
| US-030 | Eventos equivalentes incrementan contador, no crean grupos | Unit (dominio) | `unit-rust` | Lote |
| US-031 | Reconocer **no** devuelve el disco a verde | Unit | `unit` | Lote |
| US-032 | Un crítico vigente manda sobre la pausa | Unit | `unit` | Lote |
| US-033 | Solo se notifica al cruzar umbral o recaer | Unit (dominio) | `unit-rust` | Lote |
| US-040 | Nunca sobrescribe un archivo existente | Unit (rutas) | `unit-rust` | **Release** |
| US-040 | Se detiene en el umbral térmico | Integración | `integration` | Release |
| US-050 | La exportación informa de campos omitidos | Integración | `integration` | Historia |
| US-051 | Anonimiza series, equipo y rutas | Unit + integración | `unit` | **Release** |
| US-060 | Desinstalar conserva `ProgramData` | Manual documentado | — | Release |
| US-074 | Una prueba interrumpida no queda «en curso» | Integración | `integration` | Historia |

**No se convierte cada frase en un test independiente.** Los comportamientos relacionados se agrupan.

Formato Given/When/Then admitido en la redacción, **sin introducir Cucumber**: añadiría una capa de
traducción que aquí no compensa.

---

## 10. E2E: plano de interfaz (Playwright + `mockIPC`)

Playwright contra `pnpm preview`, con el IPC de Tauri simulado mediante `@tauri-apps/api/mocks`
(`mockIPC`, `mockWindows`, `clearMocks`), que ya está disponible en la versión instalada.

### Qué cubre

Navegación entre secciones, estados de pantalla, formularios, mensajes de error, responsive,
accesibilidad y regresión visual. Todo lo que dependa de la interfaz y no del binario real.

### Smoke mínimo

- La aplicación arranca y no queda en blanco.
- El chrome aparece: barra lateral, barra de herramientas, región de contenido.
- Se navega entre las seis secciones y cada una pinta algo.
- El tema se aplica: `data-theme` presente y el material resuelto.
- La tipografía empotrada carga (no ha caído al respaldo del sistema).
- No hay excepciones de JavaScript sin controlar ni errores de página.

### Errores de consola

Se vigilan `console.error`, excepciones de página y promesas rechazadas. **No todo mensaje es un
fallo**: se distingue error de aplicación de aviso conocido. La lista de excepciones permitidas debe
ser mínima y estar documentada junto a la prueba, nunca dispersa.

### Entorno

Desarrollo local puede usar `pnpm dev` por rapidez. **CI y release usan `pnpm preview`** sobre el
build real: es donde aparecen los problemas que el servidor de desarrollo esconde.

---

## 11. E2E: plano de aplicación real (`tauri-driver`)

WebdriverIO con `tauri-driver` contra el ejecutable empaquetado.

### Efecto colateral sobre `cargo test`

Al aplicar el manifiesto, `cargo test` empezó a fallar con `ERROR_SXS_CANT_GEN_ACTCTX`. La causa:
Cargo compila también un arnés de pruebas para el binario, ese ejecutable hereda el
`requireAdministrator` y no puede arrancarse desde un terminal sin elevar.

Se resuelve declarando `test = false` en el target `[[bin]]`: `main.rs` es una línea que delega en
la biblioteca, no tiene tests y no los va a tener. Todo lo testeable vive en la biblioteca, que sí
se prueba y no lleva manifiesto.

Queda escrito porque es de esas cosas que se olvidan y cuestan media hora la segunda vez.

### Requiere terminal elevado

La aplicación pide privilegios de administrador (ADR-004, manifiesto en `src-tauri/windows/`). Un
programa **no puede pulsar el botón de UAC**, así que el driver tiene que ir ya elevado para que el
hijo herede la elevación y el diálogo no aparezca.

Consecuencia práctica: **esta suite, y solo esta, se ejecuta desde un terminal de administrador**.
Las demás no lo necesitan. En los agentes Windows de GitHub el usuario ya es administrador, así que
allí funciona sin nada especial.

Se descartó compilar una variante sin elevación para las pruebas: probaría un binario distinto del
que se entrega, y precisamente en la parte que gobierna el acceso a los discos.

### El driver se sincroniza solo

`msedgedriver` debe coincidir con la versión del WebView2 instalado, y ese runtime **se actualiza
solo cada pocas semanas** (es el precio de haber elegido Evergreen en el ADR-020). Sin nada que lo
gestione, la suite fallaría periódicamente con un error de protocolo que no tiene relación con el
código.

`scripts/ensure-webdriver.mjs` lo resuelve antes de cada ejecución: lee la versión del registro
—igual que ya hace `verify-assets` con los hashes—, descarga el driver que corresponde y lo cachea.
Si no existe un driver para esa versión exacta, **avisa con un mensaje claro** en vez de dejar que
la suite falle treinta segundos después con un error críptico.

### Qué demuestra, y solo esto

- El binario arranca y abre su ventana.
- La interfaz pinta dentro del WebView2 real.
- El IPC responde: un comando devuelve datos de verdad.
- La ventana respeta su tamaño mínimo.
- El registro escribe en `%ProgramData%`.
- La aplicación cierra limpiamente y suelta sus recursos.

Es una suite **deliberadamente pequeña**. No se replican aquí flujos que el plano de interfaz ya
cubre más rápido y con mejor diagnóstico.

### Frecuencia

Rama principal y antes de release. **Nunca en cada pull request**: cuesta minutos y requiere una
compilación completa.

---

## 12. Contrato de testabilidad y selectores

Orden de preferencia, en componentes y en E2E:

1. Rol accesible (`getByRole`)
2. Nombre accesible / etiqueta (`getByLabel`)
3. Texto visible estable (`getByText`)
4. Estado semántico (`aria-current`, `aria-disabled`)
5. `data-testid` **solo si lo anterior no basta**

**Prohibido**: clases CSS, clases de Tailwind, posiciones, `nth()`, estructura interna del DOM,
identificadores generados.

Esto no es una preferencia de estilo: si un elemento no se puede localizar por su rol o su nombre
accesible, es que **no es accesible**, y eso incumple el principio VII antes que ninguna prueba.

Identificadores estables admitidos, ligados al dominio:

```text
app-root  main-layout  primary-action  disk-card
empty-state  error-state  loading-state  stale-badge
```

---

## 13. Esperas y sincronización

Se usa la sincronización automática de Playwright: locators, `expect` con reintento, esperas por
estado observable.

**Prohibido**: `waitForTimeout`, `sleep`, esperas arbitrarias, bucles manuales. Subir un tiempo de
espera para tapar una condición de carrera no la arregla, la esconde.

Si una prueba necesita cada vez más tiempo, es un defecto: se investiga la causa.

---

## 14. Datos, dobles y aislamiento

Cada prueba debe poder ejecutarse sola, en cualquier orden, repetidamente y en paralelo.

| Dependencia | Doble | Por qué |
|---|---|---|
| `smartctl` | Ejecutable falso que devuelve fixtures | Probar timeouts, códigos con bits y JSON corrupto sin hardware |
| Registro de eventos de Windows | Volcados XML reales anonimizados | Reproducir ráfagas correlacionadas (`alert-rules.md` §3.5) |
| SQLite | Fichero temporal por prueba | Nunca `%ProgramData%` |
| Reloj | Reloj inyectado / *fake timers* | Histéresis, retención y expiraciones |
| Acento de Windows | Valor fijo | El acento del equipo de CI no puede decidir si una prueba pasa |
| IPC de Tauri | `mockIPC` | Estados de error y casos límite imposibles de provocar de verdad |

**Los fixtures se anonimizan al capturarlos, nunca al usarlos.** Ninguno contiene números de serie
ni nombres de equipo reales (principio IX).

Se declara explícitamente en cada suite si usa IPC simulado, backend real o base de datos temporal.
No se mezclan estrategias dentro de un mismo fichero.

---

## 15. Responsive

Tres viewports, ni uno más, tomados de la medición de `open-questions.md` §L:

| Viewport | Por qué ese |
|---|---|
| 1024 × 560 | Mínimo técnico. Barra lateral colapsada |
| 1280 × 720 | Objetivo de diseño |
| 1920 × 1032 | 1920 × 1080 al 100 %, el caso más común |

Se comprueba: navegación utilizable, acciones alcanzables, sin recortes silenciosos, sin scroll
horizontal en el cuerpo, y que la rejilla reorganiza en vez de comprimir.

No se prueban decenas de resoluciones arbitrarias: no hay más breakpoints funcionales que esos.

---

## 16. Accesibilidad

Automatizable, y se automatiza: `axe-core` sobre cada pantalla, en ambos temas.

Cubre roles, nombres accesibles, asociación de etiquetas, estados de los controles y contraste.

**Comprobaciones propias que `axe` no hace** y que este proyecto sí exige:

- El contraste se mide sobre el **material compuesto**, no sobre un fondo plano.
- El acento heredado se verifica con un acento claro deliberado (`#ffb900`).
- El foco no queda tapado por el chrome translúcido (WCAG 2.4.11).
- Toda gráfica tiene lectura textual equivalente cerca, no solo `aria-label`.

Una prueba automática **no es una auditoría**. Antes de la 1.0 se hace una pasada manual con lector
de pantalla; queda como tarea, no como comprobación automatizable.

---

## 17. Regresión visual

Doce capturas, solo en Chromium. Comparar entre motores daría falsos positivos por renderizado de
fuentes sin cubrir ningún riesgo real: solo se distribuye WebView2.

| Captura | Temas |
|---|---|
| Panel general con cuatro discos | claro, oscuro |
| Detalle de disco | claro, oscuro |
| Alertas con grupo seleccionado | claro, oscuro |
| Pruebas y diagnóstico | claro, oscuro |
| Estado vacío | claro |
| Estado de error de fuente | claro |
| Disco no compatible (gris, no rojo) | claro |
| Dato obsoleto | claro |

**Se controla antes de capturar**: datos fijos, reloj congelado, locale, viewport, tipografía local,
`prefers-reduced-motion` activo y animaciones desactivadas. Se enmascaran únicamente las regiones
legítimamente dinámicas, como la marca de antigüedad.

Una captura **no sustituye una prueba funcional** y no se actualiza porque falle: se actualiza cuando
el cambio es intencionado, se ha revisado y va en el mismo commit que lo provoca.

### La línea base se genera en CI

La misma pantalla no se dibuja igual en dos equipos: el suavizado de fuentes depende de la
configuración de ClearType, y el desenfoque del material translúcido lo calcula la tarjeta gráfica
—que los agentes de CI no tienen, así que Chromium lo hace por software—. Justo el material, que es
lo que más interesa vigilar, es lo que peor se reproduce entre máquinas.

Por eso **la comparación que decide es la de CI**:

| Dónde | Qué hace |
|---|---|
| Local | Genera las capturas para poder mirarlas. **No compara** |
| CI | Compara contra la línea base versionada en el repositorio |

Aprobar un cambio visual intencionado: se sube, CI falla y adjunta la captura nueva como artefacto,
se revisa, y la línea base actualizada va **en el mismo commit** que el cambio que la provoca.

Se descartó generar la base en local con tolerancia amplia: la holgura necesaria para absorber la
diferencia de desenfoque dejaría pasar regresiones reales, y una prueba así da tranquilidad sin dar
cobertura.

Frecuencia: rama principal. Es donde una regresión de tokens se detecta antes de llegar a release.

---

## 18. Property-based testing

Se aplica donde hay propiedades generales que valen para todo el dominio de entrada, no como
sustituto de casos concretos.

| Candidato | Propiedad |
|---|---|
| `accessibleAccent` | Para **todo** color, el resultado alcanza 4.5:1 |
| `accentOnSurface` | Para todo color y ambos temas, alcanza 4.5:1 |
| `formatBytes` | Monótona: más bytes nunca produce una unidad menor |
| `capacityState` | Monótona: menos espacio libre nunca mejora el estado |
| Deduplicación de alertas | N evaluaciones equivalentes producen 1 grupo y N ocurrencias |
| Detección de codificación | Todo volcado válido decodifica sin excepción |

Semillas reproducibles y registro del caso mínimo que falle. **No se aplica a componentes visuales.**

Nota: `accessibleAccent` ya tiene hoy un barrido exhaustivo escrito a mano; sustituirlo por
property-based es opcional y de bajo valor añadido.

---

## 19. Mutation testing

Mide si los tests detectan errores, no si el código funciona. **Fuera del bucle de desarrollo.**

| Ámbito | Herramienta | Qué se muta |
|---|---|---|
| TypeScript | Stryker | `design/health.ts`, `design/format.ts`, `design/accent.ts` |
| Rust | `cargo-mutants` | `domain/`, `alerts/`, parsers de `smartctl` |

**Se excluye**: componentes visuales, CSS, configuración, envoltorios finos, registro, código
generado, y todo `src/routes/`.

Prioridad en el motor de alertas: si una regla tiene un test que pasa con la lógica invertida, el
producto **miente sobre la salud de un disco**, que es el fallo más caro que puede tener.

Ejecución: programada semanal y antes de release. **Nunca en un pull request**, nunca sobre suites
inestables, nunca durante una refactorización.

### El informe tiene que ser accionable, no exhaustivo

Un motor de alertas de tamaño medio produce del orden de 400 a 600 mutantes, y cada uno obliga a
recompilar: entre dos y tres horas de ejecución. El tiempo no es el problema —se ejecuta de
madrugada y nadie espera—; el problema es que un informe con sesenta supervivientes **no lo revisa
nadie**, y a la tercera semana deja de abrirse. Una herramienta que nadie mira no cubre ningún
riesgo: solo da la sensación de que sí.

Por eso se versiona una línea base en `docs/mutation-baseline.md` con los supervivientes conocidos y
la razón por la que se aceptan: mutación equivalente, comportamiento no observable, código muerto.
El informe semanal destaca únicamente **los supervivientes nuevos**.

| | |
|---|---|
| Informe útil | «3 supervivientes nuevos» — se revisan los tres |
| Informe inútil | «60 supervivientes» — no se revisa ninguno |

**Un superviviente nuevo sin justificar bloquea la release.** Uno ya justificado en la línea base, no.

Se empieza con un piloto sobre `health.ts` para fijar la primera línea base (mutantes, muertos,
supervivientes, sin cobertura, timeouts, duración). **No se fija el 100 % como objetivo**: un
superviviente puede ser una mutación equivalente, no un hueco.

---

## 20. Cobertura

Umbrales del principio VIII, ya configurados en `vitest.config.ts` y en CI:

| Ámbito | Mínimo |
|---|---|
| `domain/`, `alerts/` (Rust) | 90 % |
| Resto de `src-tauri/` | 80 % |
| `src/lib/` sin componentes | 70 % |
| Componentes y rutas | Sin umbral: se exige **cobertura de estados** |

Atención especial a las ramas de validación, error, límites y reglas.

**La cobertura es una señal, no un objetivo.** No se escriben pruebas sin valor para subirla, y una
cobertura alta no sustituye a las pruebas de aceptación, al E2E ni al mutation testing.

---

## 21. Determinismo

Prohibido depender de `Date.now()`, `new Date()`, `Math.random()`, temporizadores reales o del
entorno del equipo cuando afecte al resultado.

Se prueban explícitamente: cambio de día, expiraciones, ventanas de histéresis y cambios de hora,
que en este producto **importan de verdad** — la retención, el cooldown de notificaciones y la
correlación de eventos dependen de ello.

---

## 22. Suites escalonadas

### Nivel 0 — Edición

Comprobación de tipos del fichero tocado. Una prueba concreta si se está diagnosticando. Nada más.

### Nivel 1 — Ultrarrápida (segundos)

```sh
pnpm test -- <fichero>
cargo test <modulo>
```

### Nivel 2 — Lote (cierre de checkpoint, < 1 min)

```sh
pnpm check && pnpm lint && pnpm verify
pnpm test
cargo test
```

Más los tests de componente del lote si toca interfaz. **Es la suite del checkpoint.**

### Nivel 3 — Historia / Pull Request (minutos)

Todo lo anterior más: componentes completos, integración, aceptación, E2E de interfaz, accesibilidad
y cobertura con umbrales.

### Nivel 4 — Rama principal

Añade: regresión visual, E2E de aplicación real, cobertura de Rust.

### Nivel 5 — Programada y release

Añade: mutation testing, property-based extendido, verificación del instalador y la lista manual de
accesibilidad.

**Nunca se ejecuta el nivel 4 o 5 tras un cambio pequeño.**

---

## 23. Selección de pruebas afectadas

Se ejecuta primero lo más cercano al cambio. Se **amplía a la suite completa** cuando se toca:

`tokens.css` · `tailwind.config.cjs` · `AppShell`, `Sidebar` o `Toolbar` · `src/lib/api/` ·
`src/lib/design/health.ts` · `schemas.ts` · el motor de alertas · migraciones ·
`svelte.config.js`, `vite.config.ts` o `tauri.conf.json` · cualquier dependencia principal.

Son los puntos donde un cambio local tiene efecto global.

---

## 24. Estructura y comandos

Estructura propuesta, respetando la que ya existe:

Dos configuraciones de Vitest, no una: los entornos son distintos y los tiempos también.
`pnpm test` debe seguir siendo la suite rápida que se teclea mientras se programa.

| Configuración | Entorno | Qué incluye | Duración |
|---|---|---|---|
| `vitest.config.ts` | Node | `src/lib/**/*.test.ts` | ~22 s |
| `vitest.browser.config.ts` | Chromium | `src/lib/**/*.svelte.test.ts` | más lenta |

Stryker apunta a la de Node y por tanto nunca abre un navegador, que era el riesgo real: mutar
código exige lanzar la suite cientos de veces.

```text
src/lib/**/*.test.ts        unit, junto al código (ya en uso)
src/lib/**/*.svelte.test.ts componentes, en navegador
tests/integration/          frontera IPC, SQLite, colectores
tests/fixtures/             volcados anonimizados de smartctl, eventos, chkdsk
e2e/ui/                     Playwright + mockIPC
e2e/ui/visual.spec.ts       regresión visual
e2e/ui/a11y.spec.ts         accesibilidad
e2e/app/                    WebdriverIO + tauri-driver
src-tauri/src/**            tests en módulo `#[cfg(test)]` (ya en uso)
src-tauri/tests/            integración de Rust
```

Scripts a añadir, sin romper los existentes:

```text
test:unit          las pruebas de Node actuales
test:component     Vitest Browser Mode
test:integration   frontera IPC y persistencia
test:e2e           Playwright, plano de interfaz
test:e2e:smoke     solo el smoke
test:e2e:ui        modo interactivo de Playwright
test:e2e:visual    regresión visual
test:e2e:app       WebdriverIO contra el ejecutable
test:a11y          accesibilidad
test:mutation      Stryker sobre el dominio
test:lote          nivel 2 completo
```

**Los comandos reales se documentan al crearlos**, no antes: no se prometen scripts que no existan.

---

## 25. CI

Ampliación del workflow actual, que ya tiene trece puertas:

| Disparador | Añade |
|---|---|
| Pull request | Componentes, integración, aceptación, E2E de interfaz, accesibilidad |
| Rama principal | Regresión visual, E2E de aplicación real, cobertura de Rust |
| Programado semanal | Mutation testing, property-based extendido |
| Release | Verificación del instalador y del artefacto que se distribuye |

Artefactos de diagnóstico: captura y traza **de los fallos**, en CI. Vídeo solo donde aporte. No se
guardan artefactos pesados de pruebas correctas.

---

## 26. Pruebas inestables

Una prueba inestable es un **defecto**, no una molestia. Se investiga: estado compartido, condición
de carrera, datos compartidos, orden, temporizadores, animaciones.

No se arregla subiendo reintentos ni tiempos de espera. Un reintento vale como mitigación temporal
**documentada**, nunca como solución. Ninguna prueba crítica se ignora de forma permanente sin
justificación en `docs/known-issues.md`.

---

## 27. Integración con Spec Kit

**Especificación** (`spec.md`): cada historia aporta reglas verificables, criterios de aceptación,
camino correcto, errores, límites, estados y riesgos.

**Plan** (`plan.md`): decide niveles de test, dobles, datos, suites, checkpoints y presupuesto.

**Tareas** (`tasks.md`): agrupadas por lote. **No** se genera una tarea por test ni la secuencia
«implementar función → crear test → ejecutar todo». El testing va dentro del checkpoint del lote.

---

## 28. Definition of Checkpoint

Un lote cierra cuando:

- la implementación prevista está completa;
- `pnpm check`, `pnpm lint` y `pnpm verify` pasan;
- las pruebas obligatorias del lote existen y pasan;
- la aceptación relacionada pasa;
- los fixtures y dobles necesarios existen;
- no queda deuda de test del lote.

**No hace falta** ejecutar en cada checkpoint: E2E completo, visual, aplicación real, mutation
testing ni suites no afectadas.

## 29. Definition of Done

Una historia termina cuando, además de sus lotes:

- los criterios de aceptación son trazables a pruebas concretas;
- errores, límites y estados relevantes están cubiertos;
- los estados vacío, cargando, no compatible, error y obsoleto están probados;
- la suite de historia pasa, incluido el E2E de interfaz cuando aplique;
- los tres viewports están verificados;
- la accesibilidad automática pasa en ambos temas;
- las capturas visuales coinciden o se actualizaron intencionadamente;
- no hay esperas arbitrarias ni pruebas ignoradas sin justificar;
- la cobertura no baja sin razón escrita;
- los comandos están documentados y las pruebas integradas en CI.

**Mutation testing solo es obligatorio** si la historia toca el dominio crítico y el plan lo pide.

---

## 30. Tareas para `tasks.md`

Se adaptan; no se crean todas de golpe.

```text
T-TEST-001  Documentar comandos reales y medir la línea base de cada suite
T-TEST-002  Definir la selección de pruebas afectadas por tipo de cambio

T-UNIT-001  Fixtures anonimizados: smartctl (ATA, NVMe, USB, RAID, VM)
T-UNIT-002  Fixtures de eventos de Windows y volcados de chkdsk/fsutil
T-UNIT-003  Reloj inyectable y control de aleatoriedad

T-COMP-001  Configurar Vitest Browser Mode; retirar testing-library si es redundante
T-COMP-002  Probar los cinco estados obligatorios del catálogo
T-COMP-003  Verificar contraste y material en ambos temas con getComputedStyle

T-INT-001   smartctl falso: timeouts, códigos con bits, JSON corrupto
T-INT-002   SQLite temporal: migraciones, restricciones, retención
T-INT-003   Contrato Zod ↔ serde con caso de prueba compartido

T-ACC-001   Trazabilidad criterio → prueba de las historias P0

T-PLAY-001  Configurar Playwright contra preview con mockIPC
T-PLAY-002  Instalar msedgedriver y configurar tauri-driver
T-PLAY-003  Smoke de interfaz
T-PLAY-004  Smoke de aplicación real
T-PLAY-005  Captura de errores de consola con allowlist documentada
T-PLAY-006  E2E de los flujos críticos con navegador

T-A11Y-001  axe-core en ambos temas
T-A11Y-002  Verificaciones propias: contraste sobre material, foco no tapado
T-VIS-001   Doce capturas controladas en Chromium

T-QUAL-001  Cobertura de Rust en CI
T-QUAL-002  Detección de pruebas inestables

T-MUT-001   Piloto de Stryker sobre health.ts y línea base
T-MUT-002   cargo-mutants sobre el dominio cuando exista
```

---

## 31. Preguntas abiertas

Las cinco que planteaba la primera versión de este documento están **resueltas** (2026-09-04):

| # | Era | Resolución |
|---|---|---|
| 1 | ¿Qué versión de `msedgedriver` hace falta? | Ninguna fija: `scripts/ensure-webdriver.mjs` la deduce del registro y la descarga (§11) |
| 2 | ¿Puede `tauri-driver` con una app elevada? | Sí, si el driver va elevado. Esa suite se ejecuta desde terminal de administrador (§11) |
| 3 | ¿Stryker convive con Browser Mode? | No hace falta: dos configuraciones separadas, Stryker usa la de Node (§24) |
| 4 | ¿CI dibuja igual que un equipo local? | No. La línea base se genera y compara en CI; en local solo se miran (§17) |
| 5 | ¿Cuánto tarda `cargo-mutants`? | Horas, y da igual: lo que se acota es el informe, no el tiempo (§19) |

De paso, la número 2 destapó que **la elevación del ADR-004 no estaba implementada**. Se corrigió con
un manifiesto propio en `src-tauri/windows/app.manifest`, que además declara consciencia de DPI por
monitor, necesaria para los escalados de 125 %, 150 % y 200 % que exige `AGENTS.md` §4.

### Lo que sigue sin respuesta

| # | Pregunta | Cuándo se sabrá |
|---|---|---|
| 1 | ¿Cuánto tarda de verdad la suite de componentes en navegador? | Al implementar `T-COMP-001` |
| 2 | ¿Las doce capturas visuales son estables entre ejecuciones del mismo agente de CI? | Al implementar `T-VIS-001` |

Ninguna bloquea. Se miden cuando toque.
