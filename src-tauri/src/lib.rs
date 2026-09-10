//! SmartDisk Monitor — biblioteca de la aplicación.
//!
//! El binario (`main.rs`) es una línea; todo vive aquí para que el dominio se pueda testear sin
//! arrancar una ventana. La aplicación se ejecuta siempre elevada (ADR-004) y vive en la bandeja
//! del sistema mientras está activa.

use tauri::Manager;

pub mod alerts;
pub mod collectors;
pub mod commands;
pub mod domain;
pub mod error;
pub mod logging;
pub mod persistence;
pub mod platform;
pub mod reporting;
#[cfg(test)]
pub mod test_util;
pub mod tests;

/// Lista cerrada de comandos invocables desde la interfaz. Nada fuera de aquí es alcanzable:
/// no hay shell genérica ni `fs` abierto (ADR-004, `docs/ui-contract.md` §5).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // El registro arranca antes que nada: un fallo al abrir la ventana también debe quedar escrito.
    // Precedencia del nivel: --log-level > settings > info (constitución §XV). `settings` todavía
    // no es legible aquí (no hay base abierta): arranca con el nivel de línea de órdenes o `info`,
    // y se recarga en cuanto `AppState` existe, en el `.setup()` de más abajo.
    let args: Vec<String> = std::env::args().collect();
    let level = logging::level_from_cli(&args).unwrap_or(logging::LogLevel::Info);
    let log_dir = platform::paths::log_dir();
    let (_guard, manejador_nivel) = logging::init(level, &log_dir);

    tauri::Builder::default()
        // Primero de todos a propósito: los plugins se ejecutan en el orden en que se registran,
        // y este tiene que decidir si el proceso sigue vivo antes de que nada más se inicialice.
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // Corre en el proceso que YA estaba en marcha. El segundo termina solo.
            tracing::info!(
                argumentos = ?argv,
                directorio = %cwd,
                "segunda instancia rechazada; se restaura la ventana existente"
            );
            platform::ventana::restaurar_ventana_principal(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // apariencia y ajustes
            commands::get_appearance_settings,
            commands::get_system_accent_color,
            commands::set_setting,
            commands::get_settings,
            commands::reset_settings,
            // ayuda con IA (spec 005-explicacion-ia)
            commands::estado_ia,
            commands::guardar_clave_ia,
            commands::activar_ayuda_ia_compartida,
            commands::probar_clave_ia,
            commands::borrar_clave_ia,
            commands::establecer_envio_sin_revision,
            commands::listar_modelos_ia,
            commands::explicar_detalle_tecnico,
            // inventario
            commands::get_devices,
            commands::get_device_detail,
            commands::set_device_monitoring,
            commands::set_device_alias,
            commands::refresh_now,
            // series
            commands::get_metric_series,
            // alertas
            commands::get_alert_groups,
            commands::get_alert_detail,
            commands::get_alert_smart_raw_json,
            commands::acknowledge_alert,
            commands::mute_alert,
            commands::unmute_alert,
            commands::archive_alert,
            commands::ignore_alert,
            commands::unignore_alert,
            // eventos
            commands::get_system_events,
            commands::get_event_raw_xml,
            // pruebas
            commands::start_benchmark,
            commands::run_chkdsk_scan,
            commands::run_smart_short_test,
            commands::cancel_test,
            commands::get_test_runs,
            // informes
            commands::export_report,
            commands::preview_diagnostic_zip,
            commands::create_diagnostic_zip,
            // ciclo de vida
            commands::pause_monitoring,
            commands::resume_monitoring,
            commands::get_app_info,
            commands::delete_all_data,
            commands::check_smartctl_defender_exception,
            commands::add_smartctl_defender_exception,
            // registro
            commands::log_from_ui,
            commands::get_log_level,
            commands::set_log_level,
            commands::open_log_folder,
        ])
        .setup(move |app| {
            // La base se abre aquí, no en cada comando: una sola conexión compartida y las
            // migraciones ya aplicadas antes de que la interfaz pueda pedir nada.
            let estado = persistence::db::AppState::open(&platform::paths::data_dir())
                .expect("no se pudo abrir la base de datos ni aplicar sus migraciones");

            // Ahora que la base existe, se puede completar la precedencia del nivel de registro
            // (§XV): si la línea de órdenes no forzó ninguno, se recarga con el que diga
            // `logging.verbose`. Antes de este punto solo podía saberse el de la línea de órdenes.
            if logging::level_from_cli(&args).is_none() {
                let verbose = {
                    let conn = estado
                        .conn
                        .lock()
                        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
                    commands::leer_ajuste_bool(&conn, "logging.verbose", false)
                };
                if verbose {
                    let _ = manejador_nivel.establecer(logging::LogLevel::Debug);
                }
            }
            app.manage(manejador_nivel);
            app.manage(estado);

            // La ventana nace oculta y se muestra cuando el frontend ha pintado: así no se ve un
            // rectángulo blanco antes de que se aplique el tema. Antes de mostrarla se le aplica la
            // geometría de la última sesión (ADR-040): al estar oculta, no hay salto.
            if let Some(ventana) = app.get_webview_window(platform::ventana::VENTANA_PRINCIPAL) {
                let estado = app.state::<persistence::db::AppState>();
                let conn = estado
                    .conn
                    .lock()
                    .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
                platform::ventana::aplicar_geometria_guardada(&ventana, &conn);
            }
            platform::ventana::restaurar_ventana_principal(app.handle());

            // El icono de la bandeja vive mientras la aplicación vive (FR-012): se construye una
            // sola vez aquí y se recalcula desde los comandos que pueden cambiar su color.
            platform::bandeja::instalar(app.handle())?;

            // El bucle de recopilación en segundo plano (T020/T021/T022): sondea cada 1 s desde su
            // propio hilo bloqueante, reanuda siempre al arrancar (`AppState.paused` nunca se
            // persiste) y se detiene con `RunEvent::Exit`/`ExitRequested`, más abajo.
            commands::iniciar_planificador(app.handle().clone());

            // Cerrar con la X no termina la aplicación por defecto: sigue monitorizando en la
            // bandeja (`docs/product-specification.md` §3), salvo que `lifecycle.close_action`
            // diga lo contrario (US-072, T099, `docs/open-questions.md` J.19/J.32) — se lee en
            // cada cierre, no solo al arrancar, porque la pantalla de Ajustes puede cambiarlo
            // mientras la aplicación sigue abierta.
            if let Some(ventana) = app.get_webview_window(platform::ventana::VENTANA_PRINCIPAL) {
                let ventana_a_ocultar = ventana.clone();
                let app_handle = app.handle().clone();
                ventana.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let salir = {
                            let estado = app_handle.state::<persistence::db::AppState>();
                            let conn = estado.conn.lock().expect(
                                "el mutex de la conexión no se envenena: sin pánicos dentro",
                            );
                            // La ventana sigue visible aquí: es el momento de guardar su geometría
                            // (ADR-040), tanto si se minimiza como si se cierra de verdad.
                            platform::ventana::persistir_geometria(&ventana_a_ocultar, &conn);
                            commands::leer_ajuste_string(
                                &conn,
                                "lifecycle.close_action",
                                "minimize",
                            ) == "exit"
                        };
                        platform::ventana::gestionar_cierre(&app_handle, &ventana_a_ocultar, salir);
                    }
                });
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error al arrancar SmartDisk Monitor")
        .run(|app_handle, event| {
            // Único punto de parada del bucle en segundo plano, sea cual sea la vía de salida
            // (cierre real de ventana, "Salir" de la bandeja, señal del sistema): más fiable que
            // interceptar cada `app.exit(0)` por separado.
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                let estado = app_handle.state::<persistence::db::AppState>();
                estado
                    .detener_planificador
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
            // «Salir» desde la bandeja llama a `app.exit(0)` directamente (no pasa por
            // `CloseRequested`): se guarda aquí la geometría para esa vía y para el apagado del
            // sistema (ADR-040). La ventana sigue accesible en `ExitRequested`.
            if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
                if let Some(ventana) =
                    app_handle.get_webview_window(platform::ventana::VENTANA_PRINCIPAL)
                {
                    let estado = app_handle.state::<persistence::db::AppState>();
                    let conn = estado
                        .conn
                        .lock()
                        .expect("el mutex de la conexión no se envenena: sin pánicos dentro");
                    platform::ventana::persistir_geometria(&ventana, &conn);
                }
            }
        });
}
