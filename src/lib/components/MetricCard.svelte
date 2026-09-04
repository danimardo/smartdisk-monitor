<script lang="ts">
  /** Cifra grande + etiqueta + procedencia. `value` null ⇒ "No disponible", nunca 0.
   *
   *  Dos reglas que no son estéticas:
   *
   *  1. **Un valor que no es una cifra no se compone como cifra.** "No disponible" a 27 px mide
   *     174 px y no cabe en ninguna celda de métrica realista (una tarjeta de 460 px deja unos
   *     70 px por celda): se recortaría en todas las resoluciones, y justo en el caso más frecuente
   *     de la aplicación, que es un disco USB, RAID o virtual sin SMART. Además, componerlo como
   *     cifra miente sobre lo que es. Se compone como texto, en `text-base` y en gris tenue.
   *
   *  2. **Un bloque interno no es una tarjeta.** Va sobre `bg-glass-3` con `rounded-inner`, no
   *     sobre material con radio de tarjeta: `AGENTS.md` §2.bis prohíbe apilar materiales.
   */
  import { NOT_AVAILABLE } from "$lib/design/format";
  import { healthToken } from "$lib/design/health";
  import type { HealthState } from "$lib/design/types";

  let {
    label = "",
    value = null as string | null,
    state = null as HealthState | null,
    provenance = "",
    /** Marca de dato obsoleto, p. ej. "hace 12 min". Se muestra junto a la procedencia. */
    age = null as string | null
  } = $props();

  const missing = $derived(value === null || value === "");
  const color = $derived(
    missing ? "var(--sdm-text-faint)" : state ? healthToken[state].fg : "var(--sdm-text)"
  );
</script>

<div class="flex min-w-0 flex-col gap-1 rounded-inner bg-glass-3 p-3">
  <span class="truncate text-2xs text-fg-faint">{label}</span>

  {#if missing}
    <span class="text-base font-medium" style="color: {color}">{NOT_AVAILABLE()}</span>
  {:else}
    <span
      class="sdm-num truncate text-metric font-semibold"
      style="color: {color}; letter-spacing: var(--sdm-tracking-metric)"
      title={value}
    >
      {value}
    </span>
  {/if}

  {#if provenance || age}
    <span class="truncate text-2xs text-fg-faint">
      {provenance}{provenance && age ? " · " : ""}{age ?? ""}
    </span>
  {/if}
</div>
