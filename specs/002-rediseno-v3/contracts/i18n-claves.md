# Contrato — claves i18n nuevas (es + en)

Paridad exacta de claves e interpolaciones (`pnpm verify:i18n`). Cada PR añade sus claves a `es.json` **y** `en.json` en el mismo commit. Los textos de `dashboard.hero.*`, `onboarding.*` y `settings.appearance.useSystemAccent*` vienen de `RESUMEN.md` («Cambios de texto visible»).

## PR 1 — tokens (ninguna clave i18n)

## PR 2 — iconos

Ninguna clave nueva: los `aria-label` de iconos reutilizan claves de estado existentes (`health.ok`, etc.) o las de la sección que los usa.

## PR 3 — chrome

| Clave | es | en |
|---|---|---|
| `nav.about` | *(ya existe)* | |
| `global.allGood` | Todo en orden | All good |
| `global.needAttention.one` | {count} necesita atención | {count} needs attention |
| `global.needAttention.other` | {count} necesitan atención | {count} need attention |
| `global.paused` | En pausa | Paused |
| `nav.alertsUnread` | Alertas, {count} sin revisar | Alerts, {count} unread |

(La clave de pausa que hoy vive en el pie de la `Sidebar` se mueve a Ajustes; su texto se reubica, no se crea.)

## PR 4 — gráfica

| Clave | es | en |
|---|---|---|
| `chart.gap` | sin datos {from} – {to} | no data {from} – {to} |
| `chart.noSamples` | Sin muestras en las últimas 24 h | No samples in the last 24 h |
| `chart.vendorLimit` | límite del fabricante | vendor limit |

(Reutiliza `chart.tempWarnLabel`, `chart.resolution.*` existentes.)

## PR 5 — perfiles de alerta

| Clave | es | en |
|---|---|---|
| `settings.alerts.profile.title` | Perfil de alertas | Alert profile |
| `settings.alerts.profile.cautious` | Prudente | Cautious |
| `settings.alerts.profile.cautiousHint` | Avisa antes. Más avisos. | Warns earlier. More alerts. |
| `settings.alerts.profile.balanced` | Equilibrado | Balanced |
| `settings.alerts.profile.balancedHint` | Recomendado. | Recommended. |
| `settings.alerts.profile.quiet` | Solo lo grave | Only serious |
| `settings.alerts.profile.quietHint` | Solo condiciones críticas. Menos avisos. | Critical conditions only. Fewer alerts. |
| `settings.alerts.profile.custom` | Personalizado (a partir de {base}) | Custom (based on {base}) |
| `settings.alerts.profile.showThresholds` | Ver los umbrales exactos de este perfil | Show this profile's exact thresholds |
| `settings.alerts.wear` | Desgaste | Wear |
| `settings.alerts.mediaErrors` | Errores de medios | Media errors |
| `settings.alerts.driverRetries` | Reintentos del controlador | Controller retries |

Si research.md D12 introduce reglas nuevas: sus `alert.rule.<key>.title` / `.summary` en los dos idiomas (ADR-030).

## PR 6 — panel y detalle

| Clave | es | en |
|---|---|---|
| `dashboard.hero.allGood` | Todo en orden | All good |
| `dashboard.hero.allGoodBody` | Ningún disco necesita atención ahora mismo. | No disk needs attention right now. |
| `dashboard.hero.overVendorLimit` | {count} picos por encima del límite desde las {time} | {count} peaks above the limit since {time} |
| `dashboard.hero.openDisk` | Abrir el disco | Open disk |
| `dashboard.hero.viewAlert` | Ver la alerta | View alert |
| `dashboard.hero.lastValid` | último dato válido a las {time} | last valid reading at {time} |
| `dashboard.hero.noSeries` | Sin muestras en las últimas 24 h | No samples in the last 24 h |
| `dashboard.spread.title` | Reparto de estados | Status spread |
| `dashboard.events.title` | Sucesos del sistema | System events |

## PR 7 — pantallas secundarias

| Clave | es | en |
|---|---|---|
| `tests.running.title` | Prueba en curso | Test in progress |
| `settings.data.dangerZone` | Borrado de datos | Data deletion |

(El resto son cambios de token, sin literales nuevos.)

## PR 8 — acento

| Clave | es | en |
|---|---|---|
| `settings.appearance.useSystemAccent` | Usar el color de acento de Windows | Use the Windows accent colour |
| `settings.appearance.useSystemAccentHint` | Sustituye el morado de la aplicación por el color que tengas configurado en Windows. | Replaces the app's purple with the colour configured in Windows. |

(Reescribe las dos claves `settings.appearance.useSystemAccent.label` / `.hint` actuales — su texto asumía el comportamiento contrario.)

## PR 9 — asistente inicial

| Clave | es | en |
|---|---|---|
| `onboarding.step.welcome` | Bienvenida | Welcome |
| `onboarding.step.disks` | Discos | Disks |
| `onboarding.step.alerts` | Alertas | Alerts |
| `onboarding.step.done` | Listo | Done |
| `onboarding.stepIndicator` | Paso {n} de {total} | Step {n} of {total} |
| `onboarding.skip` | Omitir y usar los valores de fábrica | Skip and use factory defaults |
| `onboarding.welcome.title` | *(titular)* | |
| `onboarding.welcome.guarantee` | SmartDisk solo lee. No modifica, no repara y no borra nada de tus discos. | SmartDisk only reads. It does not modify, repair or delete anything on your disks. |
| `onboarding.welcome.cta` | Buscar mis discos | Find my disks |
| `onboarding.disks.title` | Hemos encontrado {count} discos en este equipo | We found {count} disks on this PC |
| `onboarding.disks.body` | *(ver RESUMEN.md)* | |
| `onboarding.disks.aliasLabel` | Ponle un nombre | Give it a name |
| `onboarding.disks.usbNote` | *(ver RESUMEN.md)* | |
| `onboarding.disks.cta` | Continuar con las alertas | Continue to alerts |
| `onboarding.selectedCount` | {selected} de {total} discos seleccionados | {selected} of {total} disks selected |
| `onboarding.alerts.notifyWindows` | Avisarme con una notificación de Windows | Notify me with a Windows notification |
| `onboarding.alerts.startWithSystem` | Arrancar SmartDisk con el sistema | Start SmartDisk with the system |
| `onboarding.done.title` | Listo | You're set |
| `onboarding.done.cta` | Ir al panel | Go to the dashboard |
| `onboarding.done.footnote` | Todo esto se cambia en Ajustes. | All of this can be changed in Settings. |
| `settings.onboarding.repeat` | Repetir la configuración inicial | Repeat the initial setup |

`back` / `next` reutilizan claves comunes si existen; si no, `common.back` / `common.continue`.
