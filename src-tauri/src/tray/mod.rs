//! Tray icon and menu. Builds the menu, maps clicks to [`MenuAction`]s and shows status.
//! Holds no logic: the caller decides what each action does.

mod icon;
mod menu_action;

pub(crate) use icon::{build, show_status};
pub use menu_action::MenuAction;
