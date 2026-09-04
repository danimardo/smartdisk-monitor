//! Comandos Tauri: la superficie completa que la interfaz puede invocar.
//!
//! Cada comando es una función fina que valida sus argumentos y delega en `domain/`. El grueso del
//! esqueleto devuelve `not_implemented` a propósito: es preferible que la aplicación diga en voz
//! alta lo que falta a que devuelva datos inventados que alguien confunda con reales.
//!
//! El contrato normativo está en `docs/ui-contract.md`. La lista de comandos que se registran en
//! `lib.rs` es la lista cerrada: la interfaz no puede invocar nada que no esté aquí.

use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::platform::accent::{self, WindowsAccent};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,
    pub language: Option<String>,
    /// BCP-47 de Windows. La interfaz usa este, no `navigator.language`: el formato de números
    /// debe seguir al idioma de la aplicación (`open-questions.md` A.6).
    pub system_locale: String,
    pub use_system_accent: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceListResponse {
    pub devices: Vec<serde_json::Value>,
    pub excluded: Vec<serde_json::Value>,
    pub sources: Vec<serde_json::Value>,
    pub paused: bool,
    pub paused_since: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub author: String,
}

/// Apariencia persistida. Mientras no exista la tabla `settings`, devuelve los valores de arranque
/// que la especificación define: seguir al sistema en tema e idioma.
#[tauri::command]
pub fn get_appearance_settings() -> AppResult<AppearanceSettings> {
    Ok(AppearanceSettings {
        theme: "system".into(),
        language: None,
        system_locale: crate::platform::locale::system_locale(),
        use_system_accent: true,
    })
}

#[tauri::command]
pub fn get_system_accent_color() -> AppResult<WindowsAccent> {
    accent::read()
}

#[tauri::command]
pub fn set_setting(key: String, _value: serde_json::Value) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "set_setting({key})"
    ))))
}

/// Inventario. Devuelve una lista vacía y no un error: no tener discos todavía no es un fallo, y la
/// interfaz ya distingue "vacío" de "error de fuente" con estados distintos.
#[tauri::command]
pub fn get_devices() -> AppResult<DeviceListResponse> {
    Ok(DeviceListResponse {
        devices: vec![],
        excluded: vec![],
        sources: vec![],
        paused: false,
        paused_since: None,
    })
}

#[tauri::command]
pub fn get_device_detail(device_id: String) -> AppResult<serde_json::Value> {
    Err(Box::new(AppError::not_implemented(&format!(
        "get_device_detail({device_id})"
    ))))
}

#[tauri::command]
pub fn set_device_monitoring(device_id: String, _enabled: bool) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "set_device_monitoring({device_id})"
    ))))
}

#[tauri::command]
pub fn set_device_alias(device_id: String, _alias: Option<String>) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "set_device_alias({device_id})"
    ))))
}

#[tauri::command]
pub fn refresh_now(scope: String, _device_id: Option<String>) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "refresh_now({scope})"
    ))))
}

#[tauri::command]
pub fn get_metric_series(metric_key: String) -> AppResult<serde_json::Value> {
    Err(Box::new(AppError::not_implemented(&format!(
        "get_metric_series({metric_key})"
    ))))
}

#[tauri::command]
pub fn get_alert_groups() -> AppResult<Vec<serde_json::Value>> {
    Ok(vec![])
}

#[tauri::command]
pub fn get_alert_detail(alert_group_id: String) -> AppResult<serde_json::Value> {
    Err(Box::new(AppError::not_implemented(&format!(
        "get_alert_detail({alert_group_id})"
    ))))
}

#[tauri::command]
pub fn acknowledge_alert(alert_group_id: String) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "acknowledge_alert({alert_group_id})"
    ))))
}

#[tauri::command]
pub fn mute_alert(alert_group_id: String, _minutes: Option<u32>) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "mute_alert({alert_group_id})"
    ))))
}

#[tauri::command]
pub fn unmute_alert(alert_group_id: String) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "unmute_alert({alert_group_id})"
    ))))
}

#[tauri::command]
pub fn archive_alert(alert_group_id: String) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "archive_alert({alert_group_id})"
    ))))
}

#[tauri::command]
pub fn get_system_events() -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({ "events": [], "nextCursor": null, "total": 0 }))
}

#[tauri::command]
pub fn get_event_raw_xml(event_id: String) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented(&format!(
        "get_event_raw_xml({event_id})"
    ))))
}

#[tauri::command]
pub fn start_benchmark(volume_id: String) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented(&format!(
        "start_benchmark({volume_id})"
    ))))
}

#[tauri::command]
pub fn run_chkdsk_scan(volume_id: String) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented(&format!(
        "run_chkdsk_scan({volume_id})"
    ))))
}

#[tauri::command]
pub fn run_smart_short_test(device_id: String) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented(&format!(
        "run_smart_short_test({device_id})"
    ))))
}

#[tauri::command]
pub fn cancel_test(test_run_id: String) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented(&format!(
        "cancel_test({test_run_id})"
    ))))
}

#[tauri::command]
pub fn get_test_runs() -> AppResult<Vec<serde_json::Value>> {
    Ok(vec![])
}

#[tauri::command]
pub fn export_report(format: String) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented(&format!(
        "export_report({format})"
    ))))
}

#[tauri::command]
pub fn preview_diagnostic_zip(_include_identifiers: bool) -> AppResult<serde_json::Value> {
    Err(Box::new(AppError::not_implemented(
        "preview_diagnostic_zip",
    )))
}

#[tauri::command]
pub fn create_diagnostic_zip(
    _include_identifiers: bool,
    _destination_path: String,
) -> AppResult<String> {
    Err(Box::new(AppError::not_implemented("create_diagnostic_zip")))
}

#[tauri::command]
pub fn pause_monitoring() -> AppResult<()> {
    Err(Box::new(AppError::not_implemented("pause_monitoring")))
}

#[tauri::command]
pub fn resume_monitoring() -> AppResult<()> {
    Err(Box::new(AppError::not_implemented("resume_monitoring")))
}

/// Nombre y versión salen del manifiesto de compilación, nunca de un literal duplicado (ADR-011).
#[tauri::command]
pub fn get_app_info() -> AppResult<AppInfo> {
    Ok(AppInfo {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        author: env!("CARGO_PKG_AUTHORS").into(),
    })
}

#[tauri::command]
pub fn delete_all_data(_confirmation_phrase: String) -> AppResult<()> {
    Err(Box::new(AppError::not_implemented("delete_all_data")))
}

/* ------------------------------------------------------------------ registro */

/// Recibe del frontend las entradas que deben quedar en el fichero.
///
/// Solo llegan `warn` y `error` (constitución §XV): `debug` e `info` se quedan en la consola del
/// WebView, porque enviarlos por IPC costaría más que el valor que aportan. Así un fallo de
/// interfaz en el equipo de un usuario aparece en el ZIP de diagnóstico sin pedirle que abra las
/// herramientas de desarrollo.
#[tauri::command]
pub fn log_from_ui(
    level: String,
    scope: String,
    message: String,
    context: Option<serde_json::Value>,
) -> AppResult<()> {
    let ctx = context
        .as_ref()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "-".to_owned());

    match level.as_str() {
        "error" => tracing::error!(target: "ui", scope, contexto = %ctx, "{message}"),
        "warn" => tracing::warn!(target: "ui", scope, contexto = %ctx, "{message}"),
        // Un nivel inesperado se registra igual, pero degradado: perder la entrada sería peor.
        other => {
            tracing::info!(target: "ui", scope, nivel_recibido = other, contexto = %ctx, "{message}")
        }
    }
    Ok(())
}

/// Nivel efectivo, para que el frontend filtre igual que el backend y no envíe lo que se
/// descartaría de todos modos.
#[tauri::command]
pub fn get_log_level() -> AppResult<String> {
    // De momento solo manda la línea de órdenes; cuando exista `settings`, esta función devolverá
    // el valor resuelto con la precedencia completa (§XV).
    let args: Vec<String> = std::env::args().collect();
    let level = crate::logging::level_from_cli(&args).unwrap_or(crate::logging::LogLevel::Info);
    Ok(serde_json::to_value(level)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "info".to_owned()))
}
