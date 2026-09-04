//! Idioma del sistema, leído de Windows.
//!
//! La interfaz **no** usa `navigator.language`: el WebView puede no reflejar la configuración
//! regional real del usuario, y el formato de números y fechas debe seguir al idioma elegido en la
//! aplicación conservando la variante regional del sistema cuando comparten idioma
//! (`open-questions.md` A.6). Por eso el locale viaja en `get_appearance_settings`.

/// Etiqueta BCP-47 del idioma de la interfaz de Windows, por ejemplo `es-ES`.
#[cfg(windows)]
pub fn system_locale() -> String {
    use std::os::windows::ffi::OsStringExt;

    // `GetUserDefaultLocaleName` escribe como mucho LOCALE_NAME_MAX_LENGTH (85) UTF-16.
    const MAX: usize = 85;
    let mut buf = [0u16; MAX];

    // SAFETY: se pasa un búfer propio con su longitud real; la función solo escribe dentro de él.
    let len = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), MAX as i32) };
    if len <= 1 {
        return "en-US".into();
    }

    // El valor devuelto incluye el terminador nulo.
    let s = std::ffi::OsString::from_wide(&buf[..(len as usize - 1)]);
    s.into_string().unwrap_or_else(|_| "en-US".into())
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
}

#[cfg(not(windows))]
pub fn system_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|v| v.split('.').next().map(|s| s.replace('_', "-")))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "en-US".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devuelve_una_etiqueta_bcp47_plausible() {
        let l = system_locale();
        assert!(l.len() >= 2, "locale vacío: {l:?}");
        assert!(
            l.chars().next().is_some_and(|c| c.is_ascii_alphabetic()),
            "no parece una etiqueta de idioma: {l:?}"
        );
    }
}
