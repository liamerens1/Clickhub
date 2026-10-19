//! Low-level mouse and keyboard hooks. The only place with Win32 hook code.
//!
//! - [`HookThread`]: installs `WH_MOUSE_LL` / `WH_KEYBOARD_LL` on a dedicated thread with its
//!   own message loop.
//! - The callbacks only match the trigger and post a [`HookEvent`] (hard rule 5).
//! - [`Watchdog`]: re-installs the hooks when Windows silently removes them.
//! - [`SHARED`]: atomics the rest of the app uses to configure and observe the callback.
//!
//! Never touches UI, Tauri, IPC, config or files. One of the two modules allowed `unsafe`.
#![allow(unsafe_code)]

mod callbacks;
mod error;
mod event;
mod shared;
mod thread;
mod watchdog;

pub use error::HookError;
pub use event::{ButtonAction, HookEvent, HookEventKind};
pub use shared::SHARED;
pub use thread::HookThread;
pub use watchdog::Watchdog;
