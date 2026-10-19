//! Thin, safe wrappers over Win32 that are not hook code: clocks, input state. No business
//! logic and no Tauri. One of the two modules allowed to use `unsafe`.
#![allow(unsafe_code)]

mod keys;
mod time;

pub use keys::held_modifiers;
pub use time::{last_input_tick, qpc_now, qpc_to_micros, tick_count, tick_count_64};
