# Phase 0 — Investigación y decisiones

Todas las preguntas abiertas de la spec están resueltas (por el usuario o por análisis del código). Aquí se consolidan con su razón.

## D1. Alcance de los perfiles de alerta

- **Decisión**: alcance completo (opción A del usuario). Se añaden umbrales configurables de desgaste, errores de medios por 24 h y reintentos de controlador por 24 h a `settings.alerts`, más `settings.alerts.profile`. El motor de alertas los consume.
- **Razón**: sin umbrales reales el paso 3 del asistente sería decorativo, y la constitución §I exige que la UI no mienta sobre qué está activo. El usuario ha aceptado el coste de backend.
- **Alternativas descartadas**: alcance reducido (solo temperatura + capacidad) y sin paso 3 — rechazadas por el usuario.
- **Aislamiento**: todo el trabajo de backend va en el PR 5, precedido de modo plan y con su ADR. El resto de la entrega no depende de él salvo el asistente (PR 9).

## D2. Patrón de carga de las sparklines del panel

- **Decisión**: carga perezosa por tarjeta visible (opción B del usuario). El `load` de la ruta trae solo el inventario (como hoy). Tras el primer render, cada `DiskCard`/`HeroPanel` visible pide su serie de temperatura de 24 h con `getMetricSeries`. Las tarjetas fuera de la ventana de la `VirtualList` no disparan petición.
- **Razón**: cargar 20 series en el `load` universal bloquea la primera pintura y rompe el presupuesto de 50 ms (SC-007). La carga perezosa mantiene la pantalla reactiva y degrada por disco si una serie falla (constitución §X).
- **Implementación**: un pequeño caché en memoria en el store (`app.svelte.ts`) evita repetir la petición al re-montar filas de la `VirtualList` durante el scroll. La medición real se registra en `docs/open-questions.md`.
- **Alternativas descartadas**: `load` universal (bloquea), comando agregado (comando nuevo, contra el alcance).

## D3. Segunda familia tipográfica

- **Decisión**: no se empaqueta Bricolage Grotesque (opción «b» del usuario). `--sdm-font-display` resuelve a `var(--sdm-font-sans)`. `.sdm-display` = familia sans + peso 600 + `tabular-nums` + `letter-spacing` del token + `line-height: 1`.
- **Razón**: evita gestionar un binario más, una licencia OFL más y un ADR de fuente, en un binario privilegiado que se distribuye a terceros (constitución §III). Se pierde el carácter condensado de las cifras grandes, no función.
- **Corrección al entregable**: `00b-tipografia-y-fuentes.md` dice que Instrument Sans se carga hoy por `@import` de Google Fonts. **Es incorrecto**: `tokens.css` ya la empaqueta localmente con `@font-face` desde ADR-018 / `open-questions.md` K.1. No hay nada que arreglar ahí.
- **Puerta abierta**: si más adelante se quiere la segunda familia, el único punto de cambio es `.sdm-display` y el `@font-face`; ningún componente cambia.

## D4. Cambio de identidad de acento — ADR

- **Decisión**: dos ADR nuevos en `docs/decisions.md`, redactados en el PR 1:
  - **ADR-034 — Sistema de diseño v3: paleta propia «Ciruela»**. Enmienda ADR-013. Registra el cambio de neutros, acento y rojo crítico; qué NO cambia (material, radios, movimiento, catálogo); y que `docs/ui-design.md` pasa a describir v3.
  - **ADR-035 — La herencia del acento de Windows pasa a opción apagada de fábrica**. Enmienda ADR-017 (que se conserva en su parte técnica: `accessibleAccent()`/`accentOnSurface()` siguen corrigiendo el acento del usuario cuando el interruptor está encendido).
- **Nota de constitución**: §VI dice «El acento… se hereda de Windows y se corrige antes de aplicarse». Se añade una nota al pie de ese punto remitiendo a ADR-035: la herencia es ahora opcional; la corrección de contraste sigue siendo obligatoria cuando se usa. No es una enmienda de principio (el acento sigue sin comunicar salud, sigue siendo acción/selección, sigue corregido si se hereda); es una precisión del mecanismo, permitida por el propio §VI que remite a `ui-design.md`.
- **Autorización**: el usuario ha aprobado la dirección; los ADR documentan, no vuelven a someter a decisión.

## D5. Estado global calculado una sola vez

- **Situación actual**: `Toolbar` recibe `globalState`/`globalLabel` como props sueltas; `Sidebar` recibe `footerNote` (texto libre). En las capturas discrepan.
- **Decisión**: el `+layout.svelte` (o su `load`) calcula el estado global **una vez** con `worstState()` sobre los discos monitorizados del store, y pasa el mismo resultado a `Toolbar` (píldora con icono) y a `Sidebar` (icono + contador al pie, sin texto). El riel no lleva texto, así que no puede contradecir a la píldora.
- **`selectHeroDisk()`**: función nueva en `$lib/design/health.ts`, junto a `worstState()`. Criterio del entregable (`HeroPanel.md`): (1) disco con alerta activa de mayor severidad, empate → ocurrencia más reciente; (2) disco de sistema; (3) primero del inventario; (4) los sin SMART nunca, salvo que sean los únicos. Con su prueba unitaria.
- **Los 3 bugs preexistentes** se corrigen aquí y en el PR 4:
  - Título «Panel general» fijo → `Toolbar` recibe `title`/`subtitle` de `$page.data`/`$page.route` (PR 3).
  - «Sin discos monitorizados» con un disco → el cálculo pasa por `worstState()` sobre la lista real (PR 3).
  - Gráfica vacía + serie de dos tramos → `vector-effect="non-scaling-stroke"` y un `polyline` por tramo en `Sparkline`, usado por `TimeSeriesChart` (PR 4).

## D6. Montaje del sprite de iconos

- **Decisión**: el bloque `<svg width="0" height="0">` con los 15 `<symbol>` se pega en `AppShell.svelte` una sola vez, **sin** el bloque `<metadata><c2pa:manifest>` (procedencia del fichero de diseño, ~5 KB de base64, irrelevante en el bundle). `Icon.svelte` renderiza `<svg><use href="#i-{name}" /></svg>`.
- **`stroke-width`**: el sprite fija `stroke-width="1.7"` inline. Se deja así (no es color/radio/sombra/tamaño de fuente, `verify:tokens` no lo marca) y `Icon.svelte` puede además exponerlo por CSS si algún uso necesita otro grosor. `--sdm-icon-stroke` queda como token documentado para coherencia.
- **`viewBox`**: todos comparten `0 0 24 24`. El tamaño de render lo pone el contenedor vía `width`/`height` en el `<svg>` de `Icon`.
- **Accesibilidad**: `label` presente → `role="img"` + `aria-label`; ausente → `aria-hidden="true"`. Regla en `Icon.md`.

## D7. Asistente inicial — enrutado y guardián

- **Decisión**: `settings.onboarding.completedAt` (ISO UTC o `null`). En `+layout.ts` `load` (universal, se ejecuta antes del primer render), si `completedAt` es nulo y la ruta no es `/onboarding`, se redirige con `redirect(302, "/onboarding")` (redirección programática, permitida por constitución §XIV; no es `goto()` en un `onclick`).
- **Instalaciones existentes** (FR-043): en el arranque, si `completedAt` es nulo **pero** existe cualquier otra clave de `settings` guardada por el usuario (o hay discos ya excluidos/con alias), se graba `completedAt = now` sin mostrar el asistente. Alternativa considerada: migración en el backend — se descarta por no tocar `src-tauri` fuera del PR 5; la comprobación en el `load` es suficiente y no persiste nada engañoso.
- **Relanzar** (FR-037): Ajustes → «Repetir la configuración inicial» pone `completedAt = null` y navega a `/onboarding`. No borra discos, alias ni umbrales: el asistente abre con los valores actuales.
- **Chrome**: el asistente vive fuera del `AppShell` (sin riel ni `Toolbar`). El `+layout.svelte` comprueba `$page.route.id === "/onboarding"` y renderiza solo `{@render children()}` con su cabecera propia.

## D8. «Reparto de estados» vs `HealthDonut`

- **Decisión**: `HealthDonut` se queda en el catálogo y en su prueba, **sale del panel general**. El panel usa un bloque de composición (`Card` + `Icon` + barra de proporción de 9 px + cifras), no un componente nuevo (no cumple la regla de ≥3 pantallas de `ui-design.md` §3).
- **Razón**: con 2–4 discos un anillo de cuatro segmentos es ilegible; con muchas muestras (Informes) sí lo es, pero meter `HealthDonut` en Informes es funcionalidad nueva (qué mide) y queda fuera de esta feature.

## D9. Claves de `settings.alerts` — nombres finales

El entregable (`08b`) propone nombres que no coinciden con el esquema actual y dice «manda tu esquema». Mapeo adoptado (detalle en `data-model.md` y `contracts/settings-alerts.md`):

| Concepto del entregable | Clave adoptada | ¿Existe hoy? |
|---|---|---|
| Temperatura advertencia/crítico | `tempConfiguredWarnC` / `tempConfiguredCritC` | sí |
| Espacio libre % advertencia/crítico | `capacityWarnPercent` / `capacityCritPercent` | sí |
| Espacio libre abs. advertencia/crítico | `capacityAbsoluteFloorWarnBytes` / `capacityAbsoluteFloorCritBytes` | sí |
| Desgaste advertencia/crítico | `wearWarnPercent` / `wearCritPercent` | **NUEVO** |
| Errores de medios nuevos /24 h adv./crít. | `mediaErrorsWarnPer24h` / `mediaErrorsCritPer24h` | **NUEVO** |
| Reintentos de controlador /24 h adv./crít. | `driverRetryWarnPer24h` / `driverRetryCritPer24h` | **NUEVO** |
| Identificador de perfil | `profile` (`cautious`/`balanced`/`quiet`/`custom`) | **NUEVO** |

Son **7 claves nuevas**, no ~10: la capacidad y la temperatura ya están. El «espacio libre abs.» del entregable (30/20/10 GB) se traslada a los `...FloorWarn/CritBytes` existentes.

## D10. Compatibilidad de `pnpm verify` durante la transición

- `verify:tokens` exige **una sola copia** del sistema de diseño y **cero literales**. El sprite va en `AppShell.svelte` como marcado (sin literales de color: `currentColor`). El velo del héroe usa `linear-gradient(..., var(--sdm-glass) ...)` — variables, no literales. Los gradientes de `Sparkline` usan `stop-color="currentColor"` + `stop-opacity` (número, no color literal). Todo pasa.
- `verify:i18n` exige paridad es/en y que toda clave usada exista. Cada PR añade sus claves a los dos diccionarios en el mismo commit.
- `verify:boundaries` y `verify:assets` no se ven afectados (sin assets nuevos, sin cruces de frontera nuevos salvo el esquema de settings del PR 5, que sí añade su prueba de rechazo).

## D12. Semántica de conteo de los umbrales «por 24 h» — RESUELTO (clarify Q1 → opción i)

**Decisión (2026-09-06)**: opción (i). `mediaErrors*` se reinterpreta como umbral sobre la magnitud del incremento de `media_errors_total` por ciclo; `driverRetry*` es la `N` configurable de `events.controller_reset` / `events.io_retry` (hoy fija en 3/1 h y 5/1 h). Se renombran en la interfaz sin «/24 h». No se construye conteo por ventana de 24 h ni ninguna regla nueva. Además (clarify Q2): el perfil Equilibrado fija la temperatura de fábrica en 60/70 °C.

---

*(Contexto original, conservado.)*

- **Problema**: `08b-perfiles-de-alerta.md` define `mediaErrorsWarnPer24h`/`CritPer24h` y `driverRetryWarnPer24h`/`CritPer24h` como conteos **por 24 h**. El motor actual **no cuenta así**:
  - `smart.media_errors` activa cuando `media_errors_total` **aumenta** respecto a la lectura anterior (crítico inmediato), sin ventana.
  - `smart.error_log` activa al aumentar; crítico si aumenta **3 ciclos seguidos**.
  - `events.controller_reset` es «≥ 3 en 1 h»; `events.io_retry` es «≥ 5 en 1 h».
  - Ninguna regla usa una ventana de 24 h para contar ocurrencias.
- **Parametrización limpia y sin discusión** (entra seguro en el PR 5):
  - `smart.wear_high` 90/100 → `wearWarnPercent` / `wearCritPercent`.
  - `temp.above_configured_warn/crit` 70/80 → `tempConfiguredWarnC` / `tempConfiguredCritC` (ya existen).
  - `capacity.low/critical` → porcentajes y suelos absolutos de `settings.alerts` (ya existen; hoy `capacityState()` lee constantes, pasará a leer `settings`).
- **Opciones para los dos umbrales «por 24 h»**:
  | Opción | Qué se hace | Coste |
  |---|---|---|
  | (i) *(recomendada)* | Se reinterpretan sobre las reglas que ya existen: `mediaErrors*` como umbral sobre la **magnitud del incremento** de `media_errors_total` por ciclo; `driverRetry*` como la **N configurable** de `events.controller_reset` / `events.io_retry` (hoy 3/1 h y 5/1 h). Se renombran en la UI para no prometer una ventana de 24 h que no existe. | Bajo: parametriza reglas existentes |
  | (ii) | Se implementa conteo real por ventana de 24 h para esas dos reglas. | Alto: nueva mecánica de conteo en el motor + sus 5 pruebas |
  | (iii) | Los perfiles solo tocan desgaste + temperatura + capacidad; errores de medios y reintentos quedan fuera de esta feature. | Contradice «alcance completo» elegido por el usuario |
- **Estado**: RESUELTO por `/speckit-clarify` (Q1 → opción i, Q2 → temperatura de fábrica a 60/70). Se registra en `docs/open-questions.md` en el PR 5 antes de escribir código.

## D13. Disco de sistema para el héroe — RESUELTO (clarify Q3)

- **Decisión**: el backend añade `isSystemVolume: boolean` a `VolumeSummary` (Zod + ts-rs + prueba de rechazo). `selectHeroDisk()` regla 2 = el disco que contiene un volumen con `isSystemVolume === true`. Ninguna inferencia en presentación (constitución §IV). Entra en el PR 6, o se adelanta al PR 5 que ya toca `src-tauri` y regenera tipos.
- **Razón**: hoy ningún campo de `DiskSummary`/`VolumeSummary` marca el volumen de arranque; deducirlo por «unidad C:» es frágil y viola la separación dominio/presentación.

## D14. Guardián del asistente para instalaciones existentes — RESUELTO (clarify Q4)

- **Decisión**: comprobación en `+layout.ts` `load`: si `completedAt` es nulo pero ya hay cualquier ajuste guardado / alias / disco excluido, se graba `completedAt = ahora` con `setSetting` y no se muestra el asistente. Sin migración de backend. Una instalación v3 desde cero interrumpida antes de guardar nada vuelve a ver el asistente.
- **Razón**: evita tocar la lógica de migraciones de `src-tauri` (fuera del alcance salvo PR 5) y cubre correctamente los dos casos (actualización con datos / instalación nueva a medias).

## D11. Fuentes de documentación consultadas

- Entregable del diseñador: `design/propuesta-redisenov2/` (fuente principal).
- Código actual: `src/design-system/tokens.css`, `src/lib/components/*`, `src/lib/design/health.ts`, `src/lib/api/schemas.ts`, `src/routes/*`, `src-tauri/src/alerts/*`.
- Normativa: `.specify/memory/constitution.md`, `docs/ui-design.md`, `docs/alert-rules.md` (§2 tabla de reglas, §5 pruebas exigidas), `docs/decisions.md` (ADR-013, ADR-017, ADR-018, ADR-029), `docs/open-questions.md` (§O acento, J.32 settings, K.1 fuentes, J.41 riel).
- No se ha consultado Context7: no hay librería ni framework nuevo; las versiones de la pila las fija la constitución y no se tocan.
