//! Repositorio de `devices`, `volumes` y `device_volume_links` (`docs/data-model.md` §2).

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::tipos::{Device, DeviceType, IdentityConfidence, MappingConfidence, Volume};

fn identity_confidence_to_str(v: IdentityConfidence) -> &'static str {
    match v {
        IdentityConfidence::Serial => "serial",
        IdentityConfidence::Fingerprint => "fingerprint",
    }
}

fn identity_confidence_from_str(s: &str) -> IdentityConfidence {
    match s {
        "serial" => IdentityConfidence::Serial,
        _ => IdentityConfidence::Fingerprint,
    }
}

fn device_type_to_str(v: DeviceType) -> &'static str {
    match v {
        DeviceType::Nvme => "nvme",
        DeviceType::SataSsd => "sata_ssd",
        DeviceType::Hdd => "hdd",
        DeviceType::Usb => "usb",
        DeviceType::Virtual => "virtual",
        DeviceType::RaidLogical => "raid_logical",
        DeviceType::Unknown => "unknown",
    }
}

fn device_type_from_str(s: &str) -> DeviceType {
    match s {
        "nvme" => DeviceType::Nvme,
        "sata_ssd" => DeviceType::SataSsd,
        "hdd" => DeviceType::Hdd,
        "usb" => DeviceType::Usb,
        "virtual" => DeviceType::Virtual,
        "raid_logical" => DeviceType::RaidLogical,
        _ => DeviceType::Unknown,
    }
}

fn mapping_confidence_to_str(v: Option<MappingConfidence>) -> Option<&'static str> {
    v.map(|v| match v {
        MappingConfidence::Exact => "exact",
        MappingConfidence::Inferred => "inferred",
        MappingConfidence::Unknown => "unknown",
    })
}

fn mapping_confidence_from_str(s: Option<String>) -> Option<MappingConfidence> {
    s.map(|s| match s.as_str() {
        "exact" => MappingConfidence::Exact,
        "inferred" => MappingConfidence::Inferred,
        _ => MappingConfidence::Unknown,
    })
}

fn row_to_device(row: &rusqlite::Row) -> rusqlite::Result<Device> {
    Ok(Device {
        id: row.get("id")?,
        fingerprint: row.get("fingerprint")?,
        identity_confidence: identity_confidence_from_str(
            &row.get::<_, String>("identity_confidence")?,
        ),
        serial_number: row.get("serial_number")?,
        model: row.get("model")?,
        manufacturer: row.get("manufacturer")?,
        firmware: row.get("firmware")?,
        device_type: device_type_from_str(&row.get::<_, String>("device_type")?),
        bus_type: row.get("bus_type")?,
        smartctl_path: row.get("smartctl_path")?,
        capacity_bytes: row.get("capacity_bytes")?,
        alias: row.get("alias")?,
        monitoring_enabled: row.get::<_, i64>("monitoring_enabled")? != 0,
        first_seen_at: row.get("first_seen_at")?,
        last_seen_at: row.get("last_seen_at")?,
        removed_at: row.get("removed_at")?,
        capabilities_json: row.get("capabilities_json")?,
    })
}

/// Inserta un dispositivo nuevo, o actualiza el existente con la misma huella (alta/baja en
/// caliente: el mismo `fingerprint` que reaparece continúa su historial, FR-007).
pub fn upsert_device(conn: &Connection, d: &Device) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO devices (
            id, fingerprint, identity_confidence, serial_number, model, manufacturer, firmware,
            device_type, bus_type, smartctl_path, capacity_bytes, alias, monitoring_enabled,
            first_seen_at, last_seen_at, removed_at, capabilities_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
        ON CONFLICT(fingerprint) DO UPDATE SET
            serial_number = excluded.serial_number,
            model = excluded.model,
            manufacturer = excluded.manufacturer,
            firmware = excluded.firmware,
            device_type = excluded.device_type,
            bus_type = excluded.bus_type,
            smartctl_path = excluded.smartctl_path,
            capacity_bytes = excluded.capacity_bytes,
            monitoring_enabled = excluded.monitoring_enabled,
            last_seen_at = excluded.last_seen_at,
            removed_at = excluded.removed_at,
            capabilities_json = excluded.capabilities_json",
        params![
            d.id,
            d.fingerprint,
            identity_confidence_to_str(d.identity_confidence),
            d.serial_number,
            d.model,
            d.manufacturer,
            d.firmware,
            device_type_to_str(d.device_type),
            d.bus_type,
            d.smartctl_path,
            d.capacity_bytes,
            d.alias,
            d.monitoring_enabled as i64,
            d.first_seen_at,
            d.last_seen_at,
            d.removed_at,
            d.capabilities_json,
        ],
    )?;
    Ok(())
}

pub fn get_device(conn: &Connection, id: &str) -> rusqlite::Result<Option<Device>> {
    conn.query_row(
        "SELECT * FROM devices WHERE id = ?1",
        params![id],
        row_to_device,
    )
    .optional()
}

pub fn get_device_by_fingerprint(
    conn: &Connection,
    fingerprint: &str,
) -> rusqlite::Result<Option<Device>> {
    conn.query_row(
        "SELECT * FROM devices WHERE fingerprint = ?1",
        params![fingerprint],
        row_to_device,
    )
    .optional()
}

/// Todos los dispositivos que no se han retirado, monitorizados o no —la interfaz decide cómo se
/// listan; el repositorio no oculta nada.
pub fn list_present_devices(conn: &Connection) -> rusqlite::Result<Vec<Device>> {
    let mut stmt =
        conn.prepare("SELECT * FROM devices WHERE removed_at IS NULL ORDER BY first_seen_at")?;
    let filas = stmt.query_map([], row_to_device)?;
    filas.collect()
}

pub fn mark_device_removed(
    conn: &Connection,
    id: &str,
    removed_at_utc: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE devices SET removed_at = ?2 WHERE id = ?1",
        params![id, removed_at_utc],
    )?;
    Ok(())
}

pub fn set_monitoring(conn: &Connection, id: &str, enabled: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE devices SET monitoring_enabled = ?2 WHERE id = ?1",
        params![id, enabled as i64],
    )?;
    Ok(())
}

pub fn set_alias(conn: &Connection, id: &str, alias: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE devices SET alias = ?2 WHERE id = ?1",
        params![id, alias],
    )?;
    Ok(())
}

fn row_to_volume(row: &rusqlite::Row) -> rusqlite::Result<Volume> {
    Ok(Volume {
        id: row.get("id")?,
        volume_guid: row.get("volume_guid")?,
        label: row.get("label")?,
        filesystem: row.get("filesystem")?,
        drive_letters_json: row.get("drive_letters_json")?,
        capacity_bytes: row.get("capacity_bytes")?,
        free_bytes: row.get("free_bytes")?,
        device_mapping_confidence: mapping_confidence_from_str(
            row.get::<_, Option<String>>("device_mapping_confidence")?,
        ),
        first_seen_at: row.get("first_seen_at")?,
        last_seen_at: row.get("last_seen_at")?,
    })
}

pub fn upsert_volume(conn: &Connection, v: &Volume) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO volumes (
            id, volume_guid, label, filesystem, drive_letters_json, capacity_bytes, free_bytes,
            device_mapping_confidence, first_seen_at, last_seen_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(volume_guid) DO UPDATE SET
            label = excluded.label,
            filesystem = excluded.filesystem,
            drive_letters_json = excluded.drive_letters_json,
            capacity_bytes = excluded.capacity_bytes,
            free_bytes = excluded.free_bytes,
            device_mapping_confidence = excluded.device_mapping_confidence,
            last_seen_at = excluded.last_seen_at",
        params![
            v.id,
            v.volume_guid,
            v.label,
            v.filesystem,
            v.drive_letters_json,
            v.capacity_bytes,
            v.free_bytes,
            mapping_confidence_to_str(v.device_mapping_confidence),
            v.first_seen_at,
            v.last_seen_at,
        ],
    )?;
    Ok(())
}

pub fn get_volume(conn: &Connection, id: &str) -> rusqlite::Result<Option<Volume>> {
    conn.query_row(
        "SELECT * FROM volumes WHERE id = ?1",
        params![id],
        row_to_volume,
    )
    .optional()
}

pub fn list_volumes(conn: &Connection) -> rusqlite::Result<Vec<Volume>> {
    let mut stmt = conn.prepare("SELECT * FROM volumes ORDER BY first_seen_at")?;
    let filas = stmt.query_map([], row_to_volume)?;
    filas.collect()
}

/// Enlaza un disco con un volumen. Admite múltiples discos por volumen y viceversa
/// (`docs/data-model.md`: no se asume que una letra identifique un disco).
pub fn link_device_volume(
    conn: &Connection,
    device_id: &str,
    volume_id: &str,
    confidence: MappingConfidence,
    source: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO device_volume_links (device_id, volume_id, confidence, source)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(device_id, volume_id) DO UPDATE SET confidence = excluded.confidence, source = excluded.source",
        params![
            device_id,
            volume_id,
            mapping_confidence_to_str(Some(confidence)),
            source
        ],
    )?;
    Ok(())
}

pub fn volumes_for_device(conn: &Connection, device_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT volume_id FROM device_volume_links WHERE device_id = ?1")?;
    let filas = stmt.query_map(params![device_id], |r| r.get::<_, String>(0))?;
    filas.collect()
}

/// Inverso de `volumes_for_device`: qué discos respaldan un volumen (`docs/open-questions.md`
/// J.29, exclusión mutua de pruebas por disco físico subyacente).
pub fn devices_for_volume(conn: &Connection, volume_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT device_id FROM device_volume_links WHERE volume_id = ?1")?;
    let filas = stmt.query_map(params![volume_id], |r| r.get::<_, String>(0))?;
    filas.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::db;

    fn conn_de_prueba() -> Connection {
        let dir = crate::test_util::temp_dir_unico("repo_inventario");
        let (conn, _) = db::open(&dir).unwrap();
        conn
    }

    fn dispositivo_de_prueba(id: &str, fingerprint: &str) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: fingerprint.to_string(),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: None,
            model: "Modelo X".to_string(),
            manufacturer: Some("Fabricante".to_string()),
            firmware: Some("1.0".to_string()),
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: None,
            capacity_bytes: Some(1_000_000_000),
            alias: None,
            monitoring_enabled: true,
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    #[test]
    fn inserta_y_recupera_un_dispositivo() {
        let conn = conn_de_prueba();
        let d = dispositivo_de_prueba("d1", "huella-1");
        upsert_device(&conn, &d).unwrap();

        let recuperado = get_device(&conn, "d1").unwrap().unwrap();
        assert_eq!(recuperado, d);
    }

    #[test]
    fn la_misma_huella_continua_el_historial_en_vez_de_duplicar() {
        let conn = conn_de_prueba();
        let mut d = dispositivo_de_prueba("d1", "huella-estable");
        upsert_device(&conn, &d).unwrap();

        // El disco se retira y vuelve a conectarse: mismo fingerprint, distinto id lógico de alta.
        d.last_seen_at = "2026-09-05T00:00:00Z".to_string();
        d.alias = None; // alias no se toca por upsert de recopilación
        upsert_device(&conn, &d).unwrap();

        let todos = list_present_devices(&conn).unwrap();
        assert_eq!(
            todos.len(),
            1,
            "no debe crear una segunda entidad para la misma huella"
        );
        assert_eq!(todos[0].last_seen_at, "2026-09-05T00:00:00Z");
    }

    #[test]
    fn marcar_retirado_lo_saca_de_los_presentes() {
        let conn = conn_de_prueba();
        let d = dispositivo_de_prueba("d1", "huella-2");
        upsert_device(&conn, &d).unwrap();

        mark_device_removed(&conn, "d1", "2026-09-04T12:00:00Z").unwrap();

        assert!(list_present_devices(&conn).unwrap().is_empty());
        assert!(get_device(&conn, "d1")
            .unwrap()
            .unwrap()
            .removed_at
            .is_some());
    }

    #[test]
    fn enlaza_volumen_y_dispositivo() {
        let conn = conn_de_prueba();
        let d = dispositivo_de_prueba("d1", "huella-3");
        upsert_device(&conn, &d).unwrap();

        let v = Volume {
            id: "v1".to_string(),
            volume_guid: "guid-1".to_string(),
            label: Some("Datos".to_string()),
            filesystem: Some("NTFS".to_string()),
            drive_letters_json: Some(r#"["D:"]"#.to_string()),
            capacity_bytes: Some(500_000_000),
            free_bytes: Some(100_000_000),
            device_mapping_confidence: Some(MappingConfidence::Exact),
            first_seen_at: "2026-09-04T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
        };
        upsert_volume(&conn, &v).unwrap();
        link_device_volume(
            &conn,
            "d1",
            "v1",
            MappingConfidence::Exact,
            "windows_storage",
        )
        .unwrap();

        let vols = volumes_for_device(&conn, "d1").unwrap();
        assert_eq!(vols, vec!["v1".to_string()]);
        assert_eq!(list_volumes(&conn).unwrap().len(), 1);
    }
}
