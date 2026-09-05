//! Registro de eventos de Windows con marcador persistente (T067, `docs/architecture.md`
//! "Registro de eventos", `docs/alert-rules.md` §3).
//!
//! Enlace FFI directo a `wevtapi.dll`, mismo criterio que `perf_counters.rs` con `pdh.dll`: sin
//! añadir el crate `windows` completo por una docena de funciones estables de una API que no ha
//! cambiado desde Windows Vista. Las firmas están verificadas contra los bindings que genera
//! `windows-rs` (Context7, `Win32::System::EventLog`), no adivinadas.
//!
//! **El cursor es un bookmark real del Event Log, no un `RecordId` suelto**
//! (`docs/open-questions.md` J.7): limpiar un canal reinicia los identificadores, y un cursor
//! numérico dejaría de importar eventos nuevos sin avisar. `EvtSeek` sobre un bookmark inválido
//! (canal limpiado, evento purgado) falla explícitamente; ese fallo se trata como "sin cursor
//! previo" y se relee desde el principio del canal — la alternativa (propagar el error y dejar de
//! leer eventos para siempre) sería mucho peor.
//!
//! Este colector **no decide correlación con un disco**: eso es `domain::correlacion` (T069). Aquí
//! solo se normaliza proveedor, id, nivel, fecha y mensaje — `device_id`/`volume_id` quedan `None`
//! y `mapping_confidence` en `Unknown`, honesto hasta que otro módulo decida.

use crate::domain::tipos::EventLevel;

/// Proveedores de almacenamiento vigilados (`docs/alert-rules.md` §3.1/§3.2), verificados contra
/// un Windows 11 real. Vive aquí como constante, no en `settings`, porque esa plumbing todavía no
/// existe (`get_setting_raw`/`set_setting_raw` no tienen una tabla de ajustes reales conectada);
/// documentado como pendiente de mover cuando exista (`docs/open-questions.md`).
pub const PROVEEDORES_VIGILADOS: &[&str] = &[
    "disk",
    "Ntfs",
    "Microsoft-Windows-Ntfs",
    "volmgr",
    "Microsoft-Windows-NvmeDisk",
    "stornvme",
    "storahci",
    "Microsoft-Windows-StorageSpaces-Driver",
];

/// Evento ya normalizado, sin decidir correlación todavía. `record_id`/`channel` son la identidad
/// persistida (`UNIQUE(channel, record_id)`); `raw_xml` se conserva completo para el detalle.
#[derive(Debug, Clone, PartialEq)]
pub struct EventoLeido {
    pub channel: String,
    pub record_id: i64,
    pub occurred_at_utc: String,
    pub provider: String,
    pub event_id: i64,
    pub level: EventLevel,
    pub message: Option<String>,
    pub raw_xml: String,
}

/// El nivel numérico de Windows (`System/Level` en el XML del evento): 1=crítico, 2=error,
/// 3=advertencia, 4/5/0=informativo. No hay un "desconocido": un nivel fuera de rango se trata
/// como informativo, el lado seguro (una alerta perdida por infravalorar severidad sería peor que
/// nunca ocurre con estos valores, que son estables desde Vista).
fn nivel_desde_numero(n: u8) -> EventLevel {
    match n {
        1 => EventLevel::Critical,
        2 => EventLevel::Error,
        3 => EventLevel::Warning,
        _ => EventLevel::Information,
    }
}

/// Extrae el contenido de la primera etiqueta `<tag>...</tag>` o `<tag ...>...</tag>` dentro de
/// `xml`. Basta para el bloque `<System>` del Event Log, cuyo esquema es fijo y lo produce el
/// propio Windows — no hace falta un analizador XML completo para seis campos de un formato que
/// no varía.
fn extraer_texto(xml: &str, tag: &str) -> Option<String> {
    let apertura_corta = format!("<{tag}>");
    let apertura_con_atributos = format!("<{tag} ");
    let cierre = format!("</{tag}>");

    let inicio_apertura = xml
        .find(&apertura_corta)
        .or_else(|| xml.find(&apertura_con_atributos))?;
    let inicio_contenido = xml[inicio_apertura..].find('>')? + inicio_apertura + 1;
    let fin_contenido = xml[inicio_contenido..].find(&cierre)? + inicio_contenido;
    Some(xml[inicio_contenido..fin_contenido].to_string())
}

/// Extrae el valor de un atributo `nombre='valor'` o `nombre="valor"` dentro de la primera
/// ocurrencia de `<tag ...>`.
fn extraer_atributo(xml: &str, tag: &str, atributo: &str) -> Option<String> {
    let inicio_tag = xml.find(&format!("<{tag} "))?;
    let fin_tag = xml[inicio_tag..].find('>')? + inicio_tag;
    let bloque = &xml[inicio_tag..fin_tag];

    for patron in [format!("{atributo}='"), format!("{atributo}=\"")] {
        if let Some(pos) = bloque.find(&patron) {
            let inicio = pos + patron.len();
            let comilla = patron.chars().last().unwrap();
            let fin = bloque[inicio..].find(comilla)? + inicio;
            return Some(bloque[inicio..fin].to_string());
        }
    }
    None
}

#[derive(Debug)]
pub enum ErrorParseoEvento {
    CampoAusente(&'static str),
}

/// Parsea el XML completo de un evento (`EvtRenderEventXml`) a un `EventoLeido`. El mensaje
/// formateado (`EvtFormatMessage`) se pasa aparte porque no está en este XML — el XML crudo solo
/// trae los valores, no el texto humano, que exige metadatos del proveedor.
pub fn parse_event_xml(
    xml: &str,
    channel_esperado: &str,
    mensaje: Option<String>,
) -> Result<EventoLeido, ErrorParseoEvento> {
    let sistema_inicio = xml
        .find("<System>")
        .ok_or(ErrorParseoEvento::CampoAusente("System"))?;
    let sistema_fin = xml[sistema_inicio..]
        .find("</System>")
        .ok_or(ErrorParseoEvento::CampoAusente("System"))?
        + sistema_inicio;
    let bloque_sistema = &xml[sistema_inicio..sistema_fin];

    let provider = extraer_atributo(bloque_sistema, "Provider", "Name")
        .ok_or(ErrorParseoEvento::CampoAusente("Provider/@Name"))?;
    let event_id = extraer_texto(bloque_sistema, "EventID")
        .ok_or(ErrorParseoEvento::CampoAusente("EventID"))?
        .parse()
        .map_err(|_| ErrorParseoEvento::CampoAusente("EventID"))?;
    let nivel_numero: u8 = extraer_texto(bloque_sistema, "Level")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let occurred_at_utc = extraer_atributo(bloque_sistema, "TimeCreated", "SystemTime")
        .ok_or(ErrorParseoEvento::CampoAusente("TimeCreated/@SystemTime"))?;
    let record_id = extraer_texto(bloque_sistema, "EventRecordID")
        .ok_or(ErrorParseoEvento::CampoAusente("EventRecordID"))?
        .parse()
        .map_err(|_| ErrorParseoEvento::CampoAusente("EventRecordID"))?;
    let channel =
        extraer_texto(bloque_sistema, "Channel").unwrap_or_else(|| channel_esperado.to_string());

    Ok(EventoLeido {
        channel,
        record_id,
        occurred_at_utc,
        provider,
        event_id,
        level: nivel_desde_numero(nivel_numero),
        message: mensaje,
        raw_xml: xml.to_string(),
    })
}

#[cfg(windows)]
mod ffi {
    //! Bindings mínimos, solo lo que este módulo usa. `EVT_HANDLE` es un manejador opaco (`isize`)
    //! desde siempre en la ABI de Win32.
    #![allow(non_snake_case, non_camel_case_types)]

    pub type EVT_HANDLE = isize;

    pub const EVT_QUERY_CHANNEL_PATH: u32 = 0x1;
    pub const EVT_QUERY_FORWARD_DIRECTION: u32 = 0x100;
    pub const EVT_RENDER_EVENT_XML: u32 = 1;
    pub const EVT_RENDER_BOOKMARK: u32 = 2;
    pub const EVT_SEEK_RELATIVE_TO_BOOKMARK: u32 = 4;
    pub const EVT_FORMAT_MESSAGE_EVENT: u32 = 1;
    pub const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    pub const ERROR_NO_MORE_ITEMS: u32 = 259;

    #[link(name = "wevtapi")]
    extern "system" {
        pub fn EvtQuery(
            session: EVT_HANDLE,
            path: *const u16,
            query: *const u16,
            flags: u32,
        ) -> EVT_HANDLE;
        pub fn EvtNext(
            resultset: EVT_HANDLE,
            events_size: u32,
            events: *mut EVT_HANDLE,
            timeout: u32,
            flags: u32,
            returned: *mut u32,
        ) -> i32;
        pub fn EvtRender(
            context: EVT_HANDLE,
            fragment: EVT_HANDLE,
            flags: u32,
            buffer_size: u32,
            buffer: *mut u16,
            buffer_used: *mut u32,
            property_count: *mut u32,
        ) -> i32;
        pub fn EvtClose(object: EVT_HANDLE) -> i32;
        pub fn EvtCreateBookmark(bookmark_xml: *const u16) -> EVT_HANDLE;
        pub fn EvtUpdateBookmark(bookmark: EVT_HANDLE, event: EVT_HANDLE) -> i32;
        pub fn EvtSeek(
            resultset: EVT_HANDLE,
            position: i64,
            bookmark: EVT_HANDLE,
            timeout: u32,
            flags: u32,
        ) -> i32;
        pub fn EvtOpenPublisherMetadata(
            session: EVT_HANDLE,
            publisher_id: *const u16,
            log_file_path: *const u16,
            locale: u32,
            flags: u32,
        ) -> EVT_HANDLE;
        pub fn EvtFormatMessage(
            publisher_metadata: EVT_HANDLE,
            event: EVT_HANDLE,
            message_id: u32,
            value_count: u32,
            values: *const std::ffi::c_void,
            flags: u32,
            buffer_size: u32,
            buffer: *mut u16,
            buffer_used: *mut u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetLastError() -> u32;
    }
}

#[cfg(windows)]
fn a_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
fn de_wide(buf: &[u16]) -> String {
    let fin = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..fin])
}

#[derive(Debug)]
pub enum ErrorEventLog {
    Win32 { funcion: &'static str, codigo: u32 },
}

/// Renderiza un fragmento (evento o bookmark) a texto ancho, con el patrón de dos pasadas: la
/// primera solo pregunta el tamaño (`ERROR_INSUFFICIENT_BUFFER` es éxito parcial esperado, no un
/// fallo), la segunda rellena el buffer ya del tamaño correcto. `bytes_no_chars` distingue
/// `EvtRender` (tamaño en bytes) de `EvtFormatMessage` (tamaño en caracteres) — confundirlos
/// produce un buffer del doble o la mitad del necesario.
#[cfg(windows)]
fn evt_render(
    context: ffi::EVT_HANDLE,
    fragment: ffi::EVT_HANDLE,
    flags: u32,
) -> Result<String, ErrorEventLog> {
    let mut requerido: u32 = 0;
    let mut propiedades: u32 = 0;
    unsafe {
        ffi::EvtRender(
            context,
            fragment,
            flags,
            0,
            std::ptr::null_mut(),
            &mut requerido,
            &mut propiedades,
        );
    }
    if unsafe { ffi::GetLastError() } != ffi::ERROR_INSUFFICIENT_BUFFER {
        return Err(ErrorEventLog::Win32 {
            funcion: "EvtRender(tamaño)",
            codigo: unsafe { ffi::GetLastError() },
        });
    }

    let mut buffer: Vec<u16> = vec![0; (requerido as usize).div_ceil(2)];
    let ok = unsafe {
        ffi::EvtRender(
            context,
            fragment,
            flags,
            requerido,
            buffer.as_mut_ptr(),
            &mut requerido,
            &mut propiedades,
        )
    };
    if ok == 0 {
        return Err(ErrorEventLog::Win32 {
            funcion: "EvtRender",
            codigo: unsafe { ffi::GetLastError() },
        });
    }
    Ok(de_wide(&buffer))
}

/// Formatea el mensaje humano de un evento con los metadatos de su proveedor. `None` si el
/// proveedor no tiene metadatos registrados (puede pasar con proveedores muy poco comunes) o si el
/// formateo falla: un evento sin mensaje sigue siendo un evento válido, no un error.
#[cfg(windows)]
fn formatear_mensaje(provider: &str, event: ffi::EVT_HANDLE) -> Option<String> {
    let provider_ancho = a_wide(provider);
    let metadata = unsafe {
        ffi::EvtOpenPublisherMetadata(0, provider_ancho.as_ptr(), std::ptr::null(), 0, 0)
    };
    if metadata == 0 {
        return None;
    }

    let mut requerido: u32 = 0;
    unsafe {
        ffi::EvtFormatMessage(
            metadata,
            event,
            0xFFFF_FFFF,
            0,
            std::ptr::null(),
            ffi::EVT_FORMAT_MESSAGE_EVENT,
            0,
            std::ptr::null_mut(),
            &mut requerido,
        );
    }
    let resultado =
        if unsafe { ffi::GetLastError() } == ffi::ERROR_INSUFFICIENT_BUFFER && requerido > 0 {
            let mut buffer: Vec<u16> = vec![0; requerido as usize];
            let ok = unsafe {
                ffi::EvtFormatMessage(
                    metadata,
                    event,
                    0xFFFF_FFFF,
                    0,
                    std::ptr::null(),
                    ffi::EVT_FORMAT_MESSAGE_EVENT,
                    requerido,
                    buffer.as_mut_ptr(),
                    &mut requerido,
                )
            };
            if ok != 0 {
                Some(de_wide(&buffer))
            } else {
                None
            }
        } else {
            None
        };

    unsafe { ffi::EvtClose(metadata) };
    resultado
}

/// XPath que filtra el canal a los proveedores vigilados: `Get-WinEvent -FilterXml` usa el mismo
/// dialecto. `or` encadenado en vez de una lista, porque `EvtQuery` no acepta un `in()`.
fn construir_filtro(proveedores: &[&str]) -> String {
    let condiciones: Vec<String> = proveedores.iter().map(|p| format!("@Name='{p}'")).collect();
    format!("*[System[Provider[{}]]]", condiciones.join(" or "))
}

/// Lee los eventos nuevos de `channel` desde el último bookmark guardado (o desde el principio si
/// no hay ninguno, o si el guardado ya no es válido — canal limpiado entre lecturas). Devuelve los
/// eventos y el nuevo bookmark serializado, listo para persistir en `event_cursors`.
#[cfg(windows)]
pub fn leer_eventos_nuevos(
    channel: &str,
    bookmark_previo: Option<&str>,
) -> Result<(Vec<EventoLeido>, Option<String>), ErrorEventLog> {
    let channel_ancho = a_wide(channel);
    let filtro = construir_filtro(PROVEEDORES_VIGILADOS);
    let filtro_ancho = a_wide(&filtro);

    let resultset = unsafe {
        ffi::EvtQuery(
            0,
            channel_ancho.as_ptr(),
            filtro_ancho.as_ptr(),
            ffi::EVT_QUERY_CHANNEL_PATH | ffi::EVT_QUERY_FORWARD_DIRECTION,
        )
    };
    if resultset == 0 {
        return Err(ErrorEventLog::Win32 {
            funcion: "EvtQuery",
            codigo: unsafe { ffi::GetLastError() },
        });
    }

    if let Some(xml_previo) = bookmark_previo {
        let xml_ancho = a_wide(xml_previo);
        let bookmark = unsafe { ffi::EvtCreateBookmark(xml_ancho.as_ptr()) };
        if bookmark != 0 {
            let ok = unsafe {
                ffi::EvtSeek(
                    resultset,
                    1,
                    bookmark,
                    0,
                    ffi::EVT_SEEK_RELATIVE_TO_BOOKMARK,
                )
            };
            unsafe { ffi::EvtClose(bookmark) };
            if ok == 0 {
                // El bookmark ya no es válido (canal limpiado, evento purgado): se relee desde el
                // principio del canal — perder algo de historial es mucho mejor que dejar de leer
                // eventos para siempre (`docs/open-questions.md` J.7).
                tracing::warn!(
                    canal = %channel,
                    "el bookmark guardado ya no es válido; releyendo el canal desde el principio"
                );
            }
        }
    }

    let mut eventos = Vec::new();
    let mut ultimo_handle: ffi::EVT_HANDLE = 0;
    let mut lote = [0isize; 32];

    loop {
        let mut devueltos: u32 = 0;
        let ok = unsafe {
            ffi::EvtNext(
                resultset,
                lote.len() as u32,
                lote.as_mut_ptr(),
                0,
                0,
                &mut devueltos,
            )
        };
        if ok == 0 {
            let codigo = unsafe { ffi::GetLastError() };
            if codigo == ffi::ERROR_NO_MORE_ITEMS {
                break;
            }
            unsafe { ffi::EvtClose(resultset) };
            return Err(ErrorEventLog::Win32 {
                funcion: "EvtNext",
                codigo,
            });
        }

        for &handle in &lote[..devueltos as usize] {
            match evt_render(0, handle, ffi::EVT_RENDER_EVENT_XML) {
                Ok(xml) => {
                    // El proveedor se necesita antes de parsear el resto para pedir su mensaje;
                    // un fallo al extraerlo descarta el evento entero, no inventa uno vacío.
                    if let Some(provider) = extraer_atributo(&xml, "Provider", "Name") {
                        let mensaje = formatear_mensaje(&provider, handle);
                        match parse_event_xml(&xml, channel, mensaje) {
                            Ok(evento) => eventos.push(evento),
                            Err(e) => {
                                tracing::warn!(canal = %channel, error = ?e, "evento con XML irreconocible")
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(canal = %channel, error = ?e, "no se pudo renderizar un evento")
                }
            }
            if ultimo_handle != 0 {
                unsafe { ffi::EvtClose(ultimo_handle) };
            }
            ultimo_handle = handle;
        }
    }

    let nuevo_bookmark = if ultimo_handle != 0 {
        let bookmark = unsafe { ffi::EvtCreateBookmark(std::ptr::null()) };
        let resultado = if bookmark != 0 {
            let actualizado = unsafe { ffi::EvtUpdateBookmark(bookmark, ultimo_handle) };
            let xml = if actualizado != 0 {
                evt_render(0, bookmark, ffi::EVT_RENDER_BOOKMARK).ok()
            } else {
                None
            };
            unsafe { ffi::EvtClose(bookmark) };
            xml
        } else {
            None
        };
        unsafe { ffi::EvtClose(ultimo_handle) };
        resultado
    } else {
        // Sin eventos nuevos, el bookmark previo (si lo había) sigue siendo válido tal cual.
        bookmark_previo.map(|s| s.to_string())
    };

    unsafe { ffi::EvtClose(resultset) };
    Ok((eventos, nuevo_bookmark))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Capturado el 2026-09-05 de un Windows 11 real (`Get-WinEvent -LogName System | ... .ToXml()`):
    /// proveedor con manifiesto, `EventData` con atributos `Name`.
    const EVENTO_NTFS_98: &str = r#"<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='Microsoft-Windows-Ntfs' Guid='{3ff37a1c-a68d-4d6e-8c9b-f79e8b16c482}'/><EventID>98</EventID><Version>0</Version><Level>4</Level><Task>0</Task><Opcode>0</Opcode><Keywords>0x8000000000000002</Keywords><TimeCreated SystemTime='2026-09-05T05:57:42.9900998Z'/><EventRecordID>811025</EventRecordID><Correlation/><Execution ProcessID='4' ThreadID='23724'/><Channel>System</Channel><Computer>Ryzen</Computer><Security UserID='S-1-5-18'/></System><EventData><Data Name='DriveName'>E:</Data><Data Name='DeviceName'>\Device\HarddiskVolume24</Data><Data Name='CorruptionActionState'>0</Data></EventData></Event>"#;

    /// Capturado el mismo día: proveedor clásico sin manifiesto (`disk`), `EventID` con atributo
    /// `Qualifiers`, `EventData` sin nombres de campo.
    const EVENTO_DISK_158: &str = r#"<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='disk'/><EventID Qualifiers='32772'>158</EventID><Version>0</Version><Level>3</Level><Task>0</Task><Opcode>0</Opcode><Keywords>0x80000000000000</Keywords><TimeCreated SystemTime='2026-09-05T05:57:38.5428166Z'/><EventRecordID>811020</EventRecordID><Correlation/><Execution ProcessID='4' ThreadID='38720'/><Channel>System</Channel><Computer>Ryzen</Computer><Security/></System><EventData><Data>\Device\Harddisk1\DR19</Data><Data>1</Data><Binary>1B00000002003000000000009E000480000000000000000000000000000000000000000000000000</Binary></EventData></Event>"#;

    #[test]
    fn un_evento_con_manifiesto_se_parsea_completo() {
        let e = parse_event_xml(EVENTO_NTFS_98, "System", Some("mensaje".to_string())).unwrap();
        assert_eq!(e.provider, "Microsoft-Windows-Ntfs");
        assert_eq!(e.event_id, 98);
        assert_eq!(e.level, EventLevel::Information);
        assert_eq!(e.record_id, 811025);
        assert_eq!(e.channel, "System");
        assert_eq!(e.occurred_at_utc, "2026-09-05T05:57:42.9900998Z");
    }

    #[test]
    fn un_proveedor_clasico_con_qualifiers_se_parsea_igual() {
        let e = parse_event_xml(EVENTO_DISK_158, "System", None).unwrap();
        assert_eq!(e.provider, "disk");
        assert_eq!(e.event_id, 158, "el bajo orden del id, sin los qualifiers");
        assert_eq!(e.level, EventLevel::Warning);
        assert_eq!(e.record_id, 811020);
        assert_eq!(e.message, None);
    }

    #[test]
    fn un_xml_sin_bloque_system_falla_en_vez_de_inventar_campos() {
        let err = parse_event_xml("<Event></Event>", "System", None).unwrap_err();
        assert!(matches!(err, ErrorParseoEvento::CampoAusente("System")));
    }

    #[test]
    fn los_niveles_numericos_se_traducen_segun_la_convencion_de_windows() {
        for (n, esperado) in [
            (1u8, EventLevel::Critical),
            (2, EventLevel::Error),
            (3, EventLevel::Warning),
            (4, EventLevel::Information),
            (0, EventLevel::Information),
            (5, EventLevel::Information),
        ] {
            assert_eq!(nivel_desde_numero(n), esperado);
        }
    }

    #[test]
    fn el_filtro_encadena_los_proveedores_vigilados_con_or() {
        let filtro = construir_filtro(&["disk", "Ntfs"]);
        assert_eq!(filtro, "*[System[Provider[@Name='disk' or @Name='Ntfs']]]");
    }

    #[test]
    fn extraer_atributo_ignora_otras_etiquetas_con_el_mismo_nombre_de_atributo() {
        let xml = r#"<System><Provider Name='disk'/><Other Name='no-este'/></System>"#;
        assert_eq!(
            extraer_atributo(xml, "Provider", "Name").as_deref(),
            Some("disk")
        );
    }
}
