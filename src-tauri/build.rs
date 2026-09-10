// Transformación XOR compartida con `platform::ia_clave_demo` (ADR-054). Se incluye en vez de
// importarse porque el script de compilación se compila como su propio crate.
include!("src/platform/clave_demo_ofuscacion.rs");

fn main() {
    // Manifiesto propio: la aplicación se ejecuta elevada (ADR-004) y declara consciencia de DPI
    // por monitor. Sin `app_manifest`, Tauri genera uno por defecto con `asInvoker`.
    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows/app.manifest"));

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("no se pudo generar la configuración de compilación de Tauri");

    emitir_clave_demo();
}

/// Clave de demostración compartida para la ayuda con IA (ADR-054, principio XVI).
///
/// Si la Release se construye con `SDM_OPENROUTER_DEMO_KEY` en el entorno (el flujo de
/// `.github/workflows/release.yml` la toma de un secreto; en local: `set SDM_OPENROUTER_DEMO_KEY=…
/// && pnpm app:build`), se compila **ofuscada** en el binario. Sin la variable —un clon del
/// repositorio— el binario no la trae y la ayuda con IA sigue exigiendo una clave propia.
///
/// La clave **nunca** se escribe en un fichero del repositorio: solo pasa por esta variable de
/// entorno.
fn emitir_clave_demo() {
    println!("cargo:rerun-if-env-changed=SDM_OPENROUTER_DEMO_KEY");
    let clave = std::env::var("SDM_OPENROUTER_DEMO_KEY").unwrap_or_default();
    let clave = clave.trim();
    if clave.is_empty() {
        return;
    }
    println!(
        "cargo:rustc-env=SDM_OPENROUTER_DEMO_KEY_OFUSCADA={}",
        ofuscar_xor(clave.as_bytes())
    );
}
