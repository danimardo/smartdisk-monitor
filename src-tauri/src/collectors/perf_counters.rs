//! Contadores de rendimiento de `PhysicalDisk` vía PDH (T058, `docs/architecture.md` "Colector
//! Windows"): `% Idle Time` (del que se deriva `activity_percent`, **nunca** `% Disk Time`, que
//! supera el 100 % con varias operaciones simultáneas), `Disk Read/Write Bytes/sec` y
//! `Avg. Disk sec/Read`/`Write`.
//!
//! Enlace FFI directo a `pdh.dll`, sin el crate `windows` completo: mismo criterio que
//! `platform::locale.rs` con `kernel32` — cinco funciones estables de una superficie de Win32 que
//! no ha cambiado en dos décadas no justifican añadir una dependencia entera a un binario
//! privilegiado (`AGENTS.md`, límites duros). Las firmas están verificadas contra los bindings que
//! genera `windows-rs` (Context7, `Win32::System::Performance`), no adivinadas.
//!
//! La instancia del contador se resuelve por número de disco (`\PhysicalDisk(0 *)\...`), no por
//! letra de unidad: la instancia real incluye las letras (`"0 C: D:"`) y Windows no permite
//! abreviarla al añadir el contador. Se usa `PdhAddEnglishCounterW`, no `PdhAddCounterW`: medido
//! contra un Windows real en español, los nombres de objeto y contador de PDH están **localizados**
//! (`PhysicalDisk` es "Disco físico", `% Idle Time` es "% de tiempo inactivo") y una ruta en inglés
//! falla fuera de un Windows en inglés. `PdhAddEnglishCounterW` traduce el nombre y resuelve el
//! comodín de instancia en la misma llamada, sin un paso de expansión aparte.
//!
//! Una tasa (bytes/s, sec/operación) exige dos muestras separadas en el tiempo: la primera
//! recogida tras abrir la consulta no tiene con qué compararse. Esta primera versión abre,
//! recoge dos veces con un segundo de espera entre medias, formatea y cierra — autónomo, sin
//! estado que sobreviva entre ciclos de recopilación, porque el planificador en segundo plano
//! (T020) todavía no existe para mantener una consulta abierta entre ciclos de 30 s
//! (`docs/open-questions.md` J.19 documenta el mismo hueco para la bandeja).

use std::thread;
use std::time::Duration;

/// Lectura de una sola pasada, ya derivada. `None` en un campo es una fuente que no respondió,
/// nunca un cero inventado.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LecturaRendimiento {
    pub activity_percent: Option<f64>,
    pub read_bytes_per_second: Option<f64>,
    pub write_bytes_per_second: Option<f64>,
    pub read_latency_ms: Option<f64>,
    pub write_latency_ms: Option<f64>,
}

/// `% Idle Time` → `activity_percent`: el complemento a 100, acotado — un redondeo de PDH
/// podría llevarlo a 100,4 o a -0,2, y ninguno de los dos es un porcentaje real.
pub fn derivar_activity_percent(idle_time_percent: f64) -> f64 {
    (100.0 - idle_time_percent).clamp(0.0, 100.0)
}

/// `Avg. Disk sec/...` de PDH llega en segundos; el contrato de la aplicación los quiere en ms.
pub fn segundos_a_ms(segundos: f64) -> f64 {
    segundos * 1000.0
}

#[derive(Debug)]
pub enum ErrorPdh {
    /// Código de error de PDH devuelto por la función indicada.
    Codigo { funcion: &'static str, codigo: u32 },
}

#[cfg(windows)]
mod ffi {
    //! Bindings mínimos, solo lo que este módulo usa. `PDH_HQUERY`/`PDH_HCOUNTER` son manejadores
    //! opacos (`isize`) desde siempre en la ABI de Win32; el formato de valor y los códigos de
    //! éxito no han cambiado desde Windows 2000.
    #![allow(non_snake_case, non_camel_case_types)]

    pub type PDH_HQUERY = isize;
    pub type PDH_HCOUNTER = isize;

    pub const PDH_FMT_DOUBLE: u32 = 0x0000_0200;
    pub const ERROR_SUCCESS: u32 = 0;

    #[repr(C)]
    pub union PdhValor {
        pub long_value: i32,
        pub double_value: f64,
        pub large_value: i64,
    }

    #[repr(C)]
    pub struct PdhFmtCounterValue {
        pub c_status: u32,
        pub valor: PdhValor,
    }

    #[link(name = "pdh")]
    extern "system" {
        pub fn PdhOpenQueryW(
            sz_data_source: *const u16,
            dw_user_data: usize,
            ph_query: *mut PDH_HQUERY,
        ) -> u32;
        pub fn PdhAddEnglishCounterW(
            h_query: PDH_HQUERY,
            sz_full_counter_path: *const u16,
            dw_user_data: usize,
            ph_counter: *mut PDH_HCOUNTER,
        ) -> u32;
        pub fn PdhCollectQueryData(h_query: PDH_HQUERY) -> u32;
        pub fn PdhGetFormattedCounterValue(
            h_counter: PDH_HCOUNTER,
            dw_format: u32,
            lpdw_type: *mut u32,
            p_value: *mut PdhFmtCounterValue,
        ) -> u32;
        pub fn PdhCloseQuery(h_query: PDH_HQUERY) -> u32;
    }
}

#[cfg(windows)]
fn a_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Añade un contador a la consulta ya abierta y devuelve su manejador, sin recoger nada
/// todavía: las cinco fuentes se añaden primero y se recogen juntas, para pagar la ventana de
/// muestreo de 1 s **una sola vez** por disco, no cinco. `PdhAddEnglishCounterW` (ver cabecera
/// del módulo): traduce el nombre en inglés y resuelve el comodín de instancia en la misma llamada.
#[cfg(windows)]
fn anadir_contador(
    h_query: ffi::PDH_HQUERY,
    ruta_comodin: &str,
) -> Result<ffi::PDH_HCOUNTER, ErrorPdh> {
    let ruta_ancha = a_wide(ruta_comodin);
    let mut h_counter: ffi::PDH_HCOUNTER = 0;
    let codigo =
        unsafe { ffi::PdhAddEnglishCounterW(h_query, ruta_ancha.as_ptr(), 0, &mut h_counter) };
    if codigo != ffi::ERROR_SUCCESS {
        return Err(ErrorPdh::Codigo {
            funcion: "PdhAddEnglishCounterW",
            codigo,
        });
    }
    Ok(h_counter)
}

#[cfg(windows)]
fn formatear(h_counter: ffi::PDH_HCOUNTER) -> Result<f64, ErrorPdh> {
    let mut valor = ffi::PdhFmtCounterValue {
        c_status: 0,
        valor: ffi::PdhValor { double_value: 0.0 },
    };
    let codigo = unsafe {
        ffi::PdhGetFormattedCounterValue(
            h_counter,
            ffi::PDH_FMT_DOUBLE,
            std::ptr::null_mut(),
            &mut valor,
        )
    };
    if codigo != ffi::ERROR_SUCCESS {
        return Err(ErrorPdh::Codigo {
            funcion: "PdhGetFormattedCounterValue",
            codigo,
        });
    }
    Ok(unsafe { valor.valor.double_value })
}

/// Lee las cinco fuentes de un disco físico en una sola consulta PDH, abierta y cerrada en esta
/// misma llamada (ver nota de cabecera sobre por qué es autónoma en vez de persistente). Una sola
/// ventana de muestreo de 1 s para las cinco, no una por contador.
#[cfg(windows)]
pub fn leer(disk_number: i64) -> Result<LecturaRendimiento, ErrorPdh> {
    let mut h_query: ffi::PDH_HQUERY = 0;
    let codigo = unsafe { ffi::PdhOpenQueryW(std::ptr::null(), 0, &mut h_query) };
    if codigo != ffi::ERROR_SUCCESS {
        return Err(ErrorPdh::Codigo {
            funcion: "PdhOpenQueryW",
            codigo,
        });
    }

    let resultado = (|| {
        let h_idle = anadir_contador(
            h_query,
            &format!(r"\PhysicalDisk({disk_number} *)\% Idle Time"),
        )?;
        let h_lectura_bps = anadir_contador(
            h_query,
            &format!(r"\PhysicalDisk({disk_number} *)\Disk Read Bytes/sec"),
        )?;
        let h_escritura_bps = anadir_contador(
            h_query,
            &format!(r"\PhysicalDisk({disk_number} *)\Disk Write Bytes/sec"),
        )?;
        let h_lectura_seg = anadir_contador(
            h_query,
            &format!(r"\PhysicalDisk({disk_number} *)\Avg. Disk sec/Read"),
        )?;
        let h_escritura_seg = anadir_contador(
            h_query,
            &format!(r"\PhysicalDisk({disk_number} *)\Avg. Disk sec/Write"),
        )?;

        // Dos recogidas: la primera solo abre la ventana de muestreo, la tasa sale de la segunda.
        for _ in 0..2 {
            let codigo = unsafe { ffi::PdhCollectQueryData(h_query) };
            if codigo != ffi::ERROR_SUCCESS {
                return Err(ErrorPdh::Codigo {
                    funcion: "PdhCollectQueryData",
                    codigo,
                });
            }
            thread::sleep(Duration::from_millis(1000));
        }

        Ok(LecturaRendimiento {
            activity_percent: Some(derivar_activity_percent(formatear(h_idle)?)),
            read_bytes_per_second: Some(formatear(h_lectura_bps)?),
            write_bytes_per_second: Some(formatear(h_escritura_bps)?),
            read_latency_ms: Some(segundos_a_ms(formatear(h_lectura_seg)?)),
            write_latency_ms: Some(segundos_a_ms(formatear(h_escritura_seg)?)),
        })
    })();

    // Se cierra siempre, acierte o falle la lectura: un `HQUERY` abierto es un recurso del
    // sistema que no se libera solo.
    unsafe { ffi::PdhCloseQuery(h_query) };
    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cien_por_cien_idle_es_cero_actividad() {
        assert_eq!(derivar_activity_percent(100.0), 0.0);
    }

    #[test]
    fn cero_idle_es_cien_por_cien_actividad() {
        assert_eq!(derivar_activity_percent(0.0), 100.0);
    }

    #[test]
    fn un_redondeo_por_encima_de_cien_no_produce_actividad_negativa() {
        assert_eq!(derivar_activity_percent(100.4), 0.0);
    }

    #[test]
    fn un_redondeo_por_debajo_de_cero_no_produce_mas_de_cien() {
        assert_eq!(derivar_activity_percent(-0.2), 100.0);
    }

    #[test]
    fn segundos_a_ms_convierte_correctamente() {
        assert_eq!(segundos_a_ms(0.5), 500.0);
        assert_eq!(segundos_a_ms(0.0), 0.0);
    }
}
