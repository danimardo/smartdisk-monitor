//! Capacidad de volumen (T059, FR de historia 3): letra, sistema de archivos, tamaño y espacio
//! libre, más el número de disco físico al que pertenece — necesario para enlazar con el
//! dispositivo ya reconciliado por `windows_storage::list_physical_disks`, cuyo índice es la misma
//! numeración de Windows (`DiskNumber` en `Get-Partition`, el mismo que `DeviceId` en
//! `Get-PhysicalDisk`).
//!
//! Un único `Get-Partition | Get-Volume` en PowerShell trae ambos datos juntos: separar la
//! consulta en dos y correlacionar en Rust por letra de unidad sería más frágil (la letra puede
//! faltar) sin ganar nada, ya que `Get-Partition` ya sabe a qué disco pertenece cada partición.
//! Mismo patrón que `windows_storage.rs`: proceso externo + `serde_json`, parseo puro y probado
//! con fixtures; la invocación en sí no se prueba en unidad.

use serde::Deserialize;
use serde_json::Value;

/// Lo que entrega la consulta combinada, tal cual — sin normalizar todavía.
#[derive(Debug, Clone, Deserialize, PartialEq)]
struct VolumenJson {
    #[serde(rename = "DiskNumber")]
    disk_number: Option<i64>,
    #[serde(rename = "DriveLetter")]
    drive_letter: Option<String>,
    #[serde(rename = "FileSystemLabel")]
    file_system_label: Option<String>,
    #[serde(rename = "FileSystem")]
    file_system: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
    #[serde(rename = "SizeRemaining")]
    size_remaining: Option<u64>,
    #[serde(rename = "UniqueId")]
    unique_id: Option<String>,
}

/// Volumen ya normalizado, listo para que `persistence::repo_inventario` lo guarde y lo enlace.
/// Sin `id` todavía: lo decide quien orquesta la recopilación (`identidad`, igual que un disco).
#[derive(Debug, Clone, PartialEq)]
pub struct VolumenLeido {
    /// `\\?\Volume{guid}\`: identidad estable del volumen entre reconexiones, a diferencia de la
    /// letra de unidad, que puede cambiar.
    pub volume_guid: String,
    pub label: Option<String>,
    pub filesystem: Option<String>,
    pub drive_letter: Option<char>,
    pub capacity_bytes: Option<i64>,
    pub free_bytes: Option<i64>,
    /// El disco físico al que pertenece, en la numeración efímera de Windows (`Get-PhysicalDisk`
    /// `DeviceId` / `Get-Partition` `DiskNumber`): la correlación con el `Device` estable la hace
    /// quien orquesta, en la misma pasada en que ya conoce esa numeración para los discos.
    pub disk_number: Option<i64>,
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

/// Igual que `Get-PhysicalDisk`: un solo resultado llega como objeto suelto, no como array de un
/// elemento.
pub fn parse_volumes_json(json: &str) -> Result<Vec<VolumenLeido>, ErrorParseo> {
    let valor: Value = serde_json::from_str(json)?;
    let crudos: Vec<VolumenJson> = match valor {
        Value::Array(_) => serde_json::from_value(valor)?,
        Value::Null => vec![],
        objeto => vec![serde_json::from_value(objeto)?],
    };

    Ok(crudos
        .into_iter()
        // Sin `UniqueId` no hay identidad estable que guardar: mejor omitir el volumen que
        // inventar una clave a partir de la letra, que es justo lo que no es estable.
        .filter_map(|v| {
            let volume_guid = v.unique_id?;
            Some(VolumenLeido {
                volume_guid,
                label: v.file_system_label.filter(|s| !s.trim().is_empty()),
                filesystem: v.file_system,
                drive_letter: v.drive_letter.and_then(|s| s.chars().next()),
                capacity_bytes: v.size.and_then(|s| i64::try_from(s).ok()),
                free_bytes: v.size_remaining.and_then(|s| i64::try_from(s).ok()),
                disk_number: v.disk_number,
            })
        })
        .collect())
}

/// Mismo límite y misma razón que `windows_storage::TIEMPO_MAXIMO_INVENTARIO` (J.57): un WMI lento
/// a arrancar no debe bloquear el hilo que llama para siempre.
#[cfg(windows)]
const TIEMPO_MAXIMO_INVENTARIO: std::time::Duration = std::time::Duration::from_secs(20);

#[cfg(windows)]
pub fn list_volumes() -> Result<Vec<VolumenLeido>, ErrorParseo> {
    use std::process::Command;

    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "Get-Partition | Where-Object DriveLetter | ForEach-Object { \
         $vol = $_ | Get-Volume; \
         [PSCustomObject]@{ \
           DiskNumber = $_.DiskNumber; DriveLetter = $_.DriveLetter; \
           FileSystemLabel = $vol.FileSystemLabel; FileSystem = $vol.FileSystem; \
           Size = $vol.Size; SizeRemaining = $vol.SizeRemaining; UniqueId = $vol.UniqueId \
         } \
         } | ConvertTo-Json",
    ]);
    let salida =
        crate::platform::proceso_externo::ejecutar_con_limite(cmd, TIEMPO_MAXIMO_INVENTARIO);

    let json = match salida {
        Ok(Some(o)) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => "null".to_string(),
    };

    parse_volumes_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UN_VOLUMEN: &str = r#"{
        "DiskNumber": 0,
        "DriveLetter": "C",
        "FileSystemLabel": "Sistema",
        "FileSystem": "NTFS",
        "Size": 2000398934016,
        "SizeRemaining": 1204000000000,
        "UniqueId": "\\\\?\\Volume{a1b2c3d4-0000-0000-0000-000000000000}\\"
    }"#;

    const DOS_VOLUMENES: &str = r#"[
        {"DiskNumber":0,"DriveLetter":"C","FileSystemLabel":"Sistema","FileSystem":"NTFS","Size":100,"SizeRemaining":50,"UniqueId":"u1"},
        {"DiskNumber":1,"DriveLetter":"D","FileSystemLabel":null,"FileSystem":"exFAT","Size":200,"SizeRemaining":150,"UniqueId":"u2"}
    ]"#;

    #[test]
    fn un_solo_volumen_no_envuelto_en_array_se_parsea_igual() {
        let volumenes = parse_volumes_json(UN_VOLUMEN).unwrap();
        assert_eq!(volumenes.len(), 1);
        assert_eq!(volumenes[0].disk_number, Some(0));
        assert_eq!(volumenes[0].drive_letter, Some('C'));
        assert_eq!(volumenes[0].label.as_deref(), Some("Sistema"));
        assert_eq!(volumenes[0].capacity_bytes, Some(2000398934016));
        assert_eq!(volumenes[0].free_bytes, Some(1204000000000));
    }

    #[test]
    fn dos_volumenes_en_array_se_parsean_todos() {
        let volumenes = parse_volumes_json(DOS_VOLUMENES).unwrap();
        assert_eq!(volumenes.len(), 2);
        assert_eq!(volumenes[1].drive_letter, Some('D'));
        assert_eq!(
            volumenes[1].label, None,
            "etiqueta ausente, no cadena vacía"
        );
    }

    #[test]
    fn ausencia_total_de_volumenes_no_es_un_error() {
        assert!(parse_volumes_json("null").unwrap().is_empty());
    }

    #[test]
    fn un_volumen_sin_unique_id_se_omite_en_vez_de_inventar_una_clave() {
        let volumenes =
            parse_volumes_json(r#"{"DiskNumber":0,"DriveLetter":"E","Size":10,"SizeRemaining":5}"#)
                .unwrap();
        assert!(volumenes.is_empty());
    }

    #[test]
    fn una_etiqueta_en_blanco_se_trata_como_ausente() {
        let volumenes = parse_volumes_json(
            r#"{"DiskNumber":0,"DriveLetter":"C","FileSystemLabel":"   ","UniqueId":"u1"}"#,
        )
        .unwrap();
        assert_eq!(volumenes[0].label, None);
    }
}
