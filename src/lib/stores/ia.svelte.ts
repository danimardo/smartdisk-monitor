/** Estado de la ayuda con IA (spec 005-explicacion-ia) en runes de Svelte 5. Sin stores clásicos
 *  (AGENTS.md §1), sin `localStorage` (constitución §V).
 *
 *  `estado` se rellena desde `estado_ia` (en el `load` de Ajustes o al arrancar la app) y se
 *  refresca tras guardar/borrar la clave. `enCurso` evita lanzar dos consultas de explicación a
 *  la vez para el mismo detalle (FR-020).
 */

import type { EstadoIaWire } from "$lib/api";
import { estadoIa } from "$lib/api";

class IaState {
  estado = $state<EstadoIaWire | null>(null);

  /** Claves `${tipo}:${id}` de las explicaciones en curso. */
  #enCurso = $state(new Set<string>());

  /** `true` si la función está activada (hay clave). */
  get activa(): boolean {
    return this.estado?.activa ?? false;
  }

  /** `true` si el modo «enviar sin revisar» está activo (spec 006, FR-007). */
  get sendWithoutReview(): boolean {
    return this.estado?.sendWithoutReview ?? false;
  }

  /** `true` si este binario trae compilada la clave de demostración compartida (ADR-054). */
  get claveCompartidaDisponible(): boolean {
    return this.estado?.claveCompartidaDisponible ?? false;
  }

  /** `true` si la credencial en uso es la clave de demostración compartida (ADR-054). */
  get usandoClaveCompartida(): boolean {
    return this.estado?.usandoClaveCompartida ?? false;
  }

  estaEnCurso(clave: string): boolean {
    return this.#enCurso.has(clave);
  }

  marcarEnCurso(clave: string): void {
    this.#enCurso = new Set(this.#enCurso).add(clave);
  }

  liberar(clave: string): void {
    const copia = new Set(this.#enCurso);
    copia.delete(clave);
    this.#enCurso = copia;
  }

  set(estado: EstadoIaWire): void {
    this.estado = estado;
  }

  async refrescar(): Promise<void> {
    this.estado = await estadoIa();
  }
}

export const ia = new IaState();
