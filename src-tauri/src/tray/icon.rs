use tauri::menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use super::MenuAction;

const TRAY_ID: &str = "main";

/// Handles kept so the menu and tooltip can change after the tray is built.
struct TrayHandles {
    pause: MenuItem<Wry>,
}

/// Creates the tray icon and its menu. Clicks are mapped to [`MenuAction`]s and handed to
/// `on_action`; a left-click on the icon counts as [`MenuAction::OpenSettings`].
///
/// # Errors
///
/// Fails if Tauri cannot create the menu or the tray icon.
pub fn build(
    app: &AppHandle,
    show_debug_items: bool,
    on_action: fn(&AppHandle, MenuAction),
) -> tauri::Result<()> {
    let mut items: Vec<MenuItem<Wry>> = Vec::new();
    let mut pause = None;
    for action in MenuAction::ALL {
        if action == MenuAction::Quit || (action.is_debug() && !show_debug_items) {
            continue;
        }
        let item = MenuItem::with_id(app, action.id(), action.label(), true, None::<&str>)?;
        if action == MenuAction::Pause {
            pause = Some(item.clone());
        }
        items.push(item);
    }
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(
        app,
        MenuAction::Quit.id(),
        MenuAction::Quit.label(),
        true,
        None::<&str>,
    )?;
    let mut entries: Vec<&dyn IsMenuItem<Wry>> = items
        .iter()
        .map(|item| item as &dyn IsMenuItem<Wry>)
        .collect();
    entries.push(&separator);
    entries.push(&quit);
    let menu = Menu::with_items(app, &entries)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("ClickHub")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            if let Some(action) = MenuAction::from_id(event.id().as_ref()) {
                on_action(app, action);
            } else {
                tracing::warn!(id = event.id().as_ref(), "unknown tray menu id");
            }
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                on_action(tray.app_handle(), MenuAction::OpenSettings);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    } else {
        tracing::warn!("no default window icon; the tray icon will be blank");
    }
    builder.build(app)?;

    if let Some(pause) = pause {
        app.manage(TrayHandles { pause });
    }
    Ok(())
}

/// Updates the Pause item and the tooltip to reflect the current state.
pub fn show_status(app: &AppHandle, paused: bool, problem: Option<&str>) {
    if let Some(handles) = app.try_state::<TrayHandles>()
        && let Err(error) = handles.pause.set_text(MenuAction::pause_label(paused))
    {
        tracing::warn!(%error, "could not update the Pause menu item");
    }

    let tooltip = match (paused, problem) {
        (_, Some(problem)) => format!("ClickHub: {problem}"),
        (true, None) => "ClickHub (paused)".to_owned(),
        (false, None) => "ClickHub".to_owned(),
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID)
        && let Err(error) = tray.set_tooltip(Some(tooltip))
    {
        tracing::warn!(%error, "could not update the tray tooltip");
    }
}
