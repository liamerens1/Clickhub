use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Wry};

use super::MenuAction;
use crate::settings;

const TRAY_ID: &str = "main";

/// Creates the tray icon and its menu. Tauri keeps the icon alive until exit.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let item = |action: MenuAction| -> tauri::Result<MenuItem<Wry>> {
        MenuItem::with_id(
            app,
            action.id(),
            action.label(),
            action.enabled(),
            None::<&str>,
        )
    };
    let open_settings = item(MenuAction::OpenSettings)?;
    let pause = item(MenuAction::Pause)?;
    let quit = item(MenuAction::Quit)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open_settings, &pause, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("ClickHub")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| on_tray_icon_event(tray, &event));
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    } else {
        tracing::warn!("no default window icon; the tray icon will be blank");
    }
    builder.build(app)?;
    Ok(())
}

fn on_menu_event(app: &AppHandle, id: &str) {
    match MenuAction::from_id(id) {
        Some(MenuAction::OpenSettings) => settings::open(app),
        // Disabled in the menu until Phase 1; nothing to do if it ever fires.
        Some(MenuAction::Pause) => {}
        Some(MenuAction::Quit) => {
            tracing::info!("quit requested from the tray");
            app.exit(0);
        }
        None => tracing::warn!(id, "unknown tray menu id"),
    }
}

fn on_tray_icon_event(tray: &TrayIcon, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        settings::open(tray.app_handle());
    }
}
