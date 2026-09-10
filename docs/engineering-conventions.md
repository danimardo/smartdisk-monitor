# Convenciones de ingeniería

Lo que un programador necesitaría asumir si no estuviera escrito. Todo lo de aquí es normativo pero
revisable: si una convención estorba, se cambia en este documento y se aplica en todo el proyecto,
no se hace una excepción local.

---

## 1. Pila y versiones

| Pieza | Versión | Nota |
|---|---|---|
| Tauri | 2.x | ADR-001 |
| Rust | edición 2021, MSRV 1.77 | se fija en `rust-toolchain.toml` para que CI y desarrollo coincidan |
| Node | 20 LTS | se fija en `.nvmrc` |
| Gestor de paquetes | pnpm | `packageManager` en `package.json`; el lockfile se versiona |
| SvelteKit | 2.x con `adapter-static` | ADR-014, SSR desactivado |
| Svelte | 5 con runes | `ui-design.md` §1 |
| TypeScript | 5.x, `strict: true` | sin `any` implícito, sin `@ts-ignore` sin justificar |
| Tailwind | 3.x | solo utilidades mapeadas desde tokens |
| SQLite | vía `rusqlite` con `bundled` | evita depender de la DLL del sistema |
| `reqwest` | 0.13, `default-features = false`, features `native-tls` + `json` | **solo** para la ayuda con IA (principio XVI, ADR-046); ya lo arrastra `tauri`. `native-tls` = SChannel del sistema, no `rustls` (que traería `aws-lc-sys`) |
| Vitest | 5.x | dos configuraciones: Node y navegador (ADR-027) |
| Playwright | 1.x | solo Chromium: es el motor del WebView2 (ADR-028) |
| `@axe-core/playwright` | 4.x | accesibilidad automática, ambos temas |

Windows mínimo soportado: **Windows 10 1809 (build 17763)** y **Windows Server 2016**, x64. Edge y
WebView2 llegan en realidad hasta Windows 10 1709, pero por debajo de 1809 las APIs de
almacenamiento dejan de comportarse de forma homogénea y no se van a probar ahí. Microsoft mantiene
actualizaciones de WebView2 en Windows 10 22H2 al menos hasta octubre de 2028.

WebView2 se empaqueta con el instalador sin conexión (ADR-020):

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": { "type": "offlineInstaller" }
    }
  }
}
```

**No lo cambies a `fixedRuntime` por ahorrar tamaño**: congelaría Chromium sin parches de seguridad
en un producto que no tiene actualizador automático. El razonamiento completo está en el ADR-020.

---

## 2. Estructura del proyecto

```
smartdisk-monitor/
  src/                          Frontend SvelteKit
    routes/                     Una carpeta por pantalla
      +layout.svelte            Arranque: tokens.css, tema, idioma, acento, suscripción a eventos
      +page.svelte              Panel general
      disks/[id]/               Detalle de disco
      alerts/                   Alertas
      events/                   Eventos
      tests/                    Pruebas y diagnóstico
      reports/                  Informes
      settings/                 Ajustes
      onboarding/               Asistente inicial
    lib/
      components/               Catálogo cerrado (del paquete de diseño)
      design/                   types, format, health, theme, accent
      i18n/                     Diccionarios
      api/                      Envoltorios tipados de invoke y listen. NINGUNA pantalla llama a
                                invoke directamente: siempre a través de aquí
      stores/                   Estado de aplicación en runes
      assets/                   Único recurso raster empaquetado: la foto del autor (ADR-052)
    design-system/              tokens.css, tokens.json, fonts/ (del paquete de diseño)

  src-tauri/
    src/
      main.rs
      commands/                 Un módulo por área; cada comando es una función fina que valida y
                                delega en domain/
      domain/                   Reglas de negocio. Sin dependencias de Tauri: es lo que se testea
      collectors/               smartctl, windows_storage, perf_counters, event_log
      alerts/                   Motor de reglas (docs/alert-rules.md)
      tests/                    Benchmark, chkdsk, autotest
      persistence/              Repositorios, migraciones, retención
      reporting/                Exportaciones y ZIP de diagnóstico
      platform/                 Envolturas de API de Windows, aisladas para poder simularlas
    migrations/                 SQL numerado, nunca editado una vez publicado
    capabilities/               Política Tauri de mínimo privilegio
    windows/app.manifest        requireAdministrator (ADR-004) + PerMonitorV2

  docs/                         Documentación normativa
  design/                       Bocetos navegables (.dc.html). Referencia visual, no código

  AGENTS.md                     Instrucciones para agentes de IA: fuente canónica
  CLAUDE.md                     Importa AGENTS.md y añade lo específico de Claude Code
  GEMINI.md                     Importa AGENTS.md y añade lo específico de Gemini
  CODEX.md                      Puntero a AGENTS.md, que Codex ya lee de forma nativa
  .claude/rules/                Reglas por ámbito; se cargan al tocar sus `paths:`
  .claude/skills/               Procedimientos; se cargan al activarse
  .claude/settings.json         Permisos y hooks (enforcement)
  .claude/hooks/                Scripts de los hooks
```

### Instrucciones para agentes de IA

`AGENTS.md` es el núcleo y se carga en cada sesión: **objetivo, menos de 200 líneas**. No se
escribe ahí nada que un agente pueda deducir leyendo el repositorio, porque cuanto más ruido, menos
adherencia a lo que importa.

| Si algo… | Va a |
|---|---|
| Hace falta en cualquier tarea | `AGENTS.md` |
| Solo al tocar cierto código | `.claude/rules/<tema>.md` con `paths:` |
| Es un procedimiento de varios pasos | `.claude/skills/<nombre>/SKILL.md` |
| Es estado o historia del producto | `docs/` o `specs/` |
| Debe cumplirse siempre y de forma determinista | Linter, prueba, verificador o CI |

Los `@imports` **no ahorran contexto**: se expanden al arrancar. Lo que ahorra es `paths:` en las
reglas y la carga diferida de las skills, de las que en el arranque solo entran nombre y
descripción.

**Regla de dependencias.** `domain/` no conoce Tauri, ni Windows, ni SQLite: recibe datos y devuelve
decisiones. Es lo que permite probar el motor de alertas con fixtures y sin hardware.

**Ninguna pantalla llama a `invoke` directamente.** Todo pasa por `src/lib/api/`, que es donde viven
los tipos generados, el manejo de `AppError` y la suscripción a eventos. Así, cuando una firma
cambia, rompe en un sitio y no en once.

---

## 3. Calidad

| Herramienta | Qué exige |
|---|---|
| `rustfmt` | formato; CI falla si hay diferencias |
| `clippy` | `-D warnings`; nada de `unwrap()` fuera de tests |
| `eslint` + `svelte-check` | sin errores de tipo ni de accesibilidad |
| `prettier` | formato del frontend |
| `cargo test` / `vitest` | pruebas unitarias |
| `ts-rs` | los DTO generados deben coincidir con los versionados (puerta pendiente: se activa con el primer DTO real) |
| `@vitest/coverage-v8` | umbrales del frontend, en `vitest.config.ts` |
| `cargo-llvm-cov` | umbrales del backend, en CI |

### Comprobaciones propias del proyecto

Estas no las da ninguna herramienta estándar; hay que escribirlas, y son las que impiden que el
sistema de diseño se erosione:

1. **Sin valores visuales literales.** Un lint que falle si un `.svelte` contiene un color
   hexadecimal, un `rgb(`, un `border-radius` en px, un `box-shadow` literal o un `font-size` en px
   fuera de `tokens.css`.
2. **Sin `backdrop-filter` a mano.** Solo puede aparecer en `tokens.css`.
3. **Sin literales de interfaz.** Todo texto visible, incluidos `aria-label`, `title` y `alt`, debe
   venir de `t()` o `tp()`. Un lint que detecte cadenas literales en marcado.
4. **Paridad de diccionarios.** `es.json` y `en.json` deben tener exactamente el mismo juego de
   claves. Ya se comprueba trivialmente y evita textos que solo existen en un idioma.
5. **La tipografía existe y es la que dice ser.** La compilación falla si falta cualquiera de los dos
   `.woff2` de `design-system/fonts/` o si su SHA-256 no coincide con el registrado en
   `THIRD_PARTY_NOTICES.md`: sin ellos la aplicación se ve distinta de los bocetos aprobados y nadie
   se entera. El empaquetado debe copiarlos junto con `OFL.txt`.
6. **Sin permisos Tauri nuevos.** Un diff sobre `capabilities/` que exija revisión explícita.
7. **Decodificación de procesos auxiliares.** Test con volcados reales de `chkdsk` (CP1252) y de
   `fsutil` (CP850) que verifique que la detección de `open-questions.md` §Q elige bien en ambos.
   Los volcados se guardan como fixtures: es la única forma de que una regresión aquí se note.

---

## 4. Pruebas

| Nivel | Qué cubre | Dónde |
|---|---|---|
| Unitarias Rust | parsers de smartctl, normalización, reglas de alerta, retención, rutas seguras | `domain/`, `alerts/` |
| Fixtures | salidas reales anonimizadas de ATA, NVMe, USB, RAID y VM | `src-tauri/tests/fixtures/` |
| Integración | `smartctl` simulado: salidas válidas, timeouts, códigos de salida con bits, JSON corrupto | `src-tauri/tests/` |
| Migraciones | migrar desde cada versión publicada hasta la actual, con copia previa | `persistence/` |
| Unitarias TS | `format`, `health`, `accent`, `i18n` | `pnpm test`, jsdom |
| Componentes | estados vacío, cargando, no compatible, error y dato obsoleto de cada componente, más lo que solo se ve en un navegador real: contraste sobre material, respaldo sin `backdrop-filter`, foco visible | `pnpm test:component`, Chromium real. Sufijo `*.browser.test.ts` |
| Interfaz | arranque, chrome, navegación, tema, tipografía, errores de consola | `pnpm test:e2e`, Playwright con IPC propio |
| Accesibilidad | foco, teclado, contraste AA, `prefers-reduced-motion` | automatizado donde se pueda, lista de comprobación donde no |
| Visuales | ambos temas, acento del sistema y de respaldo, sin `backdrop-filter`, 1024 × 560 y 1280 × 720, escalado 125/150/200 % | capturas comparadas; `tools/scale-check.html` como banco de pruebas |
| Manuales | hardware real, sin exigir una marca concreta | documentadas en el informe de Fase 0 |

**Los datos fixture nunca contienen números de serie ni nombres de equipo reales.** Se anonimizan al
capturarlos, no al usarlos.

---

## 5. Git y entrega

- Rama principal protegida; el trabajo va en ramas por historia (`us-030-alertas-agrupadas`).
- Commits en imperativo, en español, referenciando la historia.
- Un *pull request* no se fusiona sin la definición de terminado de la historia y, si toca interfaz,
  la de `ui-design.md` §8.
- Versionado semántico. Nombre y versión salen del manifiesto (ADR-011): no se escriben a mano en
  ningún otro sitio.
- Las publicaciones son manuales en GitHub, sin actualizador automático (ADR-007).
- La **clave de demostración de la ayuda con IA** (ADR-054) se pasa a la compilación por la variable
  de entorno `SDM_OPENROUTER_DEMO_KEY` —secreto `OPENROUTER_DEMO_KEY` en el flujo de Release; en
  local, `set SDM_OPENROUTER_DEMO_KEY=… && pnpm app:build`—. `build.rs` la lee, la ofusca y la
  compila en el binario; sin la variable, el binario sale sin ella y la ayuda con IA sigue exigiendo
  clave propia. La clave **nunca** se escribe en un fichero versionado (`.gitignore` cubre
  `src-tauri/.env.local` por si se prefiere exportarla desde ahí a mano).

---

## 6. Datos y rutas en desarrollo

En desarrollo, la base de datos **no** va a `C:\ProgramData\SmartDisk Monitor\`: va a una carpeta
local ignorada por git, para no mezclar datos reales con pruebas ni exigir elevación en cada
ejecución de test. La ruta se resuelve siempre por función, nunca por literal, y el modo se decide
por variable de entorno.

Ejecutar la aplicación completa **sí** requiere elevación, también en desarrollo: es la única forma
de que lo que se prueba sea lo que se entrega. En la práctica, `pnpm app:dev` muestra el diálogo de
UAC en cada arranque.

Las suites de prueba **no** necesitan elevación, con una excepción: la de aplicación real
(`test:e2e:app`), que arranca el ejecutable y por tanto debe lanzarse desde un terminal de
administrador para que el hijo herede la elevación y UAC no bloquee la automatización
(`docs/testing-strategy.md` §11).
