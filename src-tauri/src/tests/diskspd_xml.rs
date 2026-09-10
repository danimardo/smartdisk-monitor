//! Parseo del XML de DiskSpd (`-Rxml`) — la prueba de Rendimiento (ADR-053).
//!
//! **A mano, sin dependencia nueva** (constitución §III): el bloque de resultados de DiskSpd es
//! plano y estable, igual que el XML del Event Log que `collectors::event_log` ya deserializa así.
//! Helpers `extraer_*` sobre `str::find`, struct explícito de destino — nunca `serde_json::Value`
//! (`backend-rust.md`).
//!
//! Una salida que no trae los campos esperados, que lleva un `<Error>` de DiskSpd, o que no se
//! puede recorrer → [`SalidaIlegible`]. **Nunca** una cifra a cero (constitución §I).

use super::diskspd::Sentido;

/// Lo que se extrae de una medición de DiskSpd. Todos los campos vienen del XML; ninguno se rellena
/// con un valor por defecto.
#[derive(Debug, Clone, PartialEq)]
pub struct SalidaDiskspd {
    /// `<TestTimeSeconds>`: segundos reales de medición (sin el calentamiento `-W`).
    pub test_time_s: f64,
    /// Bytes movidos en el sentido de la medición (suma de los `<Thread>`).
    pub bytes: u64,
    /// Operaciones de E/S en el sentido de la medición (suma de los `<Thread>`).
    pub ops: u64,
    /// Latencia media en milisegundos, del bloque `<Latency>` agregado.
    pub avg_latency_ms: f64,
    /// Versión de DiskSpd, de `<System><Tool><Version>` (FR-012).
    pub tool_version: String,
}

/// El XML de DiskSpd no se pudo interpretar: campos ausentes, `<Error>` de la herramienta, o
/// estructura irreconocible. La ejecución falla con `test.tool_output_unreadable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SalidaIlegible(pub String);

impl std::fmt::Display for SalidaIlegible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "salida de DiskSpd ilegible: {}", self.0)
    }
}

/// Interpreta el `-Rxml` de una medición. `sentido` decide qué contadores se leen (`Read*` vs
/// `Write*`) y qué media de latencia.
pub fn parsear(xml: &str, sentido: Sentido) -> Result<SalidaDiskspd, SalidaIlegible> {
    // El bloque de resultados es el último `<TimeSpan>`: el primero es el eco del perfil, sin datos.
    let inicio_resultados = xml
        .rfind("<TimeSpan>")
        .ok_or_else(|| SalidaIlegible("no hay bloque <TimeSpan> de resultados".to_owned()))?;
    let bloque = &xml[inicio_resultados..];

    if let Some(err) = extraer_texto(bloque, "Error") {
        return Err(SalidaIlegible(format!("DiskSpd informó: {}", err.trim())));
    }

    let test_time_s = extraer_texto(bloque, "TestTimeSeconds")
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|s| *s > 0.0)
        .ok_or_else(|| {
            SalidaIlegible("falta <TestTimeSeconds> o no es un número > 0".to_owned())
        })?;

    let (tag_bytes, tag_ops) = match sentido {
        Sentido::Lectura => ("ReadBytes", "ReadCount"),
        Sentido::Escritura => ("WriteBytes", "WriteCount"),
    };
    let bytes = sumar_todos(bloque, tag_bytes);
    let ops = sumar_todos(bloque, tag_ops);
    if bytes == 0 || ops == 0 {
        return Err(SalidaIlegible(format!(
            "sin datos de E/S en el sentido esperado (<{tag_bytes}>/<{tag_ops}> suman 0)"
        )));
    }

    let avg_latency_ms = latencia_media(bloque, sentido).ok_or_else(|| {
        SalidaIlegible("falta la latencia media en el bloque <Latency>".to_owned())
    })?;

    let tool_version = extraer_texto(xml, "Version")
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| SalidaIlegible("falta <System><Tool><Version>".to_owned()))?;

    Ok(SalidaDiskspd {
        test_time_s,
        bytes,
        ops,
        avg_latency_ms,
        tool_version,
    })
}

fn latencia_media(bloque: &str, sentido: Sentido) -> Option<f64> {
    let tag = match sentido {
        Sentido::Lectura => "AverageReadMilliseconds",
        Sentido::Escritura => "AverageWriteMilliseconds",
    };
    extraer_texto(bloque, tag)
        .or_else(|| extraer_texto(bloque, "AverageTotalMilliseconds"))
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
}

/// Suma el contenido numérico de **todas** las ocurrencias de `<tag>...</tag>` dentro de `xml`
/// (un `<Thread>` por hilo; los perfiles fijos usan `-t1`, pero se suma por si algún día cambia).
fn sumar_todos(xml: &str, tag: &str) -> u64 {
    let apertura = format!("<{tag}>");
    let cierre = format!("</{tag}>");
    let mut total: u64 = 0;
    let mut resto = xml;
    while let Some(i) = resto.find(&apertura) {
        let tras = &resto[i + apertura.len()..];
        let Some(j) = tras.find(&cierre) else { break };
        if let Ok(v) = tras[..j].trim().parse::<u64>() {
            total = total.saturating_add(v);
        }
        resto = &tras[j + cierre.len()..];
    }
    total
}

/// Contenido de la primera ocurrencia de `<tag>...</tag>` (o `<tag ...>...</tag>`), sin recortar.
fn extraer_texto(xml: &str, tag: &str) -> Option<String> {
    let apertura_corta = format!("<{tag}>");
    let apertura_atributos = format!("<{tag} ");
    let cierre = format!("</{tag}>");
    let inicio = xml
        .find(&apertura_corta)
        .or_else(|| xml.find(&apertura_atributos))?;
    let inicio_contenido = xml[inicio..].find('>')? + inicio + 1;
    let fin_contenido = xml[inicio_contenido..].find(&cierre)? + inicio_contenido;
    Some(xml[inicio_contenido..fin_contenido].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(nombre: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/tests/fixtures/diskspd")
                .join(nombre),
        )
        .unwrap_or_else(|e| panic!("no se pudo leer el fixture {nombre}: {e}"))
    }

    #[test]
    fn seq1m_q8_lectura_da_caudal_iops_y_latencia_reales() {
        let s = parsear(&fixture("seq1m_q8_read.xml"), Sentido::Lectura).unwrap();
        assert!(s.test_time_s > 0.0);
        assert!(s.bytes > 0);
        assert!(s.ops > 0);
        assert!(s.avg_latency_ms > 0.0);
        assert_eq!(s.tool_version, "2.3.0");
        // El caudal (bytes/s) debe ser plausible para 1 MiB secuencial: > 10 MB/s, < 100 GB/s.
        let mb_s = s.bytes as f64 / s.test_time_s / 1_000_000.0;
        assert!(
            (10.0..100_000.0).contains(&mb_s),
            "caudal irreal: {mb_s} MB/s"
        );
    }

    #[test]
    fn seq1m_q1_escritura_lee_los_contadores_de_escritura() {
        let s = parsear(&fixture("seq1m_q1_write.xml"), Sentido::Escritura).unwrap();
        assert!(s.bytes > 0);
        assert!(s.ops > 0);
        assert!(s.avg_latency_ms > 0.0);
        assert_eq!(s.tool_version, "2.3.0");
    }

    #[test]
    fn rnd4k_q32_lectura_y_rnd4k_q1_escritura_parsean() {
        let r = parsear(&fixture("rnd4k_q32_read.xml"), Sentido::Lectura).unwrap();
        assert!(
            r.ops > r.bytes / (4096 * 2),
            "4K aleatorio: muchas ops pequeñas"
        );
        let w = parsear(&fixture("rnd4k_q1_write.xml"), Sentido::Escritura).unwrap();
        assert!(w.bytes > 0 && w.ops > 0);
    }

    #[test]
    fn pedir_el_sentido_equivocado_no_inventa_ceros() {
        // El fixture de lectura no tiene bytes de escritura → error, nunca una fila a 0.
        let err = parsear(&fixture("seq1m_q8_read.xml"), Sentido::Escritura).unwrap_err();
        assert!(err.0.contains("sin datos de E/S"));
    }

    #[test]
    fn una_ejecucion_que_diskspd_aborto_es_ilegible_no_ceros() {
        let err = parsear(&fixture("error_target_inaccesible.xml"), Sentido::Lectura).unwrap_err();
        assert!(err.0.contains("DiskSpd informó") || err.0.contains("TestTimeSeconds"));
    }

    #[test]
    fn un_xml_vacio_o_sin_timespan_es_ilegible() {
        assert!(parsear("", Sentido::Lectura).is_err());
        assert!(parsear("<Results></Results>", Sentido::Lectura).is_err());
    }

    #[test]
    fn sin_version_de_herramienta_es_ilegible() {
        // TimeSpan con datos pero sin bloque <System><Tool><Version>.
        let xml = "<Results><TimeSpan><TestTimeSeconds>2.00</TestTimeSeconds>\
            <Thread><Target><ReadBytes>1048576</ReadBytes><ReadCount>1</ReadCount>\
            <AverageReadLatencyMilliseconds>1.0</AverageReadLatencyMilliseconds></Target></Thread>\
            <Latency><AverageReadMilliseconds>1.0</AverageReadMilliseconds></Latency>\
            </TimeSpan></Results>";
        let err = parsear(xml, Sentido::Lectura).unwrap_err();
        assert!(err.0.contains("Version"));
    }
}
