use std::path::PathBuf;

use tauri::{App, AppHandle, Manager, RunEvent};

use super::runtime::Runtime;
use crate::settings;
use crate::tray::{self, MenuAction};

/// Set to `1` to show the debug items in the tray menu (simulate hook loss, stall core).
const DEBUG_MENU_ENV: &str = "CLICKHUB_DEBUG_MENU";

/// Builds and runs the app. Returns when the user quits from the tray.
pub fn run() {
    let context = tauri::generate_context!();
    // Same directory Tauri's `app_log_dir` resolves to on Windows, known before the app exists.
    let log_dir = std::env::var_os("LOCALAPPDATA").map(|dir| {
        PathBuf::from(dir)
            .join(&context.config().identifier)
            .join("logs")
    });
    let log_guard = super::logging::init(log_dir.as_deref());
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "ClickHub starting");

    let app = tauri::Builder::default()
        // Registered first: a second launch hands over to this instance, then exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tracing::info!("second instance launched; opening settings here instead");
            settings::open(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(setup)
        .build(context);

    let app = match app {
        Ok(app) => app,
        Err(error) => {
            tracing::error!(%error, "failed to build the Tauri app");
            std::process::exit(1);
        }
    };

    let code = app.run_return(on_run_event);
    tracing::info!(code, "ClickHub exited");
    // Flush buffered log lines before the process ends.
    drop(log_guard);
    std::process::exit(code);
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show_debug_items = std::env::var(DEBUG_MENU_ENV).is_ok_and(|value| value == "1");
    tray::build(app.handle(), show_debug_items, on_menu_action)?;
    let runtime = Runtime::start(app.handle())?;
    app.manage(runtime);
    // Starts hidden: no window exists until the user opens settings.
    tracing::info!(show_debug_items, "tray ready");
    Ok(())
}

fn on_menu_action(app: &AppHandle, action: MenuAction) {
    let runtime = app.try_state::<Runtime>();
    match (action, runtime) {
        (MenuAction::OpenSettings, _) => settings::open(app),
        (MenuAction::Quit, _) => {
            tracing::info!("quit requested from the tray");
            app.exit(0);
        }
        (MenuAction::Pause, Some(runtime)) => runtime.toggle_pause(app),
        (MenuAction::ReloadConfig, Some(runtime)) => {
            tracing::info!("config reload requested from the tray");
            runtime.apply_config(app);
        }
        (MenuAction::SimulateHookLoss, Some(runtime)) => runtime.simulate_hook_loss(),
        (MenuAction::StallCore, Some(runtime)) => runtime.stall_core(),
        (action, None) => tracing::warn!(?action, "runtime not ready; ignoring tray action"),
    }
}

fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        // `code: None` means the last window closed. A tray app keeps running;
        // only an explicit `exit` (Quit in the tray) carries a code.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        RunEvent::Exit => {
            if let Some(runtime) = app.try_state::<Runtime>() {
                runtime.stop();
            }
        }
        _ => {}
    }
}
