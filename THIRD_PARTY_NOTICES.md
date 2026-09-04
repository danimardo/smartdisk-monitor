# Third-party notices

This file will be completed from the exact dependency lockfiles and bundled artifacts before the first distributed build.

## smartmontools / smartctl

SmartDisk Monitor is designed to invoke `smartctl` as a separate executable.

- Project: https://www.smartmontools.org/
- Copyright (C) 2002-2011 Bruce Allen; 2008-2025 Christian Franke; 2000 Michael Cornwell; and others
  (see `third-party/smartmontools/licenses/AUTHORS.txt`)
- License: GNU General Public License, version 2 or later (`SPDX-License-Identifier: GPL-2.0-or-later`)
- Version: **smartmontools 7.5**, released 12 May 2025 (build r5714)
- Distribution status: **bundled**

| Bundled file | MD5 | SHA-256 |
|---|---|---|
| `bin/smartctl.exe` (x64) | `c1d1d016a33b09014517636c61d30947` | `b5db94e5082c042be44994b7a4fa8f7b5c8e713b2ab1c9a560d8f7a7995ea27d` |
| `bin/drivedb.h` | | `dd39c6a520d38895da61923fe26fe7c9c5eb3f42325f2e4477a06ed7a61966d0` |
| `source/smartmontools-7.5.tar.gz` | `38c38b0b82db7fc4906cdd50d15a7931` | `690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e` |

MD5 sums verified against `doc/checksums64.txt` inside the official package and against the `.md5`
files published alongside the download.

When a binary is added, this distribution must include the corresponding copyright and license text and satisfy the source-code obligations applicable to that binary. This notice does not replace those materials.

`smartctl` runs as a separate process, communicating over the command line and JSON on standard
output. It is never linked into the application, so no derivative work is created and the project's
own MIT licence is unaffected.

The GPLv2 section 3 source obligation does apply to the redistributed binary, and is satisfied via
**option 3(a)**: the corresponding source archive ships **inside the installer**, at
`licenses\smartmontools\smartmontools-7.5.tar.gz`, so the source always accompanies the binary in
the same delivery. Option 3(b), a written offer valid for three years, was rejected because it
requires keeping the source available and answering requests for that period; a 1 MB file inside a
140 MB installer costs less and does not expire.

The source archive version must always match the binary version, or the "corresponding source"
requirement is no longer met.

## Instrument Sans

The user interface embeds the Instrument Sans variable font. The application performs no network
requests, so the font is shipped as a local file rather than loaded from a font CDN (ADR-018).

- Project: https://github.com/Instrument/instrument-sans
- Authors: Rodrigo Fuenzalida, Jordan Egstad
- Copyright 2022 The Instrument Sans Project Authors
- License: SIL Open Font License 1.1
- Version: v4 of the Google Fonts catalogue, which publishes the subsetted `.woff2` files
- Distribution status: **bundled**

| Bundled file | SHA-256 |
|---|---|
| `src/design-system/fonts/InstrumentSans-latin.woff2` | `2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971` |
| `src/design-system/fonts/InstrumentSans-latin-ext.woff2` | `c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2` |

The unmodified `OFL.txt` ships alongside the font files in the same folder. The font is
redistributed under its original family name and is not modified, so the licence requires no name
change. Both files must also be copied into the installed application folder.

## Application dependencies

Rust and JavaScript dependency notices will be generated and reviewed from the locked dependency graph before release. No dependency is yet vendored at the specification stage.

