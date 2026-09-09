import { page, userEvent } from "vitest/browser";
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
    activity: {
      estado: "valido",
      mediaPercent: 12,
      picoPercent: 38,
      muestras: 30,
      ventanaSegundos: 30
    },
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
          activity: {
            estado: "no_disponible",
            mediaPercent: null,
            picoPercent: null,
            muestras: 0,
            ventanaSegundos: 30
          }
        }),
        href: "/disks/d1"
      }
    });
    await expect.element(page.getByText("Sin datos SMART")).toBeInTheDocument();
    expect(container.textContent).not.toContain("41 °C");
    await expect.element(page.getByText("—").first()).toBeInTheDocument();
  });

  it("un dato ausente se muestra como «—» discreto, nunca como 0 ni vacío", async () => {
    await render(DiskCard, {
      props: {
        disk: discoBase({
          temperatureC: null,
          percentageUsed: null,
          activity: {
            estado: "no_disponible",
            mediaPercent: null,
            picoPercent: null,
            muestras: 0,
            ventanaSegundos: 30
          }
        }),
        href: "/disks/d1"
      }
    });
    await expect.element(page.getByText("—").first()).toBeInTheDocument();
    expect(page.getByText("0", { exact: true }).query()).toBeNull();
  });

  it("al pasar el ratón por una métrica sale un tooltip que la explica y da un veredicto; no añade foco", async () => {
    const { container } = await render(DiskCard, {
      props: { disk: discoBase({ percentageUsed: 5 }), href: "/disks/d1" }
    });
    // El panel: tooltip solo con el ratón, la tarjeta sigue siendo un enlace limpio.
    expect(container.querySelector("button")).toBeNull();

    await userEvent.hover(page.getByText("Desg."));
    const texto = container.querySelector('[role="tooltip"]')?.textContent ?? "";
    expect(texto).toContain("Percentage Used"); // la explicación
    expect(texto).toContain("95"); // el veredicto: queda ~95 %
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

  it("como enlace lleva la marca del realce de fondo al pasar el ratón (pátina de acento, no aclarado)", async () => {
    // La `::after` que pinta el violeta translúcido cuelga de `.sdm-hover-bloque`; sin ese enganche
    // la tarjeta no tendría feedback de que se puede pulsar. El color se verifica sobre el material
    // compuesto en la prueba de contraste, no aquí.
    const { container } = await render(DiskCard, { props: { disk: discoBase(), href: "/disks/d1" } });
    expect(container.querySelector("a")?.classList.contains("sdm-hover-bloque")).toBe(true);
  });

  it("sin href no lleva la marca del realce: un bloque no interactivo no responde al ratón", async () => {
    const { container } = await render(DiskCard, { props: { disk: discoBase() } });
    expect(container.querySelector(".sdm-hover-bloque")).toBeNull();
  });

  it("sin href se renderiza como bloque no interactivo, sin rol de enlace ni de botón", async () => {
    const { container } = await render(DiskCard, { props: { disk: discoBase() } });
    expect(container.querySelector("a")).toBeNull();
  });

  it("el alias, cuando existe, sustituye al modelo como título de la tarjeta", async () => {
    await render(DiskCard, { props: { disk: discoBase({ alias: "Disco de trabajo" }), href: "/disks/d1" } });
    await expect.element(page.getByText("Disco de trabajo")).toBeInTheDocument();
  });

  // ---- actividad: pico de la ventana con sus tres estados (spec 007, FR-012 / FR-014) ----

  const actividad = (
    estado: "valido" | "parcial" | "no_disponible",
    pico: number | null
  ): DiskSummary["activity"] => ({
    estado,
    mediaPercent: pico === null ? null : Math.round(pico / 2),
    picoPercent: pico,
    muestras: estado === "no_disponible" ? 0 : 20,
    ventanaSegundos: 30
  });

  it("actividad `valido`: el panel muestra el pico de la ventana", async () => {
    await render(DiskCard, {
      props: { disk: discoBase({ activity: actividad("valido", 73) }), href: "/disks/d1" }
    });
    await expect.element(page.getByText("73 %")).toBeInTheDocument();
  });

  it("actividad `parcial`: el pico va con la marca «~» de que la ventana aún no está completa", async () => {
    await render(DiskCard, {
      props: { disk: discoBase({ activity: actividad("parcial", 40) }), href: "/disks/d1" }
    });
    await expect.element(page.getByText("~40 %")).toBeInTheDocument();
  });

  it("actividad `no_disponible`: «—» discreto, nunca 0 %", async () => {
    const { container } = await render(DiskCard, {
      props: { disk: discoBase({ activity: actividad("no_disponible", null) }), href: "/disks/d1" }
    });
    await expect.element(page.getByText("—").first()).toBeInTheDocument();
    expect(container.textContent).not.toContain("0 %");
  });

  it("la actividad no se apaga por falta de lectura SMART fresca: sigue su propio estado", async () => {
    await render(DiskCard, {
      props: {
        disk: discoBase({
          state: "unknown",
          unknownReason: "unreadable",
          temperatureC: 41,
          activity: actividad("valido", 66)
        }),
        href: "/disks/d1"
      }
    });
    // Temperatura y desgaste sí se ocultan; la actividad no.
    await expect.element(page.getByText("66 %")).toBeInTheDocument();
  });
});
