# Plan de implementación: SmartDisk Monitor 1.0

**Rama**: `main` (no hay extensión de git registrada; el directorio de la funcionalidad no depende del nombre de rama)
**Fecha**: 2026-09-04
**Especificación**: [spec.md](spec.md)

**Entrada**: especificación de `specs/001-monitor-discos-windows/spec.md`, aclarada el 2026-09-04.

## Resumen

Convertir el esqueleto actual —chrome de la aplicación, catálogo de componentes, frontera IPC tipada
y 31 comandos que devuelven `not_implemented`— en un producto que **mide de verdad**: recopilar datos
de los discos, persistirlos, evaluar reglas de alerta y presentarlos sin mentir sobre lo que no sabe.

El grueso del trabajo está en el backend, y sigue una dirección clara: **cada comando deja de
devolver un marcador cuando su fuente de datos y su almacén existen**. La interfaz ya está montada
contra el contrato, así que cada comando que se completa enciende una pantalla que ya espera datos.

El orden lo dicta el flujo del dato, no las pantallas: sin persistencia no hay historial, sin
historial no hay reglas, y sin reglas no hay alertas. Las historias 1 y 2 de la especificación
—inventario con estado, y alertas agrupadas— son las que hacen producto; el resto se apoya en ellas.

## Contexto técnico

Todo está fijado por la constitución (§Pila y versiones). **Ninguna casilla necesita aclaración**:
este proyecto no elige tecnología en fase de plan, la tiene decidida y versionada.

**Lenguaje/versión**: Rust 1.94.0 (edición 2021, MSRV 1.77) · TypeScript 5.9.3 `strict` · Svelte 5.57.0 con runes

**Dependencias principales**: Tauri 2.11.5 · SvelteKit 2.70.3 con `adapter-static` y SSR desactivado ·
Tailwind 3.4.19 (solo utilidades mapeadas desde tokens) · Zod 4.5.4 en toda frontera · `tracing` en
Rust y envoltorio propio en TypeScript · `tauri-plugin-single-instance` 2.4.4

**Almacenamiento**: SQLite mediante `rusqlite` con `bundled`, en `%ProgramData%\SmartDisk Monitor\`,
con WAL, transacciones breves y migraciones numeradas que nunca se editan una vez publicadas

**Pruebas**: `cargo test` · Vitest 5 en dos configuraciones (Node y navegador) · Playwright 1.62.1
con IPC propio · `@axe-core/playwright` en las seis pantallas por ambos temas

**Plataforma objetivo**: Windows 10 (1809+), 11 y Server 2016–2025, x64, con Experiencia de
escritorio y WebView2 Runtime. Ejecución elevada obligatoria

**Tipo de proyecto**: aplicación de escritorio, local, de usuario único y sin red

**Objetivos de rendimiento**: la interfaz nunca deja de responder más de 50 ms seguidos; toda acción
produce respuesta visible en menos de 100 ms (SC-007, aclaración del 2026-09-04)

**Restricciones**: cero peticiones de red, ni en instalación ni en uso · ventana mínima 1024 × 560,
correcta al 125 %, 150 % y 200 % de escalado · sin variables de entorno en la aplicación · el backend
empuja por eventos, la interfaz no sondea

**Escala/alcance**: 20 discos monitorizados y 5.000 eventos sin salirse de los umbrales de SC-007 ·
6 pantallas ya montadas más el asistente inicial · 31 comandos en la lista cerrada del contrato

## Comprobación de la constitución

*PUERTA: debe pasar antes de la fase 0 y volver a comprobarse tras la fase 1.*

| Principio | Cómo lo satisface este plan | Estado |
|---|---|---|
| I. Veracidad del dato | Ningún comando devuelve datos inventados: el esqueleto responde `not_implemented` hasta tener fuente real. Dato ausente ⇒ «No disponible». Sin lecturas frescas ⇒ `unknown`, nunca `ok` | ✅ |
| II. Prioridades ante conflicto | El orden de trabajo antepone no mentir a mostrar algo: primero procedencia y frescura, después presentación | ✅ |
| III. Pila fija | El plan no introduce ninguna tecnología. Toda dependencia nueva exigiría enmienda y ADR | ✅ |
| IV. Dominio separado de presentación | `domain/` sin dependencias de Tauri, Windows ni SQLite. Los colectores se aíslan tras `platform/` para poder simularlos | ✅ |
| V. Persistencia local, íntegra y trazable | SQLite con WAL y migraciones numeradas; copia antes de migrar; procedencia por métrica | ✅ |
| VI. Sistema de diseño vinculante | Las pantallas ya usan el catálogo cerrado y los tokens. No entra ningún componente nuevo salvo los cuatro ya autorizados | ✅ |
| VII. Accesibilidad AA | Las suites de `axe` ya cubren las seis pantallas por ambos temas y se amplían con cada pantalla nueva | ✅ |
| VIII. Testeabilidad | Test-first donde el fallo es silencioso: reglas de alerta, retención, formateadores y validación de fronteras | ✅ |
| IX. Seguridad y privacidad | Sin shell genérica, sin `fs` abierto; la interfaz no construye órdenes; el paquete de diagnóstico va anonimizado por defecto | ✅ |
| X. Errores comprensibles | Todo fallo cruza como `AppError` con código estable, clave i18n y detalle conservado | ✅ |
| XI. Validación de fronteras | Zod en `invoke`, `listen` y lectura de ficheros; cada esquema con su prueba de rechazo | ✅ |
| XII. Sin variables de entorno | Ninguna. La configuración vive en la tabla `settings` | ✅ |
| XIII. Tipos antes que nada | `pnpm check` con cero errores y cero avisos, antes de las pruebas | ✅ |
| XIV. SvelteKit idiomático | Estado inicial por `load` en `+page.ts`; navegación con enlaces; `$derived` antes que `$effect`; sin sondeo | ✅ |
| XV. Registro sin ruido ni datos personales | Una sola API en cada lado; el modo detallado que introduce FR-029a se suma a ella, no la duplica | ✅ |

**Resultado: la puerta pasa, sin violaciones que justificar.** La sección de seguimiento de
complejidad queda vacía a propósito.

Dos observaciones que el plan asume y conviene tener presentes:

1. **La puerta de DTO generados se activa en esta fase.** La constitución la declara *pendiente*
   «hasta que el primer comando devuelva un DTO real», y eso ocurre en el primer entregable de este
   plan (el inventario). A partir de ahí, los tipos de `src/lib/api/types.ts` dejan de mantenerse a
   mano y pasan a generarse. Activarla es parte del trabajo, no un efecto colateral.
2. **Los umbrales de espacio libre son valores de partida, no medidos.** La especificación adopta
   1 GB y 256 MB (Supuestos). El principio de la fase 0 —«los riesgos se miden, no se estiman»—
   obliga a registrarlos en `docs/open-questions.md` antes de programarlos.

## Estructura del proyecto

### Documentación de esta funcionalidad

```text
specs/001-monitor-discos-windows/
├── spec.md              # Especificación, aclarada
├── plan.md              # Este fichero
├── research.md          # Fase 0: riesgos a medir y decisiones a registrar
├── data-model.md        # Fase 1: remisión al modelo canónico + delta de esta funcionalidad
├── quickstart.md        # Fase 1: cómo validar de extremo a extremo
├── contracts/           # Fase 1: remisión al contrato canónico + estado comando a comando
│   └── README.md
└── tasks.md             # Fase 2 (`/speckit-tasks`), no lo crea este comando
```

> **Los artefactos de la fase 1 no duplican la documentación normativa.** El modelo de datos vive en
> `docs/data-model.md` y el contrato en `docs/ui-contract.md`; los dos son normativos y ya existen.
> Copiarlos aquí crearía dos versiones que divergirían en silencio, que es exactamente el fallo que
> ADR-029 acaba de cerrar en este repositorio, y con el mismo mecanismo: una copia «de referencia»
> que nadie actualiza. Los ficheros de `specs/` **remiten** al canónico y aportan solo lo que es
> propio de esta funcionalidad: el delta de las aclaraciones, la trazabilidad y el estado de avance.

### Código fuente (raíz del repositorio)

Estructura ya establecida en `docs/engineering-conventions.md`. En negrita, lo que este plan crea:

```text
src/
├── routes/                     6 pantallas montadas + asistente inicial
├── lib/
│   ├── components/             catálogo cerrado (25 componentes) + los 4 autorizados pendientes
│   ├── design/                 tipos, formato, salud, tema, acento
│   ├── i18n/                   diccionarios es/en
│   ├── api/                    envoltorios tipados de invoke y listen; ninguna pantalla llama a invoke
│   └── stores/                 estado de aplicación en runes
└── design-system/              tokens.css, tokens.json, fonts/

src-tauri/
├── src/
│   ├── commands/               31 comandos: funciones finas que validan y delegan
│   ├── domain/                 **reglas de negocio; sin Tauri, sin Windows, sin SQLite**
│   ├── collectors/             **smartctl, windows_storage, perf_counters, event_log**
│   ├── alerts/                 **motor de reglas (docs/alert-rules.md)**
│   ├── tests/                  **benchmark, chkdsk, autotest**
│   ├── persistence/            **repositorios, migraciones, retención**
│   ├── reporting/              **exportaciones y ZIP anonimizado**
│   ├── platform/               envolturas de API de Windows, aisladas para simularlas
│   ├── error.rs                AppError
│   └── logging.rs              registro (§XV)
├── migrations/                 **SQL numerado, nunca editado una vez publicado**
├── capabilities/               política Tauri de mínimo privilegio
└── windows/app.manifest        requireAdministrator + PerMonitorV2
```

**Decisión de estructura**: se conserva la existente sin cambios. Los seis directorios en negrita
están declarados en las convenciones pero aún no creados; este plan los llena. La regla de
dependencias es la que hace testeable el motor de alertas: **`domain/` no conoce Tauri, ni Windows,
ni SQLite** —recibe datos y devuelve decisiones—, y **ninguna pantalla llama a `invoke`
directamente**, todo pasa por `src/lib/api/`.

## Orden de construcción

No es un calendario, es una cadena de dependencias. Cada eslabón deja el producto en un estado
utilizable y verificable.

| # | Eslabón | Desbloquea | Historias |
|---|---|---|---|
| 1 | **Persistencia**: migraciones, repositorios, retención, copia previa a migrar | todo lo demás | — |
| 2 | **Inventario**: colector de almacenamiento de Windows, identidad por serie o huella, altas y bajas | primer DTO real ⇒ **activa la puerta de DTO generados** | 1 |
| 3 | **Salud**: colector `smartctl` con su cascada de `-d`, procedencia y frescura por métrica | estado real por disco | 1 |
| 4 | **Métricas y series**: contadores de rendimiento, capacidad, muestreo y compactación | gráficas con huecos reales | 3 |
| 5 | **Motor de alertas**: reglas, histéresis, agrupación, ciclo de vida, bandeja | avisos sin estar mirando | 2 |
| 6 | **Eventos de Windows**: lectura con marcador persistente, correlación con confianza declarada | evidencia del sistema | 4 |
| 7 | **Pruebas**: benchmark, `chkdsk /scan`, autotest corto, parada por temperatura | diagnóstico bajo demanda | 5 |
| 8 | **Informes y diagnóstico**: exportación y ZIP anonimizado | compartir un incidente | 6 |
| 9 | **Ajustes y ciclo de vida**: frecuencias, umbrales, retención, modo detallado, borrado, cierre | producto configurable | 7 |
| 10 | **Empaquetado**: instalador sin conexión, WebView2, desinstalación, «Acerca de» | entregable | 8 |

Los eslabones 1 a 3 son el producto mínimo: responden «¿qué discos tengo y están bien?». El 5 lo
convierte en un monitor de verdad.

## Riesgos que este plan asume

Los tres que `docs/open-questions.md` mantiene abiertos, más el que introdujeron las aclaraciones.
Se **miden**, no se estiman (constitución, flujo de desarrollo §6). Detalle en [research.md](research.md).

| Riesgo | Se mide en | Alternativa decidida |
|---|---|---|
| Notificaciones desde un proceso elevado (I.2) | eslabón 5, al empaquetar | ventana propia anclada sobre la bandeja |
| Alcance real de `smartctl` tras RAID y puentes USB (I.5) | eslabón 3, con hardware | documentar la limitación y declarar «no compatible» |
| Interfaz con 20 discos y 5.000 eventos (I.7) | eslabón 6, con la lista virtualizada | recortar densidad del panel o paginar |
| Umbrales de espacio libre, 1 GB / 256 MB | eslabón 1, antes de programar la retención | ajustar el valor con la medición |

## Seguimiento de complejidad

Sin violaciones de la constitución que justificar. Esta sección queda vacía a propósito: rellenarla
sin motivo diluiría su función, que es hacer visible una desviación cuando de verdad exista.
