import { page } from "vitest/browser";
import { describe, expect, it } from "vitest";
import { render } from "vitest-browser-svelte";
import StatusPill from "./StatusPill.svelte";

/** El color de la píldora nunca viaja solo: siempre hay texto (`label`, obligatorio) y en v3 puede
 *  acompañarlo un icono. El icono va `aria-hidden` — el texto ya dice el estado. */

describe("StatusPill", () => {
  it("muestra siempre el label", async () => {
    await render(StatusPill, { props: { state: "warn", label: "Advertencia" } });
    await expect.element(page.getByText("Advertencia")).toBeInTheDocument();
  });

  it("icon='auto' resuelve el icono por el estado, y va aria-hidden", async () => {
    const { container } = await render(StatusPill, {
      props: { state: "crit", label: "Crítico", icon: "auto" }
    });
    const use = container.querySelector("use")!;
    expect(use.getAttribute("href")).toBe("#i-bolt");
    expect(container.querySelector("svg")!.getAttribute("aria-hidden")).toBe("true");
  });

  it("icon explícito manda sobre el mapa automático", async () => {
    const { container } = await render(StatusPill, {
      props: { state: "unknown", label: "En pausa", icon: "clock" }
    });
    expect(container.querySelector("use")!.getAttribute("href")).toBe("#i-clock");
  });

  it("con icon y withDot a la vez: gana icon, el punto no se pinta", async () => {
    const { container } = await render(StatusPill, {
      props: { state: "ok", label: "Todo en orden", icon: "auto", withDot: true }
    });
    expect(container.querySelector("use")).not.toBeNull();
    expect(container.querySelector('[class*="size-1.5"]')).toBeNull();
  });

  it("sin icon, withDot pinta el punto de color", async () => {
    const { container } = await render(StatusPill, {
      props: { state: "ok", label: "Todo en orden", withDot: true }
    });
    expect(container.querySelector("use")).toBeNull();
    expect(container.querySelector('[class*="size-1.5"]')).not.toBeNull();
  });
});
