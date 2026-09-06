<script lang="ts">
  /** Píldora de estado sobre material: fondo translúcido del token + texto del token.
   *  Es el único indicador textual de salud; el color nunca viaja solo.
   *
   *  v3: ranura de icono opcional (`icon`), para barrer el estado sin leer. `icon="auto"` lo resuelve
   *  por `healthIcon[state]`. El icono va `aria-hidden`: el texto de la píldora ya dice el estado y
   *  anunciarlo dos veces molesta. `icon` y `withDot` son excluyentes por construcción: si se pasan
   *  los dos, `icon` gana y el punto no se pinta. */
  import Icon from "./Icon.svelte";
  import { healthToken } from "$lib/design/health";
  import { healthIcon, type IconName } from "$lib/design/icons";
  import type { HealthState } from "$lib/design/types";

  let {
    state = "ok" as HealthState,
    label = "",
    icon = undefined as IconName | "auto" | undefined,
    withDot = false
  } = $props();

  const tone = $derived(healthToken[state]);
  const resolvedIcon = $derived<IconName | undefined>(icon === "auto" ? healthIcon[state] : icon);
</script>

<span
  class="inline-flex items-center gap-1.5 rounded-pill px-3 py-1 text-2xs font-semibold whitespace-nowrap"
  style="background: {tone.soft}; color: {tone.fg};"
>
  {#if resolvedIcon}
    <Icon name={resolvedIcon} size={13} />
  {:else if withDot}
    <span class="size-1.5 rounded-pill" style="background: {tone.fg}"></span>
  {/if}
  {label}
</span>
