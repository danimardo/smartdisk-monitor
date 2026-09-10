//! Ventana deslizante de actividad de disco (spec `007-actividad-disco-representativa`,
//! `docs/open-questions.md` D.4, ADR-050).
//!
//! **Sin PDH, sin Tauri, sin SQLite** (constitución §IV): recibe muestras ya derivadas
//! (`activity_percent`, 0–100) fechadas con un reloj **monotónico** (`std::time::Instant`) y
//! devuelve el agregado —media, pico y estado del dato— del último tramo del tamaño de la cadencia
//! de «métricas rápidas». El muestreo real y la consulta PDH persistente viven en
//! `collectors::perf_counters` y en el bucle de `commands::iniciar_planificador`.
//!
//! La agregación es un área de fallo silencioso (constitución §VIII): un error aquí no se ve como
//! un fallo, sino como un número equivocado que alguien se cree. Por eso las pruebas van primero y
//! cubren media, pico, ventana parcial, corte por hueco, purga por el frente y que una muestra
//! fallida —que el colector simplemente no registra— nunca introduce un `0`.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// La ventana cuenta como «completa» cuando cubre al menos esta fracción de su tamaño. Con
/// muestreo de 1 s sobre una ventana de 30 s, equivale a tener muestras desde hace ≥ 27 s; el 10 %
/// de margen absorbe el jitter del planificador y el descarte del primer muestreo PDH tras
/// reconstruir la consulta (`research.md` R1).
const FRACCION_COBERTURA_VALIDA: f64 = 0.9;

/// Estado del dato de actividad: decide si el número que se muestra es fiable. Nunca se presenta
/// un `parcial` como si fuera `válido` ni un hueco como un `0` (constitución §I).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub enum EstadoActividad {
    /// La ventana cubre al menos la cadencia: media y pico son representativos del periodo.
    Valido,
    /// Hay muestras pero aún no cubren la cadencia (arranque, reanudación tras pausa, tras un hueco).
    Parcial,
    /// La ventana está vacía: ni una muestra reciente (fuente degradada, o aún sin arrancar).
    NoDisponible,
}

/// Agregado de la ventana, listo para cruzar a la interfaz. Sustituye al antiguo
/// `activity_percent: Option<f64>` de `DiskSummary` (spec 007, FR-012a). No lleva marca de tiempo
/// ni procedencia: el agregado se recalcula en cada emisión desde la ventana viva, así que es
/// actual por construcción, y la procedencia es siempre «contadores de rendimiento»
/// (`research.md` R9). `media_percent`/`pico_percent` son `None` solo cuando
/// `estado == NoDisponible`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/api/generated/")]
pub struct ActividadDisco {
    pub estado: EstadoActividad,
    pub media_percent: Option<f64>,
    pub pico_percent: Option<f64>,
    /// Cuántas muestras respaldan la ventana en este instante.
    pub muestras: u32,
    /// Periodo que la ventana pretende cubrir, en segundos (= cadencia de «métricas rápidas»).
    pub ventana_segundos: u32,
}

impl ActividadDisco {
    /// El agregado de un disco cuya ventana todavía no existe (arranque) o cuya fuente está
    /// degradada. `ventana_segundos` se conserva para que la interfaz pueda decir «media de los
    /// últimos N s» en cuanto haya datos.
    pub fn no_disponible(ventana_segundos: u32) -> Self {
        Self {
            estado: EstadoActividad::NoDisponible,
            media_percent: None,
            pico_percent: None,
            muestras: 0,
            ventana_segundos,
        }
    }
}

/// Una lectura instantánea de `activity_percent`, fechada con el reloj monotónico.
#[derive(Debug, Clone, Copy)]
struct MuestraActividad {
    instante: Instant,
    valor: f64,
}

/// Ventana deslizante por disco. `ventana` es la cadencia de «métricas rápidas» vigente. Estado en
/// memoria del proceso: se pierde al reiniciar y arranca vacía.
#[derive(Debug, Clone)]
pub struct VentanaActividad {
    ventana: Duration,
    muestras: VecDeque<MuestraActividad>,
}

impl VentanaActividad {
    pub fn nueva(ventana: Duration) -> Self {
        Self {
            ventana,
            muestras: VecDeque::new(),
        }
    }

    /// El tamaño con el que se creó. Permite comparar contra la cadencia vigente y recrear la
    /// ventana si el usuario la cambió en Ajustes.
    pub fn ventana(&self) -> Duration {
        self.ventana
    }

    /// Registra una lectura **real**. Si desde la última muestra ha pasado más que `umbral_hueco`
    /// —3× el intervalo de muestreo vigente (`docs/open-questions.md` D.4); suspensión del equipo,
    /// subsistema de rendimiento bloqueado— se descarta lo anterior en vez de promediar a través
    /// del hueco (FR-007). El colector solo llama a esto con lecturas válidas: una muestra fallida
    /// no entra, y por tanto nunca se cuela un `0.0` de relleno (FR-005).
    pub fn registrar(&mut self, ahora: Instant, valor: f64, umbral_hueco: Duration) {
        if let Some(ultima) = self.muestras.back() {
            if ahora.saturating_duration_since(ultima.instante) > umbral_hueco {
                self.muestras.clear();
            }
        }
        self.muestras.push_back(MuestraActividad {
            instante: ahora,
            valor,
        });
        self.purgar(ahora);
    }

    /// Descarta por el frente las muestras más viejas que `ventana`. La llama `registrar`; también
    /// es pública para acotar la memoria de un disco que dejó de muestrearse sin recalcular nada.
    pub fn purgar(&mut self, ahora: Instant) {
        while let Some(primera) = self.muestras.front() {
            if ahora.saturating_duration_since(primera.instante) > self.ventana {
                self.muestras.pop_front();
            } else {
                break;
            }
        }
    }

    /// Media y pico del tramo vigente a `ahora`, con el estado del dato. No muta: filtra por
    /// antigüedad, de modo que un disco cuya fuente cayó pasa a `NoDisponible` solo con el paso del
    /// tiempo, sin necesidad de una llamada a `registrar`.
    pub fn agregado(&self, ahora: Instant) -> ActividadDisco {
        let ventana_segundos = self.ventana.as_secs() as u32;

        let mut vigentes = self
            .muestras
            .iter()
            .filter(|m| ahora.saturating_duration_since(m.instante) <= self.ventana)
            .peekable();

        let Some(primera) = vigentes.peek().copied() else {
            return ActividadDisco::no_disponible(ventana_segundos);
        };

        let mut n: u32 = 0;
        let mut suma = 0.0_f64;
        let mut pico = f64::MIN;
        for m in vigentes {
            n += 1;
            suma += m.valor;
            if m.valor > pico {
                pico = m.valor;
            }
        }

        let antiguedad_primera = ahora.saturating_duration_since(primera.instante);
        let estado = if antiguedad_primera.as_secs_f64()
            >= self.ventana.as_secs_f64() * FRACCION_COBERTURA_VALIDA
        {
            EstadoActividad::Valido
        } else {
            EstadoActividad::Parcial
        };

        ActividadDisco {
            estado,
            media_percent: Some(suma / f64::from(n)),
            pico_percent: Some(pico),
            muestras: n,
            ventana_segundos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Umbral de hueco de las pruebas: 3 s (el de `docs/open-questions.md` D.4 en red eléctrica).
    const HUECO: Duration = Duration::from_secs(3);

    fn ventana(seg: u64) -> VentanaActividad {
        VentanaActividad::nueva(Duration::from_secs(seg))
    }

    /// Azúcar: registrar con el umbral de hueco de las pruebas.
    fn reg(w: &mut VentanaActividad, t: Instant, v: f64) {
        w.registrar(t, v, HUECO);
    }

    #[test]
    fn ventana_vacia_es_no_disponible() {
        let w = ventana(30);
        let a = w.agregado(Instant::now());
        assert_eq!(a.estado, EstadoActividad::NoDisponible);
        assert_eq!(a.media_percent, None);
        assert_eq!(a.pico_percent, None);
        assert_eq!(a.muestras, 0);
        assert_eq!(a.ventana_segundos, 30);
    }

    #[test]
    fn media_aritmetica_y_pico_sobre_muestras_conocidas() {
        let mut w = ventana(30);
        let base = Instant::now();
        reg(&mut w, base, 10.0);
        reg(&mut w, base + Duration::from_secs(1), 40.0);
        reg(&mut w, base + Duration::from_secs(2), 70.0);
        let a = w.agregado(base + Duration::from_secs(2));
        assert_eq!(a.muestras, 3);
        assert!((a.media_percent.unwrap() - 40.0).abs() < 1e-9);
        assert!((a.pico_percent.unwrap() - 70.0).abs() < 1e-9);
    }

    #[test]
    fn ventana_que_aun_no_cubre_la_cadencia_es_parcial() {
        let mut w = ventana(30);
        let base = Instant::now();
        reg(&mut w, base, 50.0);
        reg(&mut w, base + Duration::from_secs(2), 60.0);
        let a = w.agregado(base + Duration::from_secs(2));
        assert_eq!(a.estado, EstadoActividad::Parcial);
        assert_eq!(a.muestras, 2);
        assert_eq!(a.media_percent, Some(55.0));
    }

    #[test]
    fn ventana_que_cubre_la_cadencia_es_valida() {
        let mut w = ventana(10);
        let base = Instant::now();
        for i in 0..=10 {
            reg(&mut w, base + Duration::from_secs(i), 30.0);
        }
        let a = w.agregado(base + Duration::from_secs(10));
        assert_eq!(a.estado, EstadoActividad::Valido);
        assert_eq!(a.pico_percent, Some(30.0));
    }

    #[test]
    fn un_hueco_mayor_que_el_umbral_vacia_la_ventana_antes_de_agregar() {
        let mut w = ventana(30);
        let base = Instant::now();
        reg(&mut w, base, 80.0);
        reg(&mut w, base + Duration::from_secs(1), 80.0);
        // Hueco de 10 s (> 3 s de umbral): suspensión o bloqueo del subsistema.
        reg(&mut w, base + Duration::from_secs(11), 5.0);
        let a = w.agregado(base + Duration::from_secs(11));
        assert_eq!(a.muestras, 1, "las dos muestras del 80 % se descartaron");
        assert_eq!(a.pico_percent, Some(5.0));
    }

    #[test]
    fn las_muestras_fuera_de_ventana_se_descartan_por_el_frente() {
        let mut w = ventana(10);
        let base = Instant::now();
        for i in 0..=6 {
            // Muestras cada 2 s (todas dentro del umbral de hueco).
            reg(
                &mut w,
                base + Duration::from_secs(i * 2),
                if i == 0 { 90.0 } else { 50.0 },
            );
        }
        // En base+12, la muestra de base+0 (antigüedad 12 s > 10) queda fuera.
        let a = w.agregado(base + Duration::from_secs(12));
        assert_eq!(a.muestras, 6);
        assert_eq!(a.pico_percent, Some(50.0), "el 90 % de base+0 ya no cuenta");
    }

    #[test]
    fn una_muestra_fallida_no_introduce_un_cero() {
        // El colector, ante un `Err` de PDH, simplemente no llama a `registrar`.
        let mut w = ventana(30);
        let base = Instant::now();
        reg(&mut w, base, 80.0);
        // ciclo fallido: sin registrar
        reg(&mut w, base + Duration::from_secs(2), 82.0);
        // ciclo fallido: sin registrar
        reg(&mut w, base + Duration::from_secs(4), 78.0);
        let a = w.agregado(base + Duration::from_secs(4));
        assert_eq!(a.muestras, 3);
        assert!(
            a.media_percent.unwrap() > 75.0,
            "ningún 0 arrastró la media"
        );
        assert_eq!(a.pico_percent, Some(82.0));
    }

    #[test]
    fn un_muestreo_que_se_interrumpe_cada_pocos_segundos_nunca_llega_a_valido() {
        // Regresión de ADR-056: mientras el muestreo vivía en el hilo del planificador, un ciclo de
        // recopilación (métricas rápidas con varios discos, o SMART) lo bloqueaba más que el umbral
        // de hueco. Cada bloqueo vaciaba la ventana, así que nunca acumulaba la cobertura que
        // exige `Valido` y no se persistía ni una fila. Con muestreo en hilo propio esto no ocurre;
        // la prueba deja constancia de por qué el diseño anterior no servía.
        let mut w = ventana(30);
        let base = Instant::now();
        let mut t = base;
        for _ronda in 0..6 {
            // ~25 s de muestreo limpio a 1 s…
            for _ in 0..25 {
                reg(&mut w, t, 20.0);
                t += Duration::from_secs(1);
                assert_ne!(
                    w.agregado(t).estado,
                    EstadoActividad::Valido,
                    "aún no cubre los 27 s"
                );
            }
            // …y un salto de 5 s (> 3 s de umbral) que simula el bloqueo del ciclo.
            t += Duration::from_secs(5);
            reg(&mut w, t, 20.0);
            t += Duration::from_secs(1);
            assert_eq!(
                w.agregado(t).muestras,
                1,
                "el salto vació la ventana: se vuelve a empezar de cero"
            );
        }
    }

    #[test]
    fn la_ventana_decae_a_no_disponible_sin_nuevas_muestras() {
        // Fuente degradada: dejan de llegar muestras y las que había envejecen fuera de la ventana.
        let mut w = ventana(30);
        let base = Instant::now();
        reg(&mut w, base, 50.0);
        let a = w.agregado(base + Duration::from_secs(120));
        assert_eq!(a.estado, EstadoActividad::NoDisponible);
        assert_eq!(a.muestras, 0);
    }
}
