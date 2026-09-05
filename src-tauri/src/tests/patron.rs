//! Patrón de datos comprobable para el benchmark (T079, `docs/product-specification.md` §6:
//! "se escribe, sincroniza, lee y verifica el contenido mediante bloques con patrón comprobable").
//!
//! Cada bloque se rellena con su propio desplazamiento en el archivo, repetido cada 8 bytes: la
//! verificación no necesita conservar lo escrito en memoria (el archivo puede pesar gigabytes),
//! le basta con recalcular lo que un bloque en ese desplazamiento **debería** contener y
//! compararlo byte a byte con lo leído.

const TAMANO_PALABRA: usize = 8;

/// Rellena `buffer` (de cualquier tamaño) con el patrón derivado de `offset_bytes`: los ocho
/// bytes de `offset_bytes` en little-endian, repetidos hasta llenar el buffer.
pub fn generar_bloque(offset_bytes: u64, buffer: &mut [u8]) {
    let palabra = offset_bytes.to_le_bytes();
    for (i, b) in buffer.iter_mut().enumerate() {
        *b = palabra[i % TAMANO_PALABRA];
    }
}

/// Compara `buffer` (ya leído del disco) contra el patrón esperado para `offset_bytes`. Devuelve
/// el primer desplazamiento absoluto donde el contenido no coincide, o `None` si el bloque entero
/// es correcto — la posición exacta es lo único que de verdad ayuda a diagnosticar una corrupción.
pub fn verificar_bloque(offset_bytes: u64, buffer: &[u8]) -> Option<u64> {
    let palabra = offset_bytes.to_le_bytes();
    for (i, &b) in buffer.iter().enumerate() {
        if b != palabra[i % TAMANO_PALABRA] {
            return Some(offset_bytes + i as u64);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_bloque_generado_se_verifica_correcto() {
        let mut buffer = vec![0u8; 1024];
        generar_bloque(4096, &mut buffer);
        assert_eq!(verificar_bloque(4096, &buffer), None);
    }

    #[test]
    fn dos_desplazamientos_distintos_producen_patrones_distintos() {
        let mut a = vec![0u8; 64];
        let mut b = vec![0u8; 64];
        generar_bloque(0, &mut a);
        generar_bloque(4096, &mut b);
        assert_ne!(a, b);
    }

    #[test]
    fn un_byte_corrompido_se_detecta_en_su_posicion_exacta() {
        let mut buffer = vec![0u8; 4096];
        generar_bloque(0, &mut buffer);
        buffer[200] ^= 0xFF; // corrompe un byte a mitad del bloque

        let resultado = verificar_bloque(0, &buffer);
        assert_eq!(resultado, Some(200));
    }

    #[test]
    fn verificar_con_el_offset_equivocado_detecta_el_desajuste() {
        let mut buffer = vec![0u8; 1024];
        generar_bloque(4096, &mut buffer);
        // Verificar como si el bloque perteneciera a otro desplazamiento: no debe coincidir.
        assert!(verificar_bloque(8192, &buffer).is_some());
    }

    #[test]
    fn un_buffer_mas_pequeno_que_una_palabra_tambien_se_rellena_y_verifica() {
        let mut buffer = vec![0u8; 3];
        generar_bloque(300, &mut buffer);
        assert_eq!(verificar_bloque(300, &buffer), None);
    }
}
