//! Thin, safe wrappers over Win32 that are not hook code: clocks, input state, the foreground
//! app. No business logic and no Tauri. One of the two modules allowed to use `unsafe`.
#![allow(unsafe_code)]

mod foreground;
mod keys;
mod time;

pub use foreground::foreground_app;
pub use keys::held_modifiers;
pub use time::{last_input_tick, qpc_now, qpc_to_micros, tick_count, tick_count_64};
