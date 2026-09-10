//! Validación de rutas y reserva de espacio del benchmark (T077/T078,
//! `docs/product-specification.md` §6, `docs/open-questions.md` J.27).
//!
//! Puro: nada aquí toca el sistema de archivos. Decide **dónde** puede vivir el archivo temporal
//! y **cuánto** puede pesar; crear ese archivo (vía `diskspd.exe -c`) y borrarlo es
//! responsabilidad de `commands::start_benchmark` y `tests::diskspd`, que solo llaman a esto antes
//! de tocar disco.

use std::path::{Path, PathBuf};

/// Nombre visible de la carpeta dedicada, en la raíz del volumen que se está probando —no en
/// `%ProgramData%`: tiene que vivir en el mismo volumen para medir su E/S real (J.27).
pub const CARPETA_BENCHMARK: &str = "SmartDisk Monitor Benchmark";

/// La reserva de seguridad nunca invadida: 2 GiB o el 5 % del volumen, la mayor de las dos
/// (`product-specification.md` §6).
const RESERVA_MINIMA_BYTES: i64 = 2 * 1024 * 1024 * 1024;

/// Límites configurables del tamaño del archivo de prueba (`product-specification.md` §6): entre
/// 256 MiB y 8 GiB.
pub const TAMANO_MIN_BYTES: i64 = 256 * 1024 * 1024;
pub const TAMANO_MAX_BYTES: i64 = 8 * 1024 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorRuta {
    /// La ruta resuelta cae fuera de `CARPETA_BENCHMARK` en el volumen indicado.
    FueraDeLaCarpetaPermitida,
    /// El archivo ya existe: nunca se sobrescribe uno existente.
    ArchivoYaExiste,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorTamano {
    /// Ni siquiera el mínimo configurable (256 MiB) cabe una vez descontada la reserva.
    EspacioInsuficienteTrasLaReserva,
}

/// La carpeta dedicada del benchmark, colgando de la raíz del volumen indicado.
pub fn carpeta_benchmark(raiz_volumen: &Path) -> PathBuf {
    raiz_volumen.join(CARPETA_BENCHMARK)
}

/// Construye la ruta del archivo temporal y confirma que cae dentro de la carpeta permitida.
/// Comprobación **lexical** (`starts_with` sobre componentes ya normalizados), no
/// `canonicalize()`: el archivo aún no existe, así que no hay nada que resolver en el sistema de
/// archivos todavía — resolver un símlink que no existe fallaría.
pub fn resolver_ruta_archivo(
    raiz_volumen: &Path,
    nombre_archivo: &str,
) -> Result<PathBuf, ErrorRuta> {
    let carpeta = carpeta_benchmark(raiz_volumen);
    let candidata = carpeta.join(nombre_archivo);

    // Sin componentes `..`: un nombre de archivo con ellos podría escapar de la carpeta aunque
    // `starts_with` no lo detectase todavía (los componentes no se han resuelto).
    if nombre_archivo.contains("..")
        || nombre_archivo.contains('/')
        || nombre_archivo.contains('\\')
    {
        return Err(ErrorRuta::FueraDeLaCarpetaPermitida);
    }
    if !candidata.starts_with(&carpeta) {
        return Err(ErrorRuta::FueraDeLaCarpetaPermitida);
    }
    Ok(candidata)
}

/// Confirma que la ruta ya construida no apunta a un archivo existente. Se llama justo antes de
/// crear el archivo, no se asume por la aleatoriedad del nombre (J.27: "se comprueba activamente,
/// no se asume").
pub fn confirmar_no_sobrescribe(ruta: &Path) -> Result<(), ErrorRuta> {
    if ruta.exists() {
        return Err(ErrorRuta::ArchivoYaExiste);
    }
    Ok(())
}

/// La reserva que nunca se invade: 2 GiB o el 5 % del volumen, la mayor de las dos.
pub fn calcular_reserva_bytes(capacidad_bytes: i64) -> i64 {
    let cinco_por_ciento = capacidad_bytes / 20;
    RESERVA_MINIMA_BYTES.max(cinco_por_ciento)
}

/// El tamaño real a usar para el archivo de prueba: el pedido por el usuario, acotado por los
/// límites configurables (256 MiB–8 GiB) y por el espacio libre tras descontar la reserva. Un
/// tamaño solicitado por debajo de `TAMANO_MIN_BYTES` se sube al mínimo, no se rechaza —el límite
/// que sí puede rechazar la prueba entera es la falta de espacio, no una elección pequeña del
/// usuario.
pub fn resolver_tamano_bytes(
    solicitado_bytes: i64,
    libre_bytes: i64,
    capacidad_bytes: i64,
) -> Result<i64, ErrorTamano> {
    let reserva = calcular_reserva_bytes(capacidad_bytes);
    let utilizable = (libre_bytes - reserva).max(0);

    if utilizable < TAMANO_MIN_BYTES {
        return Err(ErrorTamano::EspacioInsuficienteTrasLaReserva);
    }

    let acotado_a_limites = solicitado_bytes.clamp(TAMANO_MIN_BYTES, TAMANO_MAX_BYTES);
    Ok(acotado_a_limites.min(utilizable))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: i64 = 1024 * 1024 * 1024;

    #[test]
    fn la_carpeta_cuelga_de_la_raiz_del_volumen() {
        let carpeta = carpeta_benchmark(Path::new(r"D:\"));
        assert_eq!(carpeta, PathBuf::from(r"D:\SmartDisk Monitor Benchmark"));
    }

    #[test]
    fn una_ruta_normal_dentro_de_la_carpeta_se_resuelve() {
        let ruta = resolver_ruta_archivo(Path::new(r"D:\"), "benchmark-abc123.tmp").unwrap();
        assert_eq!(
            ruta,
            PathBuf::from(r"D:\SmartDisk Monitor Benchmark\benchmark-abc123.tmp")
        );
    }

    #[test]
    fn un_nombre_con_doble_punto_se_rechaza() {
        let err = resolver_ruta_archivo(Path::new(r"D:\"), "..\\..\\windows\\system32\\evil.tmp")
            .unwrap_err();
        assert_eq!(err, ErrorRuta::FueraDeLaCarpetaPermitida);
    }

    #[test]
    fn un_nombre_con_separador_de_ruta_se_rechaza() {
        let err = resolver_ruta_archivo(Path::new(r"D:\"), "sub/archivo.tmp").unwrap_err();
        assert_eq!(err, ErrorRuta::FueraDeLaCarpetaPermitida);
    }

    #[test]
    fn confirmar_no_sobrescribe_acepta_una_ruta_que_no_existe() {
        let ruta = resolver_ruta_archivo(Path::new(r"D:\"), "benchmark-inexistente.tmp").unwrap();
        assert!(confirmar_no_sobrescribe(&ruta).is_ok());
    }

    #[test]
    fn confirmar_no_sobrescribe_rechaza_un_archivo_real() {
        let temporal = std::env::temp_dir().join("sdm_prueba_no_sobrescribe.tmp");
        std::fs::write(&temporal, b"x").unwrap();
        let err = confirmar_no_sobrescribe(&temporal).unwrap_err();
        assert_eq!(err, ErrorRuta::ArchivoYaExiste);
        std::fs::remove_file(&temporal).unwrap();
    }

    #[test]
    fn la_reserva_es_dos_gib_en_un_volumen_pequeno() {
        // 10 GiB * 5% = 0,5 GiB, menor que el mínimo de 2 GiB: gana el mínimo.
        assert_eq!(calcular_reserva_bytes(10 * GIB), 2 * GIB);
    }

    #[test]
    fn la_reserva_es_el_cinco_por_ciento_en_un_volumen_grande() {
        // 100 GiB * 5% = 5 GiB, mayor que el mínimo de 2 GiB: gana el porcentaje.
        assert_eq!(calcular_reserva_bytes(100 * GIB), 5 * GIB);
    }

    #[test]
    fn el_tamano_solicitado_dentro_de_limites_y_con_espacio_se_respeta() {
        let resultado = resolver_tamano_bytes(GIB, 50 * GIB, 100 * GIB).unwrap();
        assert_eq!(resultado, GIB);
    }

    #[test]
    fn un_tamano_por_encima_del_maximo_configurable_se_recorta() {
        let resultado = resolver_tamano_bytes(20 * GIB, 50 * GIB, 100 * GIB).unwrap();
        assert_eq!(resultado, TAMANO_MAX_BYTES);
    }

    #[test]
    fn un_tamano_por_debajo_del_minimo_configurable_se_sube_al_minimo() {
        let resultado = resolver_tamano_bytes(1024, 50 * GIB, 100 * GIB).unwrap();
        assert_eq!(resultado, TAMANO_MIN_BYTES);
    }

    #[test]
    fn el_tamano_nunca_invade_la_reserva() {
        // 100 GiB de capacidad -> reserva de 5 GiB. Solo 4 GiB libres: menos que la reserva,
        // utilizable = 0, no cabe ni el mínimo.
        let err = resolver_tamano_bytes(GIB, 4 * GIB, 100 * GIB).unwrap_err();
        assert_eq!(err, ErrorTamano::EspacioInsuficienteTrasLaReserva);
    }

    #[test]
    fn con_poco_libre_el_tamano_se_recorta_a_lo_que_queda_tras_la_reserva() {
        // 10 GiB de capacidad -> reserva de 2 GiB. 3 GiB libres -> 1 GiB utilizable.
        let resultado = resolver_tamano_bytes(GIB, 3 * GIB, 10 * GIB).unwrap();
        assert_eq!(resultado, GIB);
    }

    #[test]
    fn justo_en_el_umbral_del_minimo_tras_la_reserva_se_acepta() {
        // 10 GiB de capacidad -> reserva de 2 GiB. Libre = reserva + mínimo exacto.
        let libre = 2 * GIB + TAMANO_MIN_BYTES;
        let resultado = resolver_tamano_bytes(TAMANO_MIN_BYTES, libre, 10 * GIB).unwrap();
        assert_eq!(resultado, TAMANO_MIN_BYTES);
    }

    #[test]
    fn justo_un_byte_por_debajo_del_umbral_del_minimo_se_rechaza() {
        let libre = 2 * GIB + TAMANO_MIN_BYTES - 1;
        let err = resolver_tamano_bytes(TAMANO_MIN_BYTES, libre, 10 * GIB).unwrap_err();
        assert_eq!(err, ErrorTamano::EspacioInsuficienteTrasLaReserva);
    }
}
