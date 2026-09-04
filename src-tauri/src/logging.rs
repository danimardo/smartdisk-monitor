//! Registro de actividad (constitución §XV).
//!
//! Un log sirve para diagnosticar un fallo que ya ocurrió y entender qué estaba haciendo la
//! aplicación cuando ocurrió. Todo lo demás es ruido, y el ruido no es neutral: entierra lo que
//! importa y llena el disco del usuario, en un producto cuyo trabajo es vigilar ese disco.
//!
//! **Hora local con desplazamiento explícito**, no UTC ni una zona fija. Esta aplicación
//! correlaciona sus métricas con el Visor de eventos de Windows, que muestra hora local: un log en
//! otra zona obligaría a convertir mentalmente cada vez que se coteja un pico de temperatura con un
//! evento de disco. El desplazamiento evita la ambigüedad cuando el fichero viaja por correo.

use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use time::format_description::FormatItem;
use time::macros::format_description;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::EnvFilter;

/// Formato legible con un editor de texto, que es como se abrirá el 90 % de las veces.
const TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second],[subsecond digits:3] [offset_hour sign:mandatory]:[offset_minute]");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Silent,
}

impl LogLevel {
    fn as_filter(self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Silent => "off",
        }
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            "silent" | "off" | "none" => Ok(LogLevel::Silent),
            _ => Err(()),
        }
    }
}

/// Nivel pedido en la línea de órdenes, si lo hay.
///
/// Es la **única** forma de diagnosticar un fallo que ocurre antes de poder leer `settings`, que es
/// justo cuando más falta hace. Un valor inválido **no** arranca en modo detallado «por si acaso»:
/// se avisa y se usa el predeterminado, porque un log a máximo detalle activado por una errata
/// llenaría el disco sin que nadie lo pidiera.
pub fn level_from_cli<S: AsRef<str>>(args: &[S]) -> Option<LogLevel> {
    let value = args
        .iter()
        .find_map(|a| a.as_ref().strip_prefix("--log-level=").map(str::to_owned))?;

    match LogLevel::from_str(&value) {
        Ok(level) => Some(level),
        Err(()) => {
            eprintln!(
                "Nivel de registro no reconocido: {value:?}. Valores válidos:                  trace, debug, info, warn, error, silent. Se usa el predeterminado."
            );
            None
        }
    }
}

/// Arranca el registro. Devuelve el guardia del escritor de fichero, que **debe mantenerse vivo**
/// durante toda la ejecución: al soltarlo se vacían los buffers pendientes.
///
/// Precedencia del nivel (constitución §XV): `--log-level` > `settings` > `info`. Aquí se recibe ya
/// resuelto, porque `settings` puede no ser legible todavía cuando esto se llama.
pub fn init(level: LogLevel, log_dir: &Path) -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(log_dir, "smartdisk.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(level.as_filter()))
        .with_writer(writer)
        .with_ansi(false)
        .with_target(true);

    // Si el proceso no puede determinar su zona local se cae a UTC en vez de quedarse sin registro,
    // y se dice en la primera línea para que nadie interprete mal las horas después.
    match time::UtcOffset::current_local_offset() {
        Ok(offset) => {
            subscriber
                .with_timer(OffsetTime::new(offset, TIME_FORMAT))
                .init();
        }
        Err(_) => {
            subscriber.init();
            tracing::warn!("no se pudo determinar la zona horaria local; las horas van en UTC");
        }
    }

    tracing::info!(nivel = level.as_filter(), "registro iniciado");
    guard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconoce_los_niveles_validos() {
        assert_eq!(LogLevel::from_str("debug"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("ERROR"), Ok(LogLevel::Error));
        assert_eq!(LogLevel::from_str("  warn  "), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("off"), Ok(LogLevel::Silent));
    }

    #[test]
    fn un_nivel_invalido_no_activa_el_modo_detallado() {
        // Lo importante no es que falle, sino que NO devuelva Trace: una errata en la línea de
        // órdenes no puede acabar llenando el disco del usuario.
        assert_eq!(LogLevel::from_str("verboso"), Err(()));
        assert_eq!(level_from_cli(&["--log-level=verboso".to_string()]), None);
    }

    #[test]
    fn lee_el_nivel_de_la_linea_de_ordenes() {
        let args = vec!["smartdisk.exe".to_string(), "--log-level=debug".to_string()];
        assert_eq!(level_from_cli(&args), Some(LogLevel::Debug));
    }

    #[test]
    fn sin_argumento_no_hay_nivel_forzado() {
        let args = vec!["smartdisk.exe".to_string()];
        assert_eq!(level_from_cli(&args), None);
    }

    #[test]
    fn la_hora_se_escribe_con_desplazamiento_y_es_legible() {
        // Se formatea un instante conocido en una zona conocida y se comprueba la cadena real:
        // el desplazamiento explícito evita la ambigüedad cuando el log viaja por correo.
        let momento = time::OffsetDateTime::from_unix_timestamp(1_788_525_127)
            .unwrap()
            .to_offset(time::UtcOffset::from_hms(2, 0, 0).unwrap());

        let salida = momento
            .format(&TIME_FORMAT)
            .expect("el formato debe aplicarse");

        assert!(
            salida.ends_with("+02:00"),
            "falta el desplazamiento: {salida}"
        );
        assert!(
            salida.contains(','),
            "los milisegundos van tras coma: {salida}"
        );
        // Legible con un editor de texto: fecha y hora separadas por espacio, no por "T".
        assert!(
            !salida.contains('T'),
            "no debe usar el separador ISO: {salida}"
        );
    }
}
