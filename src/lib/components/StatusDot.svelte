<script lang="ts">
  /** Punto de estado de 9 px. El color **nunca** es el único portador del estado: o lo acompaña una
   *  etiqueta visible, o el propio punto lleva el nombre del estado como texto alternativo.
   *
   *  La versión anterior emitía siempre `role="img" aria-label={label}`, y como `label` es `""` por
   *  omisión —así lo llama `Sidebar`— quedaba un `role="img"` **sin nombre accesible**: un lector de
   *  pantalla lo anuncia como «imagen» y no dice nada más. Lo detectó `axe` en `e2e/ui/a11y.spec.ts`
   *  (regla `role-img-alt`, impacto serio) en el panel general. */
  import { healthToken } from "$lib/design/health";
  import { t } from "$lib/i18n";
  import type { HealthState } from "$lib/design/types";

  let { state = "ok" as HealthState, label = "", size = 9 } = $props();

  /** Con etiqueta visible, el punto es decorativo y anunciarlo duplicaría la lectura. Sin ella, es
   *  el único portador y necesita nombre propio. */
  const decorativo = $derived(label !== "");
</script>

<span class="inline-flex items-center gap-2">
  <span
    class="rounded-pill shrink-0"
    style="width: {size}px; height: {size}px; background: {healthToken[state].fg};"
    role={decorativo ? undefined : "img"}
    aria-hidden={decorativo ? "true" : undefined}
    aria-label={decorativo ? undefined : t(`health.${state}`)}
  ></span>
  {#if label}<span class="text-xs text-fg-dim">{label}</span>{/if}
</span>
