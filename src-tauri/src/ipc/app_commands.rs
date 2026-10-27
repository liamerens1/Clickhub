use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use super::payload::{AppInfo, AppStatus, Folder};
use crate::app::Runtime;
use crate::lifecycle;
use crate::platform;

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri hands commands owned arguments.
pub fn get_status(runtime: State<'_, Runtime>) -> AppStatus {
    let (paused, problem) = runtime.status();
    AppStatus { paused, problem }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn set_paused(app: AppHandle, runtime: State<'_, Runtime>, paused: bool) -> AppStatus {
    runtime.set_paused(&app, paused);
    let (paused, problem) = runtime.status();
    AppStatus { paused, problem }
}

/// Switches the trigger off while the settings window waits for "press the button you want",
/// and back on afterwards.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn set_trigger_capture(app: AppHandle, runtime: State<'_, Runtime>, capturing: bool) {
    runtime.set_capturing(&app, capturing);
}

/// Opens the Windows file dialog over the settings window and returns the chosen path, or
/// `None` if the user cancelled. Async so the dialog never blocks the main thread.
#[tauri::command]
pub async fn pick_file(app: AppHandle) -> Option<String> {
    let mut dialog = app
        .dialog()
        .file()
        .set_title("Choose an app or file")
        .add_filter("Apps and shortcuts", &["exe", "lnk", "bat", "cmd", "msc"])
        .add_filter("All files", &["*"]);
    if let Some(window) = app.get_webview_window("settings") {
        dialog = dialog.set_parent(&window);
    }
    let picked = dialog.blocking_pick_file()?;
    match picked.into_path() {
        Ok(path) => Some(path.display().to_string()),
        Err(error) => {
            tracing::warn!(%error, "the file dialog returned something that is not a path");
            None
        }
    }
}

/// Opens the config or log folder in Explorer.
///
/// # Errors
///
/// A message if the folder cannot be resolved or the helper thread cannot start.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn open_folder(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    folder: Folder,
) -> Result<(), String> {
    let path = match folder {
        Folder::Config => runtime.config_dir().to_path_buf(),
        Folder::Logs => app
            .path()
            .app_log_dir()
            .map_err(|error| error.to_string())?,
    };
    let target = path.display().to_string();
    // ShellExecute wants a COM-initialised thread; command threads are shared, so use our own.
    std::thread::Builder::new()
        .name("clickhub-open-folder".to_owned())
        .spawn(move || {
            platform::init_shell_thread();
            if let Err(error) = platform::shell_open(&target, &[]) {
                tracing::warn!(%error, "could not open a folder from settings");
            }
        })
        .map(|_| ())
        .map_err(|error| format!("could not open {}: {error}", path.display()))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn get_app_info(app: AppHandle, runtime: State<'_, Runtime>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        installed: lifecycle::installed_exe().is_some(),
        config_path: runtime.config_path().display().to_string(),
        log_dir: app
            .path()
            .app_log_dir()
            .ok()
            .map(|dir| dir.display().to_string()),
    }
}
