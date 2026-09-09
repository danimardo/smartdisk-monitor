import { page, userEvent } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import HeroPanel from "./HeroPanel.svelte";
import es from "$lib/i18n/es.json";
import type { DiskSummary } from "$lib/design/types";

/** `HeroPanel` es el dato dominante del panel (v3, ADR-034). La pantalla elige el disco; el
 *  componente solo presenta. Se cubren aquí sus estados propios: cargando, sin serie, sin SMART,
 *  «todo en orden» y disco con alerta. Localizadores por texto visible, nunca por clase. */

function discoBase(overrides: Partial<DiskSummary> = {}): DiskSummary {
  return {
    id: "d1",
    alias: null,
    model: "Samsung 990 Pro",
    deviceType: "nvme",
    state: "ok",
    temperatureC: 44,
    percentageUsed: 3,
    activity: { estado: "valido", mediaPercent: 8, picoPercent: 22, muestras: 30, ventanaSegundos: 30 },
    powerOnHours: 500,
    unknownReason: null,
    lastReadAt: "2026-09-06T10:00:00Z",
    volumes: [],
    ...overrides
  };
}

const serie = [
  { t: 1_000, v: 40 },
  { t: 2_000, v: 43 },
  { t: 3_000, v: 44 }
];

/** Seis puntos que abarcan 25 min: suficiente para dibujar una onda. */
const serieDensa = Array.from({ length: 6 }, (_, i) => ({ t: i * 300_000, v: 42 + (i % 3) }));

describe("HeroPanel", () => {
  it("cargando: muestra el esqueleto de la cifra, no un valor", async () => {
    await render(HeroPanel, { props: { disk: discoBase(), series: serie, loading: true } });
    expect(page.getByText("44 °C").query()).toBeNull();
  });

  it("estado correcto: rotula «Todo en orden» y no colorea como alarma", async () => {
    await render(HeroPanel, { props: { disk: discoBase(), series: serie } });
    await expect.element(page.getByText(es["dashboard.hero.allGood"])).toBeInTheDocument();
    await expect.element(page.getByText("44 °C")).toBeInTheDocument();
  });

  it("sin serie de temperatura: lo dice explícitamente en vez de dejar el hueco mudo", async () => {
    await render(HeroPanel, { props: { disk: discoBase(), series: [] } });
    await expect.element(page.getByText(es["dashboard.hero.noSeries"])).toBeInTheDocument();
  });

  it("con pocas muestras (app recién abierta): «Recopilando datos», no una rayita casi plana", async () => {
    await render(HeroPanel, { props: { disk: discoBase(), series: serie } });
    await expect.element(page.getByText(es["dashboard.hero.collecting"])).toBeInTheDocument();
  });

  it("con muestras suficientes: el pie dice la ventana y no «Recopilando datos»", async () => {
    await render(HeroPanel, {
      props: { disk: discoBase(), series: serieDensa, windowLabel: "Ventana: 25 min" }
    });
    await expect.element(page.getByText("Ventana: 25 min")).toBeInTheDocument();
    expect(page.getByText(es["dashboard.hero.collecting"]).query()).toBeNull();
  });

  it("la curva de fondo tiene cursor de lectura: el teclado recorre los puntos y anuncia la temperatura", async () => {
    const { container } = await render(HeroPanel, {
      props: { disk: discoBase(), series: serieDensa }
    });
    // Dos SVG: el decorativo (aria-hidden) y la capa de lectura (role="img", enfocable).
    const lectura = container.querySelector('svg[role="img"]') as unknown as HTMLElement;
    expect(lectura).not.toBeNull();
    lectura.focus();
    await userEvent.keyboard("{Home}");
    const viva = container.querySelector('[aria-live="polite"]')!;
    expect(viva.textContent).toContain("42");
    expect(viva.textContent).toContain("°C");
  });

  it("disco que dejó de responder: píldora «Sin datos SMART», explica que dejó de responder, cifra «No disponible»", async () => {
    await render(HeroPanel, {
      props: {
        disk: discoBase({ state: "unknown", unknownReason: "unreadable", temperatureC: 41 }),
        series: [],
        alertId: "a1",
        onviewalert: () => {}
      }
    });
    await expect.element(page.getByText(es["disk.noSmartData"])).toBeInTheDocument();
    await expect.element(page.getByText(es["disk.noSmartUnreadable"])).toBeInTheDocument();
    expect(page.getByText("41 °C").query()).toBeNull();
    // Aunque sea gris, con una alerta activa ofrece verla.
    await expect
      .element(page.getByRole("button", { name: es["dashboard.hero.viewAlert"] }))
      .toBeInTheDocument();
  });

  it("disco sin SMART: explica por qué y no muestra la temperatura como cifra", async () => {
    await render(HeroPanel, {
      props: {
        disk: discoBase({ state: "unknown", unknownReason: "unsupported", temperatureC: null }),
        series: []
      }
    });
    await expect.element(page.getByText(es["disk.noSmartExplain"])).toBeInTheDocument();
    await expect.element(page.getByText(es["common.notAvailable"])).toBeInTheDocument();
  });

  it("disco con alerta: muestra la explicación y ofrece «Ver la alerta»", async () => {
    await render(HeroPanel, {
      props: {
        disk: discoBase({ state: "warn" }),
        series: serie,
        alertId: "a1",
        explanation: "La temperatura supera el umbral configurado.",
        onviewalert: () => {}
      }
    });
    await expect.element(page.getByText("La temperatura supera el umbral configurado.")).toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: es["dashboard.hero.viewAlert"] }))
      .toBeInTheDocument();
  });

  it("callbacks opcionales: sin onopen ni onviewalert renderiza sin lanzar", async () => {
    const { container } = await render(HeroPanel, { props: { disk: discoBase(), series: serie } });
    expect(container.textContent?.length).toBeGreaterThan(0);
  });
});
