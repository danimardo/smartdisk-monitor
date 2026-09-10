// Ofuscación XOR de la clave de demostración compartida de la ayuda con IA (ADR-054).
//
// Este fichero lo **incluye** (`include!`) tanto `build.rs` —para velar la variable de entorno
// `SDM_OPENROUTER_DEMO_KEY` al compilar la Release— como `platform::ia_clave_demo` —para
// des-velarla en ejecución—. Es un `include!` y no un módulo normal para que `build.rs`, que se
// compila aparte, comparta exactamente la misma transformación sin duplicarla.
//
// **No es cifrado** (ADR-054): la clave de demostración se declara *no secreta*. La ofuscación solo
// evita que un `strings binario.exe | grep sk-or` la encuentre a simple vista. El patrón está aquí,
// a la vista de cualquiera que lea el repositorio.

/// Patrón XOR. Corto y fijo: es un velo, no una clave.
#[allow(dead_code)]
const PATRON_OFUSCACION: &[u8] = b"SmartDisk Monitor/OpenRouter/clave-demo";

/// XOR byte a byte contra el patrón (cíclico); el resultado se codifica en hexadecimal en
/// minúsculas para poder viajar como valor de `cargo:rustc-env`.
#[allow(dead_code)]
fn ofuscar_xor(claro: &[u8]) -> String {
    claro
        .iter()
        .zip(PATRON_OFUSCACION.iter().cycle())
        .map(|(b, k)| format!("{:02x}", b ^ k))
        .collect()
}

/// Inversa de [`ofuscar_xor`]. `None` si el hexadecimal está mal formado o si el resultado no es
/// UTF-8 válido (binario compilado con otra versión del patrón, dato corrupto…).
#[allow(dead_code)]
fn desofuscar_xor(hex: &str) -> Option<String> {
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    let claro: Vec<u8> = bytes?
        .iter()
        .zip(PATRON_OFUSCACION.iter().cycle())
        .map(|(b, k)| b ^ k)
        .collect();
    String::from_utf8(claro).ok()
}
