//! Paquete de diagnóstico anonimizado por defecto (T090, FR-028, US-051,
//! `docs/product-specification.md` §9, `docs/open-questions.md` J.31).
//!
//! `smart_snapshots.raw_json_path` no tiene contenido real que leer (ningún colector lo escribe
//! todavía, T036): el SMART bruto se vuelve a consultar en vivo con
//! `collectors::smartctl::query_device_json` en el momento de generar el paquete, no se reconstruye
//! de una captura que nunca se guardó (J.31). Cada entrada de texto pasa por el mismo
//! `Anonimizador`, así que la sustitución es consistente en todo el paquete (US-051).

use std::io::Write;
use std::path::Path;

use super::anonimizar::Anonimizador;
use crate::domain::tipos::Device;
use crate::persistence::repo_varios::{self, FiltroEventos};

const SCHEMA_VERSION: &str = "1";
/// Volcado completo, no una pantalla paginada: un límite alto en vez de paginación real.
const LIMITE_EVENTOS: i64 = 100_000;

pub struct EntradaZip {
    pub ruta: String,
    pub contenido: Vec<u8>,
}

pub struct PaqueteDiagnostico {
    pub entradas: Vec<EntradaZip>,
    pub campos_redactados: Vec<&'static str>,
}

impl PaqueteDiagnostico {
    pub fn total_bytes(&self) -> u64 {
        self.entradas.iter().map(|e| e.contenido.len() as u64).sum()
    }
}

fn entrada_texto(anonimizador: &Anonimizador, ruta: impl Into<String>, texto: &str) -> EntradaZip {
    EntradaZip {
        ruta: ruta.into(),
        contenido: anonimizador.aplicar(texto).into_bytes(),
    }
}

fn manifest_json(anonimizador: &Anonimizador) -> String {
    let generado = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();
    serde_json::json!({
        "schemaVersion": SCHEMA_VERSION,
        "generatedAtUtc": generado,
        "appVersion": env!("CARGO_PKG_VERSION"),
        "anonymized": !anonimizador.campos_afectados().is_empty(),
        "redactedFields": anonimizador.campos_afectados(),
    })
    .to_string()
}

fn settings_json(conn: &rusqlite::Connection) -> Option<String> {
    let claves = repo_varios::list_settings(conn).ok()?;
    let objeto: serde_json::Map<String, serde_json::Value> = claves
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                serde_json::from_str(&v).unwrap_or(serde_json::Value::Null),
            )
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::Value::Object(objeto)).ok()
}

fn events_json(conn: &rusqlite::Connection) -> Option<String> {
    let eventos = repo_varios::list_events(
        conn,
        &FiltroEventos {
            device_id: None,
            volume_id: None,
            levels: None,
            providers: None,
            from_utc: None,
            to_utc: None,
            cursor: None,
            limit: LIMITE_EVENTOS,
        },
    )
    .ok()?;
    let lista: Vec<serde_json::Value> = eventos
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "occurredAtUtc": e.occurred_at_utc,
                "provider": e.provider,
                "eventId": e.event_id,
                "level": format!("{:?}", e.level),
                "message": e.message,
                "deviceId": e.device_id,
                "volumeId": e.volume_id,
            })
        })
        .collect();
    serde_json::to_string_pretty(&lista).ok()
}

/// Ficheros SMART brutos, uno por dispositivo con `smartctl_path` — o su error, para que un
/// disco que falle no tire el paquete entero (US-051 no exige un paquete todo-o-nada).
fn entradas_smart(anonimizador: &Anonimizador, dispositivos: &[Device]) -> Vec<EntradaZip> {
    dispositivos
        .iter()
        .filter_map(|d| {
            let ruta_dispositivo = d.smartctl_path.as_deref()?;
            Some(
                match crate::collectors::smartctl::query_device_json(ruta_dispositivo) {
                    Ok(json) => entrada_texto(anonimizador, format!("smart/{}.json", d.id), &json),
                    Err(e) => entrada_texto(
                        anonimizador,
                        format!("smart/{}.error.txt", d.id),
                        &format!("{e:?}"),
                    ),
                },
            )
        })
        .collect()
}

/// Todo lo que haya en la carpeta de registro, tal cual lo escribe `tracing_appender` (FR-029c):
/// un fichero que no se pueda leer como texto (rotación en curso, permisos) se omite en vez de
/// tirar el paquete entero.
fn entradas_logs(anonimizador: &Anonimizador, log_dir: &Path) -> Vec<EntradaZip> {
    let Ok(directorio) = std::fs::read_dir(log_dir) else {
        return Vec::new();
    };
    directorio
        .flatten()
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let contenido = std::fs::read_to_string(e.path()).ok()?;
            let nombre = e.file_name().to_string_lossy().into_owned();
            Some(entrada_texto(
                anonimizador,
                format!("logs/{nombre}"),
                &contenido,
            ))
        })
        .collect()
}

/// Reúne el contenido del paquete sin escribir nada a disco todavía: `preview_diagnostic_zip`
/// (US-051, "antes de guardar se muestra un resumen del contenido") usa el mismo recolector que
/// `create_diagnostic_zip`, para que la vista previa nunca diverja de lo que de verdad se guarda.
pub fn recolectar(
    conn: &rusqlite::Connection,
    dispositivos: &[Device],
    include_identifiers: bool,
    log_dir: &Path,
) -> PaqueteDiagnostico {
    let anonimizador = if include_identifiers {
        Anonimizador::sin_anonimizar()
    } else {
        let series: Vec<String> = dispositivos
            .iter()
            .filter_map(|d| d.serial_number.clone())
            .collect();
        Anonimizador::para_esta_maquina(&series)
    };

    let mut entradas = Vec::new();
    if let Some(texto) = settings_json(conn) {
        entradas.push(entrada_texto(&anonimizador, "settings.json", &texto));
    }
    if let Some(texto) = events_json(conn) {
        entradas.push(entrada_texto(&anonimizador, "events.json", &texto));
    }
    entradas.extend(entradas_smart(&anonimizador, dispositivos));
    entradas.extend(entradas_logs(&anonimizador, log_dir));

    // El manifiesto va el último en construirse pero el primero en el ZIP: así lista de verdad
    // `redactedFields` una vez que `anonimizador` ya sabe qué categorías tuvo que sustituir.
    let manifiesto = entrada_texto(
        &anonimizador,
        "manifest.json",
        &manifest_json(&anonimizador),
    );
    entradas.insert(0, manifiesto);

    PaqueteDiagnostico {
        entradas,
        campos_redactados: anonimizador.campos_afectados().to_vec(),
    }
}

/// Escribe el paquete ya recolectado como un ZIP real en `destino`. A diferencia del archivo
/// temporal del benchmark (T079, que nunca debe sobrescribir uno existente), aquí sí se sobrescribe
/// si el destino ya existe: es un «Guardar como» explícito sobre una ruta que el propio diálogo
/// nativo de guardado (ADR-031) ya le confirmó al usuario, el mismo comportamiento esperado de
/// cualquier «Guardar como» de la plataforma.
pub fn escribir_zip(paquete: &PaqueteDiagnostico, destino: &Path) -> zip::result::ZipResult<()> {
    let archivo = std::fs::File::create(destino)?;
    let mut escritor = zip::ZipWriter::new(archivo);
    let opciones = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entrada in &paquete.entradas {
        escritor.start_file(&entrada.ruta, opciones)?;
        escritor.write_all(&entrada.contenido)?;
    }
    escritor.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{DeviceType, IdentityConfidence};
    use crate::persistence::db;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("reporting_diagnostico");
        db::open(&dir).unwrap().0
    }

    fn dispositivo_sin_smartctl(id: &str, serial: Option<&str>) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: serial.map(str::to_string),
            model: "Modelo de prueba".to_string(),
            manufacturer: None,
            firmware: None,
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: None,
            capacity_bytes: Some(1_000_000_000),
            alias: None,
            monitoring_enabled: true,
            first_seen_at: "2026-09-01T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    #[test]
    fn el_manifiesto_va_primero_y_declara_los_campos_redactados() {
        let conn = conn_de_prueba();
        let dispositivos = [dispositivo_sin_smartctl("d1", Some("SERIE-XYZ"))];
        let dir = crate::test_util::temp_dir_unico("reporting_diagnostico_logs");

        let paquete = recolectar(&conn, &dispositivos, false, &dir);
        assert_eq!(paquete.entradas[0].ruta, "manifest.json");
        let manifiesto = String::from_utf8(paquete.entradas[0].contenido.clone()).unwrap();
        assert!(manifiesto.contains("\"anonymized\":true"));
        assert!(manifiesto.contains("diagnostic.redacted.serialNumber"));
    }

    #[test]
    fn incluir_identificadores_no_redacta_nada() {
        let conn = conn_de_prueba();
        let dispositivos = [dispositivo_sin_smartctl("d1", Some("SERIE-XYZ"))];
        let dir = crate::test_util::temp_dir_unico("reporting_diagnostico_logs");

        let paquete = recolectar(&conn, &dispositivos, true, &dir);
        assert!(paquete.campos_redactados.is_empty());
        let manifiesto = String::from_utf8(paquete.entradas[0].contenido.clone()).unwrap();
        assert!(manifiesto.contains("\"anonymized\":false"));
    }

    #[test]
    fn un_numero_de_serie_no_aparece_en_ningun_fichero_de_texto_por_defecto() {
        let conn = conn_de_prueba();
        let dispositivos = [dispositivo_sin_smartctl("d1", Some("SERIE-UNICA-XYZ"))];
        let dir = crate::test_util::temp_dir_unico("reporting_diagnostico_logs");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("smartdisk.log.2026-09-04"),
            "arranque con SERIE-UNICA-XYZ detectado",
        )
        .unwrap();

        let paquete = recolectar(&conn, &dispositivos, false, &dir);
        for entrada in &paquete.entradas {
            let texto = String::from_utf8_lossy(&entrada.contenido);
            assert!(
                !texto.contains("SERIE-UNICA-XYZ"),
                "el número de serie no debe sobrevivir en {}",
                entrada.ruta
            );
        }
    }

    #[test]
    fn escribir_zip_produce_un_archivo_que_se_puede_volver_a_leer() {
        let conn = conn_de_prueba();
        let dispositivos = [dispositivo_sin_smartctl("d1", None)];
        let dir = crate::test_util::temp_dir_unico("reporting_diagnostico_logs");
        let paquete = recolectar(&conn, &dispositivos, false, &dir);

        let destino = dir.join("diagnostico.zip");
        std::fs::create_dir_all(&dir).unwrap();
        escribir_zip(&paquete, &destino).unwrap();

        let archivo = std::fs::File::open(&destino).unwrap();
        let mut lector = zip::ZipArchive::new(archivo).unwrap();
        assert!(lector.by_name("manifest.json").is_ok());
        assert!(lector.by_name("settings.json").is_ok());
    }
}
