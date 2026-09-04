fn main() {
    // Manifiesto propio: la aplicación se ejecuta elevada (ADR-004) y declara consciencia de DPI
    // por monitor. Sin `app_manifest`, Tauri genera uno por defecto con `asInvoker`.
    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows/app.manifest"));

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("no se pudo generar la configuración de compilación de Tauri");
}
