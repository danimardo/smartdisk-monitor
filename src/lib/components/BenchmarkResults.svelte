<script lang="ts">
  /** Tabla de resultados de la prueba de Rendimiento (ADR-053): 8 filas (4 perfiles ×
   *  lectura/escritura) con MB/s, IOPS y latencia media, más las que no llegaron a correr.
   *
   *  Encabezados de columna reales (`<th scope="col">`) y navegable por teclado (constitución §VII).
   *  Cifras del backend; este componente solo las pinta, nunca las recalcula. */
  import type { BenchmarkResult } from "$lib/api/types";
  import { t } from "$lib/i18n";
  import { formatIops, formatLatency, formatMbPerSecond } from "$lib/design/format";

  let { result }: { result: BenchmarkResult } = $props();

  const topeAlcanzado = $derived(result.rows.some((r) => r.dataCapHit));
</script>

<div class="flex flex-col gap-2">
  <div class="overflow-x-auto">
    <table class="w-full border-collapse text-xs">
      <thead>
        <tr class="border-b border-hairline text-left text-2xs font-medium uppercase text-fg-faint">
          <th scope="col" class="py-2 pr-3 font-medium">{t("tests.benchmark.col.profile")}</th>
          <th scope="col" class="py-2 pr-3 font-medium">{t("tests.benchmark.col.direction")}</th>
          <th scope="col" class="py-2 pr-3 text-right font-medium">{t("tests.benchmark.col.throughput")}</th>
          <th scope="col" class="py-2 pr-3 text-right font-medium">{t("tests.benchmark.col.iops")}</th>
          <th scope="col" class="py-2 text-right font-medium">{t("tests.benchmark.col.latency")}</th>
        </tr>
      </thead>
      <tbody>
        {#each result.rows as row (row.profile + row.direction)}
          <tr class="border-b border-hairline">
            <th scope="row" class="py-2 pr-3 text-left font-semibold">
              {t(`tests.benchmark.profile.${row.profile}`)}
            </th>
            <td class="py-2 pr-3 text-fg-dim">
              {t(`tests.benchmark.direction.${row.direction}`)}
              {#if row.dataCapHit}
                <span class="text-fg-faint"> · {t("tests.benchmark.dataCapHit")}</span>
              {/if}
            </td>
            <td class="sdm-num py-2 pr-3 text-right font-semibold">{formatMbPerSecond(row.mbPerSecond)}</td>
            <td class="sdm-num py-2 pr-3 text-right">{formatIops(row.iops)}</td>
            <td class="sdm-num py-2 text-right">{formatLatency(row.avgLatencyMs)}</td>
          </tr>
        {/each}
        {#each result.notRun as nr (nr.profile + nr.direction)}
          <tr class="border-b border-hairline text-fg-faint">
            <th scope="row" class="py-2 pr-3 text-left font-medium">
              {t(`tests.benchmark.profile.${nr.profile}`)}
            </th>
            <td class="py-2 pr-3">{t(`tests.benchmark.direction.${nr.direction}`)}</td>
            <td class="py-2 pr-3 text-right italic" colspan="3">{t("tests.benchmark.notRun")}</td>
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
  <p class="m-0 text-2xs text-fg-faint">
    {t("tests.benchmark.measuredWith", { tool: "DiskSpd", version: result.toolVersion })}
  </p>
</div>
