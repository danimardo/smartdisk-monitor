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

Estado: aceptada.

La interfaz utilizará el paquete `Design-system/`, versión v2 de material translúcido. `Design-system/AGENTS.md` constituye la norma vinculante y `tokens.css` la fuente única de verdad visual.

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

Estado: aceptada.

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
