//! ClickHub library. `main.rs` only calls [`run`].
//!
//! Module map, thread model and boundaries: `docs/architecture.md` and
//! `docs/coding-guidelines.md`. Modules arrive with the phase that needs them.

mod app;
pub mod config;
mod settings;
pub mod tray;

pub use app::run;
