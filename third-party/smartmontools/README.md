# smartmontools (smartctl) — binario redistribuido

Fuente principal de datos SMART y NVMe de SmartDisk Monitor (ADR-005). Se ejecuta como **proceso
independiente**, nunca enlazado, lo que mantiene el código propio bajo licencia MIT
(véase *Licencia*, más abajo).

## Versión

| | |
|---|---|
| Versión | **smartmontools 7.5** |
| Fecha de publicación | 12 de mayo de 2025 (compilación del 30 de abril de 2025, r5714) |
| Origen | `smartmontools-7.5.win32-setup.exe` de las publicaciones oficiales del proyecto |
| Identificación que reporta | `smartctl 7.5 2025-04-30 r5714 [x86_64-w64-mingw32-w11-b26200]` |

El paquete oficial se llama `win32-setup` por motivos históricos, pero **contiene ambas
arquitecturas**: `bin/` es x64 y `bin32/` es x86. Aquí se redistribuye el de `bin/`, verificado como
PE de máquina `0x8664` (AMD64).

## Qué se incluye y por qué

| Fichero | Tamaño | Para qué |
|---|---|---|
| `bin/smartctl.exe` | 1.165.312 B | El programa. Lo único que ejecuta la aplicación |
| `bin/drivedb.h` | 267.943 B | **Base de datos de unidades.** Sin ella, los atributos específicos de cada fabricante se muestran como desconocidos en vez de con su nombre y su interpretación |
| `licenses/COPYING.txt` | | GPL versión 2, íntegra y sin modificar |
| `licenses/AUTHORS.txt`, `NEWS.txt` | | Avisos de autoría que la licencia obliga a conservar |
| `source/smartmontools-7.5.tar.gz` | 1.122.317 B | El código fuente correspondiente. Es lo que satisface la obligación de la GPLv2 |

### Qué se deja fuera, deliberadamente

- `smartd.exe` y toda su configuración: es el demonio de vigilancia del propio smartmontools. La
  aplicación hace ese trabajo con su propio planificador, y arrancar un segundo vigilante sería
  duplicar la función y competir por el acceso a los dispositivos.
- `update-smart-drivedb.ps1`: actualiza `drivedb.h` **descargándola de Internet**. Incompatible con
  la promesa de cero comunicaciones de red (spec §11, ADR-007). La base de datos se actualiza
  cambiando de versión de smartmontools, no en caliente.
- `smartd_mailer.ps1`, `wtssendmsg.exe`, `runcmda.exe`, `runcmdu.exe`: utilidades de notificación de
  `smartd`. No se usan y ampliarían la superficie de la instalación sin motivo.
- Los binarios de 32 bits de `bin32/`: la plataforma es x64.

### `smartctl.exe` frente a `smartctl-nc.exe`

El paquete oficial trae también `smartctl-nc.exe`, idéntico pero enlazado como aplicación de
ventanas para que no aparezca una consola al invocarlo. **Se usa `smartctl.exe`**, el canónico: el
parpadeo de consola se evita desde Rust lanzando el proceso con la bandera `CREATE_NO_WINDOW`
(`0x08000000`), que es la solución correcta y no depende de qué binario se empaquete.

## Verificación de integridad

Sumas MD5 comprobadas contra `doc/checksums64.txt` del propio paquete oficial y contra los ficheros
`.md5` publicados junto a la descarga:

| Fichero | MD5 | SHA-256 |
|---|---|---|
| `bin/smartctl.exe` | `c1d1d016a33b09014517636c61d30947` | `b5db94e5082c042be44994b7a4fa8f7b5c8e713b2ab1c9a560d8f7a7995ea27d` |
| `bin/drivedb.h` | | `dd39c6a520d38895da61923fe26fe7c9c5eb3f42325f2e4477a06ed7a61966d0` |
| `source/smartmontools-7.5.tar.gz` | `38c38b0b82db7fc4906cdd50d15a7931` | `690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e` |

La compilación debe verificar estos hashes antes de empaquetar. Un binario privilegiado que se
distribuye a terceros no se copia a ciegas.

## Licencia y obligaciones

`SPDX-License-Identifier: GPL-2.0-or-later`. Copyright (C) 2002-2011 Bruce Allen,
2008-2025 Christian Franke, 2000 Michael Cornwell y otros; véase `licenses/AUTHORS.txt`.

**La licencia propia no se ve afectada.** SmartDisk Monitor invoca `smartctl.exe` como proceso
separado, comunicándose por línea de órdenes y JSON por la salida estándar. No lo enlaza ni
incorpora su código, así que no hay obra derivada y el código propio sigue siendo MIT.

**La obligación que sí aplica** es la de la sección 3 de la GPLv2: quien recibe el binario tiene
derecho a recibir su código fuente correspondiente. Se cumple por la vía 3(a), acompañar el binario
del código: `source/smartmontools-7.5.tar.gz` viaja **dentro del instalador**, en
`licenses\smartmontools\`, de modo que el fuente acompaña siempre al binario, en la misma entrega y
sin depender de nada externo.

Se descartó la vía 3(b), la oferta escrita válida durante tres años, porque obliga a mantener
disponible el fuente y a atender solicitudes durante ese plazo. Un fichero de 1 MB dentro del
instalador cuesta menos y no caduca.

## Al actualizar de versión

1. Descargar el `win32-setup.exe` y el `.tar.gz` de la misma versión, con sus `.md5`.
2. Verificar las sumas publicadas antes de extraer nada.
3. Extraer `bin/smartctl.exe` y `bin/drivedb.h` (los de 64 bits, no los de `bin32/`).
4. Comprobar los hashes contra `doc/checksums64.txt` del paquete.
5. Sustituir también el tarball de `source/`: **debe ser exactamente el de la versión del binario**,
   o el requisito de "fuente correspondiente" deja de cumplirse.
6. Actualizar la tabla de hashes de este documento y la de `THIRD_PARTY_NOTICES.md`.
7. Repasar `NEWS.txt`: un cambio en el formato JSON obligaría a revisar los analizadores.

## Notas de comportamiento verificadas

Comprobado el 2026-09-04 sobre este mismo binario:

- `smartctl --scan-open --json` funciona **sin privilegios de administrador** y enumera los
  dispositivos con su tipo (`ata`, `nvme`).
- Leer datos de un dispositivo **sin elevación falla**, lo que confirma la premisa del ADR-004. Pero
  falla de una forma engañosa: `exit_status: 1` y el mensaje
  `"/Device/HarddiskN/Partition0: Unable to detect device type"`.

  **Ese mensaje no significa que el disco sea incompatible.** Si se tomara al pie de la letra, un
  equipo entero aparecería como "no compatible" cuando el problema es de privilegios. El colector
  debe distinguir los dos casos y, ante ese mensaje, comprobar primero si el proceso está elevado.
