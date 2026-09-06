//! Envolturas de las APIs de Windows, aisladas para poder simularlas en test y para que el
//! dominio no dependa del sistema operativo (`docs/engineering-conventions.md` §2).

pub mod accent;
pub mod autoarranque;
pub mod bandeja;
pub mod energia;
pub mod locale;
pub mod paths;
pub mod rotulos;
pub mod sistema;
pub mod ventana;
