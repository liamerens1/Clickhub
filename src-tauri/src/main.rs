// Release builds are GUI-subsystem binaries: no console window behind the tray app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    clickhub_lib::run();
}
