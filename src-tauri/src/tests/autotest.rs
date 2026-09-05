//! Autotest SMART corto (T081, `docs/product-specification.md` §6).
//!
//! **El JSON de estado no está verificado contra hardware real** (`docs/open-questions.md` J.28):
//! un autotest corto real tarda minutos y esta sesión no lo ha ejecutado. El resto de este módulo
//! —construcción de argumentos, verificación de compatibilidad— sí es determinista y se prueba
//! sin reservas.

/// `smartctl -t short -j <ruta>`: arranca el autotest corto. Es asíncrono en el propio disco —
/// `smartctl` vuelve enseguida, el resultado se consulta después con `-a -j` (`consultar_estado`).
pub fn argumentos_iniciar(device_path: &str) -> Vec<String> {
    vec![
        "-t".to_string(),
        "short".to_string(),
        "-j".to_string(),
        device_path.to_string(),
    ]
}

/// `smartctl -X <ruta>`: aborta el autotest en curso, si el dispositivo lo admite
/// (`product-specification.md` §6, "puede cancelarse si el dispositivo lo admite").
pub fn argumentos_cancelar(device_path: &str) -> Vec<String> {
    vec!["-X".to_string(), device_path.to_string()]
}

/// Un dispositivo sin compatibilidad SMART no puede ofrecer autotest: "no se trata la falta de
/// compatibilidad como una anomalía" (§6) — mismo criterio que el resto de capacidades
/// (`commands::DeviceCapability`).
pub fn admite_autotest(smartctl_path: Option<&str>) -> bool {
    smartctl_path.is_some()
}

#[derive(Debug, Clone, PartialEq)]
pub struct EstadoAutotest {
    /// `None` mientras el autotest sigue en curso: ni pasó ni falló todavía.
    pub passed: Option<bool>,
    pub descripcion: Option<String>,
    pub minutos_estimados: Option<i64>,
}

/// Extrae el estado del autotest de la misma captura `smartctl -a -j` que ya usa
/// `smartctl_parser` — no es una consulta aparte, es releer el bloque `ata_smart_data.self_test`
/// del JSON que el ciclo normal de recopilación ya trae. **Sin verificar contra una salida real**
/// (J.28): la forma asumida es la documentada de smartmontools, no una captura propia.
pub fn parse_estado_json(json: &str) -> Option<EstadoAutotest> {
    let valor: serde_json::Value = serde_json::from_str(json).ok()?;
    let self_test = valor.get("ata_smart_data")?.get("self_test")?;

    let passed = self_test
        .get("status")
        .and_then(|s| s.get("passed"))
        .and_then(|v| v.as_bool());
    let descripcion = self_test
        .get("status")
        .and_then(|s| s.get("string"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let minutos_estimados = self_test
        .get("polling_minutes")
        .and_then(|p| p.get("short"))
        .and_then(|v| v.as_i64());

    Some(EstadoAutotest {
        passed,
        descripcion,
        minutos_estimados,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_argumentos_de_inicio_incluyen_el_modo_corto_y_json() {
        assert_eq!(
            argumentos_iniciar(r"\\.\PhysicalDrive0"),
            vec!["-t", "short", "-j", r"\\.\PhysicalDrive0"]
        );
    }

    #[test]
    fn los_argumentos_de_cancelacion_usan_mayuscula_x() {
        assert_eq!(
            argumentos_cancelar(r"\\.\PhysicalDrive0"),
            vec!["-X", r"\\.\PhysicalDrive0"]
        );
    }

    #[test]
    fn un_dispositivo_sin_ruta_smartctl_no_admite_autotest() {
        assert!(!admite_autotest(None));
    }

    #[test]
    fn un_dispositivo_con_ruta_smartctl_admite_autotest() {
        assert!(admite_autotest(Some(r"\\.\PhysicalDrive0")));
    }

    /// Fixture construida a partir de la forma documentada de smartmontools, **no capturada**
    /// (J.28): es lo que se comparará contra un JSON real antes de dar T081 por terminado.
    const AUTOTEST_COMPLETADO_JSON: &str = r#"{
        "ata_smart_data": {
            "self_test": {
                "status": { "value": 0, "string": "completed without error", "passed": true },
                "polling_minutes": { "short": 2, "extended": 128 }
            }
        }
    }"#;

    #[test]
    fn un_autotest_completado_se_parsea_como_superado() {
        let estado = parse_estado_json(AUTOTEST_COMPLETADO_JSON).unwrap();
        assert_eq!(estado.passed, Some(true));
        assert_eq!(
            estado.descripcion.as_deref(),
            Some("completed without error")
        );
        assert_eq!(estado.minutos_estimados, Some(2));
    }

    #[test]
    fn un_json_sin_bloque_de_autotest_no_produce_un_estado_inventado() {
        assert_eq!(parse_estado_json(r#"{"model_name": "X"}"#), None);
    }

    #[test]
    fn un_json_irreconocible_no_produce_un_estado_inventado() {
        assert_eq!(parse_estado_json("no es json"), None);
    }
}
