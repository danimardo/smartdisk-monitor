//! Pruebas manuales bajo demanda: benchmark de lectura/escritura, `chkdsk /scan` y autotest SMART
//! corto.
//!
//! Toda acción de este módulo escribe datos o genera carga, así que exige confirmación previa con
//! impacto explícito y la orden literal cuando exista (`docs/ui-design.md` §5). La validación de
//! rutas del benchmark es una de las cinco áreas de test-first obligatorio (constitución §VIII).

pub mod autotest;
pub mod benchmark;
pub mod chkdsk;
pub mod codificacion;
pub mod guardia;
pub mod patron;
pub mod rutas;
