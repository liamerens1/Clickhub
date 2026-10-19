//! Runs slot actions (send a shortcut, launch, open a URL) on a dedicated thread, so a slow
//! launch never stalls the pipeline. Uses `platform/` for every OS call.

mod runner;

pub use runner::ActionRunner;
