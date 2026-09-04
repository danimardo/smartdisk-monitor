#!/usr/bin/env node
/**
 * Verifica que los recursos redistribuidos están presentes y son los que dicen ser.
 *
 * No es paranoia: sin los .woff2 la aplicación cae a Segoe UI, la métrica cambia, los bocetos
 * aprobados dejan de ser fieles y **nadie se entera**, porque todo sigue funcionando. Y un binario
 * privilegiado que se distribuye a terceros no se copia a ciegas.
 *
 * Los hashes son los registrados en THIRD_PARTY_NOTICES.md. Si actualizas un recurso, actualiza
 * los dos sitios.
 */

import { createHash } from "node:crypto";
import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const ASSETS = [
  {
    path: "src/design-system/fonts/InstrumentSans-latin.woff2",
    sha256: "2ee17598a98d8a59e4df8152d015bec9ab8e4d5672cc0ab42bef806b568e3971",
    why: "sin ella la interfaz no coincide con los bocetos aprobados"
  },
  {
    path: "src/design-system/fonts/InstrumentSans-latin-ext.woff2",
    sha256: "c4fcfea41f2c1cfeea9211fa43679845454a1d0e0d7e95e069c7e73c4ae302d2",
    why: "cubre los caracteres de los mensajes originales de Windows"
  },
  {
    path: "src/design-system/fonts/OFL.txt",
    why: "la SIL OFL obliga a redistribuir la licencia con la fuente"
  },
  {
    path: "third-party/smartmontools/bin/smartctl.exe",
    sha256: "b5db94e5082c042be44994b7a4fa8f7b5c8e713b2ab1c9a560d8f7a7995ea27d",
    why: "fuente principal de datos SMART"
  },
  {
    path: "third-party/smartmontools/bin/drivedb.h",
    sha256: "dd39c6a520d38895da61923fe26fe7c9c5eb3f42325f2e4477a06ed7a61966d0",
    why: "sin ella los atributos de fabricante quedan sin interpretar"
  },
  {
    path: "third-party/smartmontools/source/smartmontools-7.5.tar.gz",
    sha256: "690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e",
    why: "la GPLv2 §3a exige acompañar el binario de su fuente correspondiente"
  },
  {
    path: "third-party/smartmontools/licenses/COPYING.txt",
    why: "texto de la GPL v2"
  }
];

let failures = 0;

for (const asset of ASSETS) {
  const full = join(ROOT, asset.path);

  if (!existsSync(full)) {
    console.error(`FALTA     ${asset.path}\n          ${asset.why}`);
    failures++;
    continue;
  }

  if (!asset.sha256) {
    console.log(`ok        ${asset.path}`);
    continue;
  }

  const actual = createHash("sha256").update(readFileSync(full)).digest("hex");
  if (actual !== asset.sha256) {
    console.error(
      `ALTERADO  ${asset.path}\n` +
        `          esperado ${asset.sha256}\n` +
        `          obtenido ${actual}\n` +
        `          si la actualización es intencionada, actualiza THIRD_PARTY_NOTICES.md y este script`
    );
    failures++;
    continue;
  }

  console.log(`ok        ${asset.path}`);
}

if (failures > 0) {
  console.error(`\n${failures} recurso(s) con problemas. La compilación no debe continuar.`);
  process.exit(1);
}
console.log(`\n${ASSETS.length} recursos verificados.`);
