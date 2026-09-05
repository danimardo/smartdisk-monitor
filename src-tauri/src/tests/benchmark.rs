//! Benchmark de lectura y escritura (T079, `docs/product-specification.md` §6).
//!
//! E/S **sin caché del sistema** (`FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH`): sin esto se
//! estaría midiendo la memoria RAM, no el disco, y las cifras no serían comparables entre
//! ejecuciones ni entre discos. La E/S sin búfer de Windows exige que la dirección del buffer, su
//! tamaño y el desplazamiento en el archivo estén alineados al sector — 4096 bytes cubre tanto los
//! discos de sector físico de 512 bytes (emulado) como los de 4096 nativos.
//!
//! El orden de bloques (`orden_de_bloques`) y el cálculo de rendimiento son puros y se prueban sin
//! tocar disco; la ejecución real (`ejecutar`) solo se ha verificado con un archivo pequeño en el
//! directorio temporal de la sesión, nunca con el tamaño real de producción (256 MiB–8 GiB) ni
//! sobre un volumen de usuario — decisión explícita de alcance de esta sesión.

use std::time::{Duration, Instant};

use super::patron::{generar_bloque, verificar_bloque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoAcceso {
    Sequential,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RazonParada {
    Completed,
    Cancelled,
    Thermal,
    Space,
    Error,
}

#[derive(Debug, Clone)]
pub struct ParametrosBenchmark {
    pub size_bytes: u64,
    pub block_size_bytes: u64,
    pub mode: ModoAcceso,
    pub passes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResultadoBenchmark {
    pub passed: Option<bool>,
    pub read_bytes_per_second: Option<f64>,
    pub write_bytes_per_second: Option<f64>,
    pub read_latency_ms: Option<f64>,
    pub write_latency_ms: Option<f64>,
    pub max_temperature_c: Option<f64>,
    pub stopped_reason: RazonParada,
}

/// El orden en que se visitan los bloques del archivo. Secuencial es `0, 1, 2, ...`; aleatorio es
/// una permutación de los mismos índices (nunca inventa ni repite uno) mezclada con un generador
/// determinista de `semilla`, para que el propio orden se pueda probar sin depender del reloj.
pub fn orden_de_bloques(num_bloques: u64, modo: ModoAcceso, semilla: u64) -> Vec<u64> {
    let mut orden: Vec<u64> = (0..num_bloques).collect();
    if modo == ModoAcceso::Random {
        // Fisher-Yates con un LCG propio: no hace falta la calidad criptográfica de `rand`
        // (dependencia que exigiría justificación) para desordenar el orden de E/S de un
        // benchmark, solo evitar que el disco vea un patrón secuencial.
        let mut estado = semilla | 1; // impar: evita un LCG degenerado con semilla 0
        for i in (1..orden.len()).rev() {
            estado = estado
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (estado >> 33) as usize % (i + 1);
            orden.swap(i, j);
        }
    }
    orden
}

/// Bytes por segundo a partir de bytes movidos y tiempo transcurrido. `None` si no hubo tiempo
/// medible (cero bytes, o duración nula) — dividir por cero produciría `inf`, no un dato honesto.
pub fn calcular_bytes_por_segundo(bytes: u64, transcurrido: Duration) -> Option<f64> {
    let segundos = transcurrido.as_secs_f64();
    if bytes == 0 || segundos <= 0.0 {
        return None;
    }
    Some(bytes as f64 / segundos)
}

/// Latencia media por bloque, en milisegundos.
pub fn calcular_latencia_media_ms(transcurrido: Duration, num_bloques: u64) -> Option<f64> {
    if num_bloques == 0 {
        return None;
    }
    Some(transcurrido.as_secs_f64() * 1000.0 / num_bloques as f64)
}

#[cfg(windows)]
mod io_real {
    //! E/S sin caché real, aislada en su propio submódulo para poder mantener `benchmark.rs`
    //! testeable en cualquier plataforma sin condicionar cada función a `#[cfg(windows)]`.
    #![allow(non_snake_case)]

    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::windows::fs::OpenOptionsExt;
    use std::path::Path;

    const FILE_FLAG_WRITE_THROUGH: u32 = 0x8000_0000;
    const FILE_FLAG_NO_BUFFERING: u32 = 0x2000_0000;
    const ALINEACION: usize = 4096;

    /// Un buffer cuya dirección de memoria está alineada al sector, como exige
    /// `FILE_FLAG_NO_BUFFERING`. `Vec<u8>` no lo garantiza: el asignador global no promete más
    /// que la alineación natural del tipo (1 byte para `u8`).
    pub struct BufferAlineado {
        puntero: *mut u8,
        longitud: usize,
        layout: std::alloc::Layout,
    }

    impl BufferAlineado {
        pub fn nuevo(longitud: usize) -> Self {
            let layout = std::alloc::Layout::from_size_align(longitud, ALINEACION)
                .expect("tamaño de bloque inválido para un buffer alineado a sector");
            // SAFETY: `layout` tiene un tamaño y alineación válidos (comprobado arriba); el
            // puntero se libera en `Drop` con el mismo `layout`, nunca se comparte ni se olvida.
            let puntero = unsafe { std::alloc::alloc(layout) };
            assert!(
                !puntero.is_null(),
                "no se pudo reservar el buffer del benchmark"
            );
            Self {
                puntero,
                longitud,
                layout,
            }
        }

        pub fn as_mut_slice(&mut self) -> &mut [u8] {
            // SAFETY: `puntero` apunta a `longitud` bytes reservados y válidos para esta vida.
            unsafe { std::slice::from_raw_parts_mut(self.puntero, self.longitud) }
        }

        pub fn as_slice(&self) -> &[u8] {
            // SAFETY: mismo argumento que `as_mut_slice`, en modo lectura.
            unsafe { std::slice::from_raw_parts(self.puntero, self.longitud) }
        }
    }

    impl Drop for BufferAlineado {
        fn drop(&mut self) {
            // SAFETY: el mismo `layout` que se usó para reservar, nunca se libera dos veces.
            unsafe { std::alloc::dealloc(self.puntero, self.layout) };
        }
    }

    /// Crea el archivo del benchmark. `create_new` es la garantía atómica del sistema operativo
    /// de "nunca se sobrescribe un archivo existente"; `rutas::confirmar_no_sobrescribe` ya lo
    /// comprobó antes, esto es la segunda barrera, no la única.
    pub fn crear_archivo(ruta: &Path) -> std::io::Result<std::fs::File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH)
            .open(ruta)
    }

    pub fn escribir_bloque(
        archivo: &mut std::fs::File,
        offset: u64,
        datos: &[u8],
    ) -> std::io::Result<()> {
        archivo.seek(SeekFrom::Start(offset))?;
        archivo.write_all(datos)
    }

    pub fn leer_bloque(
        archivo: &mut std::fs::File,
        offset: u64,
        datos: &mut [u8],
    ) -> std::io::Result<()> {
        archivo.seek(SeekFrom::Start(offset))?;
        archivo.read_exact(datos)
    }
}

#[cfg(windows)]
pub use io_real::{crear_archivo, BufferAlineado};

/// Ejecuta el benchmark completo: `passes` ciclos de escritura (con patrón comprobable) seguidos
/// de lectura y verificación, en el orden que dicte `parametros.mode`. Se detiene entre bloques si
/// `cancelado` está activo o si `leer_temperatura` supera `limite_termico_c` — nunca a mitad de un
/// bloque, para no dejar una escritura a medias.
#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
pub fn ejecutar(
    ruta: &std::path::Path,
    parametros: &ParametrosBenchmark,
    limite_termico_c: f64,
    mut cancelado: impl FnMut() -> bool,
    mut leer_temperatura: impl FnMut() -> Option<f64>,
    mut on_progreso: impl FnMut(u64, u64),
) -> std::io::Result<ResultadoBenchmark> {
    let num_bloques = parametros.size_bytes.div_ceil(parametros.block_size_bytes);
    let tamano_real = num_bloques * parametros.block_size_bytes;
    let bytes_totales_pasada = tamano_real * 2; // escritura + lectura, para el progreso combinado

    let mut archivo = crear_archivo(ruta)?;
    // El tamaño se fija de antemano: sin esto, el sistema de archivos podría negar espacio a
    // mitad de escritura de un modo distinto a como lo negaría con el archivo ya del tamaño final.
    archivo.set_len(tamano_real)?;

    let mut buffer = BufferAlineado::nuevo(parametros.block_size_bytes as usize);
    let mut max_temperatura: Option<f64> = None;
    let mut bytes_procesados = 0u64;
    let mut duracion_escritura = Duration::ZERO;
    let mut duracion_lectura = Duration::ZERO;
    let mut bloques_escritos = 0u64;
    let mut bloques_leidos = 0u64;

    for pasada in 0..parametros.passes {
        let semilla = 0x9E37_79B9_7F4A_7C15u64.wrapping_add(pasada as u64);
        let orden = orden_de_bloques(num_bloques, parametros.mode, semilla);

        for &indice in &orden {
            if cancelado() {
                return Ok(cerrar_con(
                    RazonParada::Cancelled,
                    max_temperatura,
                    duracion_escritura,
                    duracion_lectura,
                    bloques_escritos,
                    bloques_leidos,
                ));
            }
            if let Some(t) = leer_temperatura() {
                max_temperatura = Some(max_temperatura.map_or(t, |m: f64| m.max(t)));
                if crate::tests::guardia::debe_detenerse_por_temperatura(Some(t), limite_termico_c)
                {
                    return Ok(cerrar_con(
                        RazonParada::Thermal,
                        max_temperatura,
                        duracion_escritura,
                        duracion_lectura,
                        bloques_escritos,
                        bloques_leidos,
                    ));
                }
            }

            let offset = indice * parametros.block_size_bytes;
            generar_bloque(offset, buffer.as_mut_slice());
            let inicio = Instant::now();
            io_real::escribir_bloque(&mut archivo, offset, buffer.as_slice())?;
            duracion_escritura += inicio.elapsed();
            bloques_escritos += 1;
            bytes_procesados += parametros.block_size_bytes;
            on_progreso(
                bytes_procesados,
                bytes_totales_pasada * parametros.passes as u64,
            );
        }
        archivo.sync_all()?;

        for &indice in &orden {
            if cancelado() {
                return Ok(cerrar_con(
                    RazonParada::Cancelled,
                    max_temperatura,
                    duracion_escritura,
                    duracion_lectura,
                    bloques_escritos,
                    bloques_leidos,
                ));
            }
            if let Some(t) = leer_temperatura() {
                max_temperatura = Some(max_temperatura.map_or(t, |m: f64| m.max(t)));
                if crate::tests::guardia::debe_detenerse_por_temperatura(Some(t), limite_termico_c)
                {
                    return Ok(cerrar_con(
                        RazonParada::Thermal,
                        max_temperatura,
                        duracion_escritura,
                        duracion_lectura,
                        bloques_escritos,
                        bloques_leidos,
                    ));
                }
            }

            let offset = indice * parametros.block_size_bytes;
            let inicio = Instant::now();
            io_real::leer_bloque(&mut archivo, offset, buffer.as_mut_slice())?;
            duracion_lectura += inicio.elapsed();
            bloques_leidos += 1;
            bytes_procesados += parametros.block_size_bytes;
            on_progreso(
                bytes_procesados,
                bytes_totales_pasada * parametros.passes as u64,
            );

            if verificar_bloque(offset, buffer.as_slice()).is_some() {
                return Ok(ResultadoBenchmark {
                    passed: Some(false),
                    read_bytes_per_second: calcular_bytes_por_segundo(
                        bloques_leidos * parametros.block_size_bytes,
                        duracion_lectura,
                    ),
                    write_bytes_per_second: calcular_bytes_por_segundo(
                        bloques_escritos * parametros.block_size_bytes,
                        duracion_escritura,
                    ),
                    read_latency_ms: calcular_latencia_media_ms(duracion_lectura, bloques_leidos),
                    write_latency_ms: calcular_latencia_media_ms(
                        duracion_escritura,
                        bloques_escritos,
                    ),
                    max_temperature_c: max_temperatura,
                    stopped_reason: RazonParada::Error,
                });
            }
        }
    }

    Ok(ResultadoBenchmark {
        passed: Some(true),
        read_bytes_per_second: calcular_bytes_por_segundo(
            bloques_leidos * parametros.block_size_bytes,
            duracion_lectura,
        ),
        write_bytes_per_second: calcular_bytes_por_segundo(
            bloques_escritos * parametros.block_size_bytes,
            duracion_escritura,
        ),
        read_latency_ms: calcular_latencia_media_ms(duracion_lectura, bloques_leidos),
        write_latency_ms: calcular_latencia_media_ms(duracion_escritura, bloques_escritos),
        max_temperature_c: max_temperatura,
        stopped_reason: RazonParada::Completed,
    })
}

#[cfg(windows)]
fn cerrar_con(
    razon: RazonParada,
    max_temperatura: Option<f64>,
    duracion_escritura: Duration,
    duracion_lectura: Duration,
    bloques_escritos: u64,
    bloques_leidos: u64,
) -> ResultadoBenchmark {
    ResultadoBenchmark {
        passed: None,
        read_bytes_per_second: None,
        write_bytes_per_second: None,
        read_latency_ms: calcular_latencia_media_ms(duracion_lectura, bloques_leidos),
        write_latency_ms: calcular_latencia_media_ms(duracion_escritura, bloques_escritos),
        max_temperature_c: max_temperatura,
        stopped_reason: razon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_orden_secuencial_es_ascendente() {
        assert_eq!(
            orden_de_bloques(5, ModoAcceso::Sequential, 0),
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn el_orden_aleatorio_contiene_todos_los_indices_una_sola_vez() {
        let mut orden = orden_de_bloques(100, ModoAcceso::Random, 42);
        orden.sort_unstable();
        assert_eq!(orden, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn el_orden_aleatorio_es_determinista_para_la_misma_semilla() {
        let a = orden_de_bloques(50, ModoAcceso::Random, 7);
        let b = orden_de_bloques(50, ModoAcceso::Random, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn semillas_distintas_producen_ordenes_distintos() {
        let a = orden_de_bloques(50, ModoAcceso::Random, 1);
        let b = orden_de_bloques(50, ModoAcceso::Random, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn cero_bloques_no_produce_ni_secuencia_ni_permutacion() {
        assert!(orden_de_bloques(0, ModoAcceso::Sequential, 0).is_empty());
        assert!(orden_de_bloques(0, ModoAcceso::Random, 0).is_empty());
    }

    #[test]
    fn bytes_por_segundo_es_ninguno_sin_tiempo_transcurrido() {
        assert_eq!(calcular_bytes_por_segundo(1024, Duration::ZERO), None);
    }

    #[test]
    fn bytes_por_segundo_es_ninguno_sin_bytes() {
        assert_eq!(calcular_bytes_por_segundo(0, Duration::from_secs(1)), None);
    }

    #[test]
    fn bytes_por_segundo_se_calcula_correctamente() {
        assert_eq!(
            calcular_bytes_por_segundo(1000, Duration::from_secs(2)),
            Some(500.0)
        );
    }

    #[test]
    fn latencia_media_es_ninguna_sin_bloques() {
        assert_eq!(calcular_latencia_media_ms(Duration::from_secs(1), 0), None);
    }

    #[test]
    fn latencia_media_se_calcula_correctamente() {
        assert_eq!(
            calcular_latencia_media_ms(Duration::from_millis(100), 10),
            Some(10.0)
        );
    }
}
