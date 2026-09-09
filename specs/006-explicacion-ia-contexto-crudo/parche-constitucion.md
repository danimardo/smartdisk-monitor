# Enmienda de la constitución — principio XVI, versión 1.8.1 → 1.9.0 (feature 006)

Enmienda **minor**: amplía una excepción ya existente (principio XVI) y añade un opt-in que relaja
al margen la garantía «sin datos identificables» para quien lo active. Ninguna otra norma se toca.

**Razón del cambio**: la ayuda con IA hoy envía una reconstrucción mínima (identificador de regla +
valor + tendencia + contexto del disco), no el detalle técnico que la persona ve. El modelo explica
a ciegas. La feature `006-explicacion-ia-contexto-crudo` le añade el volcado crudo de `smartctl` y
el contenido del suceso de Windows que originó la alerta —ambos abribles en la propia pantalla— y
un modo opcional «enviar sin revisar».

**Qué se rompe**: dos viñetas de la redacción actual de XVI («Solo el detalle técnico que la
persona ya tiene delante» y «Anonimización obligatoria…») entran en conflicto con la feature. Esta
enmienda las reescribe. El código de la 006 no se fusiona hasta que esté aplicada.

**Documentos normativos a actualizar en consecuencia**: `docs/decisions.md` (ADR-047 nuevo + retoque
de la viñeta «Datos enviados» de ADR-046), `docs/data-model.md` (cuarta clave de `settings.ai`),
`docs/ui-contract.md` (comando `establecer_envio_sin_revision`, campo nuevo en `EstadoIaWire`),
`docs/open-questions.md` (`MAX_DETALLE_CHARS` 8 000 → 40 000).

**Lo aplica la persona** — `.specify/memory/constitution.md` está protegido por el hook
`proteger-rutas.mjs`.

---

## Cambio 1 — viñeta «Solo el detalle técnico…» (principio XVI)

**Sustituir:**

```
- **Solo el detalle técnico que la persona ya tiene delante.** Se envía el texto de la alerta o del
  detalle SMART visible en ese momento y el mínimo contexto para explicarlo. Nunca el inventario,
  el historial, la configuración ni datos de otras pantallas.
```

**Por:**

```
- **Solo el detalle técnico del suceso que se explica.** Se envía el detalle técnico de la alerta o
  del detalle SMART: su resumen, el volcado crudo de la herramienta de diagnóstico del disco
  (`smartctl`) y, en alertas nacidas de un suceso de Windows, el mensaje y los campos de datos de
  ese suceso. Todo ello es información que la persona puede abrir en la propia pantalla. Nunca el
  inventario completo, el historial de otras métricas, la configuración, datos de otras pantallas
  ni de otros discos. El bloque de metadatos de sistema del suceso —nombre del equipo, principal de
  seguridad, identificadores de proceso— no se envía.
```

## Cambio 2 — viñeta «Anonimización obligatoria…» (principio XVI)

**Sustituir:**

```
- **Anonimización obligatoria antes de que el texto salga del proceso.** Rigen las mismas reglas
  del principio IX y del XV: números de serie, nombre del equipo, nombre de usuario, rutas con
  perfil de usuario y etiquetas de volumen se sustituyen por marcadores, con sustitución
  consistente dentro de una misma petición. La anonimización ocurre en el dominio (Rust), no en la
  interfaz.
```

**Por:**

```
- **Anonimización obligatoria antes de que el texto salga del proceso.** Rigen las mismas reglas
  del principio IX y del XV: números de serie, identificador mundial del disco (WWN), nombre del
  equipo, nombre de usuario, rutas con perfil de usuario, etiquetas de volumen, identificadores de
  seguridad (SID) y rutas internas de dispositivo se sustituyen por marcadores, con sustitución
  consistente dentro de una misma petición. Los datos que no identifican a una persona —marca,
  modelo, interfaz y firmware del disco— se conservan como contexto. La anonimización ocurre en el
  dominio (Rust), no en la interfaz, y combina la extracción de campos conocidos con un barrido de
  patrones; lo que no pueda garantizarse limpio se muestra a la persona para que decida entre
  enviarlo, quitarlo o cancelar. La persona puede activar de forma explícita un modo «enviar sin
  revisar» que asume ese riesgo: apagado de fábrica, su activación es un consentimiento informado
  adicional al de la vista previa, y no desactiva la anonimización por campos y patrones, solo la
  revisión manual del texto libre residual.
```

## Cambio 3 — línea de ADR al final del principio XVI

**Sustituir:**

```
Requiere el ADR-046, que fija el proveedor, el endpoint y las dependencias.
```

**Por:**

```
Requiere el ADR-046, que fija el proveedor, el endpoint y las dependencias, y el ADR-047, que fija
el alcance del dato enviado (volcado crudo y contenido del suceso), la anonimización en capas y el
modo «enviar sin revisar».
```

## Cambio 4 — footer

**Sustituir:**

```
**Versión**: 1.8.1 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08
```

**Por:**

```
**Versión**: 1.9.0 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08
```

## Cambio 5 — historial de enmiendas (tras la fila 1.8.1)

**Añadir:**

```
| 1.9.0 | 2026-09-08 | Principio XVI, dos viñetas (spec `006-explicacion-ia-contexto-crudo`): «detalle técnico visible» se amplía para incluir el volcado crudo de `smartctl` y el mensaje y los campos de datos del suceso de Windows que originó la alerta —información que la persona ya puede abrir en pantalla—, manteniendo la prohibición de inventario, historial, configuración, otras pantallas y otros discos, y descartando el bloque de metadatos de sistema del suceso. La cláusula de anonimización añade WWN, SID y rutas de dispositivo a la lista de sustituciones y reconoce un modo **opcional** «enviar sin revisar», apagado de fábrica y con consentimiento propio, que omite solo la revisión manual del texto libre residual (la anonimización por campos y patrones se sigue aplicando). Es `minor` porque **amplía** una excepción ya existente y añade un opt-in que relaja al margen la garantía «sin datos identificables» para quien lo active; ninguna otra norma se toca. Requiere ADR-047 |
```

---

## Cómo aplicarlo

Ejecútalo tú con `!` (bash / Git Bash):

```sh
cd /f/Apps/smartdisk && python - <<'PY'
p = ".specify/memory/constitution.md"
s = open(p, encoding="utf-8").read()
orig = s

c1_old = """- **Solo el detalle técnico que la persona ya tiene delante.** Se envía el texto de la alerta o del
  detalle SMART visible en ese momento y el mínimo contexto para explicarlo. Nunca el inventario,
  el historial, la configuración ni datos de otras pantallas."""
c1_new = """- **Solo el detalle técnico del suceso que se explica.** Se envía el detalle técnico de la alerta o
  del detalle SMART: su resumen, el volcado crudo de la herramienta de diagnóstico del disco
  (`smartctl`) y, en alertas nacidas de un suceso de Windows, el mensaje y los campos de datos de
  ese suceso. Todo ello es información que la persona puede abrir en la propia pantalla. Nunca el
  inventario completo, el historial de otras métricas, la configuración, datos de otras pantallas
  ni de otros discos. El bloque de metadatos de sistema del suceso —nombre del equipo, principal de
  seguridad, identificadores de proceso— no se envía."""

c2_old = """- **Anonimización obligatoria antes de que el texto salga del proceso.** Rigen las mismas reglas
  del principio IX y del XV: números de serie, nombre del equipo, nombre de usuario, rutas con
  perfil de usuario y etiquetas de volumen se sustituyen por marcadores, con sustitución
  consistente dentro de una misma petición. La anonimización ocurre en el dominio (Rust), no en la
  interfaz."""
c2_new = """- **Anonimización obligatoria antes de que el texto salga del proceso.** Rigen las mismas reglas
  del principio IX y del XV: números de serie, identificador mundial del disco (WWN), nombre del
  equipo, nombre de usuario, rutas con perfil de usuario, etiquetas de volumen, identificadores de
  seguridad (SID) y rutas internas de dispositivo se sustituyen por marcadores, con sustitución
  consistente dentro de una misma petición. Los datos que no identifican a una persona —marca,
  modelo, interfaz y firmware del disco— se conservan como contexto. La anonimización ocurre en el
  dominio (Rust), no en la interfaz, y combina la extracción de campos conocidos con un barrido de
  patrones; lo que no pueda garantizarse limpio se muestra a la persona para que decida entre
  enviarlo, quitarlo o cancelar. La persona puede activar de forma explícita un modo «enviar sin
  revisar» que asume ese riesgo: apagado de fábrica, su activación es un consentimiento informado
  adicional al de la vista previa, y no desactiva la anonimización por campos y patrones, solo la
  revisión manual del texto libre residual."""

c3_old = "Requiere el ADR-046, que fija el proveedor, el endpoint y las dependencias."
c3_new = ("Requiere el ADR-046, que fija el proveedor, el endpoint y las dependencias, y el ADR-047, que fija\n"
          "el alcance del dato enviado (volcado crudo y contenido del suceso), la anonimización en capas y el\n"
          "modo «enviar sin revisar».")

c4_old = "**Versión**: 1.8.1 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08"
c4_new = "**Versión**: 1.9.0 | **Ratificada**: 2026-09-04 | **Última enmienda**: 2026-09-08"

row_19 = "| 1.9.0 | 2026-09-08 | Principio XVI, dos viñetas (spec `006-explicacion-ia-contexto-crudo`): «detalle técnico visible» se amplía para incluir el volcado crudo de `smartctl` y el mensaje y los campos de datos del suceso de Windows que originó la alerta —información que la persona ya puede abrir en pantalla—, manteniendo la prohibición de inventario, historial, configuración, otras pantallas y otros discos, y descartando el bloque de metadatos de sistema del suceso. La cláusula de anonimización añade WWN, SID y rutas de dispositivo a la lista de sustituciones y reconoce un modo **opcional** «enviar sin revisar», apagado de fábrica y con consentimiento propio, que omite solo la revisión manual del texto libre residual (la anonimización por campos y patrones se sigue aplicando). Es `minor` porque **amplía** una excepción ya existente y añade un opt-in que relaja al margen la garantía «sin datos identificables» para quien lo active; ninguna otra norma se toca. Requiere ADR-047 |"

for old, new, name in [
    (c1_old, c1_new, "cambio 1"),
    (c2_old, c2_new, "cambio 2"),
    (c3_old, c3_new, "cambio 3"),
    (c4_old, c4_new, "cambio 4 (footer)"),
]:
    if old not in s:
        raise SystemExit(f"NO ENCONTRADO: {name}")
    s = s.replace(old, new, 1)

# cambio 5: insertar la fila 1.9.0 justo después de la fila 1.8.1 (que termina en '. ADR-046 |')
marca = "No cambia ni relaja ninguna norma. ADR-046 |\n"
if marca not in s:
    raise SystemExit("NO ENCONTRADO: fila 1.8.1 para anclar la 1.9.0")
s = s.replace(marca, marca + row_19 + "\n", 1)

if s == orig:
    raise SystemExit("sin cambios")
open(p, "w", encoding="utf-8", newline="\n").write(s)
print("aplicado: 5 cambios")
PY
pnpm docs:build
```

Tras aplicarlo, comprobar:

```sh
grep -n "1.9.0" .specify/memory/constitution.md
grep -n "enviar sin revisar" .specify/memory/constitution.md
pnpm verify
```
