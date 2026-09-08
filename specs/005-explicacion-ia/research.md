# Phase 0 — Investigación y decisiones técnicas

Feature: `005-explicacion-ia`. Todo lo que aquí se decide alimenta `plan.md`, `data-model.md` y
`contracts/`. Las decisiones que fijan una dependencia o un compromiso duradero se recogen en
ADR-046 (ya aceptado; una tarea lo precisa con la elección concreta).

---

## D1 — Cliente HTTP: `reqwest` directo con `native-tls` (SChannel en Windows)

**Decisión** (revisada durante la implementación): promover `reqwest` a dependencia directa de
`src-tauri/Cargo.toml`:

```toml
reqwest = { version = "0.13", default-features = false, features = ["native-tls", "json"] }
```

- **Por qué `native-tls` y no `rustls`**: en reqwest **0.13** el feature `rustls` declara la
  dependencia `rustls` **sin** `default-features = false`, y el default de `rustls` 0.23 incluye
  `aws_lc_rs` → `aws-lc-sys` (BoringSSL vendorizado, compilación de C, feature `prebuilt-nasm`).
  No hay forma de desactivarlo desde nuestro `Cargo.toml` (las features son aditivas). Meter una
  librería de criptografía en C vendorizada en un binario privilegiado que se distribuye a
  terceros es justo lo que la constitución evita.
- `native-tls` en un proyecto **solo-Windows** (constitución §III) usa **SChannel**, la pila TLS
  del propio sistema operativo: crate `schannel` (FFI puro, sin C), ya parcheada por Windows
  Update. `cargo tree --target x86_64-pc-windows-msvc` confirma que en Windows solo compilan
  `native-tls`, `tokio-native-tls`, `schannel` y `hyper-tls` (los `openssl-*` del `Cargo.lock`
  son de targets que no se compilan). ≈6 crates efectivos vs ≈20 con la cadena rustls/aws-lc.
- Ya estaba en `Cargo.lock` (lo arrastra `tauri` 2.11.5) **pero sin backend TLS**: no podía
  hablar HTTPS tal cual. `tokio` y `hyper` ya estaban compilados.
- `json` para (de)serializar con tipos serde explícitos (principio XI). Sin `charset` (OpenRouter
  responde siempre UTF-8; ahorra `encoding_rs` + `simdutf8` + `multiversion`).
- **`json`** para (de)serializar el cuerpo con tipos `serde` explícitos (principio XI).
- La llamada se hace desde un **comando `#[tauri::command] async`**, que corre sobre el runtime
  tokio que Tauri ya tiene. No hace falta `#[tokio::main]` ni un runtime propio.
- El `Client` se construye **dentro del comando**, solo si hay clave. No hay cliente, ni *pool*,
  ni resolución de nombres al arrancar → cumple FR-005 y SC-001.
- `Client::builder().timeout(Duration::from_secs(60))` para FR-019; además
  `.connect_timeout(10s)` para distinguir «sin red» rápido.

**Alternativas descartadas**:
- **`ureq` + `rustls`** (bloqueante). Encaja con el ethos «sin tokio» (ADR-033), pero ADR-033
  aplica al *planificador*; este comando no es el planificador. Añadir un segundo stack HTTP
  cuando `reqwest`/`hyper`/`tokio` ya están compilados es más superficie, no menos.
- **`tauri-plugin-http`** desde el WebView. Descartado por ADR-046 y principio XVI: metería la
  clave en el proceso de la interfaz y obligaría a abrir `http` en *capabilities*.
- **`reqwest` con `default-tls` (schannel en Windows)**. Funciona sin `ring`, usa el TLS del SO.
  Se descarta por reproducibilidad: `rustls` + `webpki-roots` da el mismo comportamiento en
  cualquier Windows soportado y es lo que ADR-046 nombró.

**Consecuencia de capabilities**: **ninguna**. La red la origina Rust; `capabilities/default.json`
no cambia. ADR-046 mencionaba «un permiso de red de Tauri» como posibilidad; la vía elegida no lo
necesita y así se anota en el ADR.

---

## D2 — Almacén de la clave: crate `windows`, APIs `Win32_Security_Credentials`

**Decisión**: promover `windows` 0.61.3 a dependencia directa en
`[target.'cfg(windows)'.dependencies]`:

```toml
windows = { version = "0.61", features = ["Win32_Security_Credentials", "Win32_Foundation"] }
```

Ya compilado (lo arrastra `tauri`). **Cero crates transitivos nuevos.** Se envuelve en
`platform/credenciales.rs` (~80 líneas, mismo patrón que `platform/*.rs` y que el uso de
`windows-registry`):

| Operación | API | Nota |
|---|---|---|
| Guardar | `CredWriteW` | `CREDENTIALW` con `Type = CRED_TYPE_GENERIC`, `TargetName = "SmartDisk Monitor/OpenRouter"`, `CredentialBlob` = clave UTF-16LE, `Persist = CRED_PERSIST_LOCAL_MACHINE` |
| Leer | `CredReadW` + `CredFree` | devuelve `Option<String>`; si `ERROR_NOT_FOUND` → `None`, no error |
| Borrar | `CredDeleteW` | `ERROR_NOT_FOUND` se trata como éxito idempotente |

- **`CRED_PERSIST_LOCAL_MACHINE`**: el proceso corre elevado (`requireAdministrator`, ADR-004); la
  credencial de máquina la cifra DPAPI con la clave de la máquina y la lee el proceso elevado sin
  depender de qué administrador inició sesión. Es un producto mono-equipo, no un servicio
  multiusuario.
- El blob nunca se registra ni se incluye en el ZIP de diagnóstico (ya está fuera: no es un
  fichero ni una fila de SQLite).
- `estado_ia` no devuelve la clave: solo si existe y si es válida (última comprobación).

**Alternativas descartadas**:
- **`keyring` v3**. Más ergonómico, pero es un árbol de dependencias nuevo (`windows-sys` aparte,
  `secret-service` en otros SO que no compilamos). El proyecto ya resuelve Win32 con crates
  enfocados (`windows-registry`); esto es coherente.
- **DPAPI directo (`CryptProtectData`) a un fichero**. Descartado: sería «un fichero junto al
  ejecutable», justo lo que el principio XII desaconseja. El Administrador de credenciales es el
  almacén que nombran el principio XII y ADR-046.

---

## D3 — Render de markdown: analizador de subconjunto propio, sin `{@html}`

**Decisión**: `src/lib/design/markdown.ts` analiza un **subconjunto** de CommonMark a un árbol de
tokens tipado; `src/lib/components/Markdown.svelte` lo renderiza con **marcado Svelte** (`{#each}`,
`<h2>`, `<strong>`, `<code>`, `<ul>`…), nunca `{@html}`.

Subconjunto soportado (suficiente para una explicación de un LLM):

| Bloque | Inline |
|---|---|
| Párrafo, encabezados `#`–`###`, lista `-`/`*`, lista `1.`, bloque de código ```` ``` ````, cita `>` | `**negrita**`, `*cursiva*`, `` `código` ``, enlace `[texto](url)` |

- **Enlaces**: se renderiza **solo el texto**; la URL se muestra entre paréntesis como texto
  plano, no como `<a href>`. La app no abre nada que diga un tercero (principio IX, XVI).
- Todo lo no reconocido (tablas HTML, `<script>`, `<img>`, `<iframe>`, entidades raras) se emite
  como **texto literal escapado**. El componente no tiene ninguna vía a `innerHTML`.
- ≈120–160 líneas + su prueba. Test-first no es obligatorio (no es un parser de datos de salud),
  pero la prueba acompaña al código y cubre: cada bloque, anidamiento simple, y que
  `<img onerror=...>` y `[x](javascript:...)` salen como texto inerte.

**Alternativas descartadas**:
- **`marked` / `markdown-it` / `micromark` + `DOMPurify`**. Dos dependencias nuevas de frontend +
  un ADR, y `{@html}` con contenido no confiable como superficie permanente. La constitución
  («ante dos soluciones que funcionan, la más simple») y el principio XVI («markdown seguro,
  jamás como HTML») empujan al analizador propio.
- **Mostrar la respuesta como texto plano sin formato.** Más simple aún, pero FR-012/FR-014 piden
  markdown y una explicación con listas y encabezados se lee mucho mejor. El coste del subconjunto
  es bajo.

`Markdown.svelte` entra al **catálogo cerrado** (`ui-design.md` §3): es un primitivo de
presentación, sin lógica de negocio, reutilizable. Tarea de diseño para aprobarlo.

---

## D4 — Superficie de comandos

Seis comandos nuevos (detalle de firmas en `contracts/comandos-ia.md`):

| Comando | Tipo | Para |
|---|---|---|
| `estado_ia` | sync | ¿activa? modelo elegido, `previewAcknowledged`, validez de la última comprobación |
| `guardar_clave_ia` | async | guarda la clave en el almacén y la valida con una llamada mínima; devuelve el nuevo `EstadoIa` |
| `borrar_clave_ia` | sync | borra credencial + limpia `settings.ai.*`; la función queda inactiva |
| `probar_clave_ia` | async | valida la clave actual sin cambiarla (botón «Probar» de Ajustes) |
| `listar_modelos_ia` | async | `GET /api/v1/models` → lista para el `Select`, con marca de coste |
| `explicar_detalle_tecnico` | async | el gesto «Explícamelo»: recibe origen + opción de revisión, devuelve markdown + modelo, o `RevisionAnonimizacion`, o `AppError` |

Patrón: el comando valida argumentos y delega en `domain::ia` (puro) + `platform::ia_openrouter`
(transporte). Ningún comando lleva lógica de prompt.

---

## D5 — Construcción del prompt y anonimización

**Origen de la consulta** (`OrigenExplicacion`, lo arma el frontend con lo que ya tiene en
pantalla y lo revalida el backend):

- `tipo`: `"alerta"` | `"smart"`.
- `deviceId`: para que el backend recupere del inventario el contexto acotado del disco.
- Para `"alerta"`: `alertGroupId` — el backend recompone titular + hechos + `ruleKey` + el JSON
  crudo de `smartctl` que ya sirve `get_alert_smart_raw_json`.
- Para `"smart"`: la lista de contadores visibles (`SmartCounter`) del `DeviceDetail`.

**Contexto del disco que se envía** (FR-008, aclaración Q2): `model`, `deviceType`/`busType`,
`firmware`, `powerOnHours`, antigüedad (`firstSeenAt`). Todo eso ya está en `DiskSummary` /
`DeviceDetail`. **No** se envía `serialNumber`, `fingerprint`, `alias` (puede llevar un nombre
propio), ni `volumes[].label`.

**Anonimización** (FR-009): se reutiliza
`reporting::anonimizar::Anonimizador::para_esta_maquina(&[serial])` — ya sustituye
`COMPUTERNAME`, `USERNAME` y rutas de perfil de forma consistente. Se le añaden, para esta
llamada: el `serialNumber` del disco (aunque ya suele venir de `para_esta_maquina` si está en el
inventario) y cada `volumes[].label` no vacío. La sustitución produce `<EQUIPO>`, `<USUARIO>`,
`<SERIE-1>`, `<VOLUMEN-1>`… El texto se compone **después** de anonimizar cada fragmento.

**Prompt** (constante en `domain/ia.rs`, no i18n — es instrucción al modelo, no interfaz):
- *system*: rol («explica a una persona no técnica…»), formato (markdown, encabezados/listas),
  que indique si hay que preocuparse y, si procede, pasos; que **no invente** datos que no estén
  en el texto; idioma = el de la interfaz (`es`/`en`).
- *user*: el detalle técnico anonimizado + el bloque de contexto del disco.

**Petición**: `POST {endpoint}/chat/completions`, cuerpo
`{ model, messages, max_tokens, temperature: 0.2, usage: {include: true} }`. No *streaming* en la
v1 (un `await` y el spinner bastan; *streaming* sería mejora futura).

**Respuesta OK**: tipos `serde` explícitos → `choices[0].message.content` (markdown) + `model`
(modelo real, FR-016). Vacío o sin `choices` → `AppError ia.empty_response`.

---

## D6 — Detección de texto libre no verificable (FR-026)

Tras anonimizar, `domain::ia::barrer_texto_residual(texto) -> Vec<Span>` marca fragmentos que
**podrían** seguir llevando datos del equipo:

- rutas con letra de unidad `X:\…` y UNC `\\host\share`;
- secuencias que parezcan un nombre de equipo/host largo no sustituido;
- tokens alfanuméricos largos tipo número de serie (heurística: ≥8, mezcla letra+dígito) que no
  hayan sido ya sustituidos.

Si `barrer_texto_residual` devuelve algo y el comando no trae `revision: "enviar_igual"` ni
`"quitar_fragmentos"`, el comando responde `RevisionAnonimizacion { textoCompleto, spans }` en
vez de llamar al proveedor. El frontend muestra el diálogo de 3 opciones (FR-026); al reenviar
con `"quitar_fragmentos"` el backend sustituye esos spans por `<OMITIDO>` antes de enviar.

Es una **heurística conservadora**: puede pedir revisión de más (molesto pero seguro), nunca
callar un dato (inseguro). Test-first, con fixtures de descripciones reales de eventos de Windows
anonimizadas.

---

## D7 — Modelos: lista y detección de coste (FR-015a)

`GET {endpoint}/models` (no requiere `Authorization`). Se llama solo cuando la persona abre el
selector (gesto explícito, principio XVI). Se mapea a `ModeloIaWire { id, nombre, esDePago }`.

`esDePago` = `pricing.prompt != "0"` **o** `pricing.completion != "0"` (OpenRouter da el precio
por token como cadena; `"0"` = gratis). El identificador `openrouter/free` se antepone siempre
como opción «modelo gratuito automático», con `esDePago = false`.

El aviso de coste (FR-015a) y su confirmación viven en el **frontend** (es UX): al elegir un
`ModeloIaWire` con `esDePago`, se muestra `ConfirmDialog` con impacto «puede generar cargos en tu
cuenta de OpenRouter»; solo tras confirmar se llama a `set_setting("settings.ai.model", …)`.

---

## D8 — Errores (`AppError`, principio X)

| `code` | Cuándo | `message_key` | `retryable` |
|---|---|---|---|
| `ia.no_key` | comando de explicación sin clave | `error.ia.noKey` | no |
| `ia.unauthorized` | HTTP 401 / 403 | `error.ia.unauthorized` | no |
| `ia.rate_limited` | HTTP 429 / 402 | `error.ia.rateLimited` | sí |
| `ia.timeout` | 60 s superados | `error.ia.timeout` | sí |
| `ia.network` | error de conexión / DNS / TLS | `error.ia.network` | sí |
| `ia.empty_response` | 200 sin `choices` usable | `error.ia.emptyResponse` | sí |
| `ia.provider` | otro 4xx/5xx | `error.ia.provider` | sí (5xx) |

`detail` lleva siempre el texto técnico crudo (código HTTP, `error.message` de OpenRouter,
mensaje de `reqwest`), **nunca** el prompt ni la respuesta.

---

## D9 — Distinción «clave incorrecta» vs «límite» en validación (FR-018)

`guardar_clave_ia` / `probar_clave_ia` hacen una llamada mínima (`GET /models` con
`Authorization`, o un `chat/completions` de 1 token). 401/403 → la clave no se guarda y se
devuelve `ia.unauthorized`. 429/402 → la clave **sí** se guarda (es válida) y se informa de que
el proveedor está limitando ahora mismo.

---

## D10 — Idioma y textos

- ~35 claves i18n nuevas en `es.json` y `en.json` (paso del asistente, sección de Ajustes,
  botón «Explícamelo», modal, diálogo de revisión, diálogo de coste, los 7 `error.ia.*`).
- El prompt **no** pasa por i18n (constitución XV/XIV: instrucción al modelo, no interfaz). El
  idioma se le pasa como parámetro (`"es"`/`"en"`), tomado de `settings.appearance.language`.

---

## D11 — Contratos generados (`ts-rs`)

DTO nuevos con `#[derive(Serialize/Deserialize, TS)]` y `#[ts(export_to="../../src/lib/api/generated/")]`:

- `EstadoIaWire { activa: bool, modelo: String, previewAcknowledged: bool, claveValida: Option<bool> }`
- `ModeloIaWire { id: String, nombre: String, esDePago: bool }`
- `ExplicacionIaWire { markdown: String, modeloUsado: String }`
- `RevisionAnonimizacionWire { textoCompleto: String, spans: Vec<SpanWire> }`,
  `SpanWire { inicio: u32, fin: u32, motivoKey: String }`

`cargo test` los regenera; una diferencia con lo versionado bloquea la integración (puerta de
tipos del contrato). `schemas.ts` gana su esquema Zod y `schemas.test.ts` su prueba de rechazo.

---

## D12 — Estructura de `settings`

Grupo nuevo `AiSettingsWire` dentro de `SettingsWire`:

```
ai: {
  enabled: bool,                 // settings.ai.enabled — espejo de «existe credencial»
  model: string,                 // settings.ai.model — default "openrouter/free"
  previewAcknowledged: bool,     // settings.ai.preview_acknowledged — default false
}
```

- `enabled` lo escribe solo `guardar_clave_ia` (true) y `borrar_clave_ia` (false); la UI no lo
  edita a mano. Existe para que `get_settings` / el `load` no tengan que tocar el almacén de
  credenciales.
- `reset_settings({scope:"appearance"|…})`: se añade `ai` a un ámbito nuevo o al que corresponda;
  resetear `ai` **también borra la credencial** (coherencia con FR-004). Decisión para
  `open-questions.md`: `reset` de `ai` = `borrar_clave_ia`.
- FR-024 dice «tres datos»: son exactamente estos tres. La clave no cuenta (no está en
  `settings`).

---

## D13 — Onboarding

El asistente tiene hoy 4 pasos (`onboarding/+page.svelte`, `TOTAL = 4`). Se añade un **5.º paso
opcional** «Ayuda con IA»:

- Explica qué es, qué se envía (con ejemplo), a dónde y que es opcional.
- `TextField` para la clave + botón «Activar» (llama `guardar_clave_ia`) o «Omitir».
- Si se activa, muestra el `Select` de modelo (por defecto «automático»).
- «Omitir y usar los valores de fábrica» de la cabecera sigue saltándose todo.
- `TOTAL` pasa a 5; `ProgressBar` y textos de paso se ajustan.

No se toca `settings.onboarding.completed_at` ni su lógica.

---

## D14 — Pruebas

| Nivel | Qué |
|---|---|
| `cargo test` (test-first) | `domain/ia.rs`: prompt bien formado; `Anonimizador` aplicado a todos los fragmentos; `modelo_es_de_pago`; `barrer_texto_residual` (fixtures de eventos Windows); `analizar_respuesta` OK / vacía / malformada; `analizar_error` por código HTTP |
| `cargo test` | `platform/credenciales.rs`: guardar→leer→borrar→leer=None (usa un `TargetName` de prueba con sufijo aleatorio; se limpia) |
| `pnpm test` | `markdown.ts`: cada bloque, anidamiento, payloads hostiles salen inertes |
| `pnpm test:component` | `Markdown.svelte`, `ExplicacionModal.svelte` (progreso/error/ok), sección de Ajustes (vacío/con clave/probando/error), paso del asistente |
| `pnpm test:e2e` | `ia.spec.ts`: con un doble del comando (o clave de prueba), activar → explicar alerta → ver modal; simular `ia.network` → la pantalla sigue |
| `pnpm test:a11y` | modal y formularios nuevos |

**Red en pruebas**: ningún test unitario ni de componente llega a `openrouter.ai`. El transporte
(`ia_openrouter.rs`) se prueba con un `#[cfg(test)]` que apunta a un servidor local efímero, o se
deja como código fino cubierto por el e2e con doble. Se decide en `tasks.md`.

---

## D15 — Documentos normativos a actualizar (tareas de cierre)

- `docs/decisions.md` — ADR-046: fijar reqwest+rustls y windows/Win32_Security_Credentials como
  elección; anotar «sin permiso de capabilities».
- `.specify/memory/constitution.md` — tabla de pila: sustituir «por fijar en el plan» por las
  versiones/features reales. Cambio **patch** (precisión, no relaja norma). Lo aplica el usuario
  (hook). Se le prepara el parche.
- `docs/ui-contract.md` — los 6 comandos, sus DTO y los `error.ia.*`.
- `docs/architecture.md` — `domain/ia.rs`, `platform/ia_openrouter.rs`, `platform/credenciales.rs`
  y la regla «la red solo la origina Rust, bajo gesto».
- `docs/data-model.md` — grupo `ai` de `settings`; entidades efímeras.
- `docs/product-specification.md` — la capacidad opcional en el alcance.
- `docs/engineering-conventions.md` — dependencias nuevas.
- `docs/open-questions.md` — decisiones adoptadas: `CRED_PERSIST_LOCAL_MACHINE`, `reset` de `ai` =
  borrar credencial, tope 60 s, subconjunto markdown, sin streaming en v1, «tamaño razonable» del
  extracto = **8 000 caracteres** (valor propuesto, PROPUESTO).
- `historias.md` — `pnpm docs:build`.

---

## 7 — Cumplimiento del principio XVI, punto por punto

| Condición del principio XVI | Cómo se cumple |
|---|---|
| Apagada de fábrica; sin clave no hay ruta de red | El `Client` de `reqwest` se construye dentro del comando solo si `CredReadW` devuelve algo. Sin clave, los comandos de explicación devuelven `ia.no_key` sin tocar la red. SC-001. |
| Iniciada por la persona, nunca automática | Todos los comandos que salen a la red se disparan por un gesto: botón «Explícamelo», «Activar», «Probar», abrir el `Select`. Nada en el planificador, en el arranque ni en el motor de alertas. |
| Un solo proveedor, un solo destino | `const ENDPOINT: &str = "https://openrouter.ai/api/v1"` en `ia_openrouter.rs`. No hay parámetro de host. |
| Solo el detalle que la persona tiene delante | El `Origen` lo arma el frontend con datos ya en pantalla; el backend recompone desde inventario/alertas, sin añadir historial ni otras vistas (FR-008). |
| Anonimización antes de salir del proceso | En `domain/ia.rs`, con `reporting::anonimizar`, antes de componer el cuerpo. FR-009, SC-003. |
| La persona ve qué se envía (primera vez) | `explicar_detalle_tecnico` con `previewAcknowledged=false` devuelve el texto compuesto sin llamar al proveedor; el frontend lo muestra y, al confirmar, marca `settings.ai.preview_acknowledged=true` y reintenta. FR-010, FR-027. |
| Clave en el Administrador de credenciales (DPAPI) | D2. Nunca en SQLite/fichero/localStorage. FR-004, SC-004. |
| Llamada desde el backend, no el WebView | Comando Rust + `reqwest`. Sin `tauri-plugin-http`, sin `http` en capabilities. |
| Respuesta = contenido no confiable | `Markdown.svelte` sin `{@html}`; enlaces inertes; nada de la respuesta alimenta una decisión (FR-023, SC-007). El modal la rotula como orientación por IA (FR-013). |
| El fallo degrada la función, no la app | `AppError` por rama; el modal muestra el error, el resto de la pantalla intacto. FR-017, SC-005. |
| Sin telemetría propia; contenido fuera del log | `tracing` solo con metadatos (modelo, ok/error). FR-022. |
