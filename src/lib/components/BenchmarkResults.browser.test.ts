import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import es from "$lib/i18n/es.json";
import type { BenchmarkResult } from "$lib/api/types";

/** La tabla de resultados de la prueba de Rendimiento (spec 008 / ADR-053): estado completado
 *  (8 filas con cifras) y estado con `notRun` (parada anticipada → filas parciales + «no
 *  ejecutado», nunca 0). */

const { default: BenchmarkResults } = await import("./BenchmarkResults.svelte");

const fila = (
  profile: BenchmarkResult["rows"][number]["profile"],
  direction: "read" | "write",
  mb: number
): BenchmarkResult["rows"][number] => ({
  profile,
  direction,
  mbPerSecond: mb,
  iops: mb * 4,
  avgLatencyMs: 1.2,
  actualDurationS: 5,
  bytesMoved: mb * 5_000_000,
  dataCapHit: false
});

const completo: BenchmarkResult = {
  tool: "diskspd",
  toolVersion: "2.3.0",
  fileSizeBytes: 1_073_741_824,
  rows: [
    fila("seq1m_q8", "read", 3200),
    fila("seq1m_q8", "write", 2800),
    fila("seq1m_q1", "read", 2600),
    fila("seq1m_q1", "write", 2400),
    fila("rnd4k_q32", "read", 720),
    fila("rnd4k_q32", "write", 640),
    fila("rnd4k_q1", "read", 68),
    fila("rnd4k_q1", "write", 190)
  ],
  notRun: []
};

describe("BenchmarkResults", () => {
  it("completado: 8 filas con cifras, encabezados reales y el pie con la versión", async () => {
    const { container } = await render(BenchmarkResults, { props: { result: completo } });

    await expect
      .element(page.getByRole("columnheader", { name: es["tests.benchmark.col.throughput"] }))
      .toBeInTheDocument();
    await expect
      .element(page.getByRole("rowheader", { name: es["tests.benchmark.profile.seq1m_q8"] }).first())
      .toBeInTheDocument();
    // 8 filas de datos en el cuerpo de la tabla.
    expect(container.querySelectorAll("tbody tr").length).toBe(8);
    await expect.element(page.getByText("MB/s").first()).toBeInTheDocument();
    await expect.element(page.getByText("Medido con DiskSpd 2.3.0")).toBeInTheDocument();
  });

  it("parada anticipada: filas parciales + «No ejecutado», nunca un 0", async () => {
    const parcial: BenchmarkResult = {
      ...completo,
      rows: completo.rows.slice(0, 3),
      notRun: [
        { profile: "seq1m_q1", direction: "write" },
        { profile: "rnd4k_q32", direction: "read" },
        { profile: "rnd4k_q32", direction: "write" },
        { profile: "rnd4k_q1", direction: "read" },
        { profile: "rnd4k_q1", direction: "write" }
      ]
    };
    await render(BenchmarkResults, { props: { result: parcial } });

    await expect.element(page.getByText(es["tests.benchmark.notRun"]).first()).toBeInTheDocument();
    // No debe aparecer «0 MB/s» para una fila no ejecutada.
    await expect.element(page.getByText("0 MB/s")).not.toBeInTheDocument();
  });

  it("marca «tope de datos alcanzado» cuando una fila lo trae", async () => {
    const conTope: BenchmarkResult = {
      ...completo,
      rows: [{ ...fila("seq1m_q8", "write", 4200), dataCapHit: true }]
    };
    await render(BenchmarkResults, { props: { result: conTope } });
    await expect
      .element(page.getByText(es["tests.benchmark.dataCapHit"], { exact: false }))
      .toBeInTheDocument();
  });
});
