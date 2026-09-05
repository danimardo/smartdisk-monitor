//! Ayuda exclusiva de pruebas: un directorio temporal garantizado único por llamada.
//!
//! `std::process::id()` + `SystemTime::now().as_nanos()` puede colisionar bajo ejecución paralela
//! de pruebas si la resolución del reloj es gruesa: dos hilos que piden un directorio en el mismo
//! tick reciben el mismo nombre, abren el mismo fichero SQLite y una escritura de un test choca
//! con la otra con `database is locked` — un fallo intermitente que no tiene nada que ver con la
//! lógica probada. Un contador atómico dentro del proceso no puede repetirse.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static CONTADOR: AtomicU64 = AtomicU64::new(0);

pub fn temp_dir_unico(prefijo: &str) -> PathBuf {
    let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("smartdisk_{prefijo}_{}_{n}", std::process::id()))
}
