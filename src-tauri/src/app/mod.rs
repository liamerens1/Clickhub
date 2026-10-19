//! Startup, the trigger pipeline, the runtime that owns background threads, and shutdown.

mod fullscreen_watch;
mod logging;
mod pipeline;
mod run;
mod runtime;

pub use run::run;
pub(crate) use runtime::Runtime;
