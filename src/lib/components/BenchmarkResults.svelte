<script lang="ts">
  /** Rejilla de resultados de la prueba de Rendimiento (spec 008 / ADR-053), con la disposición de
   *  CrystalDiskMark pero con los tokens de color y de tema de la aplicación: 4 perfiles × 2
   *  sentidos (Lectura / Escritura), cifra grande de MB/s (o IOPS con el selector), barra
   *  proporcional al máximo de la ejecución, y la latencia como dato secundario.
   *
   *  Es una `<table>` de verdad (`<th scope>`, navegable por teclado) pintada como una rejilla. Se
   *  llena **celda a celda** mientras corre la prueba (`running`): una celda sin dato aún es
   *  «pendiente», no «no ejecutada». Tooltips en los encabezados y en cada etiqueta de perfil. */
  import type { BenchmarkResult, BenchmarkProfile } from "$lib/api/types";
  import { t } from "$lib/i18n";
  import { formatBenchLatency, formatIops, formatMbPerSecond } from "$lib/design/format";
  import SegmentedControl from "./SegmentedControl.svelte";
  import Tooltip from "./Tooltip.svelte";

  let { result, running = false }: { result: BenchmarkResult; running?: boolean } = $props();

  const PERFILES: BenchmarkProfile[] = ["seq1m_q8", "seq1m_q1", "rnd4k_q32", "rnd4k_q1"];
  const SENTIDOS = ["read", "write"] as const;
  const CLAVE_UNIDAD = "sdm.benchmark.unit";

  let unidad = $state<"mb" | "iops">("mb");
  $effect(() => {
    try {
      const guardada = localStorage.getItem(CLAVE_UNIDAD);
      if (guardada === "mb" || guardada === "iops") unidad = guardada;
    } catch {
      /* localStorage puede no estar disponible; se queda en "mb" */
    }
  });
  function cambiarUnidad(id: string) {
    unidad = id === "iops" ? "iops" : "mb";
    try {
      localStorage.setItem(CLAVE_UNIDAD, unidad);
    } catch {
      /* sin persistencia, no pasa nada */
    }
  }

  type Fila = BenchmarkResult["rows"][number];
  const fila = (p: BenchmarkProfile, d: "read" | "write"): Fila | undefined =>
    result.rows.find((r) => r.profile === p && r.direction === d);
  const noEjecutada = (p: string, d: string): boolean =>
    result.notRun.some((n) => n.profile === p && n.direction === d);

  /** La medición en curso: la primera de la matriz sin dato y sin marcar como no ejecutada. Solo
   *  tiene sentido mientras `running`. */
  const enCurso = $derived.by(() => {
    if (!running) return null;
    for (const p of PERFILES) {
      for (const d of SENTIDOS) {
        if (!fila(p, d) && !noEjecutada(p, d)) return `${p}/${d}`;
      }
    }
    return null;
  });

  const valor = (f: Fila): number => (unidad === "mb" ? f.mbPerSecond : f.iops);
  const maxValor = $derived(Math.max(1, ...result.rows.map(valor)));
  const topeAlcanzado = $derived(result.rows.some((r) => r.dataCapHit));
  const etiquetaUnidad = $derived(
    unidad === "mb" ? t("tests.benchmark.col.throughput") : t("tests.benchmark.col.iops")
  );
</script>

<div class="flex flex-col gap-3">
  <div class="flex items-center justify-end">
    <SegmentedControl
      value={unidad}
      onchange={cambiarUnidad}
      options={[
        { id: "mb", label: t("tests.benchmark.col.throughput") },
        { id: "iops", label: t("tests.benchmark.col.iops") }
      ]}
    />
  </div>

  <div class="overflow-x-auto">
    <table class="w-full min-w-[26rem] border-separate" style="border-spacing: 0.375rem">
      <thead>
        <tr>
          <td class="w-[5.5rem]"></td>
          {#each SENTIDOS as sentido}
            <th scope="col" class="p-0 text-center">
              <Tooltip
                titulo={t(`tests.benchmark.direction.${sentido}`)}
                text={t(`tests.benchmark.help.${sentido}`)}
                disparador="mx-auto flex items-center justify-center gap-1 rounded-nav px-2 py-1 text-2xs font-semibold uppercase text-fg-dim"
              >
                {t(`tests.benchmark.direction.${sentido}`)} · {etiquetaUnidad}
              </Tooltip>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each PERFILES as perfil}
          <tr>
            <th scope="row" class="p-0 align-middle">
              <Tooltip
                titulo={t(`tests.benchmark.profile.${perfil}`)}
                text={t(`tests.benchmark.help.profile.${perfil}`)}
                disparador="w-[5.5rem] rounded-nav px-1 py-1 text-left text-2xs font-semibold leading-tight text-fg"
              >
                {t(`tests.benchmark.short.${perfil}`)}
              </Tooltip>
            </th>

            {#each SENTIDOS as sentido}
              {@const f = fila(perfil, sentido)}
              <td class="relative h-[4.25rem] overflow-hidden rounded-inner bg-glass-3 px-3 align-middle">
                {#if f}
                  <span
                    class="absolute inset-y-0 left-0 bg-accent-soft"
                    style="width: {Math.max(2, (valor(f) / maxValor) * 100)}%"
                  ></span>
                  <span class="sdm-num sdm-display relative block text-metric leading-none">
                    {unidad === "mb" ? formatMbPerSecond(f.mbPerSecond) : formatIops(f.iops)}
                    {#if f.dataCapHit}
                      <span class="align-super text-xs text-warn" title={t("tests.benchmark.dataCapHit")}
                        >*</span
                      >
                    {/if}
                  </span>
                  <span class="relative mt-0.5 block text-2xs text-fg-dim">
                    {unidad === "mb" ? `${formatIops(f.iops)} IOPS` : formatMbPerSecond(f.mbPerSecond)} · {formatBenchLatency(
                      f.avgLatencyMs
                    )}
                  </span>
                {:else if noEjecutada(perfil, sentido)}
                  <span class="text-sm font-medium text-fg-faint">{t("tests.benchmark.notRun")}</span>
                {:else if enCurso === `${perfil}/${sentido}`}
                  <span class="sdm-shimmer absolute inset-y-0 left-0 w-full bg-accent-soft opacity-60"></span>
                  <span class="relative text-sm text-fg-dim">{t("tests.benchmark.measuring")}</span>
                {:else}
                  <span class="text-sm text-fg-faint">—</span>
                {/if}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if topeAlcanzado}
    <p class="m-0 text-2xs text-fg-faint" style="text-wrap: pretty">
      {t("tests.benchmark.dataCapHitNote")}
    </p>
  {/if}
  {#if result.toolVersion}
    <p class="m-0 text-2xs text-fg-faint">
      {t("tests.benchmark.measuredWith", { tool: "DiskSpd", version: result.toolVersion })}
    </p>
  {/if}
</div>

<style>
  /* Indicador de «midiendo» de la celda en curso: un relleno que late de lado a lado. El ancho y
     el recorrido son geometría de la animación, no valores de tema — misma consideración que la
     barra indeterminada de `ProgressBar` y las keyframes de `AppShell`. */
  .sdm-shimmer {
    animation: sdm-bench-shimmer 1.4s var(--sdm-ease) infinite;
    transform-origin: left;
  }
  @keyframes sdm-bench-shimmer {
    0%,
    100% {
      transform: scaleX(0.15);
    }
    50% {
      transform: scaleX(1);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .sdm-shimmer {
      animation: none;
      transform: scaleX(0.4);
    }
  }
</style>
