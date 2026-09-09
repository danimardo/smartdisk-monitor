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
    /// FR-007 de la spec `006`: modo «enviar sin revisar» activo. Apagado de fábrica; se resetea al
    /// borrar la clave (FR-012).
    pub send_without_review: bool,
    pub clave_valida: Option<bool>,
}

/// Grupo `ai` de `SettingsWire` (`docs/data-model.md`). Cuatro datos: los tres de FR-024 de la 005
/// más `send_without_review` (FR-011 de la 006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct AiSettingsWire {
    pub enabled: bool,
    pub model: String,
    pub preview_acknowledged: bool,
    pub send_without_review: bool,
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
    /// `true` si la explicación se hizo sin el volcado técnico del disco porque no se pudo obtener
    /// en ese momento (FR-014, spec `006-explicacion-ia-contexto-crudo`).
    pub sin_volcado: bool,
    /// `true` si se hizo sin el contenido del suceso de Windows que originó la alerta (FR-015,
    /// spec `006-explicacion-ia-contexto-crudo`).
    pub sin_suceso: bool,
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

/// ¿La consulta debe pararse en la pantalla de revisión de fragmentos dudosos (FR-026 de la 005)?
/// `false` = seguir. Con «enviar sin revisar» activo (FR-009 de la 006) nunca para aquí; la vista
/// previa de la primera consulta (FR-010) es una comprobación aparte, posterior a esta.
pub fn debe_parar_en_revision(
    send_without_review: bool,
    revision: RevisionEnvio,
    hay_fragmentos_dudosos: bool,
) -> bool {
    !send_without_review && hay_fragmentos_dudosos && revision == RevisionEnvio::Ninguna
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
            // Los avisos de FR-014/FR-015 los conoce el comando (fue él quien recolectó los datos):
            // aquí se dejan en `false` y el comando los fija sobre el wire devuelto.
            sin_volcado: false,
            sin_suceso: false,
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

/// Compone el par `(system, user)` para la petición. **Ensambla el `user` en bruto** en orden
/// `resumen → volcado → suceso`; **no anonimiza**: el comando aplica `Anonimizador` +
/// [`redactar_identificadores`] sobre el texto completo después (spec `006`, §D1). Que el suceso
/// vaya al final hace que sea lo primero que se pierde al recortar (FR-013). El `system` es una
/// constante, no i18n (constitución §XV: es instrucción al modelo, no interfaz).
pub fn componer_consulta(
    detalle: &Detalle,
    disco: &ContextoDisco,
    idioma: Idioma,
    volcado: Option<&str>,
    suceso: Option<&str>,
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

    if let Some(v) = volcado {
        let v = v.trim();
        if !v.is_empty() {
            user.push_str("\nVolcado técnico del disco (smartctl -a -j):\n");
            user.push_str(v);
            user.push('\n');
        }
    }
    if let Some(s) = suceso {
        let s = s.trim();
        if !s.is_empty() {
            user.push_str("\nContenido del suceso de Windows que originó la alerta:\n");
            user.push_str(s);
            user.push('\n');
        }
    }

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

// ================================================================ anonimización de capas (spec 006)

/// Sustituye por marcadores los identificadores con **gramática fija** que la sustitución literal
/// del `Anonimizador` (capa 1) no cubre porque no vienen de un campo conocido: SID de Windows,
/// rutas NT de dispositivo y WWN hexadecimales (FR-004 de la spec `006`). Byte-a-byte, **sin
/// `regex`**, mismo criterio conservador que [`barrer_texto_residual`]: puede sustituir de más (una
/// explicación un poco más pobre), nunca de menos.
pub fn redactar_identificadores(texto: &str) -> String {
    let b = texto.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(n);
    let en_limite = |idx: usize| {
        idx == 0 || {
            let c = b[idx - 1];
            !(c.is_ascii_alphanumeric() || c == b'_')
        }
    };
    let mut i = 0;
    while i < n {
        // SID: "S-1-" seguido de al menos un grupo "-<dígitos>".
        if b[i] == b'S'
            && en_limite(i)
            && i + 3 < n
            && b[i + 1] == b'-'
            && b[i + 2] == b'1'
            && b[i + 3] == b'-'
        {
            let mut j = i + 3; // apunta al '-' tras el "1"
            let mut grupos = 0;
            while j < n && b[j] == b'-' {
                let mut k = j + 1;
                while k < n && b[k].is_ascii_digit() {
                    k += 1;
                }
                if k == j + 1 {
                    break; // '-' sin dígitos detrás
                }
                grupos += 1;
                j = k;
            }
            if grupos >= 1 {
                out.push_str("<SID>");
                i = j;
                continue;
            }
        }
        // Ruta NT de dispositivo: "\Device\..." hasta un separador.
        if b[i] == b'\\' && texto[i..].starts_with("\\Device\\") {
            let mut j = i + "\\Device\\".len();
            while j < n {
                let c = b[j];
                if c.is_ascii_whitespace()
                    || matches!(c, b'"' | b'\'' | b'<' | b'(' | b')' | b',' | b';')
                {
                    break;
                }
                j += 1;
            }
            out.push_str("<DISPOSITIVO>");
            i = j;
            continue;
        }
        // WWN hexadecimal: "0x" + 12..=16 dígitos hex, en límite de palabra por ambos lados.
        if (b[i] == b'0')
            && en_limite(i)
            && i + 2 < n
            && (b[i + 1] == b'x' || b[i + 1] == b'X')
            && b[i + 2].is_ascii_hexdigit()
        {
            let mut j = i + 2;
            while j < n && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            let hexlen = j - (i + 2);
            let fin_limpio = j == n || !(b[j].is_ascii_alphanumeric() || b[j] == b'_');
            if (12..=16).contains(&hexlen) && fin_limpio {
                out.push_str("<WWN>");
                i = j;
                continue;
            }
        }
        let ch = texto[i..]
            .chars()
            .next()
            .expect("i está en un límite de carácter");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[derive(Deserialize)]
struct VolcadoIds {
    #[serde(default)]
    serial_number: Option<String>,
    #[serde(default)]
    wwn: Option<VolcadoWwn>,
}

#[derive(Deserialize)]
struct VolcadoWwn {
    #[serde(default)]
    id: Option<i64>,
}

/// Extrae del JSON de `smartctl -a -j` los identificadores del disco que hay que anonimizar por
/// sustitución literal (capa 1, spec `006` §D3): número de serie y WWN (`wwn.id`, en la
/// representación decimal exacta con la que aparece en el JSON). Devuelve `(serie, wwn)`. Un JSON
/// ilegible o sin esos campos da `(None, None)` sin error: el volcado se enviará igual y el barrido
/// de patrones ([`redactar_identificadores`]) es la red de seguridad. La marca, el modelo y el
/// firmware **no** se extraen: son contexto necesario y no identifican a una persona (FR-005).
pub fn identificadores_de_volcado(json: &str) -> (Option<String>, Option<String>) {
    match serde_json::from_str::<VolcadoIds>(json) {
        Ok(v) => (
            v.serial_number.filter(|s| !s.trim().is_empty()),
            v.wwn.and_then(|w| w.id).map(|id| id.to_string()),
        ),
        Err(_) => (None, None),
    }
}

/// Contenido legible de un suceso de Windows para la ayuda con IA (spec `006`, FR-002): su mensaje
/// renderizado (si Windows lo guardó) más los pares `Nombre = valor` de los nodos `<EventData>` del
/// XML. **No** incluye el bloque `<System>` (proveedor, GUID, ProcessID/ThreadID, `Computer`,
/// `Security`): son metadatos de transporte que concentran identificadores y no explican nada.
/// `None` si no hay ni mensaje ni datos legibles (FR-015). Lector acotado sobre la cadena, sin
/// parser XML ni dependencia nueva; el `raw_xml` lo produce el colector de la 003 y su forma es
/// estable.
pub fn extraer_contenido_suceso(message: Option<&str>, raw_xml: Option<&str>) -> Option<String> {
    let mut partes: Vec<String> = Vec::new();
    if let Some(m) = message {
        let m = m.trim();
        if !m.is_empty() {
            partes.push(m.to_owned());
        }
    }
    if let Some(xml) = raw_xml {
        let datos = extraer_event_data(xml);
        if !datos.is_empty() {
            partes.push(format!("Datos del suceso:\n{}", datos.join("\n")));
        }
    }
    if partes.is_empty() {
        None
    } else {
        Some(partes.join("\n\n"))
    }
}

fn extraer_event_data(xml: &str) -> Vec<String> {
    let Some(ini) = xml.find("<EventData") else {
        return Vec::new();
    };
    let tras_ini = &xml[ini..];
    let Some(fin) = tras_ini.find("</EventData>") else {
        return Vec::new();
    };
    let bloque = &tras_ini[..fin];
    let mut out = Vec::new();
    let mut resto = bloque;
    while let Some(p) = resto.find("<Data") {
        resto = &resto[p + "<Data".len()..];
        let Some(cierre_tag) = resto.find('>') else {
            break;
        };
        let attrs = &resto[..cierre_tag];
        // `<Data .../>` sin contenido: ignorar y seguir.
        if attrs.ends_with('/') {
            resto = &resto[cierre_tag + 1..];
            continue;
        }
        let nombre = atributo_xml(attrs, "Name");
        resto = &resto[cierre_tag + 1..];
        let Some(fin_val) = resto.find("</Data>") else {
            break;
        };
        let valor = desescapar_xml(resto[..fin_val].trim());
        resto = &resto[fin_val + "</Data>".len()..];
        if valor.is_empty() {
            continue;
        }
        match nombre {
            Some(n) => out.push(format!("{n} = {valor}")),
            None => out.push(valor),
        }
    }
    out
}

fn atributo_xml(attrs: &str, nombre: &str) -> Option<String> {
    let clave = format!("{nombre}=");
    let pos = attrs.find(&clave)? + clave.len();
    let tras = &attrs[pos..];
    let comilla = tras.chars().next()?;
    if comilla != '\'' && comilla != '"' {
        return None;
    }
    let tras = &tras[1..];
    let fin = tras.find(comilla)?;
    Some(tras[..fin].to_owned())
}

fn desescapar_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
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
        let (system, user) = componer_consulta(
            &Detalle::Alerta(detalle),
            &disco_de_prueba(),
            Idioma::Es,
            None,
            None,
        );
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
        let (system, user) = componer_consulta(
            &Detalle::Alerta(detalle),
            &disco_de_prueba(),
            Idioma::En,
            None,
            None,
        );
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
        let (_, user) = componer_consulta(
            &Detalle::Alerta(detalle),
            &disco_de_prueba(),
            Idioma::Es,
            None,
            None,
        );
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
        let (system, user) = componer_consulta(
            &Detalle::Smart(&contadores),
            &disco_de_prueba(),
            Idioma::Es,
            None,
            None,
        );
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

    // ------------------------------------------- 006: volcado y suceso en componer_consulta (T102)

    #[test]
    fn componer_consulta_ensambla_resumen_volcado_suceso_en_ese_orden() {
        let detalle = DetalleAlerta {
            regla: "smart.reallocated_high",
            valor_actual: Some(8.0),
            tendencia: &[],
        };
        let (_, user) = componer_consulta(
            &Detalle::Alerta(detalle),
            &disco_de_prueba(),
            Idioma::Es,
            Some(r#"{"serial_number":"ABC123"}"#),
            Some("El controlador de disco detectó un error."),
        );
        let pos_resumen = user.find("smart.reallocated_high").unwrap();
        let pos_volcado = user.find("serial_number").unwrap();
        let pos_suceso = user.find("controlador de disco").unwrap();
        assert!(
            pos_resumen < pos_volcado,
            "el resumen va antes que el volcado"
        );
        assert!(
            pos_volcado < pos_suceso,
            "el volcado va antes que el suceso"
        );
        assert!(user.contains("Volcado técnico del disco"));
        assert!(user.contains("Contenido del suceso de Windows"));
    }

    #[test]
    fn componer_consulta_sin_volcado_ni_suceso_no_anade_secciones() {
        let detalle = DetalleAlerta {
            regla: "smart.wear_high",
            valor_actual: None,
            tendencia: &[],
        };
        let (_, user) = componer_consulta(
            &Detalle::Alerta(detalle),
            &disco_de_prueba(),
            Idioma::Es,
            None,
            None,
        );
        assert!(!user.contains("Volcado técnico"));
        assert!(!user.contains("suceso de Windows"));
    }

    #[test]
    fn al_recortar_se_pierde_antes_el_suceso_que_el_volcado_y_el_resumen() {
        let detalle = DetalleAlerta {
            regla: "smart.reallocated_high",
            valor_actual: Some(8.0),
            tendencia: &[],
        };
        let volcado = "V".repeat(200);
        let suceso = "S".repeat(200);
        let (_, user) = componer_consulta(
            &Detalle::Alerta(detalle),
            &disco_de_prueba(),
            Idioma::Es,
            Some(&volcado),
            Some(&suceso),
        );
        let (cortado, recortado) = recortar(&user, 120);
        assert!(recortado);
        assert!(
            cortado.contains("smart.reallocated_high"),
            "el resumen sobrevive"
        );
        assert!(!cortado.contains(&suceso), "el suceso completo se pierde");
    }

    // ------------------------------------------- 006: redactar_identificadores (T100)

    #[test]
    fn redactar_sustituye_sid_ruta_de_dispositivo_y_wwn_hex() {
        let t = redactar_identificadores(
            "user S-1-5-21-1004336348-1177238915-682003330-512 en \\Device\\HarddiskVolume24, wwn 0x5002538e40abc123 fin",
        );
        assert!(t.contains("<SID>"), "{t}");
        assert!(t.contains("<DISPOSITIVO>"), "{t}");
        assert!(t.contains("<WWN>"), "{t}");
        assert!(!t.contains("S-1-5-21"));
        assert!(!t.contains("HarddiskVolume24"));
        assert!(!t.contains("5002538e40abc123"));
    }

    #[test]
    fn redactar_no_toca_hex_corto_palabras_normales_ni_marcadores() {
        let t = redactar_identificadores(
            "color 0xFF, palabra Serial1234, marcador <SERIE-1> y <EQUIPO>",
        );
        assert_eq!(
            t,
            "color 0xFF, palabra Serial1234, marcador <SERIE-1> y <EQUIPO>"
        );
    }

    #[test]
    fn redactar_respeta_utf8_alrededor() {
        let t = redactar_identificadores("año 2024, disco \\Device\\Harddisk1\\DR19 señalado");
        assert!(t.starts_with("año 2024"));
        assert!(t.contains("<DISPOSITIVO> señalado"));
    }

    // ------------------------------------------- 006: identificadores_de_volcado (T101)

    #[test]
    fn identificadores_de_volcado_extrae_serie_y_wwn_id_decimal() {
        let json = r#"{"model_name":"WDC WD40EFAX","serial_number":"WD-WX12A34B5678","firmware_version":"83.00A83","wwn":{"naa":5,"oui":6478,"id":1234567890}}"#;
        let (serie, wwn) = identificadores_de_volcado(json);
        assert_eq!(serie.as_deref(), Some("WD-WX12A34B5678"));
        assert_eq!(wwn.as_deref(), Some("1234567890"));
    }

    #[test]
    fn identificadores_de_volcado_json_sin_wwn_o_ilegible_no_rompe() {
        let (serie, wwn) = identificadores_de_volcado(r#"{"serial_number":"ABC"}"#);
        assert_eq!(serie.as_deref(), Some("ABC"));
        assert_eq!(wwn, None);
        assert_eq!(identificadores_de_volcado("no es json"), (None, None));
    }

    // ------------------------------------------- 006: extraer_contenido_suceso (T300)

    const EVENTO_NTFS_98: &str = r#"<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='Microsoft-Windows-Ntfs' Guid='{3ff37a1c-a68d-4d6e-8c9b-f79e8b16c482}'/><EventID>98</EventID><TimeCreated SystemTime='2026-09-05T05:57:42Z'/><EventRecordID>811025</EventRecordID><Execution ProcessID='4' ThreadID='23724'/><Channel>System</Channel><Computer>Ryzen</Computer><Security UserID='S-1-5-18'/></System><EventData><Data Name='DriveName'>E:</Data><Data Name='DeviceName'>\Device\HarddiskVolume24</Data><Data Name='CorruptionActionState'>0</Data></EventData></Event>"#;
    const EVENTO_DISK_158: &str = r#"<Event><System><Provider Name='disk'/><EventID Qualifiers='32772'>158</EventID><Computer>Ryzen</Computer><Security/></System><EventData><Data>\Device\Harddisk1\DR19</Data><Data>1</Data><Binary>1B00000002003000</Binary></EventData></Event>"#;

    #[test]
    fn extraer_contenido_suceso_toma_mensaje_y_eventdata_sin_bloque_system() {
        let c = extraer_contenido_suceso(
            Some("El sistema de archivos de la estructura del disco está dañado."),
            Some(EVENTO_NTFS_98),
        )
        .unwrap();
        assert!(c.contains("estructura del disco"));
        assert!(c.contains("DriveName = E:"));
        assert!(c.contains("DeviceName = \\Device\\HarddiskVolume24"));
        assert!(c.contains("CorruptionActionState = 0"));
        assert!(!c.contains("Ryzen"), "no lleva <Computer>");
        assert!(!c.contains("S-1-5-18"), "no lleva <Security UserID>");
        assert!(!c.contains("ProcessID"));
        assert!(!c.contains("EventRecordID"));
    }

    #[test]
    fn extraer_contenido_suceso_data_sin_nombre_y_sin_mensaje() {
        let c = extraer_contenido_suceso(None, Some(EVENTO_DISK_158)).unwrap();
        assert!(c.contains("\\Device\\Harddisk1\\DR19"));
        assert!(!c.contains("Binary"));
        assert!(!c.contains("1B00000002003000"));
    }

    #[test]
    fn extraer_contenido_suceso_sin_nada_legible_es_none() {
        assert_eq!(
            extraer_contenido_suceso(None, Some("<Event><System/></Event>")),
            None
        );
        assert_eq!(extraer_contenido_suceso(Some("   "), None), None);
        assert_eq!(extraer_contenido_suceso(None, None), None);
    }

    // ------------------------------------------- 006: debe_parar_en_revision (T500)

    #[test]
    fn debe_parar_en_revision_con_fragmentos_y_sin_modo() {
        assert!(debe_parar_en_revision(false, RevisionEnvio::Ninguna, true));
    }

    #[test]
    fn enviar_sin_revisar_nunca_para_en_la_pantalla_de_fragmentos() {
        assert!(!debe_parar_en_revision(true, RevisionEnvio::Ninguna, true));
    }

    #[test]
    fn sin_fragmentos_o_ya_revisado_no_para() {
        assert!(!debe_parar_en_revision(
            false,
            RevisionEnvio::Ninguna,
            false
        ));
        assert!(!debe_parar_en_revision(
            false,
            RevisionEnvio::EnviarIgual,
            true
        ));
        assert!(!debe_parar_en_revision(
            false,
            RevisionEnvio::QuitarFragmentos,
            true
        ));
    }

    #[test]
    fn pasada_completa_de_anonimizacion_deja_marca_y_firmware_pero_no_ids() {
        // T103: anon.aplicar (capa 1) + redactar_identificadores (capa 2) sobre el texto ensamblado.
        let json = r#"{"model_name":"WDC WD40EFAX-68JH4N1","serial_number":"WD-WX12A34B5678","firmware_version":"83.00A83","wwn":{"naa":5,"oui":6478,"id":1234567890}}"#;
        let (serie, wwn) = identificadores_de_volcado(json);
        let anon = crate::reporting::anonimizar::Anonimizador::sin_anonimizar()
            .con_numero_de_serie(serie.as_deref().unwrap())
            .con_numero_de_serie(wwn.as_deref().unwrap());
        let crudo = format!("{json}\nSID S-1-5-21-1-2-3-513 en \\Device\\HarddiskVolume7");
        let limpio = redactar_identificadores(&anon.aplicar(&crudo));
        assert!(!limpio.contains("WD-WX12A34B5678"));
        assert!(!limpio.contains("1234567890"));
        assert!(!limpio.contains("S-1-5-21"));
        assert!(!limpio.contains("HarddiskVolume7"));
        assert!(
            limpio.contains("WDC WD40EFAX-68JH4N1"),
            "la marca/modelo se conserva"
        );
        assert!(limpio.contains("83.00A83"), "el firmware se conserva");
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
