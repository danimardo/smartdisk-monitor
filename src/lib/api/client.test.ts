import { beforeEach, describe, expect, it, vi } from "vitest";

/** El mock debe declararse antes de importar el módulo bajo prueba. */
const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));

const saveMock = vi.fn();
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: (...args: unknown[]) => saveMock(...args) }));

const api = await import("./client");
const {
  acknowledgeAlert,
  chooseSavePath,
  getDevices,
  muteAlert,
  refreshNow,
  setDeviceAlias,
  setSetting,
  startBenchmark,
  toAppError
} = api;

/** Esta capa es el único punto de contacto con el backend (constitución §IV). Sus pruebas
 *  verifican dos cosas que a simple vista no se ven: que el nombre y los argumentos de cada
 *  comando son los del contrato, y que ningún fallo llega a la interfaz sin la forma `AppError`
 *  que exige el principio X. */

describe("toAppError — ningún fallo llega sin forma", () => {
  it("deja intacto un AppError que ya viene del backend", () => {
    const original = {
      code: "smartctl.timeout",
      messageKey: "error.smartctlTimeout",
      detail: "exit status 4",
      retryable: true
    };
    expect(toAppError(original)).toBe(original);
  });

  it("envuelve un Error de JavaScript conservando el detalle", () => {
    const e = toAppError(new TypeError("no se pudo serializar"));
    expect(e.code).toBe("ipc.unexpected");
    expect(e.messageKey).toBe("error.unexpected");
    expect(e.detail).toContain("TypeError");
    expect(e.detail).toContain("no se pudo serializar");
    expect(e.retryable).toBe(false);
  });

  it("envuelve una cadena suelta, que es lo que devuelve un pánico de Rust", () => {
    const e = toAppError("called `Option::unwrap()` on a `None` value");
    expect(e.code).toBe("ipc.unexpected");
    expect(e.detail).toContain("Option::unwrap");
  });

  it("nunca pierde información: incluso un objeto raro conserva su contenido", () => {
    const e = toAppError({ algo: "inesperado" });
    expect(e.detail).toContain("inesperado");
  });
});

describe("cada comando usa el nombre y los argumentos del contrato", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it("get_devices no recibe argumentos", async () => {
    invokeMock.mockResolvedValue({
      devices: [],
      excluded: [],
      sources: [],
      paused: false,
      pausedSince: null
    });
    await getDevices();
    expect(invokeMock).toHaveBeenCalledWith("get_devices", undefined);
  });

  it("set_setting pasa clave y valor", async () => {
    await setSetting("appearance.theme", "dark");
    expect(invokeMock).toHaveBeenCalledWith("set_setting", { key: "appearance.theme", value: "dark" });
  });

  it("set_device_alias admite null para borrar el alias", async () => {
    await setDeviceAlias("dev-1", null);
    expect(invokeMock).toHaveBeenCalledWith("set_device_alias", { deviceId: "dev-1", alias: null });
  });

  it("refresh_now transporta el ámbito", async () => {
    await refreshNow("device", "dev-1");
    expect(invokeMock).toHaveBeenCalledWith("refresh_now", { scope: "device", deviceId: "dev-1" });
  });

  it("mute_alert admite null como silencio indefinido", async () => {
    await muteAlert("grp-1", null);
    expect(invokeMock).toHaveBeenCalledWith("mute_alert", { alertGroupId: "grp-1", minutes: null });
  });

  it("acknowledge_alert identifica el grupo, no el disco", async () => {
    await acknowledgeAlert("grp-1");
    expect(invokeMock).toHaveBeenCalledWith("acknowledge_alert", { alertGroupId: "grp-1" });
  });

  it("start_benchmark envía todos los parámetros de la prueba", async () => {
    invokeMock.mockResolvedValue("run-1");
    const params = {
      volumeId: "vol-1",
      sizeBytes: 1024 ** 3,
      blockSizeBytes: 1024 ** 2,
      mode: "sequential" as const,
      passes: 1
    };
    await startBenchmark(params);
    expect(invokeMock).toHaveBeenCalledWith("start_benchmark", params);
  });
});

describe("un rechazo del backend siempre sale como AppError", () => {
  // Con llaves, no sin ellas: `mockReset()` devuelve el propio mock, y una flecha sin llaves lo
  // devolvería. Vitest interpreta un valor de función devuelto por un hook como su teardown y lo
  // ejecutaría al terminar cada test — llamando al mock, que aquí lanza, fuera de todo try/catch.
  beforeEach(() => {
    invokeMock.mockReset();
  });

  /** `.catch(e => e)` convierte el rechazo en resolución: así el valor se inspecciona como un dato
   *  normal y no queda ninguna promesa rechazada suelta que el runner marque como no gestionada. */
  const errorOf = (p: Promise<unknown>) =>
    p.then(
      () => null,
      (e) => e as Record<string, unknown>
    );

  it("aunque el backend rechace con una cadena, como hace un pánico de Rust", async () => {
    // Lanza de forma síncrona: el `try/catch` de `call()` lo captura igual que un rechazo, y así no
    // queda ninguna promesa rechazada suelta que el runner marque como no gestionada.
    invokeMock.mockImplementation(() => {
      throw "algo se rompió";
    });
    const e = await errorOf(getDevices());
    expect(e).toBeTruthy();
    expect(e!.code).toBe("ipc.unexpected");
    expect(e!.retryable).toBe(false);
    expect(String(e!.detail)).toContain("algo se rompió");
  });

  it("y respeta el AppError del backend cuando lo trae", async () => {
    invokeMock.mockImplementation(() => {
      throw { code: "db.locked", messageKey: "error.dbLocked", retryable: true };
    });
    const e = await errorOf(getDevices());
    expect(e).toBeTruthy();
    expect(e!.code).toBe("db.locked");
    expect(e!.retryable).toBe(true);
  });
});

describe("la superficie completa del contrato", () => {
  /** Nombre del comando Tauri que debe invocar cada función exportada. La tabla es el contrato de
   *  `docs/ui-contract.md` escrito de forma ejecutable: una errata en un nombre de comando no la
   *  detecta el compilador —son cadenas— y en ejecución solo se manifiesta cuando alguien abre esa
   *  pantalla concreta. */
  const CONTRATO: Record<string, string> = {
    getAppearanceSettings: "get_appearance_settings",
    getSystemAccentColor: "get_system_accent_color",
    setSetting: "set_setting",
    getSettings: "get_settings",
    resetSettings: "reset_settings",
    getDevices: "get_devices",
    getDeviceDetail: "get_device_detail",
    setDeviceMonitoring: "set_device_monitoring",
    setDeviceAlias: "set_device_alias",
    refreshNow: "refresh_now",
    getMetricSeries: "get_metric_series",
    getAlertGroups: "get_alert_groups",
    getAlertDetail: "get_alert_detail",
    getAlertSmartRawJson: "get_alert_smart_raw_json",
    acknowledgeAlert: "acknowledge_alert",
    muteAlert: "mute_alert",
    unmuteAlert: "unmute_alert",
    archiveAlert: "archive_alert",
    getSystemEvents: "get_system_events",
    getEventRawXml: "get_event_raw_xml",
    startBenchmark: "start_benchmark",
    runChkdskScan: "run_chkdsk_scan",
    runSmartShortTest: "run_smart_short_test",
    cancelTest: "cancel_test",
    getTestRuns: "get_test_runs",
    exportReport: "export_report",
    previewDiagnosticZip: "preview_diagnostic_zip",
    createDiagnosticZip: "create_diagnostic_zip",
    pauseMonitoring: "pause_monitoring",
    resumeMonitoring: "resume_monitoring",
    getAppInfo: "get_app_info",
    deleteAllData: "delete_all_data",
    checkSmartctlDefenderException: "check_smartctl_defender_exception",
    addSmartctlDefenderException: "add_smartctl_defender_exception",
    getLogLevel: "get_log_level",
    setLogLevel: "set_log_level",
    openLogFolder: "open_log_folder"
  };

  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it.each(Object.entries(CONTRATO))("%s invoca '%s'", async (fnName, comando) => {
    const fn = (api as Record<string, unknown>)[fnName] as (...a: unknown[]) => Promise<unknown>;
    expect(fn, `${fnName} no está exportado`).toBeTypeOf("function");

    // El mock devuelve `undefined`, que no cumple ningún esquema, así que los comandos con
    // respuesta rechazarán: es exactamente lo que debe pasar (constitución §XI) y aquí no estorba,
    // porque lo que se comprueba es el nombre del comando invocado, no su carga útil.
    await fn({}).catch(() => undefined);
    expect(invokeMock.mock.calls[0][0]).toBe(comando);
  });

  it("todo comando con respuesta la valida: `undefined` no pasa", async () => {
    // Si alguien añade un comando y olvida su esquema, este test lo caza.
    const conRespuesta = ["getDevices", "getAppInfo", "getAppearanceSettings", "getSystemAccentColor"];
    for (const nombre of conRespuesta) {
      const fn = (api as Record<string, unknown>)[nombre] as () => Promise<unknown>;
      const err = await fn().then(
        () => null,
        (e) => e as { code: string }
      );
      expect(err, `${nombre} aceptó una respuesta vacía`).toBeTruthy();
      expect(err!.code).toBe("ipc.schema_mismatch");
    }
  });

  it("no hay funciones exportadas fuera del contrato", () => {
    // `chooseSavePath` no está en `CONTRATO`: envuelve el diálogo nativo de guardado
    // (`@tauri-apps/plugin-dialog`, ADR-031), no uno de los comandos Tauri registrados en
    // `commands/mod.rs` — no tiene entrada en `docs/ui-contract.md` §3 porque no es un comando
    // propio, es un permiso de plugin (`dialog:allow-save` en `capabilities/default.json`).
    const exportadas = Object.entries(api)
      .filter(([, v]) => typeof v === "function")
      .map(([k]) => k)
      .filter((k) => k !== "toAppError" && k !== "chooseSavePath");
    const noContempladas = exportadas.filter((k) => !(k in CONTRATO));
    expect(noContempladas, "hay comandos sin declarar en el contrato").toEqual([]);
  });
});

describe("chooseSavePath", () => {
  beforeEach(() => saveMock.mockReset());

  it("pasa el nombre y las extensiones del filtro al diálogo nativo", async () => {
    saveMock.mockResolvedValueOnce("C:\\destino\\informe.csv");
    const ruta = await chooseSavePath({
      defaultFileName: "informe.csv",
      filterName: "CSV",
      extensions: ["csv"]
    });
    expect(ruta).toBe("C:\\destino\\informe.csv");
    expect(saveMock).toHaveBeenCalledWith({
      defaultPath: "informe.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }]
    });
  });

  it("un diálogo cancelado devuelve null, no un error", async () => {
    saveMock.mockResolvedValueOnce(null);
    const ruta = await chooseSavePath({ defaultFileName: "x.csv", filterName: "CSV", extensions: ["csv"] });
    expect(ruta).toBeNull();
  });

  it("un fallo del diálogo se normaliza a AppError, igual que un invoke fallido", async () => {
    saveMock.mockRejectedValueOnce(new Error("el usuario no tiene permiso"));
    await expect(
      chooseSavePath({ defaultFileName: "x.csv", filterName: "CSV", extensions: ["csv"] })
    ).rejects.toMatchObject({ code: "ipc.unexpected" });
  });
});
