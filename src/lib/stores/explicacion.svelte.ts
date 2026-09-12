/** Máquina de estados del modal de explicación con IA (spec 005, US2/US4). Una sola instancia:
 *  solo puede haber un modal abierto a la vez. La usan `/alerts` y `/disks/[id]` sin duplicar la
 *  orquestación.
 */

import type { OrigenExplicacion } from "$lib/api";
import { explicarDetalleTecnico, setSetting, toAppError } from "$lib/api";
import type { AppError } from "$lib/design/types";
import { ia } from "./ia.svelte";

type Fase = "progreso" | "resultado" | "error" | "vistaPrevia" | "revision";
type Revision = OrigenExplicacion["revision"];

/** El origen sin los campos que gestiona la máquina (`revision`, `previewConfirmada`,
 *  `modeloSolicitado`: este último lo gestiona `reprocesar()`, spec 010 — quien llama a `lanzar()`
 *  nunca lo toca). */
type Origen = Omit<OrigenExplicacion, "revision" | "previewConfirmada" | "modeloSolicitado">;

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
  /** Spec 010, US2 (FR-008): `true` si la respuesta en primer plano se obtuvo reprocesando —
   *  nunca si es la respuesta inicial del modo automático. Decide si se ofrece «fijar por
   *  defecto». */
  esReprocesada = $state(false);
  /** Spec 010, US2: error de la última llamada a `fijarPorDefecto`, si la hubo. Aparte de `error`
   *  (que es del flujo de pedir la explicación) para no tocar la fase ni perder la respuesta que
   *  ya se estaba mostrando. */
  errorFijarPorDefecto = $state<AppError | null>(null);
  /** Spec 010, US3 (FR-007/FR-012): respuestas que dejaron de estar en primer plano al
   *  reprocesar, en orden de llegada (la más reciente archivada, última). Sin límite; se vacía en
   *  `lanzar()`. */
  historial = $state<{ modelo: string; markdown?: string; error?: AppError }[]>([]);

  #origen: Origen | null = null;
  #revision: Revision = "ninguna";
  /** Spec 010, US1: modelo elegido al reprocesar; `undefined` = usa el ajuste general. */
  #modeloOverride: string | undefined = undefined;

  get #clave(): string {
    const o = this.#origen;
    return `${o?.tipo}:${o?.alertGroupId ?? o?.deviceId ?? o?.eventId}`;
  }

  /** Punto de entrada: abre el modal y pide la explicación desde cero. */
  async lanzar(origen: Origen): Promise<void> {
    this.#origen = origen;
    this.#revision = "ninguna";
    this.#modeloOverride = undefined;
    this.esReprocesada = false;
    this.historial = [];
    await this.#pedir(false);
  }

  confirmarPreview = () => this.#pedir(true);
  reintentar = () => this.#pedir(true);
  enviarIgual = () => this.#conRevision("enviar_igual");
  quitarFragmentos = () => this.#conRevision("quitar_fragmentos");
  cerrar = () => {
    this.open = false;
  };

  /** Spec 010, US1 (FR-001 a FR-005): reprocesa la misma petición ya construida con otro modelo,
   *  desde una respuesta (`resultado`) o desde un error. Reutiliza `#revision`, ya resuelta si esta
   *  sesión necesitó vista previa o revisión de fragmentos (FR-004). */
  reprocesar(modelo: string): Promise<void> {
    // FR-007: archiva lo que hay en primer plano antes de sobrescribirlo. Un error no tiene
    // `modeloUsado` propio (la llamada nunca respondió); se identifica por el modelo que se le
    // había pedido, o cadena vacía si fue la primera llamada del modo automático.
    if (this.fase === "resultado") {
      this.historial.push({ modelo: this.modeloUsado, markdown: this.markdown });
    } else if (this.fase === "error" && this.error) {
      this.historial.push({ modelo: this.#modeloOverride ?? "", error: this.error });
    }
    this.#modeloOverride = modelo;
    this.esReprocesada = true;
    return this.#pedir(true);
  }

  /** Spec 010, US2 (FR-008/FR-009): fija `modelo` como modelo por defecto de toda la aplicación.
   *  Solo tiene sentido llamarlo sobre una respuesta con `esReprocesada`; eso lo decide la interfaz
   *  (no oculta ni valida aquí). Un fallo al guardar no toca la respuesta que se estaba mostrando
   *  (edge case de la spec): se expone en `errorFijarPorDefecto`, aparte de `error`. */
  async fijarPorDefecto(modelo: string): Promise<void> {
    this.errorFijarPorDefecto = null;
    try {
      await setSetting("settings.ai.model", modelo);
      await ia.refrescar();
    } catch (cause) {
      this.errorFijarPorDefecto = toAppError(cause);
    }
  }

  #conRevision(r: Revision): Promise<void> {
    this.#revision = r;
    return this.#pedir(false);
  }

  async #pedir(previewConfirmada: boolean): Promise<void> {
    if (!this.#origen || ia.estaEnCurso(this.#clave)) return;
    // FR-013: se captura aquí porque `#origen` puede cambiar mientras esta llamada está en curso
    // (otra explicación lanzada mientras tanto, tras cerrar esta). Una respuesta tardía se descarta
    // en silencio si ya no corresponde a la sesión activa — nunca pisa su estado ni la reabre.
    const clave = this.#clave;
    ia.marcarEnCurso(clave);
    this.open = true;
    this.fase = "progreso";
    this.error = null;
    try {
      const r = await explicarDetalleTecnico({
        ...this.#origen,
        revision: this.#revision,
        previewConfirmada,
        modeloSolicitado: this.#modeloOverride ?? null
      });
      if (this.#clave !== clave) return;
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
      if (this.#clave !== clave) return;
      this.error = toAppError(cause);
      this.fase = "error";
    } finally {
      ia.liberar(clave);
    }
  }
}

export const explicacion = new Explicacion();
