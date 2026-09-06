import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import DiskCard from "./DiskCard.svelte";
import type { DiskSummary } from "$lib/design/types";

/** Cobertura de estados exigida por la constitución §VIII: cada componente del catálogo prueba
 *  vacío, no compatible y dato ausente como mínimo. `DiskCard` no tiene estados propios de "cargando"
 *  ni "error de fuente" — esos los presenta la pantalla que lo contiene (`EmptyState`) —, así que
 *  aquí se cubre lo que sí es responsabilidad del componente: salud, ausencia de dato, ausencia de
 *  soporte SMART, ausencia de volumen y el contrato de navegación por enlace (constitución §XIV).
 *
 *  Localizadores por rol y nombre accesible, nunca por clase (`docs/testing-strategy.md` §12).
 */

function discoBase(overrides: Partial<DiskSummary> = {}): DiskSummary {
  return {
    id: "d1",
    alias: null,
    model: "Samsung 980",
    deviceType: "nvme",
    state: "ok",
    temperatureC: 42,
    percentageUsed: 5,
    activityPercent: 12,
    powerOnHours: 1000,
    unknownReason: null,
    lastReadAt: "2026-09-04T10:00:00Z",
    volumes: [
      {
        id: "v1",
        label: "Datos",
        driveLetters: ["D:"],
        capacityBytes: 500_000_000_000,
        freeBytes: 100_000_000_000,
        mappingConfidence: "exact",
        chkdskAvailable: true,
        isSystemVolume: false
      }
    ],
    ...overrides
  };
}

describe("DiskCard", () => {
  it("sin disco (vacío) no renderiza nada", async () => {
    const { container } = await render(DiskCard, { props: { disk: null } });
    expect(container.textContent?.trim()).toBe("");
  });

  it("un disco correcto muestra su estado y sus tres métricas con valor", async () => {
    await render(DiskCard, { props: { disk: discoBase(), href: "/disks/d1" } });
    await expect.element(page.getByText("Correcto")).toBeInTheDocument();
    await expect.element(page.getByText("42 °C")).toBeInTheDocument();
    await expect.element(page.getByText("5 %")).toBeInTheDocument();
  });

  it("sin compatibilidad SMART se rotula «Sin datos SMART», nunca en rojo ni como avería", async () => {
    await render(DiskCard, {
      props: {
        disk: discoBase({
          state: "unknown",
          unknownReason: "unsupported",
          temperatureC: null,
          percentageUsed: null
        }),
        href: "/disks/d1"
      }
    });
    await expect.element(page.getByText("Sin datos SMART")).toBeInTheDocument();
  });

  it("un disco que dejó de responder: píldora «Sin datos SMART» (gris) y sin enseñar la lectura vieja", async () => {
    // Un `unknown` es «sin datos SMART», sea cual sea el motivo; que cuente para «necesitan
    // atención» lo decide el chrome, no la tarjeta. Magnitudes a «—» aunque el backend conserve
    // la última lectura (boceto §4).
    const { container } = await render(DiskCard, {
      props: {
        disk: discoBase({
          state: "unknown",
          unknownReason: "unreadable",
          temperatureC: 41,
          percentageUsed: 3,
          activityPercent: 0
        }),
        href: "/disks/d1"
      }
    });
    await expect.element(page.getByText("Sin datos SMART")).toBeInTheDocument();
    expect(container.textContent).not.toContain("41 °C");
    await expect.element(page.getByText("—").first()).toBeInTheDocument();
  });

  it("un dato ausente se muestra como «—» discreto con «No disponible» en el title, nunca como 0 ni vacío", async () => {
    const { container } = await render(DiskCard, {
      props: {
        disk: discoBase({ temperatureC: null, percentageUsed: null, activityPercent: null }),
        href: "/disks/d1"
      }
    });
    const ausentes = page.getByText("—");
    await expect.element(ausentes.first()).toBeInTheDocument();
    expect(container.querySelector('[title="No disponible"]')).not.toBeNull();
  });

  it("sin volumen asociado lo dice explícitamente, no deja el hueco en blanco", async () => {
    await render(DiskCard, { props: { disk: discoBase({ volumes: [] }), href: "/disks/d1" } });
    await expect.element(page.getByText("Sin volúmenes montados")).toBeInTheDocument();
  });

  it("con href se renderiza como enlace real, no como botón con callback", async () => {
    await render(DiskCard, { props: { disk: discoBase(), href: "/disks/d1" } });
    const enlace = page.getByRole("link");
    await expect.element(enlace).toBeInTheDocument();
    await expect.element(enlace).toHaveAttribute("href", "/disks/d1");
  });

  it("sin href se renderiza como bloque no interactivo, sin rol de enlace ni de botón", async () => {
    const { container } = await render(DiskCard, { props: { disk: discoBase() } });
    expect(container.querySelector("a")).toBeNull();
  });

  it("el alias, cuando existe, sustituye al modelo como título de la tarjeta", async () => {
    await render(DiskCard, { props: { disk: discoBase({ alias: "Disco de trabajo" }), href: "/disks/d1" } });
    await expect.element(page.getByText("Disco de trabajo")).toBeInTheDocument();
  });
});
