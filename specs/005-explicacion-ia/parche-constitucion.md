# Parche patch de la constitución — precisar la tabla de pila (feature 005)

Revisión **patch** (precisión de redacción, no relaja ninguna norma): fija las dependencias que la
enmienda 1.8.0 ya introdujo «por fijar en el plan». El plan de `005-explicacion-ia` y el ADR-046 ya
las concretan.

**Lo aplica el usuario** — `.specify/memory/constitution.md` está protegido por el hook
`proteger-rutas.mjs`.

---

## Cambio 1 — tabla «Dependencias de Rust» (líneas 633–634)

**Sustituir:**

```
| Cliente HTTP (candidato `reqwest` con `rustls`) | por fijar en el plan (principio XVI, ADR-046) |
| Almacén de credenciales de Windows (candidato: crate `windows` ampliado, o `keyring`) | por fijar en el plan (principio XVI, ADR-046) |
```

**Por:**

```
| `reqwest` | 0.13, features `rustls` + `json` | solo principio XVI; ya en el árbol vía `tauri`, se le añade el backend TLS. Única dependencia nueva de la feature. ADR-046 |
```

> **Cambio respecto a lo previsto en la enmienda 1.8.0**: el almacén de credenciales **no** añade
> el crate `windows` ni `keyring`. Se implementa con FFI a mano contra `advapi32` (`CredReadW` /
> `CredWriteW` / `CredDeleteW`), siguiendo el patrón que el proyecto ya usa en
> `src-tauri/src/platform/energia.rs`. Cero dependencias nuevas para esa parte. Por eso la fila de
> «Almacén de credenciales» se elimina en vez de concretarse.

## Cambio 2 — footer (línea 790)

**Sustituir:**

```
**Versión**: 1.8.0 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08
```

**Por:**

```
**Versión**: 1.8.1 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08
```

## Cambio 3 — historial de enmiendas (tras la fila 1.8.0, línea 773)

**Añadir:**

```
| 1.8.1 | 2026-09-08 | Precisión (patch) de la tabla de pila (spec `005-explicacion-ia`): la única dependencia nueva es `reqwest` 0.13 (`rustls`, `json`), ya en el árbol vía `tauri`; el almacén de credenciales se hace con FFI a mano contra `advapi32`, sin crate nuevo (patrón de `platform/energia.rs`). Se confirma que la vía elegida (llamada desde Rust) no requiere permiso de *capabilities*. No cambia ni relaja ninguna norma. ADR-046 |
```

---

## Cómo aplicarlo

Opción rápida (bash, ejecútalo tú con `!`):

```sh
python - <<'PY'
p = ".specify/memory/constitution.md"
s = open(p, encoding="utf-8").read()
s = s.replace(
 "| Cliente HTTP (candidato `reqwest` con `rustls`) | por fijar en el plan (principio XVI, ADR-046) |\n| Almacén de credenciales de Windows (candidato: crate `windows` ampliado, o `keyring`) | por fijar en el plan (principio XVI, ADR-046) |\n",
 "| `reqwest` | 0.13, features `rustls` + `json` | solo principio XVI; ya en el árbol vía `tauri`, se le añade el backend TLS. Única dependencia nueva. ADR-046 |\n", 1)
s = s.replace(
 "**Versión**: 1.8.0 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08",
 "**Versión**: 1.8.1 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08", 1)
s = s.replace(
 "| 1.8.0 | 2026-09-08 | Principio XVI (asistencia con IA en la nube): capacidad opcional, apagada de fábrica, de proveedor y destino único, iniciada siempre por la persona, con anonimización obligatoria y clave en el almacén de credenciales de Windows. Los principios III y IX ganan una excepción **acotada** que no se aplica en estado de fábrica; ninguna otra norma se relaja. Entran dependencias nuevas (cliente HTTP, almacén de credenciales) y un permiso de red de Tauri, cada uno con su ADR. Requiere ADR-046 |\n",
 "| 1.8.0 | 2026-09-08 | Principio XVI (asistencia con IA en la nube): capacidad opcional, apagada de fábrica, de proveedor y destino único, iniciada siempre por la persona, con anonimización obligatoria y clave en el almacén de credenciales de Windows. Los principios III y IX ganan una excepción **acotada** que no se aplica en estado de fábrica; ninguna otra norma se relaja. Entran dependencias nuevas (cliente HTTP, almacén de credenciales) y un permiso de red de Tauri, cada uno con su ADR. Requiere ADR-046 |\n| 1.8.1 | 2026-09-08 | Precisión (patch) de la tabla de pila (spec `005-explicacion-ia`): la única dependencia nueva es `reqwest` 0.13 (`rustls`, `json`), ya en el árbol vía `tauri`; el almacén de credenciales se hace con FFI a mano contra `advapi32`, sin crate nuevo. La vía elegida (llamada desde Rust) no requiere permiso de *capabilities*. No cambia ni relaja ninguna norma. ADR-046 |\n", 1)
open(p, "w", encoding="utf-8", newline="\n").write(s)
print("aplicado")
PY
pnpm docs:build
```
