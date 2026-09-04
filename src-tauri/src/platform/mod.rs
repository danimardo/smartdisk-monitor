//! Envolturas de las APIs de Windows, aisladas para poder simularlas en test y para que el
//! dominio no dependa del sistema operativo (`docs/engineering-conventions.md` §2).

pub mod accent;
pub mod locale;
pub mod paths;
pub mod ventana;
