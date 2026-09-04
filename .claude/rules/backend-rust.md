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

Al terminar: `cargo clippy --all-targets -- -D warnings` y `cargo fmt --check` en verde.
