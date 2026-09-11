//! Exportación de informes y generación del paquete de diagnóstico.
//!
//! El paquete de diagnóstico es **anonimizado por defecto** (FR-028): sin números de serie,
//! nombres de equipo, rutas de usuario ni nada que identifique a una persona. Incluir datos
//! identificativos exige que el usuario lo pida expresamente, con aviso previo del contenido.

pub mod anonimizar;
pub mod diagnostico;
pub mod export;
pub mod informe;
pub mod informe_ia;
pub mod minigrafica;
pub mod resumen_metricas;
