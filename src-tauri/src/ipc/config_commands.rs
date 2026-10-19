use std::str::FromStr;

use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::Shortcut as GlobalShortcut;

use super::payload::ConfigSnapshot;
use crate::app::Runtime;
use crate::config::{Config, Trigger};

/// The config in use and whether `config.json` had a problem.
///
/// # Errors
///
/// Only before the runtime has loaded a config, which `setup` does before any window opens.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri hands commands owned arguments.
pub fn get_config(runtime: State<'_, Runtime>) -> Result<ConfigSnapshot, String> {
    snapshot(&runtime)
}

/// Validates, saves and applies a config from the settings window, then returns what is now
/// running. serde has already checked the shape (unknown keys, kinds, buttons); saving checks
/// every rule `config.json` is held to, plus that a keyboard trigger can be registered.
///
/// # Errors
///
/// A plain-English message when the config breaks a rule or cannot be written. Nothing is
/// written or applied in that case.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri hands commands owned arguments.
pub fn save_config(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    config: Config,
) -> Result<ConfigSnapshot, String> {
    if let Trigger::Keyboard(keyboard) = &config.trigger
        && GlobalShortcut::from_str(&keyboard.shortcut).is_err()
    {
        return Err(format!(
            "`{}` cannot be used as a keyboard trigger",
            keyboard.shortcut
        ));
    }
    runtime
        .save_config(&app, &config)
        .map_err(|error| error.to_string())?;
    snapshot(&runtime)
}

/// Re-reads `config.json` (after the user fixed it by hand) and returns what is now running.
///
/// # Errors
///
/// Only before the runtime has loaded a config.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri hands commands owned arguments.
pub fn reload_config(
    app: AppHandle,
    runtime: State<'_, Runtime>,
) -> Result<ConfigSnapshot, String> {
    tracing::info!("config reload requested from settings");
    runtime.apply_config(&app);
    snapshot(&runtime)
}

fn snapshot(runtime: &Runtime) -> Result<ConfigSnapshot, String> {
    runtime
        .loaded()
        .map(|loaded| ConfigSnapshot::new(loaded, &runtime.config_path()))
        .ok_or_else(|| "the config has not been loaded yet".to_owned())
}
