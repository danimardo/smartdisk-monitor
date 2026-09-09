# Phase 1 — Quickstart / guía de validación: 006

Feature `006-explicacion-ia-contexto-crudo`. Escenarios ejecutables que prueban el incremento de
punta a punta. Presupone la 005 ya funcionando (clave de OpenRouter configurable, modal de
explicación operativo).

## Prerrequisitos

- `pnpm install` y toolchain de Rust del proyecto (1.77.2).
- Enmienda al principio XVI aplicada: `parche-constitucion.md` ejecutado por la persona y
  `pnpm docs:build` pasado. (No bloquea el desarrollo, sí el cierre; hazlo antes de validar.)
- Una clave de API de OpenRouter para las pruebas manuales de red, o el mock del transporte para
  las de `pnpm test:e2e`.

## Comandos de verificación

```sh
cargo test -p smartdisk-monitor ia::            # dominio: barrido por patrones, extracción, ensamblado
cargo test -p smartdisk-monitor explicar        # comando: volcado + suceso + send_without_review
cargo clippy --all-targets -- -D warnings
pnpm check && pnpm lint
pnpm test:component -- settings                  # Switch «enviar sin revisar» + ConfirmDialog
pnpm test:e2e:smoke
pnpm verify                                      # tokens, i18n, fronteras, hashes
```

## Escenario 1 — Alerta SMART con volcado crudo (US1, FR-001, FR-004)

1. Con la ayuda con IA activada, abrir una alerta `smart.*` en `/alerts`.
2. Pulsar «Explícamelo en lenguaje claro»; confirmar la vista previa si es la primera vez.
3. **Comprobar en la vista previa / en el texto capturado por el mock**: contiene el resumen
   estructurado **y** un bloque JSON de `smartctl`; el `serial_number` y el `wwn` aparecen como
   `<SERIE-1>` / `<WWN>`; la marca, el modelo y el firmware se leen sin sustituir.
4. **Comprobar en el modal**: la explicación cita al menos un valor concreto (un atributo, un
   umbral, horas de encendido, una entrada del error log).

**Esperado**: SC-001 (la explicación se apoya en cifras reales), SC-002 de la 005 (≤20 s).

## Escenario 2 — Alerta de suceso de Windows con el contenido del evento (US2, FR-002)

1. Provocar o localizar una alerta originada por un suceso de Windows (p. ej. `Ntfs`/`disk` en el
   registro del sistema) y abrir su detalle.
2. Pulsar la acción de explicación.
3. **Comprobar en el texto enviado**: incluye el `message` del suceso y sus pares `EventData`
   (`DriveName`, `DeviceName`…); **no** incluye `<Computer>`, `<Security UserID>`, ProcessID ni
   GUID del proveedor; las rutas `\Device\HarddiskVolumeN` salen como `<DISPOSITIVO>`.

**Esperado**: FR-002; SC-002.

## Escenario 3 — Reserva cuando falta el volcado (FR-014)

1. Desconectar (o simular ausente) el disco de una alerta.
2. Pulsar la acción de explicación.
3. **Comprobar**: la consulta se envía solo con el resumen estructurado; el modal muestra el aviso
   `ia.explain.withoutDump`; el resto de la aplicación sigue funcionando.

**Esperado**: SC-006.

## Escenario 4 — Pantalla de revisión con el modo apagado (FR-010, defecto)

1. Con `send_without_review` apagado (fábrica), lanzar una explicación de un suceso cuyo `message`
   contenga un token que el barrido no pueda clasificar (p. ej. un nombre de host raro).
2. **Comprobar**: aparece la pantalla de revisión con el fragmento resaltado y las tres opciones
   (enviar tal cual / quitar / cancelar). Elegir «quitar» → el fragmento sale como `<OMITIDO>`.

**Esperado**: SC-003.

## Escenario 5 — Activar «enviar sin revisar» (US4, FR-007, FR-008, FR-009)

1. Ajustes → tarjeta «Ayuda con IA» → activar el `Switch` «Enviar sin revisar».
2. **Comprobar**: se abre un `ConfirmDialog` con el aviso de riesgo. Cancelar → el `Switch` vuelve
   a apagado y `estado_ia().sendWithoutReview == false`.
3. Repetir y **confirmar**. `estado_ia().sendWithoutReview == true`.
4. Lanzar la explicación del escenario 4. **Comprobar**: no aparece la pantalla de revisión; el
   texto va anonimizado (serie/WWN/SID/dispositivo sustituidos). Si es la primera consulta tras
   activar la función, la **vista previa sí se muestra**.

**Esperado**: SC-004, SC-005.

## Escenario 6 — Desactivar y reactivar la función resetea el modo (FR-012)

1. Con `send_without_review == true`, Ajustes → «Ayuda con IA» → borrar la clave.
2. Volver a introducir una clave válida.
3. **Comprobar**: `estado_ia().sendWithoutReview == false`; el `Switch` está apagado.

**Esperado**: SC-008.

## Escenario 7 — El log no filtra contenido (FR-016)

1. Activar el modo detallado del registro (Ajustes → Registro).
2. Lanzar varias explicaciones (SMART y de suceso).
3. Abrir la carpeta del registro y **comprobar**: no aparece el volcado, ni el `message` del
   suceso, ni la respuesta del modelo; como mucho, `consulta de explicación con IA` con
   `modelo`/`resultado`/`ms`.

**Esperado**: SC-007.

## Referencias

- Contrato: [`contracts/comandos-ia.md`](./contracts/comandos-ia.md)
- Modelo de datos: [`data-model.md`](./data-model.md)
- Decisiones de diseño: [`research.md`](./research.md)
- Enmienda constitucional: [`parche-constitucion.md`](./parche-constitucion.md)
