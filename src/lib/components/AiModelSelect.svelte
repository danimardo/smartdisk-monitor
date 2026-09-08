<script lang="ts">
  /** Selector del modelo de IA (spec 005, US3). Se usa en Ajustes y en el asistente. Carga la
   *  lista del proveedor al montarse; si falla, deja seguir con «modelo gratuito automático»
   *  (US3 escenario 3). Al elegir un modelo de pago, pide confirmación (FR-015a). */
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import Select from "./Select.svelte";
  import { listarModelosIa, toAppError } from "$lib/api";
  import type { AppError, ModeloIaWire } from "$lib/api";
  import { t } from "$lib/i18n";

  let {
    modelo = "openrouter/free",
    disabled = false,
    onchange = undefined as ((id: string) => void) | undefined
  } = $props();

  let modelos = $state<ModeloIaWire[]>([]);
  let cargando = $state(true);
  let error = $state<AppError | null>(null);
  let pendienteDePago = $state<ModeloIaWire | null>(null);

  $effect(() => {
    let cancelado = false;
    cargando = true;
    error = null;
    void (async () => {
      try {
        const lista = await listarModelosIa();
        if (!cancelado) modelos = lista;
      } catch (cause) {
        if (!cancelado) error = toAppError(cause);
      } finally {
        if (!cancelado) cargando = false;
      }
    })();
    return () => {
      cancelado = true;
    };
  });

  /** Siempre incluye «automático», aunque la lista no cargue. */
  const opciones = $derived([
    { id: "openrouter/free", label: t("settings.ai.model.auto") },
    ...modelos
      .filter((m) => m.id !== "openrouter/free")
      .map((m) => ({
        id: m.id,
        label: m.esDePago ? t("settings.ai.model.paidSuffix", { name: m.nombre }) : m.nombre
      }))
  ]);

  function elegir(id: string) {
    const dato = modelos.find((m) => m.id === id);
    if (dato?.esDePago) {
      pendienteDePago = dato;
      return;
    }
    onchange?.(id);
  }
</script>

<div class="flex flex-col gap-2">
  <Select
    value={modelo}
    label={t("settings.ai.model.change")}
    options={opciones}
    disabled={disabled || cargando}
    onchange={elegir}
  />
  {#if error}
    <p class="m-0 text-xs text-warn" style="text-wrap: pretty">
      {t("settings.ai.model.listUnavailable")}
    </p>
  {/if}
</div>

<ConfirmDialog
  open={pendienteDePago !== null}
  title={t("settings.ai.model.paidTitle")}
  body={t("settings.ai.model.paidBody", { name: pendienteDePago?.nombre ?? "" })}
  impact={t("settings.ai.model.paidImpact")}
  confirmLabel={t("settings.ai.model.paidConfirm")}
  onconfirm={() => {
    const id = pendienteDePago?.id;
    pendienteDePago = null;
    if (id) onchange?.(id);
  }}
  oncancel={() => (pendienteDePago = null)}
/>
