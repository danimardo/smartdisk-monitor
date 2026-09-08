# Quickstart — validación de «Ignorar una alerta»

Guía para comprobar que la feature funciona de extremo a extremo. No incluye código de
implementación; los detalles están en `data-model.md` y `contracts/comandos-alertas.md`.

## Prerrequisitos

- Rama `004-ignorar-alertas` con la feature implementada.
- Base de datos de una instalación real o de desarrollo con al menos una alerta **activa de una
  regla ignorable** (p. ej. `events.controller_reset` o `temp.above_configured_warn`) y una
  **activa de una regla no ignorable** (p. ej. `smart.wear_high`).
- `pnpm app:dev` (muestra UAC: la app va elevada).

## Puertas automáticas (deben pasar todas)

```sh
pnpm check          # tipos + accesibilidad, cero avisos
pnpm lint
pnpm verify         # incluye verify:i18n — las claves nuevas en es y en
pnpm test           # health.ts: ignored no cuenta para la salud
pnpm test:component # botones Ignorar / Dejar de ignorar y estado deshabilitado
pnpm build
pnpm docs:check

# desde src-tauri/
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test          # dominio + repo_alertas + agrupacion + ciclo + migración 0003 + comandos
                    # (también regenera generated/AlertStatus.ts)
pnpm test:e2e       # e2e/ui/alerts.spec.ts: flujo de ignorar
```

## Escenarios manuales

### 1. Ignorar una alerta ignorable (US-1)

1. Abre `/alerts`, pestaña **Activas**, selecciona la alerta de regla ignorable.
2. Pulsa **Ignorar** → aparece el `ConfirmDialog` con el impacto («se sigue registrando… deja de
   avisar y de contar para el color… reversible desde Ignoradas»).
3. Confirma.
   - **Esperado**: la alerta desaparece de **Activas**; el color del disco en el panel general
     refleja solo el resto de alertas; la bandeja se recalcula.
4. Ve a la pestaña **Ignoradas** → la alerta está ahí, con su disco, severidad y última ocurrencia.

### 2. Una ocurrencia posterior no avisa pero se registra (US-1, esc. 2 y 3)

1. Con la alerta del paso 1 ya ignorada, provoca (o espera) otra ocurrencia de su condición.
   - **Esperado**: **no** hay notificación; el color **no** cambia; el `cycle` del grupo **no**
     avanza; la ocurrencia aparece en la cronología del detalle (pestaña Ignoradas → detalle).
2. Si la nueva ocurrencia es de severidad mayor, la severidad mostrada en Ignoradas sube a
   «crítico», pero sigue sin avisar ni teñir el disco.

### 3. No se puede ignorar un fallo físico (US-3)

1. Selecciona la alerta de regla **no ignorable** (`smart.wear_high`).
2. **Esperado**: el botón **Ignorar** aparece **deshabilitado**; al pasar el ratón / enfocar,
   el motivo: «Esta alerta señala un posible fallo del disco y no se puede ignorar».
3. (Contrato) Una llamada directa a `ignore_alert` con ese grupo devuelve `AppError`
   `alert.rule_not_ignorable` y el grupo no cambia de estado.

### 4. Dejar de ignorar (US-2)

1. En **Ignoradas**, selecciona un grupo, pulsa **Dejar de ignorar**.
   - **Caso A — la condición ya no se cumple**: el grupo pasa a **Resueltas**.
   - **Caso B — la condición se sigue cumpliendo**: en el siguiente ciclo del recopilador
     correspondiente el grupo aparece en **Activas** (con `cycle` incrementado) y vuelve a contar
     para el color.

### 5. Integridad (SC-006)

1. Antes de ignorar, anota `count` y número de entradas de la cronología de un grupo.
2. Ignóralo y luego deja de ignorarlo.
   - **Esperado**: ni una ocurrencia perdida; `count` solo crece; `first_occurrence_at_utc`
     intacto.

## Comprobaciones de tema y tamaño

- `/alerts` correcto en tema claro y oscuro (pestaña Ignoradas, botones, diálogo).
- Ventana mínima 1024 × 560: los 5 segmentos del filtro caben sin recorte.
- Escalado 125 % / 150 % / 200 %: sin solapes en la barra de acciones del detalle.
