//! Datos del propio Windows que la aplicación necesita para clasificar sus volúmenes.
//!
//! Aislado igual que `locale.rs`: FFI directo a `kernel32` (mismo criterio que el resto de
//! `platform/` — no se añade el crate `windows` completo por una función estable).

/// Letra de la unidad donde vive Windows (normalmente `C`), en mayúscula. Se usa para marcar el
/// volumen de sistema en el inventario (`VolumeSummary.is_system_volume`) sin que la interfaz tenga
/// que adivinarlo comparando letras a mano (constitución §IV). `None` si no se puede determinar.
#[cfg(windows)]
pub fn letra_unidad_sistema() -> Option<char> {
    // `GetSystemWindowsDirectoryW` devuelve p. ej. `C:\Windows`. La primera letra es la unidad.
    const MAX: usize = 260;
    let mut buf = [0u16; MAX];

    // SAFETY: se pasa un búfer propio con su longitud real; la función solo escribe dentro de él.
    let len = unsafe { GetSystemWindowsDirectoryW(buf.as_mut_ptr(), MAX as u32) };
    if len == 0 || (len as usize) > MAX {
        return None;
    }
    char::from_u32(buf[0] as u32)
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetSystemWindowsDirectoryW(lpBuffer: *mut u16, uSize: u32) -> u32;
}

#[cfg(not(windows))]
pub fn letra_unidad_sistema() -> Option<char> {
    None
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn devuelve_una_letra_de_unidad_plausible() {
        // En cualquier Windows real hay una unidad de sistema; su letra es A–Z.
        let letra = letra_unidad_sistema().expect("Windows siempre tiene unidad de sistema");
        assert!(letra.is_ascii_uppercase());
    }
}
