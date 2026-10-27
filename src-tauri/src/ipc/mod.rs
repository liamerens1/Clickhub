//! Tauri commands for the settings window and the events Rust sends it. Validates payloads and
//! delegates; holds no logic of its own.

// `tauri::generate_handler!` needs each command by the path of the module that defines it (it
// looks for macro-generated items next to the function), so the command modules are public
// instead of re-exported.
pub(crate) mod app_commands;
pub(crate) mod config_commands;
mod events;
pub(crate) mod lifecycle_commands;
mod payload;

pub(crate) use events::{emit_config_changed, emit_status_changed, emit_update_changed};
