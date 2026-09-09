/** Máquina de estados del modal de explicación con IA (spec 005, US2/US4). Una sola instancia:
 *  solo puede haber un modal abierto a la vez. La usan `/alerts` y `/disks/[id]` sin duplicar la
 *  orquestación.
 */

import type { OrigenExplicacion } from "$lib/api";
import { explicarDetalleTecnico, toAppError } from "$lib/api";
import type { AppError } from "$lib/design/types";
import { ia } from "./ia.svelte";

type Fase = "progreso" | "resultado" | "error" | "vistaPrevia" | "revision";
type Revision = OrigenExplicacion["revision"];

/** El origen sin los campos que gestiona la máquina (`revision`, `previewConfirmada`). */
type Origen = Omit<OrigenExplicacion, "revision" | "previewConfirmada">;

class Explicacion {
  open = $state(false);
  fase = $state<Fase>("progreso");
  markdown = $state("");
  modeloUsado = $state("");
  detalleRecortado = $state(false);
  sinVolcado = $state(false);
  sinSuceso = $state(false);
  error = $state<AppError | null>(null);
  textoRevision = $state("");
  fragmentos = $state<{ texto: string; motivoKey: string }[]>([]);

  #origen: Origen | null = null;
  #revision: Revision = "ninguna";

  get #clave(): string {
    return `${this.#origen?.tipo}:${this.#origen?.alertGroupId ?? this.#origen?.deviceId}`;
  }

  /** Punto de entrada: abre el modal y pide la explicación desde cero. */
  async lanzar(origen: Origen): Promise<void> {
    this.#origen = origen;
    this.#revision = "ninguna";
    await this.#pedir(false);
  }

  confirmarPreview = () => this.#pedir(true);
  reintentar = () => this.#pedir(true);
  enviarIgual = () => this.#conRevision("enviar_igual");
  quitarFragmentos = () => this.#conRevision("quitar_fragmentos");
  cerrar = () => {
    this.open = false;
  };

  #conRevision(r: Revision): Promise<void> {
    this.#revision = r;
    return this.#pedir(false);
  }

  async #pedir(previewConfirmada: boolean): Promise<void> {
    if (!this.#origen || ia.estaEnCurso(this.#clave)) return;
    ia.marcarEnCurso(this.#clave);
    this.open = true;
    this.fase = "progreso";
    this.error = null;
    try {
      const r = await explicarDetalleTecnico({
        ...this.#origen,
        revision: this.#revision,
        previewConfirmada
      });
      if (r.estado === "ok") {
        this.markdown = r.markdown;
        this.modeloUsado = r.modeloUsado;
        this.detalleRecortado = r.detalleRecortado;
        this.sinVolcado = r.sinVolcado;
        this.sinSuceso = r.sinSuceso;
        this.fase = "resultado";
      } else {
        this.textoRevision = r.textoCompleto;
        this.fragmentos = r.fragmentos;
        this.fase = r.fragmentos.length > 0 ? "revision" : "vistaPrevia";
      }
    } catch (cause) {
      this.error = toAppError(cause);
      this.fase = "error";
    } finally {
      ia.liberar(this.#clave);
    }
  }
}

export const explicacion = new Explicacion();
