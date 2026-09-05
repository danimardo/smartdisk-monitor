//! Parser de la salida JSON de `smartctl -j` (constitución §VIII: test-first, con salidas reales).
//!
//! **Nota sobre las fixtures.** No hay hardware disponible en este entorno de desarrollo para
//! capturar una salida real. Las fixtures de las pruebas siguen el esquema documentado de
//! smartctl 7.5 (`third-party/smartmontools/`, versión que fija ADR-021) campo por campo, pero no
//! sustituyen a una verificación contra un disco real antes de publicar (`docs/testing-strategy.md`
//! §quickstart). Cuando exista esa verificación, estas fixtures deben sustituirse o contrastarse.
//!
//! Se leen los campos **de nivel superior** (`temperature.current`, `power_on_time.hours`,
//! `power_cycle_count`, `smart_status.passed`) en vez de las secciones específicas de cada tipo de
//! dispositivo: `smartctl` ya los normaliza a Celsius y a un conteo simple para ATA y NVMe por
//! igual, y evita un error clásico de esta clase de parser — el log NVMe crudo guarda la
//! temperatura en **Kelvin**, no en Celsius.

use serde::Deserialize;
use serde_json::Value;

/// Una métrica ya en la unidad del modelo de datos (`docs/data-model.md` §3): sin ella no hay
/// medición que mostrar, así que el campo se omite en vez de mandar un cero.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricaLeida {
    pub metric_key: &'static str,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmartctlResult {
    pub exit_status: i32,
    pub smartctl_version: Option<String>,
    /// `"nvme"`, `"sat"`, `"ata"`… tal cual lo informa `device.type`.
    pub device_type: Option<String>,
    pub model_name: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub user_capacity_bytes: Option<i64>,
    /// Autoevaluación SMART global (`smart_status.passed`). `None` cuando el dispositivo no la
    /// expone, que no es lo mismo que haberla fallado.
    pub health_passed: Option<bool>,
    pub metrics: Vec<MetricaLeida>,
}

#[derive(Debug)]
pub enum ErrorParseo {
    Json(serde_json::Error),
}

impl From<serde_json::Error> for ErrorParseo {
    fn from(e: serde_json::Error) -> Self {
        ErrorParseo::Json(e)
    }
}

#[derive(Debug, Deserialize)]
struct RaizJson {
    smartctl: Option<SmartctlInfoJson>,
    device: Option<DeviceJson>,
    model_name: Option<String>,
    serial_number: Option<String>,
    firmware_version: Option<String>,
    user_capacity: Option<CapacityJson>,
    temperature: Option<TemperatureJson>,
    power_on_time: Option<PowerOnTimeJson>,
    power_cycle_count: Option<i64>,
    smart_status: Option<SmartStatusJson>,
    nvme_smart_health_information_log: Option<NvmeLogJson>,
}

#[derive(Debug, Deserialize)]
struct SmartctlInfoJson {
    version: Option<Vec<i64>>,
    exit_status: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct DeviceJson {
    #[serde(rename = "type")]
    tipo: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CapacityJson {
    bytes: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TemperatureJson {
    current: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct PowerOnTimeJson {
    hours: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SmartStatusJson {
    passed: Option<bool>,
}

/// Solo los campos NVMe que **no** tienen equivalente de nivel superior: quedan `None` en
/// dispositivos ATA, y eso es correcto — un SATA no tiene desgaste de celdas NAND que reportar.
#[derive(Debug, Deserialize)]
struct NvmeLogJson {
    critical_warning: Option<i64>,
    available_spare: Option<f64>,
    available_spare_threshold: Option<f64>,
    percentage_used: Option<f64>,
    unsafe_shutdowns: Option<i64>,
    media_errors: Option<i64>,
    num_err_log_entries: Option<i64>,
    /// Unidades de 512.000 bytes cada una (especificación NVMe): la conversión a bytes vive aquí,
    /// no en el dominio, porque es un detalle del formato del log, no del modelo de datos.
    data_units_read: Option<i64>,
    data_units_written: Option<i64>,
}

const UNIDAD_NVME_BYTES: i64 = 512_000;

pub fn parse_smartctl_json(json: &str) -> Result<SmartctlResult, ErrorParseo> {
    let valor: Value = serde_json::from_str(json)?;
    let raiz: RaizJson = serde_json::from_value(valor)?;

    let mut metrics = Vec::new();
    let mut agregar = |metric_key: &'static str, value: Option<f64>| {
        if let Some(v) = value {
            metrics.push(MetricaLeida {
                metric_key,
                value: v,
            });
        }
    };

    agregar(
        "temperature_celsius",
        raiz.temperature.as_ref().and_then(|t| t.current),
    );
    agregar(
        "power_on_hours",
        raiz.power_on_time.as_ref().and_then(|p| p.hours),
    );
    agregar("power_cycles", raiz.power_cycle_count.map(|v| v as f64));
    // `health_passed` como métrica 1.0/0.0 (`docs/data-model.md` §3), no solo como campo suelto:
    // el motor de alertas (`smart.health.failed`) necesita poder consultarla como cualquier otra
    // serie, incluida su ausencia cuando el dispositivo no expone esta autoevaluación.
    agregar(
        "health_passed",
        raiz.smart_status
            .as_ref()
            .and_then(|s| s.passed)
            .map(|passed| if passed { 1.0 } else { 0.0 }),
    );

    if let Some(log) = &raiz.nvme_smart_health_information_log {
        agregar("critical_warning", log.critical_warning.map(|v| v as f64));
        agregar("available_spare_percent", log.available_spare);
        agregar(
            "available_spare_threshold_percent",
            log.available_spare_threshold,
        );
        agregar("percentage_used", log.percentage_used);
        agregar("unsafe_shutdowns", log.unsafe_shutdowns.map(|v| v as f64));
        agregar("media_errors_total", log.media_errors.map(|v| v as f64));
        agregar(
            "error_log_entries_total",
            log.num_err_log_entries.map(|v| v as f64),
        );
        agregar(
            "data_read_bytes_total",
            log.data_units_read.map(|u| (u * UNIDAD_NVME_BYTES) as f64),
        );
        agregar(
            "data_written_bytes_total",
            log.data_units_written
                .map(|u| (u * UNIDAD_NVME_BYTES) as f64),
        );
    }

    Ok(SmartctlResult {
        exit_status: raiz
            .smartctl
            .as_ref()
            .and_then(|s| s.exit_status)
            .unwrap_or(0),
        smartctl_version: raiz
            .smartctl
            .as_ref()
            .and_then(|s| s.version.as_ref())
            .map(|v| v.iter().map(i64::to_string).collect::<Vec<_>>().join(".")),
        device_type: raiz.device.and_then(|d| d.tipo),
        model_name: raiz.model_name,
        serial_number: raiz.serial_number,
        firmware_version: raiz.firmware_version,
        user_capacity_bytes: raiz.user_capacity.and_then(|c| c.bytes),
        health_passed: raiz.smart_status.and_then(|s| s.passed),
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Salida representativa de un NVMe (esquema smartctl 7.5). Ver la nota del módulo: no
    /// capturada de hardware real en este entorno.
    const SALIDA_NVME: &str = r#"{
        "json_format_version": [1, 0],
        "smartctl": { "version": [7, 5], "exit_status": 0 },
        "device": { "name": "/dev/nvme0", "type": "nvme" },
        "model_name": "SAMSUNG MZVL21T0",
        "serial_number": "S6B2NS0T123456",
        "firmware_version": "GXA7801Q",
        "user_capacity": { "bytes": 1024209543168 },
        "temperature": { "current": 42 },
        "power_on_time": { "hours": 1234 },
        "power_cycle_count": 56,
        "smart_status": { "passed": true },
        "nvme_smart_health_information_log": {
            "critical_warning": 0,
            "temperature": 315,
            "available_spare": 100,
            "available_spare_threshold": 10,
            "percentage_used": 5,
            "data_units_read": 1000,
            "data_units_written": 2000,
            "power_cycles": 56,
            "power_on_hours": 1234,
            "unsafe_shutdowns": 3,
            "media_errors": 0,
            "num_err_log_entries": 2
        }
    }"#;

    /// Salida representativa de un SATA/ATA: sin sección NVMe, con `ata_smart_attributes` que este
    /// parser no necesita porque usa los campos normalizados de nivel superior.
    const SALIDA_ATA: &str = r#"{
        "smartctl": { "version": [7, 5], "exit_status": 0 },
        "device": { "name": "/dev/sda", "type": "sat" },
        "model_name": "WDC WD40EFAX-68JH4N1",
        "serial_number": "WD-WX12A34B5678",
        "firmware_version": "83.00A83",
        "user_capacity": { "bytes": 4000787030016 },
        "temperature": { "current": 31 },
        "power_on_time": { "hours": 8760 },
        "power_cycle_count": 120,
        "smart_status": { "passed": true }
    }"#;

    /// Dispositivo no compatible: `smartctl` devuelve un `exit_status` con el bit de "no se pudo
    /// abrir el dispositivo" y prácticamente ningún campo. Debe leerse como ausencia, no como cero.
    const SALIDA_NO_COMPATIBLE: &str = r#"{
        "smartctl": { "version": [7, 5], "exit_status": 2 },
        "device": { "type": "unknown" }
    }"#;

    #[test]
    fn nvme_extrae_las_metricas_propias_y_las_de_nivel_superior() {
        let r = parse_smartctl_json(SALIDA_NVME).unwrap();
        assert_eq!(r.exit_status, 0);
        assert_eq!(r.device_type.as_deref(), Some("nvme"));
        assert_eq!(r.health_passed, Some(true));

        let buscar = |clave: &str| {
            r.metrics
                .iter()
                .find(|m| m.metric_key == clave)
                .map(|m| m.value)
        };
        assert_eq!(buscar("temperature_celsius"), Some(42.0));
        assert_eq!(buscar("power_on_hours"), Some(1234.0));
        assert_eq!(buscar("critical_warning"), Some(0.0));
        assert_eq!(buscar("available_spare_percent"), Some(100.0));
        assert_eq!(buscar("percentage_used"), Some(5.0));
        assert_eq!(buscar("data_read_bytes_total"), Some(1000.0 * 512_000.0));
        assert_eq!(buscar("data_written_bytes_total"), Some(2000.0 * 512_000.0));
        assert_eq!(
            buscar("health_passed"),
            Some(1.0),
            "también persiste como métrica, no solo como campo suelto"
        );
    }

    #[test]
    fn ata_no_inventa_metricas_nvme_que_no_tiene() {
        let r = parse_smartctl_json(SALIDA_ATA).unwrap();
        assert_eq!(r.device_type.as_deref(), Some("sat"));

        let buscar = |clave: &str| r.metrics.iter().find(|m| m.metric_key == clave);
        assert!(buscar("temperature_celsius").is_some());
        assert!(buscar("power_on_hours").is_some());
        assert!(
            buscar("critical_warning").is_none(),
            "un SATA no expone critical_warning: no se inventa un valor"
        );
        assert!(buscar("percentage_used").is_none());
        assert!(buscar("media_errors_total").is_none());
    }

    #[test]
    fn una_autoevaluacion_fallida_se_persiste_como_cero_no_como_ausente() {
        let salida = r#"{
            "smartctl": { "version": [7, 5], "exit_status": 4 },
            "device": { "type": "nvme" },
            "smart_status": { "passed": false }
        }"#;
        let r = parse_smartctl_json(salida).unwrap();
        assert_eq!(r.health_passed, Some(false));

        let metrica = r
            .metrics
            .iter()
            .find(|m| m.metric_key == "health_passed")
            .unwrap();
        assert_eq!(
            metrica.value, 0.0,
            "un fallo real es 0.0, no la ausencia de la métrica"
        );
    }

    #[test]
    fn dispositivo_no_compatible_no_produce_metricas_falsas() {
        let r = parse_smartctl_json(SALIDA_NO_COMPATIBLE).unwrap();
        assert_eq!(r.exit_status, 2);
        assert!(
            r.metrics.is_empty(),
            "sin datos, no debe haber ni una sola métrica inventada"
        );
        assert_eq!(r.health_passed, None);
        assert_eq!(r.model_name, None);
    }

    #[test]
    fn la_temperatura_nvme_se_lee_del_campo_ya_convertido_a_celsius_no_del_log_crudo_en_kelvin() {
        // El log crudo trae 315 (Kelvin, ~42°C); el campo de nivel superior ya trae 42 (Celsius).
        // Si el parser leyera el log crudo sin convertir, esta prueba fallaría con 315.0.
        let r = parse_smartctl_json(SALIDA_NVME).unwrap();
        let temp = r
            .metrics
            .iter()
            .find(|m| m.metric_key == "temperature_celsius")
            .unwrap();
        assert_eq!(temp.value, 42.0);
    }

    #[test]
    fn version_de_smartctl_se_compone_como_texto_con_puntos() {
        let r = parse_smartctl_json(SALIDA_NVME).unwrap();
        assert_eq!(r.smartctl_version.as_deref(), Some("7.5"));
    }
}
