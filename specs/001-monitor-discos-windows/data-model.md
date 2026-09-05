# Fase 1 — Modelo de datos

**Funcionalidad**: SmartDisk Monitor 1.0 · **Fecha**: 2026-09-04 · **Plan**: [plan.md](plan.md)

## El modelo canónico está en `docs/data-model.md`

> **Este fichero no lo reproduce, y es deliberado.** El modelo de datos es normativo y ya existe:
> doce tablas, veintitrés métricas normalizadas, política de retención y reglas de tiempo y unidades.
> Copiarlo aquí crearía dos versiones que divergirían en silencio, exactamente el fallo que ADR-029
> acaba de cerrar en este repositorio y por el mismo mecanismo: una copia «de referencia» que nadie
> actualiza. **Ante cualquier discrepancia, manda `docs/data-model.md`.**

Lo que sí aporta este fichero: el **delta** que introduce esta funcionalidad, la **trazabilidad**
entre requisitos y entidades, y las **invariantes** que la implementación no puede romper aunque el
esquema las admita.

| Necesitas | Ve a |
|---|---|
| Tablas, campos y tipos | `docs/data-model.md` §2 |
| Las 23 métricas normalizadas | `docs/data-model.md` §3 |
| Retención y compactación | `docs/data-model.md` §4 |
| Tiempo y unidades | `docs/data-model.md` §5 |
| Umbrales de las reglas | `docs/alert-rules.md` |
| Forma de los datos al cruzar a la interfaz | `docs/ui-contract.md` §2 |

## Trazabilidad: requisito → entidad

| Requisitos | Entidades canónicas |
|---|---|
| FR-001, FR-002, FR-007 | `devices`, `volumes`, `device_volume_links` |
| FR-003 a FR-006 | `metric_samples`, `smart_snapshots` (procedencia y frescura por muestra) |
| FR-008 a FR-013 | `alert_groups`, `alert_occurrences` |
| FR-014 a FR-016, FR-020 | `metric_samples` + política de retención §4 |
| FR-017 a FR-019 | `system_events`, `event_cursors` |
| FR-021 a FR-026 | `test_runs` |
| FR-027, FR-028 | ninguna entidad nueva: se leen las existentes |
| FR-029 a FR-031 | `settings` |
| FR-034, FR-035 | `schema_migrations` |

## Delta de esta funcionalidad

Tres cambios sobre el modelo canónico, todos nacidos de las aclaraciones del 2026-09-04. **Ninguno
se implementa antes de estar reflejado en `docs/data-model.md`**: la especificación de una
funcionalidad no puede adelantar al documento normativo.

### D1 · Preferencias nuevas en `settings`

Claves nuevas, con sus límites, derivadas de FR-020a y FR-029a:

| Clave | Tipo | Valor por defecto | Límites |
|---|---|---|---|
| `storage.free_space_warn_bytes` | entero | 1 073 741 824 (1 GB) | ≥ el umbral de parada |
| `storage.free_space_halt_bytes` | entero | 268 435 456 (256 MB) | > 0 |
| `logging.verbose` | booleano | `false` | — |

Los dos primeros son **valores de partida no medidos** (riesgo R4 de [research.md](research.md)):
antes de programarlos hay que registrarlos en `docs/open-questions.md` como `PROPUESTO`.

### D2 · Estado «escritura de historial detenida»

FR-020a/b exige que, al cruzar el umbral de parada, se deje de escribir historial **y se diga**. Eso
implica un estado observable que hoy no está modelado. No es una tabla nueva: es estado de ejecución
que la interfaz debe poder consultar y que debe viajar en los eventos de recopilación, junto a la
salud de las fuentes.

La invariante que introduce: **mientras la escritura está detenida, ninguna serie temporal puede
presentarse como continua**. El hueco resultante es un hueco real y se dibuja como tal, igual que
cualquier otro (FR-015).

### D3 · Ninguna purga por iniciativa propia

FR-020c cierra una puerta que el esquema deja abierta: los datos solo desaparecen por la retención
configurada o por borrado explícito del usuario. **Ningún camino de código puede eliminar filas para
ganar espacio.** Es una invariante de implementación, no una restricción del esquema, y por eso se
escribe aquí: no hay `CHECK` de SQLite que la garantice.

## Invariantes que el esquema no puede imponer

El modelo admite estados que el producto prohíbe. Estas siete reglas viven en `domain/` y son
material de prueba obligatorio, porque su incumplimiento es silencioso:

1. **Un dato ausente no se almacena como cero.** Los campos no disponibles se omiten
   (`docs/data-model.md` §3). Escribir un cero es indistinguible de una medición real de cero, y
   destruye la información para siempre.
2. **El firmware no forma parte de la huella de identidad.** Si lo formara, actualizarlo partiría el
   historial de un disco en dos entidades. El cambio de firmware se registra como apunte sobre el
   mismo dispositivo.
3. **El estado de un dispositivo es la peor severidad de sus alertas `active` o `acknowledged`.**
   Reconocer o silenciar no lo altera. Solo `resolved` y `archived` dejan de contar.
4. **Sin lecturas frescas, el estado es `unknown`, nunca `ok`.** No saber que algo está bien no es
   saber que está bien.
5. **Un dispositivo sin datos de salud es `unknown`, no crítico**, y no genera alerta.
6. **Las series no son equiespaciadas.** El eje es tiempo real; todo salto mayor que 1,5× la cadencia
   es un hueco, incluidos los extremos del intervalo pedido.
7. **La hora se persiste en tiempo universal y se presenta en local.** Las magnitudes se guardan en
   unidades base —bytes, grados Celsius— y se formatean solo al mostrarlas.

Las cuatro primeras están además elevadas a principio en la constitución (§I y §VI): incumplirlas no
es un defecto de implementación, es una violación normativa.

## Migraciones

`src-tauri/migrations/` está vacío: **todo el esquema es trabajo del eslabón 1**. Reglas que aplican
desde la primera:

- SQL numerado, y **nunca editado una vez publicado**. Un cambio posterior es una migración nueva.
- Copia consistente de la base antes de migrar; se conservan las tres más recientes.
- Verificadas desde cada versión publicada anterior, no solo desde cero (puerta de calidad por
  versión publicada).
