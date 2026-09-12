# Fase 0 — Investigación: Eventos de Windows en el detalle de disco

No queda ningún `NEEDS CLARIFICATION` en el Technical Context del plan: el alcance ya se acordó
con el usuario antes de escribir la spec (resumen con enlace, no lista completa embebida; 5
eventos). Este documento recoge las decisiones de diseño técnico necesarias para implementar ese
alcance, siguiendo patrones ya existentes en el proyecto en vez de inventar uno nuevo.

## D1 · Reutilizar el patrón exacto del widget "Últimos eventos" del panel general

**Decisión**: la nueva sección es una `Card` con un `action` snippet ("Ver todos" enlazando a
`/events?deviceId={disk.id}`) y, dentro, cada evento como `EventRow` con
`href="/events?focus={id}"`.

**Razón**: `src/routes/+page.svelte` (líneas ~244-264) ya resuelve exactamente este problema —
mostrar un resumen de eventos con salida hacia el detalle completo— para el caso global (todos los
discos, 3 eventos). Repetir el patrón en vez de inventar uno propio cumple el principio de
simplicidad (constitución, Orden de prioridades §II.5) y no añade ningún componente nuevo al
catálogo.

**Alternativas consideradas**: un componente nuevo `EventSummaryCard` que encapsule la lista +
enlace. Descartada: la composición actual (`Card` + `EventRow` + un `{#each}`) ya es tan corta como
sería ese componente, y crear uno nuevo exigiría justificar por qué `Card` no basta
(`ui-design.md` §3), cosa que no ocurre aquí.

## D2 · Límite de 5 eventos, pedido con `deviceId` y `limit`

**Decisión**: `getSystemEvents({ deviceId: disk.id, limit: 5 })`, sin `levels` ni `providers` ni
`cursor`.

**Razón**: acordado con el usuario. Es mayor que los 3 del resumen global (que mezcla todos los
discos) porque aquí el resumen es de un solo disco y puede permitirse algo más de detalle sin
saturar la pantalla.

**Alternativas consideradas**: sincronizar el listado con el selector de rango (1 h/24 h/7 d/30 d)
de las gráficas de arriba, usando `fromUtc`/`toUtc` (que el comando ya admite aunque hoy ninguna
pantalla los use). Descartada explícitamente por el usuario en la fase de aclaración: quiere un
resumen de los más recientes, no una vista correlacionada con el intervalo de las gráficas.

## D3 · Sin filtros, paginación ni panel de detalle propios

**Decisión**: la sección no lleva `FilterBar`, no pide más páginas con `nextCursor`, y no abre un
panel de detalle in situ con el XML — todo eso ya existe en `/events` y se alcanza con el enlace
"Ver todos" o pulsando un evento.

**Razón**: acordado con el usuario; evita duplicar la pantalla de Eventos casi entera dentro de la
de detalle de disco, que era la opción explícitamente descartada.

## D4 · Aislamiento de fallos, mismo patrón que las dos gráficas ya existentes

**Decisión**: la petición de eventos vive en su propio estado (`serie`/`error`/`loading`-like, en
este caso `eventos`/`error`/`loading`), en un `$effect` que depende de `disk.id`, con su función de
limpieza — el mismo patrón que ya usan `serieTemp` y `serieActividad` en
`src/routes/disks/[id]/+page.svelte` (líneas ~163-202). Un fallo en la consulta de eventos muestra
su propio `EmptyState kind="error"` sin afectar a la cabecera, las métricas, las gráficas ni los
contadores.

**Razón**: es la regla general de la constitución (principio X: "un fallo de fuente no es un fallo
de aplicación") ya aplicada dos veces en esta misma pantalla; no hay razón para tratar esta tercera
fuente de forma distinta.

**Alternativas consideradas**: cargar los eventos en el `load` de `+page.ts` junto al resto. Se
descarta porque el resto de fuentes de esta pantalla (series, comprobación de Defender) ya se piden
desde el propio componente tras el `load` inicial, precisamente para que cada una pueda fallar por
su cuenta sin bloquear el resto del `load` (constitución XIV: "un `load` que falle deja que el error
suba a `+error.svelte`" — eso sería una regresión aquí, porque un fallo del registro de eventos no
debería tumbar toda la pantalla de detalle).

## D5 · Estado vacío explícito, nunca ausente

**Decisión**: sin eventos, la `Card` se sigue mostrando con un `EmptyState kind="empty"` (o el texto
corto ya usado en el widget del panel general, a decidir por consistencia visual en implementación)
en vez de ocultar la sección entera.

**Razón**: constitución principio I y `ui-design.md` ("estados diseñados: vacío... una pantalla sin
ellos no está terminada").

## D6 · Documentar `?deviceId=` en `docs/ui-contract.md`

**Decisión**: añadir, junto al párrafo ya existente sobre `?focus=` (§3.5), uno equivalente para
`?deviceId=`.

**Razón**: el parámetro ya existe en el código (`src/routes/events/+page.ts`, desde
`open-questions.md` J.10) pero nunca se documentó como contrato ni se usó desde ninguna pantalla
real; esta funcionalidad es la primera en ejercitarlo de verdad, así que es el momento correcto de
documentarlo (`docs/ui-contract.md` es normativo frente a "cualquier descripción informal de
comandos", `AGENTS.md`).

## Resumen

Ninguna decisión de esta fase introduce una dependencia, un comando, un permiso o una tabla nuevos.
Todas reutilizan un patrón ya existente en el propio repositorio.
