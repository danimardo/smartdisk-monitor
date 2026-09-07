# Fase 1 — Guía de validación (quickstart)

Feature: Puente del registro de eventos de Windows al motor de alertas.

Cómo comprobar, de extremo a extremo, que la funcionalidad hace lo que la spec pide. No incluye
código de implementación; los detalles de cada regla están en `data-model.md` y en
`docs/alert-rules.md` §2.

---

## Prerrequisitos

- Toolchain del repo (`rust-toolchain.toml`, `.nvmrc`), `pnpm install` hecho.
- No hace falta hardware: toda la evaluación de reglas es pura y se prueba con fixtures.
- Fixtures de eventos: XML real capturado de un Windows 11 (ya hay dos en
  `collectors/event_log.rs`); se añaden los necesarios por regla, anonimizados al capturarlos
  (constitución §VIII), o sintéticos siguiendo el esquema documentado para los proveedores no
  observados (Storage Spaces, RAID — `alert-rules.md` §3.8).

---

## Validación por capas (de más barata a más cara)

### 1. Motor puro (`cargo test`, `src-tauri/src/alerts/`)

Por cada una de las ~13 reglas, las 5 pruebas de `alert-rules.md` §5:

```powershell
cd src-tauri
cargo test alerts::motor::         # funciones puras de evaluación por familia
cargo test alerts::reglas_eventos  # prueba de completitud contra alert-rules.md §3.2 y §3.3
cargo test alerts::correlacion_rafaga
cargo test alerts::                 # orquestador evaluar_eventos + barrido de resolución temporal
```

Esperado: cada regla tiene test de activación (fixture real), de **no** activación ante evento
ausente, de histéresis (oscilar el umbral de frecuencia → 1 grupo), de deduplicación (N eventos
equivalentes → 1 grupo, N ocurrencias) y de ciclo de recaída (resolver por tiempo y recaer →
`cycle+1`, contador conservado).

Casos específicos que deben tener prueba propia:

| Escenario | Comprobación |
|---|---|
| `Microsoft-Windows-Ntfs` 98 (informativo) | no crea grupo (FR-005) |
| `disk` 51 × 9 en 1 h sobre disco fijo | sin alerta; el décimo → `events.paging_error` warn, nunca crit |
| `disk` 51 sobre medio extraíble | no cuenta, no alerta |
| `disk` 11 / `storahci` 129 × 3 en 1 h, disco fijo | `events.controller_reset` escala a crítico |
| `Ntfs` 50 sobre volumen no extraíble | `events.delayed_write` crítico; extraíble → advertencia |
| ráfaga con `disk` 157 + 3 derivados en 60 s | **un** grupo `device.removed_unexpected`, 4 ocurrencias con su `triggering_event_id` |
| ráfaga sin `disk` 157 (`Ntfs` 55 + `disk` 7) | **dos** grupos (FR-009a) |
| `disk` 157 que llega **después** de un derivado ya agrupado | el grupo derivado se resuelve con nota de causa; sus eventos pasan a ocurrencias del `device.removed_unexpected` (D4) |
| disco fijo que desaparece del inventario sin `disk` 157 | `device.removed_unexpected` crítico (D2) |
| disco USB que desaparece sin `disk` 157 | **sin** alerta (D2) |
| evento con `mapping_confidence = unknown` | grupo **sin objeto**, dedup por `provider:event_id`, `context_json` lo anota (FR-004a) |
| eventos históricos ya en `system_events` al arrancar el puente | cero grupos (FR-018 / D6): se insertan directos en la tabla, se corre un ciclo, no aparece nada |
| resolución temporal | grupo `active`; se avanza el reloj 24 h/7 días sin evento nuevo → `resolved`; llega otro → `active`, `cycle 2` |

Cobertura: `src-tauri/src/alerts/` y `src-tauri/src/domain/` ≥ 90 % (`cargo llvm-cov`).

### 2. Integración del ciclo (`cargo test`, `src-tauri/src/commands/`)

```powershell
cargo test commands::   # refresh_events devuelve transiciones; post_procesar_ciclo las procesa
```

Esperado:
- Un ciclo con eventos nuevos que activan reglas produce transiciones que llegan a
  `procesar_transiciones` (notificación) y a `emitir_alerts_changed`.
- Un fallo simulado del colector de eventos no impide que el ciclo SMART/capacidad produzca sus
  alertas, ni al revés (FR-015, SC-008).
- `device.removed_unexpected` se dispara desde la baja de inventario (`ids_dados_de_baja`).

### 3. Contrato de tipos (`cargo test` + `pnpm check`)

```powershell
cargo test          # regenera los DTO ts-rs si la ocurrencia gana triggeringEventId
cd ..
pnpm check          # cero errores, cero avisos
pnpm verify         # i18n sincronizado (es/en), fronteras, tokens
```

Esperado: `alert.rule.<rule_key>.title/summary` de las 12 `rule_key` nuevas en ambos diccionarios;
el DTO de ocurrencia con `triggeringEventId` (generado o Zod + prueba de rechazo).

### 4. Interfaz (`pnpm test:e2e`, `pnpm test:a11y`)

```powershell
pnpm test:e2e alerts
pnpm test:e2e events
pnpm test:a11y
```

Esperado:
- `/alerts` con alertas de reglas de eventos activas: se ven los **títulos traducidos**, nunca la
  `rule_key` ni `provider:event_id` (SC-006). Ampliar la lista de `alerts.spec.ts`.
- El detalle de una alerta de evento muestra un enlace «Ver el suceso» que lleva a
  `/events?focus=<id>` y resalta ese evento (SC-004).
- Cero incumplimientos de axe en `/alerts` y `/events`, ambos temas.

### 5. Prueba de humo sobre el registro real (manual, opcional)

Con la app compilada y elevada en un Windows real: dejar correr unos minutos y comprobar en
`/alerts` que **no** aparece ninguna alerta crítica falsa en un equipo sano (SC-001). Desconectar en
caliente un disco USB de descarte y comprobar que aparece **una** alerta `device.removed_unexpected`
(advertencia, por ser USB), no cuatro (SC-002), y que se resuelve al reconectarlo.

---

## Puertas antes de proponer commit

Las nueve del proyecto (`AGENTS.md` §«Al cerrar una tarea» / skill `cierre-tarea`):

```powershell
pnpm check ; pnpm lint ; pnpm verify ; pnpm test ; pnpm test:component ; pnpm test:e2e ; pnpm test:a11y ; pnpm build ; pnpm docs:check
cd src-tauri ; cargo fmt --check ; cargo clippy --all-targets -- -D warnings ; cargo test
```

Documentación a actualizar al cerrar (skill `cierre-tarea`): `docs/alert-rules.md` (filas
implementadas + §3.5 si se precisó), `docs/open-questions.md` (§J con D2–D6, y J.16), `docs/ui-contract.md`
(`triggeringEventId`, `/events?focus`), `docs/data-model.md` (§2/§3 nota de `triggering_event_id`),
y `pnpm docs:build`.
