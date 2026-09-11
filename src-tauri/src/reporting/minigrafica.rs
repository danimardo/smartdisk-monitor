//! Mini-gráfica SVG de una serie para el informe HTML (spec `009-informe-mejorado`).
//!
//! **Embebida y autónoma**: el SVG va dentro del HTML, sin recursos remotos, y debe abrirse sin
//! conexión. Reutiliza `domain::series::completar_serie` para marcar los huecos (un hueco es un
//! hueco: se rompe el trazo, nunca se interpola ni se dibuja a cero, `open-questions.md` E.1) y
//! `submuestrear` para acotar el número de puntos. Print-safe: trazo gris oscuro fijo, sin
//! dependencia del color, legible en escala de grises; tamaño fijo para que la impresión no lo
//! deforme.

use time::OffsetDateTime;

use crate::domain::series::{completar_serie, submuestrear, Punto};

use super::export::{serie_device, RangoExport, ResolucionInforme};

const ANCHO: f64 = 480.0;
const ALTO: f64 = 120.0;
const GUTTER: f64 = 38.0; // eje Y, fuera del trazo
const PAD_SUP: f64 = 8.0;
const PAD_INF: f64 = 8.0;
const TOPE_PUNTOS: usize = 600;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn fmt_num(v: f64) -> String {
    // Sin decimales si es redondo; si no, uno.
    if (v - v.round()).abs() < 0.05 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

/// SVG de la serie `metric_key` de `device_id` en el rango del informe. `unidad` se muestra en el
/// pie junto a la resolución. Sin ninguna muestra con valor en el intervalo, `dibujar` ya produce
/// el estado vacío «Sin muestras en el periodo» — este comando nunca falla por falta de datos.
pub fn minigrafica_device(
    conn: &rusqlite::Connection,
    device_id: &str,
    metric_key: &str,
    unidad: &str,
    rango: RangoExport,
) -> rusqlite::Result<String> {
    let (serie, resolucion) = serie_device(conn, device_id, metric_key, rango)?;
    Ok(dibujar(
        &serie,
        resolucion,
        resolucion.cadencia_ms(metric_key),
        rango.desde,
        rango.hasta,
        unidad,
    ))
}

/// Puro: dado el conjunto de muestras y el intervalo, produce el SVG. Separado para poder probarlo
/// sin base de datos.
pub fn dibujar(
    muestras: &[(String, f64)],
    resolucion: ResolucionInforme,
    cadencia_ms: i64,
    desde: OffsetDateTime,
    hasta: OffsetDateTime,
    unidad: &str,
) -> String {
    let puntos = completar_serie(muestras, desde, hasta, cadencia_ms);
    let (puntos, _) = submuestrear(puntos, TOPE_PUNTOS);

    let valores: Vec<f64> = puntos.iter().filter_map(|p| p.v).collect();
    if valores.is_empty() {
        return svg_vacio(unidad);
    }
    let mut vmin = valores.iter().copied().fold(f64::INFINITY, f64::min);
    let mut vmax = valores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (vmax - vmin).abs() < 1e-9 {
        vmin -= 1.0;
        vmax += 1.0;
    }
    let aire = (vmax - vmin) * 0.08;
    vmin -= aire;
    vmax += aire;

    let t0 = puntos.first().map(|p| p.t_epoch_ms).unwrap_or(0);
    let t1 = puntos.last().map(|p| p.t_epoch_ms).unwrap_or(t0 + 1);
    let span_t = (t1 - t0).max(1) as f64;
    let alto_trazo = ALTO - PAD_SUP - PAD_INF;

    let x = |t: i64| GUTTER + ((t - t0) as f64 / span_t) * (ANCHO - GUTTER - 2.0);
    let y = |v: f64| PAD_SUP + (1.0 - (v - vmin) / (vmax - vmin)) * alto_trazo;

    // Marcas del eje Y: mínimo, medio, máximo.
    let mut ejes = String::new();
    for frac in [0.0_f64, 0.5, 1.0] {
        let v = vmin + (vmax - vmin) * frac;
        let yy = y(v);
        ejes.push_str(&format!(
            "<line x1=\"{GUTTER:.0}\" y1=\"{yy:.1}\" x2=\"{:.0}\" y2=\"{yy:.1}\" stroke=\"#e5e5e5\" stroke-width=\"1\"/>\
             <text x=\"{:.0}\" y=\"{:.1}\" text-anchor=\"end\" font-size=\"9\" fill=\"#777\">{}</text>",
            ANCHO,
            GUTTER - 4.0,
            (yy + 3.0).clamp(9.0, ALTO - 2.0),
            esc(&fmt_num(v)),
        ));
    }

    // Trazo: una polilínea por tramo continuo (se rompe en cada `Punto` sin valor).
    let mut tramos: Vec<String> = Vec::new();
    let mut actual: Vec<(f64, f64)> = Vec::new();
    for Punto { t_epoch_ms, v } in &puntos {
        match v {
            Some(val) => actual.push((x(*t_epoch_ms), y(*val))),
            None => {
                if actual.len() > 1 {
                    tramos.push(polilinea(&actual));
                }
                actual.clear();
            }
        }
    }
    if actual.len() > 1 {
        tramos.push(polilinea(&actual));
    } else if actual.len() == 1 {
        tramos.push(format!(
            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"2\" fill=\"#333\"/>",
            actual[0].0, actual[0].1
        ));
    }

    let pie = format!("{} · {}", esc(unidad), resolucion_texto(resolucion));

    // Sin `xmlns`: el SVG va **en línea** en el HTML5, el parser ya lo sabe; así el documento no
    // contiene ninguna URL (ni siquiera la del namespace) y la comprobación de autonomía es limpia.
    format!(
        "<svg viewBox=\"0 0 {ANCHO:.0} {alto_total:.0}\" width=\"{ANCHO:.0}\" height=\"{alto_total:.0}\" role=\"img\" aria-label=\"{aria}\">\
         {ejes}{trazo}\
         <text x=\"{GUTTER:.0}\" y=\"{pie_y:.0}\" font-size=\"9\" fill=\"#777\">{pie}</text>\
         </svg>",
        alto_total = ALTO + 14.0,
        pie_y = ALTO + 11.0,
        aria = esc(&format!(
            "Serie: mínimo {}, media {}, máximo {} {}",
            fmt_num(vmin + aire),
            fmt_num((vmin + vmax) / 2.0),
            fmt_num(vmax - aire),
            unidad
        )),
        trazo = tramos.join(""),
    )
}

fn polilinea(puntos: &[(f64, f64)]) -> String {
    let d: String = puntos
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("<polyline points=\"{d}\" fill=\"none\" stroke=\"#333\" stroke-width=\"1.6\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>")
}

fn svg_vacio(unidad: &str) -> String {
    format!(
        "<svg viewBox=\"0 0 {ANCHO:.0} {ALTO:.0}\" width=\"{ANCHO:.0}\" height=\"{ALTO:.0}\" role=\"img\" aria-label=\"Sin muestras en el periodo\">\
         <text x=\"{cx:.0}\" y=\"{cy:.0}\" text-anchor=\"middle\" font-size=\"11\" fill=\"#999\">Sin muestras en el periodo{u}</text>\
         </svg>",
        cx = ANCHO / 2.0,
        cy = ALTO / 2.0,
        u = if unidad.is_empty() { String::new() } else { format!(" ({})", esc(unidad)) },
    )
}

fn resolucion_texto(r: ResolucionInforme) -> &'static str {
    match r {
        ResolucionInforme::Raw => "muestras cada 30 s",
        ResolucionInforme::FiveMinutes => "promedios de 5 min",
        ResolucionInforme::Hourly => "promedios horarios",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    fn t(base: OffsetDateTime, min: i64) -> String {
        (base + Duration::minutes(min))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap()
    }

    fn sin_recursos_remotos(svg: &str) {
        assert!(!svg.contains("http://"), "{svg}");
        assert!(!svg.contains("https://"), "{svg}");
        assert!(!svg.contains("<script"), "{svg}");
        assert!(!svg.contains("<image"), "{svg}");
        assert!(!svg.contains("src="), "{svg}");
    }

    #[test]
    fn sin_datos_devuelve_el_svg_vacio_y_no_lleva_recursos_remotos() {
        let base = OffsetDateTime::now_utc();
        let svg = dibujar(
            &[],
            ResolucionInforme::Raw,
            30_000,
            base,
            base + Duration::hours(1),
            "°C",
        );
        assert!(svg.contains("Sin muestras en el periodo"));
        sin_recursos_remotos(&svg);
    }

    #[test]
    fn una_serie_con_un_hueco_produce_dos_polilineas() {
        let base = OffsetDateTime::now_utc() - Duration::hours(2);
        // Muestras cada minuto 0..20, hueco de 30 min, luego 50..60.
        let mut m: Vec<(String, f64)> = (0..20).map(|i| (t(base, i), 40.0 + i as f64)).collect();
        m.extend((50..60).map(|i| (t(base, i), 45.0)));
        let svg = dibujar(
            &m,
            ResolucionInforme::Raw,
            60_000,
            base,
            base + Duration::minutes(61),
            "°C",
        );
        assert_eq!(
            svg.matches("<polyline").count(),
            2,
            "el hueco parte el trazo"
        );
        assert!(svg.contains("promedios") || svg.contains("muestras cada 30 s"));
    }

    #[test]
    fn el_svg_es_autonomo() {
        let base = OffsetDateTime::now_utc() - Duration::hours(1);
        let m: Vec<(String, f64)> = (0..30)
            .map(|i| (t(base, i * 2), 10.0 + (i % 5) as f64))
            .collect();
        let svg = dibujar(
            &m,
            ResolucionInforme::Raw,
            120_000,
            base,
            base + Duration::hours(1),
            "%",
        );
        sin_recursos_remotos(&svg);
    }
}
