//! Clave de demostración compartida para la ayuda con IA (ADR-054, principio XVI).
//!
//! La clave se compila en el binario **solo** si la Release se construyó con la variable de entorno
//! `SDM_OPENROUTER_DEMO_KEY` (ver `build.rs`). **No es un secreto** (ADR-054): la ofuscación XOR
//! —el patrón está en `clave_demo_ofuscacion.rs`, a la vista— solo evita que la clave aparezca en
//! un `strings` del ejecutable. Un clon del repositorio compila sin ella y la función de IA sigue
//! exigiendo una clave propia.
//!
//! La clave **nunca cruza a la interfaz**: `usando_clave_demo` compara en el backend, y el comando
//! `activar_ayuda_ia_compartida` la copia al Administrador de credenciales sin devolverla.

include!("clave_demo_ofuscacion.rs");

use crate::domain::ia::estado_clave_compartida;
use crate::platform::credenciales;

/// La clave de demostración compilada en este binario, o `None` si no se compiló con ella.
pub fn clave_demo() -> Option<String> {
    let hex = option_env!("SDM_OPENROUTER_DEMO_KEY_OFUSCADA")?;
    desofuscar_xor(hex)
}

/// `true` si la credencial guardada en el Administrador de credenciales es exactamente la clave de
/// demostración de este binario. Comparación en el backend: la clave no sale de aquí.
pub fn usando_clave_demo() -> bool {
    let demo = clave_demo();
    let guardada = credenciales::leer();
    estado_clave_compartida(demo.as_deref(), guardada.as_deref()).1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ofuscar_y_desofuscar_es_ida_y_vuelta_y_no_deja_ver_la_clave() {
        let clave = "sk-or-v1-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let velada = ofuscar_xor(clave.as_bytes());
        assert!(
            !velada.contains("sk-or"),
            "el velo deja ver el prefijo: {velada}"
        );
        assert_eq!(desofuscar_xor(&velada).as_deref(), Some(clave));
    }

    #[test]
    fn desofuscar_hexadecimal_invalido_o_vacio_es_none() {
        assert_eq!(desofuscar_xor(""), None);
        assert_eq!(desofuscar_xor("abc"), None); // longitud impar
        assert_eq!(desofuscar_xor("zz"), None); // no es hex
    }

    #[test]
    fn sin_variable_de_compilacion_no_hay_clave_demo() {
        // El arnés de pruebas no define `SDM_OPENROUTER_DEMO_KEY`. Si algún día un build de Release
        // con la clave ejecutara esta suite, el `if` la deja pasar sin fallar.
        if option_env!("SDM_OPENROUTER_DEMO_KEY_OFUSCADA").is_none() {
            assert_eq!(clave_demo(), None);
            assert!(!usando_clave_demo());
        }
    }
}
