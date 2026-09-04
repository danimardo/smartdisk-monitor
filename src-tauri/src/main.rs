// Sin consola en la compilación de release: es una aplicación de ventana, no de línea de órdenes.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    smartdisk_lib::run()
}
