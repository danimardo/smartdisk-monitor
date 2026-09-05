//! Identidad estable de un disco y reconciliación de altas y bajas (`docs/data-model.md` §2,
//! FR-001, FR-002, FR-007).
//!
//! El firmware queda fuera del cálculo a propósito: si formara parte de la huella, actualizarlo
//! partiría el historial del disco en dos entidades, y la arquitectura pide justo lo contrario.

use sha2::{Digest, Sha256};

use crate::domain::tipos::IdentityConfidence;

/// `sha256(model | capacity_bytes | bus_type | wwn_o_pnp_device_id)` (`docs/data-model.md` §2).
/// `wwn_o_pnp_device_id` es lo único que puede faltar de verdad: sin él la huella sigue siendo
/// estable mientras el llamante use consistentemente la misma cadena vacía.
pub fn compute_fingerprint(
    model: &str,
    capacity_bytes: Option<i64>,
    bus_type: &str,
    wwn_o_pnp_device_id: &str,
) -> String {
    let capacidad = capacity_bytes.map(|c| c.to_string()).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(model.as_bytes());
    hasher.update(b"|");
    hasher.update(capacidad.as_bytes());
    hasher.update(b"|");
    hasher.update(bus_type.as_bytes());
    hasher.update(b"|");
    hasher.update(wwn_o_pnp_device_id.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// La interfaz marca como identidad inferida los discos sin número de serie
/// (`docs/data-model.md` §2): sin serie, la única ancla es la huella calculada.
pub fn identity_confidence(serial_number: Option<&str>) -> IdentityConfidence {
    match serial_number {
        Some(s) if !s.trim().is_empty() => IdentityConfidence::Serial,
        _ => IdentityConfidence::Fingerprint,
    }
}

/// Resultado de comparar el inventario recién leído contra el conjunto de huellas presentes en
/// la base. `domain/`, no `collectors/`, decide qué es alta y qué es baja: el colector solo
/// entrega huellas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciliacion {
    /// Huellas que aparecen ahora y no estaban presentes: FR-001, FR-014 (alta en caliente).
    pub altas: Vec<String>,
    /// Huellas que estaban presentes y ya no aparecen: FR-014 (baja en caliente).
    pub bajas: Vec<String>,
    /// Huellas que siguen presentes en ambos conjuntos: solo se actualiza `last_seen_at`.
    pub siguen_presentes: Vec<String>,
}

/// Compara el inventario leído ahora contra el que estaba presente. No decide si una baja fue una
/// expulsión segura o una retirada sin aviso: eso depende del registro de eventos y es trabajo de
/// `collectors::deteccion` (T026), que sí puede consultarlo.
pub fn reconcile(
    huellas_presentes_antes: &[String],
    huellas_leidas_ahora: &[String],
) -> Reconciliacion {
    let antes: std::collections::HashSet<&str> =
        huellas_presentes_antes.iter().map(String::as_str).collect();
    let ahora: std::collections::HashSet<&str> =
        huellas_leidas_ahora.iter().map(String::as_str).collect();

    Reconciliacion {
        altas: ahora.difference(&antes).map(|s| s.to_string()).collect(),
        bajas: antes.difference(&ahora).map(|s| s.to_string()).collect(),
        siguen_presentes: antes.intersection(&ahora).map(|s| s.to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_huella_es_estable_para_las_mismas_entradas() {
        let a = compute_fingerprint("Samsung 980", Some(1_000_000_000_000), "nvme", "pnp-123");
        let b = compute_fingerprint("Samsung 980", Some(1_000_000_000_000), "nvme", "pnp-123");
        assert_eq!(a, b);
    }

    #[test]
    fn modelos_distintos_dan_huellas_distintas() {
        let a = compute_fingerprint("Samsung 980", Some(1_000_000_000_000), "nvme", "pnp-123");
        let b = compute_fingerprint("Crucial P3", Some(1_000_000_000_000), "nvme", "pnp-123");
        assert_ne!(a, b);
    }

    #[test]
    fn el_firmware_no_entra_en_la_huella_por_no_recibirlo_como_parametro() {
        // La firma de compute_fingerprint no acepta firmware: un cambio de firmware no puede
        // partir el historial porque no hay forma de que altere el resultado.
        let antes_de_actualizar =
            compute_fingerprint("WD Blue", Some(500_000_000_000), "sata", "wwn-9");
        let despues_de_actualizar =
            compute_fingerprint("WD Blue", Some(500_000_000_000), "sata", "wwn-9");
        assert_eq!(antes_de_actualizar, despues_de_actualizar);
    }

    #[test]
    fn con_numero_de_serie_la_confianza_es_serial() {
        assert_eq!(
            identity_confidence(Some("S3ZXNX0M123456")),
            IdentityConfidence::Serial
        );
    }

    #[test]
    fn sin_numero_de_serie_la_confianza_es_por_huella() {
        assert_eq!(identity_confidence(None), IdentityConfidence::Fingerprint);
        assert_eq!(
            identity_confidence(Some("")),
            IdentityConfidence::Fingerprint
        );
        assert_eq!(
            identity_confidence(Some("   ")),
            IdentityConfidence::Fingerprint
        );
    }

    #[test]
    fn reconcile_detecta_alta_baja_y_permanencia() {
        let antes = vec!["huella-a".to_string(), "huella-b".to_string()];
        let ahora = vec!["huella-b".to_string(), "huella-c".to_string()];

        let r = reconcile(&antes, &ahora);

        assert_eq!(r.altas, vec!["huella-c".to_string()]);
        assert_eq!(r.bajas, vec!["huella-a".to_string()]);
        assert_eq!(r.siguen_presentes, vec!["huella-b".to_string()]);
    }

    #[test]
    fn reconcile_sin_cambios_no_reporta_ni_altas_ni_bajas() {
        let inventario = vec!["huella-x".to_string()];
        let r = reconcile(&inventario, &inventario);
        assert!(r.altas.is_empty());
        assert!(r.bajas.is_empty());
        assert_eq!(r.siguen_presentes, inventario);
    }

    #[test]
    fn reconcile_desde_vacio_es_todo_altas() {
        let ahora = vec!["huella-nueva".to_string()];
        let r = reconcile(&[], &ahora);
        assert_eq!(r.altas, ahora);
        assert!(r.bajas.is_empty());
    }
}
