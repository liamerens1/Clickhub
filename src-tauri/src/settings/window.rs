use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

const SETTINGS_WINDOW_LABEL: &str = "settings";
const SETTINGS_ENTRY: &str = "settings/index.html";

/// Shows the settings window, creating it on first use. Failures are logged, never fatal.
pub fn open(app: &AppHandle) {
    let window = match app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        Some(window) => window,
        None => match create(app) {
            Ok(window) => window,
            Err(error) => {
                tracing::error!(%error, "could not create the settings window");
                return;
            }
        },
    };
    if let Err(error) = reveal(&window) {
        tracing::warn!(%error, "could not bring the settings window to the front");
    }
}

fn create(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let url = WebviewUrl::App(SETTINGS_ENTRY.into());
    let window = WebviewWindowBuilder::new(app, SETTINGS_WINDOW_LABEL, url)
        .title("ClickHub settings")
        .inner_size(960.0, 640.0)
        .min_inner_size(640.0, 420.0)
        .center()
        .build()?;

    // Closing hides instead of destroying: reopening is instant and the app keeps running.
    let handle = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Err(error) = handle.hide() {
                tracing::warn!(%error, "could not hide the settings window");
            }
        }
    });
    Ok(window)
}

fn reveal(window: &WebviewWindow) -> tauri::Result<()> {
    window.show()?;
    window.unminimize()?;
    window.set_focus()
}
