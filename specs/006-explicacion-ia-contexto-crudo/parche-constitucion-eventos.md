# Precisión de la constitución — principio XVI, viñeta «detalle técnico visible» (ADR-049)

Revisión **patch** (precisión de redacción, no relaja ninguna norma). El contenido de un suceso de
Windows ya está autorizado por la enmienda 1.9.0; esto solo nombra también el **detalle de un
evento** (pantalla Eventos) como origen, no solo la alerta que nació de él.

**Razón**: ADR-049 añade el botón «Explícamelo en lenguaje claro» al panel de detalle de un evento.
La redacción actual de la viñeta dice «en alertas nacidas de un suceso de Windows», que se lee como
si solo valiera desde una alerta.

**Qué no se rompe**: nada. Mismo dato (mensaje + campos `EventData`), misma anonimización, mismo
principio.

**Lo aplica la persona** — `.specify/memory/constitution.md` está protegido por el hook
`proteger-rutas.mjs`.

---

## Cambio único — viñeta «Solo el detalle técnico del suceso que se explica» (principio XVI)

**Sustituir:**

```
  (`smartctl`) y, en alertas nacidas de un suceso de Windows, el mensaje y los campos de datos de
  ese suceso. Todo ello es información que la persona puede abrir en la propia pantalla. Nunca el
```

**Por:**

```
  (`smartctl`) y, en una alerta nacida de un suceso de Windows o en el detalle de un evento del
  registro de Windows, el mensaje y los campos de datos de ese suceso. Todo ello es información que
  la persona puede abrir en la propia pantalla. Nunca el
```

---

## Cómo aplicarlo

Ejecútalo tú con `!` (bash / Git Bash):

```sh
cd /f/Apps/smartdisk && python - <<'PY'
p = ".specify/memory/constitution.md"
s = open(p, encoding="utf-8").read()
old = (
    "  (`smartctl`) y, en alertas nacidas de un suceso de Windows, el mensaje y los campos de datos de\n"
    "  ese suceso. Todo ello es información que la persona puede abrir en la propia pantalla. Nunca el\n"
)
new = (
    "  (`smartctl`) y, en una alerta nacida de un suceso de Windows o en el detalle de un evento del\n"
    "  registro de Windows, el mensaje y los campos de datos de ese suceso. Todo ello es información que\n"
    "  la persona puede abrir en la propia pantalla. Nunca el\n"
)
if old not in s:
    raise SystemExit("NO ENCONTRADO: la viñeta a precisar")
open(p, "w", encoding="utf-8", newline="\n").write(s.replace(old, new, 1))
print("aplicado")
PY
pnpm docs:build
```

Tras aplicarlo:

```sh
grep -n "detalle de un evento del" .specify/memory/constitution.md
pnpm verify
```
