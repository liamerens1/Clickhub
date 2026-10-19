//! Startup, the trigger pipeline, the runtime that owns background threads, and shutdown.

mod logging;
mod pipeline;
mod run;
mod runtime;

pub use run::run;
