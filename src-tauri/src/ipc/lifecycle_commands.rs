use tauri::{AppHandle, State};

use crate::lifecycle::{self, AutostartState, UpdateState, Updates};

/// Whether ClickHub starts with Windows (and whether this copy can).
///
/// # Errors
///
/// A message when the registry cannot be read.
#[tauri::command]
pub fn get_autostart() -> Result<AutostartState, String> {
    lifecycle::autostart_state()
}

/// # Errors
///
/// A message when this copy is not installed (switching on) or the registry cannot be written.
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<AutostartState, String> {
    lifecycle::set_autostart(enabled)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri hands commands owned arguments.
pub fn get_update_state(updates: State<'_, Updates>) -> UpdateState {
    updates.state()
}

/// Checks for a newer version now and returns the result.
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    updates: State<'_, Updates>,
) -> Result<UpdateState, String> {
    Ok(updates.check(&app, true).await)
}

/// Downloads and installs the update found by the last check. On success ClickHub exits and
/// the installer restarts it, so the reply may never arrive.
///
/// # Errors
///
/// A message when there is nothing to install or the download or install fails.
#[tauri::command]
pub async fn install_update(app: AppHandle, updates: State<'_, Updates>) -> Result<(), String> {
    updates.install(&app).await
}

/// Starts the uninstaller and quits. The page asks the user first.
///
/// # Errors
///
/// A message when this copy was not installed or the uninstaller cannot be started.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn uninstall(app: AppHandle) -> Result<(), String> {
    lifecycle::launch_uninstaller(&app)
}
