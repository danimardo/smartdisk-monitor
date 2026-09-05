//! Detección de la codificación de la salida de `chkdsk` y demás procesos auxiliares
//! (T080, `docs/open-questions.md` §Q, `tools/console-encoding.py` — la heurística de
//! referencia, ya validada contra un Windows real; este módulo es su puerto a Rust, mismo
//! algoritmo, mismo orden de decisión).
//!
//! **Por qué no hay una constante**: en el mismo equipo, el mismo día, por el mismo tipo de
//! tubería, `chkdsk`/`chkntfs` emiten CP1252 (ANSI) y `fsutil`/`vssadmin` emiten CP850 (OEM) — no
//! hay una regla del sistema que seguir, depende de cómo se escribió cada herramienta. Fijar
//! `chcp` antes de invocar tampoco sirve: no cambia nada cuando la salida está redirigida
//! (medido, §Q.3).
//!
//! Las tablas de las tres páginas de un byte están generadas con los códecs reales de Python
//! (`bytes([b]).decode(cp)` para cada `b` de 0x80 a 0xFF), no transcritas a mano: un error de
//! transcripción en una tabla de 128 entradas sería fácil de cometer y difícil de notar.

/// Letras que un texto real en español o inglés produce de verdad.
const ESPERADAS: &str = "áéíóúüñÁÉÍÓÚÜÑ¿¡ºª°·—€\u{201c}\u{201d}\u{2018}\u{2019}";
/// Lo que sale cuando se acierta la familia pero no la página: griego, dibujo de cajas, matemáticas.
const SOSPECHOSAS: &str = "ßÚÝ±¾·░▒▓│┤╡╢╖╕╣║╗╝┐└┴┬├─┼╞╟╚╔╩╦╠═╬¤ðÐÊËÈıÍÎÏ┘┌█▄▌▐▀αΓπΣσµτΦΘΩδ∞φε∩";

/// Bytes 0x80-0xFF → codepoint Unicode. Generada con `bytes([b]).decode("cp1252")` en Python.
const CP1252_ALTA: [u32; 128] = [
    0x20AC, 0xFFFD, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039,
    0x0152, 0xFFFD, 0x017D, 0xFFFD, 0xFFFD, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014,
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0xFFFD, 0x017E, 0x0178, 0x00A0, 0x00A1, 0x00A2, 0x00A3,
    0x00A4, 0x00A5, 0x00A6, 0x00A7, 0x00A8, 0x00A9, 0x00AA, 0x00AB, 0x00AC, 0x00AD, 0x00AE, 0x00AF,
    0x00B0, 0x00B1, 0x00B2, 0x00B3, 0x00B4, 0x00B5, 0x00B6, 0x00B7, 0x00B8, 0x00B9, 0x00BA, 0x00BB,
    0x00BC, 0x00BD, 0x00BE, 0x00BF, 0x00C0, 0x00C1, 0x00C2, 0x00C3, 0x00C4, 0x00C5, 0x00C6, 0x00C7,
    0x00C8, 0x00C9, 0x00CA, 0x00CB, 0x00CC, 0x00CD, 0x00CE, 0x00CF, 0x00D0, 0x00D1, 0x00D2, 0x00D3,
    0x00D4, 0x00D5, 0x00D6, 0x00D7, 0x00D8, 0x00D9, 0x00DA, 0x00DB, 0x00DC, 0x00DD, 0x00DE, 0x00DF,
    0x00E0, 0x00E1, 0x00E2, 0x00E3, 0x00E4, 0x00E5, 0x00E6, 0x00E7, 0x00E8, 0x00E9, 0x00EA, 0x00EB,
    0x00EC, 0x00ED, 0x00EE, 0x00EF, 0x00F0, 0x00F1, 0x00F2, 0x00F3, 0x00F4, 0x00F5, 0x00F6, 0x00F7,
    0x00F8, 0x00F9, 0x00FA, 0x00FB, 0x00FC, 0x00FD, 0x00FE, 0x00FF,
];

/// Bytes 0x80-0xFF → codepoint Unicode. Generada con `bytes([b]).decode("cp850")` en Python.
const CP850_ALTA: [u32; 128] = [
    0x00C7, 0x00FC, 0x00E9, 0x00E2, 0x00E4, 0x00E0, 0x00E5, 0x00E7, 0x00EA, 0x00EB, 0x00E8, 0x00EF,
    0x00EE, 0x00EC, 0x00C4, 0x00C5, 0x00C9, 0x00E6, 0x00C6, 0x00F4, 0x00F6, 0x00F2, 0x00FB, 0x00F9,
    0x00FF, 0x00D6, 0x00DC, 0x00F8, 0x00A3, 0x00D8, 0x00D7, 0x0192, 0x00E1, 0x00ED, 0x00F3, 0x00FA,
    0x00F1, 0x00D1, 0x00AA, 0x00BA, 0x00BF, 0x00AE, 0x00AC, 0x00BD, 0x00BC, 0x00A1, 0x00AB, 0x00BB,
    0x2591, 0x2592, 0x2593, 0x2502, 0x2524, 0x00C1, 0x00C2, 0x00C0, 0x00A9, 0x2563, 0x2551, 0x2557,
    0x255D, 0x00A2, 0x00A5, 0x2510, 0x2514, 0x2534, 0x252C, 0x251C, 0x2500, 0x253C, 0x00E3, 0x00C3,
    0x255A, 0x2554, 0x2569, 0x2566, 0x2560, 0x2550, 0x256C, 0x00A4, 0x00F0, 0x00D0, 0x00CA, 0x00CB,
    0x00C8, 0x0131, 0x00CD, 0x00CE, 0x00CF, 0x2518, 0x250C, 0x2588, 0x2584, 0x00A6, 0x00CC, 0x2580,
    0x00D3, 0x00DF, 0x00D4, 0x00D2, 0x00F5, 0x00D5, 0x00B5, 0x00FE, 0x00DE, 0x00DA, 0x00DB, 0x00D9,
    0x00FD, 0x00DD, 0x00AF, 0x00B4, 0x00AD, 0x00B1, 0x2017, 0x00BE, 0x00B6, 0x00A7, 0x00F7, 0x00B8,
    0x00B0, 0x00A8, 0x00B7, 0x00B9, 0x00B3, 0x00B2, 0x25A0, 0x00A0,
];

/// Bytes 0x80-0xFF → codepoint Unicode. Generada con `bytes([b]).decode("cp437")` en Python.
const CP437_ALTA: [u32; 128] = [
    0x00C7, 0x00FC, 0x00E9, 0x00E2, 0x00E4, 0x00E0, 0x00E5, 0x00E7, 0x00EA, 0x00EB, 0x00E8, 0x00EF,
    0x00EE, 0x00EC, 0x00C4, 0x00C5, 0x00C9, 0x00E6, 0x00C6, 0x00F4, 0x00F6, 0x00F2, 0x00FB, 0x00F9,
    0x00FF, 0x00D6, 0x00DC, 0x00A2, 0x00A3, 0x00A5, 0x20A7, 0x0192, 0x00E1, 0x00ED, 0x00F3, 0x00FA,
    0x00F1, 0x00D1, 0x00AA, 0x00BA, 0x00BF, 0x2310, 0x00AC, 0x00BD, 0x00BC, 0x00A1, 0x00AB, 0x00BB,
    0x2591, 0x2592, 0x2593, 0x2502, 0x2524, 0x2561, 0x2562, 0x2556, 0x2555, 0x2563, 0x2551, 0x2557,
    0x255D, 0x255C, 0x255B, 0x2510, 0x2514, 0x2534, 0x252C, 0x251C, 0x2500, 0x253C, 0x255E, 0x255F,
    0x255A, 0x2554, 0x2569, 0x2566, 0x2560, 0x2550, 0x256C, 0x2567, 0x2568, 0x2564, 0x2565, 0x2559,
    0x2558, 0x2552, 0x2553, 0x256B, 0x256A, 0x2518, 0x250C, 0x2588, 0x2584, 0x258C, 0x2590, 0x2580,
    0x03B1, 0x00DF, 0x0393, 0x03C0, 0x03A3, 0x03C3, 0x00B5, 0x03C4, 0x03A6, 0x0398, 0x03A9, 0x03B4,
    0x221E, 0x03C6, 0x03B5, 0x2229, 0x2261, 0x00B1, 0x2265, 0x2264, 0x2320, 0x2321, 0x00F7, 0x2248,
    0x00B0, 0x2219, 0x00B7, 0x221A, 0x207F, 0x00B2, 0x25A0, 0x00A0,
];

/// Decodifica `datos` como una página de un byte (0x00-0x7F es ASCII en las tres), o `None` si
/// algún byte no tiene equivalente en la tabla (`0xFFFD`, solo posible en CP1252).
fn decodificar_pagina_de_un_byte(datos: &[u8], tabla_alta: &[u32; 128]) -> Option<String> {
    let mut resultado = String::with_capacity(datos.len());
    for &b in datos {
        let cp = if b < 0x80 {
            b as u32
        } else {
            tabla_alta[(b - 0x80) as usize]
        };
        if cp == 0xFFFD {
            return None;
        }
        resultado.push(char::from_u32(cp)?);
    }
    Some(resultado)
}

/// Cuantas más letras plausibles y menos símbolos raros, mejor (mismo criterio que
/// `console-encoding.py::puntuar`).
fn puntuar(texto: &str) -> i64 {
    let esperadas = texto.chars().filter(|c| ESPERADAS.contains(*c)).count() as i64;
    let sospechosas = texto.chars().filter(|c| SOSPECHOSAS.contains(*c)).count() as i64;
    esperadas - 3 * sospechosas
}

/// Codificación detectada más el texto ya decodificado. Nunca falla ni pierde la salida: el
/// último recurso es CP1252 con sustitución (`replace`), que es lo mismo que hace `chkdsk` en la
/// inmensa mayoría de los casos observados (§Q.4).
#[derive(Debug, Clone, PartialEq)]
pub struct DeteccionCodificacion {
    pub codificacion: String,
    pub texto: String,
}

/// Orden de decisión, idéntico a `console-encoding.py::detectar`:
/// 1. Un BOM manda — declaración explícita, no conjetura.
/// 2. Todo ASCII: UTF-8 vale y no se adivina nada.
/// 3. UTF-8 estricto: cubre los equipos con "Beta: usar Unicode UTF-8" activado.
/// 4. Si no, se puntúan CP1252/CP850/CP437 y gana la de mayor puntuación positiva.
/// 5. Empate o puntuación nula: CP1252, la página que emite `chkdsk` (nunca se falla ni se
///    pierde la salida).
pub fn detectar(datos: &[u8]) -> DeteccionCodificacion {
    if let Some(resto) = datos.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return DeteccionCodificacion {
            codificacion: "utf-8-sig".to_string(),
            texto: String::from_utf8_lossy(resto).into_owned(),
        };
    }

    if !datos.iter().any(|&b| b > 127) {
        // Solo ASCII: `from_utf8` no puede fallar aquí.
        return DeteccionCodificacion {
            codificacion: "ascii".to_string(),
            texto: String::from_utf8_lossy(datos).into_owned(),
        };
    }

    if let Ok(texto) = std::str::from_utf8(datos) {
        return DeteccionCodificacion {
            codificacion: "utf-8".to_string(),
            texto: texto.to_string(),
        };
    }

    let candidatas: [(&str, &[u32; 128]); 3] = [
        ("cp1252", &CP1252_ALTA),
        ("cp850", &CP850_ALTA),
        ("cp437", &CP437_ALTA),
    ];
    let mut mejor: Option<(&str, String, i64)> = None;
    for (nombre, tabla) in candidatas {
        let Some(texto) = decodificar_pagina_de_un_byte(datos, tabla) else {
            continue;
        };
        let puntuacion = puntuar(&texto);
        let mejora = match &mejor {
            Some((_, _, p)) => puntuacion > *p,
            None => true,
        };
        if mejora {
            mejor = Some((nombre, texto, puntuacion));
        }
    }

    match mejor {
        Some((nombre, texto, puntuacion)) if puntuacion > 0 => DeteccionCodificacion {
            codificacion: nombre.to_string(),
            texto,
        },
        _ => DeteccionCodificacion {
            codificacion: "cp1252".to_string(),
            texto: decodificar_pagina_de_un_byte(datos, &CP1252_ALTA)
                .unwrap_or_else(|| String::from_utf8_lossy(datos).into_owned()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `áéíñóú` en CP1252 (medido, §Q.1).
    const CHKDSK_CP1252: [u8; 6] = [0xE1, 0xE9, 0xED, 0xF1, 0xF3, 0xFA];
    /// `áíóúñ` en CP850 (medido, §Q.2).
    const FSUTIL_CP850: [u8; 5] = [0xA0, 0xA1, 0xA2, 0xA3, 0xA4];

    #[test]
    fn un_bom_utf8_manda_sobre_cualquier_conjetura() {
        let datos = [0xEF, 0xBB, 0xBF, b'h', b'i'];
        let d = detectar(&datos);
        assert_eq!(d.codificacion, "utf-8-sig");
        assert_eq!(d.texto, "hi");
    }

    #[test]
    fn solo_ascii_no_adivina_nada() {
        let d = detectar(b"CHKDSK is verifying files");
        assert_eq!(d.codificacion, "ascii");
        assert_eq!(d.texto, "CHKDSK is verifying files");
    }

    #[test]
    fn utf8_estricto_se_reconoce_como_tal() {
        let d = detectar("café".as_bytes());
        assert_eq!(d.codificacion, "utf-8");
        assert_eq!(d.texto, "café");
    }

    #[test]
    fn la_salida_real_de_chkdsk_se_detecta_como_cp1252() {
        let d = detectar(&CHKDSK_CP1252);
        assert_eq!(d.codificacion, "cp1252");
        assert_eq!(d.texto, "áéíñóú");
    }

    #[test]
    fn la_salida_real_de_fsutil_se_detecta_como_cp850() {
        let d = detectar(&FSUTIL_CP850);
        assert_eq!(d.codificacion, "cp850");
        assert_eq!(d.texto, "áíóúñ");
    }

    #[test]
    fn cp850_interpretado_como_cp1252_produciria_simbolos_sospechosos_y_pierde() {
        // Los mismos bytes de fsutil, forzados por CP1252, deben perder frente a CP850.
        let como_cp1252 = decodificar_pagina_de_un_byte(&FSUTIL_CP850, &CP1252_ALTA).unwrap();
        let como_cp850 = decodificar_pagina_de_un_byte(&FSUTIL_CP850, &CP850_ALTA).unwrap();
        assert!(puntuar(&como_cp850) > puntuar(&como_cp1252));
    }

    #[test]
    fn sin_ninguna_senal_clara_cae_a_cp1252_sin_perder_la_salida() {
        // 0xFF: ÿ en CP1252 (ni esperada ni sospechosa), espacio de no separación en CP850/CP437
        // (tampoco). Empate a cero en las tres: cae al último recurso (CP1252) y produce algo,
        // nunca un error.
        let datos = [0xFF];
        let d = detectar(&datos);
        assert_eq!(d.codificacion, "cp1252");
        assert_eq!(d.texto, "ÿ");
    }
}
