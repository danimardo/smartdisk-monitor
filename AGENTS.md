<!--
  MANTENIMIENTO (nota para personas, no entra en el contexto del agente).

  Este es el núcleo: se carga en TODAS las sesiones. Objetivo: menos de 200 líneas.

  Antes de añadir algo aquí, pregúntate:
    - ¿Hace falta en cualquier tarea?          -> sí: aquí
    - ¿Solo al tocar cierto código?            -> .claude/rules/<tema>.md con `paths:`
    - ¿Es un procedimiento de varios pasos?    -> .claude/skills/<nombre>/SKILL.md
    - ¿Es estado o historia del producto?      -> docs/ o specs/
    - ¿Puede deducirlo leyendo el repositorio? -> no lo escribas

  Los `@imports` NO ahorran contexto: se expanden al arrancar. Lo que ahorra es `paths:` y skills.
-->

# AGENTS.md

Guía operativa para agentes de IA en este repositorio. **Fuente canónica**: este fichero.
`CLAUDE.md` lo importa y añade solo lo específico de su herramienta.

> No confundir con `Design-system/AGENTS.md`, que es otra cosa: las reglas vinculantes de
> interfaz. Se carga sola al tocar componentes (véase `.claude/rules/interfaz.md`).

## Idioma

**Toda comunicación con el usuario en español**: respuestas, planes, resúmenes, documentación
generada, comentarios de diseño, mensajes de progreso y checklists.

El código y sus comentarios siguen las convenciones del proyecto, que también son en español.

## Comandos

Gestor de paquetes: **pnpm**. Los comandos de Rust se ejecutan desde `src-tauri/`.

| Para | Comando |
|---|---|
| Aplicación en desarrollo | `pnpm app:dev` (**muestra UAC**: la app va elevada) |
| Compilar frontend | `pnpm build` |
| Instalador | `pnpm app:build` |
| Tipos y accesibilidad | `pnpm check` |
| Formato y análisis estático | `pnpm lint` |
| Verificadores propios | `pnpm verify` |
| Pruebas de lógica (Node) | `pnpm test` · con cobertura: `pnpm test:coverage` |
| Pruebas de componente (Chromium) | `pnpm test:component` |
| Pruebas de interfaz (Playwright) | `pnpm test:e2e` · solo humo: `pnpm test:e2e:smoke` |
| Accesibilidad | `pnpm test:a11y` |
| Pruebas de Rust | `cargo test` |
| Análisis estático de Rust | `cargo clippy --all-targets -- -D warnings` |
| Regenerar el consolidado | `pnpm docs:build` |

`pnpm verify` es el que impide que el sistema de diseño se erosione: comprueba hashes de recursos,
valores visuales literales, diccionarios sincronizados y validación de fronteras.

## Fuentes de verdad y orden de autoridad

1. **`.specify/memory/constitution.md`** — 15 principios innegociables. Prevalece sobre todo lo
   demás; si una fase la contradice, la fase está mal.
2. **`Design-system/AGENTS.md`** — reglas de interfaz, vinculantes.
3. **`docs/alert-rules.md`** y **`docs/ui-contract.md`** — normativos frente a cualquier
   descripción informal de alertas o de comandos.
4. **`docs/open-questions.md`** — decisiones adoptadas y mediciones. Recoge correcciones
   posteriores, varias nacidas de medir sobre un Windows real.
5. **`docs/decisions.md`** — registro de decisiones técnicas (ADR).
6. Resto de `docs/` — especificación, arquitectura, modelo de datos, historias, testing.
7. `specs/<feature>/` cuando exista — spec, plan y tasks de la feature en curso.
8. Código existente.

El código se lee siempre, pero **no se asume que represente la intención correcta** si contradice
un documento normativo. Ante contradicción: parar, explicar el conflicto y pedir autorización.

`historias.md` es un **consolidado generado**: se edita el fichero de `docs/` correspondiente y se
ejecuta `pnpm docs:build`. Editarlo directamente se pierde en la siguiente regeneración.

## Metodología

Desarrollo guiado por especificaciones: Spec → Plan → Tasks → Implement.

- **Cambio pequeño** — corrección localizada, texto, ajuste visual, refactor sin efecto
  observable: no requiere spec. Si al abordarlo aparece impacto en comportamiento observable,
  contratos, modelo de datos, seguridad, permisos, registro o arquitectura, **deja de ser pequeño**.
- **Cambio relevante** — funcionalidad nueva, cambio de comportamiento, reglas de negocio,
  permisos, contratos, modelo de datos, arquitectura: flujo completo.

Las skills de SpecKit ya están instaladas (`speckit-specify`, `speckit-clarify`, `speckit-plan`,
`speckit-tasks`, `speckit-analyze`, `speckit-implement`, y otras). **Actívalas en lugar de
reconstruir el procedimiento de memoria.**

## Antes de implementar

Skill: **`preparacion`**. En resumen:

- No modificar comportamiento fuera de los criterios documentados.
- No hacer refactorizaciones ni optimizaciones no solicitadas.
- Si la intención no está clara al 100 %, **preguntar antes de implementar**.

## Límites duros

Nada de esto se hace sin autorización explícita:

- **Modificar `.specify/memory/constitution.md`.** Si algo debería ser principio permanente, se
  propone y se espera respuesta.
- **Cambiar contratos, modelo de datos, permisos de Tauri o seguridad.** Un permiso nuevo exige
  su ADR en `docs/decisions.md`.
- **Añadir dependencias.** Amplían la superficie de un binario privilegiado que se distribuye a
  terceros. Toda dependencia nueva exige justificación escrita.
- **Commit, push o abrir PR.** Puedes consultar estado, issues y checks libremente. Cuando haga
  falta confirmar trabajo, **pídelo**.
- **Borrar o fusionar specs históricas.**
- **Tocar `third-party/`.** Los hashes están registrados en `THIRD_PARTY_NOTICES.md` y
  `pnpm verify` falla si cambian.

Los tres primeros **están bloqueados por un hook** (`.claude/hooks/proteger-rutas.mjs`): el intento
se rechaza con un mensaje que explica la vía correcta. No es una sugerencia.
- **Resolver una ambigüedad dentro del código, en silencio.** Si al construir aparece una decisión
  que no está escrita, se añade a `docs/open-questions.md` con su valor propuesto **antes** de
  programarla. Es la infracción más grave del proceso.

## Al cerrar una tarea

Skill: **`cierre-tarea`**. Todo cambio observable se documenta; la excepción exige justificación,
no al revés.

## Documentación de librerías

Para lenguajes, librerías y frameworks, consulta **Context7** si está disponible y verifica la
versión real que usa el repositorio antes de aplicar un patrón. La constitución fija las versiones
exactas. Si Context7 no está disponible, dilo y trabaja con documentación oficial.

## Instrucciones adicionales

- **Reglas por ámbito**: `.claude/rules/` — se cargan solas al tocar los ficheros que les
  corresponden. No hace falta invocarlas.
- **Procedimientos**: `.claude/skills/` — se activan por su descripción o con `/nombre`.
- **Fallos conocidos y silencios**: `docs/known-issues.md`. Todo `svelte-ignore` o
  `eslint-disable` debe enlazar a una entrada suya, y `pnpm verify` lo comprueba.
- **Enforcement**: `.claude/settings.json` preaprueba los comandos de solo lectura y registra el
  hook que protege las rutas críticas.
- La **memoria automática** del agente es local a la máquina y no se comparte con el equipo. Nada
  que deba conocer otra persona u otro agente vive solo ahí: va versionado en el repositorio.
