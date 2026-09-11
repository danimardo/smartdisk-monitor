# Implementation Plan: Informe HTML por disco (contenido útil + resumen con IA)

**Branch**: `009-informe-mejorado` | **Date**: 2026-09-10 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/009-informe-mejorado/spec.md`

## Summary

Rehacer el **formato HTML** del informe de la pantalla de Informes para que, por disco, sea un
documento útil (identidad, salud «a fecha de hoy» con nota de antigüedad, contadores SMART con
delta del intervalo, alertas del disco con frase legible, eventos de Windows, y dos mini-gráficas
SVG embebidas) y, opcionalmente, un **resumen en lenguaje llano por disco** generado con IA
(principio XVI 1.11.0): una llamada por disco, con vista previa del texto anonimizado, degradación
sin tirar la exportación, progreso y cancelación. CSV y JSON no cambian.

**Enfoque técnico**: el HTML se sigue generando en el backend (`reporting::informe`); la interfaz
le **inyecta un mapa de textos legibles** para las claves de regla de alerta (respeta ADR-030 en su
espíritu: el backend no *posee* el texto, se le pasa para este render). El resumen con IA reutiliza
toda la pila de la ayuda existente (`domain::ia`, `platform::ia_openrouter`, `Anonimizador` +
`redactar_identificadores`), con una nueva variante de composición de consulta para el informe y un
comando de exportación asíncrono con eventos de progreso y bandera de cancelación.

## Technical Context

**Language/Version**: Rust 1.x (backend, `src-tauri/`), TypeScript + Svelte 5 (frontend, `src/`) —
versiones fijadas en la constitución.

**Primary Dependencies**: Tauri 2, `reqwest` 0.13 (ya en el árbol, principio XVI), `time`, `rusqlite`,
`sha2`. **Sin dependencias nuevas.**

**Storage**: SQLite (`metric_samples`, `metric_aggregates`, `alert_groups`, `alert_occurrences`,
`system_events`, `devices`, `volumes`). Sin cambios de esquema.

**Testing**: `cargo test` (dominio + comandos con `conn` en memoria), `pnpm test` (lógica Node),
`pnpm test:component` (Chromium), `pnpm test:e2e` (Playwright + IPC falso), `pnpm test:a11y`.

**Target Platform**: Windows 10/11 y Windows Server con Experiencia de escritorio; el informe HTML
se abre en cualquier navegador, también sin conexión.

**Project Type**: Aplicación de escritorio (Tauri + SvelteKit `adapter-static`, SSR desactivado).

**Performance Goals**: exportación **sin** IA prácticamente inmediata (percepción «al instante»,
igual que hoy). Exportación **con** IA: una llamada por disco en serie; con 4 discos, decenas de
segundos; con 20, minutos — con progreso visible y cancelación.

**Constraints**: HTML autónomo (CSS embebido, cero recursos remotos, SVG en línea); llamada de IA
solo desde Rust; anonimización en el dominio antes de que el texto salga del proceso; sin reintento
automático; el informe se genera aunque la IA falle.

**Scale/Scope**: equipos con 1–~20 discos físicos y sus volúmenes. Un informe = una exportación
(sin cola).

## Constitution Check

*GATE: pasa antes de la Fase 0; se revisa tras la Fase 1.*

| Principio | Cómo lo cumple este plan |
|---|---|
| **I** — Datos de salud reales, nunca inventados | El resumen con IA se presenta como **orientación**, marcado como generado por IA con su procedencia; no alimenta ninguna decisión ni sustituye al estado de salud, que sale de los datos. |
| **III / IX** — Cero red / no se transmite nada | Excepción **acotada** del principio XVI. Con la casilla «incluir resumen con IA» apagada (fábrica) no hay cliente, conexión ni resolución de nombres. |
| **VI** — Sistema de diseño | La casilla nueva y el modal de vista previa usan el catálogo (`Switch`, patrón `ExplicacionModal`/`ConfirmDialog`); tokens, i18n en los dos idiomas. |
| **VII** — Accesibilidad | Modal de vista previa con foco atrapado y `Escape`; progreso con `aria-live`; el HTML del informe con jerarquía de encabezados y `<table>` reales. |
| **VIII** — Fallo silencioso = defecto | La generación registra (`tracing`) cada llamada de IA (modelo, resultado, ms), como el flujo existente; los discos con resumen fallido lo dicen en el informe. |
| **XVI** (1.11.0) — Asistencia con IA | **Opt-in apagado de fábrica**; **una llamada por disco**, nunca combinando discos; alcance limitado a alertas + contexto de sucesos + SMART + resumen numérico de ese disco; **anonimización en el dominio**; **vista previa** del texto exacto de cada disco antes de la primera llamada, una confirmación; llamada **desde el backend**; respuesta como **markdown seguro**; **degradación** sin tirar la exportación. |
| **XV** — Registro sin identificadores | El log de la generación no lleva número de serie, nombre de equipo ni ruta de perfil (mismo criterio que el flujo de IA existente). |
| Dependencias (III) | **Ninguna nueva.** `reqwest` ya está. |
| Permisos de Tauri | **Ninguno nuevo.** La llamada de red es desde Rust (ADR-046 / constitución 1.8.1). |
| Contratos (límite duro) | `export_report` gana parámetros; nuevo comando `preview_informe_ia`; nuevo comando `cancelar_informe`; nuevo evento `report:progress`. Todo ello en **ADR-057** (requerido por la fila 1.11.0 de la constitución). |
| ADR-030 (el backend no manda texto de alerta) | Se respeta el espíritu: el backend **no conoce** el texto; la interfaz le pasa un **mapa clave→texto** solo para este render, igual que ya le pasa `includeSerials` o el destino. Documentado en ADR-057. |

**Veredicto (pre-Fase 0)**: pasa. Las desviaciones (contratos nuevos) están cubiertas por ADR-057,
exigido por la propia enmienda constitucional ya aprobada.

### Revisión post-diseño (tras Fase 1)

Los artefactos de diseño no introducen ninguna violación nueva. Puntos confirmados en `research.md`:

- **XVI — respuesta como «texto o markdown seguro, jamás HTML»**: el resumen se incrusta como
  **texto plano HTML-escapado** con `white-space: pre-wrap`, sin renderizar markdown (D10). Cumple
  al pie de la letra sin añadir un parser/saneador.
- **XVI — anonimización**: `research.md` D4 confirma que la pila existente cubre el payload del
  informe; la única adición es `.con_etiqueta_volumen(...)` en `Anonimizador` (categoría ya listada
  en el principio desde 1.8.0), que además cierra un hueco latente del flujo de «Explícamelo».
- **Sin dependencias ni permisos nuevos**: confirmado (SVG en línea, no rasterizado; red desde
  Rust).
- **Contratos**: los tres comandos/evento nuevos y los parámetros de `export_report` están en
  `contracts/`; entran en ADR-057 y en `docs/ui-contract.md`.

**Veredicto post-diseño**: pasa.

## Project Structure

### Documentation (this feature)

```text
specs/009-informe-mejorado/
├── plan.md              # Este fichero
├── spec.md
├── research.md          # Fase 0
├── data-model.md        # Fase 1
├── quickstart.md        # Fase 1
├── contracts/           # Fase 1
│   ├── export_report.md
│   ├── preview_informe_ia.md
│   ├── cancelar_informe.md
│   └── evento-report-progress.md
└── tasks.md             # (/speckit-tasks)
```

### Source Code (repository root)

```text
src-tauri/src/
├── reporting/
│   ├── informe.rs             # AMPLÍA: secciones por disco (salud, SMART+delta, alertas legibles,
│   │                          #   eventos, mini-gráficas, hueco de resumen IA)
│   ├── minigrafica.rs         # NUEVO: SVG de una serie (polyline por tramo, huecos, print-safe)
│   ├── resumen_metricas.rs    # NUEVO: mín/media/máx/pico de una serie en un intervalo
│   ├── informe_ia.rs          # NUEVO: por disco → payload (alertas+sucesos+SMART+resumen num.),
│   │                          #   anonimización, vista previa, orquestación de las N llamadas
│   ├── anonimizar.rs          # AMPLÍA: `.con_etiqueta_volumen(...)` (lista del principio XVI)
│   └── export.rs              # sin cambios funcionales (CSV/JSON intactos)
├── domain/
│   └── ia.rs                  # AMPLÍA: `Detalle::Informe(...)` + `componer_consulta` para el informe
├── commands/
│   └── mod.rs                 # AMPLÍA: `export_report` (params), NUEVO `preview_informe_ia`,
│                              #   NUEVO `cancelar_informe`, evento `report:progress`
├── persistence/
│   ├── repo_alertas.rs        # (usa `list_groups` + filtro por `target_device_id` + rango)
│   ├── repo_varios.rs         # AMPLÍA: eventos de un disco en [desde, hasta] con mensaje
│   └── repo_metricas.rs       # (series + agregados, ya existen)
└── lib.rs                     # registra los comandos nuevos

src/
├── routes/reports/+page.svelte   # AMPLÍA: casilla «incluir resumen con IA», flujo de vista
│                                 #   previa + progreso + cancelar para el HTML
├── lib/
│   ├── api/client.ts             # AMPLÍA: `exportReport` params; NUEVO `previewInformeIa`,
│   │                             #   `cancelarInforme`; suscripción a `report:progress`
│   ├── api/schemas.ts            # esquemas Zod nuevos
│   ├── components/               # reusa `ExplicacionModal`/`ConfirmDialog` o un `InformeIaModal`
│   ├── stores/                   # NUEVO store del flujo de exportación con IA (o ampliar reports)
│   └── i18n/{es,en}.json         # claves nuevas + el mapa de textos de regla que se inyecta
└── ...

e2e/ui/reports.spec.ts            # AMPLÍA: casilla, vista previa, degradación, cancelación
```

**Structure Decision**: aplicación de escritorio existente; se amplían módulos que ya existen
(`reporting/`, `domain/ia.rs`, `commands/mod.rs`, `routes/reports/`) y se añaden cuatro ficheros
Rust acotados (`minigrafica`, `resumen_metricas`, `informe_ia`, y la ampliación de `anonimizar`).
El dominio (`domain/`, `reporting/` salvo el comando) sigue sin conocer Tauri: la orquestación de
red y los eventos viven en `commands/mod.rs`.

## Complexity Tracking

*Sin violaciones de la constitución que justificar.* Los contratos nuevos (comandos y evento) son
la vía prevista por la enmienda 1.11.0 y quedan en ADR-057; no hay alternativa más simple que
evite un flujo asíncrono con progreso para N llamadas de red cancelables.
