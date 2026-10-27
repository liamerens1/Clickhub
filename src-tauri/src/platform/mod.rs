//! Thin, safe wrappers over Win32 that are not hook code: clocks, input state and injection,
//! the foreground app, full-screen detection, shell open, window styles and the per-user Run
//! key. No business logic and no Tauri. One of the two modules allowed to use `unsafe`.
#![allow(unsafe_code)]

mod foreground;
mod fullscreen;
mod input;
mod keys;
mod run_key;
mod screen;
mod shell;
mod time;
mod window_icon;
mod window_style;

pub use foreground::foreground_app;
pub use fullscreen::foreground_is_fullscreen;
pub use input::send_shortcut;
pub use keys::held_modifiers;
pub use run_key::{remove_run_entry, run_entry_active, set_run_entry};
pub use screen::{LumaGrid, sample_luma};
pub use shell::{init_shell_thread, shell_open};
pub use time::{last_input_tick, qpc_now, qpc_to_micros, tick_count, tick_count_64};
pub use window_icon::set_large_window_icon;
pub use window_style::{keep_on_top, make_overlay_window, place_overlay};
