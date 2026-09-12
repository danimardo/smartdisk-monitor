<script lang="ts">
  /** Barra de título propia (spec 011). Presentacional: no importa `@tauri-apps/api` — quien monta
   *  este componente (`src/routes/+layout.svelte`) pasa los tres gestos como *callbacks*, conectados
   *  a `src/lib/window.ts`. Mismo patrón que `ExplicacionModal` con `onreprocesar`/`oncancel`
   *  (constitución §IV).
   *
   *  Fondo: `bg-solid` (`--sdm-solid`), el mismo plano opaco que usan los materiales con cristal
   *  como respaldo — indistinguible a ojo del resto de la interfaz, que es justo lo que pide FR-008
   *  (corrección post-validación tras ver la barra en vivo: el degradado `bg`/`bg-2` se leía más
   *  oscuro que el resto del lienzo a esa altura tan fina).
   *  Iconos: color de acento en reposo (FR-009); el botón cerrar pasa a `crit` solo en `hover`,
   *  nunca en reposo (sigue siendo un acento propio de la app, no el rojo de Windows). */
  import Icon from "./Icon.svelte";
  import { t } from "$lib/i18n";
  import iconoApp from "$lib/assets/icono-app.png";

  let {
    maximized = false,
    onminimize = undefined as (() => void) | undefined,
    ontogglemaximize = undefined as (() => void) | undefined,
    onclose = undefined as (() => void) | undefined
  } = $props();
</script>

<div class="flex h-control-lg shrink-0 items-stretch bg-solid">
  <!-- Región de arrastre: toda la franja salvo los tres controles, incluido el icono y el nombre
       (así se puede arrastrar la ventana también desde ahí, como en una barra nativa). El doble
       clic maximiza/restaura (US3, FR-012) — atado aquí y no a la barra entera para que un doble
       clic sobre un botón no burbujee a esta acción (los botones están fuera de este `<div>`). -->
  <div
    class="flex min-w-0 flex-1 items-center gap-2 pl-3"
    role="presentation"
    data-tauri-drag-region
    ondblclick={() => ontogglemaximize?.()}
  >
    <img src={iconoApp} alt="" width="16" height="16" class="shrink-0" />
    <span class="truncate text-xs font-medium text-fg-dim">{t("app.name")}</span>
  </div>
  <div class="flex items-stretch">
    <button
      type="button"
      class="flex w-12 items-center justify-center text-accent transition-colors duration-base ease-sdm hover:bg-accent-soft hover:text-accent-hi"
      aria-label={t("titlebar.minimize")}
      onclick={() => onminimize?.()}
    >
      <Icon name="minimize" size={14} />
    </button>
    <button
      type="button"
      class="flex w-12 items-center justify-center text-accent transition-colors duration-base ease-sdm hover:bg-accent-soft hover:text-accent-hi"
      aria-label={maximized ? t("titlebar.restore") : t("titlebar.maximize")}
      onclick={() => ontogglemaximize?.()}
    >
      <Icon name={maximized ? "restore" : "maximize"} size={14} />
    </button>
    <button
      type="button"
      class="flex w-12 items-center justify-center text-accent transition-colors duration-base ease-sdm hover:bg-crit-soft hover:text-crit"
      aria-label={t("common.close")}
      onclick={() => onclose?.()}
    >
      <Icon name="close" size={14} />
    </button>
  </div>
</div>
