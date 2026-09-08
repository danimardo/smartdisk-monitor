---
paths:
  - "src-tauri/**/*.rs"
  - "src-tauri/Cargo.toml"
  - "src-tauri/tauri.conf.json"
---

# Backend

- **`domain/` no conoce Tauri, ni Windows, ni SQLite.** Recibe datos y devuelve decisiones: es lo
  que permite probar el motor de alertas con fixtures y sin hardware.
- **Los comandos son funciones finas**: validan argumentos y delegan. La lógica no vive ahí.
- **La lista de comandos de `lib.rs` es cerrada.** Lo que no esté registrado no es alcanzable desde
  la interfaz, y eso es deliberado: no hay shell genérica ni `fs` abierto.
- **Todo error que cruce a la interfaz es un `AppError`** con código estable, clave i18n, detalle
  técnico sin traducir y si merece reintento.
- **Registro con `tracing`, nunca `println!`.** Y ningún número de serie, nombre de equipo, ruta de
  perfil ni etiqueta de volumen entra en un log: **el log viaja dentro del ZIP de diagnóstico**.
- **`serde_json::Value` no es un tipo de dominio.** Sirve para conservar una captura en bruto,
  jamás para alimentar una decisión.
- **Toda fuente externa se deserializa en tipos explícitos**: salida de `smartctl`, XML de eventos,
  filas de SQLite y valores del registro de Windows. Ninguna avisa cuando cambia de forma.
- **Rutas calculadas y validadas canónicamente** antes de crear o borrar nada. Nunca se sobrescribe
  un archivo existente.

## Trampas ya pisadas

Están documentadas porque volver a descubrirlas cuesta horas:

- El registro guarda el acento en **ABGR**, no en RGB. Leerlo mal convierte el azul de fábrica en
  naranja.
- La salida de `chkdsk` llega en **CP1252**, y la de `fsutil` en CP850. Las herramientas de Windows
  no coinciden entre sí: la codificación se detecta, no se supone.
- `smartctl` sin elevación enumera dispositivos pero falla al leerlos con `Unable to detect device
  type`, que **no** significa que el disco sea incompatible.
- `main.rs` lleva `test = false`: hereda el manifiesto de elevación y su arnés de pruebas no puede
  arrancar sin privilegios.
- `%ProgramData%` **no** restringe la escritura a administradores: concede a `Usuarios`
  `(CI)(WD,AD,WEA,WA)`, heredado a toda subcarpeta. La raíz de datos la crea el instalador con su
  ACL explícita; crearla desde Rust reproduce el agujero (ADR-026).
- Restablecer una ACL **sin tomar antes la propiedad** no cierra nada: el propietario conserva
  `WRITE_DAC` y se vuelve a conceder Control total.
- Git Bash convierte los modificadores de `icacls` en rutas: `/grant` acaba como
  `C:/Program Files/Git/grant`. Estos comandos se lanzan desde PowerShell.
- `windows/app.manifest` **no puede llevar comentarios `<!-- -->` con tildes/eñes** (probado:
  falla incluso sin la declaración `<?xml ?>`, y funciona en cuanto se quitan los comentarios).
  `tauri-winres` 0.3.6 empaqueta el manifiesto línea a línea (`res.set_manifest`,
  `writeln!(f, "\" {} \"", ...)`  en `tauri-winres/src/lib.rs`); con texto no ASCII dentro de un
  comentario, el recurso `RT_MANIFEST` que produce el compilador de recursos queda con una
  secuencia de bytes que Windows rechaza al arrancar: `os error 14001`
  (`ERROR_SXS_CANT_GEN_ACTCTX`, "sintaxis XML no válida en la línea 1" en el registro de eventos,
  proveedor `SideBySide`) — un error que no menciona el manifiesto por su nombre y hace perder
  tiempo. Diagnosticado bisecando: el manifiesto de Tauri por defecto (sin comentarios, ASCII)
  arranca bien con la misma técnica de empaquetado; uno con la misma estructura pero comentarios en
  español con tildes reproduce el fallo, y el mismo texto sin comentarios (o en ASCII puro) no.
  Si hace falta explicar algo del manifiesto, la explicación va en este fichero o en `build.rs`
  (comentarios Rust normales, fuera del recurso empaquetado), nunca dentro del XML.
- `windows/app.manifest` **necesita declarar la dependencia de Common Controls v6**
  (`Microsoft.Windows.Common-Controls`, `version="6.0.0.0"`, `publicKeyToken="6595b64144ccf1df"`
  — el mismo bloque `<dependency>` que trae el manifiesto por defecto de `tauri-build`). Sin ella,
  la aplicación compila y arranca hasta cargar todas las DLL del sistema, y muere de golpe con
  `STATUS_ENTRYPOINT_NOT_FOUND` (`0xc0000139`) sin ningún otro rastro en Process Monitor — el
  registro de eventos no dice nada útil, pero lanzando el `.exe` directamente desde el Explorador
  (nunca desde `cargo run`, que se traga el diálogo) Windows sí muestra el nombre exacto:
  `TaskDialogIndirect`, una función que **solo existe en Common Controls v6** y que algo en la pila
  de Tauri (el propio WRY/diálogo nativo) necesita. Era un fallo preexistente al principio de esta
  sesión, nunca detectado porque nadie había llegado a ejecutar `pnpm app:dev` hasta el final: el
  manifiesto llevaba `trustInfo`/`compatibility`/`application` pero le faltaba justo esta
  dependencia. Si `pnpm app:dev` deja de arrancar sin más pista que un código de salida, comprobar
  primero que este bloque sigue en el manifiesto antes de sospechar de la caché de compilación o
  del antivirus (ambos se descartaron primero, sin necesidad, en esta misma investigación).

- **Nunca se hace E/S externa con el candado de `AppState.conn` tomado.** Hay una sola
  `Mutex<Connection>`: si un colector lanza `smartctl.exe` (cascada de hasta 75 s/disco), duerme
  entre muestras PDH o entra a `wevtapi` con el candado en la mano, cualquier comando de la interfaz
  que haga `conn.lock()` se cuelga ese mismo tiempo y la navegación de la interfaz se congela. Los
  colectores van en tres fases: candado breve para planificar → E/S sin candado → candado único
  para persistir. Los orquestadores reciben `&Mutex<…>`, no una guarda, y no anidan `conn` con
  `source_health`. La guarda `recoleccion_smart` serializa ciclo y refresco manual sin retener
  `conn` (ADR-042).
- Un `Child` con `stdout`/`stderr` en `Stdio::piped()` **se cuelga si nadie vacía esos pipes
  mientras el proceso sigue vivo** (J.55): el búfer del pipe que da el sistema operativo es
  limitado, y en cuanto el hijo lo llena se bloquea en su propio `write()` esperando a que alguien
  lea — si el padre solo llama a `try_wait()` en bucle y deja `read_to_end()` para después de que
  la salida se confirme, ese "después" no llega nunca. Se disparaba con discos SATA cuya tabla de
  atributos completa (10-13 KB medidos en esta máquina) superaba el búfer; un NVMe con salida más
  corta (7 KB) nunca lo mostraba, lo que lo hizo parecer un problema específico de SATA hasta
  medirlo contra hardware real. Reproducido sin necesidad de `smartctl.exe`: cualquier proceso
  hijo que escriba lo bastante (`cmd /c "for /L %i in (…) do @echo …"` en la prueba) se cuelga
  igual con el patrón viejo. Resuelto de una vez para todo el backend en
  `platform::proceso_externo::ejecutar_con_limite`: vacía los pipes en hilos aparte mientras el
  hilo principal solo vigila si el proceso ha terminado, nunca los lee "cuando termine". Cualquier
  proceso externo nuevo debe pasar por esta función, no reinventar `Command::output()` a mano.
- **`Command::output()` no tiene límite de tiempo** (J.57): si el proceso se cuelga o tarda mucho
  (un WMI lento a inicializar, típico justo tras instalar o nada más arrancar Windows), el hilo que
  llama espera para siempre. `windows_storage::list_physical_disks` y `capacidad::list_volumes` lo
  sufrían así desde el principio — nunca se había medido contra una instalación recién hecha, que
  es justo cuando WMI está más frío. Mismo arreglo que el punto anterior:
  `platform::proceso_externo::ejecutar_con_limite` con un límite explícito (20 s para las consultas
  de inventario) en vez de `Command::output()`.
- **Un `Command::new(...)` sin `CREATE_NO_WINDOW` hace parpadear una ventana de consola visible**
  (J.57), aunque el proceso termine en milisegundos y aunque se le pida `-WindowStyle Hidden`
  (PowerShell): Windows le asigna una consola nueva al lanzar el proceso, antes de que el propio
  proceso decida nada sobre su estilo — ese modificador llega demasiado tarde para evitar la
  ventana en sí, solo evita que *PowerShell* muestre la suya si fuera a crear una. `platform::proceso_externo::ejecutar_con_limite`
  lo aplica siempre (`std::os::windows::process::CommandExt::creation_flags(0x0800_0000)`); si
  algún día hace falta lanzar un proceso sin pasar por esa función, hay que poner el flag a mano
  (ya lo hacía `platform::autoarranque` para `schtasks`, antes de que existiera un sitio común).

Al terminar: `cargo clippy --all-targets -- -D warnings` y `cargo fmt --check` en verde.
