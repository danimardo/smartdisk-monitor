//! Resumen numérico (mínimo, media, máximo, pico) de una serie métrica en el intervalo del
//! informe (spec `009-informe-mejorado`). Se usa en dos sitios: la cifra que se pinta en la
//! sección del disco, y el dato que viaja al modelo en el resumen con IA.
//!
//! El **pico** es el máximo; se conserva el nombre porque es el que la aplicación ya usa para la
//! actividad (`ActividadDisco.pico_percent`). Sin ninguna muestra en el intervalo, todo es `None`
//! («sin datos»), nunca `0` (constitución §I).

use rusqlite::Connection;

use super::export::{serie_device, RangoExport, ResolucionInforme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResumenMetrico {
    pub minimo: Option<f64>,
    pub media: Option<f64>,
    pub maximo: Option<f64>,
    pub pico: Option<f64>,
    pub muestras: u32,
    pub resolucion: ResolucionInforme,
}

impl ResumenMetrico {
    pub fn sin_datos(resolucion: ResolucionInforme) -> Self {
        Self {
            minimo: None,
            media: None,
            maximo: None,
            pico: None,
            muestras: 0,
            resolucion,
        }
    }

    pub fn hay_datos(&self) -> bool {
        self.muestras > 0
    }

    /// Media/mín/máx de valores ya leídos. `pico = maximo`.
    pub fn de_valores(valores: &[f64], resolucion: ResolucionInforme) -> Self {
        if valores.is_empty() {
            return Self::sin_datos(resolucion);
        }
        let n = valores.len() as f64;
        let suma: f64 = valores.iter().sum();
        let minimo = valores.iter().copied().fold(f64::INFINITY, f64::min);
        let maximo = valores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        Self {
            minimo: Some(minimo),
            media: Some(suma / n),
            maximo: Some(maximo),
            pico: Some(maximo),
            muestras: valores.len() as u32,
            resolucion,
        }
    }
}

/// Resumen de `metric_key` para `device_id` en el rango del informe.
pub fn resumen_dispositivo(
    conn: &Connection,
    device_id: &str,
    metric_key: &str,
    rango: RangoExport,
) -> rusqlite::Result<ResumenMetrico> {
    let (serie, resolucion) = serie_device(conn, device_id, metric_key, rango)?;
    let valores: Vec<f64> = serie.into_iter().map(|(_, v)| v).collect();
    Ok(ResumenMetrico::de_valores(&valores, resolucion))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_muestras_es_todo_none() {
        let r = ResumenMetrico::de_valores(&[], ResolucionInforme::Raw);
        assert_eq!(r.minimo, None);
        assert_eq!(r.media, None);
        assert_eq!(r.maximo, None);
        assert_eq!(r.pico, None);
        assert_eq!(r.muestras, 0);
        assert!(!r.hay_datos());
    }

    #[test]
    fn media_minimo_maximo_y_pico_sobre_valores_conocidos() {
        let r = ResumenMetrico::de_valores(&[10.0, 40.0, 70.0], ResolucionInforme::FiveMinutes);
        assert_eq!(r.minimo, Some(10.0));
        assert!((r.media.unwrap() - 40.0).abs() < 1e-9);
        assert_eq!(r.maximo, Some(70.0));
        assert_eq!(r.pico, Some(70.0));
        assert_eq!(r.muestras, 3);
        assert_eq!(r.resolucion, ResolucionInforme::FiveMinutes);
    }

    #[test]
    fn un_solo_valor_es_min_media_y_max() {
        let r = ResumenMetrico::de_valores(&[42.5], ResolucionInforme::Hourly);
        assert_eq!(r.minimo, Some(42.5));
        assert_eq!(r.media, Some(42.5));
        assert_eq!(r.pico, Some(42.5));
    }
}
