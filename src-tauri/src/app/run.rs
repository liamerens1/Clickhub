use tauri::{App, AppHandle, RunEvent};

use crate::{settings, tray};

/// Builds and runs the app. Blocks until the user quits from the tray.
pub fn run() {
    super::logging::init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "ClickHub starting");

    let app = tauri::Builder::default()
        // Registered first: a second launch hands over to this instance, then exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tracing::info!("second instance launched; opening settings here instead");
            settings::open(app);
        }))
        .setup(setup)
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(error) => {
            tracing::error!(%error, "failed to build the Tauri app");
            std::process::exit(1);
        }
    };

    app.run(on_run_event);
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    tray::build(app.handle())?;
    // Starts hidden: no window exists until the user opens settings.
    tracing::info!("tray ready");
    Ok(())
}

fn on_run_event(_app: &AppHandle, event: RunEvent) {
    match event {
        // `code: None` means the last window closed. A tray app keeps running;
        // only an explicit `exit` (Quit in the tray) carries a code.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        RunEvent::Exit => tracing::info!("ClickHub exited"),
        _ => {}
    }
}
