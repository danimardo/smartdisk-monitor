//! Anonimización consistente de identificadores personales (T089/T090, FR-028, US-051).
//!
//! Todo lo que entra en el ZIP de diagnóstico pasa por aquí antes de escribirse, salvo que el
//! usuario pida expresamente incluir identificadores (`includeIdentifiers`). La sustitución es la
//! misma cadena real → el mismo remplazo en todo el paquete (US-051, "la sustitución es
//! consistente"): un número de serie que aparece en el JSON bruto de `smartctl` y también en el
//! registro de actividad se convierte en el mismo `<SERIE-1>` en los dos sitios.
//!
//! Las cadenas se sustituyen de más larga a más corta: si el nombre de usuario fuera, por
//! ejemplo, un prefijo del nombre de equipo, sustituir la más corta primero podría partir la más
//! larga a medias y dejarla irreconocible en vez de sustituida entera.

/// Categoría de dato redactado, para `DiagnosticPreview.redactedFields`
/// (`docs/ui-contract.md` §3.7): claves de i18n, nunca texto — el backend no manda frases
/// (mismo criterio que `AlertGroup.ruleKey`, ADR-030).
pub const CAMPO_NUMERO_SERIE: &str = "diagnostic.redacted.serialNumber";
pub const CAMPO_NOMBRE_EQUIPO: &str = "diagnostic.redacted.computerName";
pub const CAMPO_RUTAS_USUARIO: &str = "diagnostic.redacted.userPaths";
pub const CAMPO_ETIQUETA_VOLUMEN: &str = "diagnostic.redacted.volumeLabel";

pub struct Anonimizador {
    sustituciones: Vec<(String, String)>,
    campos_afectados: Vec<&'static str>,
}

impl Anonimizador {
    /// Sin sustituciones: `aplicar` devuelve el texto tal cual. Para cuando el usuario pide
    /// expresamente incluir identificadores (US-051).
    pub fn sin_anonimizar() -> Self {
        Self {
            sustituciones: Vec::new(),
            campos_afectados: Vec::new(),
        }
    }

    /// Nombre de equipo y usuario de esta máquina (`COMPUTERNAME`/`USERNAME`, siempre presentes
    /// en un proceso de Windows) más los números de serie que se le pasen. Una variable ausente o
    /// vacía no genera una sustitución vacía, que borraría texto real por accidente
    /// (`str::replace("", x)` sustituiría entre cada carácter).
    pub fn para_esta_maquina(numeros_de_serie: &[String]) -> Self {
        let mut constructor = Self::sin_anonimizar();
        if let Ok(equipo) = std::env::var("COMPUTERNAME") {
            constructor = constructor.con_equipo(&equipo);
        }
        if let Ok(usuario) = std::env::var("USERNAME") {
            constructor = constructor.con_ruta_usuario(&usuario);
        }
        for serie in numeros_de_serie {
            constructor = constructor.con_numero_de_serie(serie);
        }
        constructor
    }

    pub fn con_numero_de_serie(mut self, real: &str) -> Self {
        if !real.is_empty() {
            let indice = self
                .sustituciones
                .iter()
                .filter(|(_, remplazo)| remplazo.starts_with("<SERIE-"))
                .count()
                + 1;
            self.push(real, &format!("<SERIE-{indice}>"), CAMPO_NUMERO_SERIE);
        }
        self
    }

    pub fn con_equipo(mut self, real: &str) -> Self {
        if !real.is_empty() {
            self.push(real, "<EQUIPO>", CAMPO_NOMBRE_EQUIPO);
        }
        self
    }

    pub fn con_ruta_usuario(mut self, real: &str) -> Self {
        if !real.is_empty() {
            self.push(real, "<USUARIO>", CAMPO_RUTAS_USUARIO);
        }
        self
    }

    /// Etiqueta de volumen (p. ej. «Sistema», «Datos»). Está en la lista del principio XVI desde
    /// 1.8.0; el informe con resumen IA (spec 009) es el primer sitio que la necesita, porque
    /// envía la lista de volúmenes del disco. Marcador numerado, como el número de serie.
    pub fn con_etiqueta_volumen(mut self, real: &str) -> Self {
        if !real.is_empty() {
            let indice = self
                .sustituciones
                .iter()
                .filter(|(_, remplazo)| remplazo.starts_with("<VOLUMEN-"))
                .count()
                + 1;
            self.push(real, &format!("<VOLUMEN-{indice}>"), CAMPO_ETIQUETA_VOLUMEN);
        }
        self
    }

    fn push(&mut self, real: &str, remplazo: &str, campo: &'static str) {
        self.sustituciones
            .push((real.to_string(), remplazo.to_string()));
        self.sustituciones
            .sort_by_key(|(real, _)| std::cmp::Reverse(real.len()));
        if !self.campos_afectados.contains(&campo) {
            self.campos_afectados.push(campo);
        }
    }

    /// Aplica todas las sustituciones, de más larga a más corta.
    pub fn aplicar(&self, texto: &str) -> String {
        let mut resultado = texto.to_string();
        for (real, remplazo) in &self.sustituciones {
            resultado = resultado.replace(real.as_str(), remplazo.as_str());
        }
        resultado
    }

    /// Qué categorías tienen al menos una sustitución real registrada — para
    /// `DiagnosticPreview.redactedFields`, mostrado antes de guardar (US-051).
    pub fn campos_afectados(&self) -> &[&'static str] {
        &self.campos_afectados
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_anonimizar_no_toca_el_texto() {
        let a = Anonimizador::sin_anonimizar();
        assert_eq!(
            a.aplicar("EQUIPO-DANI, S/N ABC123"),
            "EQUIPO-DANI, S/N ABC123"
        );
        assert!(a.campos_afectados().is_empty());
    }

    #[test]
    fn un_numero_de_serie_se_sustituye_por_un_marcador_numerado() {
        let a = Anonimizador::sin_anonimizar().con_numero_de_serie("S6B2NS0T123456");
        assert_eq!(a.aplicar("serie: S6B2NS0T123456"), "serie: <SERIE-1>");
        assert_eq!(a.campos_afectados(), &[CAMPO_NUMERO_SERIE]);
    }

    #[test]
    fn dos_numeros_de_serie_reciben_marcadores_distintos_y_consistentes() {
        let a = Anonimizador::sin_anonimizar()
            .con_numero_de_serie("SERIE-A")
            .con_numero_de_serie("SERIE-B");
        let texto = "disco 1: SERIE-A, disco 2: SERIE-B, otra vez disco 1: SERIE-A";
        assert_eq!(
            a.aplicar(texto),
            "disco 1: <SERIE-1>, disco 2: <SERIE-2>, otra vez disco 1: <SERIE-1>"
        );
    }

    #[test]
    fn el_nombre_de_equipo_se_sustituye_en_cualquier_ocurrencia() {
        let a = Anonimizador::sin_anonimizar().con_equipo("PC-DANIEL");
        assert_eq!(
            a.aplicar("host=PC-DANIEL; conectado desde PC-DANIEL"),
            "host=<EQUIPO>; conectado desde <EQUIPO>"
        );
        assert_eq!(a.campos_afectados(), &[CAMPO_NOMBRE_EQUIPO]);
    }

    #[test]
    fn una_ruta_de_usuario_se_sustituye_dentro_de_una_ruta_de_windows() {
        let a = Anonimizador::sin_anonimizar().con_ruta_usuario("daniel");
        assert_eq!(
            a.aplicar(r"C:\Users\daniel\AppData\Local\SmartDisk Monitor\log.txt"),
            r"C:\Users\<USUARIO>\AppData\Local\SmartDisk Monitor\log.txt"
        );
    }

    #[test]
    fn una_cadena_real_vacia_no_genera_una_sustitucion_que_borre_texto() {
        // `"".replace("", x)` insertaría `x` entre cada carácter si no se filtrara antes: una
        // variable de entorno vacía o ausente no debe llegar a `push`.
        let a = Anonimizador::sin_anonimizar()
            .con_numero_de_serie("")
            .con_equipo("")
            .con_ruta_usuario("");
        assert_eq!(a.aplicar("texto normal"), "texto normal");
        assert!(a.campos_afectados().is_empty());
    }

    #[test]
    fn la_cadena_mas_larga_se_sustituye_primero() {
        // Si "DANI" fuera literalmente un prefijo del nombre de equipo, sustituir la más corta
        // primero dejaría el resto irreconocible; sustituir la más larga primero lo evita.
        let a = Anonimizador::sin_anonimizar()
            .con_ruta_usuario("DANI")
            .con_equipo("DANI-PC");
        assert_eq!(a.aplicar("equipo DANI-PC"), "equipo <EQUIPO>");
    }

    #[test]
    fn una_etiqueta_de_volumen_se_sustituye_por_un_marcador_numerado_y_consistente() {
        let a = Anonimizador::sin_anonimizar()
            .con_etiqueta_volumen("Sistema")
            .con_etiqueta_volumen("Datos");
        assert_eq!(
            a.aplicar("volumen Sistema (C:), volumen Datos (D:), otra vez Sistema"),
            "volumen <VOLUMEN-1> (C:), volumen <VOLUMEN-2> (D:), otra vez <VOLUMEN-1>"
        );
        assert!(a.campos_afectados().contains(&CAMPO_ETIQUETA_VOLUMEN));
    }

    #[test]
    fn una_etiqueta_de_volumen_vacia_no_genera_sustitucion() {
        let a = Anonimizador::sin_anonimizar().con_etiqueta_volumen("");
        assert_eq!(a.aplicar("texto"), "texto");
        assert!(a.campos_afectados().is_empty());
    }

    #[test]
    fn dos_categorias_distintas_se_listan_las_dos_en_campos_afectados() {
        let a = Anonimizador::sin_anonimizar()
            .con_equipo("PC-1")
            .con_numero_de_serie("S1");
        let campos = a.campos_afectados();
        assert!(campos.contains(&CAMPO_NOMBRE_EQUIPO));
        assert!(campos.contains(&CAMPO_NUMERO_SERIE));
        assert_eq!(campos.len(), 2);
    }
}
