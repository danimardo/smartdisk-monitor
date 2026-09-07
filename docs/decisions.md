# Registro de decisiones técnicas

## ADR-001 — Tauri 2 para escritorio

Estado: aceptada.

Se utilizará Tauri 2 por su integración nativa, tamaño razonable, backend Rust, systray y capacidad de empaquetar un ejecutable auxiliar. La primera plataforma es Windows x64.

## ADR-002 — Svelte en lugar de React

Estado: aceptada.

La interfaz usará Svelte + TypeScript. React no se incluirá. Utilizar simultáneamente ambos frameworks duplicaría responsabilidades sin una necesidad funcional.

Tailwind CSS proporcionará utilidades de estilo sobre los tokens aprobados. Se utilizará Svelte 5 con runes y no se usarán bibliotecas externas de componentes.

## ADR-003 — Aplicación local sin servicio

Estado: aceptada.

No existirá agente, servidor central ni servicio de Windows. La recopilación ocurre solo con sesión iniciada y aplicación activa, aunque la ventana esté en la bandeja.

## ADR-004 — Elevación de todo el proceso

Estado: aceptada.

El ejecutable solicitará `requireAdministrator`. Simplifica el acceso a dispositivos y eventos, aceptando que aparezca UAC en cada inicio y que toda la UI viva dentro de un proceso elevado.

Mitigación: capacidades Tauri mínimas, sin shell genérica desde JavaScript y comandos privilegiados cerrados en Rust.

**Implementación** (2026-09-04): manifiesto propio en `src-tauri/windows/app.manifest`, aplicado
desde `build.rs`. Sin él, Tauri genera uno por defecto con `asInvoker` y la aplicación **no pediría
elevación**, que es como estuvo hasta que lo destapó el diseño de la estrategia de pruebas.

El manifiesto declara además `PerMonitorV2`: sin esa marca, Windows escala la ventana por su cuenta
y el resultado se ve borroso al 125 %, 150 % y 200 %, justo los escalados que exige verificar
`AGENTS.md` §4.

Consecuencia para el desarrollo: `pnpm app:dev` muestra UAC en cada arranque, y la suite E2E de
aplicación real necesita un terminal elevado (`docs/testing-strategy.md` §11).

## ADR-005 — smartctl como auxiliar independiente

Estado: aceptada.

`smartctl` será la fuente principal de SMART/NVMe y se ejecutará como binario independiente con salida JSON. Se incluirá en el instalador manteniendo avisos, licencia y obligaciones aplicables de smartmontools. El código propio conserva licencia MIT.

La API nativa de Windows será complementaria y permitirá contrastar topología, volúmenes, actividad y datos de fiabilidad disponibles.

## ADR-006 — SQLite local

Estado: aceptada.

SQLite almacenará configuración, inventario, muestras, eventos, alertas y pruebas en `ProgramData`. Se utilizarán WAL, migraciones versionadas, transacciones breves y copias previas a migraciones.

## ADR-007 — Sin red ni telemetría

Estado: aceptada.

El funcionamiento normal no necesita red. No se recopila ni transmite telemetría. Las actualizaciones son totalmente manuales.

## ADR-008 — Identidad por dispositivo físico

Estado: aceptada.

La clave de presentación no será la letra de unidad ni el número de disco mutable. Se usará el número de serie y, cuando falte, una huella calculada con atributos estables, conservando el grado de confianza.

## ADR-009 — Benchmarks basados en archivos

Estado: aceptada.

Las pruebas de escritura no acceden a bloques sin formato. Utilizan un archivo temporal nuevo, limitado y verificable dentro de un volumen montado. Se reservan espacio y umbral térmico, se permite cancelación y se intenta una limpieza segura.

## ADR-010 — Alertas por estado y cambio

Estado: aceptada.

El motor combina límites absolutos con incrementos de contadores y persistencia temporal. Las alertas se agrupan para reducir ruido, pero cada ocurrencia conserva su hora.

## ADR-011 — Versionado como fuente única

Estado: aceptada.

Nombre y versión proceden de los manifiestos del proyecto durante compilación y ejecución. La UI, “Acerca de”, informes e instalador no mantendrán copias manuales independientes.

## ADR-012 — Datos persistentes tras desinstalar

Estado: aceptada.

El desinstalador conserva SQLite, configuración, historial y logs en `ProgramData`. La aplicación ofrece una acción separada y confirmada para eliminarlos, y la documentación explica la limpieza manual.

## ADR-013 — Sistema de diseño v2 vinculante

Estado: aceptada; **enmendada por ADR-034** (2026-09-06): el sistema de diseño evoluciona a v3
(paleta propia «Ciruela», dos escalones tipográficos de «display», riel de navegación). El fondo de
ADR-013 no cambia: `tokens.css` sigue siendo la fuente única de verdad, el catálogo sigue cerrado,
el material de tres capas y los radios concéntricos no se tocan. Lo que cambia es la paleta y que la
herencia del acento de Windows pasa a opción apagada de fábrica (ADR-035).

La interfaz utilizará el paquete de diseño entregado, versión v2 de material translúcido, con su norma vinculante y `tokens.css` como fuente única de verdad visual.

> **Rutas actualizadas por ADR-029.** Cuando se escribió esta decisión, la norma era `Design-system/AGENTS.md` y el paquete vivía sin integrar. Hoy la norma es `docs/ui-design.md` y el sistema de diseño vive en `src/`. El fondo de la decisión no cambia.

Se adoptan el catálogo cerrado de componentes Svelte, los tipos y formateadores entregados, los diccionarios español/inglés, la herencia del acento de Windows y los bocetos v2 aprobados. No se introducirán valores visuales literales, niveles adicionales de material ni bibliotecas de componentes sin una decisión nueva.

Las pantallas todavía no diseñadas —Informes, Ajustes, asistente inicial, Acerca de y systray— se compondrán inicialmente con el catálogo existente y requerirán revisión visual antes de considerarse terminadas.

## ADR-014 — SvelteKit con `adapter-static` y SSR desactivado

Estado: aceptada.

El frontend se monta sobre SvelteKit compilado a estático, sin renderizado en servidor. Es la vía
que Tauri documenta oficialmente y la que el paquete de diseño ya asumía (`$lib` de serie). Aporta
enrutado por ficheros para las siete pantallas sin añadir dependencias de terceros, que es lo que
haría falta con Vite + Svelte puro y que `AGENTS.md` §1 prohíbe.

Consecuencias: hay que desactivar SSR explícitamente y prerrenderizar, porque no existe servidor.
Ninguna carga de datos ocurre en `load`: todo pasa por comandos Tauri desde el cliente.

## ADR-015 — El backend empuja, la interfaz no sondea

Estado: aceptada.

La comunicación de actualizaciones es por eventos Tauri tipados (`metrics:updated`,
`alerts:changed`, `inventory:changed`, `test:progress`, …), no por sondeo periódico desde la
interfaz. El planificador ya sabe cuándo hay datos nuevos; hacer que la UI pregunte cada pocos
segundos duplicaría el reloj, gastaría batería y garantizaría que lo mostrado va siempre un poco
por detrás.

Cada evento trae el objeto completo que ha cambiado, no un parche: la UI reemplaza por
identificador sin reconciliar. Al montar y tras cualquier error de deserialización, la interfaz pide
el estado completo con los comandos `get_*`.

Contrato completo en `docs/ui-contract.md`.

## ADR-016 — Reconocer una alerta no apaga su color

Estado: aceptada.

El estado de salud que se pinta en un disco es la peor severidad de sus alertas `active` **o**
`acknowledged`. Solo `resolved` y `archived` dejan de contar, y el silencio no afecta nunca al color.

La alternativa —que reconocer devolviera el disco a verde— permitiría limpiar el panel, pero a
cambio el color dejaría de ser una señal fiable del estado del hardware, que es justamente para lo
que existe. Se asume que un disco con un problema crónico se quede en rojo: se mitiga con el
distintivo de reconocida y con el orden de la lista, no apagando la señal.

Implementado en `deviceState()` y `alertCountsTowardHealth()`, un único sitio para toda la interfaz.

## ADR-017 — El acento heredado se corrige antes de aplicarse

Estado: aceptada; **matizada por ADR-035** (2026-09-06): la herencia del acento de Windows deja de
ser el comportamiento de fábrica y pasa a un interruptor de Ajustes apagado por defecto. La parte
técnica de ADR-017 se conserva entera: cuando el interruptor está encendido, `accessibleAccent()` y
`accentOnSurface()` siguen corrigiendo el color del usuario para no romper el contraste.

El acento de Windows se hereda, pero no a ciegas: `accessibleAccent()` elige texto blanco o negro
según cuál contraste mejor y, si aun así no se alcanza 4.5:1, oscurece o aclara el acento hasta
lograrlo.

Sin esto, un usuario con acento amarillo, lima o cian claro dejaría el botón primario por debajo de
AA y la aplicación incumpliría su propia norma de accesibilidad. Se prefiere alterar mínimamente el
color elegido por el usuario antes que entregar texto ilegible, y se prefiere eso a renunciar a la
herencia del acento, que es parte de la identidad visual de v2.

## ADR-018 — Tipografía empotrada, no descargada

Estado: aceptada.

Instrument Sans se distribuye como fichero variable dentro de la aplicación y se declara con
`@font-face` local. La versión original de `tokens.css` la importaba de Google Fonts, lo que
contradecía la promesa de cero comunicaciones de red (spec §11, ADR-007) y habría dejado la
aplicación con otra tipografía en cualquier equipo sin salida a Internet, que es precisamente el
tipo de equipo donde se instala un monitor de discos.

Obligación asociada: incluir la SIL Open Font License 1.1 y registrar la fuente en
`THIRD_PARTY_NOTICES.md`. La compilación falla si el fichero no está.

## ADR-019 — El suelo absoluto de capacidad solo se aplica a volúmenes grandes

Estado: aceptada.

Los umbrales de espacio libre son siempre porcentuales (10 % advertencia, 5 % crítico) y, además,
absolutos (20 GB / 10 GB) **solo en volúmenes de 256 GB o más**.

La regla original —el mayor entre porcentaje y valor absoluto, sin condición— marcaba como crítico
un volumen de 64 GB con 15 GB libres, que es casi una cuarta parte del disco. En equipos con
particiones de sistema pequeñas eso sería ruido permanente, y una alerta que siempre está encendida
deja de leerse.

El corte de 256 GB es configurable, porque es el único número de la regla que no se deduce de nada.

## ADR-020 — WebView2 se distribuye con el instalador sin conexión

Estado: aceptada.

`tauri.conf.json` usará `webviewInstallMode: { "type": "offlineInstaller" }`.

### El problema

WebView2 es un requisito de ejecución que no siempre está presente. Solo Windows 11 incluye el
runtime como parte del sistema; en Windows 10 lo tiene la gran mayoría de equipos porque Microsoft
lo desplegó por Windows Update a partir de diciembre de 2022, y **en Windows Server no viene
preinstalado en ninguna versión**. Como Windows Server 2016 a 2025 están dentro del alcance, la
aplicación no arrancaría en un servidor recién instalado.

Los sistemas soportados sí son los que declara la especificación: Microsoft Edge —y con él
WebView2— soporta Windows Server 2016, 2019, 2022 y 2025 en LTSC, y Windows 10 desde 1709. La
matriz de compatibilidad no cambia; lo que cambia es el instalador.

### Las opciones

| Modo | Tamaño añadido | ¿Internet al instalar? | ¿El runtime se parchea solo? |
|---|---|---|---|
| `downloadBootstrapper` (predeterminado) | 0 MB | **sí** | sí |
| `embedBootstrapper` | ~1,8 MB | **sí** | sí |
| `offlineInstaller` | ~127 MB | no | **sí** |
| `fixedRuntime` | ~180 MB | no | **no** |
| `skip` | 0 MB | no | la aplicación no arranca |

### Por qué el instalador sin conexión

Los dos modos de bootstrapper quedan descartados porque exigen conexión durante la instalación, y
un servidor de almacenamiento aislado de Internet es justo uno de los escenarios para los que se
escribe este producto.

Entre las dos opciones sin conexión, la diferencia decisiva no es el tamaño sino **quién parchea el
runtime**. `fixedRuntime` congela una versión concreta de Chromium dentro de la aplicación: dejaría
de recibir parches de seguridad hasta que nosotros publicásemos una versión nueva, y como no hay
actualizador automático (ADR-007) eso significa "hasta que el usuario se entere y descargue a mano".
La aplicación renderiza texto procedente de dispositivos y del registro de eventos de Windows, así
que arrastrar un Chromium sin parchear no es aceptable. `fixedRuntime` añade además tres problemas
prácticos: no funciona desde una ruta de red o UNC, exige conceder permisos con `icacls` a los
contenedores de aplicación en Windows 10 desde la versión 120, y en disco ocupa más de 250 MB.

`offlineInstaller` instala el runtime *Evergreen*: la instalación funciona sin conexión y, a partir
de ahí, Microsoft lo mantiene actualizado por su cuenta sin que nosotros publiquemos nada.

### Consecuencias

- El instalador pasa de unos 10 MB a unos 140 MB. Es un coste asumible en una distribución manual
  por GitHub, y se documenta en la página de descarga.
- El instalador comprueba antes si el runtime ya está presente y solo lo instala si falta, así que
  en un Windows 11 o en un Windows 10 al día no se instala nada.
- Como la aplicación se instala elevada, el runtime queda instalado *por equipo*, que es lo que
  corresponde a una instalación para todos los usuarios.
- Si el runtime faltara igualmente en tiempo de ejecución, la aplicación debe detectarlo y decirlo
  con una frase comprensible, no fallar con una ventana en blanco.

## ADR-021 — smartmontools 7.5, con el fuente dentro del instalador

Estado: aceptada.

Concreta el ADR-005, que dejaba sin fijar la versión y el mecanismo de cumplimiento de la licencia.

### Versión

Se redistribuye **smartmontools 7.5** (publicada el 12 de mayo de 2025, compilación r5714), tomando
el `smartctl.exe` de 64 bits del paquete oficial de Windows. El paquete se llama `win32-setup` por
razones históricas pero contiene las dos arquitecturas; se usa la de `bin/`, verificada como PE
AMD64. Las sumas MD5 se comprueban contra las que publica el propio proyecto.

Se empaqueta también **`drivedb.h`**, la base de datos de unidades. No es opcional: sin ella,
`smartctl` no sabe interpretar los atributos específicos de cada fabricante y los presenta como
desconocidos, que es justo la información que hace útil a un monitor de discos. Queda congelada con
la versión: el script oficial que la actualiza descarga de Internet y no se distribuye, porque
contradiría la promesa de cero comunicaciones de red.

No se empaquetan `smartd` ni sus utilidades: la aplicación ya tiene su propio planificador, y un
segundo vigilante competiría por el acceso a los dispositivos.

### Licencia

`smartctl` es `GPL-2.0-or-later`. **El código propio sigue siendo MIT**: se invoca como proceso
independiente, por línea de órdenes y JSON, sin enlazarlo ni incorporar su código, así que no hay
obra derivada.

La obligación que sí aplica es la de la sección 3 de la GPLv2: quien recibe el binario tiene derecho
al código fuente correspondiente. Se cumple por la vía **3(a)**, acompañar el binario del fuente:
`smartmontools-7.5.tar.gz` viaja dentro del instalador, en `licenses\smartmontools\`.

Se descarta la vía 3(b), la oferta escrita válida tres años, porque obliga a mantener el fuente
disponible y a atender solicitudes durante ese plazo. Un fichero de 1 MB dentro de un instalador de
140 MB cuesta menos y no caduca. La versión del tarball debe coincidir siempre con la del binario,
o el requisito deja de cumplirse.

### Consecuencia operativa comprobada

`--scan-open` funciona sin elevación, pero leer datos de un dispositivo sin privilegios falla con
`exit_status: 1` y el mensaje `Unable to detect device type`. Ese mensaje **no** significa que el
dispositivo sea incompatible: tomarlo al pie de la letra marcaría un equipo entero como "no
compatible" por un problema de privilegios. El colector debe distinguir los dos casos.

## ADR-022 — Zod para validar toda frontera de datos

Estado: aceptada.

### El problema

`invoke<DeviceListResponse>(...)` **no valida nada**. Es una aserción de tipo sobre un dato que
viene de otro proceso: TypeScript se limita a creerse lo que se le dice. Si el backend cambia un
campo, renombra otro o devuelve `null` donde antes había un número, la interfaz lo acepta y muestra
`undefined` como si fuera un dato bueno.

Eso choca de frente con el principio I: *nunca se inventa un valor*. Un `undefined` renderizado es
exactamente un valor inventado, con el agravante de que nadie se entera.

### Por qué aquí importa más que en una aplicación web

Este producto no tiene red, así que la tentación es concluir que no hay datos no confiables. Los
hay, y son peores que los de una API propia:

- la **salida de `smartctl`**, un binario de terceros cuya versión puede cambiar bajo los pies;
- el **registro de eventos de Windows**, escrito por controladores de fabricantes distintos;
- **filas de SQLite** que escribió la propia aplicación hace seis meses con otro esquema.

Ninguno de los tres avisa cuando cambia de forma.

### La decisión

**Zod** como biblioteca única de validación y análisis en TypeScript, con los tipos inferidos de los
esquemas (`z.infer<>`) y no declarados a mano en paralelo. Dos declaraciones de la misma forma
acaban divergiendo; una sola no puede.

Toda respuesta de comando y toda carga útil de evento pasa por su esquema antes de tocar el estado.
Un fallo produce un `AppError` con código `ipc.schema_mismatch` y, en el detalle técnico, la ruta
del campo y lo que se esperaba.

En Rust la simetría no necesita dependencia nueva: tipos serde explícitos en cada frontera externa,
y `serde_json::Value` solo como paso intermedio para conservar una captura en bruto, nunca para
alimentar una decisión.

### Alternativas descartadas

- **Esperar a `ts-rs`.** Genera los tipos desde Rust y evita que front y back diverjan *al compilar*,
  pero no comprueba nada *en ejecución*: no cubre datos de SQLite escritos por una versión anterior,
  que es justo el caso que más preocupa. Ambas cosas son complementarias, no alternativas.
- **Validar solo los eventos.** Los eventos llegan de forma asíncrona y parecen más arriesgados,
  pero una respuesta de `invoke` malformada entra igual al estado. La frontera es la misma.
- **Valibot, ArkType.** Más ligeras, pero Zod es el estándar de hecho del ecosistema y su coste
  —unos pocos KB en una aplicación de escritorio de 140 MB— es irrelevante aquí.

### Coste asumido

Un esquema por cada tipo del contrato, que hay que mantener al día. Se mitiga con la puerta de CI
que exige prueba de rechazo por esquema: si alguien añade un comando sin esquema, el test de
superficie del contrato lo detecta.

## ADR-023 — Sin variables de entorno en la aplicación

Estado: aceptada.

La aplicación se compila con `adapter-static` y `ssr = false`: **no hay servidor**. De ahí se derivan
consecuencias técnicas que conviene dejar escritas antes de que alguien las descubra a base de
depurar:

- `$env/dynamic/private` y `$env/static/private` **no existen** sin un runtime de servidor. No hay
  ficheros `.server.ts` en este proyecto y no puede haberlos.
- `$env/dynamic/public` **no funciona con prerenderizado**. La vía sería `$env/static/public`, que
  requiere enmienda de la constitución.
- `process.env` no existe en el navegador.

**Decisión:** `.env` se usa únicamente para variables de construcción leídas por Node durante el
build (`vite.config.ts`, `svelte.config.js`, `vitest.config.ts`); hoy son las `TAURI_*` que inyecta
la propia herramienta. Ese fichero nunca se empaqueta. La configuración del producto vive en la
tabla `settings` de SQLite (ADR-006), que es la única fuente.

**Sobre secretos:** la aplicación no tiene cuentas, claves ni servicios externos (ADR-007). Y
conviene dejarlo escrito para el futuro: *un `.env` empaquetado en un instalador de escritorio no es
secreto*, cualquiera puede abrir el instalador y leerlo. Si algún día hiciera falta guardar una
credencial, la vía es el almacén de credenciales de Windows (DPAPI), nunca un fichero de texto junto
al ejecutable.

## ADR-024 — `tracing` en Rust y envoltorio propio en TypeScript

Estado: aceptada.

### Qué se descarta y por qué

La propuesta inicial era Pino en el servidor y `loglevel` en el cliente, con el nivel controlado por
`LOG_LEVEL` y `PUBLIC_LOG_LEVEL`. Nada de eso encaja aquí:

- **Pino es una biblioteca de Node.** El «servidor» de esta aplicación es Rust; no hay proceso Node
  en ejecución, solo durante la compilación.
- **Las variables de entorno están prohibidas** por el principio XII, y no hay servidor donde
  vivirían. El nivel va a `settings`, que es donde vive la configuración (principio V).

### Lo que se adopta

**Rust: `tracing`.** Es el estándar de hecho del ecosistema y aporta algo que aquí importa de
verdad: los *spans* permiten seguir una recopilación completa o un benchmark de veinte minutos con
su contexto, sin repetir el identificador en cada llamada. Con `log` + `env_logger` habría que
arrastrarlo a mano por todas partes.

Se acompaña de `tracing-appender` para la rotación diaria y de `time` para el formato de hora.

**TypeScript: envoltorio propio** en `src/lib/logger.ts`, unas cuarenta líneas. No se añade
`loglevel`: son unos 40 KB para lo que aquí resuelven cuarenta líneas, y el envío selectivo por IPC
—que es el requisito real del frontend— habría que escribirlo igual, porque ninguna biblioteca
genérica sabe hablar con Tauri.

### Consecuencias

- Una dependencia nueva en Rust y ninguna en TypeScript.
- El frontend envía a Rust solo `warn` y `error`; `debug` e `info` se quedan en la consola del
  WebView. Enviar todo saturaría el canal y cambiaría el rendimiento que se intenta diagnosticar.
- El nivel se resuelve con la precedencia `--log-level` > `settings` > `info`. El argumento de línea
  de órdenes es la única forma de diagnosticar un fallo **anterior** a poder leer la configuración.
- **La hora se escribe en local con desplazamiento explícito**, no en UTC ni en una zona fija. Esta
  aplicación correlaciona sus métricas con el Visor de eventos de Windows, que muestra hora local:
  un log en otra zona obligaría a convertir mentalmente cada vez que se coteja un pico de
  temperatura con un evento de disco. El desplazamiento evita la ambigüedad cuando el fichero viaja
  por correo.


## ADR-025 — Instancia única con el plugin oficial de Tauri

Estado: aceptada.

### El problema

La especificación no pide solo impedir una segunda instancia: pide que **abrir una segunda restaure
la ventana de la primera** (`docs/product-specification.md` §11, `docs/architecture.md` §4). Son dos
requisitos distintos, y el segundo obliga a comunicar los dos procesos.

### La decisión

`tauri-plugin-single-instance` 2.4, del propio equipo de Tauri, registrado **el primero** de todos
los plugins: se ejecutan en el orden en que se añaden al `Builder`, y este tiene que decidir si el
proceso sigue vivo antes de que nada más se inicialice.

Su devolución de llamada corre en el proceso que ya estaba en marcha y recibe los argumentos y el
directorio de trabajo del segundo, que termina solo. Ahí se llama a
`platform::ventana::restaurar_ventana_principal()`, que desminimiza, muestra y enfoca **en ese
orden**: una ventana minimizada sigue contando como visible, y `set_focus()` sobre una ventana
oculta no hace nada.

### Alternativas descartadas

- **Mutex con nombre (`CreateMutexW`) sin dependencias.** Quince líneas y cero superficie añadida,
  pero solo resuelve la mitad: la segunda instancia muere en silencio y el usuario, que no ve
  aparecer nada, concluye que la aplicación no arranca. Restaurar la primera ventana exigiría
  escribir igualmente el canal entre procesos, que es exactamente lo que aporta el plugin.
- **Fichero de bloqueo en `%ProgramData%`.** Sobrevive a un cierre inesperado y deja la aplicación
  inarrancable hasta que alguien lo borra a mano. Un mutex del núcleo desaparece con el proceso.

### Consecuencias

- Una dependencia más en un binario privilegiado. Se acepta porque es oficial, está en el mismo
  espacio de versiones que Tauri y su alternativa exigiría escribir el mismo mecanismo peor.
- El nivel de registro del segundo proceso **no se aplica**: el suscriptor de `tracing` ya está
  instalado con el nivel del primero. Los argumentos del segundo se registran en el log, que es lo
  útil para diagnosticar; cambiar el nivel en caliente requeriría un `reload::Handle` y no compensa.
- La restauración de ventana queda en un único sitio, compartida con el arranque normal.

## ADR-026 — ACL explícita y toma de propiedad de la carpeta de `ProgramData`

Estado: aceptada.

### El problema

La aplicación guarda en `%ProgramData%\SmartDisk Monitor\` la base SQLite, los logs y los informes.
La suposición de partida era que `ProgramData` ya restringe la escritura a administradores. **Es
falsa**, y se ha medido en un Windows 11 real (`docs/open-questions.md` §R):

```text
C:\ProgramData  BUILTIN\Usuarios:(CI)(WD,AD,WEA,WA)
                CREATOR OWNER:(OI)(CI)(IO)(F)
```

`(CI)` propaga a toda subcarpeta. Un usuario **sin privilegios** puede crear
`C:\ProgramData\SmartDisk Monitor\` antes de que se instale nada y, por `CREATOR OWNER`, queda con
Control total sobre ella. Se comprobó ejecutándolo desde una sesión no elevada.

### La decisión

El instalador, y solo el instalador, crea la carpeta y le aplica:

```text
icacls "%ProgramData%\SmartDisk Monitor" /setowner *S-1-5-32-544 /t /c
icacls "%ProgramData%\SmartDisk Monitor" /inheritance:r ^
  /grant:r *S-1-5-18:(OI)(CI)F ^
  /grant:r *S-1-5-32-544:(OI)(CI)F ^
  /grant:r *S-1-5-32-545:(OI)(CI)RX
```

Tres detalles que no son opcionales:

1. **`/setowner` primero.** Restablecer la ACL no basta: el propietario conserva `WRITE_DAC`
   implícito y vuelve a concederse Control total en silencio. Medido: tras endurecer la carpeta, el
   usuario que la había creado recuperó la escritura con un solo `icacls /grant`.
2. **SID numéricos, no nombres.** En esta máquina el grupo se llama `Administradores`; en un Windows
   en inglés, `Administrators`. Un instalador que use nombres falla en la mitad del planeta.
   `S-1-5-18` es `SYSTEM`, `S-1-5-32-544` administradores, `S-1-5-32-545` usuarios.
3. **`/inheritance:r` y sin `CREATOR OWNER`.** Sin cortar la herencia, los permisos de `ProgramData`
   siguen aplicándose por debajo de los explícitos.

`platform::paths::log_dir()` deja de crear la raíz en compilación de publicación: si falta, la
instalación está rota y debe notarse, no repararse creando una carpeta con la ACL heredada débil.

### Alternativas descartadas

- **Comprobar y reparar la ACL al arrancar.** La aplicación va elevada y podría hacerlo, pero exige
  el crate `windows` con `Win32_Security` y código `unsafe` para leer descriptores de seguridad,
  para cubrir un hueco que el instalador ya cierra por completo: después de instalar, nadie sin
  privilegios puede cambiar esos permisos. Se descarta por coste frente a beneficio.
- **Usar `%LocalAppData%` por usuario.** Evitaría el problema, pero rompe el requisito de que el
  historial sea del equipo y no de la cuenta que abrió la aplicación.

### Consecuencias

- El instalador gana un paso obligatorio y verificable, que forma parte de los criterios de US-060.
- Un usuario sin privilegios puede seguir **leyendo** la carpeta. Es deliberado: la interfaz muestra
  informes y el ZIP de diagnóstico se genera ahí. Los datos ya se anonimizan por defecto y ningún
  número de serie ni ruta de perfil entra en un log.
- Desinstalar conserva `ProgramData` (US-060), así que la ACL endurecida sobrevive a la
  desinstalación y una reinstalación se la vuelve a encontrar. `/setowner` la deja consistente
  igualmente.

## ADR-027 — Vitest 5 con Browser Mode, en dos configuraciones separadas

Estado: aceptada.

### El problema

El sistema de diseño es vinculante y su incumplimiento es un defecto de producto, no estético. Aun
así había **25 componentes y cero pruebas de componente**, y `@testing-library/svelte` instalado sin
un solo uso. Faltaba el nivel entero.

El proyecto estaba en Vitest 2.1.9, donde el Browser Mode es experimental y su configuración
(`browser.name`) fue **sustituida** en la 3 por `browser.instances` con proveedores en paquetes
aparte. Montarlo sobre la 2.1.9 era escribir una configuración ya retirada.

### La decisión

**Vitest 5.0.0**, con `@vitest/browser` + `@vitest/browser-playwright` y `vitest-browser-svelte`.
Se retira `@testing-library/svelte`. Momento elegido a propósito: 165 pruebas de lógica pura y
ningún producto encima es lo más barato que va a estar nunca.

**Dos configuraciones, no una** (`docs/testing-strategy.md` §24):

| Configuración | Entorno | Incluye | Medido |
|---|---|---|---|
| `vitest.config.ts` | jsdom | `src/**/*.test.ts` menos las de navegador | 165 pruebas, ~16 s |
| `vitest.browser.config.ts` | Chromium | `src/**/*.browser.test.ts` | 8 pruebas, ~2 s |

`pnpm test` sigue siendo la suite rápida que se teclea mientras se programa, y Stryker apunta solo a
la de Node: mutar código lanza la suite cientos de veces y abrir un navegador en cada una la haría
inviable.

**El sufijo es `*.browser.test.ts`, no `*.svelte.test.ts`** como proponía la primera versión de la
estrategia. Ese sufijo ya estaba tomado por las pruebas de los módulos `.svelte.ts` con runas
—`theme.svelte.test.ts`, `app.svelte.test.ts`—, que corren en Node. El discriminante real es el
entorno de ejecución, no el tipo de fichero.

La zona horaria de la suite se fija en `Europe/Madrid`, no en UTC: es la que tiene cambio de hora, y
ahí es donde aparecen los fallos de retención, de enfriamiento y de correlación con el Visor de
eventos, que muestra hora local.

### Alternativas descartadas

- **Quedarse en Vitest 2.1.9** fijando `@vitest/browser@2.1.9` y `vitest-browser-svelte@1.1.0`.
  Funciona hoy, pero se escribiría con la API ya retirada, para reescribirla al actualizar y ya con
  producto encima. Y el Browser Mode de la 2 lo marcaba experimental su propio equipo.
- **Vitest 3.2.7** como salto intermedio. Ecosistema más asentado, pero deja dos mayores de deuda.
- **jsdom para los componentes.** Es lo que hay que evitar: el contraste sobre material compuesto,
  el respaldo a `--sdm-solid`, la resolución de variables en tema oscuro y la visibilidad del foco
  **no son observables en jsdom**. Se comprobó en la práctica: los dos defectos que destapó esta
  infraestructura habrían pasado en verde con jsdom.
- **Un proyecto único con `projects`.** Mezclaría los tiempos y obligaría a Stryker a filtrar.

### Consecuencias

- Cuatro dependencias de desarrollo nuevas y una retirada. Ninguna entra en el binario.
- `tsconfig.json` redefine `include`: al hacerlo **sustituye** al de SvelteKit y sus rutas pasan a
  ser relativas a la raíz. Se repite su contenido y se añade `e2e/`. Sin eso, ESLint analiza las
  pruebas sin tipos y las reglas que los necesitan quedan mudas justo donde más falta hacen.
- `vitest-browser-svelte` 3.1.0 se publicó el mismo día en que se instaló y salta la política de
  antigüedad mínima de `pnpm`. Se comprobó a mano contra la 3.0.0: el `dist/` es **byte a byte
  idéntico** y solo cambia el rango de pares. La excepción queda razonada en `pnpm-workspace.yaml`.

## ADR-028 — Plano de interfaz con Playwright y un IPC propio

Estado: aceptada.

### El problema

Playwright no puede conducir una ventana de Tauri, pero sí puede conducir la interfaz: es la misma
aplicación sobre el mismo motor, Chromium. Lo que falta es el backend.

### La decisión

Playwright contra `pnpm preview` —el build, no `vite dev`: es donde aparecen los problemas que el
servidor de desarrollo esconde— con un doble de IPC instalado por `addInitScript`.

**No se usa `mockIPC()` de `@tauri-apps/api/mocks`**: esa función se ejecuta en el proceso de Node y
aquí el doble tiene que existir **dentro de la página**, antes del primer `invoke`. `e2e/ui/ipc-falso.ts`
hace lo mismo que ella —poblar `window.__TAURI_INTERNALS__`— pero en un script de inicialización, y
además registra las llamadas para poder afirmar sobre ellas.

Los fixtures se validan contra **los mismos esquemas Zod** que usa la aplicación. No es ceremonia:
en la primera ejecución rechazaron tres valores inventados (`certain`, `no_smart_support`, `wmi`)
que no existen en el contrato. Sin esa validación, las pruebas habrían pasado en verde probando una
forma de datos que no existe.

Solo Chromium. El WebView2 de la aplicación es Chromium; probar en Firefox o WebKit mediría un motor
que ningún usuario va a ejecutar.

### Consecuencias

- 17 pruebas de interfaz, ~38 s incluyendo compilación y arranque del servidor de vista previa.
- La accesibilidad se comprueba con `axe` en las seis pantallas **por ambos temas**, doce
  combinaciones. Encontró un incumplimiento real en la primera ejecución.
- El plano de aplicación real (`tauri-driver`) sigue pendiente y es otra cosa: exige el ejecutable
  empaquetado y terminal elevada. Entra con US-060.

## ADR-029 — Una sola copia del sistema de diseño, y su norma en `docs/`

Estado: aceptada. Reemplaza las rutas de ADR-013, cuyo fondo sigue vigente.

### El problema

El paquete del diseñador llegó como `Design-system/` y se integró en el árbol de la aplicación
(`src/design-system/` y `src/lib/`). La copia original se conservó "como referencia". El resultado
fueron **dos sistemas de diseño vivos a la vez**, y divergieron:

| Fichero | Divergencia |
|---|---|
| `tokens.css` | el arreglo de foco `:focus-visible:focus-visible` (WCAG 2.4.7) solo llegó a la copia de `src/` |
| `design/accent.ts` | 23 líneas distintas |
| `design/format.ts` | 14 líneas distintas |
| `design/health.ts` | 6 líneas distintas |
| `design/types.ts`, `i18n/es.json`, `i18n/en.json` | 3 líneas distintas cada uno |

Lo grave no es la divergencia, sino **quién la leía**: `tools/build-historias.py` generaba el
consolidado desde la copia del paquete. `historias.md` —el documento que se entrega entero a un
agente de IA o a quien se incorpora al proyecto— estuvo publicando los tokens sin el arreglo de
foco. Un agente que se fiara del consolidado habría reintroducido un fallo de accesibilidad ya
resuelto, y todas las puertas de calidad habrían pasado en verde, porque ninguna miraba ahí.

La prueba de que el coste era real y ya se estaba pagando: existía `tools/_nav.py`, un script cuyo
único cometido era aplicar cada cambio **dos veces**, una en cada copia. Se ha eliminado con esta
decisión.

Había además un problema de nombres. La norma de interfaz se llamaba `Design-system/AGENTS.md` y
convivía con el `AGENTS.md` de la raíz, que es la guía general para agentes de IA. Dos ficheros con
el mismo nombre y significados distintos: `AGENTS.md` de la raíz tenía que dedicar un párrafo a
avisar de la confusión.

### La decisión

**Una sola copia, dentro de `src/`.** La carpeta `Design-system/` desaparece:

| Qué era | Dónde está ahora |
|---|---|
| `Design-system/AGENTS.md` | `docs/ui-design.md` |
| `Design-system/HANDOFF.md` | absorbido en los apéndices A–C de `docs/ui-design.md` |
| `Design-system/design-system/**` | ya estaba en `src/design-system/`; la copia se borra |
| `Design-system/src/**` | ya estaba en `src/lib/`; la copia se borra |
| `Design-system/tailwind.config.cjs` | ya estaba en la raíz; la copia se borra |
| Los tres `.dc.html`, `support.js` | `design/`, con su propio `README.md` |

La norma pasa a llamarse **`docs/ui-design.md`**: elimina la colisión de nombres, la coloca junto al
resto de documentos normativos y hace pareja con `docs/ui-contract.md` —aquel dice qué puede pedir
la interfaz, este cómo se pinta lo que recibe—. Gana además un **§0 «Dónde vive cada cosa»**, que es
lo que antes no existía en ningún sitio: un agente tenía que deducir las rutas.

Los bocetos se conservan porque no viven en ningún otro sitio; el resto no, porque duplicar para
"conservar la referencia" es precisamente lo que causó el problema. **El paquete original íntegro
sigue en el historial de git**, que es donde va lo que se conserva por trazabilidad.

**Y se hace determinista.** `pnpm verify:tokens` gana una comprobación que falla la integración si
aparece un segundo `tokens.css`, un segundo `tokens.json` o un segundo barrel de componentes fuera
de `src/`. Un principio que solo vive en un documento dura hasta el primer día de prisa.

### Consecuencias

- `historias.md` se genera desde las rutas vivas y suma dos fuentes que faltaban: `tokens.json` y
  `src/lib/components/index.ts`, que es el catálogo real y ejecutable. Pasa de 29 a 31 ficheros.
- Los ficheros de agentes (`AGENTS.md`, `CLAUDE.md`, y los nuevos `GEMINI.md` y `CODEX.md`) llevan
  el mapa de rutas y las órdenes duras de interfaz. Ninguno duplica la norma: apuntan a ella.
- `AGENTS.md` recupera el párrafo que gastaba en avisar de la colisión de nombres.
- Ningún valor de token, umbral ni regla visual cambia. Es reorganización, no rediseño.

## ADR-030 — El backend no manda texto de alerta; solo `ruleKey`, y el frontend lo resuelve por i18n

Estado: aceptada.

### El problema

Al conectar el motor de alertas (Historia 2, spec `001-monitor-discos-windows`) con los comandos
reales, el contrato existente de `AlertGroup` (`src/lib/design/types.ts`) exigía `title` y
`summary` como **cadenas ya resueltas**, y `AlertCard.svelte` las pintaba directamente. El
componente además tenía literales en español escritos a mano (`"Informativa"`, `"reconocida"`…),
en violación de la regla de cero-literales-de-interfaz.

Rellenar `title`/`summary` desde Rust exigía generar texto en español directamente en el backend
—el propio principio VI lo prohíbe— o cambiar el contrato para mandar una clave que el frontend
resuelva. Es un cambio de contrato, y por tanto exige esta decisión antes de programarlo
(`AGENTS.md`, límites duros).

### La decisión

El backend deja de mandar `title` y `summary`. `get_alert_groups` y `get_alert_detail` mandan
`ruleKey` (ya lo hacían) y nada más de texto libre; `AlertCard.svelte` resuelve el título y el
resumen con `t(\`alert.rule.${ruleKey}.title\`)` / `.summary`, una clave por cada `rule_key` del
catálogo implementado (`docs/alert-rules.md` §2, subconjunto de `open-questions.md` J.16). El
estado (`activa`/`reconocida`/…) también deja de estar hardcodeado: nuevas claves
`alert.status.*`.

`target` se conserva como campo del backend: no es texto de interfaz traducible, es el alias o
modelo del disco (dato del usuario, no una frase de la aplicación), igual que `disk.alias ??
disk.model` en el resto de la interfaz.

De paso se corrige un desajuste de cable independiente: `AlertSeverity` serializaba
`"warning"`/`"critical"`, pero el esquema Zod de `severity` (compartido con el vocabulario general
de `Severity`) espera `"warn"`/`"crit"`. Se corrige la serialización de cable de `AlertSeverity`
sin tocar el almacenamiento en SQLite (`alert_groups.severity` sigue guardando `warning`/`critical`,
que es lo que exige el `CHECK` de la migración): son dos representaciones distintas del mismo dato,
como ya ocurre con `DeviceType`.

### Consecuencias

- `src/lib/design/types.ts`: `AlertGroup` pierde `title` y `summary`.
- `src/lib/api/schemas.ts`: el esquema `alertGroup` pierde esos dos campos.
- `AlertCard.svelte` deja de tener literales de interfaz; sus dos diccionarios ganan las claves
  `alert.rule.<rule_key>.title/summary` (una por regla implementada) y `alert.status.*`.
- Ningún componente que ya estuviera consumiendo `title`/`summary` queda roto: el único consumidor
  era este mismo componente, corregido en el mismo cambio.

## ADR-031 — `tauri-plugin-dialog` para elegir el destino de una exportación

Estado: aceptada.

### El problema

`export_report`, `preview_diagnostic_zip` y `create_diagnostic_zip` (`docs/ui-contract.md` §3.7)
necesitan una ruta de destino que hoy nadie puede producir: `capabilities/default.json` solo
declara `core:default`, sin ningún permiso de acceso al sistema de archivos ni de diálogo. Aceptar
`destinationPath` como una cadena libre construida por la interfaz sería justo la segunda vía de
acceso al sistema de ficheros que el principio IX prohíbe — el mismo motivo por el que
`open_log_folder` no recibe una ruta como argumento.

### La decisión

`tauri-plugin-dialog` 2.7 (equipo de Tauri), con un único permiso concedido:
`dialog:allow-save` (no el conjunto `dialog:default`, que además habilita `allow-open` y
`allow-message`, innecesarios aquí — mínimo privilegio real, no solo declarado). El usuario elige
carpeta y nombre con el selector nativo de Windows desde `$lib/api`; el backend solo escribe en la
ruta que ese diálogo devuelve, nunca en una construida por la interfaz.

### Alternativas descartadas

- **Carpeta fija sin diálogo** (como la carpeta controlada del benchmark, T077/J.27): evita la
  dependencia y el permiso nuevos, pero un informe o un ZIP de diagnóstico está pensado para
  salir del equipo — adjuntarlo a un correo, subirlo a un ticket de soporte—, y forzarlo siempre a
  la misma carpeta interna contradice ese uso. Se ofreció como alternativa real (`AskUserQuestion`)
  y el usuario prefirió el diálogo nativo.

### Consecuencias

- Dependencia nueva en un binario privilegiado (`tauri-plugin-dialog` + su equivalente JS
  `@tauri-apps/plugin-dialog`) y permiso nuevo en `capabilities/default.json`. Se acepta por ser
  oficial del equipo de Tauri, la UX esperada de cualquier «Guardar como» de la plataforma, y por
  decisión explícita del usuario.
- `$lib/api` gana el envoltorio de `save()` del plugin; ninguna pantalla lo llama directamente
  (mismo criterio que el resto de la frontera IPC).

## ADR-032 — El crate `zip` para el paquete de diagnóstico

Estado: aceptada.

### El problema

T090 necesita empaquetar varios ficheros (ajustes, eventos exportados, capturas SMART brutas,
registro de actividad) en un único ZIP de diagnóstico (US-051). No había ninguna dependencia de
compresión en `Cargo.toml`.

### La decisión

`zip` 2.4 (`zip-rs/zip2`, mantenido, muy usado), con `default-features = false` y la sola
característica `deflate` (arrastra a su vez `deflate-flate2` y `deflate-zopfli`: la variante solo
`flate2` no compila sin depender también de una de las dos, así que se acepta `zopfli` en el árbol
en vez de pelear con la selección de características). Sin `aes-crypto`, `bzip2`, `lzma`, `zstd`,
`xz` ni `chrono`: el ZIP de diagnóstico no necesita cifrado ni otros algoritmos de compresión, y
cada uno de esos suma dependencias transitivas propias.

### Alternativas descartadas

- **Escritor ZIP propio, sin comprimir** (mismo criterio que el LCG de T079 frente a `rand`):
  descartado porque el formato ZIP tiene más superficie de la que parece a primera vista —cabeceras
  local y central, CRC32, el registro de fin de directorio central— y un error ahí no falla alto:
  produce un ZIP que algunos lectores abren mal y otros no, que es peor que no tener la función. El
  LCG de T079 era mucho más simple (un generador congruencial lineal, no un formato de contenedor
  con implicaciones de compatibilidad). Se ofreció como alternativa real (`AskUserQuestion`) y el
  usuario prefirió el crate.

### Consecuencias

- Dependencia nueva en un binario privilegiado, con `zopfli` como dependencia transitiva
  (algoritmo de compresión, sin superficie de seguridad relevante: no toca red ni entrada externa
  sin confiar, solo comprime bytes ya generados por la propia aplicación).
- El ZIP de diagnóstico admite compresión real (no solo `stored`), lo que mantiene manejable el
  tamaño del registro de actividad incluido por FR-029c.

## ADR-033 — El planificador en segundo plano es un hilo bloqueante que sondea cada 1 s, sin `tokio`

Estado: aceptada.

### El problema

T020/T021/T022 necesitaban un bucle real que ejecutara la recopilación (SMART, contadores de
rendimiento, eventos de Windows, altas/bajas de inventario) sin que la interfaz tuviera que pedirlo:
hasta esta historia solo existía `refresh_now`, un comando manual. Cada uno de los cuatro trabajos
tiene su propia cadencia configurable (`collectors::planificador`), que además se reduce en batería
para los que no alimentan alertas graves (FR-030), y debe reaccionar a la pausa manual, a un cambio
de ajuste de frecuencia y al cierre real de la aplicación sin quedarse colgado.

### La decisión

Un único hilo bloqueante (`tauri::async_runtime::spawn_blocking`, lanzado en `.setup()`), con un
bucle que **sondea cada 1 segundo** (`open-questions.md` J.34) en vez de dormir el intervalo
completo del próximo trabajo: en cada sondeo comprueba la señal de parada, si está pausado, y para
cada uno de los cuatro trabajos si ya toca ejecutarse (`collectors::planificador::trabajos_debidos`,
función pura, probada con tiempo inyectado). Cada trabajo debido se ejecuta de forma independiente
—el fallo de uno no bloquea a los demás (`open-questions.md` J.39)— y el post-proceso común
(notificaciones, `alerts:changed`, `metrics:updated`, `inventory:changed`, `source:degraded`, icono
de bandeja) se comparte con `refresh_now` a través de `post_procesar_ciclo`. Todos los colectores son
E/S síncrona (procesos, SQLite, FFI de Windows), nunca futuros, así que no hace falta ningún runtime
asíncrono nuevo: `tauri::async_runtime::spawn_blocking` ya viene con Tauri, cero dependencias nuevas.
El apagado limpio pasa por `RunEvent::Exit`/`ExitRequested` (`lib.rs`, vía `.build().run(|_,event|
...)` en vez de `.run(...)` directo), que marca un `Arc<AtomicBool>` en `AppState` comprobado en
cada sondeo — un único punto de parada, sea cual sea la vía de salida real.

### Alternativas descartadas

- **Un temporizador (`tokio::time::interval` o similar) por trabajo**: exigiría añadir `tokio` como
  dependencia directa (hoy solo llega transitivamente vía Tauri, y no para Windows) y sincronizar
  cuatro relojes independientes contra pausa, batería y ajustes que cambian en caliente —más
  complejidad para el mismo resultado que un sondeo de 1 s ya da con cuatro comparaciones de
  `Instant`.
- **Interceptar cada `app.exit(0)` por separado** para señalizar la parada: descartado porque hay
  al menos dos puntos de salida (cierre real de ventana, "Salir" de la bandeja) y cualquier futuro
  tercero se olvidaría con facilidad; `RunEvent::Exit`/`ExitRequested` es el único punto que Tauri
  garantiza que se dispara siempre, venga de donde venga la salida.

### Consecuencias

- La aplicación monitoriza de verdad sin intervención manual (T022 queda satisfecho además "gratis":
  `AppState.paused` nunca se persiste, así que todo arranque empieza activo).
- `SourceHealth` pasa de estar sin tipar (`status: String`) a un enum `SourceStatus` real
  (`ok`/`partial`/`unsupported`/`timeout`/`error`), aunque esta historia solo produce `ok`/`timeout`/
  `error` (`open-questions.md` J.37); `partial`/`unsupported` quedan para cuando alguien los pida.
- `refresh_smart` se separó en `refresh_smart` (solo SMART) y `refresh_metricas_rendimiento` (solo
  PDH): antes estaban acopladas porque nada las llamaba con cadencias distintas
  (`open-questions.md` J.38).
- `metrics:updated.historyWriteHalted` refleja un cálculo real de espacio libre, pero **no** detiene
  todavía ninguna escritura (`open-questions.md` J.40): queda como seguimiento explícito, no como
  olvido.

## ADR-034 — Sistema de diseño v3: «escena de datos» con paleta propia «Ciruela»

Estado: aceptada. Fecha: 2026-09-06. Enmienda ADR-013. Spec: `specs/002-rediseno-v3/`.

### El problema

Las capturas de la interfaz v2 sobre un Windows real (`design/entregable-rediseno/salida/capturas/`)
mostraron tres defectos de presentación que no son de implementación:

1. El panel general parece a medio cargar: con dos discos la rejilla ocupa ~230 px y deja ~570 px de
   lienzo vacío.
2. Cada magnitud es texto plano del mismo tamaño y color; nada dice si «41 °C» está bien, y un dato
   ausente pesa más que un dato presente.
3. El acento azul heredado de Windows (`#0067c0`) es correcto pero indistinguible de cualquier
   utilidad del sistema; en tema oscuro el conjunto queda gris plano.

El diseñador entregó una propuesta (`design/propuesta-redisenov2/`) que los resuelve sin reescribir
el sistema.

### La decisión

El sistema de diseño evoluciona a **v3**. `docs/ui-design.md` pasa a describir v3. Cambios:

- **Paleta «Ciruela»**: neutros malva, acento morado de tinta (`#7a3f9d` claro / `#c79aec` oscuro) y
  crítico desplazado al bermellón (`#b03434` / `#ef8080`) para no confundirse con el acento. Todos
  los ratios de contraste están medidos sobre el material compuesto en `docs/open-questions.md`.
- **`--sdm-on-accent` deja de ser blanco en tema oscuro**: el acento oscuro es claro y el texto
  blanco encima daba 2,27:1. Pasa a tinta (`#20132a`, 7,80:1). Todo texto sobre el acento usa
  `--sdm-on-accent`, nunca `text-white`.
- **Familia de «display»** (`--sdm-font-display`) y dos escalones nuevos (58 px, 76 px) para cifras y
  titulares (nunca texto corrido), mediante la clase `.sdm-display`. **No se empaqueta una segunda
  familia tipográfica**: `--sdm-font-display` resuelve a la familia sans ya empotrada. Se descartó Bricolage
  Grotesque para no ampliar la superficie de un binario privilegiado que se distribuye a terceros
  (constitución §III); si se revisa, `.sdm-display` y su `@font-face` son el único punto de cambio.
- **Tres componentes nuevos** en el catálogo, cada uno con su justificación contra `ui-design.md` §3:
  `Icon` (juego propio de 15 iconos de línea que heredan `currentColor`), `Sparkline` (trazo sin
  ejes) y `HeroPanel` (dato dominante del panel). Ningún componente se elimina.
- **La `Sidebar` pasa a un riel de 74 px** solo con iconos, devolviendo 176 px de ancho al contenido
  (crítico a 1024 px). La lista de discos sale de la barra (ya está en la rejilla del panel).

### Qué NO cambia

El material de tres capas y sus desenfoques (28 / 24 / 44), la escala de radios concéntricos
(18 → 13 → 9 → cápsula), el movimiento (220 ms, `cubic-bezier(.32,.72,0,1)`, `active:scale-[0.98]`,
`prefers-reduced-motion`), el catálogo cerrado, `tokens.css` como fuente única de verdad visual, y
**todas** las reglas de producto. No se añade ningún comando Tauri ni ningún permiso: el rediseño es
de presentación, salvo los cambios de frontera acotados que la spec 002 documenta (una preferencia
de apariencia cuyo campo ya existía, la marca del asistente inicial y los umbrales de perfil de
alerta).

### Alternativas descartadas

- **Mantener el azul de Windows y ofrecer Ciruela como tema alternativo**: duplica el mantenimiento
  de dos identidades y deja sin resolver el problema original (la aplicación no se reconoce) para la
  mayoría de usuarios, que no cambian de tema.
- **Empaquetar Bricolage Grotesque**: aporta carácter a las cifras grandes pero obliga a gestionar
  un binario y una licencia OFL más en un instalador privilegiado. El coste no compensa; se deja la
  puerta abierta con un único punto de cambio.

### Consecuencias

- `docs/ui-design.md` §0, §2, §2.bis y §3 se reescriben para v3. §4 (composición), §6
  (accesibilidad) y §8 (definición de terminado) se conservan literalmente y siguen siendo el
  criterio de aceptación visual de cada pantalla.
- El catálogo pasa de N a N+3 componentes.
- La entrega se hace en nueve PR ordenados por dependencia (`specs/002-rediseno-v3/plan.md`).

## ADR-035 — La herencia del acento de Windows pasa a opción apagada de fábrica

Estado: aceptada. Fecha: 2026-09-06. Matiza ADR-017. Spec: `specs/002-rediseno-v3/`.

### El problema

ADR-013 adoptó «la herencia del acento de Windows» como parte de la identidad visual de v2, y
ADR-017 construyó toda la corrección de contraste (`accessibleAccent()`, `accentOnSurface()`, el
barrido de los 262.144 acentos posibles de `open-questions.md` §O) sobre esa premisa. Pero heredar
el acento del sistema es justo lo que hace que la aplicación no se distinga de cualquier utilidad de
Windows (ADR-034, problema 3).

### La decisión

Con la paleta Ciruela, el acento propio es el comportamiento **de fábrica**. Heredar el acento de
Windows pasa a un interruptor en Ajustes → Apariencia, **apagado por defecto**
(`settings.appearance.useSystemAccent`, cuyo campo ya existía en el backend; solo cambia su valor de
fábrica de `true` a `false`, implantado en PR 8 de `specs/002-rediseno-v3/`). Al encenderlo,
`applySystemAccent()` sobrescribe los **tres roles de acento** —`--sdm-accent` (fondo),
`--sdm-accent-fg` (texto), `--sdm-on-accent` (texto sobre el fondo)— más sus dos derivados
(`--sdm-accent-hi`, `--sdm-accent-soft`): cinco propiedades CSS en total. Al apagarlo,
`clearSystemAccent()` las restaura todas.

**La parte técnica de ADR-017 se conserva entera**: cuando el interruptor está encendido, el acento
del usuario sigue pasando por `accessibleAccent()` / `accentOnSurface()` para no bajar de AA en
ningún tema. El acento sigue sin comunicar salud y sigue siendo acción/selección: no se toca ningún
principio de la constitución §VI, solo se precisa que la herencia es opcional. La viñeta del acento
de §VI se ajustó en consecuencia (constitución 1.7.0, 2026-09-06).

### Alternativas descartadas

- **Quitar la herencia por completo**: se pierde valor para el usuario que prefiere integrarse con
  su sistema, y se tira una inversión de trabajo (ADR-017, `open-questions.md` §O) que ya está hecha
  y probada.
- **Dejar la herencia encendida de fábrica y Ciruela como respaldo**: no resuelve el problema para
  la mayoría, que no toca los ajustes de apariencia.

### Consecuencias

- El texto de la preferencia (`settings.appearance.useSystemAccent.label` / `.hint`) se reescribe:
  hoy asume el comportamiento contrario.
- La definición de terminado de `ui-design.md` §8 sigue exigiendo verificar cada pantalla con un
  acento del sistema claro y en los dos temas: el camino de la herencia no se abandona, se hace
  opcional.

## ADR-036 — El motor de alertas se parametriza por perfil

Estado: aceptada. Fecha: 2026-09-06. Spec: `specs/002-rediseno-v3/` (US10). Amplía `alert-rules.md` §2.

### El problema

El rediseño v3 añade un paso al asistente inicial y una sección a Ajustes para elegir «cuánto avisa»
la aplicación con un perfil (Prudente / Equilibrado / Solo lo grave). Para que esa elección no sea
decorativa —y la constitución §I exige que la interfaz no mienta sobre qué está activo— el motor de
alertas tiene que **consumir de verdad** los umbrales.

Hasta ahora no lo hacía: `alerts::motor` es puro y sus umbrales estaban **escritos a mano**
(`90/100` para desgaste, `70/80` para temperatura). `settings.alerts.temp_configured_warn_c` se
guardaba y `get_settings` lo devolvía, pero **ninguna regla lo leía** — el mismo hueco que
`open-questions.md` J.32 describía para las claves de capacidad.

### La decisión

1. **`settings.alerts` gana siete claves**: `profile` (`cautious`|`balanced`|`quiet`|`custom`),
   `wear_warn_percent`, `wear_crit_percent`, `media_errors_warn_per24h`, `media_errors_crit_per24h`,
   `driver_retry_warn_per24h`, `driver_retry_crit_per24h`. Cada una con su rango, su validación
   (`crit` más severo que `warn`) y su prueba de rechazo (constitución §XI).
2. **El motor las lee**. `alerts::motor` sigue puro: recibe los umbrales como parámetros;
   `commands::refresh_smart` los resuelve de `settings` una vez por ciclo y se los pasa. Reglas
   afectadas:
   - `smart.wear_high` → `wear_warn_percent` / `wear_crit_percent`.
   - `temp.above_configured_warn/crit` → `temp_configured_warn_c` / `_crit_c`. La histéresis de
     resolución conserva su margen (aviso − 3 °C, crítico − 5 °C).
   - `smart.media_errors` → la activación pasa de «el contador aumentó» a «el **incremento** entre
     dos lecturas alcanza `media_errors_warn/crit_per24h`». El sufijo `Per24h` es histórico: **no**
     es una ventana de 24 h (spec 002, clarify Q1). `smart.error_log` no se parametriza.
   - **`capacity.low` / `capacity.critical`**: este ADR las **implementa en el motor** (antes solo
     figuraban en `alert-rules.md`, sin código). Requiere persistir `volume_free_bytes` como muestra
     periódica de cada volumen — antes la capacidad solo vivía como instantánea en
     `volumes.free_bytes`. `domain::capacidad::estado_capacidad` es el espejo Rust de
     `capacityState()` de `src/lib/design/health.ts`.
3. **El umbral térmico de fábrica baja a 60/70 °C** (era 70/80), que es el valor del perfil
   Equilibrado. Decisión de producto adoptada (spec 002, clarify Q2): 60 °C sigue siendo temperatura
   alta para un SSD de consumo y mantener dos números («fábrica» vs «Equilibrado») confundiría.
4. **Elegir un perfil escribe sus doce umbrales de golpe** y guarda el identificador. **Editar a
   mano cualquiera de esos umbrales** pone `profile = "custom"`; la única forma de volver a un perfil
   concreto es elegirlo. La interfaz muestra «Personalizado (a partir de \<perfil anterior\>)».

### Lo que queda fuera

- **`driver_retry_warn/crit_per24h` se guardan pero ninguna regla los consume todavía**: las reglas
  `events.controller_reset` / `events.io_retry` necesitan el colector de eventos completo (Historia
  4). El perfil escribe los doce valores igualmente, así el día que exista esa regla ya tiene su
  umbral — el mismo patrón con el que las claves de capacidad y `logging.verbose` vivieron guardadas
  sin consumidor (J.32, FR-029a). Registrado en `docs/open-questions.md`.
- No se añade ningún comando Tauri ni ningún permiso: todo pasa por el `set_setting` genérico.

### Alternativas descartadas

- **Guardar los perfiles pero no cablear el motor** (opción del planteamiento inicial): el paso 3 del
  asistente y la sección de Ajustes serían decoración. El usuario eligió el alcance completo.
- **Implementar una ventana de conteo real «por 24 h»** para errores de medios y reintentos: mucho
  más código en el motor (mecánica de conteo nueva + sus cinco pruebas) para un matiz que la
  reinterpretación sobre las reglas existentes ya cubre (clarify Q1).

### Consecuencias

- `alert-rules.md` §2: la tabla pasa a decir «valor configurado (`settings.alerts.*`)» donde antes
  ponía `> 70 °C` / `≥ 90` / «aumenta»; `capacity.low`/`critical` dejan de estar pendientes.
- `metric_samples` gana un tipo de muestra: `volume_free_bytes` con `MetricTarget::Volume`. La
  retención lo compacta igual que el resto (misma columna `volume_id` del esquema).
- `VolumeSummary` gana `is_system_volume` (necesario para `selectHeroDisk()`, spec 002 clarify Q3),
  calculado al leer con `GetSystemWindowsDirectoryW`, sin migración de esquema.

## ADR-037 — Apagar las notificaciones es un ajuste propio, no solo pausar

Estado: aceptada. Fecha: 2026-09-06. Spec: `specs/002-rediseno-v3/` (US8, paso 3 del asistente).

### El problema

El paso 3 del asistente inicial (`cambios/08-onboarding.md`) ofrece un `Switch` «Avisarme con una
notificación de Windows». Hasta v3 el toast nativo estaba **siempre activo** cuando la ventana estaba
minimizada (`product-specification.md` §Notificaciones); lo único que se podía apagar era el
**sonido** (`notifications.sound_enabled`). Sonido ≠ presencia: alguien puede querer el aviso sin el
«ding», y también puede querer ningún aviso emergente sin tener que **pausar toda la recopilación**
(que es lo que hoy silencia las notificaciones, y de paso deja de vigilar los discos).

### La decisión

Clave nueva `notifications.enabled` (booleano, **fábrica: `true`**). La consume
`alerts::notificaciones::procesar_una`: si está en `false`, no se muestra el toast, con independencia
de la transición de la alerta. La alerta **sigue existiendo** en la lista y sigue contando para el
color de salud — apagar el aviso emergente no apaga la vigilancia (constitución §I). Se persiste con
el `set_setting` genérico; no añade comando ni permiso. La decisión de enviar se factoriza a una
función pura `debe_enviar(...)` con sus pruebas (el resto de `procesar_una` necesita un proceso Tauri
real, `research.md` R1).

### Alternativas descartadas

- **Reutilizar `sound_enabled`**: cambia la semántica de una clave existente y confunde («sin sonido»
  no es «sin aviso»).
- **Depender de pausar**: pausar es una acción temporal y global; no es una preferencia de «no quiero
  ventanas emergentes».

## ADR-038 — El autoarranque es una tarea programada, no una entrada `Run`

Estado: aceptada. Fecha: 2026-09-06. Spec: `specs/002-rediseno-v3/` (US8, paso 3 del asistente).

### El problema

El paso 3 ofrece «Arrancar SmartDisk con el sistema». La aplicación corre bajo
`requireAdministrator` (manifiesto, UAC al abrir). Una entrada en
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run` la lanzaría con el **token sin elevar** en cada
inicio de sesión: Windows mostraría un diálogo de UAC en cada login, o el arranque fallaría en
silencio. El plugin `tauri-plugin-autostart` usa exactamente esa clave `Run` (y además sería una
dependencia nueva).

### La decisión

Clave nueva `lifecycle.start_with_system` (booleano, **fábrica: `false`**). Al activarla,
`platform::autoarranque::aplicar(true)` registra una **tarea programada** —
`schtasks.exe /Create /TN "SmartDisk Monitor - Autostart" /SC ONLOGON /RL HIGHEST`— que el
Programador de tareas eleva **sin diálogo**. Al desactivarla, `/Delete`. `reset_settings` con
`scope: "all"` también borra la tarea. Solo se mira el **código de salida** de `schtasks`: la
codificación de su salida de texto no es fiable entre configuraciones de Windows
(`.claude/rules/backend-rust.md`), así que no se parsea `stdout`. El comando real no se ejecuta en
`cargo test` (crearía una tarea en el equipo del desarrollador): se prueba el formato de la línea de
comando y se verifica a mano, igual que J.28 y `platform/sistema.rs`.

### Alternativas descartadas

- **Clave `Run` de HKCU** (y `tauri-plugin-autostart`, que la usa): UAC en cada login por la
  elevación; y el plugin es dependencia nueva sin justificación (límite duro de `AGENTS.md`).
- **Carpeta «Inicio» del menú**: mismo problema de elevación que `Run`.

## ADR-039 — Ilustraciones del asistente inicial: SVG propio con tokens, no recurso empaquetado

Estado: aceptada. Fecha: 2026-09-06. Extiende `docs/ui-design.md` §3 (catálogo). Origen: revisión
de las capturas del asistente sobre un Windows real.

### El problema

Los cuatro pasos del asistente inicial (US-002) tenían huecos donde el diseño pedía una figura y
solo había un cuadrado de `Icon` diminuto —cuando llegaba a verse: el sprite se montaba solo en
`AppShell`, que esta ruta no usa (corregido aparte: `IconSprite` en `+layout.svelte`)—. Un icono de
18 px no llena una pantalla de bienvenida a ancho completo. El asistente es la primera impresión del
producto y quedaba pobre.

### La decisión

Un componente nuevo, **`OnboardingArt`**, con **cuatro escenas** (`welcome`, `disks`, `alerts`,
`done`), una por paso. Son **SVG en línea escritos a mano**, planas, estilo «Corporate Memphis /
Alegría» adaptado a la «escena de datos» de v3: formas geométricas rotundas, **sin figuras
humanas** (el motivo es siempre el hardware y su vigilancia — y así no hace falta un tono de piel,
que no es un token).

Reglas que cumple, como cualquier pieza del catálogo:

- **Solo `currentColor` y `var(--sdm-*)`.** Ni un color literal; lo verifica `pnpm verify:tokens`.
  Por eso funciona en tema claro y oscuro **sin una sola condicional**. Las coordenadas y los
  `stroke-width` del dibujo son geometría, no valores de tema (misma consideración que el sprite de
  `IconSprite`).
- **Decorativa**: sale `aria-hidden`, sin nombre accesible. El texto de cada paso ya lo dice todo;
  un `role="img"` sin contenido informativo sería peor (`ui-design.md` §6).
- La paleta del dibujo es sobre todo **acento + neutro**; el verde `ok` solo aparece donde refuerza
  el mensaje real del producto (el latido de «constantes vitales» del paso 1, el sello de
  conformidad del paso 4). El acento **no** codifica salud aquí: es identidad visual.
- Exportada en el barrel; `width` como única prop (el alto sale de la proporción 8:5).

### Alternativas descartadas

- **Empaquetar PNG/SVG generados con una herramienta de ilustración**: un binario más en un
  instalador privilegiado, con su hash que mantener (constitución §III y §IX), y un recurso que no
  reacciona al tema — habría que entregar dos juegos (claro/oscuro) y conmutarlos. El SVG con
  tokens se adapta solo.
- **Seguir con cuadros de `Icon`**: no es una ilustración, es un pictograma; no llena la pantalla
  ni da carácter a la primera impresión.
- **Ilustración con personajes al estilo Corporate Memphis puro**: obliga a decidir tonos de piel
  sin un token que los represente, y desentona con una aplicación de sistema. Se conserva el
  lenguaje de formas, no las figuras.

### Consecuencias

- El catálogo suma `OnboardingArt`. Su uso está acotado al asistente; no es un patrón general de
  «mete una ilustración donde quieras».
- `docs/ui-design.md` §3 lo recoge y §7.6 (asistente) menciona la escena por paso.
- Si en el futuro otra pantalla quiere una ilustración, se decide entonces con el criterio de
  `ui-design.md` §3, no por analogía con esta.

## ADR-040 — La geometría de la ventana se recuerda en `settings`, no con un plugin

Estado: aceptada. Fecha: 2026-09-06.

### El problema

La ventana principal nacía siempre con el tamaño fijo de `tauri.conf.json` (1360 × 880, centrada).
Se quiere que la primera vez abra a **1695 × 988** y que, a partir de ahí, recuerde entre sesiones
el **tamaño, la posición y si estaba maximizada**.

### La decisión

El estado de la ventana se persiste en la tabla **`settings`** de SQLite, en cinco claves
internas (`window.width`, `window.height`, `window.x`, `window.y`, `window.maximized`), en
**píxeles lógicos** (independientes del escalado de Windows, igual que `tauri.conf.json`).

- **El frontend no participa.** No hay comando nuevo ni evento nuevo. Todo ocurre en Rust:
  `platform::ventana::aplicar_geometria_guardada` se llama en `.setup()` **antes** de mostrar la
  ventana (que nace `visible: false`, así que no hay salto), y `persistir_geometria` se llama al
  cerrar (`CloseRequested`) y al salir (`RunEvent::ExitRequested`, que cubre «Salir» de la bandeja
  y el apagado).
- **Ausente ⇒ valor de fábrica.** Sin filas `window.*` manda `tauri.conf.json`: el primer
  arranque abre a 1695 × 988 centrada. `reset_settings` (ámbito «resto» o «all») borra las claves.
- **Maximizada**: se guarda solo `window.maximized = true` y no se tocan tamaño/posición, para que
  al restaurar y quitar la maximización la ventana vuelva al tamaño que el usuario había elegido.
- **Posición fuera de pantalla**: `aplicar_geometria_guardada` comprueba con `geometria_visible`
  (función pura, con pruebas) que la barra de título cae dentro de algún monitor actual con margen;
  si el monitor donde estaba se ha desconectado, se ignora la posición y la ventana abre centrada.
- **Cuándo se guarda**: al pulsar la X (aunque esa X minimice a la bandeja) y en `ExitRequested`.
  Una muerte dura del proceso (Administrador de tareas) pierde el último movimiento; es aceptable
  (constitución §II.5, simplicidad antes que generalidad) y evita un temporizador de *debounce*
  sobre `Resized`/`Moved`.

### Alternativas descartadas

- **`tauri-plugin-window-state`** (el estándar de Tauri para esto): guarda su estado en un fichero
  JSON propio, fuera de SQLite — choca de frente con el principio **V** (INNEGOCIABLE): «`localStorage`
  está prohibido para estado del producto… Todo se almacena en SQLite… Ningún otro almacén de datos
  estructurados». Además, añadirlo sería **dependencia nueva** (enmienda de la constitución, §III) y
  traería **permisos de Tauri nuevos** (su ADR). La vía `settings` no necesita nada de eso.
- **`localStorage` + redimensionar al arrancar desde el frontend**: mismo choque con el principio V,
  y produce un salto visible (la ventana ya está pintada cuando se redimensiona), y el
  almacenamiento del WebView es frágil ante limpiezas.
- **Debounce sobre `Resized`/`Moved`**: más robusto ante una muerte dura, pero necesita un
  temporizador y escribe en SQLite durante el arrastre. No compensa para el caso que se da.

### Consecuencias

- `docs/data-model.md` §2 documenta las cinco claves `window.*`.
- `docs/open-questions.md` J.8 y `docs/ui-design.md` §4.0 pasan la predeterminada a 1695 × 988; el
  **objetivo de diseño** (1280 × 720) y el **mínimo técnico** (1024 × 560) no cambian.
- Sin contrato nuevo, sin DTO `ts-rs`, sin esquema Zod, sin permiso de Tauri, sin dependencia.

## ADR-041 — `DiskSummary` lleva la autoevaluación SMART (`smartHealthPassed`)

Estado: aceptada. Fecha: 2026-09-06.

### El problema

El `HeroPanel` del panel general muestra cuatro «hechos» del disco protagonista. El boceto aprobado
(`design/propuesta-rediseno`, `smartdisk-v3.html`) pone como primero **«Salud del firmware ·
Correcta»**; la implementación mostraba en su lugar **«Ocupación · N %»** (porcentaje ocupado del
volumen principal). La autoevaluación SMART (`smart_status.passed`) sí se recopila —se persiste como
métrica `health_passed` (1.0/0.0) y se expone en `DeviceDetail.counters`—, pero **no viaja en
`DiskSummary`**, que es el único DTO que reciben el `HeroPanel` y la `DiskCard`. La clave i18n
`disk.firmwareHealth` ya existía en los dos diccionarios, dejada preparada.

### La decisión

Añadir a `DiskSummary` el campo `smart_health_passed: Option<bool>` (`smartHealthPassed` en el
wire): `Some(true)` autoevaluación superada, `Some(false)` fallida, `None` sin dato o disco sin
SMART. Lo rellena `enrich_with_smart_data` leyendo la última muestra de `health_passed`, igual que
ya lee temperatura, desgaste y horas. El `HeroPanel` sustituye el hecho «Ocupación» por «Salud del
firmware» (`icon: shield`, en rojo solo si `false`); el orden de hechos pasa a ser el del boceto:
firmware, desgaste, actividad, horas.

### Alternativas descartadas

- **Mantener «Ocupación» y aceptar la desviación del boceto.** Coste cero (nada de backend). Se
  descarta porque el boceto es la referencia vinculante (`ADR-034`, `docs/ui-design.md` §0), la
  ocupación de un volumen ya la comunica la barra de capacidad de cada `DiskCard`, y el trabajo del
  Hero es «¿tengo un problema?» —donde «el disco ha fallado su propia autoevaluación» encaja y «el
  disco está lleno al 93 %» ya tiene su alerta y su barra—.
- **Derivarlo en el frontend de `disk.state`.** No sirve: `state` refleja alertas y frescura, no el
  resultado del autotest. Un disco puede tener `state = ok` y `health_passed = false` en el mismo
  ciclo en que se está creando la alerta `smart.health.failed`.
- **Reusar `provenance` o un campo existente.** Opaco y frágil; un booleano nuevo es más honesto.

### Consecuencias

- Un campo anulable más en un DTO que `ts-rs` ya refleja: regenera `generated/DiskSummary.ts` y
  `generated/DeviceDetail.ts` al compilar. `DeviceDetail` lo hereda por `#[serde(flatten)]` —
  inofensivo, ya tenía el mismo dato en `counters`.
- Esquema Zod (`schemas.ts`) y su prueba de rechazo; interfaz en `src/lib/design/types.ts`;
  `docs/ui-contract.md` §3.2.
- Sin comando nuevo, sin permiso de Tauri, sin dependencia.

## ADR-042 — Los colectores no retienen el mutex de la conexión durante la E/S externa

Estado: aceptada. Fecha: 2026-09-07.

### El problema

`AppState` tiene una única `Mutex<Connection>`. El bucle de recopilación en segundo plano
(`iniciar_planificador` → `ejecutar_ciclo`) tomaba ese candado y, **con el candado en la mano**,
lanzaba la E/S externa lenta de cada colector:

- `refresh_smart`: por cada disco, una cascada de hasta 5 modos de `smartctl.exe` × 15 s de límite
  = **hasta 75 s por disco**.
- `refresh_metricas_rendimiento`: `perf_counters::leer` **duerme 1 s** entre las dos muestras PDH
  que exige calcular una tasa → ≥ N s con N discos.
- `refresh_events`: la lectura del registro de eventos por FFI (`wevtapi.dll`), segundos con
  backlog grande.

Cualquier comando de consulta de la interfaz (`get_system_events`, `get_alert_groups`,
`get_devices`, `set_setting`…) hace `conn.lock()` y se quedaba esperando todo ese tiempo. El
usuario lo vivía como **congelación de varios segundos al cambiar de sección** en el sidebar: la
navegación de SvelteKit espera al `load` de la ruta, el `load` espera al comando, y el comando
espera al candado. `docs/architecture.md` §4 ya decía «la UI y los colectores no comparten
operaciones bloqueantes» y la constitución §V exige «transacciones breves»: el código lo incumplía.

### La decisión

Reestructurar `refresh_smart`, `refresh_metricas_rendimiento` y `refresh_events` en **tres fases**:

1. **Planificar** — candado breve: leer los dispositivos y la configuración necesarios y construir
   un plan de trabajo.
2. **Recopilar** — **sin candado**: toda la E/S externa (subprocesos, PDH, FFI).
3. **Persistir** — candado único: escribir muestras, registrar fallos y evaluar alertas.

Los orquestadores reciben `&Mutex<…>` (no una guarda) y toman y sueltan el candado ellos mismos;
`conn` y `source_health` nunca se anidan. Una guarda `AppState.recoleccion_smart: Mutex<()>`
conserva la semántica de «el refresco manual espera al ciclo en curso» sin retener `conn`.
`refresh_inventory` ya cumplía (su E/S por PowerShell corre antes del candado) y no se toca.

### Alternativas descartadas

- **Una segunda `Connection` de solo lectura en `AppState`.** La interfaz leería por su conexión
  mientras el colector escribe por la suya. Se descarta: introduce `SQLITE_BUSY` real entre las dos
  conexiones (que hoy no existe con una sola), obliga a manejar reintentos, y **no arregla el lado
  escritor** —`set_setting`, `acknowledge_alert` y demás comandos que escriben seguirían detrás del
  candado del colector—. Queda como posible mejora futura independiente para las lecturas pesadas
  de informes.
- **Bajar el límite de la cascada de `smartctl`.** Reduce el síntoma, no la causa: con dos discos
  lentos se vuelve a notar, y perder modos de sondeo deja discos sin leer.

### Consecuencias

- El bucle de fondo y un `refresh_now` manual pueden **solapar sus subprocesos `smartctl` en la
  fase 2**. Sin corrupción —filas nuevas, borrado lógico de `devices`, evaluación de alertas
  serializada en la fase 3— y la guarda `recoleccion_smart` lo evita del todo.
- Hay un desfase entre el `ahora` de la fase 1 y la escritura de la fase 3; ya ocurría antes y la
  fase 3 dura milisegundos.
- La asimetría de contabilidad de `smartctl` (fallo de consulta suma intento y fallo; JSON
  inválido suma solo intento; fallo de persistencia no toca contadores) se traslada intacta y
  ahora está cubierta por pruebas de `smart_planificar` y `smart_persistir`.
- Sin comando nuevo, sin permiso de Tauri, sin dependencia. `.claude/rules/backend-rust.md` recoge
  la trampa.
