//! Correlación evento → dispositivo (T068/T069, `docs/alert-rules.md` §3.6).
//!
//! Puro: recibe el texto del evento y la lista de discos conocidos del inventario, decide
//! `device_id` y `mapping_confidence`. **Nunca por coincidencia textual pura**: un número de disco
//! que no aparece en `discos_conocidos` no correlaciona con nada, aunque el patrón encaje.
//!
//! Distingue dos formas de identificador que conviven en un mismo mensaje real
//! (`\Device\Harddisk1\DR19` en el XML crudo, "El disco 1..." en el mensaje ya traducido) con
//! confianza distinta (`docs/open-questions.md` J.25): `exact` para la ruta de dispositivo
//! estructurada que genera el propio sistema, `inferred` para el texto humano ya formateado. El
//! número que sigue a `DR` **no** es el número de disco físico y nunca se extrae como tal.
//!
//! `\Device\HarddiskVolumeNN` (identifica un volumen, no un disco) y los nombres PDO
//! (`\Device\0003d2a5`) quedan sin resolver: el colector de capacidad no persiste todavía el
//! identificador que haría falta para cruzarlos (J.25).

use crate::domain::tipos::MappingConfidence;

/// Un disco ya reconciliado, con el número efímero de Windows que aparece en las rutas de
/// dispositivo (`\\.\PhysicalDriveN`, `\Device\HarddiskN`) — el mismo que usa
/// `commands::disk_number_from_smartctl_path` para enlazar volúmenes.
#[derive(Debug, Clone, Copy)]
pub struct DiscoConocido<'a> {
    pub disk_number: i64,
    pub device_id: &'a str,
}

/// Busca `Harddisk<n>` (ruta de dispositivo estructurada, XML crudo) y devuelve `n`. No confunde
/// el `n` con el número que sigue a `DR`: se detiene en el primer separador tras los dígitos.
fn extraer_harddisk(texto: &str) -> Option<i64> {
    let inicio = texto.find("Harddisk")? + "Harddisk".len();
    let resto = &texto[inicio..];
    // "Volume" tras "Harddisk" identifica un volumen, no un disco: no es este patrón.
    if resto.starts_with("Volume") {
        return None;
    }
    let fin_digitos = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    if fin_digitos == 0 {
        return None;
    }
    resto[..fin_digitos].parse().ok()
}

/// Busca "disco <n>" o "disk <n>" (texto humano ya formateado, sin distinguir mayúsculas) y
/// devuelve `n`.
fn extraer_disco_en_texto(texto: &str) -> Option<i64> {
    let minusculas = texto.to_lowercase();
    for palabra in ["disco ", "disk "] {
        if let Some(pos) = minusculas.find(palabra) {
            let inicio = pos + palabra.len();
            let resto = &texto[inicio..];
            let fin_digitos = resto
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(resto.len());
            if fin_digitos > 0 {
                if let Ok(n) = resto[..fin_digitos].parse() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Decide `device_id` y confianza a partir del XML crudo y el mensaje humano de un evento.
/// `discos_conocidos` es el inventario ya reconciliado en el momento de la correlación — un
/// disco fuera de esa lista nunca produce una correlación, por muy bien que el patrón encaje.
pub fn correlacionar(
    raw_xml: &str,
    message: Option<&str>,
    discos_conocidos: &[DiscoConocido],
) -> (Option<String>, MappingConfidence) {
    let buscar = |n: i64| -> Option<&str> {
        discos_conocidos
            .iter()
            .find(|d| d.disk_number == n)
            .map(|d| d.device_id)
    };

    if let Some(n) = extraer_harddisk(raw_xml) {
        if let Some(device_id) = buscar(n) {
            return (Some(device_id.to_string()), MappingConfidence::Exact);
        }
    }

    if let Some(n) = message.and_then(extraer_disco_en_texto) {
        if let Some(device_id) = buscar(n) {
            return (Some(device_id.to_string()), MappingConfidence::Inferred);
        }
    }

    (None, MappingConfidence::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DISCOS: &[DiscoConocido] = &[
        DiscoConocido {
            disk_number: 0,
            device_id: "dev-0",
        },
        DiscoConocido {
            disk_number: 1,
            device_id: "dev-1",
        },
    ];

    #[test]
    fn una_ruta_de_dispositivo_estructurada_correlaciona_como_exacta() {
        let xml = r#"<EventData><Data>\Device\Harddisk1\DR19</Data></EventData>"#;
        let (device_id, confianza) = correlacionar(xml, None, DISCOS);
        assert_eq!(device_id.as_deref(), Some("dev-1"));
        assert_eq!(confianza, MappingConfidence::Exact);
    }

    #[test]
    fn el_numero_de_dr_nunca_se_confunde_con_el_numero_de_disco() {
        // \Device\Harddisk1\DR19: el disco es 1, el 19 es del DR y no debe usarse.
        let xml = r#"<Data>\Device\Harddisk1\DR19</Data>"#;
        let (device_id, _) = correlacionar(xml, None, DISCOS);
        assert_eq!(
            device_id.as_deref(),
            Some("dev-1"),
            "no debe resolver al disco 19 (no existe)"
        );
    }

    #[test]
    fn el_texto_humano_en_espanol_correlaciona_como_inferida() {
        let mensaje = "El disco 1 se ha extraído de forma imprevista del sistema.";
        let (device_id, confianza) = correlacionar("<Data>sin ruta</Data>", Some(mensaje), DISCOS);
        assert_eq!(device_id.as_deref(), Some("dev-1"));
        assert_eq!(confianza, MappingConfidence::Inferred);
    }

    #[test]
    fn el_texto_humano_en_ingles_tambien_correlaciona_como_inferida() {
        let mensaje = "Disk 0 has a bad block.";
        let (device_id, confianza) = correlacionar("<Data>sin ruta</Data>", Some(mensaje), DISCOS);
        assert_eq!(device_id.as_deref(), Some("dev-0"));
        assert_eq!(confianza, MappingConfidence::Inferred);
    }

    #[test]
    fn una_ruta_estructurada_gana_al_texto_humano_si_ambas_aparecen() {
        let xml = r#"<Data>\Device\Harddisk0\DR5</Data>"#;
        let mensaje = "El disco 1 puede fallar pronto.";
        let (device_id, confianza) = correlacionar(xml, Some(mensaje), DISCOS);
        assert_eq!(
            device_id.as_deref(),
            Some("dev-0"),
            "la ruta estructurada manda"
        );
        assert_eq!(confianza, MappingConfidence::Exact);
    }

    #[test]
    fn un_numero_de_disco_que_no_esta_en_el_inventario_no_correlaciona() {
        let xml = r#"<Data>\Device\Harddisk9\DR1</Data>"#;
        let (device_id, confianza) = correlacionar(xml, None, DISCOS);
        assert_eq!(
            device_id, None,
            "resolución contra el inventario, nunca por coincidencia textual"
        );
        assert_eq!(confianza, MappingConfidence::Unknown);
    }

    #[test]
    fn un_volumen_no_se_confunde_con_un_disco() {
        let xml = r#"<Data>\Device\HarddiskVolume24</Data>"#;
        let (device_id, confianza) = correlacionar(xml, None, DISCOS);
        assert_eq!(
            device_id, None,
            "HarddiskVolume no es HarddiskN: sin colector que lo capture, queda unknown"
        );
        assert_eq!(confianza, MappingConfidence::Unknown);
    }

    #[test]
    fn un_nombre_pdo_no_correlaciona() {
        let xml = r#"<Data>\Device\0003d2a5</Data>"#;
        let (device_id, confianza) = correlacionar(xml, None, DISCOS);
        assert_eq!(device_id, None);
        assert_eq!(confianza, MappingConfidence::Unknown);
    }

    #[test]
    fn sin_ningun_identificador_reconocible_queda_desconocido() {
        let (device_id, confianza) = correlacionar("<Data>nada relevante</Data>", None, DISCOS);
        assert_eq!(device_id, None);
        assert_eq!(confianza, MappingConfidence::Unknown);
    }

    #[test]
    fn sin_discos_conocidos_nada_correlaciona_aunque_el_patron_encaje() {
        let xml = r#"<Data>\Device\Harddisk0\DR1</Data>"#;
        let (device_id, confianza) = correlacionar(xml, None, &[]);
        assert_eq!(device_id, None);
        assert_eq!(confianza, MappingConfidence::Unknown);
    }
}
