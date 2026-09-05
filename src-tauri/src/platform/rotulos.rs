//! Traducción mínima para las superficies nativas que Svelte no puede alcanzar: el menú de la
//! bandeja del sistema y las notificaciones viven fuera del webview, así que no pueden pasar por
//! `t()`/`tp()` de `$lib/i18n`.
//!
//! No hay una segunda copia del vocabulario: este módulo **lee las mismas claves** de
//! `src/lib/i18n/{es,en}.json`, empotradas en el binario con `include_str!`. Solo el lector existe
//! dos veces (una por lenguaje), nunca el texto.

use std::collections::HashMap;
use std::sync::OnceLock;

type Dict = HashMap<String, String>;

static ES: OnceLock<Dict> = OnceLock::new();
static EN: OnceLock<Dict> = OnceLock::new();

fn cargar(json: &str) -> Dict {
    serde_json::from_str(json).expect("los diccionarios de i18n del repositorio son JSON válido")
}

fn es_dict() -> &'static Dict {
    ES.get_or_init(|| cargar(include_str!("../../../src/lib/i18n/es.json")))
}

fn en_dict() -> &'static Dict {
    EN.get_or_init(|| cargar(include_str!("../../../src/lib/i18n/en.json")))
}

fn dict(locale: &str) -> &'static Dict {
    if locale == "es" {
        es_dict()
    } else {
        en_dict()
    }
}

/// El idioma de la interfaz nativa: sigue al idioma del sistema, igual que `i18n.svelte.ts` hace
/// al arrancar sin preferencia guardada (`get_appearance_settings` todavía no persiste una
/// elección explícita — `docs/open-questions.md` J.12).
pub fn locale_actual() -> &'static str {
    if crate::platform::locale::system_locale()
        .to_lowercase()
        .starts_with("es")
    {
        "es"
    } else {
        "en"
    }
}

/// Traduce una clave. Una clave ausente en el idioma activo cae al inglés y, si tampoco está ahí,
/// devuelve la propia clave — mismo respaldo que `t()` en `i18n.svelte.ts`.
pub fn t(locale: &str, key: &str) -> String {
    dict(locale)
        .get(key)
        .or_else(|| en_dict().get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Plural por cantidad: `<key>.one` si `count == 1`, si no `<key>.other` — la regla cardinal real
/// de `Intl.PluralRules` para español e inglés es exactamente esta, así que reproducirla no exige
/// una dependencia de reglas ICU. Interpola `{count}`.
pub fn tp(locale: &str, key: &str, count: i64) -> String {
    let sufijo = if count == 1 { "one" } else { "other" };
    let plantilla = t(locale, &format!("{key}.{sufijo}"));
    plantilla.replace("{count}", &count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traduce_una_clave_conocida_en_espanol() {
        assert_eq!(t("es", "global.allGood"), "Todo en orden");
    }

    #[test]
    fn traduce_una_clave_conocida_en_ingles() {
        assert_eq!(t("en", "global.allGood"), "All good");
    }

    #[test]
    fn una_clave_ausente_devuelve_la_propia_clave() {
        assert_eq!(t("es", "no.existe.esta.clave"), "no.existe.esta.clave");
    }

    #[test]
    fn plural_uno_usa_el_singular() {
        assert_eq!(
            tp("es", "global.needsAttention", 1),
            "1 disco necesita atención"
        );
    }

    #[test]
    fn plural_distinto_de_uno_usa_other() {
        let texto = tp("es", "global.needsAttention", 3);
        assert!(texto.contains('3'), "no interpoló el conteo: {texto:?}");
    }
}
