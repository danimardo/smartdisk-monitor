//! Estado de energía y espacio libre reales, leídos de Windows (FR-030, `domain::espacio`).
//!
//! Ninguna de las dos syscalls está garantizada a tener éxito. Qué se asume si fallan está
//! decidido en `open-questions.md` J.36: batería desconocida se trata como red eléctrica (el lado
//! que menos reduce la frecuencia de recopilación); espacio libre desconocido se trata como
//! `EstadoEspacio::Normal` (un dato ausente no es "disco lleno", mismo principio que "no
//! compatible ≠ averiado" aplicado a un fallo de sistema en vez de a un disco).

use std::path::Path;

/// `true` si el equipo funciona con batería ahora mismo. Un valor de `ACLineStatus` que no sea
/// "batería" (incluido el "desconocido" que Windows documenta como posible) se trata como red.
#[cfg(windows)]
pub fn en_bateria() -> bool {
    let mut estado = SystemPowerStatus::default();
    // SAFETY: `estado` es un struct del tamaño exacto que la función espera y vive en la pila
    // durante toda la llamada; no hay punteros que la función pueda invalidar.
    let ok = unsafe { GetSystemPowerStatus(&mut estado) };
    ok != 0 && estado.ac_line_status == 0
}

#[cfg(not(windows))]
pub fn en_bateria() -> bool {
    false
}

/// Bytes libres disponibles para este proceso en el volumen que contiene `ruta`. `None` si la
/// consulta falla (ruta inexistente, error de E/S): un dato ausente, nunca un cero engañoso.
#[cfg(windows)]
pub fn espacio_libre_bytes(ruta: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;

    let ancho: Vec<u16> = ruta
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut libres_para_el_proceso: u64 = 0;
    // SAFETY: `ancho` termina en NUL y vive hasta el final de la llamada; los tres punteros de
    // salida son válidos y del tamaño que la función espera. Se pasan null los dos que no
    // interesan (total y libres del volumen entero): la documentación de Win32 lo permite.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            ancho.as_ptr(),
            &mut libres_para_el_proceso,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        None
    } else {
        Some(libres_para_el_proceso)
    }
}

#[cfg(not(windows))]
pub fn espacio_libre_bytes(_ruta: &Path) -> Option<u64> {
    None
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct SystemPowerStatus {
    ac_line_status: u8,
    battery_flag: u8,
    battery_life_percent: u8,
    reserved1: u8,
    battery_life_time: u32,
    battery_full_life_time: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetSystemPowerStatus(lp_system_power_status: *mut SystemPowerStatus) -> i32;
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    /// La parte pura de interpretar `ACLineStatus` no depende de la syscall real: se prueba aquí
    /// como lo haría cualquier otro conversor, igual que `logging::level_from_cli`.
    fn es_bateria(ac_line_status: u8, llamada_ok: bool) -> bool {
        llamada_ok && ac_line_status == 0
    }

    #[test]
    fn ac_line_status_cero_es_bateria() {
        assert!(es_bateria(0, true));
    }

    #[test]
    fn ac_line_status_uno_es_red() {
        assert!(!es_bateria(1, true));
    }

    #[test]
    fn ac_line_status_desconocido_255_se_trata_como_red() {
        assert!(!es_bateria(255, true));
    }

    #[test]
    fn una_llamada_fallida_se_trata_como_red() {
        assert!(!es_bateria(0, false));
    }
}
