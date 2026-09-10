//! Prueba de **Rendimiento**: orquesta Microsoft DiskSpd (ADR-053) sobre un archivo temporal del
//! volumen, con una matriz fija de 4 perfiles estilo CrystalDiskMark × lectura/escritura.
//!
//! **Puro salvo el lanzamiento**: la configuración de los perfiles, la construcción de la línea de
//! comandos y el cálculo de la duración efectiva son funciones puras, probadas con fixtures. Lanzar
//! DiskSpd y parsear su `-Rxml` es E/S real y se valida en `quickstart.md`, como el resto del
//! proyecto.

use std::path::Path;
use std::time::Duration;

use super::diskspd_xml::{self, SalidaIlegible};

// ---------------------------------------------------------------- valores adoptados (open-questions.md J.62)

/// Ventana de medición objetivo por medición, en segundos.
pub const D_OBJETIVO: u32 = 5;
/// Suelo de medición: por debajo, la cifra es ruido.
pub const D_MIN: u32 = 2;
/// Calentamiento (`-W`): descarta caché SLC y colas frías. Escribe pero no cuenta como medido.
pub const D_CALENTAMIENTO: u32 = 2;
/// Tope de datos por medición de **escritura**: techo de desgaste por perfil. La lectura no tiene.
pub const TOPE_DATOS_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// Tamaño del archivo del benchmark (`-c`). DiskSpd itera sobre él; no lo hace crecer.
pub const TAMANO_ARCHIVO_BYTES: u64 = 1024 * 1024 * 1024;

/// Margen de tiempo extra sobre `D_OBJETIVO + D_CALENTAMIENTO` para el límite de cada invocación de
/// DiskSpd (crear el archivo la primera vez, cerrar, escribir el XML).
const MARGEN_INVOCACION: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------- tipos de dominio

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acceso {
    Secuencial,
    Aleatorio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentido {
    Lectura,
    Escritura,
}

impl Sentido {
    /// Clave estable para el contrato / JSON (`"read"` | `"write"`).
    pub fn clave(self) -> &'static str {
        match self {
            Sentido::Lectura => "read",
            Sentido::Escritura => "write",
        }
    }
}

/// Configuración fija de un perfil. **No** editable por la persona (decisión de producto).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Perfil {
    /// `"seq1m_q8"`, `"seq1m_q1"`, `"rnd4k_q32"`, `"rnd4k_q1"`. Estable, para i18n y JSON.
    pub clave: &'static str,
    pub acceso: Acceso,
    pub bloque_bytes: u64,
    /// Profundidad de cola (`-o`).
    pub cola: u32,
}

/// Los cuatro perfiles «de pico» de CrystalDiskMark, en el orden en que corre la matriz.
pub const PERFILES: [Perfil; 4] = [
    Perfil {
        clave: "seq1m_q8",
        acceso: Acceso::Secuencial,
        bloque_bytes: 1024 * 1024,
        cola: 8,
    },
    Perfil {
        clave: "seq1m_q1",
        acceso: Acceso::Secuencial,
        bloque_bytes: 1024 * 1024,
        cola: 1,
    },
    Perfil {
        clave: "rnd4k_q32",
        acceso: Acceso::Aleatorio,
        bloque_bytes: 4096,
        cola: 32,
    },
    Perfil {
        clave: "rnd4k_q1",
        acceso: Acceso::Aleatorio,
        bloque_bytes: 4096,
        cola: 1,
    },
];

/// Una medición: un perfil en un sentido. Sabe construir su línea de DiskSpd y su `-d` efectivo.
#[derive(Debug, Clone, Copy)]
pub struct Medicion {
    pub perfil: Perfil,
    pub sentido: Sentido,
}

impl Medicion {
    /// Duración efectiva y si el **tope de datos** la recortó (D3 de `research.md`).
    ///
    /// - Lectura, o sin caudal de referencia → `(D_OBJETIVO, false)` (la lectura no desgasta).
    /// - Escritura con caudal `C` bps → `clamp(TOPE_DATOS / C, D_MIN, D_OBJETIVO)`; el booleano es
    ///   `true` si respetar el tope obligó a medir menos de `D_OBJETIVO` (incluso si hubo que subir
    ///   al suelo `D_MIN`: en ese caso el resultado se etiqueta «disco muy rápido»).
    pub fn duracion_efectiva(&self, caudal_referencia_bps: Option<f64>) -> (u32, bool) {
        match (self.sentido, caudal_referencia_bps) {
            (Sentido::Lectura, _) | (_, None) => (D_OBJETIVO, false),
            (Sentido::Escritura, Some(caudal)) if caudal > 0.0 => {
                let segundos_para_tope = TOPE_DATOS_BYTES as f64 / caudal;
                let tope_recorta = segundos_para_tope < D_OBJETIVO as f64;
                let d = segundos_para_tope
                    .clamp(D_MIN as f64, D_OBJETIVO as f64)
                    .round() as u32;
                (d, tope_recorta)
            }
            (Sentido::Escritura, _) => (D_OBJETIVO, false),
        }
    }

    /// Límite de tiempo para la invocación de DiskSpd de esta medición.
    pub fn limite_invocacion(&self, duracion_s: u32) -> Duration {
        Duration::from_secs((duracion_s + D_CALENTAMIENTO) as u64) + MARGEN_INVOCACION
    }

    /// Línea de comandos de DiskSpd para esta medición. **Pura.** `duracion_s` sale de
    /// [`Medicion::duracion_efectiva`].
    pub fn args(&self, ruta_archivo: &Path, tamano_archivo: u64, duracion_s: u32) -> Vec<String> {
        let mut a: Vec<String> = Vec::with_capacity(14);
        a.push(format!("-b{}", self.perfil.bloque_bytes));
        a.push(format!("-o{}", self.perfil.cola));
        a.push("-t1".to_owned());
        match self.perfil.acceso {
            Acceso::Aleatorio => a.push("-r4096".to_owned()),
            Acceso::Secuencial => a.push("-s".to_owned()),
        }
        a.push(match self.sentido {
            Sentido::Lectura => "-w0".to_owned(),
            Sentido::Escritura => "-w100".to_owned(),
        });
        a.push(format!("-d{duracion_s}"));
        a.push(format!("-W{D_CALENTAMIENTO}"));
        a.push("-Sh".to_owned());
        if self.sentido == Sentido::Escritura {
            // Buffer de datos aleatorios: un SSD con compresión no infla el caudal de escritura.
            a.push("-Z1M".to_owned());
        }
        a.push("-L".to_owned());
        a.push("-Rxml".to_owned());
        a.push(format!("-c{tamano_archivo}"));
        a.push(ruta_archivo.to_string_lossy().into_owned());
        a
    }
}

// ---------------------------------------------------------------- resultado de la matriz

/// Una fila de la tabla de resultados (una `Medicion` que llegó a correr).
#[derive(Debug, Clone, PartialEq)]
pub struct FilaResultado {
    pub perfil: &'static str,
    pub sentido: &'static str,
    pub mb_por_segundo: f64,
    pub iops: f64,
    pub latencia_media_ms: f64,
    pub duracion_real_s: f64,
    pub bytes_movidos: u64,
    pub tope_alcanzado: bool,
}

/// Por qué terminó la matriz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RazonParada {
    Completada,
    Cancelada,
    Termica,
    Error,
}

impl RazonParada {
    pub fn clave(self) -> &'static str {
        match self {
            RazonParada::Completada => "completed",
            RazonParada::Cancelada => "cancelled",
            RazonParada::Termica => "thermal",
            RazonParada::Error => "error",
        }
    }
}

/// Lo que devuelve [`orquestar_matriz`].
#[derive(Debug, Clone)]
pub struct ResultadoMatriz {
    pub filas: Vec<FilaResultado>,
    pub no_ejecutadas: Vec<(&'static str, &'static str)>,
    pub razon: RazonParada,
    pub tool_version: Option<String>,
    pub max_temperatura_c: Option<f64>,
    /// Detalle técnico si `razon == Error` (nunca el volcado completo de DiskSpd, puede llevar
    /// rutas).
    pub detalle_error: Option<String>,
}

/// Convierte una [`SalidaDiskspd`] en una [`FilaResultado`]. MB **decimales** por segundo, como
/// CrystalDiskMark.
fn fila_de(
    perfil: &'static str,
    sentido: Sentido,
    salida: &diskspd_xml::SalidaDiskspd,
    tope_alcanzado: bool,
) -> FilaResultado {
    FilaResultado {
        perfil,
        sentido: sentido.clave(),
        mb_por_segundo: salida.bytes as f64 / salida.test_time_s / 1_000_000.0,
        iops: salida.ops as f64 / salida.test_time_s,
        latencia_media_ms: salida.avg_latency_ms,
        duracion_real_s: salida.test_time_s,
        bytes_movidos: salida.bytes,
        tope_alcanzado,
    }
}

/// Recorre la matriz fija: para cada perfil, **lectura** (da el caudal de referencia) y luego
/// **escritura** (su `-d` se dimensiona con ese caudal, D3). Una invocación de DiskSpd por medición.
///
/// - `lanzar`: ejecuta DiskSpd con los `args` dados y un límite, y devuelve `Ok(Some(stdout))` si
///   terminó, `Ok(None)` si se lo mató (límite, cancelación o guardia térmica), `Err` si no se
///   pudo ni lanzar. Se inyecta para poder no depender de `platform::proceso_externo` en test.
/// - `debe_parar`: se consulta antes de cada medición **y** dentro de `lanzar`; `true` → matar y
///   parar. El llamador distingue cancelación de guardia térmica releyendo su estado.
/// - `razon_de_parada`: tras un `Ok(None)`, devuelve por qué (cancelación vs. térmica).
/// - `leer_temperatura`: para `max_temperatura_c` del resumen.
/// - `on_progreso(clave_perfil, sentido, hechas)`: 0..=8.
#[allow(clippy::too_many_arguments)]
pub fn orquestar_matriz(
    ruta_archivo: &Path,
    tamano_archivo: u64,
    mut lanzar: impl FnMut(&[String], Duration) -> std::io::Result<Option<Vec<u8>>>,
    debe_parar: impl Fn() -> bool,
    razon_de_parada: impl Fn() -> RazonParada,
    mut leer_temperatura: impl FnMut() -> Option<f64>,
    mut on_progreso: impl FnMut(&'static str, Sentido, u64),
) -> ResultadoMatriz {
    let mut filas: Vec<FilaResultado> = Vec::with_capacity(8);
    let mut no_ejecutadas: Vec<(&'static str, &'static str)> = Vec::new();
    let mut tool_version: Option<String> = None;
    let mut max_temp: Option<f64> = None;
    let mut hechas: u64 = 0;

    let mut registrar_temp = |t: Option<f64>| {
        if let Some(v) = t {
            max_temp = Some(max_temp.map_or(v, |m: f64| m.max(v)));
        }
    };

    for perfil in PERFILES {
        let mut caudal_lectura_bps: Option<f64> = None;

        for sentido in [Sentido::Lectura, Sentido::Escritura] {
            registrar_temp(leer_temperatura());
            if debe_parar() {
                no_ejecutadas.push((perfil.clave, sentido.clave()));
                marcar_resto_no_ejecutado(perfil, sentido, &mut no_ejecutadas);
                return ResultadoMatriz {
                    filas,
                    no_ejecutadas,
                    razon: razon_de_parada(),
                    tool_version,
                    max_temperatura_c: max_temp,
                    detalle_error: None,
                };
            }

            let medicion = Medicion { perfil, sentido };
            let (duracion_s, tope_alcanzado) = medicion.duracion_efectiva(caudal_lectura_bps);
            let args = medicion.args(ruta_archivo, tamano_archivo, duracion_s);
            let limite = medicion.limite_invocacion(duracion_s);

            match lanzar(&args, limite) {
                Ok(Some(stdout)) => {
                    let xml = String::from_utf8_lossy(&stdout);
                    match diskspd_xml::parsear(&xml, sentido) {
                        Ok(salida) => {
                            tool_version.get_or_insert_with(|| salida.tool_version.clone());
                            if sentido == Sentido::Lectura {
                                caudal_lectura_bps = Some(salida.bytes as f64 / salida.test_time_s);
                            }
                            filas.push(fila_de(perfil.clave, sentido, &salida, tope_alcanzado));
                            hechas += 1;
                            on_progreso(perfil.clave, sentido, hechas);
                        }
                        Err(SalidaIlegible(detalle)) => {
                            no_ejecutadas.push((perfil.clave, sentido.clave()));
                            marcar_resto_no_ejecutado(perfil, sentido, &mut no_ejecutadas);
                            return ResultadoMatriz {
                                filas,
                                no_ejecutadas,
                                razon: RazonParada::Error,
                                tool_version,
                                max_temperatura_c: max_temp,
                                detalle_error: Some(detalle),
                            };
                        }
                    }
                }
                Ok(None) => {
                    let razon = razon_de_parada();
                    no_ejecutadas.push((perfil.clave, sentido.clave()));
                    marcar_resto_no_ejecutado(perfil, sentido, &mut no_ejecutadas);
                    return ResultadoMatriz {
                        filas,
                        no_ejecutadas,
                        razon,
                        tool_version,
                        max_temperatura_c: max_temp,
                        detalle_error: None,
                    };
                }
                Err(e) => {
                    no_ejecutadas.push((perfil.clave, sentido.clave()));
                    marcar_resto_no_ejecutado(perfil, sentido, &mut no_ejecutadas);
                    return ResultadoMatriz {
                        filas,
                        no_ejecutadas,
                        razon: RazonParada::Error,
                        tool_version,
                        max_temperatura_c: max_temp,
                        detalle_error: Some(e.to_string()),
                    };
                }
            }
        }
    }

    ResultadoMatriz {
        filas,
        no_ejecutadas,
        razon: RazonParada::Completada,
        tool_version,
        max_temperatura_c: max_temp,
        detalle_error: None,
    }
}

/// Añade a `no_ejecutadas` todo lo que quedaba de la matriz tras `(perfil, sentido)` inclusive el
/// resto de ese perfil.
fn marcar_resto_no_ejecutado(
    perfil_actual: Perfil,
    sentido_fallido: Sentido,
    no_ejecutadas: &mut Vec<(&'static str, &'static str)>,
) {
    if sentido_fallido == Sentido::Lectura {
        no_ejecutadas.push((perfil_actual.clave, Sentido::Escritura.clave()));
    }
    let mut alcanzado = false;
    for perfil in PERFILES {
        if perfil.clave == perfil_actual.clave {
            alcanzado = true;
            continue;
        }
        if alcanzado {
            no_ejecutadas.push((perfil.clave, Sentido::Lectura.clave()));
            no_ejecutadas.push((perfil.clave, Sentido::Escritura.clave()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ruta() -> PathBuf {
        PathBuf::from("C:\\ProgramData\\SmartDisk Monitor\\SmartDisk Monitor Benchmark\\bench.dat")
    }

    // ---------------------------------------------------------------- Medicion::args

    #[test]
    fn args_seq1m_q8_lectura() {
        let m = Medicion {
            perfil: PERFILES[0],
            sentido: Sentido::Lectura,
        };
        let a = m.args(&ruta(), TAMANO_ARCHIVO_BYTES, 5);
        assert!(a.contains(&"-b1048576".to_owned()));
        assert!(a.contains(&"-o8".to_owned()));
        assert!(a.contains(&"-t1".to_owned()));
        assert!(a.contains(&"-s".to_owned()));
        assert!(a.contains(&"-w0".to_owned()));
        assert!(a.contains(&"-d5".to_owned()));
        assert!(a.contains(&"-W2".to_owned()));
        assert!(a.contains(&"-Sh".to_owned()));
        assert!(a.contains(&"-L".to_owned()));
        assert!(a.contains(&"-Rxml".to_owned()));
        assert!(a.contains(&format!("-c{TAMANO_ARCHIVO_BYTES}")));
        assert!(!a.contains(&"-Z1M".to_owned()), "la lectura no lleva -Z");
        assert!(!a.iter().any(|s| s == "-r4096"), "seq no lleva -r");
        assert_eq!(a.last().unwrap(), &ruta().to_string_lossy().into_owned());
    }

    #[test]
    fn args_rnd4k_q32_escritura() {
        let m = Medicion {
            perfil: PERFILES[2],
            sentido: Sentido::Escritura,
        };
        let a = m.args(&ruta(), TAMANO_ARCHIVO_BYTES, 3);
        assert!(a.contains(&"-b4096".to_owned()));
        assert!(a.contains(&"-o32".to_owned()));
        assert!(a.contains(&"-r4096".to_owned()));
        assert!(a.contains(&"-w100".to_owned()));
        assert!(a.contains(&"-d3".to_owned()));
        assert!(a.contains(&"-Z1M".to_owned()), "la escritura lleva -Z1M");
    }

    #[test]
    fn args_cubre_los_cuatro_perfiles_en_ambos_sentidos() {
        for perfil in PERFILES {
            for sentido in [Sentido::Lectura, Sentido::Escritura] {
                let a = Medicion { perfil, sentido }.args(&ruta(), TAMANO_ARCHIVO_BYTES, 4);
                assert!(a.contains(&format!("-o{}", perfil.cola)));
                assert!(a.contains(&format!("-b{}", perfil.bloque_bytes)));
                assert!(a.contains(&"-Rxml".to_owned()));
            }
        }
    }

    // ---------------------------------------------------------------- duracion_efectiva (D3)

    #[test]
    fn lectura_siempre_corre_el_objetivo_sin_tope() {
        let m = Medicion {
            perfil: PERFILES[0],
            sentido: Sentido::Lectura,
        };
        assert_eq!(m.duracion_efectiva(Some(500e6)), (D_OBJETIVO, false));
        assert_eq!(m.duracion_efectiva(None), (D_OBJETIVO, false));
    }

    #[test]
    fn escritura_sin_caudal_de_referencia_corre_el_objetivo() {
        let m = Medicion {
            perfil: PERFILES[0],
            sentido: Sentido::Escritura,
        };
        assert_eq!(m.duracion_efectiva(None), (D_OBJETIVO, false));
    }

    #[test]
    fn escritura_en_disco_lento_no_alcanza_el_tope_y_corre_el_objetivo() {
        // HDD 150 MB/s: 4 GiB / 150 MB/s ≈ 28 s > D_OBJETIVO → corre 5 s, sin recorte.
        let m = Medicion {
            perfil: PERFILES[1],
            sentido: Sentido::Escritura,
        };
        assert_eq!(m.duracion_efectiva(Some(150e6)), (D_OBJETIVO, false));
    }

    #[test]
    fn escritura_en_disco_medio_recorta_por_el_tope() {
        // SATA SSD 1 GB/s: 4 GiB / 1e9 ≈ 4.29 s < 5 s → corre ~4 s, recortado.
        let m = Medicion {
            perfil: PERFILES[1],
            sentido: Sentido::Escritura,
        };
        let (d, recortado) = m.duracion_efectiva(Some(1.0e9));
        assert!(recortado);
        assert!((D_MIN..=D_OBJETIVO).contains(&d));
    }

    #[test]
    fn escritura_en_disco_muy_rapido_baja_al_suelo_y_se_etiqueta() {
        // Gen5 ~12 GB/s: 4 GiB / 12e9 ≈ 0.36 s < D_MIN → sube al suelo, etiquetado.
        let m = Medicion {
            perfil: PERFILES[1],
            sentido: Sentido::Escritura,
        };
        assert_eq!(m.duracion_efectiva(Some(12.0e9)), (D_MIN, true));
    }

    // ---------------------------------------------------------------- orquestar_matriz

    fn xml_de(nombre: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/tests/fixtures/diskspd")
                .join(nombre),
        )
        .unwrap()
    }

    #[test]
    fn matriz_completa_devuelve_ocho_filas_y_la_version() {
        // Cada invocación devuelve el fixture que toca por su sentido/perfil aproximado.
        let seq_r = xml_de("seq1m_q8_read.xml");
        let seq_w = xml_de("seq1m_q1_write.xml");
        let rnd_r = xml_de("rnd4k_q32_read.xml");
        let rnd_w = xml_de("rnd4k_q1_write.xml");
        let lanzar = |args: &[String], _l: Duration| {
            let es_escritura = args.iter().any(|s| s == "-w100");
            let es_aleatorio = args.iter().any(|s| s == "-r4096");
            let xml = match (es_aleatorio, es_escritura) {
                (false, false) => seq_r.clone(),
                (false, true) => seq_w.clone(),
                (true, false) => rnd_r.clone(),
                (true, true) => rnd_w.clone(),
            };
            Ok(Some(xml))
        };
        let r = orquestar_matriz(
            &ruta(),
            TAMANO_ARCHIVO_BYTES,
            lanzar,
            || false,
            || RazonParada::Cancelada,
            || Some(44.0),
            |_, _, _| {},
        );
        assert_eq!(r.razon, RazonParada::Completada);
        assert_eq!(r.filas.len(), 8);
        assert!(r.no_ejecutadas.is_empty());
        assert_eq!(r.tool_version.as_deref(), Some("2.3.0"));
        assert_eq!(r.max_temperatura_c, Some(44.0));
        assert!(r
            .filas
            .iter()
            .all(|f| f.mb_por_segundo > 0.0 && f.iops > 0.0));
    }

    #[test]
    fn cancelacion_a_mitad_conserva_las_filas_hechas_y_marca_el_resto() {
        let seq_r = xml_de("seq1m_q8_read.xml");
        let seq_w = xml_de("seq1m_q1_write.xml");
        let mut invocaciones = 0;
        let lanzar = |args: &[String], _l: Duration| {
            invocaciones += 1;
            if invocaciones >= 3 {
                return Ok(None); // "matado" por cancelación
            }
            Ok(Some(if args.iter().any(|s| s == "-w100") {
                seq_w.clone()
            } else {
                seq_r.clone()
            }))
        };
        let r = orquestar_matriz(
            &ruta(),
            TAMANO_ARCHIVO_BYTES,
            lanzar,
            || false,
            || RazonParada::Cancelada,
            || None,
            |_, _, _| {},
        );
        assert_eq!(r.razon, RazonParada::Cancelada);
        assert_eq!(r.filas.len(), 2);
        assert_eq!(r.filas.len() + r.no_ejecutadas.len(), 8);
    }

    #[test]
    fn xml_ilegible_es_error_con_detalle_y_filas_parciales() {
        let seq_r = xml_de("seq1m_q8_read.xml");
        let malo = xml_de("error_target_inaccesible.xml");
        let mut invocaciones = 0;
        let lanzar = |_a: &[String], _l: Duration| {
            invocaciones += 1;
            if invocaciones == 1 {
                Ok(Some(seq_r.clone()))
            } else {
                Ok(Some(malo.clone()))
            }
        };
        let r = orquestar_matriz(
            &ruta(),
            TAMANO_ARCHIVO_BYTES,
            lanzar,
            || false,
            || RazonParada::Cancelada,
            || None,
            |_, _, _| {},
        );
        assert_eq!(r.razon, RazonParada::Error);
        assert!(r.detalle_error.is_some());
        assert_eq!(r.filas.len(), 1);
    }

    #[test]
    fn parada_termica_antes_de_empezar_no_ejecuta_nada() {
        let r = orquestar_matriz(
            &ruta(),
            TAMANO_ARCHIVO_BYTES,
            |_a: &[String], _l: Duration| -> std::io::Result<Option<Vec<u8>>> {
                panic!("no debería lanzarse nada")
            },
            || true,
            || RazonParada::Termica,
            || Some(90.0),
            |_, _, _| {},
        );
        assert_eq!(r.filas.len(), 0);
        assert_eq!(r.no_ejecutadas.len(), 8);
        assert_eq!(r.razon, RazonParada::Termica);
    }
}
