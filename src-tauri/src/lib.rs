//! SmartDisk Monitor — biblioteca de la aplicación.
//!
//! El binario (`main.rs`) es una línea; todo vive aquí para que el dominio se pueda testear sin
//! arrancar una ventana. La aplicación se ejecuta siempre elevada (ADR-004) y vive en la bandeja
//! del sistema mientras está activa.

pub mod commands;
pub mod error;
pub mod logging;
pub mod platform;

/// Lista cerrada de comandos invocables desde la interfaz. Nada fuera de aquí es alcanzable:
/// no hay shell genérica ni `fs` abierto (ADR-004, `docs/ui-contract.md` §5).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // El registro arranca antes que nada: un fallo al abrir la ventana también debe quedar escrito.
    // Precedencia del nivel: --log-level > settings > info (constitución §XV). `settings` todavía
    // no es legible aquí, así que de momento solo manda la línea de órdenes.
    let args: Vec<String> = std::env::args().collect();
    let level = logging::level_from_cli(&args).unwrap_or(logging::LogLevel::Info);
    let log_dir = platform::paths::log_dir();
    let _guard = logging::init(level, &log_dir);

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
        .invoke_handler(tauri::generate_handler![
            // apariencia y ajustes
            commands::get_appearance_settings,
            commands::get_system_accent_color,
            commands::set_setting,
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
            commands::acknowledge_alert,
            commands::mute_alert,
            commands::unmute_alert,
            commands::archive_alert,
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
            // registro
            commands::log_from_ui,
            commands::get_log_level,
        ])
        .setup(|app| {
            // La ventana nace oculta y se muestra cuando el frontend ha pintado: así no se ve un
            // rectángulo blanco antes de que se aplique el tema.
            platform::ventana::restaurar_ventana_principal(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error al arrancar SmartDisk Monitor");
}
