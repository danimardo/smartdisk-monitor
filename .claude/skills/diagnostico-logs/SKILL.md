---
name: diagnostico-logs
description: Dónde están los logs de esta aplicación, cómo activar el modo detallado y qué nunca puede copiarse de ellos a una respuesta, un commit o un documento. Úsala al investigar un fallo en ejecución, cuando el usuario diga "no funciona", "se ha colgado", "mira el log" o "por qué falla", y antes de pegar cualquier fragmento de log en una respuesta.
---

# Diagnóstico con logs

## Dónde están

La ruta **cambia según cómo se ejecute**, y es el error más común al buscarlos:

| Modo | Ruta |
|---|---|
| Desarrollo (`pnpm app:dev`, `cargo test`) | `.dev-data/logs/` en la raíz del repositorio |
| Instalada | `%ProgramData%\SmartDisk Monitor\logs\` |

Los ficheros rotan a diario: `smartdisk.log.AAAA-MM-DD`.

En desarrollo, `.dev-data/` está en `.gitignore`. Si no existe la carpeta, es que la aplicación no
ha llegado a arrancar: eso ya es información.

## Cómo leerlos

```sh
# Última ejecución
tail -50 .dev-data/logs/smartdisk.log.$(date +%Y-%m-%d)

# Solo errores y avisos
grep -E " (ERROR|WARN) " .dev-data/logs/smartdisk.log.*

# Lo que emitió la interfaz (llega por IPC con target `ui`)
grep " ui: " .dev-data/logs/smartdisk.log.*
```

Formato de una entrada:

```text
2026-09-04 15:12:35,344 +02:00  INFO smartdisk_lib::logging: registro iniciado nivel="debug"
```

La hora es **local con desplazamiento explícito**, a propósito: así se puede cotejar directamente
con el Visor de eventos de Windows, que también muestra hora local. Si aparece un aviso de que no
se pudo determinar la zona, las horas van en UTC y hay que tenerlo en cuenta.

## Modo detallado

```sh
./src-tauri/target/debug/smartdisk-monitor.exe --log-level=debug
```

Niveles: `trace`, `debug`, `info` (por defecto), `warn`, `error`, `silent`. Un valor no reconocido
**no** activa el modo detallado: avisa y usa el predeterminado.

El argumento prevalece sobre `settings` y es la única forma de diagnosticar un fallo que ocurre
**antes** de que la configuración sea legible, que es justo cuando más falta hace.

## Qué NO se copia de un log

Nunca a una respuesta, un commit, un documento ni un issue:

- números de serie de dispositivos;
- nombre del equipo o del usuario;
- rutas que contengan un perfil de usuario;
- etiquetas de volumen, que a menudo llevan nombres propios.

Si necesitas citar una línea que los contenga, **sustitúyelos** por un marcador antes de pegarla.

Regla de fondo: si algo no puede salir en un ZIP de diagnóstico, tampoco puede salir de un log —
porque **el log va dentro del ZIP**.

## Si el log está vacío o no existe

En ese orden:

1. ¿Arrancó la aplicación? Un fallo anterior al registro no deja rastro en el fichero, pero sí en
   la salida estándar de la consola.
2. ¿Se está mirando la ruta correcta? Desarrollo e instalación usan carpetas distintas.
3. Reproduce con `--log-level=trace` y vuelve a mirar.

## Antes de dar por buena una hipótesis

Un log dice qué pasó, no por qué. Si la explicación depende de una suposición sobre el
comportamiento de Windows, de `smartctl` o del WebView2, **compruébala** antes de escribirla en la
documentación. Cuatro suposiciones de la especificación original resultaron falsas al medirlas.
