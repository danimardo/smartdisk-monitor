//! Resumen con IA del informe imprimible, por disco (spec `009-informe-mejorado`, principio XVI
//! 1.11.0, ADR-057).
//!
//! **Un payload = un disco**: nunca se combinan dos discos en una misma petición. El alcance por
//! disco es exactamente el que autoriza la enmienda: sus alertas del intervalo + el contenido de
//! los sucesos de Windows que las originaron + sus contadores SMART + el resumen numérico de
//! temperatura y actividad del intervalo. La **anonimización ocurre aquí**, antes de que nada
//! salga del proceso — ni siquiera para la vista previa, que muestra el texto ya anonimizado.

use std::collections::HashMap;

use crate::domain::ia::{
    self, AlertaParaModelo, ContadorSmart, ContextoDisco, Detalle, DetalleInforme,
    FragmentoDudosoWire, Idioma, ResumenNumerico,
};
use crate::domain::tipos::{AlertGroup, Device};
use crate::persistence::{repo_alertas, repo_inventario, repo_varios};
use crate::reporting::anonimizar::Anonimizador;
use crate::reporting::export::{etiqueta_dispositivo, RangoExport};
use crate::reporting::informe::{alerta_en_rango, contadores_con_delta, DeltaContador};
use crate::reporting::resumen_metricas::resumen_dispositivo;

/// El texto exacto que se enviaría por un disco, ya anonimizado — para la vista previa (FR-013).
pub struct PreviewDisco {
    pub device_id: String,
    pub device_label: String,
    pub texto_enviado: String,
    pub fragmentos: Vec<FragmentoDudosoWire>,
    pub recortado: bool,
}

/// El resultado del resumen de un disco: generado, o no disponible con el motivo (degradación,
/// FR-019 — nunca tira el resto del informe).
#[derive(Debug, Clone, PartialEq)]
pub enum ResumenIaSeccion {
    Generado {
        markdown: String,
        modelo_usado: String,
    },
    NoDisponible {
        motivo_key: String,
    },
}

/// Anonimizador con los números de serie de todos los discos presentes y las etiquetas de todos
/// los volúmenes conocidos — no solo los del disco que se está resumiendo: un suceso de otro disco
/// citado en el mensaje de un evento también debe quedar cubierto.
pub(crate) fn anonimizador_base(conn: &rusqlite::Connection) -> rusqlite::Result<Anonimizador> {
    let series: Vec<String> = repo_inventario::list_present_devices(conn)?
        .into_iter()
        .filter_map(|d| d.serial_number)
        .collect();
    let mut anon = Anonimizador::para_esta_maquina(&series);
    for v in repo_inventario::list_volumes(conn)? {
        if let Some(label) = v.label {
            anon = anon.con_etiqueta_volumen(&label);
        }
    }
    Ok(anon)
}

/// Contenido (ya extraído, sin el bloque de metadatos de sistema) de los sucesos de Windows que
/// originaron las ocurrencias de `alerta` dentro de `[desde, hasta]`.
fn contenidos_de_alerta(
    conn: &rusqlite::Connection,
    alerta: &AlertGroup,
    desde: &str,
    hasta: &str,
) -> rusqlite::Result<Vec<String>> {
    let ocurrencias = repo_alertas::list_occurrences(conn, &alerta.id)?;
    let mut ids: Vec<i64> = ocurrencias
        .into_iter()
        .filter(|o| o.occurred_at_utc.as_str() >= desde && o.occurred_at_utc.as_str() <= hasta)
        .filter_map(|o| o.triggering_event_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();

    let mut out = Vec::new();
    for id in ids {
        if let Some(ev) = repo_varios::get_event_by_id(conn, id)? {
            if let Some(c) =
                ia::extraer_contenido_suceso(ev.message.as_deref(), ev.raw_xml.as_deref())
            {
                out.push(c);
            }
        }
    }
    Ok(out)
}

/// Construye `(system, user, recortado)` de un disco: compone la consulta y **anonimiza el
/// `user`** (la única parte con datos reales) antes de devolverla. `system` es una constante, sin
/// datos que anonimizar.
pub(crate) fn payload_disco(
    conn: &rusqlite::Connection,
    device: &Device,
    rango: RangoExport,
    todas_las_alertas: &[AlertGroup],
    alert_labels: &HashMap<String, String>,
    anon: &Anonimizador,
) -> rusqlite::Result<(String, String, bool)> {
    let desde = rango.desde_como_texto();
    let hasta = rango.hasta_como_texto();

    let alertas_del_disco: Vec<&AlertGroup> = todas_las_alertas
        .iter()
        .filter(|a| a.target_device_id.as_deref() == Some(device.id.as_str()))
        .filter(|a| alerta_en_rango(a, &desde, &hasta))
        .collect();

    let alertas_modelo: Vec<AlertaParaModelo> = alertas_del_disco
        .iter()
        .map(|a| AlertaParaModelo {
            descripcion: alert_labels
                .get(&a.rule_key)
                .map(String::as_str)
                .unwrap_or(a.rule_key.as_str()),
            severidad: repo_alertas::severity_to_str(a.severity),
            primera_vez: a.first_occurrence_at_utc.as_str(),
            ultima_vez: a.last_occurrence_at_utc.as_str(),
            veces: a.occurrence_count,
        })
        .collect();

    let mut contenidos_suceso: Vec<String> = Vec::new();
    for a in &alertas_del_disco {
        contenidos_suceso.extend(contenidos_de_alerta(conn, a, &desde, &hasta)?);
    }
    let contenidos_ref: Vec<&str> = contenidos_suceso.iter().map(String::as_str).collect();

    let contadores_delta = contadores_con_delta(conn, &device.id, rango)?;
    let contadores_modelo: Vec<ContadorSmart> = contadores_delta
        .iter()
        .map(|c| ContadorSmart {
            nombre: c.clave.as_str(),
            valor: c.valor_final,
            unidad: None,
            significativo: matches!(c.delta, DeltaContador::Valor(d) if d > 0.0),
        })
        .collect();

    let resumen_temp = resumen_dispositivo(conn, &device.id, "temperature_celsius", rango)?;
    let resumen_act = resumen_dispositivo(conn, &device.id, "activity_percent", rango)?;
    let resumenes = [
        ResumenNumerico {
            etiqueta: "Temperatura (°C)",
            minimo: resumen_temp.minimo,
            media: resumen_temp.media,
            maximo: resumen_temp.maximo,
            pico: resumen_temp.pico,
        },
        ResumenNumerico {
            etiqueta: "Actividad (%)",
            minimo: resumen_act.minimo,
            media: resumen_act.media,
            maximo: resumen_act.maximo,
            pico: resumen_act.pico,
        },
    ];

    let ctx = ContextoDisco {
        modelo: &device.model,
        tipo: repo_inventario::device_type_to_str(device.device_type),
        bus: device.bus_type.as_deref(),
        firmware: device.firmware.as_deref(),
        antiguedad_meses: None,
    };
    let detalle = DetalleInforme {
        alertas: &alertas_modelo,
        contenidos_suceso: &contenidos_ref,
        contadores: &contadores_modelo,
        resumenes: &resumenes,
        desde_local: &desde,
        hasta_local: &hasta,
    };

    let (system, user) =
        ia::componer_consulta(&Detalle::Informe(detalle), &ctx, Idioma::Es, None, None);
    let user_anon = ia::redactar_identificadores(&anon.aplicar(&user));
    let (user_final, recortado) = ia::recortar(
        &user_anon,
        crate::platform::ia_openrouter::MAX_DETALLE_CHARS,
    );
    Ok((system, user_final, recortado))
}

/// Vista previa (sin red) del texto exacto y anonimizado que se enviaría por cada disco incluido.
pub fn preview_datos(
    conn: &rusqlite::Connection,
    rango: RangoExport,
    dispositivos: &[Device],
    todas_las_alertas: &[AlertGroup],
    alert_labels: &HashMap<String, String>,
) -> rusqlite::Result<Vec<PreviewDisco>> {
    let anon = anonimizador_base(conn)?;
    let mut out = Vec::with_capacity(dispositivos.len());
    for d in dispositivos {
        let (system, user, recortado) =
            payload_disco(conn, d, rango, todas_las_alertas, alert_labels, &anon)?;
        let fragmentos = ia::barrer_texto_residual(&user);
        out.push(PreviewDisco {
            device_id: d.id.clone(),
            device_label: etiqueta_dispositivo(d),
            texto_enviado: format!("{system}\n\n{user}"),
            fragmentos,
            recortado,
        });
    }
    Ok(out)
}

/// Genera el resumen de **un** disco: compone su payload (ya anonimizado) y hace la llamada al
/// modelo. Nunca falla el informe entero: un error de red, cuota o proveedor se traduce a
/// `NoDisponible` con la clave del motivo, para que la sección del disco lleve su nota (FR-019).
/// Sin reintento automático (principio XVI).
#[allow(clippy::too_many_arguments)]
pub async fn generar_resumen_disco(
    conn: &rusqlite::Connection,
    device: &Device,
    rango: RangoExport,
    todas_las_alertas: &[AlertGroup],
    alert_labels: &HashMap<String, String>,
    anon: &Anonimizador,
    clave_api: &str,
    modelo: &str,
) -> rusqlite::Result<ResumenIaSeccion> {
    let (system, user, _) =
        payload_disco(conn, device, rango, todas_las_alertas, alert_labels, anon)?;
    let respuesta =
        crate::platform::ia_openrouter::chat_completions(clave_api, modelo, &system, &user).await;
    Ok(resultado_a_seccion(respuesta))
}

/// Traduce el resultado de la llamada (éxito u error de red/proveedor) a la sección del informe.
/// Puro y sin red: aquí vive la garantía de que un fallo de este disco **nunca** se propaga como
/// error del comando — se convierte en `NoDisponible` y el llamante sigue con el resto (FR-019,
/// sin reintento automático — principio XVI).
pub(crate) fn resultado_a_seccion(
    respuesta: Result<crate::domain::ia::RespuestaChat, crate::domain::ia::ErrorTransporte>,
) -> ResumenIaSeccion {
    match respuesta {
        Ok(cuerpo) => match ia::analizar_respuesta(cuerpo, false) {
            Ok(expl) => ResumenIaSeccion::Generado {
                markdown: expl.markdown,
                modelo_usado: expl.modelo_usado,
            },
            Err(err) => ResumenIaSeccion::NoDisponible {
                motivo_key: err.message_key,
            },
        },
        Err(err) => ResumenIaSeccion::NoDisponible {
            motivo_key: ia::analizar_error(err).message_key,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{
        AlertSeverity, AlertStatus, DeviceType, IdentityConfidence, MetricQuality, MetricSource,
        MetricTarget, Resolution, SystemEvent,
    };
    use crate::persistence::{db, repo_inventario, repo_metricas};
    use time::format_description::well_known::Rfc3339;

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("informe_ia");
        db::open(&dir).unwrap().0
    }

    fn dispositivo(id: &str, serie: &str) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: Some(serie.to_string()),
            model: format!("Modelo {id}"),
            manufacturer: None,
            firmware: Some("FW1".to_string()),
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: Some(r"\\.\PhysicalDrive0".to_string()),
            capacity_bytes: Some(1_000_000_000),
            alias: None,
            monitoring_enabled: true,
            first_seen_at: "2026-09-01T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    fn rango() -> RangoExport {
        RangoExport {
            desde: time::OffsetDateTime::parse("2026-09-04T00:00:00Z", &Rfc3339).unwrap(),
            hasta: time::OffsetDateTime::parse("2026-09-04T10:00:00Z", &Rfc3339).unwrap(),
        }
    }

    fn alerta(id: &str, device_id: &str) -> AlertGroup {
        AlertGroup {
            id: id.to_string(),
            deduplication_key: format!("smart.wear_high|device:{device_id}"),
            rule_key: "smart.wear_high".to_string(),
            target_device_id: Some(device_id.to_string()),
            target_volume_id: None,
            severity: AlertSeverity::Warning,
            status: AlertStatus::Active,
            muted_until: None,
            cycle: 1,
            first_occurrence_at_utc: "2026-09-04T01:00:00Z".to_string(),
            last_occurrence_at_utc: "2026-09-04T05:00:00Z".to_string(),
            occurrence_count: 2,
            acknowledged_at_utc: None,
            resolved_at_utc: None,
            archived_at_utc: None,
            ignored_at_utc: None,
            last_value_real: Some(92.0),
            context_json: None,
        }
    }

    #[test]
    fn el_payload_de_un_disco_anonimiza_serie_etiqueta_de_volumen_y_equipo() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", "S1B2C3SERIE");
        repo_inventario::upsert_device(&conn, &d).unwrap();
        repo_inventario::upsert_volume(
            &conn,
            &crate::domain::tipos::Volume {
                id: "v1".to_string(),
                volume_guid: "guid-1".to_string(),
                label: Some("MiVolumenSecreto".to_string()),
                filesystem: Some("NTFS".to_string()),
                drive_letters_json: Some(r#"["C:"]"#.to_string()),
                capacity_bytes: Some(1),
                free_bytes: Some(1),
                device_mapping_confidence: None,
                first_seen_at: "2026-09-01T00:00:00Z".to_string(),
                last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            },
        )
        .unwrap();

        std::env::set_var("COMPUTERNAME", "EQUIPO-DE-PRUEBA");

        let previews = preview_datos(&conn, rango(), &[d], &[], &HashMap::new()).unwrap();
        assert_eq!(previews.len(), 1);
        let texto = &previews[0].texto_enviado;
        assert!(
            !texto.contains("S1B2C3SERIE"),
            "el número de serie no debe viajar en claro"
        );
        assert!(
            !texto.contains("MiVolumenSecreto"),
            "la etiqueta de volumen no debe viajar en claro"
        );
        assert!(
            !texto.contains("EQUIPO-DE-PRUEBA"),
            "el nombre de equipo no debe viajar en claro"
        );
    }

    #[test]
    fn el_payload_de_un_disco_no_lleva_datos_de_otro() {
        let conn = conn_de_prueba();
        let d1 = dispositivo("d1", "SERIE-D1");
        let d2 = dispositivo("d2", "SERIE-D2");
        repo_inventario::upsert_device(&conn, &d1).unwrap();
        repo_inventario::upsert_device(&conn, &d2).unwrap();

        let alertas = vec![alerta("a1", "d1"), alerta("a2", "d2")];
        let mut labels = HashMap::new();
        labels.insert(
            "smart.wear_high".to_string(),
            "Desgaste elevado".to_string(),
        );

        let previews = preview_datos(&conn, rango(), &[d1, d2], &alertas, &labels).unwrap();
        assert_eq!(previews.len(), 2);
        assert!(previews[0].texto_enviado.contains("Modelo d1"));
        assert!(!previews[0].texto_enviado.contains("Modelo d2"));
        assert!(previews[1].texto_enviado.contains("Modelo d2"));
        assert!(!previews[1].texto_enviado.contains("Modelo d1"));
    }

    #[test]
    fn el_payload_incluye_el_contenido_del_suceso_que_origino_la_alerta() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", "SERIE-D1");
        repo_inventario::upsert_device(&conn, &d).unwrap();

        let evento = SystemEvent {
            id: 0,
            channel: "System".to_string(),
            record_id: 1,
            occurred_at_utc: "2026-09-04T02:00:00Z".to_string(),
            provider: "disk".to_string(),
            event_id: 51,
            level: crate::domain::tipos::EventLevel::Warning,
            message: Some("El dispositivo \\Device\\Harddisk0\\DR0 tuvo un error.".to_string()),
            raw_xml: None,
            device_id: Some("d1".to_string()),
            volume_id: None,
            mapping_confidence: crate::domain::tipos::MappingConfidence::Exact,
            dedup_hash: "hash-1".to_string(),
        };
        let evento_id = repo_varios::insert_event_returning_new_id(&conn, &evento)
            .unwrap()
            .unwrap();

        let a = alerta("a1", "d1");
        repo_alertas::create_group(&conn, &a).unwrap();
        repo_alertas::record_occurrence(&conn, &a.id, 2, "2026-09-04T02:00:00Z", Some(92.0), None)
            .unwrap();
        repo_alertas::set_triggering_event_ultima_ocurrencia(&conn, &a.id, evento_id).unwrap();

        let previews = preview_datos(&conn, rango(), &[d], &[a], &HashMap::new()).unwrap();
        assert!(previews[0].texto_enviado.contains("tuvo un error"));
    }

    #[test]
    fn una_respuesta_correcta_produce_generado_con_el_modelo_usado() {
        let respuesta = crate::domain::ia::RespuestaChat {
            model: Some("vendor/model:free".to_string()),
            choices: vec![crate::domain::ia::EleccionChat {
                message: Some(crate::domain::ia::MensajeChat {
                    content: Some("El disco está bien.".to_string()),
                }),
            }],
            error: None,
        };
        let seccion = resultado_a_seccion(Ok(respuesta));
        match seccion {
            ResumenIaSeccion::Generado {
                markdown,
                modelo_usado,
            } => {
                assert_eq!(markdown, "El disco está bien.");
                assert_eq!(modelo_usado, "vendor/model:free");
            }
            ResumenIaSeccion::NoDisponible { .. } => panic!("se esperaba Generado"),
        }
    }

    #[test]
    fn un_fallo_de_red_produce_no_disponible_sin_tirar_el_resto_del_informe() {
        let seccion = resultado_a_seccion(Err(crate::domain::ia::ErrorTransporte::Timeout));
        assert!(matches!(seccion, ResumenIaSeccion::NoDisponible { .. }));
    }

    #[test]
    fn un_error_del_proveedor_tambien_produce_no_disponible() {
        let respuesta = crate::domain::ia::RespuestaChat {
            model: None,
            choices: vec![],
            error: Some(crate::domain::ia::ErrorProveedor {
                message: Some("cuota agotada".to_string()),
            }),
        };
        let seccion = resultado_a_seccion(Ok(respuesta));
        assert!(matches!(seccion, ResumenIaSeccion::NoDisponible { .. }));
    }

    #[test]
    fn sin_linea_base_de_smart_los_contadores_igual_viajan_con_su_valor() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", "SERIE-D1");
        repo_inventario::upsert_device(&conn, &d).unwrap();
        repo_metricas::insert_sample(
            &conn,
            &crate::domain::tipos::MetricSample {
                target: MetricTarget::Device("d1".to_string()),
                metric_key: "power_cycles".to_string(),
                value_real: Some(812.0),
                value_integer: None,
                unit: "count".to_string(),
                sampled_at_utc: "2026-09-04T05:00:00Z".to_string(),
                source: MetricSource::Smartctl,
                quality: MetricQuality::Exact,
                resolution: Resolution::Raw,
            },
        )
        .unwrap();

        let previews = preview_datos(&conn, rango(), &[d], &[], &HashMap::new()).unwrap();
        assert!(previews[0].texto_enviado.contains("power_cycles"));
        assert!(previews[0].texto_enviado.contains("812"));
    }
}
