<script lang="ts">
  /** Dato dominante del panel general (v3, ADR-034). Responde «¿tengo un problema?» sin leer: la
   *  curva de temperatura de 24 h a sangre de fondo, la cifra a 76 px, el umbral, cuatro hechos y
   *  hasta dos acciones.
   *
   *  **La pantalla elige el disco protagonista** (`selectHeroDisk()` en `$lib/design/health.ts`),
   *  no este componente. Todos los callbacks son opcionales.
   *
   *  El **velo de legibilidad** entre la curva y el texto no es decorativo: el trazo puede cruzar
   *  la zona de texto a cualquier altura, así que se garantiza el contraste con un degradado
   *  horizontal de material opaco (`HeroPanel.md`). */
  import Button from "./Button.svelte";
  import Card from "./Card.svelte";
  import Icon from "./Icon.svelte";
  import Sparkline from "./Sparkline.svelte";
  import StatusPill from "./StatusPill.svelte";
  import { healthToken } from "$lib/design/health";
  import type { IconName } from "$lib/design/icons";
  import { formatTemperature, NOT_AVAILABLE } from "$lib/design/format";
  import { t } from "$lib/i18n";
  import type { DiskSummary, HealthState } from "$lib/design/types";
  import type { Punto } from "$lib/design/series";

  let {
    disk = null as DiskSummary | null,
    series = [] as Punto[],
    threshold = null as number | null,
    /** Explicación humana (de la alerta, si hay una). */
    explanation = "",
    alertId = null as string | null,
    facts = [] as { label: string; value: string | null; state?: HealthState | null; icon: IconName }[],
    /** Marca de dato obsoleto: hora de la última lectura válida (`formatTime` ya aplicado) o "". */
    lastValidAt = "",
    loading = false,
    onopen = undefined as ((diskId: string) => void) | undefined,
    onviewalert = undefined as ((alertId: string) => void) | undefined
  } = $props();

  const state = $derived<HealthState>(disk?.state ?? "unknown");
  const tone = $derived(healthToken[state]);
  const atencion = $derived(state === "warn" || state === "crit");
  const sinSmart = $derived(
    disk?.state === "unknown" && (disk.unknownReason ?? "unsupported") === "unsupported"
  );

  /** La cifra se colorea solo cuando es una alarma; «todo en orden» va en el color de texto normal. */
  const cifraColor = $derived(atencion ? tone.fg : "var(--sdm-text)");
  const serieColor = $derived(atencion ? tone.fg : "var(--sdm-accent)");

  const cifra = $derived(
    sinSmart || disk?.temperatureC == null ? NOT_AVAILABLE() : formatTemperature(disk.temperatureC)
  );
  const identidad = $derived(disk ? `${disk.model} · ${disk.deviceType}` : "");
  const hayCurva = $derived(!sinSmart && series.some((p) => p.v !== null));
</script>

<Card padding="none">
  <div class="relative h-hero overflow-hidden rounded-card">
    {#if hayCurva && !loading}
      <div class="pointer-events-none absolute inset-0">
        <Sparkline points={series} {threshold} color={serieColor} fill strokeWidth={2.6} height={246} />
      </div>
    {/if}

    <!-- Velo de legibilidad: material opaco hasta el 42 % del ancho, transparente en el 68 %. -->
    <div
      class="pointer-events-none absolute inset-0"
      style="background: linear-gradient(90deg, var(--sdm-glass) 0%, var(--sdm-glass) 42%, transparent 68%)"
    ></div>

    <div class="relative flex h-full gap-4 p-5">
      <div class="flex min-w-0 flex-1 flex-col gap-2">
        <div class="flex items-center gap-2">
          <StatusPill
            {state}
            label={atencion ? t(`health.${state}`) : t("dashboard.hero.allGood")}
            icon="auto"
          />
          {#if identidad}<span class="sdm-selectable truncate text-xs text-fg-dim">{identidad}</span>{/if}
        </div>

        {#if loading}
          <div class="h-16 w-40 rounded-inner bg-glass-3"></div>
        {:else}
          <div class="flex items-baseline gap-2">
            <span class="sdm-display text-hero" style="color: {cifraColor}">{cifra}</span>
            {#if threshold != null && !sinSmart}
              <span class="sdm-num text-xs text-fg-faint"
                >{t("chart.tempWarnLabel", { value: formatTemperature(threshold) })}</span
              >
            {/if}
          </div>
        {/if}

        {#if lastValidAt}
          <span class="text-2xs text-warn">{t("dashboard.hero.lastValid", { time: lastValidAt })}</span>
        {:else if !hayCurva && !sinSmart && !loading}
          <span class="text-2xs text-fg-faint">{t("dashboard.hero.noSeries")}</span>
        {/if}

        <p class="m-0 max-w-[520px] text-sm text-fg-dim" style="text-wrap: pretty">
          {#if sinSmart}
            {t("disk.noSmartExplain")}
          {:else if atencion && explanation}
            {explanation}
          {:else}
            {t("dashboard.hero.allGoodBody")}
          {/if}
        </p>

        <div class="mt-auto flex items-center gap-2">
          {#if atencion && alertId && onviewalert}
            <Button variant="primary" onclick={() => onviewalert?.(alertId)}
              >{t("dashboard.hero.viewAlert")}</Button
            >
          {/if}
          {#if disk && onopen}
            <Button variant={atencion && alertId ? "secondary" : "primary"} onclick={() => onopen?.(disk.id)}
              >{t("dashboard.hero.openDisk")}</Button
            >
          {/if}
        </div>
      </div>

      <div class="flex w-[290px] flex-none flex-col justify-center gap-2 max-[1100px]:hidden">
        {#each facts as fact}
          <div class="flex items-center gap-3 rounded-inner bg-glass-3 p-3">
            <span
              class="grid size-8 shrink-0 place-items-center rounded-nav"
              style="background: {(fact.state
                ? healthToken[fact.state]
                : { soft: 'var(--sdm-accent-soft)', fg: 'var(--sdm-accent-fg)' }
              ).soft}; color: {(fact.state
                ? healthToken[fact.state]
                : { soft: 'var(--sdm-accent-soft)', fg: 'var(--sdm-accent-fg)' }
              ).fg}"
            >
              <Icon name={fact.icon} size={16} />
            </span>
            <span class="flex min-w-0 flex-col">
              <span class="truncate text-2xs text-fg-faint">{fact.label}</span>
              <span class="sdm-num truncate text-sm font-semibold">{fact.value ?? NOT_AVAILABLE()}</span>
            </span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</Card>
