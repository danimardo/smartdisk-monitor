# Fase 0 — Investigación y riesgos a medir

**Funcionalidad**: SmartDisk Monitor 1.0 · **Fecha**: 2026-09-04 · **Plan**: [plan.md](plan.md)

## Cómo se ha ejecutado esta fase

La fase 0 de SpecKit sirve normalmente para resolver decisiones tecnológicas pendientes. **Aquí no
hay ninguna**: la constitución fija la pila con versiones exactas y cambiarla exige enmienda, no
investigación. El contexto técnico del plan no tiene ni una casilla marcada como pendiente de
aclarar.

Lo que sí queda por resolver es de otra clase, y el proyecto es explícito al respecto: *«los riesgos
de la fase 0 se miden, no se estiman. Una decisión de arquitectura basada en una suposición no
comprobada no es una decisión: es una apuesta»*. Cuatro suposiciones de la especificación original
resultaron falsas al medirlas (`docs/open-questions.md`, §L a §Q). Esta fase, por tanto, no elige
tecnología: **enumera lo que hay que medir, cuándo, con qué, y qué se hace si sale mal**.

## Decisiones ya tomadas que este plan hereda

No se reabren. Se listan porque son las que sostienen el orden de construcción.

| Decisión | Dónde | Por qué importa aquí |
|---|---|---|
| Tauri 2 + Rust para el backend privilegiado | ADR-001, constitución §III | La interfaz nunca ejecuta órdenes; toda lectura privilegiada vive en Rust |
| SvelteKit con `adapter-static` y SSR desactivado | ADR-014 | El estado inicial llega por `load`, no por `onMount` |
| El backend empuja por eventos; la interfaz no sondea | ADR-015 | Ningún `setInterval` para pedir datos; el muestreo lo marca el backend |
| SQLite con `rusqlite` y `bundled` | ADR-006 | Sin depender de la DLL del sistema; WAL y migraciones numeradas |
| Zod en toda frontera | ADR-022, constitución §XI | Un cambio de forma en el backend se detecta al entrar, no tres pantallas después |
| `smartctl` 7.5 redistribuido con su fuente | ADR-021 | Fuente principal de datos de salud; hashes verificados en cada compilación |
| Ejecución elevada obligatoria | ADR-004 | Sin ella no hay lectura de salud; y sin elevación **no se arranca a medias** (US-001) |
| Registro con `tracing` y envoltorio propio | ADR-024, constitución §XV | El modo detallado de FR-029a se suma a esta API, no crea otra |
| Instancia única con el plugin oficial | ADR-025 | FR-033 ya está resuelto por decisión previa |

## Riesgos a medir

### R1 · Notificaciones del sistema desde un proceso elevado

**Qué hay que saber**: si Windows entrega notificaciones emergentes cuando la aplicación corre bajo
`requireAdministrator` con su identificador de aplicación registrado.

**Por qué no se puede asumir**: el comportamiento depende del registro del identificador y de la
elevación, y la documentación no es concluyente para el caso combinado.

**Cuándo se mide**: eslabón 5 (motor de alertas), al empaquetar. No antes: en desarrollo la
aplicación no tiene identificador registrado, así que una prueba temprana mediría otra cosa.

**Qué se hace si falla**: ventana propia con el componente `Toast` del catálogo, anclada sobre el
área de notificación. El componente ya existe; el coste es de posicionamiento, no de diseño.

**Trazabilidad**: `docs/open-questions.md` I.2 · US-060 · afecta a la historia 2.

---

### R2 · Alcance real de la lectura de salud tras controladoras RAID y puentes USB

**Qué hay que saber**: qué cascada de modos de acceso (`sat`, `nvme`, `sntjmicron`, `csmi`) merece
la pena intentar antes de declarar un dispositivo «no compatible», y cuánto tarda esa cascada.

**Por qué no se puede asumir**: depende del modelo concreto del puente o de la controladora. No hay
forma de saberlo sin el hardware delante.

**Cuándo se mide**: eslabón 3 (colector de salud), con dispositivos reales.

**Qué se hace si falla**: se documenta la limitación por modelo y se declara «no compatible», que es
un resultado **correcto** y no un fallo: el producto ya está diseñado para que eso sea gris y no
rojo. El riesgo real no es no poder leer, es tardar tanto intentándolo que se degrade el ciclo de
recopilación. Por eso la medición debe incluir **tiempo**, no solo éxito.

**Trazabilidad**: `docs/open-questions.md` I.5 · US-010 · afecta a la historia 1.

---

### R3 · Comportamiento de la interfaz con 20 discos y 5.000 eventos

**Qué hay que saber**: si el panel y la lista de eventos se mantienen dentro de los umbrales de
SC-007 —la interfaz nunca deja de responder más de 50 ms seguidos— en el escenario de servidor.

**Por qué ya no es ambiguo**: hasta la aclaración del 2026-09-04 el criterio era «sin bloqueo
perceptible», que no se puede medir. Ahora hay un número, y con él una prueba escribible.

**Cuándo se mide**: eslabón 6, en cuanto exista `VirtualList` —uno de los cuatro componentes ya
autorizados—. Medir antes, con la lista sin virtualizar, solo confirmaría lo que ya se sabe.

**Cómo se mide**: con el plano de interfaz que el proyecto ya tiene montado (Playwright con IPC
propio), alimentando el doble de IPC con 20 discos y 5.000 eventos y observando las tareas largas
durante el desplazamiento.

**Qué se hace si falla**: recortar la densidad del panel o paginar la lista.

**Trazabilidad**: `docs/open-questions.md` I.7 · US-021 · afecta a las historias 1 y 4.

---

### R4 · Umbrales de espacio libre para detener la escritura de historial

**Qué hay que saber**: si 1 GB para avisar y 256 MB para detener son valores adecuados en los
equipos objetivo, incluidos servidores con volúmenes de sistema ajustados.

**Por qué aparece ahora**: lo introdujo la aclaración del 2026-09-04. La especificación los adopta
como valores de partida y los marca expresamente como **no medidos**.

**Cuándo se mide**: eslabón 1, **antes** de programar la retención. Es requisito del flujo de
desarrollo: una decisión que no está escrita se registra antes de programarla, nunca se resuelve en
silencio dentro del código.

**Acción inmediata, previa a escribir código**: registrar la entrada en `docs/open-questions.md`
como `PROPUESTO`, con estos valores y su razón. Mientras no esté registrada, programarla sería la
infracción de proceso más grave que contempla el proyecto.

**Qué se hace si falla**: ajustar el valor con el dato medido. La forma del requisito (avisar,
detener, decirlo, no purgar nunca por iniciativa propia) no depende del número.

**Trazabilidad**: FR-020a/b/c · SC-015a · afecta a la historia 7.

## Trabajo de fase 0 que no es medición

Dos elementos que hay que dejar resueltos antes de que el código los dé por supuestos:

**Activación de la puerta de DTO generados.** La constitución la declara pendiente «hasta que el
primer comando devuelva un DTO real». Ese momento llega en el eslabón 2. Hay que preparar la
generación y retirar el mantenimiento manual de `src/lib/api/types.ts` **en el mismo cambio** que
introduce el primer DTO: si se posterga, quedan dos fuentes de verdad para los tipos del contrato, y
divergirán. Es el mismo patrón de fallo que cerró ADR-029, aplicado a otro material.

**Registro de la decisión sobre el modo detallado.** FR-029a/b/c introducen comportamiento nuevo
—interruptor de modo detallado, acción que abre la carpeta, inclusión en el paquete de diagnóstico—
que no estaba en `docs/user-stories.md`. Antes de implementarlo hay que llevarlo a US-071 o a una
historia propia, para que la fuente normativa no vaya por detrás de la especificación de la
funcionalidad.

## Conclusión de la fase

**Ninguna casilla del contexto técnico quedó pendiente de aclarar.** Los cuatro riesgos tienen
momento de medición asignado y alternativa decidida, así que ninguno bloquea el comienzo de la
implementación. Dos de ellos —R4 y el registro del modo detallado— exigen una anotación en la
documentación normativa **antes** de escribir el código que los usa; están recogidos como tales en
el orden de construcción del plan.
