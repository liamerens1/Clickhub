//! Tray icon and menu. Builds the menu and dispatches; holds no logic.

mod icon;
mod menu_action;

pub(crate) use icon::build;
pub use menu_action::MenuAction;
