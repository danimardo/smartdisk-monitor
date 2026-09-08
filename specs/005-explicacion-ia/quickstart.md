# Phase 1 — Guía de validación end-to-end

Feature `005-explicacion-ia`. Escenarios que demuestran que la feature funciona. No incluye código
de implementación; los detalles están en `contracts/comandos-ia.md` y `data-model.md`.

## Requisitos previos

- `pnpm install`, toolchain de Rust 1.77.2, entorno de desarrollo habitual.
- Para los escenarios que salen a la red: una **clave de API de OpenRouter** real (nivel
  gratuito basta). Para el resto, no hace falta.
- `pnpm app:dev` (muestra UAC: la app va elevada).

## Comprobaciones automáticas (deben pasar todas)

```sh
cargo test            # incl. domain/ia.rs (test-first) y platform/credenciales.rs
cargo clippy --all-targets -- -D warnings
pnpm check            # cero errores y cero avisos
pnpm test             # incl. src/lib/design/markdown.test.ts
pnpm test:component   # Markdown, ExplicacionModal, sección Ajustes, paso asistente
pnpm test:e2e         # e2e/ui/ia.spec.ts
pnpm test:a11y
pnpm verify           # hashes, tokens, i18n sincronizado, fronteras
pnpm lint
```

`pnpm verify:i18n` debe confirmar que las ~35 claves nuevas están en `es.json` **y** `en.json`.
La puerta de tipos del contrato (`cargo test` regenera `generated/`) no debe mostrar diferencias.

## Escenario 1 — La función está apagada de fábrica (FR-005, SC-001)

1. Instalación limpia (sin `settings.ai.*`, sin credencial).
2. Arranca la app con un monitor de red abierto (p. ej. Resource Monitor) durante 10 min de uso
   normal: abrir alertas, ver un disco, forzar un refresco.
3. **Esperado**: cero conexiones salientes originadas por `smartdisk-monitor.exe`. En
   `/alerts` y `/disks/[id]` **no** aparece el botón «Explícamelo». En `/settings` la sección
   «Ayuda con IA» muestra el estado «desactivada».

## Escenario 2 — Activar la clave en Ajustes (US1, FR-003, FR-004, SC-004)

1. `/settings` → sección «Ayuda con IA» → pegar la clave → «Activar».
2. **Esperado**: llamada de validación; al volver, estado «activada», modelo «automático».
3. Con `reg query` / el Administrador de credenciales de Windows: existe
   `SmartDisk Monitor/OpenRouter`. Inspeccionar `%ProgramData%\SmartDisk Monitor\*.db` y los
   ficheros de la app: **la clave no aparece** en ninguno.
4. Pegar una clave con formato inválido → mensaje `error.ia.invalidKeyFormat`, no se guarda nada.
5. Pegar una clave sintácticamente plausible pero falsa → `error.ia.unauthorized`, no se guarda.

## Escenario 3 — Explicar una alerta (US2, FR-006..FR-014, SC-002)

1. Con la función activada y al menos una alerta activa con detalle SMART, abrir su detalle en
   `/alerts`.
2. Pulsar «Explícamelo en lenguaje claro».
3. **Primera vez**: aparece la **vista previa** con el texto exacto que se enviará (detalle
   anonimizado + contexto del disco) y a dónde va. Confirmar.
4. **Esperado**: indicador de progreso; en <20 s (red normal) un modal con la explicación en
   markdown (encabezados/listas reales), el modelo usado y la nota «orientación por IA, no un dato
   de salud».
5. `Escape` cierra el modal y devuelve el foco al botón.
6. Repetir en la misma sesión y tras reiniciar la app: **no** vuelve a pedir la vista previa
   (FR-027, SC-009).
7. Revisar el log: hay una línea `debug` de la consulta **sin** el prompt ni la respuesta
   (FR-022).

## Escenario 4 — Anonimización (FR-009, SC-003)

1. Preparar (fixture o disco real) una alerta cuyo detalle contenga el número de serie del disco y
   una ruta con el perfil de usuario.
2. Disparar la explicación y mirar la vista previa.
3. **Esperado**: el número de serie aparece como `<SERIE-1>`, la ruta de usuario como
   `<USUARIO>`; el modelo/marca del disco y el firmware **sí** aparecen (aclaración Q2).
4. Prueba automática equivalente en `cargo test` sobre un conjunto de casos que contienen esos
   datos en origen: 0 fugas.

## Escenario 5 — Revisión de texto libre (FR-026)

1. Fixture de alerta con la descripción de un evento de Windows que incluya `D:\Usuarios\...\`
   (texto libre no cubierto por el `Anonimizador`).
2. Disparar la explicación.
3. **Esperado**: diálogo con el texto exacto, el fragmento dudoso resaltado y tres opciones:
   «Enviar igual», «Quitar el fragmento», «Cancelar».
4. «Quitar el fragmento» → se envía con `<OMITIDO>` en su lugar; el modal luego lo confirma.
5. «Cancelar» → no sale nada a la red.

## Escenario 6 — Elegir modelo y aviso de coste (US3, FR-015, FR-015a, SC-008)

1. `/settings` → «Ayuda con IA» → abrir el selector de modelo.
2. **Esperado**: «modelo gratuito automático» seleccionado; lista poblada desde el proveedor.
3. Elegir un modelo **de pago** → `ConfirmDialog` con «puede generar cargos en tu cuenta de
   OpenRouter»; hasta confirmar, no se guarda.
4. Elegir un modelo **gratuito** → se guarda sin aviso.
5. Sin red al abrir el selector → se puede seguir con «automático» y se informa de que la lista
   no está disponible (US3 escenario 3).

## Escenario 7 — Fallos degradan solo la función (FR-017, FR-018, FR-019, SC-005)

| Situación | Esperado |
|---|---|
| Sin red al pulsar «Explícamelo» | `error.ia.network` (reintentable) en el modal; el resto de `/alerts` intacto |
| Respuesta > 60 s (simulada) | `error.ia.timeout`; opción de reintentar |
| Clave revocada tras activarla | `error.ia.unauthorized`; la sección de Ajustes invita a corregir |
| Cuota agotada (429) | `error.ia.rateLimited`, mensaje distinto de «clave incorrecta» |

## Escenario 8 — Explicar el detalle SMART de un disco (US4)

1. `/disks/[id]` con la función activada → en la tarjeta de contadores SMART, «Explícamelo».
2. **Esperado**: misma mecánica que en alertas (vista previa ya confirmada no se repite; modal con
   markdown).
3. Con la función desactivada: el botón no está.

## Escenario 9 — Respuesta hostil del modelo (principio XVI, FR-012, FR-023)

1. Forzar (doble de test) una respuesta con `<img src=x onerror=alert(1)>`, `<script>` y
   `[pulsa aquí](javascript:...)`.
2. **Esperado**: se muestran como **texto literal**; no se ejecuta nada, no hay enlace navegable.
3. Ningún color de estado, alerta ni ajuste cambia por el contenido de la respuesta (SC-007).

## Verificación de pantalla (revisión humana, `AGENTS.md` §8)

Para `ExplicacionModal`, la sección de Ajustes y el 5.º paso del asistente: tema claro y oscuro;
acento propio y del sistema; 1024×560 y 1280×720; escalado 125/150/200 %; estados cargando, vacío,
error; teclado y foco; textos en `es` y `en`.
