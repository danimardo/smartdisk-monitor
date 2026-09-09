# Phase 0 — Research: Contexto crudo para la explicación con IA

Feature `006-explicacion-ia-contexto-crudo`. No hay `NEEDS CLARIFICATION` en la spec (las cuatro
decisiones abiertas se resolvieron en la sesión de clarificación). Este documento fija las
decisiones de diseño que la spec deja al plan.

---

## D1 — Estrategia de anonimización del volcado y del suceso

**Decisión**: anonimización en **tres capas**, todas en el dominio (Rust), en este orden:

1. **Extracción estructurada → sustitución literal.** Antes de tocar el texto libre, se parsean
   los datos que tienen forma conocida y se alimentan al `reporting::anonimizar::Anonimizador`
   (que ya sustituye cadenas reales por marcadores, de forma consistente dentro de una petición):
   - del JSON de `smartctl`: `serial_number` y `wwn` (ver D3);
   - del disco de inventario: su `serial_number` (ya se hace en la 005) y el de todos los discos
     presentes (ya se hace);
   - del entorno: `COMPUTERNAME` y `USERNAME` (ya se hace, `Anonimizador::para_esta_maquina`).
2. **Barrido por patrones → sustitución por marcador.** Función pura nueva en `domain::ia`
   (`redactar_identificadores` o similar), **mismo estilo byte-a-byte que `barrer_texto_residual`,
   sin crate `regex`**. Sustituye lo que la capa 1 no cubre porque no viene de un campo conocido:
   - **SID**: `S-1-` seguido de grupos de dígitos separados por `-` → `<SID>`.
   - **Ruta NT de dispositivo**: `\Device\` seguido de no-espacios → `<DISPOSITIVO>`.
   - **WWN hexadecimal**: `0x` + 12–16 dígitos hex → `<WWN>` (por si aparece rendida como texto en
     algún campo o en el mensaje del suceso).
   Conservador: puede sustituir de más (una explicación un poco más pobre), nunca de menos.
3. **Barrido residual → señalar para revisión.** `barrer_texto_residual` (ya existe, **sin
   endurecer**) sobre el texto ya ensamblado y pasado por las capas 1–2. Lo que quede marcado
   dispara la pantalla de revisión (FR-010) salvo que «enviar sin revisar» esté activo (FR-009).

**Orden concreto en `explicar_detalle_tecnico`** (una sola pasada, sobre el `user_crudo` ya
ensamblado por `componer_consulta`; `componer_consulta` **no** anonimiza, solo ensambla en bruto):

```
user_crudo  = resumen + "\n\n" + volcado_bruto + "\n\n" + suceso_bruto      // componer_consulta
texto       = anon.aplicar(&user_crudo)                                      // capa 1 (literal)
texto       = redactar_identificadores(&texto)                               // capa 2 (patrones)
(texto, rec)= recortar(&texto, 40_000)                                       // FR-013
fragmentos  = barrer_texto_residual(&texto)                                  // capa 3 (señala)
```

Esto evita el modelo mixto (anonimizar unos fragmentos antes de ensamblar y el resto después): la
anonimización es **una operación sobre el texto completo**, después de ensamblar y antes de
recortar.

**Relación con FR-006**: FR-006 dice «el barrido de texto residual se endurece». La 006 lo cumple
**añadiendo la capa 2** (una pasada de sustitución de identificadores con gramática fija) y dejando
`barrer_texto_residual` como está. El efecto —menos fragmentos identificables llegan a la capa 3—
es el que FR-006 busca; la redacción de FR-006 se lee en ese sentido. (Si se prefiere endurecer
`barrer_texto_residual` en sí, sería una tarea adicional; la capa 2 se consideró más limpia porque
*sustituye* en vez de solo *señalar*.)

**Rationale**: la capa 1 es determinista y es la más fuerte; se apoya en un módulo ya auditado. La
capa 2 cubre identificadores con gramática fija que no vienen de un campo (típico del `raw_xml` de
sucesos). La capa 3 es la red de seguridad que la 005 ya tenía. Ninguna capa añade dependencias.

**Alternativas descartadas**:
- **Crate `regex`**: cubriría los patrones con menos código, pero es una dependencia nueva en un
  binario privilegiado (límite duro de `AGENTS.md`, exige ADR y autorización) para un problema que
  el patrón byte-a-byte de `barrer_texto_residual` ya resuelve en este proyecto. Descartada.
- **Enviar el volcado sin la capa 2, confiando solo en la 1 + 3**: la capa 3 solo *señala*, no
  sustituye; sin la capa 2, un SID o una ruta `\Device\` en el mensaje de un suceso obligaría a
  pasar por la pantalla de revisión en casi todas las consultas de suceso. La capa 2 los limpia de
  entrada. Descartada.
- **Anonimizar en el frontend**: prohibido por el principio XVI (la anonimización precede a la
  salida del texto y ocurre en Rust). Descartada.

---

## D2 — Qué se extrae del suceso de Windows

**Decisión**: para una alerta de suceso, se toma el evento disparador (el `triggering_event_id` de
la ocurrencia más reciente del grupo → `repo_varios::get_event_by_id`) y se envía:

- su `message` (el texto renderizado que Windows ya guardó, `Option<String>`), si existe;
- los pares `Name = valor` de los nodos `<EventData><Data>` de su `raw_xml`.

**No** se envía el bloque `<System>` del `raw_xml` (proveedor, GUID, `EventID` numérico crudo,
`Version`, `Level`, `Task`, `Keywords`, `TimeCreated`, `EventRecordID`, `Execution` con ProcessID/
ThreadID, `Channel`, `Computer`, `Security UserID`). El identificador de suceso legible (proveedor
+ número) ya viaja en el resumen estructurado como parte del contexto; el resto de `<System>` es
metadato de transporte que concentra identificadores y no explica nada.

**Rationale**: el `message` es lo que un técnico leería; los `EventData` llevan el volumen, el
dispositivo y los códigos concretos. `<Computer>` y `<Security UserID>` son identificadores puros;
descartarlos de raíz es más simple y más seguro que anonimizarlos.

**Extracción del `EventData`**: lector acotado sobre la cadena `raw_xml` (buscar `<EventData>`,
recorrer nodos `<Data ...>`), no un parser XML completo ni una dependencia nueva. El `raw_xml` ya
lo produce el colector de la 003 y su forma es estable (`event_log.rs` tiene fixtures de las dos
variantes: `Data Name='...'` y `Data` sin nombre).

**Fallback (FR-015)**: sin `message` y sin `EventData` legible → se envía solo el resumen
estructurado y el modal avisa.

---

## D3 — WWN: de dónde sale y cómo se sustituye

**Decisión**: el WWN **se anonimiza** (identifica la unidad física de forma única, como la serie).
El JSON de `smartctl` lo trae como objeto `"wwn": { "naa": N, "oui": N, "id": N }` con enteros en
decimal. Se lee `wwn.id` (y, si se quiere robustez extra, el par `oui`+`id`), se convierte a la
**cadena decimal exacta que aparece en el JSON** y se pasa al `Anonimizador` como sustitución
literal → `<WWN>`. La capa 2 (patrones) cubre además cualquier forma `0x…` hexadecimal.

`naa` y `oui` solos no se sustituyen: `naa` es el formato (5) y `oui` es el fabricante, no la
unidad.

**Rationale**: `wwn.id` es la parte por dispositivo. Sustituir la representación textual exacta que
va en el payload es determinista; el `Anonimizador` ya garantiza consistencia dentro de la
petición.

**Nota de implementación**: el parser actual (`smartctl_parser::RaizJson`) no deserializa `wwn`.
Se añade un tipo `serde` mínimo para leerlo **solo** para anonimizar; no entra en `SmartctlResult`
ni en ninguna métrica.

---

## D4 — Tamaño máximo del detalle enviado

**Decisión**: `MAX_DETALLE_CHARS` sube de **8 000 a 40 000** caracteres. El texto se ensambla en
el orden `resumen estructurado → volcado smartctl → contenido del suceso` y se pasa una sola vez
por `domain::ia::recortar(total, MAX_DETALLE_CHARS)`.

- El resumen va primero → nunca se recorta.
- El suceso va al final → es lo primero que se pierde si hay que recortar (cumple FR-013 sin lógica
  extra).
- `recortar` ya corta en el último salto de línea y marca `detalle_recortado`, que el modal ya
  muestra (FR-021 de la 005). No hace falta un aviso nuevo.

**Rationale**: un `smartctl -a -j` típico ocupa 6–20 KB; con `ata_smart_error_log` e historial de
autotests puede llegar a ~35 KB. 40 000 caracteres cubren el caso normal completo más el suceso.
Para `openrouter/free` (contexto habitual 8k–128k tokens) 40 KB ≈ 12–14k tokens de entrada: dentro
de lo que aceptan los modelos gratuitos actuales; si uno lo rechaza, ya es un `AppError`
`ia.provider` que la 005 maneja.

**Alternativas descartadas**:
- **Presupuesto por sección** (p. ej. 30 KB volcado + 8 KB suceso): más código para el mismo
  efecto que el orden de concatenación + un `recortar`. Descartada por complejidad innecesaria.
- **No subir el límite y confiar en el recorte**: dejaría casi siempre fuera el error log del
  disco, que es justo lo que da valor a la explicación (SC-001). Descartada.
- **Comprimir/resumir el volcado en el cliente antes de enviar**: reintroduce lógica de dominio en
  el front y una heurística frágil. Descartada.

---

## D5 — El ajuste «enviar sin revisar»: forma y flujo

**Decisión**:
- Clave: `settings.ai.send_without_review`, booleana, fábrica `false`. Cuarta clave del grupo
  `settings.ai` (las otras tres no cambian).
- La escribe un **comando dedicado** `establecer_envio_sin_revision(activar: bool) -> EstadoIaWire`,
  de la familia IA, **no** `set_setting` (coherente con la 005: `set_setting` solo edita
  `settings.ai.model`; `enabled` y `preview_acknowledged` los llevan comandos propios).
- `estado_ia` devuelve el valor en `EstadoIaWire.sendWithoutReview`.
- `borrar_clave_ia` lo pone a `false` (junto a `preview_acknowledged`) → cumple FR-012 (desactivar
  y reactivar la función lo devuelve a fábrica).
- `reset_settings` ámbito `ai`/`all` borra la clave (se añade a la lista `AI`).
- En `explicar_detalle_tecnico`: si `send_without_review == true`, se omite la respuesta
  `ResultadoExplicacion::Revision` **por fragmentos dudosos** (`barrer_texto_residual`), pero **no**
  la de vista previa (FR-009: la vista previa de la primera consulta se sigue mostrando).

**Flujo de UI (FR-008)**: en la tarjeta de Ajustes de IA, un `Switch` «enviar sin revisar». Al
pasar de apagado a encendido, se abre un `ConfirmDialog` con el aviso de riesgo (`impact` = «con
esta opción, fragmentos de texto libre del volcado o del suceso podrían salir del equipo sin
revisión manual»). Solo al confirmar se llama a `establecer_envio_sin_revision(true)`. Apagarlo es
directo, sin diálogo.

**Rationale**: el comando dedicado mantiene la simetría con la 005 y permite que el reset en
`borrar_clave_ia` sea una línea al lado del de `preview_acknowledged`. El diálogo es un
`ConfirmDialog` del catálogo, sin componente nuevo.

**Alternativas descartadas**:
- **Ruta genérica `set_setting("settings.ai.send_without_review", true)`**: rompería la regla de la
  005 de que `settings.ai.*` (salvo `model`) no se toca por esa vía, y dejaría el reset en
  `borrar_clave_ia` desalineado. Descartada.
- **Que el backend exija una prueba de consentimiento**: el backend no puede verificar que se
  mostró un diálogo; la puerta es del frontend, como la vista previa de la 005. Descartada.

---

## D6 — Obtención del volcado en `reunir_datos_explicacion`

**Decisión**: reutilizar el camino de `get_alert_smart_raw_json_impl` /
`collectors::smartctl::query_device_json(&ruta)` con la `smartctl_path` del dispositivo. Para el
tipo `Smart` (detalle de disco) se usa el `device_id` directo; para `Alerta` se usa el
`target_device_id` del grupo, como ya hace la 005.

- Si el disco no tiene `smartctl_path`, o la consulta falla (disco ausente, herramienta no
  disponible) → **no es error**: se continúa con el resumen estructurado y `ExplicacionIaWire`
  lleva `sin_volcado = true`, que el modal traduce a `t("ia.explain.withoutDump")` (FR-014).
  Análogamente `sin_suceso = true` para FR-015. Se descartó reutilizar `detalle_recortado`: las
  tres condiciones (recorte / sin volcado / sin suceso) son independientes y pueden coincidir; dos
  booleanos nuevos son más claros que un enum de motivos. Ver `data-model.md` §«Explicación
  devuelta» y `contracts/comandos-ia.md`.

**Rationale**: el volcado bajo demanda ya está resuelto y aceptado en la 005 (mismo patrón,
mismo coste). No se introduce histórico ni caché (asunción de la spec).

---

## D7 — Documentación normativa a tocar

| Documento | Cambio | Cuándo |
|---|---|---|
| `.specify/memory/constitution.md` | Enmienda principio XVI → **1.9.0** (dos viñetas reescritas + historial + footer). La aplica la persona con `parche-constitucion.md`. | Antes del cierre (no bloquea Phase 0/1) |
| `docs/decisions.md` | **ADR-047** nuevo: payload ampliado (volcado + suceso), anonimización en capas, modo «enviar sin revisar». Retoque de la viñeta «Datos enviados» de **ADR-046** para remitir a ADR-047. | Durante implementación |
| `docs/data-model.md` | Grupo `settings.ai`: pasa de tres a cuatro claves; añadir `send_without_review`. | Durante implementación |
| `docs/ui-contract.md` | Comando nuevo `establecer_envio_sin_revision`; campo nuevo en `EstadoIaWire`; nota de que `explicar_detalle_tecnico` ahora incluye volcado + suceso. | Al cerrar |
| `docs/open-questions.md` | `MAX_DETALLE_CHARS` pasa de 8 000 (propuesto) a 40 000 (medido/decidido para 006). | Durante implementación |
| `docs/known-issues.md` | Si el lector de `EventData` usa algún `svelte-ignore`/`eslint-disable` (improbable, es Rust) — no aplica. | — |
| `historias.md` | Regenerado con `pnpm docs:build` tras editar el fichero fuente de `docs/`. | Al cerrar |

---

## D8 — Qué NO cambia (para acotar)

- El contrato de `explicar_detalle_tecnico` (`OrigenExplicacion`, `ResultadoExplicacion`) **no
  cambia de forma**: el frontend sigue mandando el mismo `origen` y recibiendo el mismo enum. Todo
  el trabajo nuevo es backend interno + el ajuste.
- El modal de explicación, el store `explicacion.svelte`, el flujo de vista previa y de revisión,
  el tiempo máximo de 60 s, el markdown seguro, los `AppError` `ia.*`: sin cambios.
- El asistente inicial (onboarding): **no** gana el `Switch` de «enviar sin revisar» (es un ajuste
  avanzado; su sitio es la configuración). La spec no lo pide para el asistente.
- Ninguna dependencia, permiso de Tauri o componente de catálogo nuevo.
