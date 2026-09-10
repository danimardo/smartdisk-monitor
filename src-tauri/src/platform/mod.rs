//! Envolturas de las APIs de Windows, aisladas para poder simularlas en test y para que el
//! dominio no dependa del sistema operativo (`docs/engineering-conventions.md` §2).

pub mod accent;
pub mod autoarranque;
pub mod bandeja;
pub mod credenciales;
pub mod diskspd;
pub mod energia;
pub mod ia_clave_demo;
pub mod ia_openrouter;
pub mod locale;
pub mod paths;
pub mod proceso_externo;
pub mod proteccion_carpetas;
pub mod rotulos;
pub mod sistema;
pub mod ventana;
