//! Construcción de series con huecos explícitos (T060/T061, `docs/open-questions.md` E.1,
//! `TimeSeriesChart.svelte`).
//!
//! **No decide qué resolución servir** — eso es `commands::get_metric_series` (T062), que elige
//! entre `metric_samples` (crudo) y `metric_aggregates` (5 min / horario) según el intervalo
//! pedido. Este módulo recibe muestras ya elegidas y ordenadas, y decide solo dónde hay un hueco
//! real: el eje de la gráfica es tiempo real, no el índice de la muestra
//! (`src/lib/components/TimeSeriesChart.svelte`), así que no hace falta —ni se intenta— rellenar
//! una rejilla uniforme entre huecos: solo marcar dónde empiezan.

use time::OffsetDateTime;

/// Un punto de la serie servida: `t` en milisegundos epoch UTC, `v` ausente = hueco.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Punto {
    pub t_epoch_ms: i64,
    pub v: Option<f64>,
}

/// Un salto mayor que este múltiplo de la cadencia esperada es un hueco real, nunca una
/// continuidad que la gráfica deba interpolar. **2,5× ≈ dos ciclos**: un ciclo de recopilación
/// puntualmente perdido (equipo que se suspende o va cargado) ya no parte la línea en puntos
/// sueltos —lo que el usuario veía como confeti—, pero una parada de minutos u horas sí queda como
/// hueco (`docs/open-questions.md` E.1). Debe coincidir con `FACTOR_HUECO` de
/// `src/lib/design/series.ts`. `domain::retencion` y `domain::salud` usan 1,5× para lo suyo (cubos,
/// frescura), que es otra decisión.
const MULTIPLO_HUECO: f64 = 2.5;

fn parse_utc(s: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
}

fn epoch_ms(t: OffsetDateTime) -> i64 {
    (t.unix_timestamp_nanos() / 1_000_000) as i64
}

/// Completa una serie con huecos explícitos en los extremos y entre muestras consecutivas
/// (`docs/open-questions.md` E.1: "todo salto mayor que 2,5× la cadencia es hueco, incluidos los
/// extremos"). `muestras` debe venir ordenada por tiempo ascendente y con timestamps parseables;
/// una que no lo sea se descarta en vez de hacer fallar la serie entera.
pub fn completar_serie(
    muestras: &[(String, f64)],
    desde: OffsetDateTime,
    hasta: OffsetDateTime,
    cadencia_esperada_ms: i64,
) -> Vec<Punto> {
    let validas: Vec<(OffsetDateTime, f64)> = muestras
        .iter()
        .filter_map(|(t, v)| parse_utc(t).map(|t| (t, *v)))
        .collect();

    let umbral_ms = (cadencia_esperada_ms as f64 * MULTIPLO_HUECO) as i64;
    let salto_ms = |a: OffsetDateTime, b: OffsetDateTime| (b - a).whole_milliseconds() as i64;

    let Some((primera, _)) = validas.first().copied() else {
        // Sin ninguna muestra, el intervalo entero es un hueco: dos puntos ausentes en los
        // extremos, no una lista vacía que la gráfica no sabría distinguir de "aún sin pedir".
        return vec![
            Punto {
                t_epoch_ms: epoch_ms(desde),
                v: None,
            },
            Punto {
                t_epoch_ms: epoch_ms(hasta),
                v: None,
            },
        ];
    };

    let mut puntos = Vec::with_capacity(validas.len() + 2);

    if salto_ms(desde, primera) > umbral_ms {
        puntos.push(Punto {
            t_epoch_ms: epoch_ms(desde),
            v: None,
        });
    }

    for ventana in validas.windows(2) {
        let (t_a, v_a) = ventana[0];
        let (t_b, _) = ventana[1];
        puntos.push(Punto {
            t_epoch_ms: epoch_ms(t_a),
            v: Some(v_a),
        });
        if salto_ms(t_a, t_b) > umbral_ms {
            // El hueco se marca justo tras la muestra anterior: sugerir que el valor siguió
            // vigente hasta la próxima lectura real sería inventar continuidad.
            puntos.push(Punto {
                t_epoch_ms: epoch_ms(t_a) + cadencia_esperada_ms,
                v: None,
            });
        }
    }

    let (ultima_t, ultima_v) = *validas.last().unwrap();
    puntos.push(Punto {
        t_epoch_ms: epoch_ms(ultima_t),
        v: Some(ultima_v),
    });
    if salto_ms(ultima_t, hasta) > umbral_ms {
        puntos.push(Punto {
            t_epoch_ms: epoch_ms(hasta),
            v: None,
        });
    }

    puntos
}

/// Submuestrea a lo sumo `limite` cubos si la serie lo supera, conservando mínimo y máximo de
/// cada uno (`docs/open-questions.md` E.1: "el backend submuestrea conservando mínimo y máximo").
/// Devuelve la serie (intacta si ya cabía) y si se submuestreó, para que la respuesta lo declare.
pub fn submuestrear(puntos: Vec<Punto>, limite: usize) -> (Vec<Punto>, bool) {
    if puntos.len() <= limite || limite == 0 {
        return (puntos, false);
    }
    // Un cubo por cada dos puntos de salida (mínimo y máximo): así el límite declarado sigue
    // siendo un techo real sobre lo que la gráfica recibe, no el doble por sorpresa.
    let cubos_totales = (limite / 2).max(1);
    let tam_cubo = puntos.len().div_ceil(cubos_totales);

    let mut resultado = Vec::with_capacity(cubos_totales * 2);
    for cubo in puntos.chunks(tam_cubo) {
        let con_valor: Vec<f64> = cubo.iter().filter_map(|p| p.v).collect();
        if con_valor.is_empty() {
            // Un cubo sin ningún dato real sigue siendo un hueco, no se inventa un mínimo/máximo.
            resultado.push(Punto {
                t_epoch_ms: cubo[0].t_epoch_ms,
                v: None,
            });
            continue;
        }
        let min = con_valor.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = con_valor.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let t_min = cubo.iter().find(|p| p.v == Some(min)).unwrap().t_epoch_ms;
        let t_max = cubo.iter().find(|p| p.v == Some(max)).unwrap().t_epoch_ms;
        // Cronológico dentro del cubo: si el máximo ocurrió antes que el mínimo, van en ese orden.
        if t_min <= t_max {
            resultado.push(Punto {
                t_epoch_ms: t_min,
                v: Some(min),
            });
            if max != min {
                resultado.push(Punto {
                    t_epoch_ms: t_max,
                    v: Some(max),
                });
            }
        } else {
            resultado.push(Punto {
                t_epoch_ms: t_max,
                v: Some(max),
            });
            resultado.push(Punto {
                t_epoch_ms: t_min,
                v: Some(min),
            });
        }
    }
    (resultado, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> OffsetDateTime {
        parse_utc(s).unwrap()
    }

    #[test]
    fn sin_ninguna_muestra_el_intervalo_entero_es_un_hueco() {
        let puntos = completar_serie(
            &[],
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T01:00:00Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 2);
        assert!(puntos.iter().all(|p| p.v.is_none()));
    }

    #[test]
    fn sin_huecos_devuelve_una_muestra_por_punto() {
        let muestras = vec![
            ("2026-09-04T00:00:00Z".to_string(), 40.0),
            ("2026-09-04T00:01:00Z".to_string(), 41.0),
            ("2026-09-04T00:02:00Z".to_string(), 42.0),
        ];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:02:00Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 3);
        assert!(puntos.iter().all(|p| p.v.is_some()));
    }

    #[test]
    fn un_hueco_al_principio_se_marca_en_el_extremo_pedido() {
        let muestras = vec![("2026-09-04T00:10:00Z".to_string(), 40.0)];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:10:00Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 2);
        assert_eq!(puntos[0].t_epoch_ms, epoch_ms(t("2026-09-04T00:00:00Z")));
        assert_eq!(puntos[0].v, None);
        assert_eq!(puntos[1].v, Some(40.0));
    }

    #[test]
    fn un_hueco_al_final_se_marca_en_el_extremo_pedido() {
        let muestras = vec![("2026-09-04T00:00:00Z".to_string(), 40.0)];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:10:00Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 2);
        assert_eq!(puntos[0].v, Some(40.0));
        assert_eq!(puntos[1].t_epoch_ms, epoch_ms(t("2026-09-04T00:10:00Z")));
        assert_eq!(puntos[1].v, None);
    }

    #[test]
    fn un_hueco_entre_dos_muestras_reales_se_marca_entre_ellas() {
        let muestras = vec![
            ("2026-09-04T00:00:00Z".to_string(), 40.0),
            ("2026-09-04T00:10:00Z".to_string(), 45.0),
        ];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:10:00Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 3, "muestra, hueco, muestra");
        assert_eq!(puntos[0].v, Some(40.0));
        assert_eq!(puntos[1].v, None);
        assert_eq!(puntos[2].v, Some(45.0));
    }

    #[test]
    fn un_salto_justo_por_debajo_del_umbral_no_es_hueco() {
        // Cadencia de 60s, umbral 2,5× = 150s. Un salto de 149s no es hueco.
        let muestras = vec![
            ("2026-09-04T00:00:00Z".to_string(), 40.0),
            ("2026-09-04T00:02:29Z".to_string(), 41.0),
        ];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:02:29Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 2, "sin hueco intermedio");
    }

    #[test]
    fn un_salto_justo_por_encima_del_umbral_si_es_hueco() {
        // Cadencia de 60s, umbral 2,5× = 150s. Un salto de 151s sí es hueco.
        let muestras = vec![
            ("2026-09-04T00:00:00Z".to_string(), 40.0),
            ("2026-09-04T00:02:31Z".to_string(), 41.0),
        ];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:02:31Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 3, "hueco intermedio marcado");
    }

    #[test]
    fn una_muestra_con_fecha_ilegible_se_descarta_sin_romper_la_serie() {
        let muestras = vec![
            ("no-es-una-fecha".to_string(), 99.0),
            ("2026-09-04T00:00:00Z".to_string(), 40.0),
        ];
        let puntos = completar_serie(
            &muestras,
            t("2026-09-04T00:00:00Z"),
            t("2026-09-04T00:00:00Z"),
            60_000,
        );
        assert_eq!(puntos.len(), 1);
        assert_eq!(puntos[0].v, Some(40.0));
    }

    // ---- submuestrear ----

    #[test]
    fn una_serie_dentro_del_limite_no_se_toca() {
        let puntos = vec![
            Punto {
                t_epoch_ms: 0,
                v: Some(1.0),
            },
            Punto {
                t_epoch_ms: 1,
                v: Some(2.0),
            },
        ];
        let (resultado, submuestreada) = submuestrear(puntos.clone(), 10);
        assert!(!submuestreada);
        assert_eq!(resultado, puntos);
    }

    #[test]
    fn una_serie_por_encima_del_limite_conserva_minimo_y_maximo_por_cubo() {
        let puntos: Vec<Punto> = (0..100)
            .map(|i| Punto {
                t_epoch_ms: i,
                v: Some(i as f64),
            })
            .collect();
        let (resultado, submuestreada) = submuestrear(puntos, 10);
        assert!(submuestreada);
        assert!(resultado.len() <= 10);
        // El primer cubo empieza en 0: su mínimo es 0.
        assert_eq!(resultado[0].v, Some(0.0));
    }

    #[test]
    fn un_cubo_sin_ningun_dato_real_sigue_siendo_un_hueco() {
        let puntos = vec![
            Punto {
                t_epoch_ms: 0,
                v: None,
            },
            Punto {
                t_epoch_ms: 1,
                v: None,
            },
        ];
        let (resultado, _) = submuestrear(puntos, 1);
        assert!(resultado.iter().all(|p| p.v.is_none()));
    }
}
