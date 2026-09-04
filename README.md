# SmartDisk Monitor

Aplicación local de supervisión de discos para Windows, desarrollada con Tauri 2, Rust, Svelte, TypeScript y Tailwind CSS.

Estado actual: **esqueleto de Fase 0 en marcha**. La especificación está cerrada y el proyecto
compila y arranca; los comandos del backend devuelven `not_implemented` hasta que se implemente cada
recopilador.

## Puesta en marcha

```sh
pnpm install          # requiere Node 20+ y pnpm
pnpm app:dev          # levanta Vite y la aplicación Tauri
```

`pnpm app:dev` muestra el diálogo de UAC en cada arranque: la aplicación se ejecuta elevada por
necesidad de acceso a los dispositivos (ADR-004). Las pruebas no lo necesitan, salvo la suite de
aplicación real.

| Comando | Qué hace |
|---|---|
| `pnpm app:dev` | Aplicación completa en modo desarrollo |
| `pnpm app:build` | Instalador NSIS con WebView2 sin conexión |
| `pnpm check` | Tipos y accesibilidad del frontend |
| `pnpm test` | Lógica del frontend, en Node |
| `pnpm test:component` | Componentes en un Chromium real |
| `pnpm test:e2e` | Interfaz completa con Playwright y el IPC simulado |
| `pnpm test:a11y` | Accesibilidad con axe, seis pantallas por dos temas |
| `pnpm verify` | Recursos redistribuidos, tokens del diseño e i18n |
| `pnpm docs:build` | Regenera `historias.md` |
| `cargo test` / `cargo clippy` | Backend, desde `src-tauri/` |

`pnpm verify` es el que impide que el sistema de diseño se erosione: comprueba los hashes de la
tipografía y de `smartctl`, que ningún componente lleve colores o radios literales, y que los dos
diccionarios estén sincronizados.

Antes de escribir código conviene leer [`docs/open-questions.md`](docs/open-questions.md): recoge
todo lo que la especificación dejaba a interpretación y el valor que se ha adoptado en cada caso.
Las entradas marcadas `PROPUESTO` son valores por defecto pendientes de revisión; las marcadas
`ABIERTO` bloquean la historia que las cita.

[`historias.md`](historias.md) reúne todo el material normativo en un solo documento, pensado para
entregárselo entero a una herramienta de generación o a quien se incorpore al proyecto. Se genera
con `python tools/build-historias.py` y no se edita a mano.

El sistema de diseño aprobado se encuentra en [`Design-system/`](Design-system/) y es vinculante para la implementación de la interfaz.

## Documentación

- [Especificación del producto](docs/product-specification.md)
- [Arquitectura](docs/architecture.md)
- [Modelo de datos](docs/data-model.md)
- [Historias de usuario](docs/user-stories.md)
- [Reglas de alerta](docs/alert-rules.md)
- [Contrato UI ↔ backend](docs/ui-contract.md)
- [Convenciones de ingeniería](docs/engineering-conventions.md)
- [Estrategia de testing](docs/testing-strategy.md)
- [Cuestiones abiertas](docs/open-questions.md)
- [Backlog y versiones](docs/roadmap.md)
- [Decisiones técnicas](docs/decisions.md)
- [Fallos conocidos y silencios](docs/known-issues.md)
- [Constitución del proyecto](.specify/memory/constitution.md)
- [Instrucciones para agentes de IA](AGENTS.md)
- [Todo lo anterior en un solo documento](historias.md)
- [Entrega del sistema de diseño](Design-system/HANDOFF.md)
- [Reglas vinculantes de interfaz](Design-system/AGENTS.md)

## Identidad del proyecto

- Producto: SmartDisk Monitor
- Autor: Daniel Diez Mardomingo
- Repositorio previsto: https://github.com/danimardo/smartdisk-monitor
- Licencia del código propio: MIT
- Plataforma inicial: Windows x64
