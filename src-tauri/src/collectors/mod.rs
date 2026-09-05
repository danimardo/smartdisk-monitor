//! Colectores de datos: `smartctl`, inventario de almacenamiento de Windows, contadores de
//! rendimiento y registro de eventos del sistema.
//!
//! Cada colector es la única pieza que sabe invocar su fuente externa. Traduce la salida cruda a
//! los tipos de `domain/` y **nunca** decide estado ni severidad: eso es responsabilidad exclusiva
//! de `domain/` (docs/architecture.md, principio IV).

pub mod capacidad;
pub mod deteccion;
pub mod event_log;
pub mod perf_counters;
pub mod planificador;
pub mod smartctl;
pub mod smartctl_parser;
pub mod windows_storage;
