//! Reglas de negocio del producto: identidad de dispositivos, cálculo de estado de salud,
//! retención, guardia de espacio y validación de `settings`.
//!
//! **Sin dependencias de Tauri, ni de Windows, ni de SQLite** (constitución §IV,
//! `docs/engineering-conventions.md`): recibe datos y devuelve decisiones. Es lo que permite
//! probar el motor de alertas y las reglas de estado con fixtures y sin hardware.

pub mod actividad;
pub mod ajustes;
pub mod capacidad;
pub mod correlacion;
pub mod espacio;
pub mod estado;
pub mod ia;
pub mod identidad;
pub mod retencion;
pub mod salud;
pub mod series;
pub mod tipos;
