use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::payload::AppStatus;
use crate::lifecycle::UpdateState;

/// Label of the settings window (see `settings::open`).
const SETTINGS_WINDOW: &str = "settings";
/// The running config changed (saved from settings or reloaded from the tray): re-read it.
const CONFIG_CHANGED: &str = "config:changed";
/// Pause state or the tray problem changed. Payload: [`AppStatus`].
const STATUS_CHANGED: &str = "status:changed";
/// Update checking or installing moved on. Payload: [`UpdateState`].
const UPDATE_CHANGED: &str = "update:changed";

/// Tells the settings window, if it exists, to re-read the config.
pub fn emit_config_changed(app: &AppHandle) {
    emit(app, CONFIG_CHANGED, ());
}

/// Tells the settings window, if it exists, the new pause state and tray problem.
pub fn emit_status_changed(app: &AppHandle, paused: bool, problem: Option<String>) {
    emit(app, STATUS_CHANGED, AppStatus { paused, problem });
}

/// Tells the settings window, if it exists, where updating stands.
pub fn emit_update_changed(app: &AppHandle, state: &UpdateState) {
    emit(app, UPDATE_CHANGED, state.clone());
}

fn emit<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    // The window is created on first open; before that there is nobody to tell.
    if app.get_webview_window(SETTINGS_WINDOW).is_none() {
        return;
    }
    if let Err(error) = app.emit_to(SETTINGS_WINDOW, event, payload) {
        tracing::warn!(%error, event, "could not notify the settings window");
    }
}
