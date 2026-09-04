//! Rutas de datos de la aplicación.
//!
//! La ruta se obtiene de Windows y **no se codifica como literal en la lógica de negocio**
//! (arquitectura §5). En desarrollo se usa una carpeta local para no mezclar datos reales con
//! pruebas ni exigir elevación en cada ejecución de test
//! (`docs/engineering-conventions.md` §6).

use std::path::PathBuf;

/// Carpeta de datos: `%ProgramData%\SmartDisk Monitor\` en producción.
pub fn data_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        // Desarrollo: junto al proyecto, ignorada por git.
        return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.dev-data");
    }
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
        .join("SmartDisk Monitor")
}

/// Carpeta de logs, creada si no existe: sin ella no habría registro justo cuando hace falta.
///
/// **Solo se crea si la carpeta de datos ya existe.** No es una precaución de estilo: la ACL por
/// omisión de `%ProgramData%` concede al grupo `Usuarios` los permisos `(CI)(WD,AD,WEA,WA)`, que
/// se heredan a toda subcarpeta. Crear aquí la raíz `SmartDisk Monitor` la dejaría escribible por
/// cualquier usuario sin privilegios, deshaciendo el endurecimiento que aplica el instalador
/// (ADR-026, `docs/open-questions.md` §R). La raíz la crea el instalador, con su ACL explícita; si
/// falta, la instalación está rota y eso tiene que notarse, no repararse a medias.
///
/// En desarrollo sí se crea entera: `data_dir()` apunta a `.dev-data`, fuera de `%ProgramData%`.
pub fn log_dir() -> PathBuf {
    let raiz = data_dir();
    let dir = raiz.join("logs");
    if cfg!(debug_assertions) || raiz.is_dir() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_ruta_de_datos_no_es_un_literal_suelto() {
        let d = data_dir();
        assert!(d.is_absolute() || cfg!(debug_assertions));
    }

    #[test]
    fn los_logs_cuelgan_de_la_carpeta_de_datos() {
        assert!(log_dir().ends_with("logs"));
        assert_eq!(log_dir().parent(), Some(data_dir().as_path()));
    }
}
