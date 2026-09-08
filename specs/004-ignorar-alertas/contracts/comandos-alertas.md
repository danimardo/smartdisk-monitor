# Contrato — comandos de alertas (delta de la feature `004-ignorar-alertas`)

Formato: igual que `docs/ui-contract.md` §3.4. Aquí solo lo que cambia; al cerrar la feature se
integra en `docs/ui-contract.md`.

---

## Tipos afectados

```ts
// generated/AlertStatus.ts  (ts-rs, regenerado por `cargo test`)
type AlertStatus = "active" | "acknowledged" | "resolved" | "archived" | "ignored";

interface AlertDetail extends AlertGroup {
  ruleIgnorable: boolean;          // NUEVO: false para las 7 reglas de fallo físico (ver abajo)
  facts: { labelKey: string; value: string | null }[];
  occurrences: { occurredAt: string; cycle: number; value: number | null; eventId: string | null; context: string | null }[];
  relatedEvents: SystemEvent[];
}
```

`AlertGroup` / `AlertGroupWire` no cambian de forma; su `status` ya puede valer `"ignored"`.

---

## Comandos nuevos

```ts
invoke<void>("ignore_alert", { alertGroupId: string })
// Lleva el grupo a `ignored`. Falla con AppError `alert.rule_not_ignorable`
// (i18n `error.alertRuleNotIgnorable`) si la regla del grupo está en el conjunto no ignorable.
// Falla con `device.not_found` si el id no existe.
// Efecto: deja de contar para el color y de notificar; sigue registrando ocurrencias.
// Emite `alerts:changed` con el grupo. Recalcula el color de la bandeja.

invoke<void>("unignore_alert", { alertGroupId: string })
// Saca el grupo de `ignored` y lo deja en `resolved`. El motor lo promociona a `active`
// (cycle + 1) en el siguiente ciclo si la condición sigue cumpliéndose.
// Falla con `device.not_found` si el id no existe.
// Emite `alerts:changed`. Recalcula el color de la bandeja.
```

Ambos siguen el patrón de `archive_alert`: comando fino, `existe_grupo` → `alerts::ciclo::*` →
`emitir_alerts_changed` → `platform::bandeja::actualizar`.

---

## Conjunto de reglas **no ignorables** (`ruleIgnorable === false`)

| `rule_key` | Motivo |
|---|---|
| `smart.health.failed` | el disco declara FALLO de salud |
| `nvme.critical_warning` | el disco enciende su propia bandera crítica |
| `smart.wear_high` | desgaste del SSD (irreversible) |
| `smart.spare_below_threshold` | bloques de reserva agotándose |
| `smart.media_errors` | errores de medio acumulados |
| `smart.error_log` | errores en el registro SMART acumulados |
| `events.disk_predictive` | `disk` 52: Windows predice fallo próximo |

Cualquier otra regla (`temp.*`, `capacity.*`, `events.filesystem_error`, `events.disk_error`,
`events.controller_reset`, `smart.unreadable`, `collector.stalled`, …) es ignorable.

---

## Errores nuevos

| Código | Clave i18n | Frase (es) | ¿Reintentable? |
|---|---|---|---|
| `alert.rule_not_ignorable` | `error.alertRuleNotIgnorable` | «No se puede ignorar esta alerta.» | no |

---

## Filtro de `get_alert_groups`

Sin cambios de firma. La pantalla pasa `status: ["ignored"]` para la pestaña «Ignoradas». El
filtro `all` (sin `status`) devuelve también los grupos `ignored`.

---

## Cliente TypeScript (`src/lib/api/client.ts`)

```ts
export const ignoreAlert   = (alertGroupId: string) => callVoid("ignore_alert",   { alertGroupId });
export const unignoreAlert = (alertGroupId: string) => callVoid("unignore_alert", { alertGroupId });
```

---

## Registro en `lib.rs`

`tauri::generate_handler![ … , commands::ignore_alert, commands::unignore_alert ]`
(la lista es cerrada por diseño; añadir aquí es obligatorio para que sean alcanzables).
