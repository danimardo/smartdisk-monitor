import { expect, test } from "@playwright/test";
import { RESPUESTAS, apariencia, settingsDeFabrica } from "./fixtures/respuestas";
import { instalarIpcFalso, llamadas } from "./ipc-falso";
import es from "../../src/lib/i18n/es.json" with { type: "json" };

/** Asistente inicial (US-002, `specs/002-rediseno-v3/` US8): guardián de arranque (FR-032/043),
 *  los cuatro pasos, «Omitir» y «Repetir la configuración inicial». */

/** Inventario sin alias ni exclusiones: una instalación de verdad recién hecha. */
const inventarioLimpio = {
  devices: [
    {
      id: "disk-0",
      alias: null,
      model: "Samsung SSD 990 PRO 2TB",
      deviceType: "nvme",
      state: "ok",
      temperatureC: 41,
      percentageUsed: 3,
      activity: { estado: "valido", mediaPercent: 12, picoPercent: 44, muestras: 30, ventanaSegundos: 30 },
      powerOnHours: 100,
      lastReadAt: "2026-09-06T10:00:00Z",
      volumes: [
        {
          id: "vol-c",
          label: "Sistema",
          driveLetters: ["C:"],
          capacityBytes: 2_000_398_934_016,
          freeBytes: 1_200_000_000_000,
          mappingConfidence: "exact",
          chkdskAvailable: true,
          isSystemVolume: true
        }
      ]
    }
  ],
  excluded: [],
  sources: [{ source: "smartctl", status: "ok", lastSuccessAt: null, lastAttemptAt: null }],
  paused: false,
  pausedSince: null
};

const sinCompletar = { ...settingsDeFabrica, onboarding: { completedAt: null } };

/** Apariencia de una instalación recién hecha: sigue al sistema, sin idioma forzado. */
const aparienciaDeFabrica = {
  theme: "system",
  language: null,
  systemLocale: "es-ES",
  useSystemAccent: false,
  sidebarExpanded: false
};

function respuestas(overrides: Record<string, unknown>) {
  return {
    ...RESPUESTAS,
    get_settings: sinCompletar,
    get_appearance_settings: aparienciaDeFabrica,
    get_devices: inventarioLimpio,
    ...overrides
  };
}

test.describe("asistente inicial", () => {
  test("primer arranque sin configuración previa redirige al asistente y lo recorre entero", async ({
    page
  }) => {
    await instalarIpcFalso(page, respuestas({}));
    await page.goto("/");

    // FR-032: redirección al asistente.
    await expect(page).toHaveURL(/\/onboarding$/);
    await expect(page.getByText(es["onboarding.welcome.guarantee"])).toBeVisible();

    // Paso 1 → 2
    await page.getByRole("button", { name: es["onboarding.welcome.cta"] }).click();
    await expect(
      page.getByRole("heading", { name: /Samsung SSD 990 PRO 2TB|discos en este equipo/ })
    ).toBeVisible();

    // Paso 2: renombrar el disco y continuar.
    await page.getByRole("textbox", { name: es["onboarding.disks.aliasLabel"] }).fill("Disco del sistema");
    await page.getByRole("button", { name: es["onboarding.disks.cta"] }).click();

    const trasPaso2 = await llamadas(page);
    expect(trasPaso2.map((l) => l.comando)).toContain("set_device_alias");
    expect(trasPaso2.map((l) => l.comando)).toContain("set_device_monitoring");

    // Paso 3: elegir un perfil y continuar.
    await expect(page.getByRole("heading", { name: es["onboarding.alerts.title"] })).toBeVisible();
    await page.getByRole("radio", { name: new RegExp(es["settings.alerts.profile.cautious"]) }).click();
    expect((await llamadas(page)).some((l) => l.comando === "set_setting")).toBe(true);
    await page.getByRole("button", { name: es["common.continue"] }).click();

    // Paso 4: ayuda con IA (opcional, spec 005) — se deja en blanco y se continúa.
    await expect(page.getByRole("heading", { name: es["onboarding.ai.title"] })).toBeVisible();
    await page.getByRole("button", { name: es["onboarding.ai.cta.skip"] }).click();

    // Paso 5: terminar.
    await expect(page.getByRole("heading", { name: es["onboarding.done.title"] })).toBeVisible();
    await page.getByRole("button", { name: es["onboarding.done.cta"] }).click();

    await expect
      .poll(async () =>
        (await llamadas(page)).some(
          (l) =>
            l.comando === "set_setting" &&
            (l.args as { key?: string }).key === "settings.onboarding.completed_at"
        )
      )
      .toBe(true);
    await expect(page).toHaveURL(/\/$/);
  });

  test("«Omitir» aplica el perfil recomendado, marca completado y va al panel", async ({ page }) => {
    await instalarIpcFalso(page, respuestas({}));
    await page.goto("/onboarding");

    await page.getByRole("button", { name: es["onboarding.skip"] }).click();

    await expect
      .poll(async () => {
        const sets = (await llamadas(page)).filter((l) => l.comando === "set_setting");
        return (
          sets.some((l) => (l.args as { key?: string }).key === "alerts.profile") &&
          sets.some((l) => (l.args as { key?: string }).key === "settings.onboarding.completed_at")
        );
      })
      .toBe(true);
    await expect(page).toHaveURL(/\/$/);
  });

  test("una instalación con configuración previa NO ve el asistente y se marca como completada", async ({
    page
  }) => {
    // `completedAt` nulo pero un disco con alias → FR-043: no redirige, graba la marca y sigue.
    await instalarIpcFalso(
      page,
      respuestas({
        get_devices: {
          ...inventarioLimpio,
          devices: [{ ...inventarioLimpio.devices[0], alias: "Mi SSD" }]
        }
      })
    );
    await page.goto("/");

    // No redirige y, en cuanto el guardián resuelve, graba la marca (FR-043).
    await expect(page.getByRole("main")).toBeVisible();
    await expect
      .poll(async () =>
        (await llamadas(page)).some(
          (l) =>
            l.comando === "set_setting" &&
            (l.args as { key?: string }).key === "settings.onboarding.completed_at"
        )
      )
      .toBe(true);
    await expect(page).toHaveURL(/\/$/);
  });

  test("paso 2 sin discos detectados: estado vacío con «Volver a buscar», se puede continuar igualmente", async ({
    page
  }) => {
    await instalarIpcFalso(
      page,
      respuestas({
        get_devices: { devices: [], excluded: [], sources: [], paused: false, pausedSince: null }
      })
    );
    await page.goto("/onboarding");
    await page.getByRole("button", { name: es["onboarding.welcome.cta"] }).click();

    await expect(page.getByText(es["onboarding.disks.empty"])).toBeVisible();
    // El primario del pie sigue disponible: continuar igualmente (FR-038).
    await expect(page.getByRole("button", { name: es["onboarding.disks.cta"] })).toBeEnabled();
  });

  test("paso 2 con fallo de detección: EmptyState de error con «Reintentar», y «Omitir» siempre disponible", async ({
    page
  }) => {
    await instalarIpcFalso(page, {
      ...respuestas({}),
      // el `+page.ts` hace `Promise.allSettled`: un `get_devices` ausente se resuelve como rechazo
      get_devices: undefined
    });
    await page.goto("/onboarding");
    await page.getByRole("button", { name: es["onboarding.welcome.cta"] }).click();

    await expect(page.getByText(es["onboarding.disks.error"])).toBeVisible();
    await expect(page.getByRole("button", { name: es["common.retry"] })).toBeVisible();
    await expect(page.getByRole("button", { name: es["onboarding.skip"] })).toBeVisible();
  });

  test("«Repetir la configuración inicial» desde Ajustes reabre el asistente sin borrar nada", async ({
    page
  }) => {
    await instalarIpcFalso(page, { ...RESPUESTAS, get_appearance_settings: apariencia });
    await page.goto("/settings");

    await page.getByRole("button", { name: es["settings.onboarding.repeat"] }).click();
    await expect(page).toHaveURL(/\/onboarding$/);

    const marca = (await llamadas(page)).find(
      (l) =>
        l.comando === "set_setting" &&
        (l.args as { key?: string }).key === "settings.onboarding.completed_at" &&
        (l.args as { value?: unknown }).value === null
    );
    expect(marca, "pone la marca a null, no borra datos").toBeTruthy();
  });
});
