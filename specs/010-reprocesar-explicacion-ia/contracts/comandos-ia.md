# Fase 1 — Contrato de comandos: delta de la 010 sobre `docs/ui-contract.md` §3.10

Feature `010-reprocesar-explicacion-ia`. **Ningún comando nuevo.** Un campo opcional nuevo en
`OrigenExplicacion` y reutilización directa de dos comandos que ya existen fuera de este bloque
(`set_setting`, `listar_modelos_ia`). El contrato base es `docs/ui-contract.md` §3.10; aquí solo lo
que cambia.

---

## `explicar_detalle_tecnico` — `OrigenExplicacion` gana un campo opcional

```ts
type OrigenExplicacion = {
  tipo: "alerta" | "smart" | "evento";
  deviceId: string | null;
  alertGroupId: string | null;
  eventId: string | null;
  idioma: "es" | "en";
  revision: "ninguna" | "enviar_igual" | "quitar_fragmentos";
  previewConfirmada: boolean;
  modeloSolicitado: string | null;   // NUEVO (FR-002/FR-003)
};
```

Sigue la misma convención que `deviceId`/`alertGroupId`/`eventId`: campo **obligatorio**, `null`
cuando no aplica — nunca ausente.

- **`null`**: comportamiento actual, sin cambios — se usa `settings.ai.model`.
- **Con valor**: sustituye a `settings.ai.model` **solo para esta llamada**. No se escribe en
  `settings`; para eso está "fijar como predeterminado" (más abajo).
- No se valida contra el catálogo de OpenRouter en el backend (igual que `settings.ai.model` no se
  valida hoy al leerlo): un identificador que el proveedor no reconoce produce el error `ia.*`
  habitual, sin caso especial.
- El resto del flujo (anonimización, vista previa, revisión de fragmentos, `ResultadoExplicacion`)
  **no cambia de forma**: reprocesar es la misma llamada con este único campo distinto, tal como
  exige FR-003/FR-004.

`ResultadoExplicacion` no cambia de forma. `estado: "ok"` sigue trayendo `modeloUsado` (el modelo
que realmente respondió) — es lo que permite que la interfaz sepa qué modelo etiquetar en el
historial y qué modelo ofrecer para "fijar como predeterminado" (FR-008).

---

## "Fijar como predeterminado" — reutiliza `set_setting`, sin comando nuevo

```ts
invoke<void>("set_setting", { key: "settings.ai.model", value: modeloUsado })
```

Es el mismo comando genérico de ajustes que ya usan Ajustes y el asistente inicial para este mismo
valor (`docs/ui-contract.md` §3, bloque de ajustes). Tras guardar, la interfaz vuelve a pedir
`estado_ia()` para que `usandoClaveCompartida`/`modelo` queden al día en el resto de la app
(Ajustes, y la próxima vez que se abra este mismo modal).

Por FR-008, la interfaz solo ofrece este botón sobre una respuesta obtenida reprocesando con un
modelo elegido explícitamente en el selector — nunca sobre la respuesta inicial del modo automático.
Esa es una regla de la interfaz (`explicacion.svelte.ts`), no del comando: `set_setting` no distingue
de dónde viene el valor.

---

## `listar_modelos_ia` — sin cambio de forma, nuevo criterio de uso en el frontend

```ts
type ModeloIaWire = { id: string; nombre: string; esDePago: boolean };
```

Sin cambios en el backend. Lo que cambia es cómo el frontend construye las opciones del selector en
los dos sitios que lo usan (Ajustes y este modal, componente `AiModelSelect.svelte`):

- Un modelo con `esDePago: true` se marca `disabled` en la opción **si y solo si**
  `estado_ia().usandoClaveCompartida` es `true` (ya expuesto, ADR-054). No es un campo nuevo del
  wire: es una regla de presentación calculada en el cliente a partir de dos datos que ya cruzan la
  frontera.
- El selector de este modal, además, excluye la entrada `"openrouter/free"` (modo automático) de las
  opciones (FR-006) — el de Ajustes la sigue incluyendo, sin cambios.

---

## Sin cambios

- `estado_ia`, `guardar_clave_ia`, `activar_ayuda_ia_compartida`, `probar_clave_ia`,
  `borrar_clave_ia`, `establecer_envio_sin_revision`: firma y comportamiento idénticos a
  `docs/ui-contract.md` §3.10.
- El informe HTML con resumen IA (spec 009, §3.7): sigue leyendo `settings.ai.model` exactamente
  igual; hereda el nuevo valor por defecto sin que su contrato cambie una línea.
