//! Parada automática del benchmark por límite térmico (T082, `docs/product-specification.md`
//! §6: "se detiene si se alcanza el límite térmico crítico —el del fabricante si lo declara, y
//! si no el configurado—").
//!
//! Puro: no lee la temperatura ni decide cuándo comprobarla — eso es `tests::diskspd`, que
//! sondea entre mediciones y durante cada invocación de DiskSpd con el último valor ya leído.

/// El límite crítico efectivo: el del fabricante si el disco lo declara, si no el configurado en
/// `settings` (mismo criterio que `alert-rules.md` usa para las reglas de temperatura).
pub fn limite_critico_efectivo(vendor_critical_c: Option<f64>, configurado_c: f64) -> f64 {
    vendor_critical_c.unwrap_or(configurado_c)
}

/// Si el benchmark debe detenerse ya. Una temperatura ausente **nunca** detiene la prueba: no
/// saber la temperatura no es lo mismo que saber que está alta, y una guardia que se dispara sin
/// dato sería peor que no tener guardia (mismo principio que `domain::salud`).
pub fn debe_detenerse_por_temperatura(temperatura_actual_c: Option<f64>, limite_c: f64) -> bool {
    match temperatura_actual_c {
        Some(t) => t >= limite_c,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_limite_del_fabricante_manda_si_existe() {
        assert_eq!(limite_critico_efectivo(Some(85.0), 80.0), 85.0);
    }

    #[test]
    fn sin_limite_del_fabricante_manda_el_configurado() {
        assert_eq!(limite_critico_efectivo(None, 80.0), 80.0);
    }

    #[test]
    fn por_debajo_del_limite_no_se_detiene() {
        assert!(!debe_detenerse_por_temperatura(Some(79.9), 80.0));
    }

    #[test]
    fn justo_en_el_limite_se_detiene() {
        assert!(debe_detenerse_por_temperatura(Some(80.0), 80.0));
    }

    #[test]
    fn por_encima_del_limite_se_detiene() {
        assert!(debe_detenerse_por_temperatura(Some(95.0), 80.0));
    }

    #[test]
    fn sin_lectura_de_temperatura_nunca_se_detiene() {
        assert!(!debe_detenerse_por_temperatura(None, 80.0));
    }
}
