//! Ayuda con IA (spec `005-explicacion-ia`, principio XVI): tipos del contrato y análisis puro de
//! la respuesta y de los errores de OpenRouter.
//!
//! **Sin `reqwest`, sin Tauri, sin Windows** (constitución §IV): este módulo recibe tipos ya
//! deserializados y devuelve decisiones. El transporte HTTP vive en
//! `crate::platform::ia_openrouter`, que llama aquí para interpretar lo que recibe.
//!
//! Toda fuente externa (el JSON de OpenRouter) se deserializa en tipos `serde` explícitos, nunca
//! se navega como `serde_json::Value` para decidir (constitución §XI): la versión de la API del
//! proveedor puede cambiar y un campo ausente debe notarse, no convertirse en `undefined`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------- DTO del contrato (ts-rs)

/// Estado de la ayuda con IA para la interfaz. `activa` = existe credencial; `clave_valida` es la
/// última comprobación de esta sesión (`None` = sin comprobar), una pista para la UI, no un dato
/// que se persista.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct EstadoIaWire {
    pub activa: bool,
    pub modelo: String,
    pub preview_acknowledged: bool,
    pub clave_valida: Option<bool>,
}

/// Grupo `ai` de `SettingsWire` (`docs/data-model.md`). Tres datos, los que fija FR-024.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct AiSettingsWire {
    pub enabled: bool,
    pub model: String,
    pub preview_acknowledged: bool,
}

/// Un modelo del catálogo de OpenRouter, para el selector. `es_de_pago` decide si mostrar el
/// aviso de posibles cargos (FR-015a).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct ModeloIaWire {
    pub id: String,
    pub nombre: String,
    pub es_de_pago: bool,
}

/// Explicación devuelta por el modelo. Efímera: no se persiste (`data-model.md` §3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct ExplicacionIaWire {
    /// Respuesta del modelo tal cual (markdown). Contenido **no confiable**: la interfaz lo
    /// renderiza como markdown seguro, jamás como HTML (principio XVI).
    pub markdown: String,
    /// El modelo que realmente atendió la petición (campo `model` de la respuesta), FR-016.
    pub modelo_usado: String,
    /// `true` si el detalle técnico se recortó antes de enviarlo (FR-021).
    pub detalle_recortado: bool,
}

/// Un fragmento del texto a enviar que la anonimización no ha podido garantizar limpio (FR-026).
/// Se manda el texto literal, no un rango: los desplazamientos en bytes de Rust no coinciden con
/// los de UTF-16 de JavaScript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct FragmentoDudosoWire {
    pub texto: String,
    /// Clave i18n del motivo (el backend no manda frases, ADR-030).
    pub motivo_key: String,
}

/// Lo que se enviaría al proveedor, para que la persona lo revise antes (FR-010 / FR-026).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct RevisionAnonimizacionWire {
    pub texto_completo: String,
    /// Vacío = es la vista previa de FR-010, no hay nada dudoso. No vacío = revisión de FR-026.
    pub fragmentos: Vec<FragmentoDudosoWire>,
}

/// De dónde sale el detalle técnico a explicar. Lo arma la interfaz con lo que ya tiene en
/// pantalla; el backend lo revalida contra inventario/alertas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct OrigenExplicacion {
    pub tipo: TipoOrigen,
    /// Obligatorio si `tipo == Smart` (para el caso `Alerta` el disco sale del grupo de alerta).
    pub device_id: Option<String>,
    /// Obligatorio si `tipo == Alerta`.
    pub alert_group_id: Option<String>,
    /// `"es"` o `"en"`; el backend lo revalida contra `settings.appearance.language`.
    pub idioma: String,
    pub revision: RevisionEnvio,
    pub preview_confirmada: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum TipoOrigen {
    Alerta,
    Smart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum RevisionEnvio {
    /// Aún no se ha revisado; si hay fragmentos dudosos, el comando devuelve `Revision`.
    Ninguna,
    /// La persona eligió enviar el texto tal cual.
    EnviarIgual,
    /// La persona eligió quitar los fragmentos dudosos.
    QuitarFragmentos,
}

/// Lo que devuelve `explicar_detalle_tecnico` cuando no es un `AppError`: o la explicación, o una
/// pantalla de revisión/vista previa. Etiquetado por `estado` (`"ok"` | `"revision"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "estado", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum ResultadoExplicacion {
    Ok(ExplicacionIaWire),
    Revision(RevisionAnonimizacionWire),
}

// ---------------------------------------------------------------- respuesta cruda de OpenRouter

/// Cuerpo de `POST /chat/completions`. Campos opcionales y `#[serde(default)]` porque un proveedor
/// gratuito puede omitir cualquiera de ellos.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RespuestaChat {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<EleccionChat>,
    /// Algunos proveedores devuelven `200` con un objeto de error en el cuerpo.
    #[serde(default)]
    pub error: Option<ErrorProveedor>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EleccionChat {
    #[serde(default)]
    pub message: Option<MensajeChat>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MensajeChat {
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ErrorProveedor {
    #[serde(default)]
    pub message: Option<String>,
}

/// Cuerpo de `GET /models`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RespuestaModelos {
    #[serde(default)]
    pub data: Vec<ModeloBruto>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModeloBruto {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub pricing: Option<PrecioBruto>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrecioBruto {
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub completion: Option<String>,
}

// ---------------------------------------------------------------- resultado del transporte

/// Lo que `platform::ia_openrouter` puede fallar. `analizar_error` lo traduce a `AppError`.
#[derive(Debug)]
pub enum ErrorTransporte {
    /// Respuesta HTTP no-2xx: código y cuerpo crudo (para el `detail`).
    Http { estado: u16, cuerpo: String },
    /// No se pudo establecer la conexión (DNS, TLS, rechazo, red caída).
    Conexion(String),
    /// Se superó el tiempo máximo de espera.
    Timeout,
    /// La respuesta llegó pero no se pudo deserializar.
    Deserializacion(String),
}

// ---------------------------------------------------------------- análisis puro

/// Identificador del router de modelos gratuitos (ADR-046). Se antepone siempre en el selector.
pub const MODELO_AUTOMATICO: &str = "openrouter/free";

/// Interpreta el cuerpo de `chat/completions`. `detalle_recortado` lo sabe el llamador (fue él
/// quien recortó el detalle técnico) y se propaga tal cual.
pub fn analizar_respuesta(
    cuerpo: RespuestaChat,
    detalle_recortado: bool,
) -> AppResult<ExplicacionIaWire> {
    if let Some(err) = cuerpo.error {
        let detalle = err
            .message
            .unwrap_or_else(|| "el proveedor devolvió un error sin mensaje".to_owned());
        return Err(Box::new(
            AppError::new("ia.provider", "error.ia.provider")
                .with_detail(detalle)
                .retryable(),
        ));
    }

    let markdown = cuerpo
        .choices
        .into_iter()
        .find_map(|c| c.message.and_then(|m| m.content))
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());

    match markdown {
        Some(markdown) => Ok(ExplicacionIaWire {
            markdown,
            modelo_usado: cuerpo.model.unwrap_or_default(),
            detalle_recortado,
        }),
        None => Err(Box::new(
            AppError::new("ia.empty_response", "error.ia.emptyResponse").retryable(),
        )),
    }
}

/// Traduce un fallo del transporte a `AppError` (`contracts/comandos-ia.md`). El `detail` lleva el
/// texto técnico crudo; **nunca** el prompt ni la respuesta del modelo.
pub fn analizar_error(err: ErrorTransporte) -> Box<AppError> {
    match err {
        ErrorTransporte::Http { estado, cuerpo } => {
            let mensaje = mensaje_de_error_proveedor(&cuerpo);
            let detalle = format!(
                "HTTP {estado}: {}",
                mensaje.unwrap_or_else(|| recorte_para_detalle(&cuerpo))
            );
            match estado {
                401 | 403 => Box::new(
                    AppError::new("ia.unauthorized", "error.ia.unauthorized").with_detail(detalle),
                ),
                402 | 429 => Box::new(
                    AppError::new("ia.rate_limited", "error.ia.rateLimited")
                        .with_detail(detalle)
                        .retryable(),
                ),
                500..=599 => Box::new(
                    AppError::new("ia.provider", "error.ia.provider")
                        .with_detail(detalle)
                        .retryable(),
                ),
                _ => {
                    Box::new(AppError::new("ia.provider", "error.ia.provider").with_detail(detalle))
                }
            }
        }
        ErrorTransporte::Conexion(detalle) => Box::new(
            AppError::new("ia.network", "error.ia.network")
                .with_detail(detalle)
                .retryable(),
        ),
        ErrorTransporte::Timeout => Box::new(
            AppError::new("ia.timeout", "error.ia.timeout")
                .with_detail("sin respuesta en 60 s")
                .retryable(),
        ),
        ErrorTransporte::Deserializacion(detalle) => Box::new(
            AppError::new("ia.provider", "error.ia.provider")
                .with_detail(format!("respuesta ilegible: {detalle}"))
                .retryable(),
        ),
    }
}

/// Mapea la respuesta de `GET /models` al catálogo del selector: antepone «modelo gratuito
/// automático», luego los gratuitos, luego los de pago; dentro de cada grupo, por nombre.
pub fn catalogo_modelos(respuesta: RespuestaModelos) -> Vec<ModeloIaWire> {
    let mut modelos: Vec<ModeloIaWire> = respuesta
        .data
        .into_iter()
        .filter(|m| m.id != MODELO_AUTOMATICO)
        .map(|m| ModeloIaWire {
            nombre: m.name.clone().unwrap_or_else(|| m.id.clone()),
            es_de_pago: modelo_es_de_pago(m.pricing.as_ref()),
            id: m.id,
        })
        .collect();

    modelos.sort_by(|a, b| {
        a.es_de_pago
            .cmp(&b.es_de_pago)
            .then_with(|| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase()))
    });

    let mut catalogo = vec![ModeloIaWire {
        id: MODELO_AUTOMATICO.to_owned(),
        nombre: MODELO_AUTOMATICO.to_owned(),
        es_de_pago: false,
    }];
    catalogo.extend(modelos);
    catalogo
}

/// `true` si el modelo cobra por uso: cualquier precio de prompt o de compleción distinto de `"0"`
/// (OpenRouter da el precio por token como cadena). Sin datos de precio se asume de pago, el lado
/// seguro para el aviso de cargos.
pub fn modelo_es_de_pago(pricing: Option<&PrecioBruto>) -> bool {
    match pricing {
        None => true,
        Some(p) => !es_gratis(&p.prompt) || !es_gratis(&p.completion),
    }
}

fn es_gratis(precio: &Option<String>) -> bool {
    match precio {
        None => false,
        Some(s) => {
            let s = s.trim();
            s == "0" || s.parse::<f64>().map(|v| v == 0.0).unwrap_or(false)
        }
    }
}

fn mensaje_de_error_proveedor(cuerpo: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(cuerpo).ok()?;
    v.get("error")?
        .get("message")?
        .as_str()
        .map(|s| s.to_owned())
}

fn recorte_para_detalle(texto: &str) -> String {
    const MAX: usize = 300;
    let t = texto.trim();
    if t.chars().count() <= MAX {
        t.to_owned()
    } else {
        let corte: String = t.chars().take(MAX).collect();
        format!("{corte}…")
    }
}

// ================================================================ composición de la consulta

/// Idioma en el que se le pide la respuesta al modelo (el de la interfaz).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Idioma {
    Es,
    En,
}

impl Idioma {
    pub fn de_codigo(codigo: &str) -> Self {
        if codigo.eq_ignore_ascii_case("en") {
            Self::En
        } else {
            Self::Es
        }
    }
}

/// Contexto acotado del disco al que se refiere la alerta (FR-008, aclaración Q2). **Ya
/// anonimizado** por quien construye estas cadenas: aquí no llega el número de serie.
#[derive(Debug, Clone, Copy)]
pub struct ContextoDisco<'a> {
    pub modelo: &'a str,
    /// «NVMe», «SSD SATA», «HDD», «USB»…
    pub tipo: &'a str,
    pub bus: Option<&'a str>,
    pub firmware: Option<&'a str>,
    pub antiguedad_meses: Option<u32>,
}

/// El detalle técnico de la alerta que se va a explicar.
#[derive(Debug, Clone, Copy)]
pub struct DetalleAlerta<'a> {
    pub regla: &'a str,
    pub valor_actual: Option<f64>,
    /// Valores de las últimas ocurrencias, del más antiguo al más nuevo.
    pub tendencia: &'a [f64],
}

/// Un contador SMART tal y como lo muestra el detalle de disco.
#[derive(Debug, Clone, Copy)]
pub struct ContadorSmart<'a> {
    pub nombre: &'a str,
    pub valor: Option<f64>,
    pub unidad: Option<&'a str>,
    /// `true` si el incremento respecto a la lectura anterior es significativo (va subiendo).
    pub significativo: bool,
}

/// Qué se le pide explicar: una alerta o los contadores SMART de un disco.
#[derive(Debug, Clone, Copy)]
pub enum Detalle<'a> {
    Alerta(DetalleAlerta<'a>),
    Smart(&'a [ContadorSmart<'a>]),
}

const SYSTEM_ES: &str = "Eres un asistente que explica el estado de un disco de ordenador a una \
persona SIN conocimientos técnicos. Con los datos que te den:\n\
- Explica en lenguaje llano qué significa la alerta y a qué se debe.\n\
- Di claramente si es algo urgente, algo a vigilar o algo sin importancia.\n\
- Si procede, indica pasos concretos y sencillos que la persona puede dar (por ejemplo, hacer una \
copia de seguridad o sustituir el disco).\n\
Reglas: responde SIEMPRE en español. Usa Markdown con encabezados (##) y listas. Sé breve (menos \
de 250 palabras). NO inventes datos, valores ni causas que no estén en el texto que te doy. NO \
uses tecnicismos sin explicarlos.";

const SYSTEM_EN: &str = "You are an assistant that explains the state of a computer disk to a \
person with NO technical background. With the data you are given:\n\
- Explain in plain language what the alert means and what causes it.\n\
- Clearly say whether this is urgent, something to keep an eye on, or nothing to worry about.\n\
- Where appropriate, give concrete, simple steps the person can take (for example, back up their \
data or replace the disk).\n\
Rules: always answer in English. Use Markdown with headings (##) and lists. Be brief (under 250 \
words). Do NOT invent data, values or causes that are not in the text I give you. Do NOT use \
jargon without explaining it.";

/// Compone el par `(system, user)` para la petición. `detalle` y `disco` deben venir **ya
/// anonimizados**. El `system` es una constante, no i18n (constitución §XV: es instrucción al
/// modelo, no interfaz).
pub fn componer_consulta(
    detalle: &Detalle,
    disco: &ContextoDisco,
    idioma: Idioma,
) -> (String, String) {
    let system = match idioma {
        Idioma::Es => SYSTEM_ES,
        Idioma::En => SYSTEM_EN,
    };

    let mut user = String::new();
    match detalle {
        Detalle::Alerta(a) => {
            user.push_str("Regla de alerta activada: ");
            user.push_str(a.regla);
            user.push('\n');
            if let Some(v) = a.valor_actual {
                user.push_str(&format!("Valor actual: {}\n", formato_num(v)));
            }
            if !a.tendencia.is_empty() {
                let serie: Vec<String> = a.tendencia.iter().map(|v| formato_num(*v)).collect();
                user.push_str(&format!(
                    "Valores recientes (del más antiguo al más nuevo): {}\n",
                    serie.join(", ")
                ));
            }
        }
        Detalle::Smart(contadores) => {
            user.push_str("Contadores SMART del disco:\n");
            for c in *contadores {
                user.push_str("- ");
                user.push_str(c.nombre);
                user.push_str(" = ");
                match c.valor {
                    Some(v) => user.push_str(&formato_num(v)),
                    None => user.push_str("sin dato"),
                }
                if let Some(u) = c.unidad {
                    user.push(' ');
                    user.push_str(u);
                }
                if c.significativo {
                    user.push_str(" (subiendo)");
                }
                user.push('\n');
            }
        }
    }

    user.push_str("Disco: modelo ");
    user.push_str(disco.modelo);
    user.push_str(", tipo ");
    user.push_str(disco.tipo);
    if let Some(bus) = disco.bus {
        user.push_str(&format!(", conexión {bus}"));
    }
    if let Some(fw) = disco.firmware {
        user.push_str(&format!(", firmware {fw}"));
    }
    if let Some(meses) = disco.antiguedad_meses {
        user.push_str(&format!(", antigüedad aproximada {meses} meses"));
    }
    user.push('\n');

    (system.to_owned(), user)
}

fn formato_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// Recorta `texto` a `max` caracteres si lo excede, cortando en el último salto de línea que quepa
/// para no partir una frase. Devuelve `(texto, se_recorto)`.
pub fn recortar(texto: &str, max: usize) -> (String, bool) {
    if texto.chars().count() <= max {
        return (texto.to_owned(), false);
    }
    let corte_byte = texto
        .char_indices()
        .nth(max)
        .map(|(i, _)| i)
        .unwrap_or(texto.len());
    let bruto = &texto[..corte_byte];
    let limpio = bruto.rsplit_once('\n').map(|(a, _)| a).unwrap_or(bruto);
    (format!("{}\n[…]", limpio.trim_end()), true)
}

// ================================================================ barrido de texto residual (FR-026)

/// Clave i18n del motivo de un `FragmentoDudosoWire`.
pub const MOTIVO_RUTA: &str = "ia.review.path";
pub const MOTIVO_RED: &str = "ia.review.networkPath";
pub const MOTIVO_TOKEN: &str = "ia.review.identifierLike";

/// Marca fragmentos que la anonimización no ha podido garantizar limpios (FR-026): rutas con letra
/// de unidad, rutas de red UNC y tokens alfanuméricos largos tipo número de serie. Heurística
/// **conservadora**: puede señalar de más (molesto), nunca de menos (inseguro). Los marcadores ya
/// sustituidos (`<EQUIPO>`, `<SERIE-1>`, `<OMITIDO>`…) no se señalan. Sin duplicados.
pub fn barrer_texto_residual(texto: &str) -> Vec<FragmentoDudosoWire> {
    let bytes = texto.as_bytes();
    let n = bytes.len();
    let mut fragmentos: Vec<FragmentoDudosoWire> = Vec::new();
    let mut i = 0;

    let es_sep =
        |c: u8| c.is_ascii_whitespace() || c == b',' || c == b';' || c == b')' || c == b'(';
    let mut empujar = |frag: &str, motivo: &str| {
        if !frag.is_empty() && !fragmentos.iter().any(|f| f.texto == frag) {
            fragmentos.push(FragmentoDudosoWire {
                texto: frag.to_owned(),
                motivo_key: motivo.to_owned(),
            });
        }
    };

    while i < n {
        // Ruta de red UNC: \\algo\algo
        if i + 1 < n && bytes[i] == b'\\' && bytes[i + 1] == b'\\' {
            let inicio = i;
            i += 2;
            while i < n && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            empujar(&texto[inicio..i], MOTIVO_RED);
            continue;
        }
        // Ruta con letra de unidad: X:\ o X:/
        if bytes[i].is_ascii_alphabetic()
            && i + 2 < n
            && bytes[i + 1] == b':'
            && (bytes[i + 2] == b'\\' || bytes[i + 2] == b'/')
            && (i == 0 || es_sep(bytes[i - 1]))
        {
            let inicio = i;
            i += 3;
            while i < n && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            empujar(&texto[inicio..i], MOTIVO_RUTA);
            continue;
        }
        // Token: entre separadores. Se marca si parece un identificador (>=8, mezcla letra+dígito)
        // y no es un marcador ya sustituido.
        if i == 0 || es_sep(bytes[i - 1]) {
            let inicio = i;
            while i < n && !es_sep(bytes[i]) {
                i += 1;
            }
            if i == inicio {
                // El carácter actual también es un separador (dos seguidos): avanzar uno.
                i += 1;
                continue;
            }
            let token = &texto[inicio..i];
            if token_parece_identificador(token) {
                empujar(token, MOTIVO_TOKEN);
            }
            continue;
        }
        i += 1;
    }
    fragmentos
}

fn token_parece_identificador(token: &str) -> bool {
    let t = token.trim_matches(|c: char| matches!(c, '.' | ':' | '"' | '\'' | '«' | '»'));
    if t.starts_with('<') || t.len() < 8 {
        return false;
    }
    let mut letra = false;
    let mut digito = false;
    for c in t.chars() {
        if c.is_ascii_alphabetic() {
            letra = true;
        } else if c.is_ascii_digit() {
            digito = true;
        } else if c != '-' && c != '_' {
            return false;
        }
    }
    letra && digito
}

#[cfg(test)]
mod tests {
    use super::*;

    fn respuesta_con_contenido(contenido: &str) -> RespuestaChat {
        RespuestaChat {
            model: Some("vendor/model:free".to_owned()),
            choices: vec![EleccionChat {
                message: Some(MensajeChat {
                    content: Some(contenido.to_owned()),
                }),
            }],
            error: None,
        }
    }

    #[test]
    fn respuesta_ok_devuelve_markdown_y_modelo_real() {
        let r = analizar_respuesta(respuesta_con_contenido("  ## Hola\n\ntexto  "), false).unwrap();
        assert_eq!(r.markdown, "## Hola\n\ntexto");
        assert_eq!(r.modelo_usado, "vendor/model:free");
        assert!(!r.detalle_recortado);
    }

    #[test]
    fn respuesta_ok_propaga_detalle_recortado() {
        let r = analizar_respuesta(respuesta_con_contenido("x"), true).unwrap();
        assert!(r.detalle_recortado);
    }

    #[test]
    fn respuesta_sin_choices_es_empty_response_reintentable() {
        let err = analizar_respuesta(RespuestaChat::default(), false).unwrap_err();
        assert_eq!(err.code, "ia.empty_response");
        assert!(err.retryable);
    }

    #[test]
    fn respuesta_con_contenido_vacio_es_empty_response() {
        let err = analizar_respuesta(respuesta_con_contenido("   \n  "), false).unwrap_err();
        assert_eq!(err.code, "ia.empty_response");
    }

    #[test]
    fn respuesta_200_con_objeto_error_es_provider() {
        let r = RespuestaChat {
            error: Some(ErrorProveedor {
                message: Some("quota exceeded".to_owned()),
            }),
            ..RespuestaChat::default()
        };
        let err = analizar_respuesta(r, false).unwrap_err();
        assert_eq!(err.code, "ia.provider");
        assert_eq!(err.detail.as_deref(), Some("quota exceeded"));
        assert!(err.retryable);
    }

    #[test]
    fn error_401_es_unauthorized_no_reintentable() {
        let err = analizar_error(ErrorTransporte::Http {
            estado: 401,
            cuerpo: r#"{"error":{"message":"No auth credentials found"}}"#.to_owned(),
        });
        assert_eq!(err.code, "ia.unauthorized");
        assert!(!err.retryable);
        assert!(err
            .detail
            .as_deref()
            .unwrap()
            .contains("No auth credentials found"));
    }

    #[test]
    fn error_429_y_402_son_rate_limited_reintentable() {
        for estado in [429, 402] {
            let err = analizar_error(ErrorTransporte::Http {
                estado,
                cuerpo: "{}".to_owned(),
            });
            assert_eq!(err.code, "ia.rate_limited", "estado {estado}");
            assert!(err.retryable);
        }
    }

    #[test]
    fn error_503_es_provider_reintentable() {
        let err = analizar_error(ErrorTransporte::Http {
            estado: 503,
            cuerpo: "upstream unavailable".to_owned(),
        });
        assert_eq!(err.code, "ia.provider");
        assert!(err.retryable);
        assert!(err.detail.as_deref().unwrap().contains("HTTP 503"));
    }

    #[test]
    fn error_400_es_provider_no_reintentable() {
        let err = analizar_error(ErrorTransporte::Http {
            estado: 400,
            cuerpo: "bad request".to_owned(),
        });
        assert_eq!(err.code, "ia.provider");
        assert!(!err.retryable);
    }

    #[test]
    fn error_conexion_y_timeout_son_reintentables() {
        assert_eq!(
            analizar_error(ErrorTransporte::Conexion("dns".to_owned())).code,
            "ia.network"
        );
        assert!(analizar_error(ErrorTransporte::Conexion("dns".to_owned())).retryable);
        assert_eq!(analizar_error(ErrorTransporte::Timeout).code, "ia.timeout");
        assert!(analizar_error(ErrorTransporte::Timeout).retryable);
    }

    #[test]
    fn deserializacion_fallida_no_filtra_el_cuerpo_entero_en_el_detalle() {
        let err = analizar_error(ErrorTransporte::Deserializacion(
            "expected value at line 1".to_owned(),
        ));
        assert_eq!(err.code, "ia.provider");
        assert!(err.detail.as_deref().unwrap().contains("expected value"));
    }

    #[test]
    fn modelo_sin_precio_se_considera_de_pago() {
        assert!(modelo_es_de_pago(None));
    }

    #[test]
    fn modelo_con_precio_cero_es_gratis() {
        let p = PrecioBruto {
            prompt: Some("0".to_owned()),
            completion: Some("0".to_owned()),
        };
        assert!(!modelo_es_de_pago(Some(&p)));
    }

    #[test]
    fn modelo_con_algun_precio_no_cero_es_de_pago() {
        let p = PrecioBruto {
            prompt: Some("0".to_owned()),
            completion: Some("0.0000012".to_owned()),
        };
        assert!(modelo_es_de_pago(Some(&p)));
    }

    #[test]
    fn el_catalogo_antepone_el_automatico_y_ordena_gratis_antes_que_pago() {
        let respuesta = RespuestaModelos {
            data: vec![
                ModeloBruto {
                    id: "z/pago".to_owned(),
                    name: Some("Zeta de pago".to_owned()),
                    pricing: Some(PrecioBruto {
                        prompt: Some("0.001".to_owned()),
                        completion: Some("0.001".to_owned()),
                    }),
                },
                ModeloBruto {
                    id: "a/gratis".to_owned(),
                    name: Some("Alfa gratis".to_owned()),
                    pricing: Some(PrecioBruto {
                        prompt: Some("0".to_owned()),
                        completion: Some("0".to_owned()),
                    }),
                },
                ModeloBruto {
                    id: MODELO_AUTOMATICO.to_owned(),
                    name: None,
                    pricing: None,
                },
            ],
        };
        let catalogo = catalogo_modelos(respuesta);
        assert_eq!(catalogo[0].id, MODELO_AUTOMATICO);
        assert!(!catalogo[0].es_de_pago);
        assert_eq!(catalogo[1].id, "a/gratis");
        assert_eq!(catalogo[2].id, "z/pago");
        assert!(catalogo[2].es_de_pago);
    }

    // ---------------------------------------------------------------- composición de la consulta

    fn disco_de_prueba() -> ContextoDisco<'static> {
        ContextoDisco {
            modelo: "Samsung SSD 990 PRO 2TB",
            tipo: "NVMe",
            bus: Some("PCIe"),
            firmware: Some("4B2QJXD7"),
            antiguedad_meses: Some(18),
        }
    }

    #[test]
    fn el_prompt_lleva_regla_valor_tendencia_y_contexto_del_disco() {
        let detalle = DetalleAlerta {
            regla: "smart.reallocated_high",
            valor_actual: Some(8.0),
            tendencia: &[5.0, 6.0, 8.0],
        };
        let (system, user) =
            componer_consulta(&Detalle::Alerta(detalle), &disco_de_prueba(), Idioma::Es);
        assert!(system.contains("español"));
        assert!(system.contains("NO inventes"));
        assert!(user.contains("smart.reallocated_high"));
        assert!(user.contains("Valor actual: 8"));
        assert!(user.contains("5, 6, 8"));
        assert!(user.contains("Samsung SSD 990 PRO 2TB"));
        assert!(user.contains("firmware 4B2QJXD7"));
        assert!(user.contains("18 meses"));
    }

    #[test]
    fn el_prompt_en_ingles_pide_respuesta_en_ingles() {
        let detalle = DetalleAlerta {
            regla: "smart.wear_high",
            valor_actual: None,
            tendencia: &[],
        };
        let (system, user) =
            componer_consulta(&Detalle::Alerta(detalle), &disco_de_prueba(), Idioma::En);
        assert!(system.contains("English"));
        assert!(!user.contains("Valor actual"));
        assert!(!user.contains("Valores recientes"));
    }

    #[test]
    fn el_prompt_nunca_lleva_lo_que_el_llamante_no_le_dio() {
        // El contexto ya viene anonimizado; este módulo no conoce el número de serie.
        let detalle = DetalleAlerta {
            regla: "smart.temp_high",
            valor_actual: Some(61.0),
            tendencia: &[],
        };
        let (_, user) =
            componer_consulta(&Detalle::Alerta(detalle), &disco_de_prueba(), Idioma::Es);
        assert!(!user.to_lowercase().contains("serie"));
        assert!(!user.contains("<"));
    }

    #[test]
    fn el_prompt_smart_lista_los_contadores_con_su_tendencia_y_el_contexto_del_disco() {
        let contadores = [
            ContadorSmart {
                nombre: "reallocated_sectors",
                valor: Some(8.0),
                unidad: Some("count"),
                significativo: true,
            },
            ContadorSmart {
                nombre: "temperature",
                valor: Some(41.0),
                unidad: Some("celsius"),
                significativo: false,
            },
        ];
        let (system, user) =
            componer_consulta(&Detalle::Smart(&contadores), &disco_de_prueba(), Idioma::Es);
        assert!(system.contains("español"));
        assert!(user.contains("reallocated_sectors = 8 count (subiendo)"));
        assert!(user.contains("temperature = 41 celsius\n"));
        assert!(user.contains("Samsung SSD 990 PRO 2TB"));
        assert!(!user.to_lowercase().contains("serie"));
    }

    #[test]
    fn idioma_de_codigo_solo_es_ingles_con_en() {
        assert_eq!(Idioma::de_codigo("en"), Idioma::En);
        assert_eq!(Idioma::de_codigo("EN"), Idioma::En);
        assert_eq!(Idioma::de_codigo("es"), Idioma::Es);
        assert_eq!(Idioma::de_codigo("fr"), Idioma::Es);
    }

    // ---------------------------------------------------------------- recortar

    #[test]
    fn recortar_no_toca_un_texto_corto() {
        let (t, recortado) = recortar("línea 1\nlínea 2", 100);
        assert_eq!(t, "línea 1\nlínea 2");
        assert!(!recortado);
    }

    #[test]
    fn recortar_corta_en_el_ultimo_salto_de_linea() {
        let texto = "aaaa\nbbbb\ncccc\ndddd";
        let (t, recortado) = recortar(texto, 12);
        assert!(recortado);
        assert!(t.ends_with("[…]"));
        assert!(!t.contains("cccc"));
        assert!(t.contains("bbbb"));
    }

    #[test]
    fn recortar_respeta_los_limites_de_caracter_multibyte() {
        let texto = "ñ".repeat(50);
        let (t, recortado) = recortar(&texto, 10);
        assert!(recortado);
        // No debe entrar en pánico ni producir UTF-8 inválido.
        assert!(t.chars().all(|c| c == 'ñ' || "\n[…]".contains(c)));
    }

    // ---------------------------------------------------------------- barrer_texto_residual

    fn textos_marcados(texto: &str) -> Vec<String> {
        barrer_texto_residual(texto)
            .into_iter()
            .map(|f| f.texto)
            .collect()
    }

    #[test]
    fn marca_una_ruta_con_letra_de_unidad() {
        let m = textos_marcados("El evento menciona D:\\Usuarios\\Ana\\Documentos y nada más.");
        assert!(m.iter().any(|t| t.starts_with("D:\\Usuarios\\Ana")));
    }

    #[test]
    fn marca_una_ruta_de_red_unc() {
        let m = textos_marcados("copia en \\\\NAS-CASA\\backups\\disco");
        assert!(m.iter().any(|t| t.contains("NAS-CASA")));
    }

    #[test]
    fn marca_un_token_tipo_numero_de_serie_no_sustituido() {
        let m = textos_marcados("Serie detectada: S5GXNX0T104655 en el registro.");
        assert!(m.contains(&"S5GXNX0T104655".to_owned()));
    }

    #[test]
    fn no_marca_palabras_normales_ni_numeros_sueltos() {
        let frags = barrer_texto_residual(
            "El disco Samsung tiene 8 sectores reasignados desde hace 14200 horas.",
        );
        assert!(frags.is_empty(), "marcó de más: {frags:?}");
    }

    #[test]
    fn no_marca_los_marcadores_ya_sustituidos_ni_bucle_con_separadores_seguidos() {
        let frags = barrer_texto_residual("Equipo <EQUIPO>, disco <SERIE-1>, fragmento <OMITIDO>.");
        assert!(frags.is_empty());
    }

    #[test]
    fn no_duplica_un_fragmento_que_aparece_dos_veces() {
        let frags = barrer_texto_residual("ruta D:\\datos y otra vez D:\\datos aquí");
        assert_eq!(frags.len(), 1);
    }
}
