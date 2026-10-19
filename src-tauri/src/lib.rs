//! ClickHub library. `main.rs` only calls [`run`].
//!
//! Module map, thread model and boundaries: `docs/architecture.md` and
//! `docs/coding-guidelines.md`. Modules arrive with the phase that needs them.

mod actions;
mod app;
pub mod config;
pub mod core;
mod hook;
mod overlay;
mod platform;
mod settings;
pub mod tray;

pub use app::run;
