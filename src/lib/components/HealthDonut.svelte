<script lang="ts">
  /** Anillo de estado global del equipo: un segmento por estado, en orden ok → warn → crit → unknown.
   *  Acompañar SIEMPRE de la leyenda numérica (el anillo solo no es accesible). */
  import { healthToken } from "$lib/design/health";
  import { t } from "$lib/i18n";
  import type { HealthState } from "$lib/design/types";

  let { counts = { ok: 0, warn: 0, crit: 0, unknown: 0 } as Record<HealthState, number>, size = 104 } =
    $props();

  const order: HealthState[] = ["ok", "warn", "crit", "unknown"];
  const total = $derived(order.reduce((s, k) => s + (counts[k] ?? 0), 0) || 1);
  const C = 2 * Math.PI * 50;

  const segments = $derived(
    order.reduce<{ state: HealthState; dash: number; offset: number }[]>((acc, state) => {
      const prev = acc.reduce((s, x) => s + x.dash, 0);
      const dash = ((counts[state] ?? 0) / total) * C;
      if (dash > 0) acc.push({ state, dash, offset: prev });
      return acc;
    }, [])
  );
</script>

<svg viewBox="0 0 120 120" width={size} height={size} role="img" aria-label={t("donut.label")}>
  <circle cx="60" cy="60" r="50" fill="none" stroke="var(--sdm-glass-3)" stroke-width="12" />
  {#each segments as seg (seg.state)}
    <circle
      cx="60"
      cy="60"
      r="50"
      fill="none"
      stroke={healthToken[seg.state].fg}
      stroke-width="12"
      stroke-linecap="round"
      stroke-dasharray="{seg.dash} {C}"
      stroke-dashoffset={-seg.offset}
      transform="rotate(-90 60 60)"
    />
  {/each}
</svg>
