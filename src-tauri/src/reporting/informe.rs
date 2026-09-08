//! Informe HTML imprimible (T088, `docs/product-specification.md` §9, `docs/open-questions.md`
//! J.30).
//!
//! Autónomo: CSS embebido, sin fuentes ni recursos remotos, tema claro forzado y hoja de
//! impresión propia — debe abrirse igual en un equipo sin conexión. Es un **resumen legible**, no
//! el mismo volcado que el CSV/JSON: identidad y salud del dispositivo más las alertas del
//! intervalo. Las alertas muestran `ruleKey` tal cual (p. ej. `smart.wear_high`), no una frase
//! humana: este HTML lo genera el backend, sin acceso a los diccionarios de `$lib/i18n`
//! (ADR-030, J.30).

use time::format_description::well_known::Rfc3339;

use super::export::{etiqueta_dispositivo, RangoExport, SCHEMA_VERSION};
use crate::domain::tipos::{AlertGroup, Device};
use crate::persistence::{repo_alertas, repo_inventario};

fn escapar_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Un grupo de alerta "cuenta" para el informe si tuvo alguna ocurrencia dentro del intervalo:
/// basta con que su ventana `[primera, última)` se solape con `[desde, hasta]`, no que naciera
/// dentro — una alerta larga que sigue activa desde antes del intervalo sigue siendo relevante.
fn alerta_en_rango(a: &AlertGroup, desde: &str, hasta: &str) -> bool {
    a.first_occurrence_at_utc.as_str() <= hasta && a.last_occurrence_at_utc.as_str() >= desde
}

fn fila_alerta(a: &AlertGroup) -> String {
    format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
        escapar_html(&a.rule_key),
        repo_alertas::severity_to_str(a.severity),
        repo_alertas::status_to_str(a.status),
        a.first_occurrence_at_utc,
        a.last_occurrence_at_utc,
        a.occurrence_count
    )
}

fn seccion_dispositivo(d: &Device, alertas: &[&AlertGroup], include_serials: bool) -> String {
    let etiqueta = escapar_html(&etiqueta_dispositivo(d));
    let tipo = repo_inventario::device_type_to_str(d.device_type);
    let serie = if include_serials {
        d.serial_number
            .as_deref()
            .map(|s| format!(" · N.º de serie: {}", escapar_html(s)))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let filas_alertas = if alertas.is_empty() {
        "<tr><td colspan=\"6\">Sin alertas en el intervalo.</td></tr>".to_string()
    } else {
        alertas
            .iter()
            .map(|a| fila_alerta(a))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<section>
  <h2>{etiqueta}</h2>
  <p class="meta">{tipo}{serie}</p>
  <table>
    <thead>
      <tr><th>Regla</th><th>Severidad</th><th>Estado</th><th>Primera vez</th><th>Última vez</th><th>Veces</th></tr>
    </thead>
    <tbody>
      {filas_alertas}
    </tbody>
  </table>
</section>"#
    )
}

/// `dispositivos` ya viene filtrado por el llamador (`deviceIds`, o todos los monitorizados);
/// `alertas` trae el historial completo de grupos, sin filtrar todavía por dispositivo ni rango —
/// este módulo hace ambos filtros, para que el llamador no tenga que conocer la regla de
/// solapamiento de `alerta_en_rango`.
pub fn generar_html(
    dispositivos: &[Device],
    alertas: &[AlertGroup],
    rango: RangoExport,
    include_serials: bool,
) -> String {
    let rfc3339 = &Rfc3339;
    let desde_texto = rango.desde.format(rfc3339).unwrap_or_default();
    let hasta_texto = rango.hasta.format(rfc3339).unwrap_or_default();
    let generado = time::OffsetDateTime::now_utc()
        .format(rfc3339)
        .unwrap_or_default();

    let secciones = dispositivos
        .iter()
        .map(|d| {
            let alertas_del_dispositivo: Vec<&AlertGroup> = alertas
                .iter()
                .filter(|a| {
                    a.target_device_id.as_deref() == Some(d.id.as_str())
                        && alerta_en_rango(a, &desde_texto, &hasta_texto)
                })
                .collect();
            seccion_dispositivo(d, &alertas_del_dispositivo, include_serials)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<title>Informe de SmartDisk Monitor</title>
<style>
  :root {{ color-scheme: light; }}
  body {{ font-family: system-ui, -apple-system, "Segoe UI", sans-serif; margin: 2rem; color: #1a1a1a; background: #fff; }}
  h1 {{ font-size: 1.4rem; margin-bottom: .25rem; }}
  h2 {{ font-size: 1.1rem; margin-top: 2rem; border-bottom: 1px solid #ccc; padding-bottom: .25rem; }}
  table {{ border-collapse: collapse; width: 100%; margin-top: .5rem; }}
  th, td {{ border: 1px solid #ccc; padding: .4rem .6rem; text-align: left; font-size: .85rem; }}
  th {{ background: #f2f2f2; }}
  .meta {{ color: #555; font-size: .85rem; }}
  @media print {{
    body {{ margin: 0; }}
    section {{ break-inside: avoid; }}
  }}
</style>
</head>
<body>
<h1>Informe de SmartDisk Monitor</h1>
<p class="meta">schemaVersion: {SCHEMA_VERSION} · Generado: {generado}<br>Intervalo: {desde_texto} – {hasta_texto}</p>
{secciones}
</body>
</html>
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos::{AlertSeverity, AlertStatus, DeviceType, IdentityConfidence};

    fn dispositivo(id: &str, alias: Option<&str>) -> Device {
        Device {
            id: id.to_string(),
            fingerprint: format!("huella-{id}"),
            identity_confidence: IdentityConfidence::Fingerprint,
            serial_number: Some("S1B2C3".to_string()),
            model: "Modelo de prueba".to_string(),
            manufacturer: None,
            firmware: None,
            device_type: DeviceType::Nvme,
            bus_type: Some("nvme".to_string()),
            smartctl_path: None,
            capacity_bytes: Some(1_000_000_000),
            alias: alias.map(str::to_string),
            monitoring_enabled: true,
            first_seen_at: "2026-09-01T00:00:00Z".to_string(),
            last_seen_at: "2026-09-04T00:00:00Z".to_string(),
            removed_at: None,
            capabilities_json: None,
        }
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
            desde: time::OffsetDateTime::parse("2026-09-04T00:00:00Z", &Rfc3339).unwrap(),
            hasta: time::OffsetDateTime::parse("2026-09-04T10:00:00Z", &Rfc3339).unwrap(),
        }
    }

    #[test]
    fn el_numero_de_serie_solo_aparece_si_se_pide() {
        let d = dispositivo("d1", None);
        let con_serie = generar_html(std::slice::from_ref(&d), &[], rango(), true);
        assert!(con_serie.contains("S1B2C3"));

        let sin_serie = generar_html(&[d], &[], rango(), false);
        assert!(!sin_serie.contains("S1B2C3"));
    }

    #[test]
    fn una_alerta_fuera_del_intervalo_no_aparece() {
        let d = dispositivo("d1", Some("Mi disco"));
        let vieja = alerta("d1", "2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z");
        let html = generar_html(&[d], &[vieja], rango(), false);
        assert!(html.contains("Sin alertas en el intervalo."));
        assert!(!html.contains("smart.wear_high"));
    }

    #[test]
    fn una_alerta_dentro_del_intervalo_aparece_con_su_clave_literal() {
        let d = dispositivo("d1", Some("Mi disco"));
        let activa = alerta("d1", "2026-09-04T01:00:00Z", "2026-09-04T05:00:00Z");
        let html = generar_html(&[d], &[activa], rango(), false);
        assert!(html.contains("smart.wear_high"));
        assert!(html.contains("warn"));
    }

    #[test]
    fn un_alias_con_caracteres_html_se_escapa() {
        let d = dispositivo("d1", Some("<script>alert(1)</script>"));
        let html = generar_html(&[d], &[], rango(), false);
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn el_documento_es_autonomo_sin_recursos_remotos() {
        let html = generar_html(&[], &[], rango(), false);
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
        assert!(html.contains("<style>"));
    }
}
