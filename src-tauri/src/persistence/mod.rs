//! Persistencia: apertura de la base SQLite, ejecutor de migraciones y repositorios por entidad.
//!
//! Todo se almacena en `%ProgramData%\SmartDisk Monitor\`, con WAL y transacciones breves
//! (constitución §V). Las migraciones son SQL numerado y **nunca se editan una vez publicadas**;
//! un cambio posterior es una migración nueva. Antes de migrar se crea una copia consistente y se
//! conservan las tres más recientes.

pub mod db;
pub mod migrations;
pub mod repo_agregados;
pub mod repo_alertas;
pub mod repo_inventario;
pub mod repo_metricas;
pub mod repo_varios;
