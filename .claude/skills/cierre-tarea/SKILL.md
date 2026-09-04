---
name: cierre-tarea
description: Matriz de qué documento actualizar según el tipo de cambio, y las nueve puertas de verificación que hay que pasar antes de dar una tarea por terminada. Úsala al cerrar un lote o una historia, cuando el usuario diga "termina", "cierra", "ya está" o "documenta el cambio", y antes de proponer un commit.
---

# Cierre de tarea

Todo cambio observable se documenta. **La excepción exige justificación, no al revés.**

## 1. Qué documento toca

| Si has cambiado… | Actualiza |
|---|---|
| Comportamiento visible del producto | `docs/product-specification.md` |
| Criterios de aceptación | `docs/user-stories.md` |
| Una regla de alerta, su histéresis o su deduplicación | `docs/alert-rules.md` |
| Un comando, un evento o la forma de un error | `docs/ui-contract.md` |
| Entidades, campos o retención | `docs/data-model.md` |
| Estructura interna o flujo entre capas | `docs/architecture.md` |
| Versiones, estructura de carpetas o linters | `docs/engineering-conventions.md` |
| Cobertura, niveles o suites | `docs/testing-strategy.md` |
| Un criterio visual, del catálogo o del arranque de la UI | `docs/ui-design.md` |
| Una decisión con alternativas descartadas | `docs/decisions.md` — skill `adr` |
| Un valor adoptado, una medición o una duda resuelta | `docs/open-questions.md` |
| Un aviso que has tenido que silenciar | `docs/known-issues.md` |
| Un recurso de terceros o su versión | `THIRD_PARTY_NOTICES.md` |
| Comandos o puesta en marcha | `README.md` |

Después: **`pnpm docs:build`**. El consolidado no puede ir por detrás.

## 2. Las nueve puertas

Se ejecutan en este orden. Un fallo de tipos invalida todo lo que venga después.

```sh
pnpm check      # tipos y accesibilidad: cero errores Y cero avisos
pnpm lint       # prettier + eslint
pnpm verify     # recursos, tokens, i18n, fronteras
pnpm test       # pruebas de Node
pnpm build      # compilación del frontend
pnpm docs:check # el consolidado no va por detrás
```

Y desde `src-tauri/`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 3. Si tocaste interfaz

Definición de terminado de `docs/ui-design.md` §8, que ninguna herramienta comprueba sola:

- [ ] Tema claro y oscuro correctos
- [ ] Acento del sistema y azul de respaldo
- [ ] 1024 × 560 sin recortes silenciosos
- [ ] Escalado 125 %, 150 % y 200 %
- [ ] Estados: cargando, vacío, no compatible, error de fuente, dato obsoleto
- [ ] Textos en español e inglés
- [ ] Teclado y foco verificados

## 4. Antes de proponer commit

- ¿Queda alguna decisión resuelta en silencio dentro del código? Si sí, va a
  `docs/open-questions.md` antes de confirmar.
- ¿Hay cambios no relacionados mezclados? Sepáralos.
- ¿El mensaje dice **por qué**, no solo qué? Los mensajes van en español y en imperativo.

**No hagas commit ni push sin autorización.** Cuando el trabajo esté listo, propón el mensaje y
pregunta.
