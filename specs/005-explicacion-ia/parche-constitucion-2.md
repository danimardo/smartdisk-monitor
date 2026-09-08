# Corrección de la 1.8.1 — `rustls` → `native-tls`

Al implementar se comprobó que `rustls` en reqwest 0.13 arrastra `aws-lc-sys` (BoringSSL
vendorizado, build de C) sin forma de evitarlo. En un proyecto solo-Windows la vía correcta es
`native-tls` → SChannel, la pila TLS del sistema (crate `schannel`, FFI puro). Es una precisión de
la misma revisión **patch** 1.8.1 (mismo día, antes de fusionar código), no una enmienda nueva.

**Lo aplica el usuario** (fichero con hook). Comando listo:

```sh
cd /f/Apps/smartdisk && python - <<'PY'
p = ".specify/memory/constitution.md"
s = open(p, encoding="utf-8").read()
s = s.replace(
 "| `reqwest` | 0.13, features `rustls` + `json` | solo principio XVI; ya en el árbol vía `tauri`, se le añade el backend TLS. Única dependencia nueva. ADR-046 |",
 "| `reqwest` | 0.13, `default-features = false`, features `native-tls` + `json` | solo principio XVI; ya en el árbol vía `tauri`, se le añade el backend TLS. `native-tls` = SChannel (pila TLS del sistema), no `rustls` (arrastra `aws-lc-sys`). Única dependencia nueva. ADR-046 |",
 1)
s = s.replace(
 "la única dependencia nueva es `reqwest` 0.13 (`rustls`, `json`), ya en el árbol vía `tauri`;",
 "la única dependencia nueva es `reqwest` 0.13 (`native-tls`, `json` — SChannel, la pila TLS del sistema; `rustls` se descartó porque arrastra `aws-lc-sys`), ya en el árbol vía `tauri`;",
 1)
open(p, "w", encoding="utf-8", newline="\n").write(s)
print("aplicado" if "native-tls" in s and "rustls" in s else "revisar")
PY
pnpm docs:build
```
