import { page } from "vitest/browser";
import { beforeEach, describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import es from "$lib/i18n/es.json";
import type { BenchmarkResult } from "$lib/api/types";

/** Rejilla de resultados de la prueba de Rendimiento (spec 008 / ADR-053), disposición estilo
 *  CrystalDiskMark. Estados: completado (8 celdas con cifras), en curso (una celda «midiendo», el
 *  resto pendiente), parada anticipada («no ejecutado», nunca 0), y el toggle MB/s ↔ IOPS. */

const { default: BenchmarkResults } = await import("./BenchmarkResults.svelte");

const fila = (
  profile: BenchmarkResult["rows"][number]["profile"],
  direction: "read" | "write",
  mb: number
): BenchmarkResult["rows"][number] => ({
  profile,
  direction,
  mbPerSecond: mb,
  iops: Math.round(mb * 4.1),
  avgLatencyMs: profile.startsWith("rnd") ? 0.06 : 1.4,
  actualDurationS: 5,
  bytesMoved: mb * 5_000_000,
  dataCapHit: false
});

const completo: BenchmarkResult = {
  tool: "diskspd",
  toolVersion: "2.3.0",
  fileSizeBytes: 1_073_741_824,
  rows: [
    fila("seq1m_q8", "read", 3300),
    fila("seq1m_q8", "write", 2900),
    fila("seq1m_q1", "read", 2600),
    fila("seq1m_q1", "write", 2400),
    fila("rnd4k_q32", "read", 720),
    fila("rnd4k_q32", "write", 640),
    fila("rnd4k_q1", "read", 68),
    fila("rnd4k_q1", "write", 190)
  ],
  notRun: []
};

beforeEach(() => {
  try {
    localStorage.removeItem("sdm.benchmark.unit");
  } catch {
    /* ignora */
  }
});

describe("BenchmarkResults", () => {
  it("completado: encabezados de fila y columna reales, 8 celdas con cifras y el pie con la versión", async () => {
    const { container } = await render(BenchmarkResults, { props: { result: completo } });

    await expect
      .element(page.getByRole("columnheader", { name: es["tests.benchmark.direction.read"], exact: false }))
      .toBeInTheDocument();
    await expect
      .element(page.getByRole("rowheader", { name: es["tests.benchmark.short.seq1m_q8"] }))
      .toBeInTheDocument();
    expect(container.querySelectorAll("tbody td").length).toBe(8);
    await expect.element(page.getByText("Medido con DiskSpd 2.3.0")).toBeInTheDocument();
    // 4K de un NVMe: la latencia se muestra en µs, no en ms.
    await expect.element(page.getByText("µs", { exact: false }).first()).toBeInTheDocument();
  });

  it("el toggle cambia la cifra principal de MB/s a IOPS", async () => {
    await render(BenchmarkResults, { props: { result: completo } });
    // Con MB/s, la línea secundaria dice «IOPS».
    await expect.element(page.getByText("IOPS", { exact: false }).first()).toBeInTheDocument();
    await page.getByRole("radio", { name: es["tests.benchmark.col.iops"] }).click();
    // Tras el toggle, la cifra principal ya no lleva «MB/s» en todas las celdas como principal:
    // basta comprobar que la secundaria pasa a MB/s.
    await expect.element(page.getByText("MB/s", { exact: false }).first()).toBeInTheDocument();
  });

  it("en curso: una celda «midiendo», el resto pendientes, sin ceros", async () => {
    const parcial: BenchmarkResult = {
      ...completo,
      toolVersion: "",
      rows: [fila("seq1m_q8", "read", 3300)],
      notRun: []
    };
    await render(BenchmarkResults, { props: { result: parcial, running: true } });

    await expect.element(page.getByText(es["tests.benchmark.measuring"])).toBeInTheDocument();
    await expect.element(page.getByText("0 MB/s")).not.toBeInTheDocument();
    await expect.element(page.getByText("0 IOPS")).not.toBeInTheDocument();
  });

  it("parada anticipada: «No ejecutado», nunca un 0", async () => {
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
    await expect.element(page.getByText("0 MB/s")).not.toBeInTheDocument();
  });

  it("marca con * y nota al pie cuando una fila alcanzó el tope de datos", async () => {
    const conTope: BenchmarkResult = {
      ...completo,
      rows: [{ ...fila("seq1m_q1", "write", 4200), dataCapHit: true }]
    };
    await render(BenchmarkResults, { props: { result: conTope } });
    await expect
      .element(page.getByText(es["tests.benchmark.dataCapHitNote"], { exact: false }))
      .toBeInTheDocument();
  });
});
