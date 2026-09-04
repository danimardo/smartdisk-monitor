//! Forma única de los errores que cruzan la frontera hacia la interfaz.
//!
//! Todo comando que falle devuelve un `AppError` completo. La interfaz exige frase humana **y**
//! detalle técnico conservado (`AGENTS.md` §5): `message_key` es la clave i18n que se muestra y
//! `detail` el texto crudo que se guarda dentro de un `<details>` copiable. Nunca se devuelve una
//! cadena suelta, porque entonces la pantalla no sabría qué enseñar ni qué esconder.

use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    /// Identificador estable, apto para ramificar en la UI: "smartctl.timeout", "db.locked"…
    pub code: String,
    /// Clave i18n de la explicación en lenguaje humano.
    pub message_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_vars: Option<BTreeMap<String, String>>,
    /// Texto técnico literal: stderr, código de salida, mensaje de SQLite. Nunca traducido.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Qué fuente falló, cuando aplique: permite degradar una tarjeta y no la aplicación entera.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// `true` si repetir la misma acción tiene sentido (timeout, bloqueo temporal).
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: &str, message_key: &str) -> Self {
        Self {
            code: code.to_owned(),
            message_key: message_key.to_owned(),
            message_vars: None,
            detail: None,
            source: None,
            retryable: false,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    pub fn from_source(mut self, source: &str) -> Self {
        self.source = Some(source.to_owned());
        self
    }

    /// Marcador para lo que todavía no existe. Que el esqueleto lo diga en voz alta es preferible
    /// a devolver datos inventados que luego alguien confunda con reales.
    pub fn not_implemented(what: &str) -> Self {
        Self::new("app.not_implemented", "error.notImplemented").with_detail(what)
    }
}

/// Los comandos devuelven el error **en caja**.
///
/// `AppError` tiene seis campos y es la variante de error de casi todas las funciones del backend:
/// sin la caja, cada `Result<(), AppError>` arrastraría 136 bytes en el camino feliz, que es el que
/// se recorre siempre. Boxear cuesta una asignación solo cuando algo falla, que es justo cuando da
/// igual. Lo señala `clippy::result_large_err`.
///
/// La serialización no cambia: `Box<AppError>` viaja como el objeto, no como una referencia.
pub type AppResult<T> = Result<T, Box<AppError>>;
