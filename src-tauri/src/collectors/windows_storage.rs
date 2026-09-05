//! Inventario de almacenamiento de Windows: discos físicos, particiones y volúmenes (FR-001).
//!
//! Se consulta con los cmdlets de Storage vía PowerShell (`Get-PhysicalDisk`, `Get-Partition`,
//! `Get-Volume`) con salida `ConvertTo-Json`, siguiendo el mismo patrón que ya usa `smartctl`:
//! proceso externo + `serde_json`, sin depender de un binding COM/WMI que ampliaría la superficie
//! de un binario privilegiado sin necesidad (`docs/architecture.md` §33 permite WMI/CIM "según
//! disponibilidad"; esto la satisface sin dependencia nueva). El parseo es puro y se prueba con
//! fixtures reales; la invocación en sí no se prueba en unidad, igual que `platform::accent::read`.

use serde::Deserialize;
use serde_json::Value;

use crate::domain::tipos::DeviceType;

/// Lo que entrega `Get-PhysicalDisk`, tal cual llega del sistema — sin normalizar todavía.
/// Toda fuente externa se deserializa en un tipo explícito (`.claude/rules/backend-rust.md`).
#[derive(Debug, Clone, Deserialize, PartialEq)]
struct DiscoFisicoJson {
    #[serde(rename = "FriendlyName")]
    friendly_name: Option<String>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "Model")]
    model: Option<String>,
    #[serde(rename = "SerialNumber")]
    serial_number: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
    #[serde(rename = "BusType")]
    bus_type: Option<String>,
    #[serde(rename = "MediaType")]
    media_type: Option<String>,
    #[serde(rename = "DeviceId")]
    device_id: Option<String>,
    #[serde(rename = "UniqueId")]
    unique_id: Option<String>,
    #[serde(rename = "FirmwareVersion")]
    firmware_version: Option<String>,
}

/// Disco físico ya normalizado, listo para que `domain::identidad` calcule su huella y
/// `persistence::repo_inventario` lo guarde. Todavía no tiene `id` ni fechas: esas las decide
/// quien orquesta la recopilación, no el colector.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoFisico {
    pub model: String,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub firmware: Option<String>,
    pub capacity_bytes: Option<i64>,
    pub bus_type: String,
    pub device_type: DeviceType,
    /// WWN o `PNPDeviceID`: lo que `domain::identidad::compute_fingerprint` necesita cuando no
    /// hay número de serie. `UniqueId` es lo más parecido que expone `Get-PhysicalDisk`.
    pub wwn_o_pnp_device_id: String,
    /// Ruta que `smartctl` necesita para hablar con el disco (`/dev/pdN`, la convención propia de
    /// esta compilación MinGW — no la ruta nativa de Windows `\\.\PhysicalDriveN`, que
    /// `smartctl -h` no reconoce y falla con "Unable to detect device type" en todos los modos,
    /// medido contra hardware real, `open-questions.md` J.42), construida a partir del índice
    /// numérico que Windows asigna. **No** es la misma noción que `wwn_o_pnp_device_id`: ese
    /// identifica el disco de forma estable entre reconexiones, este solo dice dónde está montado
    /// *ahora* y puede cambiar entre arranques.
    pub smartctl_device_path: Option<String>,
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

/// Bus + tipo de medio → nuestro `DeviceType`. Vive aquí porque depende de las cadenas concretas
/// que `Get-PhysicalDisk` devuelve, no de nada del dominio.
fn clasificar(bus_type: &str, media_type: Option<&str>) -> DeviceType {
    let bus = bus_type.to_ascii_lowercase();
    let medio = media_type.unwrap_or("").to_ascii_lowercase();

    if bus.contains("nvme") {
        DeviceType::Nvme
    } else if bus.contains("usb") {
        DeviceType::Usb
    } else if bus.contains("raid") {
        DeviceType::RaidLogical
    } else if bus.contains("file backed virtual") || bus.contains("virtual") {
        DeviceType::Virtual
    } else if bus.contains("sata") || bus.contains("ata") || bus.contains("sas") {
        if medio.contains("ssd") {
            DeviceType::SataSsd
        } else if medio.contains("hdd") {
            DeviceType::Hdd
        } else {
            DeviceType::Unknown
        }
    } else {
        DeviceType::Unknown
    }
}

/// `Get-PhysicalDisk | ConvertTo-Json` puede devolver un objeto suelto cuando solo hay un disco
/// —PowerShell no envuelve en array un resultado de un elemento— o un array cuando hay varios.
/// Se maneja explícitamente en vez de forzar `-AsArray`, que no existe en Windows PowerShell 5.1.
pub fn parse_physical_disks_json(json: &str) -> Result<Vec<DiscoFisico>, ErrorParseo> {
    let valor: Value = serde_json::from_str(json)?;
    let crudos: Vec<DiscoFisicoJson> = match valor {
        Value::Array(_) => serde_json::from_value(valor)?,
        Value::Null => vec![],
        objeto => vec![serde_json::from_value(objeto)?],
    };

    Ok(crudos
        .into_iter()
        .map(|c| {
            let bus_type = c.bus_type.unwrap_or_else(|| "Unknown".to_string());
            let device_type = clasificar(&bus_type, c.media_type.as_deref());
            let smartctl_device_path = c
                .device_id
                .as_deref()
                .and_then(|id| id.trim().parse::<u32>().ok())
                .map(|n| format!("/dev/pd{n}"));
            DiscoFisico {
                model: c
                    .model
                    .or(c.friendly_name)
                    .unwrap_or_else(|| "Desconocido".to_string()),
                manufacturer: c.manufacturer,
                serial_number: c.serial_number.filter(|s| !s.trim().is_empty()),
                firmware: c.firmware_version,
                capacity_bytes: c.size.and_then(|s| i64::try_from(s).ok()),
                bus_type,
                device_type,
                wwn_o_pnp_device_id: c.unique_id.or(c.device_id).unwrap_or_default(),
                smartctl_device_path,
            }
        })
        .collect())
}

#[cfg(windows)]
pub fn list_physical_disks() -> Result<Vec<DiscoFisico>, ErrorParseo> {
    use std::process::Command;

    let salida = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-PhysicalDisk | Select-Object FriendlyName,Manufacturer,Model,SerialNumber,Size,BusType,MediaType,DeviceId,UniqueId,FirmwareVersion | ConvertTo-Json",
        ])
        .output();

    let json = match salida {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => "null".to_string(),
    };

    parse_physical_disks_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UN_DISCO: &str = r#"{
        "FriendlyName": "SAMSUNG MZVL21T0",
        "Manufacturer": "SAMSUNG",
        "Model": "SAMSUNG MZVL21T0",
        "SerialNumber": "S6B2NS0T123456",
        "Size": 1024209543168,
        "BusType": "NVMe",
        "MediaType": "SSD",
        "DeviceId": "0",
        "UniqueId": "eui.0025388801234567",
        "FirmwareVersion": "GXA7801Q"
    }"#;

    const DOS_DISCOS: &str = r#"[
        {"FriendlyName":"A","Model":"A","SerialNumber":"S1","Size":500000000000,"BusType":"SATA","MediaType":"SSD","DeviceId":"0","UniqueId":"u1","FirmwareVersion":"1.0"},
        {"FriendlyName":"B","Model":"B","SerialNumber":null,"Size":2000000000000,"BusType":"USB","MediaType":"Unspecified","DeviceId":"1","UniqueId":"u2","FirmwareVersion":null}
    ]"#;

    #[test]
    fn un_solo_disco_no_envuelto_en_array_se_parsea_igual() {
        let discos = parse_physical_disks_json(UN_DISCO).unwrap();
        assert_eq!(discos.len(), 1);
        assert_eq!(discos[0].model, "SAMSUNG MZVL21T0");
        assert_eq!(discos[0].device_type, DeviceType::Nvme);
        assert_eq!(discos[0].capacity_bytes, Some(1024209543168));
        assert_eq!(discos[0].serial_number.as_deref(), Some("S6B2NS0T123456"));
    }

    #[test]
    fn el_deviceid_numerico_se_convierte_en_ruta_de_smartctl() {
        let discos = parse_physical_disks_json(UN_DISCO).unwrap();
        assert_eq!(discos[0].smartctl_device_path.as_deref(), Some("/dev/pd0"));
    }

    #[test]
    fn dos_discos_reciben_rutas_distintas_segun_su_indice() {
        let discos = parse_physical_disks_json(DOS_DISCOS).unwrap();
        assert_eq!(discos[0].smartctl_device_path.as_deref(), Some("/dev/pd0"));
        assert_eq!(discos[1].smartctl_device_path.as_deref(), Some("/dev/pd1"));
    }

    #[test]
    fn un_deviceid_no_numerico_no_produce_una_ruta_inventada() {
        let discos = parse_physical_disks_json(
            r#"{"Model":"X","BusType":"RAID","DeviceId":"\\\\.\\RAIDVolume1"}"#,
        )
        .unwrap();
        assert_eq!(discos[0].smartctl_device_path, None);
    }

    #[test]
    fn varios_discos_en_array_se_parsean_todos() {
        let discos = parse_physical_disks_json(DOS_DISCOS).unwrap();
        assert_eq!(discos.len(), 2);
        assert_eq!(discos[0].device_type, DeviceType::SataSsd);
        assert_eq!(discos[1].device_type, DeviceType::Usb);
    }

    #[test]
    fn sin_numero_de_serie_queda_en_none_no_en_cadena_vacia() {
        let discos = parse_physical_disks_json(DOS_DISCOS).unwrap();
        assert_eq!(discos[1].serial_number, None);
    }

    #[test]
    fn ausencia_total_de_discos_no_es_un_error() {
        let discos = parse_physical_disks_json("null").unwrap();
        assert!(discos.is_empty());
    }

    #[test]
    fn sata_sin_media_type_reconocible_es_unknown_no_hdd_por_defecto() {
        let device_type = clasificar("SATA", Some("Unspecified"));
        assert_eq!(device_type, DeviceType::Unknown);
    }

    #[test]
    fn hdd_sata_se_reconoce() {
        assert_eq!(clasificar("SATA", Some("HDD")), DeviceType::Hdd);
    }

    #[test]
    fn raid_y_virtual_se_reconocen() {
        assert_eq!(clasificar("RAID", None), DeviceType::RaidLogical);
        assert_eq!(clasificar("File Backed Virtual", None), DeviceType::Virtual);
    }
}
