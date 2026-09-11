//! Informe HTML imprimible (T088 original; spec `009-informe-mejorado`, ADR-057).
//!
//! Autónomo: CSS embebido, sin fuentes ni recursos remotos, tema claro forzado y hoja de
//! impresión propia — debe abrirse igual en un equipo sin conexión. Es un **resumen legible**, no
//! el mismo volcado que el CSV/JSON: por cada disco, identidad, salud «a fecha de hoy», contadores
//! SMART con su variación en el intervalo, alertas del intervalo, eventos de Windows del intervalo
//! y dos mini-gráficas SVG (temperatura y actividad).
//!
//! Las alertas y los contadores muestran su clave técnica tal cual **salvo que `labels` traiga su
//! texto legible**: este módulo no posee ninguna traducción (ADR-030) — la interfaz se la pasa
//! para este render, igual que le pasa `includeSerials` o el destino (ADR-057).

use std::collections::HashMap;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use super::export::{etiqueta_dispositivo, RangoExport, SCHEMA_VERSION_HTML};
use super::minigrafica;
use crate::domain::salud::es_dato_caduco;
use crate::domain::tipos::{
    AlertGroup, AlertSeverity, AlertStatus, Device, HealthState, MetricSource,
};
use crate::persistence::{repo_alertas, repo_inventario, repo_metricas, repo_varios};

/// Tope de eventos por disco en el informe: por encima, se listan los más recientes y se dice
/// cuántos se omiten. Mismo espíritu que el tope de 1.500 puntos de serie (E.1): un informe
/// legible, no un volcado — quien necesite el detalle completo tiene el CSV/JSON.
const TOPE_EVENTOS: i64 = 50;

fn escapar_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

fn fmt_bytes(b: Option<i64>) -> String {
    let Some(b) = b else {
        return "No disponible".to_string();
    };
    let b = b as f64;
    const GB: f64 = 1_000_000_000.0;
    const TB: f64 = 1_000_000_000_000.0;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else {
        format!("{:.1} GB", b / GB)
    }
}

// ---------------------------------------------------------------- Frescura

/// El estado de una magnitud «a fecha de hoy»: si la última lectura es reciente, si es antigua
/// (el disco dejó de responder, pero **hubo** lectura), o si nunca hubo ninguna. Nunca se confunde
/// una lectura vieja con «No disponible» (constitución §I): «No disponible» se reserva a `Nunca`.
#[derive(Debug, Clone, PartialEq)]
pub enum Frescura {
    Actual(f64),
    Obsoleta { valor: f64, hace_texto: String },
    Nunca,
}

fn parse_utc(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s, &Rfc3339).ok()
}

/// Texto «hace X»; el informe es solo en español (como el resto de este HTML, que no tiene acceso
/// a los diccionarios — ver la nota del módulo).
fn hace_texto(segundos: i64) -> String {
    let s = segundos.max(0);
    if s < 3600 {
        format!("hace {} min", (s / 60).max(1))
    } else if s < 86_400 {
        format!("hace {} h", s / 3600)
    } else {
        format!("hace {} días", s / 86_400)
    }
}

/// `cadencia_esperada_segundos`: el margen de frescura de la métrica (`domain::salud::es_dato_caduco`,
/// el mismo criterio ×2 que usa el resto de la aplicación). 300 para lo que sale del ciclo SMART
/// (temperatura, desgaste, horas de encendido).
fn frescura(
    muestra: Option<(&str, f64)>,
    ahora: OffsetDateTime,
    cadencia_esperada_segundos: i64,
) -> Frescura {
    let Some((sampled_at, valor)) = muestra else {
        return Frescura::Nunca;
    };
    let Some(t) = parse_utc(sampled_at) else {
        return Frescura::Nunca;
    };
    let antiguedad = (ahora - t).whole_seconds();
    if es_dato_caduco(antiguedad, cadencia_esperada_segundos) {
        Frescura::Obsoleta {
            valor,
            hace_texto: hace_texto(antiguedad),
        }
    } else {
        Frescura::Actual(valor)
    }
}

fn frescura_html(f: &Frescura, unidad: &str) -> String {
    match f {
        Frescura::Actual(v) => format!("{}{unidad}", fmt_num(*v)),
        Frescura::Obsoleta { valor, hace_texto } => format!(
            "{}{unidad} <span class=\"stale\">(última lectura: {})</span>",
            fmt_num(*valor),
            escapar_html(hace_texto)
        ),
        Frescura::Nunca => "No disponible".to_string(),
    }
}

fn estado_html(e: HealthState) -> (&'static str, &'static str) {
    match e {
        HealthState::Ok => ("ok", "Correcto"),
        HealthState::Warn => ("warn", "Aviso"),
        HealthState::Crit => ("crit", "Crítico"),
        HealthState::Unknown => ("unknown", "Desconocido"),
    }
}

// ---------------------------------------------------------------- Contadores SMART con delta

#[derive(Debug, Clone, PartialEq)]
pub enum DeltaContador {
    Valor(f64),
    SinReferencia,
}

#[derive(Debug, Clone)]
pub struct ContadorConDelta {
    pub clave: String,
    pub valor_final: Option<f64>,
    pub delta: DeltaContador,
}

/// Contadores SMART del disco: el mismo conjunto que el panel «Contadores» del detalle de disco
/// (`latest_samples_by_source` con la fuente `Smartctl`, excluyendo las dos señales internas).
/// `valor_final` es la última muestra en o antes de `hasta`; el delta se calcula contra la última
/// muestra en o antes de `desde`; sin línea base, `SinReferencia` (nunca un valor absoluto
/// disfrazado de variación).
pub(crate) fn contadores_con_delta(
    conn: &rusqlite::Connection,
    device_id: &str,
    rango: RangoExport,
) -> rusqlite::Result<Vec<ContadorConDelta>> {
    let hasta = rango.hasta_como_texto();
    let desde = rango.desde_como_texto();
    let actuales =
        repo_metricas::latest_samples_by_source(conn, device_id, MetricSource::Smartctl)?;

    let mut out = Vec::new();
    for m in actuales {
        if matches!(
            m.metric_key.as_str(),
            "smart_query_ok" | "vendor_temp_limit_celsius"
        ) {
            continue;
        }
        let valor_final = repo_metricas::latest_device_sample_at_or_before(
            conn,
            device_id,
            &m.metric_key,
            &hasta,
        )?
        .and_then(|s| s.value_real);
        let base = repo_metricas::latest_device_sample_at_or_before(
            conn,
            device_id,
            &m.metric_key,
            &desde,
        )?
        .and_then(|s| s.value_real);
        let delta = match (valor_final, base) {
            (Some(f), Some(b)) => DeltaContador::Valor(f - b),
            _ => DeltaContador::SinReferencia,
        };
        out.push(ContadorConDelta {
            clave: m.metric_key,
            valor_final,
            delta,
        });
    }
    Ok(out)
}

fn fila_contador(c: &ContadorConDelta, labels: &HashMap<String, String>) -> String {
    let etiqueta = labels
        .get(&c.clave)
        .cloned()
        .unwrap_or_else(|| c.clave.clone());
    let valor = c
        .valor_final
        .map(fmt_num)
        .unwrap_or_else(|| "No disponible".to_string());
    let delta = match c.delta {
        DeltaContador::Valor(d) if d > 0.0 => format!("+{}", fmt_num(d)),
        DeltaContador::Valor(d) => fmt_num(d),
        DeltaContador::SinReferencia => "sin referencia".to_string(),
    };
    format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        escapar_html(&etiqueta),
        escapar_html(&valor),
        escapar_html(&delta)
    )
}

// ---------------------------------------------------------------- Alertas

/// Un grupo de alerta "cuenta" para el informe si tuvo alguna ocurrencia dentro del intervalo:
/// basta con que su ventana `[primera, última)` se solape con `[desde, hasta]`, no que naciera
/// dentro — una alerta larga que sigue activa desde antes del intervalo sigue siendo relevante.
pub(crate) fn alerta_en_rango(a: &AlertGroup, desde: &str, hasta: &str) -> bool {
    a.first_occurrence_at_utc.as_str() <= hasta && a.last_occurrence_at_utc.as_str() >= desde
}

fn fila_alerta(a: &AlertGroup, labels: &HashMap<String, String>) -> String {
    let texto = labels
        .get(&a.rule_key)
        .cloned()
        .unwrap_or_else(|| a.rule_key.clone());
    format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
        escapar_html(&texto),
        repo_alertas::severity_to_str(a.severity),
        repo_alertas::status_to_str(a.status),
        a.first_occurrence_at_utc,
        a.last_occurrence_at_utc,
        a.occurrence_count
    )
}

// ---------------------------------------------------------------- Volúmenes

struct VolumenInforme {
    etiqueta: String,
    letras: String,
    capacidad_bytes: Option<i64>,
    libre_bytes: Option<i64>,
}

fn volumenes_del_dispositivo(
    conn: &rusqlite::Connection,
    device_id: &str,
) -> rusqlite::Result<Vec<VolumenInforme>> {
    let ids = repo_inventario::volumes_for_device(conn, device_id)?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        let Some(v) = repo_inventario::get_volume(conn, &id)? else {
            continue;
        };
        let letras: Vec<String> = v
            .drive_letters_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();
        out.push(VolumenInforme {
            etiqueta: v.label.unwrap_or_default(),
            letras: letras.join(", "),
            capacidad_bytes: v.capacity_bytes,
            libre_bytes: v.free_bytes,
        });
    }
    Ok(out)
}

fn fila_volumen(v: &VolumenInforme) -> String {
    format!(
        "<tr><td>{} {}</td><td>{}</td><td>{}</td></tr>",
        escapar_html(&v.etiqueta),
        escapar_html(&v.letras),
        fmt_bytes(v.capacidad_bytes),
        fmt_bytes(v.libre_bytes)
    )
}

// ---------------------------------------------------------------- Sección de un disco

struct SeccionDiscoInforme {
    etiqueta: String,
    tipo: &'static str,
    bus: String,
    firmware: String,
    serie: String,
    estado: HealthState,
    temperatura: Frescura,
    desgaste: Frescura,
    horas: Frescura,
    volumenes: Vec<VolumenInforme>,
    contadores: Vec<ContadorConDelta>,
    alertas: Vec<AlertGroup>,
    eventos: Vec<crate::domain::tipos::SystemEvent>,
    eventos_omitidos: u32,
    minigrafica_temp: String,
    minigrafica_actividad: String,
    resumen_ia: Option<super::informe_ia::ResumenIaSeccion>,
}

#[allow(clippy::too_many_arguments)]
fn construir_seccion(
    conn: &rusqlite::Connection,
    device: &Device,
    rango: RangoExport,
    alertas_historial_completo: &[AlertGroup],
    include_serials: bool,
    resumen_ia: Option<super::informe_ia::ResumenIaSeccion>,
) -> rusqlite::Result<SeccionDiscoInforme> {
    let ahora = OffsetDateTime::now_utc();
    let desde_texto = rango.desde_como_texto();
    let hasta_texto = rango.hasta_como_texto();

    let leer = |clave: &str| -> rusqlite::Result<Option<(String, f64)>> {
        Ok(
            repo_metricas::latest_device_sample(conn, &device.id, clave)?
                .and_then(|m| m.value_real.map(|v| (m.sampled_at_utc, v))),
        )
    };
    let temp_muestra = leer("temperature_celsius")?;
    let desgaste_muestra = leer("percentage_used")?;
    let horas_muestra = leer("power_on_hours")?;

    let temperatura = frescura(
        temp_muestra.as_ref().map(|(t, v)| (t.as_str(), *v)),
        ahora,
        300,
    );
    let desgaste = frescura(
        desgaste_muestra.as_ref().map(|(t, v)| (t.as_str(), *v)),
        ahora,
        300,
    );
    let horas = frescura(
        horas_muestra.as_ref().map(|(t, v)| (t.as_str(), *v)),
        ahora,
        300,
    );

    let alertas_del_disco: Vec<&AlertGroup> = alertas_historial_completo
        .iter()
        .filter(|a| a.target_device_id.as_deref() == Some(device.id.as_str()))
        .collect();
    let peor_activa_o_reconocida = alertas_del_disco
        .iter()
        .filter(|a| matches!(a.status, AlertStatus::Active | AlertStatus::Acknowledged))
        .fold(None::<AlertSeverity>, |peor, a| match (peor, a.severity) {
            (Some(AlertSeverity::Critical), _) => Some(AlertSeverity::Critical),
            (_, AlertSeverity::Critical) => Some(AlertSeverity::Critical),
            (None, s) => Some(s),
            (Some(s), _) => Some(s),
        });
    let smart_supported = device.smartctl_path.is_some();
    let has_fresh_data = matches!(temperatura, Frescura::Actual(_));
    let estado = crate::domain::salud::device_state(
        smart_supported,
        has_fresh_data,
        peor_activa_o_reconocida,
    );

    let alertas: Vec<AlertGroup> = alertas_del_disco
        .into_iter()
        .filter(|a| alerta_en_rango(a, &desde_texto, &hasta_texto))
        .cloned()
        .collect();

    let volumenes = volumenes_del_dispositivo(conn, &device.id)?;
    let contadores = contadores_con_delta(conn, &device.id, rango)?;
    let (eventos, eventos_omitidos) = repo_varios::eventos_de_dispositivo_en_rango(
        conn,
        &device.id,
        &desde_texto,
        &hasta_texto,
        TOPE_EVENTOS,
    )?;
    let minigrafica_temp =
        minigrafica::minigrafica_device(conn, &device.id, "temperature_celsius", "°C", rango)?;
    let minigrafica_actividad =
        minigrafica::minigrafica_device(conn, &device.id, "activity_percent", "%", rango)?;
    // (ambas son `String`: sin datos, `minigrafica_device` ya produce el SVG "sin muestras")

    Ok(SeccionDiscoInforme {
        etiqueta: etiqueta_dispositivo(device),
        tipo: repo_inventario::device_type_to_str(device.device_type),
        bus: device.bus_type.clone().unwrap_or_default(),
        firmware: device.firmware.clone().unwrap_or_default(),
        serie: if include_serials {
            device.serial_number.clone().unwrap_or_default()
        } else {
            String::new()
        },
        estado,
        temperatura,
        desgaste,
        horas,
        volumenes,
        contadores,
        alertas,
        eventos,
        eventos_omitidos,
        minigrafica_temp,
        minigrafica_actividad,
        resumen_ia,
    })
}

/// El resumen con IA de la sección, o cadena vacía si no se pidió (`resumen_ia == None`, la
/// casilla estaba apagada). `Generado` se incrusta como **texto HTML-escapado** con
/// `white-space: pre-wrap` — nunca se renderiza el markdown de la respuesta (principio XVI: «texto
/// o markdown seguro, jamás HTML»; el modelo ya se le pidió texto plano, ver `SYSTEM_INFORME_ES`).
/// `NoDisponible` pinta el texto legible de `motivo_key` si `labels` lo trae (mismo mecanismo que
/// las alertas, ADR-030: el backend no manda frases); sin entrada, una nota genérica.
fn bloque_resumen_ia(
    r: &Option<super::informe_ia::ResumenIaSeccion>,
    labels: &HashMap<String, String>,
) -> String {
    use super::informe_ia::ResumenIaSeccion;
    match r {
        None => String::new(),
        Some(ResumenIaSeccion::Generado {
            markdown,
            modelo_usado,
        }) => format!(
            r#"<h3>Resumen con IA</h3>
  <div class="resumen-ia">
    <p class="meta">Orientación generada por IA (modelo: {}) — no es un diagnóstico.</p>
    <div class="resumen-ia-texto">{}</div>
  </div>"#,
            escapar_html(modelo_usado),
            escapar_html(markdown)
        ),
        Some(ResumenIaSeccion::NoDisponible { motivo_key }) => {
            let motivo = labels
                .get(motivo_key)
                .cloned()
                .unwrap_or_else(|| "no disponible en este momento".to_string());
            format!(
                r#"<h3>Resumen con IA</h3>
  <p class="meta">Resumen con IA {}.</p>"#,
                escapar_html(&motivo)
            )
        }
    }
}

fn render_seccion(s: &SeccionDiscoInforme, labels: &HashMap<String, String>) -> String {
    let etiqueta = escapar_html(&s.etiqueta);
    let (estado_clase, estado_texto) = estado_html(s.estado);
    let serie_html = if s.serie.is_empty() {
        String::new()
    } else {
        format!(" · N.º de serie: {}", escapar_html(&s.serie))
    };
    let bus = escapar_html(&s.bus);
    let firmware = if s.firmware.is_empty() {
        "No disponible".to_string()
    } else {
        escapar_html(&s.firmware)
    };

    let filas_volumenes = if s.volumenes.is_empty() {
        "<tr><td colspan=\"3\">Sin volúmenes montados.</td></tr>".to_string()
    } else {
        s.volumenes
            .iter()
            .map(fila_volumen)
            .collect::<Vec<_>>()
            .join("\n")
    };

    let filas_contadores = if s.contadores.is_empty() {
        "<tr><td colspan=\"3\">Sin contadores SMART.</td></tr>".to_string()
    } else {
        s.contadores
            .iter()
            .map(|c| fila_contador(c, labels))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let filas_alertas = if s.alertas.is_empty() {
        "<tr><td colspan=\"6\">Sin alertas en el periodo.</td></tr>".to_string()
    } else {
        s.alertas
            .iter()
            .map(|a| fila_alerta(a, labels))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let eventos_nota = if s.eventos_omitidos > 0 {
        format!(
            "<p class=\"meta\">… y {} más. El CSV o el JSON traen el detalle completo.</p>",
            s.eventos_omitidos
        )
    } else {
        String::new()
    };
    let filas_eventos = if s.eventos.is_empty() {
        "<tr><td colspan=\"4\">Sin eventos en el periodo.</td></tr>".to_string()
    } else {
        s.eventos
            .iter()
            .map(|e| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    e.occurred_at_utc,
                    escapar_html(&e.provider),
                    e.event_id,
                    escapar_html(e.message.as_deref().unwrap_or("Sin mensaje"))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let grafica_temp = &s.minigrafica_temp;
    let grafica_actividad = &s.minigrafica_actividad;

    format!(
        r#"<section>
  <h2>{etiqueta} <span class="estado estado-{estado_clase}">{estado_texto}</span></h2>
  <p class="meta">{tipo}{serie_html} · Bus: {bus} · Firmware: {firmware}</p>
  <div class="salud">
    <div><span class="etiqueta-salud">Temperatura</span><br>{temperatura}</div>
    <div><span class="etiqueta-salud">Desgaste</span><br>{desgaste}</div>
    <div><span class="etiqueta-salud">Horas de encendido</span><br>{horas}</div>
  </div>

  <h3>Volúmenes</h3>
  <table>
    <thead><tr><th>Volumen</th><th>Capacidad</th><th>Espacio libre</th></tr></thead>
    <tbody>{filas_volumenes}</tbody>
  </table>

  <h3>Temperatura en el periodo</h3>
  {grafica_temp}
  <h3>Actividad en el periodo</h3>
  {grafica_actividad}

  <h3>Contadores SMART</h3>
  <table>
    <thead><tr><th>Contador</th><th>Valor</th><th>Variación en el periodo</th></tr></thead>
    <tbody>{filas_contadores}</tbody>
  </table>

  <h3>Alertas en el periodo</h3>
  <table>
    <thead><tr><th>Alerta</th><th>Severidad</th><th>Estado</th><th>Primera vez</th><th>Última vez</th><th>Veces</th></tr></thead>
    <tbody>{filas_alertas}</tbody>
  </table>

  <h3>Eventos de Windows en el periodo</h3>
  <table>
    <thead><tr><th>Fecha</th><th>Proveedor</th><th>ID</th><th>Mensaje</th></tr></thead>
    <tbody>{filas_eventos}</tbody>
  </table>
  {eventos_nota}
  {resumen_ia}
</section>"#,
        tipo = s.tipo,
        temperatura = frescura_html(&s.temperatura, " °C"),
        desgaste = frescura_html(&s.desgaste, " %"),
        horas = frescura_html(&s.horas, " h"),
        resumen_ia = bloque_resumen_ia(&s.resumen_ia, labels),
    )
}

/// `dispositivos` ya viene filtrado por el llamador (`deviceIds`, o todos los monitorizados);
/// `alertas` trae el historial completo de grupos, sin filtrar todavía por dispositivo ni rango —
/// este módulo hace ambos filtros. `labels` traduce claves técnicas (reglas de alerta y
/// contadores SMART) a texto legible; una clave ausente cae a la clave cruda. `resumenes_ia`, si
/// se pasa, es el resultado (ya calculado) del resumen con IA por disco — `None` cuando la casilla
/// «incluir resumen con IA» estaba apagada; una clave de `device_id` ausente en el mapa se trata
/// igual que sin resumen para ese disco.
#[allow(clippy::too_many_arguments)]
pub fn generar_html(
    conn: &rusqlite::Connection,
    dispositivos: &[Device],
    alertas: &[AlertGroup],
    rango: RangoExport,
    include_serials: bool,
    labels: &HashMap<String, String>,
    resumenes_ia: Option<&HashMap<String, super::informe_ia::ResumenIaSeccion>>,
) -> rusqlite::Result<String> {
    let desde_texto = rango.desde_como_texto();
    let hasta_texto = rango.hasta_como_texto();
    let generado = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();

    let mut secciones = String::new();
    for d in dispositivos {
        let resumen_ia = resumenes_ia.and_then(|m| m.get(&d.id)).cloned();
        let seccion = construir_seccion(conn, d, rango, alertas, include_serials, resumen_ia)?;
        secciones.push_str(&render_seccion(&seccion, labels));
        secciones.push('\n');
    }
    if dispositivos.is_empty() {
        secciones.push_str("<p>No hay discos incluidos en este informe.</p>");
    }

    Ok(format!(
        r#"<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<title>Informe de SmartDisk Monitor</title>
<style>
  :root {{ color-scheme: light; }}
  body {{ font-family: system-ui, -apple-system, "Segoe UI", sans-serif; margin: 2rem; color: #1a1a1a; background: #fff; }}
  h1 {{ font-size: 1.4rem; margin-bottom: .25rem; }}
  h2 {{ font-size: 1.2rem; margin-top: 2rem; border-bottom: 1px solid #ccc; padding-bottom: .25rem; }}
  h3 {{ font-size: .95rem; margin: 1.25rem 0 .25rem; color: #333; }}
  table {{ border-collapse: collapse; width: 100%; margin-top: .25rem; }}
  th, td {{ border: 1px solid #ccc; padding: .4rem .6rem; text-align: left; font-size: .85rem; }}
  th {{ background: #f2f2f2; }}
  .meta {{ color: #555; font-size: .85rem; }}
  .stale {{ color: #a15c00; }}
  .salud {{ display: flex; gap: 1.5rem; margin: .5rem 0 1rem; }}
  .etiqueta-salud {{ color: #666; font-size: .75rem; text-transform: uppercase; letter-spacing: .02em; }}
  .estado {{ font-size: .75rem; padding: .1rem .5rem; border-radius: 999px; vertical-align: middle; }}
  .estado-ok {{ background: #e3f6e8; color: #1a7a34; }}
  .estado-warn {{ background: #fdf0d5; color: #9a6300; }}
  .estado-crit {{ background: #fbe4e4; color: #a11d1d; }}
  .estado-unknown {{ background: #eee; color: #666; }}
  .resumen-ia {{ background: #f6f4fb; border: 1px solid #e2ddf2; border-radius: .5rem; padding: .75rem 1rem; margin-top: .25rem; }}
  .resumen-ia-texto {{ white-space: pre-wrap; font-size: .85rem; margin: .35rem 0 0; }}
  @media print {{
    body {{ margin: 0; }}
    section {{ break-inside: avoid; }}
  }}
</style>
</head>
<body>
<h1>Informe de SmartDisk Monitor</h1>
<p class="meta">schemaVersion: {SCHEMA_VERSION_HTML} · Generado: {generado}<br>Intervalo: {desde_texto} – {hasta_texto}</p>
{secciones}
</body>
</html>
"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{
        DeviceType, IdentityConfidence, MetricQuality, MetricSource, MetricTarget,
    };
    use crate::persistence::{db, repo_inventario};

    fn conn_de_prueba() -> rusqlite::Connection {
        let dir = crate::test_util::temp_dir_unico("informe");
        db::open(&dir).unwrap().0
    }

    fn dispositivo(id: &str, alias: Option<&str>) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: Some("S1B2C3".to_string()),
            model: "Modelo de prueba".to_string(),
            manufacturer: None,
            firmware: Some("FW1".to_string()),
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: Some(r"\\.\PhysicalDrive0".to_string()),
            capacity_bytes: Some(1_000_000_000),
            alias: alias.map(str::to_string),
            monitoring_enabled: true,
            first_seen_at: "2026-09-01T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
    }

    fn insertar_dispositivo(conn: &rusqlite::Connection, d: &Device) {
        repo_inventario::upsert_device(conn, d).unwrap();
    }

    fn insertar_muestra(
        conn: &rusqlite::Connection,
        device_id: &str,
        clave: &str,
        valor: f64,
        cuando: &str,
        source: MetricSource,
    ) {
        crate::persistence::repo_metricas::insert_sample(
            conn,
            &crate::domain::tipos::MetricSample {
                target: MetricTarget::Device(device_id.to_string()),
                metric_key: clave.to_string(),
                value_real: Some(valor),
                value_integer: None,
                unit: "count".to_string(),
                sampled_at_utc: cuando.to_string(),
                source,
                quality: MetricQuality::Exact,
                resolution: crate::domain::tipos::Resolution::Raw,
            },
        )
        .unwrap();
    }

    fn alerta(device_id: &str, first: &str, last: &str) -> AlertGroup {
        AlertGroup {
            id: "a1".to_string(),
            deduplication_key: "smart.wear_high|device:d1".to_string(),
            rule_key: "smart.wear_high".to_string(),
            target_device_id: Some(device_id.to_string()),
            target_volume_id: None,
            severity: AlertSeverity::Warning,
            status: AlertStatus::Active,
            muted_until: None,
            cycle: 1,
            first_occurrence_at_utc: first.to_string(),
            last_occurrence_at_utc: last.to_string(),
            occurrence_count: 3,
            acknowledged_at_utc: None,
            resolved_at_utc: None,
            archived_at_utc: None,
            ignored_at_utc: None,
            last_value_real: Some(92.0),
            context_json: None,
        }
    }

    fn rango() -> RangoExport {
        RangoExport {
            desde: OffsetDateTime::parse("2026-09-04T00:00:00Z", &Rfc3339).unwrap(),
            hasta: OffsetDateTime::parse("2026-09-04T10:00:00Z", &Rfc3339).unwrap(),
        }
    }

    fn etiquetas_de_prueba() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert(
            "smart.wear_high".to_string(),
            "Desgaste por encima del umbral".to_string(),
        );
        m
    }

    #[test]
    fn el_numero_de_serie_solo_aparece_si_se_pide() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", None);
        insertar_dispositivo(&conn, &d);

        let con_serie = generar_html(
            &conn,
            std::slice::from_ref(&d),
            &[],
            rango(),
            true,
            &HashMap::new(),
            None,
        )
        .unwrap();
        assert!(con_serie.contains("S1B2C3"));

        let sin_serie =
            generar_html(&conn, &[d], &[], rango(), false, &HashMap::new(), None).unwrap();
        assert!(!sin_serie.contains("S1B2C3"));
    }

    #[test]
    fn una_alerta_fuera_del_intervalo_no_aparece() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);
        let vieja = alerta("d1", "2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
        let html = generar_html(
            &conn,
            &[d],
            &[vieja],
            rango(),
            false,
            &etiquetas_de_prueba(),
            None,
        )
        .unwrap();
        assert!(html.contains("Sin alertas en el periodo."));
        assert!(!html.contains("smart.wear_high"));
        assert!(!html.contains("Desgaste por encima del umbral"));
    }

    #[test]
    fn una_alerta_dentro_del_intervalo_aparece_con_frase_legible_no_con_la_clave_cruda() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);
        let activa = alerta("d1", "2026-09-04T01:00:00Z", "2026-09-04T05:00:00Z");
        let html = generar_html(
            &conn,
            &[d],
            &[activa],
            rango(),
            false,
            &etiquetas_de_prueba(),
            None,
        )
        .unwrap();
        assert!(html.contains("Desgaste por encima del umbral"));
        assert!(!html.contains("smart.wear_high"));
        assert!(html.contains("warn"));
    }

    #[test]
    fn una_alerta_sin_objeto_de_disco_no_aparece_en_ninguna_seccion() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);
        let mut sin_objeto = alerta("d1", "2026-09-04T01:00:00Z", "2026-09-04T05:00:00Z");
        sin_objeto.target_device_id = None;
        sin_objeto.rule_key = "events.filesystem_error".to_string();
        let html = generar_html(
            &conn,
            &[d],
            &[sin_objeto],
            rango(),
            false,
            &HashMap::new(),
            None,
        )
        .unwrap();
        assert!(!html.contains("events.filesystem_error"));
        assert!(html.contains("Sin alertas en el periodo."));
    }

    #[test]
    fn un_contador_smart_muestra_su_delta_y_sin_linea_base_dice_sin_referencia() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);
        // Con línea base (antes de "desde") y valor final dentro del intervalo.
        insertar_muestra(
            &conn,
            "d1",
            "power_cycles",
            100.0,
            "2026-09-03T00:00:00Z",
            MetricSource::Smartctl,
        );
        insertar_muestra(
            &conn,
            "d1",
            "power_cycles",
            112.0,
            "2026-09-04T05:00:00Z",
            MetricSource::Smartctl,
        );
        // Sin línea base antes del intervalo: solo una muestra, dentro del intervalo.
        insertar_muestra(
            &conn,
            "d1",
            "media_errors_total",
            2.0,
            "2026-09-04T06:00:00Z",
            MetricSource::Smartctl,
        );

        let html = generar_html(&conn, &[d], &[], rango(), false, &HashMap::new(), None).unwrap();
        assert!(html.contains("power_cycles"));
        assert!(html.contains("+12"));
        assert!(html.contains("media_errors_total"));
        assert!(html.contains("sin referencia"));
    }

    #[test]
    fn un_disco_con_lectura_smart_antigua_muestra_la_antiguedad_no_no_disponible() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);
        // Una lectura muy anterior al intervalo: el disco dejó de responder.
        insertar_muestra(
            &conn,
            "d1",
            "temperature_celsius",
            41.0,
            "2026-01-01T00:00:00Z",
            MetricSource::Smartctl,
        );

        let html = generar_html(&conn, &[d], &[], rango(), false, &HashMap::new(), None).unwrap();
        assert!(html.contains("última lectura:"));
        assert!(!html.contains("No disponible") || html.matches("No disponible").count() >= 1);
    }

    #[test]
    fn un_disco_que_nunca_tuvo_smart_muestra_no_disponible() {
        let conn = conn_de_prueba();
        let mut d = dispositivo("d1", Some("Mi disco"));
        d.smartctl_path = None;
        insertar_dispositivo(&conn, &d);

        let html = generar_html(&conn, &[d], &[], rango(), false, &HashMap::new(), None).unwrap();
        assert!(html.contains("No disponible"));
        assert!(!html.contains("última lectura:"));
    }

    #[test]
    fn un_alias_con_caracteres_html_se_escapa() {
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("<script>alert(1)</script>"));
        insertar_dispositivo(&conn, &d);
        let html = generar_html(&conn, &[d], &[], rango(), false, &HashMap::new(), None).unwrap();
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn el_documento_es_autonomo_sin_recursos_remotos() {
        let conn = conn_de_prueba();
        let html = generar_html(&conn, &[], &[], rango(), false, &HashMap::new(), None).unwrap();
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
        assert!(!html.contains("<script src"));
        assert!(!html.contains("<link "));
        assert!(html.contains("<style>"));
    }

    #[test]
    fn un_resumen_ia_generado_se_incrusta_escapado_con_el_modelo_y_sin_renderizar_markdown() {
        use super::super::informe_ia::ResumenIaSeccion;
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);

        let mut resumenes = HashMap::new();
        resumenes.insert(
            "d1".to_string(),
            ResumenIaSeccion::Generado {
                markdown: "**Ojo**: <b>revisa</b> la temperatura.".to_string(),
                modelo_usado: "vendor/modelo-x".to_string(),
            },
        );

        let html = generar_html(
            &conn,
            &[d],
            &[],
            rango(),
            false,
            &HashMap::new(),
            Some(&resumenes),
        )
        .unwrap();
        assert!(html.contains("Resumen con IA"));
        assert!(html.contains("vendor/modelo-x"));
        // Escapado como texto, nunca HTML/markdown renderizado (principio XVI).
        assert!(!html.contains("<b>revisa</b>"));
        assert!(html.contains("&lt;b&gt;revisa&lt;/b&gt;"));
        assert!(html.contains("**Ojo**"));
    }

    #[test]
    fn un_resumen_ia_no_disponible_usa_la_etiqueta_si_la_hay_o_una_nota_generica() {
        use super::super::informe_ia::ResumenIaSeccion;
        let conn = conn_de_prueba();
        let d = dispositivo("d1", Some("Mi disco"));
        insertar_dispositivo(&conn, &d);

        let mut resumenes = HashMap::new();
        resumenes.insert(
            "d1".to_string(),
            ResumenIaSeccion::NoDisponible {
                motivo_key: "error.ia.timeout".to_string(),
            },
        );

        let sin_etiqueta = generar_html(
            &conn,
            std::slice::from_ref(&d),
            &[],
            rango(),
            false,
            &HashMap::new(),
            Some(&resumenes),
        )
        .unwrap();
        assert!(sin_etiqueta.contains("no disponible en este momento"));

        let mut labels = HashMap::new();
        labels.insert(
            "error.ia.timeout".to_string(),
            "ha tardado demasiado en responder".to_string(),
        );
        let con_etiqueta =
            generar_html(&conn, &[d], &[], rango(), false, &labels, Some(&resumenes)).unwrap();
        assert!(con_etiqueta.contains("ha tardado demasiado en responder"));
    }
}
