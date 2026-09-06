# 08b · Perfiles de alerta: los números

Responde al punto 4 de tu revisión. Complementa `08-onboarding.md` §2 (paso 3).

Los tres perfiles **no** son reglas nuevas: son tres juegos de valores para las reglas que ya existen en
`settings.alerts`. Ninguna clave nueva, ningún evaluador nuevo.

## Tabla

| Regla | Clave en `settings.alerts` | Prudente | Equilibrado *(fábrica)* | Solo lo grave |
|---|---|---|---|---|
| Temperatura · advertencia | `tempConfiguredWarnC` | 55 | **60** | 70 |
| Temperatura · crítico | `tempConfiguredCritC` | 65 | **70** | 80 |
| Desgaste · advertencia | `wearWarnPercent` | 70 | **80** | 90 |
| Desgaste · crítico | `wearCritPercent` | 85 | **90** | 95 |
| Espacio libre · advertencia (%) | `volumeFreeWarnPercent` | 15 | **10** | 5 |
| Espacio libre · advertencia (abs.) | `volumeFreeWarnBytes` | 30 GB | **20 GB** | 10 GB |
| Espacio libre · crítico (%) | `volumeFreeCritPercent` | 8 | **5** | 2 |
| Espacio libre · crítico (abs.) | `volumeFreeCritBytes` | 15 GB | **10 GB** | 5 GB |
| Errores de medios nuevos · advertencia | `mediaErrorsWarnPer24h` | 1 | **1** | 5 |
| Errores de medios nuevos · crítico | `mediaErrorsCritPer24h` | 3 | **5** | 15 |
| Reintentos del controlador · advertencia | `driverRetryWarnPer24h` | 2 | **5** | 12 |
| Reintentos del controlador · crítico | `driverRetryCritPer24h` | 6 | **12** | 30 |

Las claves son **orientativas**: si en `settings.alerts` se llaman de otra forma, manda tu esquema. Lo que
importa es la correspondencia perfil → valores.

## Cuatro reglas de aplicación

1. **El límite del fabricante manda sobre el perfil.** Si el disco declara su límite térmico, la
   advertencia salta al mínimo de (límite del fabricante, valor del perfil). Un perfil «Solo lo grave»
   con un disco que declara 70 °C avisa a 70 °C, no a 70 °C del perfil por casualidad: a 70 porque lo
   dice el fabricante. Esto ya lo hace la regla `temp_above_vendor_limit`; el perfil no la desactiva.
2. **Espacio libre: se toma el criterio que salte primero** (el mayor de porcentaje y valor absoluto),
   igual que hoy. Sin cambios en `capacityState()`.
3. **Los discos sin SMART no participan** de las reglas de temperatura, desgaste y errores de medios.
   Sí de las de capacidad y de eventos del sistema. No cuentan como avería en ningún perfil.
4. **Elegir un perfil escribe los doce valores** y además `settings.alerts.profile` con el identificador
   (`cautious` | `balanced` | `quiet`). En cuanto el usuario cambie **un** número a mano en Ajustes, el
   perfil pasa a `custom` y la interfaz lo dice: «Personalizado (a partir de Equilibrado)». Así nunca
   se muestra un perfil que no se corresponde con los valores reales.

## Qué se ve en el paso 3

Tres tarjetas de perfil con radio, nombre, etiqueta (*más avisos* / *recomendado* / *menos avisos*),
una frase de qué implica y el resumen de dos umbrales (temperatura y desgaste) como pista rápida.
Debajo, un `<details>` con **esta misma tabla filtrada al perfil elegido**, que en el mockup está
abierto para que se pueda revisar. Los valores del `<details>` cambian al cambiar de perfil.

Mockup: `mockups/smartdisk-v3.html` → **Asistente inicial** → paso **3**.
Las tres tarjetas son seleccionables y la tabla reacciona.

## Por qué perfiles y no doce campos numéricos

En el primer arranque el usuario no tiene contexto para decidir si 60 °C es mucho para su SSD. Un
formulario de doce números en el paso 3 se salta siempre, y saltárselo deja los valores de fábrica: el
mismo resultado con más fricción. Tres perfiles son una pregunta que sí sabe responder («¿te molesta que
te avise de más?»), y los doce números quedan en Ajustes para quien los quiera.
