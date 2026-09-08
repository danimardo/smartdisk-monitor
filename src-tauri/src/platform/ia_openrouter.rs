//! Transporte HTTP hacia OpenRouter (spec `005-explicacion-ia`, ADR-046, principio XVI).
//!
//! **Único destino de red saliente de toda la aplicación.** El `reqwest::Client` se construye
//! aquí, dentro de la llamada: no existe cliente en el arranque (`lib.rs`) ni en `estado_ia`, y
//! `explicar_detalle_tecnico` / `listar_modelos_ia` comprueban la credencial **antes** de llamar
//! aquí (FR-005, SC-001).
//!
//! La interpretación de lo que se recibe (respuesta, errores, catálogo de modelos) vive en
//! `crate::domain::ia`, que es puro y se prueba con fixtures. Este módulo solo transporta.

use std::time::Duration;

use serde::Serialize;

use crate::domain::ia::{ErrorTransporte, RespuestaChat, RespuestaModelos};

/// Base de la API. Constante: no hay parámetro de host (principio XVI, «un solo destino»).
pub const ENDPOINT: &str = "https://openrouter.ai/api/v1";

/// Tiempo máximo de espera de una respuesta (FR-019).
pub const TIMEOUT_SEGUNDOS: u64 = 60;

/// Tiempo máximo para establecer la conexión: distingue «sin red» rápido de «modelo lento».
pub const CONNECT_TIMEOUT_SEGUNDOS: u64 = 10;

/// Recorte defensivo del detalle técnico antes de enviarlo (FR-021). Valor propuesto
/// (`docs/open-questions.md`).
pub const MAX_DETALLE_CHARS: usize = 8_000;

/// Identifica la aplicación ante OpenRouter (recomendación del proveedor para su panel). No lleva
/// ningún dato del equipo ni de la persona.
const APP_URL: &str = "https://github.com/danidiez/smartdisk-monitor";
const APP_TITLE: &str = "SmartDisk Monitor";

/// Cuerpo de `POST /chat/completions`. Struct tipado, no `serde_json::Value` (principio XI).
#[derive(Serialize)]
struct PeticionChat<'a> {
    model: &'a str,
    messages: [MensajePeticion<'a>; 2],
    /// Baja para que la explicación se ciña a lo que se le da y no divague.
    temperature: f32,
}

#[derive(Serialize)]
struct MensajePeticion<'a> {
    role: &'a str,
    content: &'a str,
}

fn cliente() -> Result<reqwest::Client, ErrorTransporte> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SEGUNDOS))
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SEGUNDOS))
        .build()
        .map_err(|e| ErrorTransporte::Conexion(e.to_string()))
}

/// Traduce un fallo de `reqwest` a `ErrorTransporte`. El texto es el `to_string()` del error, que
/// no incluye ni el cuerpo de la petición ni el de la respuesta.
fn clasificar(e: reqwest::Error) -> ErrorTransporte {
    if e.is_timeout() {
        ErrorTransporte::Timeout
    } else if e.is_connect() || e.is_request() {
        ErrorTransporte::Conexion(e.to_string())
    } else if e.is_decode() || e.is_body() {
        ErrorTransporte::Deserializacion(e.to_string())
    } else {
        ErrorTransporte::Conexion(e.to_string())
    }
}

/// Pide una explicación. `clave` es la de API; `system` y `user` ya vienen anonimizados y
/// compuestos por `crate::domain::ia`.
pub async fn chat_completions(
    clave: &str,
    modelo: &str,
    system: &str,
    user: &str,
) -> Result<RespuestaChat, ErrorTransporte> {
    let peticion = PeticionChat {
        model: modelo,
        messages: [
            MensajePeticion {
                role: "system",
                content: system,
            },
            MensajePeticion {
                role: "user",
                content: user,
            },
        ],
        temperature: 0.2,
    };

    let respuesta = cliente()?
        .post(format!("{ENDPOINT}/chat/completions"))
        .bearer_auth(clave)
        .header("HTTP-Referer", APP_URL)
        .header("X-Title", APP_TITLE)
        .json(&peticion)
        .send()
        .await
        .map_err(clasificar)?;

    interpretar(respuesta).await
}

/// Catálogo de modelos para el selector. No requiere `Authorization`, pero solo se invoca desde el
/// selector que la persona ha abierto (gesto explícito, principio XVI).
pub async fn listar_modelos() -> Result<RespuestaModelos, ErrorTransporte> {
    let respuesta = cliente()?
        .get(format!("{ENDPOINT}/models"))
        .header("HTTP-Referer", APP_URL)
        .header("X-Title", APP_TITLE)
        .send()
        .await
        .map_err(clasificar)?;

    interpretar(respuesta).await
}

/// Comprueba que una clave sirve, con una llamada mínima a `GET /key`. `Ok` si la respuesta es
/// 2xx **o** 429/402 (clave válida pero el proveedor está limitando ahora mismo, FR-018). `Err`
/// con el código en cualquier otro caso, para que `domain::ia::analizar_error` lo clasifique.
pub async fn validar_clave(clave: &str) -> Result<(), ErrorTransporte> {
    let respuesta = cliente()?
        .get(format!("{ENDPOINT}/key"))
        .bearer_auth(clave)
        .header("HTTP-Referer", APP_URL)
        .header("X-Title", APP_TITLE)
        .send()
        .await
        .map_err(clasificar)?;

    let estado = respuesta.status();
    if estado.is_success() || matches!(estado.as_u16(), 402 | 429) {
        return Ok(());
    }
    let cuerpo = respuesta.text().await.unwrap_or_default();
    Err(ErrorTransporte::Http {
        estado: estado.as_u16(),
        cuerpo,
    })
}

/// Común a las dos llamadas: un HTTP no-2xx es `ErrorTransporte::Http` con el cuerpo crudo (para el
/// `detail`); un 2xx se deserializa al tipo pedido.
async fn interpretar<T: serde::de::DeserializeOwned>(
    respuesta: reqwest::Response,
) -> Result<T, ErrorTransporte> {
    let estado = respuesta.status();
    if !estado.is_success() {
        let cuerpo = respuesta.text().await.unwrap_or_default();
        return Err(ErrorTransporte::Http {
            estado: estado.as_u16(),
            cuerpo,
        });
    }
    respuesta.json::<T>().await.map_err(clasificar)
}
