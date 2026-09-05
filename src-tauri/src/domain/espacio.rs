//! Guardia de espacio libre para el volumen del historial (FR-020a/b/c, `open-questions.md` J.13).
//!
//! Al cruzar el umbral de aviso se notifica; al cruzar el de parada se detiene la escritura de
//! historial, sin afectar a la monitorización ni a las alertas en vivo. **Nunca purga nada por
//! iniciativa propia**: los datos solo desaparecen por retención configurada o borrado explícito.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoEspacio {
    /// Por encima del umbral de aviso: nada que hacer.
    Normal,
    /// Por debajo del umbral de aviso, pero aún se escribe historial.
    Aviso,
    /// Por debajo del umbral de parada: la escritura de historial se detiene.
    Parada,
}

/// Umbrales en bytes. Los valores por defecto (`open-questions.md` J.13) los fija quien construye
/// esto a partir de `settings`; este módulo no conoce la persistencia.
#[derive(Debug, Clone, Copy)]
pub struct UmbralesEspacio {
    pub aviso_bytes: u64,
    pub parada_bytes: u64,
}

impl UmbralesEspacio {
    pub fn evaluar(&self, espacio_libre_bytes: u64) -> EstadoEspacio {
        if espacio_libre_bytes < self.parada_bytes {
            EstadoEspacio::Parada
        } else if espacio_libre_bytes < self.aviso_bytes {
            EstadoEspacio::Aviso
        } else {
            EstadoEspacio::Normal
        }
    }
}

/// `true` si, dado el estado de espacio actual, el ciclo de recopilación debe escribir historial.
/// La vigilancia y las alertas en vivo **no** consultan esta función: siguen funcionando siempre
/// (FR-020a). Solo la escritura de `metric_samples`/`smart_snapshots`/`system_events` la consulta.
pub fn debe_escribir_historial(estado: EstadoEspacio) -> bool {
    !matches!(estado, EstadoEspacio::Parada)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn umbrales() -> UmbralesEspacio {
        // Los valores por defecto de J.13: 1 GB aviso, 256 MB parada.
        UmbralesEspacio {
            aviso_bytes: 1_073_741_824,
            parada_bytes: 268_435_456,
        }
    }

    #[test]
    fn por_encima_del_aviso_es_normal() {
        assert_eq!(umbrales().evaluar(2_000_000_000), EstadoEspacio::Normal);
    }

    #[test]
    fn cruzar_el_aviso_pero_no_la_parada() {
        let estado = umbrales().evaluar(500_000_000);
        assert_eq!(estado, EstadoEspacio::Aviso);
        assert!(
            debe_escribir_historial(estado),
            "en aviso se sigue escribiendo historial"
        );
    }

    #[test]
    fn cruzar_la_parada_detiene_la_escritura() {
        let estado = umbrales().evaluar(100_000_000);
        assert_eq!(estado, EstadoEspacio::Parada);
        assert!(!debe_escribir_historial(estado));
    }

    #[test]
    fn en_parada_la_vigilancia_no_se_consulta_aqui() {
        // Este módulo no decide si la monitorización sigue: solo si se escribe historial.
        // FR-020a exige que la vigilancia y las alertas en vivo sigan funcionando siempre; eso
        // vive en domain::estado, que no llama a debe_escribir_historial en absoluto.
        let estado = umbrales().evaluar(0);
        assert_eq!(estado, EstadoEspacio::Parada);
    }

    #[test]
    fn el_limite_exacto_del_umbral_cuenta_como_por_debajo() {
        let u = umbrales();
        assert_eq!(
            u.evaluar(u.aviso_bytes),
            EstadoEspacio::Normal,
            "igual al umbral no es aviso"
        );
        assert_eq!(u.evaluar(u.aviso_bytes - 1), EstadoEspacio::Aviso);
        assert_eq!(
            u.evaluar(u.parada_bytes),
            EstadoEspacio::Aviso,
            "igual al umbral no es parada"
        );
        assert_eq!(u.evaluar(u.parada_bytes - 1), EstadoEspacio::Parada);
    }
}
