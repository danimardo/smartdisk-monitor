//! Pruebas manuales bajo demanda: **Rendimiento** (benchmark de disco con Microsoft DiskSpd,
//! ADR-053), `chkdsk /scan` y autotest SMART corto.
//!
//! Toda acción de este módulo escribe datos o genera carga, así que exige confirmación previa con
//! impacto explícito y la orden literal cuando exista (`docs/ui-design.md` §5). La validación de
//! rutas del benchmark y el parseo del XML de DiskSpd son áreas de test-first obligatorio
//! (constitución §VIII).

pub mod autotest;
pub mod chkdsk;
pub mod codificacion;
pub mod diskspd;
pub mod diskspd_xml;
pub mod guardia;
pub mod rutas;
