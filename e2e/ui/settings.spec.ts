import { expect, test } from "@playwright/test";
import { RESPUESTAS, apariencia } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Pantalla de ajustes (épica H de `docs/user-stories.md`): frecuencias con límites, restaurar
 *  valores de fábrica, y borrado de todos los datos con frase de confirmación. */

test.describe("ajustes", () => {
  test("cambiar una frecuencia llama a set_setting con la clave y el valor correctos", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/settings");

    const campo = page.getByRole("spinbutton", { name: es["settings.schedule.metricsFast"] });
    await campo.fill("45");
    await campo.blur();

    const llamada = (await llamadas(page)).find((l) => l.comando === "set_setting");
    expect(llamada?.args).toMatchObject({ key: "schedule.metrics_fast_seconds", value: 45 });
  });

  test("restaurar valores de fábrica de un ámbito llama a reset_settings con ese ámbito", async ({
    page
  }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/settings");

    const tarjetaFrecuencias = page.locator("section", { hasText: es["settings.schedule.title"] });
    await tarjetaFrecuencias.getByRole("button", { name: es["settings.cta.restoreDefaults"] }).click();

    const llamada = (await llamadas(page)).find((l) => l.comando === "reset_settings");
    expect(llamada?.args).toMatchObject({ scope: "schedule" });
  });

  test("borrar todos los datos exige escribir la frase exacta antes de confirmar", async ({ page }) => {
    await instalarIpcFalso(page, RESPUESTAS);
    await page.goto("/settings");

    const boton = page.getByRole("button", { name: es["settings.dangerZone.cta"] });
    await expect(boton).toBeDisabled();

    await page.getByRole("textbox", { name: /SmartDisk Monitor/ }).fill("SmartDisk Monitor");
    await expect(boton).toBeEnabled();
    await boton.click();

    const dialogo = page.getByRole("dialog", { name: es["settings.dangerZone.confirmTitle"] });
    await expect(dialogo).toBeVisible();
    await dialogo.getByRole("button", { name: es["settings.dangerZone.confirmLabel"] }).click();

    const llamada = (await llamadas(page)).find((l) => l.comando === "delete_all_data");
    expect(llamada?.args).toMatchObject({ confirmationPhrase: "SmartDisk Monitor" });
  });

  test("el acento de Windows: apagado de fábrica; activarlo lo persiste y sobrescribe los tokens (ADR-035, SC-003)", async ({
    page
  }) => {
    // Instalación nueva: `useSystemAccent` apagado; el acento de Windows de prueba es claro (ámbar).
    await instalarIpcFalso(page, {
      ...RESPUESTAS,
      get_appearance_settings: { ...apariencia, useSystemAccent: false },
      get_system_accent_color: { hex: "#ffb900", palette: [] }
    });
    await page.goto("/settings");

    const conmutador = page.getByRole("switch", {
      name: es["settings.appearance.useSystemAccent.label"]
    });
    await expect(conmutador).toHaveAttribute("aria-checked", "false");

    // Sin activar, la app usa la paleta Ciruela: no hay sobreescritura inline del token de acento.
    const acentoInicial = await page.evaluate(() =>
      document.documentElement.style.getPropertyValue("--sdm-accent")
    );
    expect(acentoInicial).toBe("");

    // Activar → se persiste y `applySystemAccent()` pinta el token (corregido a AA por accessibleAccent).
    await conmutador.click();
    await expect(conmutador).toHaveAttribute("aria-checked", "true");

    await expect
      .poll(() => page.evaluate(() => document.documentElement.style.getPropertyValue("--sdm-accent")))
      .not.toBe("");

    const esAjusteAcento = (l: { comando: string; args: unknown }) =>
      l.comando === "set_setting" &&
      (l.args as { key?: string }).key === "settings.appearance.use_system_accent";

    const activar = (await llamadas(page)).find(esAjusteAcento);
    expect(activar?.args).toMatchObject({
      key: "settings.appearance.use_system_accent",
      value: true
    });

    // El par acento de fondo / texto sobre él cumple AA (SC-003).
    const contraste = await page.evaluate(() => {
      const cs = getComputedStyle(document.documentElement);
      const parse = (v: string): [number, number, number] => {
        const h = v.trim().replace("#", "");
        return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
      };
      const lum = ([r, g, b]: number[]) => {
        const f = (c: number) => {
          const s = c / 255;
          return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
        };
        return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
      };
      const a = lum(parse(cs.getPropertyValue("--sdm-accent")));
      const b = lum(parse(cs.getPropertyValue("--sdm-on-accent")));
      const [hi, lo] = a > b ? [a, b] : [b, a];
      return (hi + 0.05) / (lo + 0.05);
    });
    expect(contraste).toBeGreaterThanOrEqual(4.5);

    // Desactivar → se persiste y se restauran los respaldos (la sobreescritura inline desaparece).
    await conmutador.click();
    await expect(conmutador).toHaveAttribute("aria-checked", "false");
    await expect
      .poll(() => page.evaluate(() => document.documentElement.style.getPropertyValue("--sdm-accent")))
      .toBe("");

    const desactivar = (await llamadas(page)).filter(esAjusteAcento).at(-1);
    expect(desactivar?.args).toMatchObject({ value: false });
  });
});
